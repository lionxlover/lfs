# Makefile for LFS Userland Tools

CC = gcc
CFLAGS = -Wall -Wextra -I../include
LFS_TOOLS = mkfs.lfs fsck.lfs lfs-dump

all: $(LFS_TOOLS)

mkfs.lfs: mkfs.lfs.c
	$(CC) $(CFLAGS) -o mkfs.lfs mkfs.lfs.c

fsck.lfs: fsck.lfs.c
	$(CC) $(CFLAGS) -o fsck.lfs fsck.lfs.c

lfs-dump: lfs-dump.c
	$(CC) $(CFLAGS) -o lfs-dump lfs-dump.c

clean:
	rm -f $(LFS_TOOLS) *.o

.PHONY: all clean