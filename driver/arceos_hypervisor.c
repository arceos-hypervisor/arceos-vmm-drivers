#include "arceos_hypercall.h"
#include "arceos_virq.h"
#include "arceos_vdev.h"
#include "definitions.h"

#include <linux/module.h>
#include <linux/interrupt.h>
#include <linux/kernel.h>

MODULE_AUTHOR("arceos");
MODULE_LICENSE("GPL");

/**
 * 仿照这部分驱动的几项工作：
 * 1. Hypervisor --virq--> Driver --signal--> Process
 *   a. Process通过ioctl注册自身为virq_processor
 * 2. Process --ioctl--> Driver --hypercall--> Hypervisor
 * 3. mmap到物理地址空间
 */

#define LOGHEAD "[arceos-hypervisor] "

static const struct file_operations arceos_vdev_fops = {
    .owner = THIS_MODULE,
    .open = arceos_vdev_open,
    .release = arceos_vdev_close,
    .unlocked_ioctl = arceos_vdev_ioctl,
    .compat_ioctl = arceos_vdev_ioctl,
    .mmap = arceos_vdev_mmap,
};

static struct miscdevice arceos_vdev = {
    .minor = MISC_DYNAMIC_MINOR,
    .name = "arceos_vdev",
    .fops = &arceos_vdev_fops,
};

static int __init arceos_hypervisor_register(void) {
    int err;
    
    pr_info(LOGHEAD "Initializing...\n");

    pr_info(LOGHEAD "Registering arceos_vdev...\n");
    err = misc_register(&arceos_vdev);
    if (err) {
        pr_err(LOGHEAD "Cannot register arceos_vdev! err: %d\n", err);
        goto end;
    }

    pr_info(LOGHEAD "Registering irq...\n");
    err = request_irq(ARCEOS_VIRQ, arceos_virq_handler, IRQF_SHARED, "arceos_hypervisor", &arceos_vdev);
    if (err) {
        pr_err(LOGHEAD "Cannot register irq! err: %d\n", err);
        goto err_unregister_vdev;
    }

    /**
     * Debug:
     *  arceos_hypercall(0x42, 0x77776666, 0xdeadbeef);
     */

    goto end;

err_unregister_vdev:
    misc_deregister(&arceos_vdev);

end:
    return err;
}

static void __exit arceos_hypervisor_unregister(void) {
    free_irq(ARCEOS_VIRQ, NULL);
    misc_deregister(&arceos_vdev);
    pr_info("[arceos-hypervisor] Exit.");
}

module_init(arceos_hypervisor_register);
module_exit(arceos_hypervisor_unregister);
