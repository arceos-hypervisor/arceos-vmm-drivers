#ifndef _DEFINITIONS_H
#define _DEFINITIONS_H 1

#include <stdint.h>

#define ARCEOS_VIRQ 13 /* nimbos use 13, so... */

#define ARCEOS_VIRQ_SIG_NUM 44 /* nimbos use 44, so... */

#define ARCEOS_VDEV_IOCTL_MAGIC                         0xF1 /* ('A' ^ 'r' + 'c' ^ 'e'), with no collision in ioctl-number.txt */
#define ARCEOS_VDEV_IOCTL_REGISTER_VIRQ_HANDLER         _IO(ARCEOS_VDEV_IOCTL_MAGIC, 0)
#define ARCEOS_VDEV_IOCTL_UNREGISTER_VIRQ_HANDLER       _IO(ARCEOS_VDEV_IOCTL_MAGIC, 1)
#define ARCEOS_VDEV_IOCTL_INVOKE_HYPERCALL              _IO(ARCEOS_VDEV_IOCTL_MAGIC, 2)

struct arceos_vdev_hypercall_args {
    uint32_t id;
    uint32_t return_value;
    uint32_t arg0;
    uint32_t arg1;
    uint32_t arg2;
    uint32_t arg3;
    uint32_t arg4;
    uint32_t reserved;
};

#define ARCEOS_HYPERCALL_SHADOW_PROCESS_READY           (0x53686477) /* "Shdw" */
#define ARCEOS_HYPERCALL_SHADOW_PROCESS_READY_ARG0      (0x70726373) /* "prcs" */
#define ARCEOS_HYPERCALL_SHADOW_PROCESS_READY_ARG1      (0x52647921) /* "Rdy!" */
#define ARCEOS_HYPERCALL_EPT_MAPPING_REQUEST            (0x454d6170) /* "EMap" */

#define ARCEOS_SYSCALL_DATA_BUF_PADDR                   (0x67eff000)
#define ARCEOS_SYSCALL_DATA_BUF_SIZE                    (0x00100000)
#define ARCEOS_SYSCALL_QUEUE_BUF_PADDR                  (ARCEOS_SYSCALL_DATA_BUF_PADDR + ARCEOS_SYSCALL_DATA_BUF_SIZE)
#define ARCEOS_SYSCALL_QUEUE_BUF_SIZE                   (0x00001000)
#define ARCEOS_SYSCALL_QUEUE_BUF_MAGIC                  (0x4643537f) /* "\x7fSCF" */

#endif
