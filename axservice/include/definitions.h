#ifndef _DEFINITIONS_H
#define _DEFINITIONS_H

#define AX_VIRQ 13 /* nimbos use 13, so... */

// #define ARCEOS_VIRQ_SIG_NUM 44 /* nimbos use 44, so... */

// #define ARCEOS_VDEV_IOCTL_MAGIC                         0xF1 /* ('A' ^ 'r' + 'c' ^ 'e'), with no collision in ioctl-number.txt */
// #define ARCEOS_VDEV_IOCTL_REGISTER_VIRQ_HANDLER         _IO(ARCEOS_VDEV_IOCTL_MAGIC, 0)
// #define ARCEOS_VDEV_IOCTL_UNREGISTER_VIRQ_HANDLER       _IO(ARCEOS_VDEV_IOCTL_MAGIC, 1)
// #define ARCEOS_VDEV_IOCTL_INVOKE_HYPERCALL              _IO(ARCEOS_VDEV_IOCTL_MAGIC, 2)

#define ARCEOS_SYSCALL_DATA_BUF_PADDR (0x67eff000)
#define ARCEOS_SYSCALL_DATA_BUF_SIZE (0x00100000)
#define ARCEOS_SYSCALL_QUEUE_BUF_PADDR                                         \
    (ARCEOS_SYSCALL_DATA_BUF_PADDR + ARCEOS_SYSCALL_DATA_BUF_SIZE)
#define ARCEOS_SYSCALL_QUEUE_BUF_SIZE (0x00001000)

#endif