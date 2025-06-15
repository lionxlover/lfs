#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include "lfs.h"
#include "super.h"
#include "journal.h"

// Function prototypes
static int check_superblock(int fd);
static int check_journal(int fd);
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

    // Check the superblock for consistency
    if (check_superblock(fd) != 0) {
        close(fd);
        return EXIT_FAILURE;
    }

    // Check the journal for any pending transactions
    if (check_journal(fd) != 0) {
        close(fd);
        return EXIT_FAILURE;
    }

    printf("Filesystem on %s is clean.\n", device);
    close(fd);
    return EXIT_SUCCESS;
}

static int check_superblock(int fd) {
    struct superblock sb;

    // Read the superblock from the device
    if (lseek(fd, 0, SEEK_SET) < 0) {
        perror("Failed to seek to superblock");
        return -1;
    }
    if (read(fd, &sb, sizeof(sb)) != sizeof(sb)) {
        perror("Failed to read superblock");
        return -1;
    }

    // Validate the superblock
    if (sb.magic != LFS_MAGIC) {
        fprintf(stderr, "Invalid superblock magic number: 0x%x\n", sb.magic);
        return -1;
    }

    if (sb.status == LFS_STATUS_DIRTY) {
        fprintf(stderr, "Filesystem is dirty. Running recovery...\n");
        // Recovery logic can be implemented here
    }

    return 0;
}

static int check_journal(int fd) {
    struct journal_entry entry;

    // Read the journal and check for any incomplete transactions
    // This is a simplified example; actual implementation may vary
    if (lseek(fd, sizeof(struct superblock), SEEK_SET) < 0) {
        perror("Failed to seek to journal");
        return -1;
    }

    while (read(fd, &entry, sizeof(entry)) == sizeof(entry)) {
        if (entry.status == JOURNAL_ENTRY_PENDING) {
            fprintf(stderr, "Found pending journal entry. Recovery needed.\n");
            // Recovery logic can be implemented here
            return -1;
        }
    }

    return 0;
}

static void print_usage(const char *progname) {
    fprintf(stderr, "Usage: %s <device>\n", progname);
    fprintf(stderr, "Check the consistency of the LFS filesystem.\n");
}