#ifndef LFS_JOURNAL_H
#define LFS_JOURNAL_H

#include <linux/types.h>
#include <linux/fs.h>
#include <linux/slab.h>
#include <linux/ktime.h>
#include <linux/mutex.h>
#include "lfs.h"
#include "lfs_format.h"

// Maximum number of in-memory journal entries (tune for performance/memory)
#define LFS_MAX_JOURNAL_ENTRIES 1024

// Journal entry structure (metadata journaling, WAL)
struct lfs_journal_entry {
    __le64 transaction_id;      // Unique transaction ID
    __le64 timestamp;           // Transaction time (ns since boot)
    __le32 inode_num;           // Inode number affected (0 if not inode-specific)
    __u8   op_type;             // Operation type (see below)
    void  *data;                // Pointer to data (copied for atomicity)
    size_t data_size;           // Size of data
};

// Journal operation types (expandable)
#define LFS_JOP_INODE_UPDATE   1
#define LFS_JOP_BLOCK_ALLOC    2
#define LFS_JOP_BLOCK_FREE     3
#define LFS_JOP_DIR_UPDATE     4
#define LFS_JOP_SUPER_UPDATE   5
#define LFS_JOP_COMMIT         255

// Journal structure (circular buffer)
struct lfs_journal {
    struct lfs_journal_entry entries[LFS_MAX_JOURNAL_ENTRIES];
    int head;                  // Next entry to write
    int tail;                  // Next entry to read/replay
    int count;                 // Number of valid entries
    struct mutex lock;         // Mutex for SMP safety
    __le64 next_transaction_id;// Monotonic transaction ID
};

// Journal API (kernel-space, SMP-safe)
struct lfs_journal *lfs_journal_create(void);
void lfs_journal_destroy(struct lfs_journal *journal);

// Add a journal entry (returns transaction id or <0 on error)
__le64 lfs_journal_add_entry(struct lfs_journal *journal, __le32 inode_num, __u8 op_type, const void *data, size_t data_size);

// Commit a transaction (write commit marker, flush to disk)
int lfs_journal_commit(struct lfs_journal *journal);

// Replay journal (recovery after crash)
void lfs_journal_replay(struct lfs_journal *journal);

// Clear all journal entries (after successful commit/replay)
void lfs_journal_clear(struct lfs_journal *journal);

#endif // LFS_JOURNAL_H