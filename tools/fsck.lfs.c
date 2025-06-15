#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <stdint.h>
#include "lfs_format.h"

// Userland definitions matching on-disk format
#define LFS_STATUS_CLEAN 0x00
#define LFS_STATUS_DIRTY 0x01
#define JOURNAL_ENTRY_PENDING 1

// Minimal userland superblock and journal entry structures
struct superblock {
    uint32_t magic;
    uint32_t version;
    uint32_t block_size;
    uint32_t total_blocks;
    uint32_t free_blocks;
    uint32_t total_inodes;
    uint32_t free_inodes;
    uint64_t journal_start;
    uint32_t journal_size;
    uint32_t status;
    uint8_t  uuid[16];
    uint32_t checksum;
    uint32_t reserved[13];
} __attribute__((packed));

struct journal_entry {
    uint64_t transaction_id;
    uint64_t timestamp;
    uint32_t inode_num;
    uint8_t  op_type;
    uint8_t  status;
    uint8_t  reserved[39]; // Pad to 56 bytes for alignment/future
} __attribute__((packed));

// Function prototypes
static int check_superblock(int fd, struct superblock *sb);
static int check_journal(int fd, const struct superblock *sb);
static void print_usage(const char *progname);

int main(int argc, char *argv[]) {
    if (argc != 2) {
        print_usage(argv[0]);
        return EXIT_FAILURE;
    }

    const char *device = argv[1];
    int fd = open(device, O_RDONLY);
    if (fd < 0) {
        perror("Failed to open device");
        return EXIT_FAILURE;
    }

    struct superblock sb;
    // Check the superblock for consistency
    if (check_superblock(fd, &sb) != 0) {
        close(fd);
        return EXIT_FAILURE;
    }

    // Check the journal for any pending transactions
    if (check_journal(fd, &sb) != 0) {
        close(fd);
        return EXIT_FAILURE;
    }

    printf("Filesystem on %s is clean.\n", device);
    close(fd);
    return EXIT_SUCCESS;
}

static int check_superblock(int fd, struct superblock *sb) {
    // Read the superblock from the device
    if (lseek(fd, 0, SEEK_SET) < 0) {
        perror("Failed to seek to superblock");
        return -1;
    }
    if (read(fd, sb, sizeof(*sb)) != sizeof(*sb)) {
        perror("Failed to read superblock");
        return -1;
    }

    // Validate the superblock
    if (sb->magic != LFS_MAGIC) {
        fprintf(stderr, "Invalid superblock magic number: 0x%x\n", sb->magic);
        return -1;
    }
    if (sb->block_size < 1024 || sb->block_size > 65536) {
        fprintf(stderr, "Unsupported block size: %u\n", sb->block_size);
        return -1;
    }
    if (sb->status == LFS_STATUS_DIRTY) {
        fprintf(stderr, "Filesystem is dirty. Recovery required.\n");
        // Optionally, trigger recovery logic here
        return -1;
    }
    return 0;
}

static int check_journal(int fd, const struct superblock *sb) {
    struct journal_entry entry;
    off_t journal_offset = sb->journal_start * sb->block_size;
    size_t journal_bytes = sb->journal_size * sb->block_size;
    size_t read_bytes = 0;

    if (lseek(fd, journal_offset, SEEK_SET) < 0) {
        perror("Failed to seek to journal");
        return -1;
    }

    while (read_bytes < journal_bytes &&
           read(fd, &entry, sizeof(entry)) == sizeof(entry)) {
        if (entry.status == JOURNAL_ENTRY_PENDING) {
            fprintf(stderr, "Found pending journal entry (txid=%" PRIu64 "). Recovery needed.\n", entry.transaction_id);
            // Optionally, trigger recovery logic here
            return -1;
        }
        read_bytes += sizeof(entry);
    }

    return 0;
}

static void print_usage(const char *progname) {
    fprintf(stderr, "Usage: %s <device>\n", progname);
    fprintf(stderr, "Check the consistency of the LFS filesystem.\n");
}