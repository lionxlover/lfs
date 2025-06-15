#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <errno.h>
#include "lfs.h"
#include "lfs_format.h"

// Function prototypes
void create_superblock(int fd);
void write_superblock(int fd, struct lfs_superblock *sb);
void usage(const char *progname);

int main(int argc, char *argv[]) {
    if (argc != 2) {
        usage(argv[0]);
        return EXIT_FAILURE;
    }

    const char *device = argv[1];
    int fd = open(device, O_RDWR | O_CREAT | O_TRUNC, S_IRUSR | S_IWUSR);
    if (fd < 0) {
        perror("Failed to open device");
        return EXIT_FAILURE;
    }

    create_superblock(fd);
    close(fd);
    printf("LFS filesystem created on %s\n", device);
    return EXIT_SUCCESS;
}

void create_superblock(int fd) {
    struct lfs_superblock sb;

    // Initialize the superblock with default values
    memset(&sb, 0, sizeof(sb));
    sb.magic = LFS_MAGIC; // Set the magic number
    sb.version = LFS_VERSION; // Set the version
    sb.block_size = LFS_BLOCK_SIZE; // Set the block size
    sb.total_blocks = LFS_TOTAL_BLOCKS; // Set total blocks
    sb.free_blocks = LFS_TOTAL_BLOCKS; // Initially, all blocks are free
    sb.inode_count = 0; // No inodes created yet
    sb.dirty = 0; // Clean state

    write_superblock(fd, &sb);
}

void write_superblock(int fd, struct lfs_superblock *sb) {
    if (lseek(fd, 0, SEEK_SET) < 0) {
        perror("Failed to seek to the start of the device");
        exit(EXIT_FAILURE);
    }

    if (write(fd, sb, sizeof(*sb)) != sizeof(*sb)) {
        perror("Failed to write superblock");
        exit(EXIT_FAILURE);
    }
}

void usage(const char *progname) {
    fprintf(stderr, "Usage: %s <device>\n", progname);
    fprintf(stderr, "Create a new LFS filesystem on the specified block device.\n");
}