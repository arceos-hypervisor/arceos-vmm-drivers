#include <stdio.h>
#include <unistd.h>

#include "arceos_vdev.h"
#include "arceos_scf.h"

int main() {
    int fd;
    puts("Shadow process started.");

    fd = arceos_vdev_open();
    if (fd <= 0) {
        printf("Failed to open arceos-vdev device `%s`\n", VDEV_PATH);
        return fd;
    }

    for (;;) {
        // puts("Shadow-process tick...");
        poll_requests();
        usleep(100000);
    }

    arceos_vdev_close(fd);
    return 0;
}
