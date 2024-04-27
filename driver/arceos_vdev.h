#ifndef _ARCEOS_VDEV_H
#define _ARCEOS_VDEV_H 1

#include <linux/file.h>
#include <linux/miscdevice.h>
#include <linux/mm.h>

int arceos_vdev_open(struct inode *inode, struct file *file);
int arceos_vdev_close(struct inode *inode, struct file *file);
long arceos_vdev_ioctl(struct file *file, unsigned int ioctl, unsigned long arg);
int arceos_vdev_mmap(struct file *file, struct vm_area_struct *vma);

#endif
