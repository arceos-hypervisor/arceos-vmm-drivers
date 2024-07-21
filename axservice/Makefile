obj-m := axservice.o

axservice-objs := main.o

CURRENT_PATH := $(shell pwd)

# native
LINUX_KERNEL_PATH ?= /lib/modules/$(shell uname -r)/build

.PHONY: all module clean

all: module

module:
	make -C $(LINUX_KERNEL_PATH) M=$(CURRENT_PATH) modules

clean:
	make -C $(LINUX_KERNEL_PATH) M=$(CURRENT_PATH) clean