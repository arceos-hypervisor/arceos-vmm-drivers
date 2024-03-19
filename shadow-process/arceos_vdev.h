#ifndef _ARCEOS_VDEV_H
#define _ARCEOS_VDEV_H 1

#include "definitions.h"

#define VDEV_PATH "/dev/arceos_vdev"

int arceos_vdev_open(void);
void arceos_vdev_close(int fd);

int arceos_vdev_send_hypercall(int fd, uint32_t id, uint32_t arg0, uint32_t arg1, uint32_t arg2, uint32_t arg3, uint32_t arg4, uint32_t *ret_val);
int arceos_vdev_hypercall_shadow_process_ready(int fd);
int arceos_vdev_hypercall_ept_mapping_request(int fd, uint64_t hpa, uint64_t gpa, uint32_t size);

#endif
