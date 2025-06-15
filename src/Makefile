# Makefile for Lion's File System (LFS) kernel module

# Use kernel build system for proper module compilation
KDIR ?= /lib/modules/$(shell uname -r)/build
PWD  := $(shell pwd)
MODULE := lfs

# Kernel module source files
obj-m += $(MODULE).o
$(MODULE)-y := super.o inode.o balloc.o journal.o dir.o file.o

# Default target: build with kernel build system
all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

# Clean up build artifacts
clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean

# Phony targets
.PHONY: all clean

# For userland tools, build separately in ../tools if needed