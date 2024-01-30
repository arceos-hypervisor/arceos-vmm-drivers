#ifndef _ARCEOS_SCF_H
#define _ARCEOS_SCF_H 1

#include "definitions.h"

#include <sched.h>
#include <stdint.h>
#include <assert.h>
#include <stddef.h>

typedef uint8_t spin_lock_t;

inline void spin_lock(spin_lock_t *lock) {
    spin_lock_t expected = 0;
    while (!__atomic_compare_exchange_n(lock, &expected, 1, 0, __ATOMIC_ACQUIRE, __ATOMIC_RELAXED)) {
        expected = 0;
        do {
            sched_yield();
        } while (__atomic_load_n(lock, __ATOMIC_RELAXED) != 0);
    }
}

inline void spin_unlock(spin_lock_t *lock) {
    __atomic_store_n(lock, 0, __ATOMIC_RELEASE);
}

void arceos_vdev_signal_handler(int sig);

enum scf_opcode {
    IPC_OP_NOP = 0,
    IPC_OP_READ = 1,
    IPC_OP_WRITE = 2,
    IPC_OP_OPEN = 3,
    IPC_OP_CLOSE = 4,
    IPC_OP_UNKNOWN = 0xff,
};

struct syscall_queue_buffer_metadata {
    uint32_t magic;
    spin_lock_t lock;
    uint16_t capacity;
    uint16_t req_index;
    uint16_t rsp_index;
};

struct scf_descriptor {
    uint8_t valid;
    uint8_t opcode;
    uint64_t args;
    uint64_t ret_val;
};

struct syscall_queue_buffer {
    uint16_t capacity_mask;
    uint16_t req_index_last;
    uint16_t rsp_index_shadow;
    struct syscall_queue_buffer_metadata *meta;
    struct scf_descriptor *desc;
    uint16_t *req_ring;
    uint16_t *rsp_ring;
};

static_assert(sizeof(struct syscall_queue_buffer_metadata) == 0xc);
static_assert(sizeof(struct scf_descriptor) == 0x18);

void poll_requests(void);
int arceos_setup_syscall_buffers(int nimbos_fd);
void *offset_to_ptr(uint64_t offset);
struct syscall_queue_buffer *get_syscall_queue_buffer();
struct scf_descriptor *get_syscall_request_from_index(struct syscall_queue_buffer *buf, uint16_t index);
int pop_syscall_request(struct syscall_queue_buffer *buf, uint16_t *out_index, struct scf_descriptor *out_desc);
int push_syscall_response(struct syscall_queue_buffer *buf, uint16_t index, uint64_t ret_val);

#endif
