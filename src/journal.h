#ifndef LFS_JOURNAL_H
#define LFS_JOURNAL_H

#include <linux/types.h>
#include <linux/fs.h>
#include <linux/slab.h>
#include <linux/ktime.h>
#include "lfs.h"

// Define the maximum number of journal entries
#define MAX_JOURNAL_ENTRIES 1024

// Journal entry structure
struct lfs_journal_entry {
    __le64 transaction_id; // Unique ID for the transaction
    __le64 timestamp;      // Time of the transaction
    struct lfs_inode *inode; // Pointer to the affected inode
    void *data;            // Pointer to the data being modified
    size_t data_size;      // Size of the data
};

// Journal structure
struct lfs_journal {
    struct lfs_journal_entry entries[MAX_JOURNAL_ENTRIES]; // Array of journal entries
    int head;            // Index of the next entry to write
    int tail;            // Index of the next entry to read
    int count;           // Number of entries in the journal
    struct mutex lock;   // Mutex for synchronizing access to the journal
};

// Function prototypes
struct lfs_journal *lfs_journal_create(void);
void lfs_journal_destroy(struct lfs_journal *journal);
int lfs_journal_add_entry(struct lfs_journal *journal, struct lfs_inode *inode, void *data, size_t data_size);
int lfs_journal_commit(struct lfs_journal *journal);
void lfs_journal_replay(struct lfs_journal *journal);
void lfs_journal_clear(struct lfs_journal *journal);

#endif // LFS_JOURNAL_H