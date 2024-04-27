#ifndef _DEFINITIONS_H
#define _DEFINITIONS_H 1

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

#define ARCEOS_SYSCALL_DATA_BUF_PADDR                   (0x67eff000)
#define ARCEOS_SYSCALL_DATA_BUF_SIZE                    (0x00100000)
#define ARCEOS_SYSCALL_QUEUE_BUF_PADDR                  (ARCEOS_SYSCALL_DATA_BUF_PADDR + ARCEOS_SYSCALL_DATA_BUF_SIZE)
#define ARCEOS_SYSCALL_QUEUE_BUF_SIZE                   (0x00001000)

#endif

/*
static void outb(unsigned char __value, unsigned short int __port) {
  __asm__ __volatile__ ("outb %b0,%w1": :"a" (__value), "Nd" (__port));
}

void write_com(unsigned short int port, const char *c) {
    for (int ptr = 0; c[ptr] != '\0'; ptr++) {
        outb(c[ptr], port);
    }
}

#define COM1 (0x3f8)
*/