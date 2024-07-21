#include <linux/device.h>
#include <linux/module.h>
#include <linux/slab.h>
#include <linux/uio_driver.h>

#include "include/definitions.h"
#include "include/util.h"

#define AX_VIRQ 13 /* nimbos use 13, so... */

static struct uio_info *info;
static struct device *dev;
static int irq = 1;

static void axservice_release(struct device *dev) {
    INFO(KERN_INFO "releasing axservice device\n");
}

// static irqreturn_t my_handler(int irq, struct uio_info *dev_info)
// {
// 	static int count = 0;
// 	printk(KERN_INFO "In UIO handler, count=%d\n", ++count);
// 	return IRQ_HANDLED;
// }

static int __init axservice_init(void) {
    INFO(KERN_INFO "Initializing axservice device\n");

    dev = kzalloc(sizeof(struct device), GFP_KERNEL);
    dev_set_name(dev, "axservice");
    dev->release = axservice_release;

    if (device_register(dev) < 0) {
        ERROR(KERN_INFO "Failing to register axservice device\n");
        return -1;
    }

    info = kzalloc(sizeof(struct uio_info), GFP_KERNEL);
    info->name = "axservice";
    info->version = "0.0.1";
    info->irq = AX_VIRQ;
    info->irq_flags = IRQF_SHARED;
    // info->handler = my_handler;

    // Memory used for blk req queue.
    info->mem[0].addr = ARCEOS_SYSCALL_QUEUE_BUF_PADDR;
    info->mem[0].name = "req_queue";
    info->mem[0].size = 0x1000;
    info->mem[0].memtype = UIO_MEM_PHYS;

    if (uio_register_device(dev, info) < 0) {
        device_unregister(dev);
        kfree(dev);
        kfree(info);
        ERROR(KERN_INFO "Failing to register axservice uio device\n");
        return -1;
    }
    INFO(KERN_INFO "Registered UIO handler for IRQ=%d\n", irq);
    return 0;
}

static void __exit axservice_exit(void) {
    uio_unregister_device(info);
    device_unregister(dev);
    printk(KERN_INFO "Un-Registered UIO handler for IRQ=%d\n", irq);
    kfree(info);
    kfree(dev);
}

module_init(axservice_init);
module_exit(axservice_exit);

MODULE_AUTHOR("arceos developer team");
MODULE_DESCRIPTION("Kernel driver for ArceOS-Hypervisor");
MODULE_LICENSE("GPL v2");