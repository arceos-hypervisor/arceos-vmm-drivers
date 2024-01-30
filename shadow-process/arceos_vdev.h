#ifndef _ARCEOS_VDEV_H
#define _ARCEOS_VDEV_H 1

#include "definitions.h"

#define VDEV_PATH "/dev/arceos_vdev"

int arceos_vdev_open(void);
void arceos_vdev_close(int fd);

#endif
