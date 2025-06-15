# Makefile for Lion's File System (LFS)

# This Makefile is responsible for compiling the LFS kernel module and linking the necessary object files.

# Define the kernel build directory
KDIR ?= /lib/modules/$(shell uname -r)/build

# Compiler and flags
CC = gcc
CFLAGS = -Wall -Werror -O2 -I$(KDIR)/include

# Source files
SRC = super.c inode.c balloc.c journal.c dir.c file.c
OBJ = $(SRC:.c=.o)

# Module name
MODULE = lfs

# Default target
all: $(MODULE).ko

# Build the kernel module
$(MODULE).ko: $(OBJ)
	$(CC) -shared -o $@ $^ -Wl,--no-as-needed

# Compile source files into object files
%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

# Clean up build artifacts
clean:
	rm -f $(OBJ) $(MODULE).ko

# Phony targets
.PHONY: all clean