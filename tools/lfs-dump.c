#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>
#include <stdint.h>
#include "lfs.h"
#include "lfs_format.h"

// Function to print the superblock information
void print_superblock(int fd) {
    struct lfs_superblock sb;

    // Read the superblock from disk
    if (lseek(fd, 0, SEEK_SET) < 0 || read(fd, &sb, sizeof(sb)) != sizeof(sb)) {
        perror("Failed to read superblock");
        return;
    }

    // Print superblock details
    printf("Superblock Information:\n");
    printf("Magic Number: 0x%X\n", sb.magic);
    printf("Version: %u\n", sb.version);
    printf("Block Size: %u\n", sb.block_size);
    printf("Total Blocks: %u\n", sb.total_blocks);
    printf("Free Blocks: %u\n", sb.free_blocks);
    printf("UUID: %s\n", sb.uuid);
}

// Function to print inode information
void print_inode(int fd, uint32_t inode_number) {
    struct lfs_inode inode;

    // Read the inode from disk
    if (lseek(fd, sizeof(struct lfs_superblock) + inode_number * sizeof(struct lfs_inode), SEEK_SET) < 0 ||
        read(fd, &inode, sizeof(inode)) != sizeof(inode)) {
        perror("Failed to read inode");
        return;
    }

    // Print inode details
    printf("Inode Information for inode %u:\n", inode_number);
    printf("File Size: %u\n", inode.size);
    printf("Block Count: %u\n", inode.block_count);
    printf("Direct Blocks: ");
    for (int i = 0; i < 12; i++) {
        printf("%u ", inode.direct_blocks[i]);
    }
    printf("\nIndirect Block: %u\n", inode.indirect_block);
}

// Main function for lfs-dump utility
int main(int argc, char *argv[]) {
    if (argc < 3) {
        fprintf(stderr, "Usage: %s <device> <inode_number>\n", argv[0]);
        return EXIT_FAILURE;
    }

    const char *device = argv[1];
    uint32_t inode_number = atoi(argv[2]);

    // Open the filesystem device
    int fd = open(device, O_RDONLY);
    if (fd < 0) {
        perror("Failed to open device");
        return EXIT_FAILURE;
    }

    // Print superblock information
    print_superblock(fd);

    // Print inode information
    print_inode(fd, inode_number);

    // Close the device
    close(fd);
    return EXIT_SUCCESS;
}