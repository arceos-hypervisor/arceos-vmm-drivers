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
    struct arceos_vdev_hypercall_args hypercall_args;

    if (fd < 0) {
        fprintf(stderr, "Failed to open %s, err code: %d\n", VDEV_PATH, fd);
        goto exit;
    }
    printf("Opened %s.\n", VDEV_PATH);

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

    hypercall_args.id = 0x53686477;     /* "Shdw" */
    hypercall_args.arg0 = 0x70726373;   /* "prcs" */
    hypercall_args.arg1 = 0x52647921;   /* "Rdy!" */

    printf("Invoking hypercall...\n");
    ioctl(fd, ARCEOS_VDEV_IOCTL_INVOKE_HYPERCALL, &hypercall_args);
    printf("Hypercall invoked.\n");

    printf("Processing existing requests...\n");
    poll_requests();

exit:
    return fd;
}

void arceos_vdev_close(int fd) {
    ioctl(fd, ARCEOS_VDEV_IOCTL_UNREGISTER_VIRQ_HANDLER, 0);

    close(fd);
}
