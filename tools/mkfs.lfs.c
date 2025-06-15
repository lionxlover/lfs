#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <errno.h>
#include <stdint.h>
#include <time.h>
#include "lfs_format.h"

// Tunable defaults
#define DEFAULT_BLOCK_SIZE   4096
#define DEFAULT_TOTAL_BLOCKS 65536
#define DEFAULT_TOTAL_INODES 4096
#define DEFAULT_JOURNAL_SIZE 128

// Function prototypes
static void create_superblock(int fd, uint32_t block_size, uint32_t total_blocks, uint32_t total_inodes, uint32_t journal_size);
static void write_superblock(int fd, struct lfs_superblock *sb);
static void usage(const char *progname);

int main(int argc, char *argv[]) {
    if (argc != 2) {
        usage(argv[0]);
        return EXIT_FAILURE;
    }

    const char *device = argv[1];
    int fd = open(device, O_RDWR | O_CREAT, 0666);
    if (fd < 0) {
        perror("Failed to open device");
        return EXIT_FAILURE;
    }

    // For a real mkfs, you would parse options for block size, blocks, inodes, etc.
    create_superblock(fd, DEFAULT_BLOCK_SIZE, DEFAULT_TOTAL_BLOCKS, DEFAULT_TOTAL_INODES, DEFAULT_JOURNAL_SIZE);

    close(fd);
    printf("LFS filesystem created on %s\n", device);
    return EXIT_SUCCESS;
}

static void create_superblock(int fd, uint32_t block_size, uint32_t total_blocks, uint32_t total_inodes, uint32_t journal_size) {
    struct lfs_superblock sb;
    memset(&sb, 0, sizeof(sb));

    sb.magic        = LFS_MAGIC;
    sb.version      = 1;
    sb.block_size   = block_size;
    sb.total_blocks = total_blocks;
    sb.free_blocks  = total_blocks;
    sb.total_inodes = total_inodes;
    sb.free_inodes  = total_inodes;
    sb.journal_start = 1 + ((total_inodes * sizeof(struct lfs_inode) + block_size - 1) / block_size); // after inode table
    sb.journal_size  = journal_size;
    sb.state         = LFS_STATE_CLEAN;

    // Generate a random UUID
    for (int i = 0; i < 16; ++i)
        sb.uuid[i] = (uint8_t)(rand() & 0xFF);

    // Compute a simple checksum (for demo, not cryptographically strong)
    uint32_t *p = (uint32_t *)&sb;
    uint32_t sum = 0;
    for (size_t i = 0; i < (sizeof(sb) / sizeof(uint32_t)) - 1; ++i)
        sum ^= p[i];
    sb.checksum = sum;

    write_superblock(fd, &sb);

    // Optionally, zero out the inode table and journal area for safety
    size_t inode_table_bytes = total_inodes * sizeof(struct lfs_inode);
    size_t journal_bytes = journal_size * block_size;
    char *zero_buf = calloc(1, block_size);
    if (!zero_buf) {
        perror("calloc");
        exit(EXIT_FAILURE);
    }

    // Zero inode table
    off_t off = block_size; // Block 0 = superblock, inode table starts at block 1
    for (size_t written = 0; written < inode_table_bytes; written += block_size) {
        if (lseek(fd, off + written, SEEK_SET) < 0 || write(fd, zero_buf, block_size) != block_size) {
            perror("Failed to zero inode table");
            free(zero_buf);
            exit(EXIT_FAILURE);
        }
    }

    // Zero journal area
    off = sb.journal_start * block_size;
    for (size_t written = 0; written < journal_bytes; written += block_size) {
        if (lseek(fd, off + written, SEEK_SET) < 0 || write(fd, zero_buf, block_size) != block_size) {
            perror("Failed to zero journal area");
            free(zero_buf);
            exit(EXIT_FAILURE);
        }
    }

    free(zero_buf);
}

static void write_superblock(int fd, struct lfs_superblock *sb) {
    if (lseek(fd, 0, SEEK_SET) < 0) {
        perror("Failed to seek to the start of the device");
        exit(EXIT_FAILURE);
    }

    if (write(fd, sb, sizeof(*sb)) != sizeof(*sb)) {
        perror("Failed to write superblock");
        exit(EXIT_FAILURE);
    }
}

static void usage(const char *progname) {
    fprintf(stderr, "Usage: %s <device>\n", progname);
    fprintf(stderr, "Create a new LFS filesystem on the specified block device or image file.\n");
}