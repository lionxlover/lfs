#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>
#include <stdint.h>
#include <inttypes.h>
#include "lfs_format.h"

// Helper to print UUID as hex string
static void print_uuid(const uint8_t uuid[16]) {
    for (int i = 0; i < 16; ++i)
        printf("%02x%s", uuid[i], (i == 15) ? "" : "-");
    printf("\n");
}

// Function to print the superblock information
void print_superblock(int fd) {
    struct lfs_superblock sb;

    // Read the superblock from disk
    if (lseek(fd, 0, SEEK_SET) < 0 || read(fd, &sb, sizeof(sb)) != sizeof(sb)) {
        perror("Failed to read superblock");
        return;
    }

    printf("Superblock Information:\n");
    printf("  Magic Number: 0x%X\n", sb.magic);
    printf("  Version: %u\n", sb.version);
    printf("  Block Size: %u\n", sb.block_size);
    printf("  Total Blocks: %u\n", sb.total_blocks);
    printf("  Free Blocks: %u\n", sb.free_blocks);
    printf("  Total Inodes: %u\n", sb.total_inodes);
    printf("  Free Inodes: %u\n", sb.free_inodes);
    printf("  Journal Start: %" PRIu64 "\n", (uint64_t)sb.journal_start);
    printf("  Journal Size: %u\n", sb.journal_size);
    printf("  State: %s\n", (sb.state == LFS_STATE_CLEAN) ? "CLEAN" : "DIRTY");
    printf("  UUID: ");
    print_uuid(sb.uuid);
    printf("  Checksum: 0x%X\n", sb.checksum);
}

// Function to print inode information
void print_inode(int fd, uint32_t inode_number, uint32_t block_size) {
    struct lfs_inode inode;
    off_t inode_table_start = block_size; // Block 0 = superblock, inode table starts at block 1
    off_t inode_offset = inode_table_start + inode_number * sizeof(struct lfs_inode);

    if (lseek(fd, inode_offset, SEEK_SET) < 0 ||
        read(fd, &inode, sizeof(inode)) != sizeof(inode)) {
        perror("Failed to read inode");
        return;
    }

    printf("Inode Information for inode %u:\n", inode_number);
    printf("  Mode: 0x%04x\n", inode.mode);
    printf("  Flags: 0x%04x\n", inode.flags);
    printf("  UID: %u\n", inode.uid);
    printf("  GID: %u\n", inode.gid);
    printf("  Size: %" PRIu64 "\n", (uint64_t)inode.size);
    printf("  atime: %" PRIu64 "\n", (uint64_t)inode.atime);
    printf("  mtime: %" PRIu64 "\n", (uint64_t)inode.mtime);
    printf("  ctime: %" PRIu64 "\n", (uint64_t)inode.ctime);
    printf("  Links Count: %u\n", inode.links_count);
    printf("  Generation: %u\n", inode.generation);
    printf("  Block Pointers: ");
    for (int i = 0; i < LFS_N_BLOCKS; i++) {
        printf("%u ", inode.blocks[i]);
    }
    printf("\n  Checksum: 0x%X\n", inode.checksum);
}

// Main function for lfs-dump utility
int main(int argc, char *argv[]) {
    if (argc < 3) {
        fprintf(stderr, "Usage: %s <device> <inode_number>\n", argv[0]);
        return EXIT_FAILURE;
    }

    const char *device = argv[1];
    uint32_t inode_number = (uint32_t)strtoul(argv[2], NULL, 10);

    // Open the filesystem device
    int fd = open(device, O_RDONLY);
    if (fd < 0) {
        perror("Failed to open device");
        return EXIT_FAILURE;
    }

    // Print superblock information
    struct lfs_superblock sb;
    if (lseek(fd, 0, SEEK_SET) < 0 || read(fd, &sb, sizeof(sb)) != sizeof(sb)) {
        perror("Failed to read superblock");
        close(fd);
        return EXIT_FAILURE;
    }
    print_superblock(fd);

    // Print inode information
    print_inode(fd, inode_number, sb.block_size);

    // Close the device
    close(fd);
    return EXIT_SUCCESS;
}