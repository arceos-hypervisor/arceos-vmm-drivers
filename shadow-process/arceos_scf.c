#include "arceos_scf.h"

#include "arceos_vdev.h"

#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <unistd.h>

#define ALIGN_UP(addr, align) ((addr + align - 1) & ~(align - 1))

static void *syscall_data_buf_base;
static void *syscall_queue_buf_base;
static int vdev_fd;

struct syscall_queue_buffer g_syscall_queue_buffer;

struct read_write_args {
    int fd;
    uint64_t buf_offset;
    uint64_t len;
};

struct syscall_args {
    uint64_t args[6];
};

void dump_memory(void *addr, size_t size) {
    // dump memory, 16 bytes per line
    unsigned char *p = addr;
    for (size_t i = 0; i < size; i++) {
        if (i % 16 == 0 && i != 0) {
            printf("\n%p: ", p);
        }
        printf("%02x ", p[i]);
    }
}

static void *read_thread_fn(void *arg) {
    struct read_write_args *args;
    struct syscall_queue_buffer *scf_buf = get_syscall_queue_buffer();
    uint16_t desc_index = (uint16_t)(long)arg;
    struct scf_descriptor *desc = get_syscall_request_from_index(scf_buf, desc_index);

    if (!desc) {
        return NULL;
    }

    args = offset_to_ptr(desc->args);
    char *buf = offset_to_ptr(args->buf_offset);
    int ret = read(args->fd, buf, args->len);
    // assert(ret == args->len);
    push_syscall_response(scf_buf, desc_index, ret);
    return NULL;
}

// TODO: move this to vdev driver
#define SHADOW_PROCESS_GPA_BASE 0x60000000
uint64_t alloc_shadow_process_gpa(uint32_t size) {
    static uint64_t gpa = SHADOW_PROCESS_GPA_BASE;
    uint64_t ret = gpa;

    // round size up to 4KB
    size = (size + 0xfff) & ~0xfff;

    gpa += size;
    return ret;
}

#define VDISK_BLOCK_SIZE 512
#define VDISK_BLOCK_SIZE_SHIFT 9
#define MAX_VDISK 4
#define MAX_VDISK_SIZE (16 * 1024 * 1024) // 16MB
#define MAX_VDISK_BLOCKS (MAX_VDISK_SIZE / VDISK_BLOCK_SIZE)

int vdisk_fd[MAX_VDISK];

uint64_t handle_vdisk_request(struct scf_descriptor *desc) {
    struct syscall_args *args = offset_to_ptr(desc->args);

    switch (desc->opcode) {
    case IPC_OP_SPECIAL_OPEN_VDISK: {
        int id = (int)args->args[0];
        if (id < 0 || id >= MAX_VDISK) {
            return -EINVAL;
        } else if (vdisk_fd[id] > 0) {
            return -EBUSY;
        }

        char path[64];
        snprintf(path, sizeof(path), "vdisk-%4x.img", id);
        if ((vdisk_fd[id] = open(path, O_RDWR)) < 0) {
            return -vdisk_fd[id];
        }

        return 0;
    }
    case IPC_OP_SPECIAL_READ_VDISK_BLOCK: {
        int id = (int)args->args[0];
        uint64_t block = args->args[1];
        uint64_t buf_offset = args->args[2];
        if (id < 0 || id >= MAX_VDISK || vdisk_fd[id] < 0) {
            return -EINVAL;
        }

        off_t offset = block << VDISK_BLOCK_SIZE_SHIFT;
        if (lseek(vdisk_fd[id], offset, SEEK_SET) < 0) {
            return -errno;
        }

        char *buf = offset_to_ptr(buf_offset);
        ssize_t ret = read(vdisk_fd[id], buf, VDISK_BLOCK_SIZE);
        if (ret < 0) {
            return -errno;
        }

        return ret;
    }
    case IPC_OP_SPECIAL_WRITE_VDISK_BLOCK: {
        int id = (int)args->args[0];
        uint64_t block = args->args[1];
        uint64_t buf_offset = args->args[2];
        if (id < 0 || id >= MAX_VDISK || vdisk_fd[id] < 0) {
            return -EINVAL;
        }

        off_t offset = block << VDISK_BLOCK_SIZE_SHIFT;
        if (lseek(vdisk_fd[id], offset, SEEK_SET) < 0) {
            return -errno;
        }

        char *buf = offset_to_ptr(buf_offset);
        ssize_t ret = write(vdisk_fd[id], buf, VDISK_BLOCK_SIZE);
        if (ret < 0) {
            return -errno;
        }

        return ret;
    }
    default:
        return -EINVAL;
    }
} 

void poll_requests(void) {
    uint16_t desc_index;
    struct scf_descriptor desc;
    struct syscall_queue_buffer *scf_buf = get_syscall_queue_buffer();
    pthread_t thread; // FIXME: use global threads pool
    int count = 0;

    // printf("polling requests...\n");

    while (!pop_syscall_request(scf_buf, &desc_index, &desc)) {
        // printf("syscall: desc_index=%d, opcode=%d, args=0x%lx\n", desc_index, desc.opcode, desc.args);
        switch (desc.opcode) {
        case IPC_OP_READ: {
            // todo: ~~correct this~~ if something goes wrong, revert it to async version
            // pthread_create(&thread, NULL, read_thread_fn, (void *)(long)desc_index);
            (void) thread; (void) read_thread_fn;
            struct syscall_args *args = offset_to_ptr(desc.args);
            int fd = args->args[0];
            char *buf = (char *)(args->args[1]);
            int len = args->args[2];
            int ret = 0;
            // printf("# reading %d bytes read to %p\n", ret, buf);
            ret = read(fd, buf, len);
            // printf("# read, buf now: %s]\n", buf);
            // assert(ret == len); of course not
            push_syscall_response(scf_buf, desc_index, ret);
            break;
        }
        case IPC_OP_WRITE: {
            struct syscall_args *args = offset_to_ptr(desc.args);
            int fd = args->args[0];
            char *buf = (char *)(args->args[1]);
            int len = args->args[2];
            int ret = write(fd, buf, len);
            assert(ret == len);
            push_syscall_response(scf_buf, desc_index, ret);
            break;
        }
        case IPC_OP_OPEN: {
            struct syscall_args *args = offset_to_ptr(desc.args);
            char *path = (char *)(args->args[0]);
            int flags = (int)args->args[1];
            int mode = (int)args->args[2];
            int ret = open(path, flags, mode);
            push_syscall_response(scf_buf, desc_index, ret);
            break;
        }
        case IPC_OP_CLOSE: {
            struct syscall_args *args = offset_to_ptr(desc.args);
            int fd = (int)args->args[0];
            int ret = close(fd);
            push_syscall_response(scf_buf, desc_index, ret);
            break;
        }
        case IPC_OP_SPECIAL_MUST_MMAP: {
            // hypervisor tells us to mmap a region, we should do it
            struct syscall_args *args = offset_to_ptr(desc.args);
            uint64_t hpa = args->args[0];
            uint64_t va = args->args[1];
            uint32_t size = (uint32_t)(args->args[2]);

            // allocate shadow process GPA
            // TODO: driver should do this for us, because there might be multiple shadow processes
            uint64_t gpa = alloc_shadow_process_gpa(size);

            printf("shadow mapping: hpa=0x%lx, gpa=0x%lx, va=0x%lx, size=0x%x\n", hpa, gpa, va, size);

            // send hypercall to request EPT mapping
            // printf("arceos_vdev_hypercall_ept_mapping_request: hpa=0x%lx, gpa=0x%lx, size=0x%x\n", hpa, gpa, size);
            int ret = arceos_vdev_hypercall_ept_mapping_request(vdev_fd, hpa, gpa, size);
            if (ret) {
                printf("arceos_vdev_hypercall_ept_mapping_request failed: %d\n", ret);
                push_syscall_response(scf_buf, desc_index, ret);
                break;
            }

            // map the shadow process GPA to the virtual address
            mmap((void *)va, size, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_FIXED, vdev_fd, gpa);

            push_syscall_response(scf_buf, desc_index, 0);
            break;
        }
        case IPC_OP_WRITEV: {
            struct syscall_args *args = offset_to_ptr(desc.args);
            int fd = args->args[0];
            char *buf = (char *)(args->args[1]);
            int len = args->args[2];
            int ret = write(fd, buf, len);
            assert(ret == len);
            push_syscall_response(scf_buf, desc_index, ret);
            break;
        }
        case IPC_OP_SPECIAL_OPEN_VDISK:
        case IPC_OP_SPECIAL_READ_VDISK_BLOCK:
        case IPC_OP_SPECIAL_WRITE_VDISK_BLOCK: {
            int ret = handle_vdisk_request(&desc);
            push_syscall_response(scf_buf, desc_index, ret);
            break;
        }
        default:
            break;
        }

        count++;
    }

    // printf("%d requests processed\n", count);
}

void arceos_vdev_signal_handler(int sig) {
    if (sig == ARCEOS_VIRQ_SIG_NUM) {
        poll_requests();
    }
}

int arceos_setup_syscall_buffers(int fd) {
    vdev_fd = fd;

    syscall_data_buf_base = mmap(0, ARCEOS_SYSCALL_DATA_BUF_SIZE, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_POPULATE, fd, 0);
    if (syscall_data_buf_base == MAP_FAILED) {
        return -ENOMEM;
    }

    // this offset here is just a flag to tell the kernel that we want to map the syscall queue buffer
    syscall_queue_buf_base = mmap(0, ARCEOS_SYSCALL_QUEUE_BUF_SIZE, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_POPULATE, fd, 0x1000);
    if (syscall_queue_buf_base == MAP_FAILED) {
        return -ENOMEM;
    }
    // printf("%p %p\n", syscall_data_buf_base, syscall_queue_buf_base);

    struct syscall_queue_buffer_metadata *meta = syscall_queue_buf_base;
    struct scf_descriptor *desc;
    uint16_t *req_ring, *rsp_ring;
    uint16_t capacity = meta->capacity;

    // printf("%x %d %d %d %d\n", meta->magic, meta->capacity, meta->lock, meta->req_index, meta->rsp_index);

    if (meta->magic != ARCEOS_SYSCALL_QUEUE_BUF_MAGIC) {
        return -EINVAL;
    }
    if (!capacity || (capacity & (capacity - 1)) != 0) {
        return -EINVAL;
    }

    desc = (void *)meta + ALIGN_UP(sizeof(struct syscall_queue_buffer_metadata), 8);
    req_ring = (void *)desc + capacity * sizeof(struct scf_descriptor);
    rsp_ring = (void *)req_ring + capacity * sizeof(uint16_t);

    g_syscall_queue_buffer = (struct syscall_queue_buffer){
        .capacity_mask = capacity - 1,
        .req_index_last = 0,
        .rsp_index_shadow = meta->rsp_index,
        .meta = meta,
        .desc = desc,
        .req_ring = req_ring,
        .rsp_ring = rsp_ring,
    };

    return 0;
}

inline void *offset_to_ptr(uint64_t offset) {
    return syscall_data_buf_base + offset;
}

inline struct syscall_queue_buffer *get_syscall_queue_buffer() {
    return &g_syscall_queue_buffer;
}

static inline int has_request(struct syscall_queue_buffer *buf) {
    return buf->req_index_last != buf->meta->req_index;
}

struct scf_descriptor *get_syscall_request_from_index(struct syscall_queue_buffer *buf, uint16_t index) {
    if (index > buf->capacity_mask) {
        return NULL;
    }
    return &buf->desc[index];
}

int pop_syscall_request(struct syscall_queue_buffer *buf, uint16_t *out_index, struct scf_descriptor *out_desc) {
    int err;
    spin_lock(&buf->meta->lock);
    // printf("pop_syscall_request %d %d\n", buf->req_index_last, buf->meta->req_index);

    if (has_request(buf)) {
        __sync_synchronize();
        uint16_t idx = buf->req_ring[buf->req_index_last & buf->capacity_mask];
        // printf("idx=%d\n", idx);
        if (idx > buf->capacity_mask) {
            // printf("idx > capacity_mask\n");
            err = -EINVAL;
            goto end;
        }
        *out_index = idx;
        *out_desc = buf->desc[idx];
        buf->req_index_last += 1;
        err = 0;
    } else {
        // printf("no request\n");
        err = -EBUSY;
    }

end:
    spin_unlock(&buf->meta->lock);
    return err;
}

int push_syscall_response(struct syscall_queue_buffer *buf, uint16_t index, uint64_t ret_val) {
    int err;
    spin_lock(&buf->meta->lock);

    if (index > buf->capacity_mask) {
        err = -EINVAL;
        goto end;
    }

    buf->desc[index].ret_val = ret_val;
    buf->rsp_ring[buf->rsp_index_shadow & buf->capacity_mask] = index;
    buf->rsp_index_shadow++;
    __sync_synchronize();
    buf->meta->rsp_index = buf->rsp_index_shadow;
    err = 0;

end:
    spin_unlock(&buf->meta->lock);
    return err;
}