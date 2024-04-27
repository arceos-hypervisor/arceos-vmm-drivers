#include "arceos_vdev.h"

#include "arceos_virq.h"
#include "arceos_hypercall.h"
#include "definitions.h"

#include <linux/mm.h>

#ifndef LOGHEAD
# undef LOGHEAD
#endif

#define LOGHEAD "[arceos-vdev] "

int arceos_vdev_open(struct inode *inode, struct file *file) {
    pr_info(LOGHEAD "vdev opened by %p(%d).\n", get_current(), get_current()->pid);
	return 0;
}

int arceos_vdev_close(struct inode *inode, struct file *file) {
    unregister_virq_handler(get_current());
    pr_info(LOGHEAD "vdev closed by %p(%d).\n", get_current(), get_current()->pid);

	return 0;
}

long arceos_vdev_do_hypercall(void __user* arg) {
    struct arceos_vdev_hypercall_args args;
    int not_copied;

    not_copied = copy_from_user(&args, arg, sizeof(args));
    if (not_copied != 0) {
        return -EFAULT;
    }

    pr_info(LOGHEAD "vdev hypercall from %p(%d): id 0x%x, arg0 0x%x, arg1 0x%x, arg2 0x%x, arg3 0x%x, arg4 0x%x.\n", get_current(), get_current()->pid, args.id, args.arg0, args.arg1, args.arg2, args.arg3, args.arg4);

    args.return_value = arceos_hypercall(args.id, args.arg0, args.arg1, args.arg2, args.arg3, args.arg4);
    
    not_copied = copy_to_user(arg, &args, sizeof(args));
    if (not_copied != 0) {
        return -EFAULT;
    }

    return 0;
}

long arceos_vdev_ioctl(struct file *file, unsigned int ioctl, unsigned long arg) {
    long err;

    pr_info(LOGHEAD "vdev ioctl from %p(%d): ioctl %d, arg %ld.\n", get_current(), get_current()->pid, ioctl, arg);

    switch (ioctl) {
    case ARCEOS_VDEV_IOCTL_REGISTER_VIRQ_HANDLER:
        err = register_virq_handler(get_current());
        break;
    case ARCEOS_VDEV_IOCTL_UNREGISTER_VIRQ_HANDLER:
        err = unregister_virq_handler(get_current());
        break;
    case ARCEOS_VDEV_IOCTL_INVOKE_HYPERCALL:
        err = arceos_vdev_do_hypercall((void __user*)arg);break;
        break;
    default:
        err = -EINVAL;
        break;
    }

	return err;
}

int arceos_vdev_mmap(struct file *file, struct vm_area_struct *vma) {
    unsigned long paddr;
    unsigned long size;
    unsigned long map_size;

    if (vma->vm_pgoff < 2) { // if pgoff is 0 or 1 (offset is 0 or 0x1000), then it means it's the data buffer or the queue buffer
        switch (vma->vm_pgoff) {
        case 0: // data buffer
            size = ARCEOS_SYSCALL_DATA_BUF_SIZE;
            paddr = ARCEOS_SYSCALL_DATA_BUF_PADDR;
            break;
        case 1: // queue buffer
            size = ARCEOS_SYSCALL_QUEUE_BUF_SIZE;
            paddr = ARCEOS_SYSCALL_QUEUE_BUF_PADDR;
            break;
        default:
            return -EINVAL;
        }

        if ((map_size = vma->vm_end - vma->vm_start) > size) {
            return -ENOMEM;
        }

        pr_info(LOGHEAD "vdev mmap by %p(%d): vaddr [0x%lx, 0x%lx), pgoff 0x%lx, paddr 0x%lx, size 0x%lx.\n", get_current(), get_current()->pid, vma->vm_start, vma->vm_end, vma->vm_pgoff, paddr, size);

        return remap_pfn_range(vma, vma->vm_start, paddr >> PAGE_SHIFT, map_size, vma->vm_page_prot);
    } else {
        size_t size = vma->vm_end - vma->vm_start;
        phys_addr_t offset = (phys_addr_t)vma->vm_pgoff << PAGE_SHIFT;

        /* Does it even fit in phys_addr_t? */
        if (offset >> PAGE_SHIFT != vma->vm_pgoff)
            return -EINVAL;

        /* It's illegal to wrap around the end of the physical address space. */
        if (offset + (phys_addr_t)size - 1 < offset)
            return -EINVAL;

        pr_info(LOGHEAD "vdev mmap by %p(%d): vaddr [0x%lx, 0x%lx), pgoff 0x%lx, paddr 0x%llx, size 0x%lx.\n", get_current(), get_current()->pid, vma->vm_start, vma->vm_end, vma->vm_pgoff, offset, size);

        return remap_pfn_range(vma, vma->vm_start, vma->vm_pgoff, size, vma->vm_page_prot);
    }
}
