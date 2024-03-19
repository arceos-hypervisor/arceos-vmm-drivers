#include "arceos_vdev.h"

#include "arceos_scf.h"

#include <assert.h>
#include <fcntl.h>
#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <sys/ioctl.h>
#include <unistd.h>

int arceos_vdev_open() {
    int err;
    int fd = open(VDEV_PATH, O_RDWR);

    if (fd < 0) {
        fprintf(stderr, "Failed to open %s, err code: %d\n", VDEV_PATH, fd);
        goto exit;
    }
    printf("Opened %s (%d).\n", VDEV_PATH, fd);

    err = ioctl(fd, ARCEOS_VDEV_IOCTL_REGISTER_VIRQ_HANDLER, 0);
    if (err < 0) {
        fprintf(stderr, "Failed to register virq handler, err code: %d\n", err);
        goto exit;
    }
    printf("Virq handler registered.\n");

    err = arceos_setup_syscall_buffers(fd);
    if (err < 0) {
        fprintf(stderr, "Failed to setup syscall buffers, err code: %d\n", err);
        goto exit;
    }
    printf("Syscall buffers setup done.\n");

    signal(ARCEOS_VIRQ_SIG_NUM, arceos_vdev_signal_handler);
    printf("Signal handler registered.\n");

    printf("Sending shadow process ready hypercall...\n");
    arceos_vdev_hypercall_shadow_process_ready(fd);
    printf("Shadow process ready hypercall sent.\n");
    
    printf("Processing existing requests...\n");
    poll_requests();

exit:
    return fd;
}

void arceos_vdev_close(int fd) {
    ioctl(fd, ARCEOS_VDEV_IOCTL_UNREGISTER_VIRQ_HANDLER, 0);

    close(fd);
}
/**
 * Sends a hypercall via the ArceOS virtual device.
 *
 * @param fd The file descriptor of the ArceOS virtual device.
 * @param id The identifier of the hypercall.
 * @param arg0 The first argument of the hypercall.
 * @param arg1 The second argument of the hypercall.
 * @param arg2 The third argument of the hypercall.
 * @param arg3 The fourth argument of the hypercall.
 * @param arg4 The fifth argument of the hypercall.
 * @param ret_val Pointer to store the return value of the hypercall.
 * @return 0 on success, or a negative error code on failure.
 */
int arceos_vdev_send_hypercall(int fd, uint32_t id, uint32_t arg0, uint32_t arg1, uint32_t arg2, uint32_t arg3, uint32_t arg4, uint32_t *ret_val) {
    int err;
    struct arceos_vdev_hypercall_args hypercall_args;
    hypercall_args.id = id;
    hypercall_args.arg0 = arg0;
    hypercall_args.arg1 = arg1;
    hypercall_args.arg2 = arg2;
    hypercall_args.arg3 = arg3;
    hypercall_args.arg4 = arg4;
    hypercall_args.reserved = hypercall_args.return_value = 0;

    err = ioctl(fd, ARCEOS_VDEV_IOCTL_INVOKE_HYPERCALL, &hypercall_args);
    *ret_val = hypercall_args.return_value;
    return err;
}

/**
 * Sends a hypercall to indicate that the shadow process is ready.
 *
 * @param fd The file descriptor of the device.
 * @return The return value of the hypercall.
 */
int arceos_vdev_hypercall_shadow_process_ready(int fd) {
    uint32_t ret_val;

    return arceos_vdev_send_hypercall(
        fd, 
        ARCEOS_HYPERCALL_SHADOW_PROCESS_READY, 
        ARCEOS_HYPERCALL_SHADOW_PROCESS_READY_ARG0, 
        ARCEOS_HYPERCALL_SHADOW_PROCESS_READY_ARG1, 
        0,
        0,
        0,
        &ret_val
    );
}
/**
 * Sends a hypercall to request EPT mapping between host physical address (HPA) and guest physical address (GPA).
 *
 * @param fd The file descriptor to send the hypercall to.
 * @param hpa The host physical address (HPA) to be mapped.
 * @param gpa The guest physical address (GPA) to be mapped.
 * @param size The size of the mapping.
 * @return The return value of the hypercall.
 */

int arceos_vdev_hypercall_ept_mapping_request(int fd, uint64_t hpa, uint64_t gpa, uint32_t size) {
    uint32_t ret_val;
    
    return arceos_vdev_send_hypercall(
        fd, 
        ARCEOS_HYPERCALL_EPT_MAPPING_REQUEST, 
        (uint32_t)(hpa >> 32),
        (uint32_t)hpa,
        (uint32_t)(gpa >> 32),
        (uint32_t)gpa,
        size,
        &ret_val
    );
}
