#ifndef LFS_INODE_H
#define LFS_INODE_H

#include <linux/types.h>
#include <linux/fs.h>
#include <linux/time.h>
#include "lfs_format.h" // Use on-disk inode definition for consistency

// In-memory inode structure for LFS (mirrors on-disk, adds kernel/VFS integration)
struct lfs_inode_info {
    struct inode vfs_inode;                  // Embedded VFS inode
    struct lfs_inode disk_inode;             // On-disk inode structure (see lfs_format.h)
    spinlock_t lock;                         // Per-inode lock for SMP safety
    unsigned long ino;                       // Inode number
    // Add more fields as needed for caching, journaling, etc.
};

// Inode management API (kernel-space, SMP-safe)
// Get an in-memory inode by number (loads from disk if needed)
struct lfs_inode_info *lfs_get_inode(struct super_block *sb, unsigned long ino);

// Release an in-memory inode (decrement refcount, free if needed)
void lfs_put_inode(struct lfs_inode_info *inode_info);

// Create a new inode on disk and in memory
int lfs_create_inode(struct super_block *sb, unsigned long ino, umode_t mode, kuid_t uid, kgid_t gid);

// Delete an inode (remove from disk and free memory)
void lfs_delete_inode(struct lfs_inode_info *inode_info);

// Update an inode's metadata on disk (write-back)
int lfs_update_inode(struct super_block *sb, struct lfs_inode_info *inode_info);

// Utility: Convert VFS inode to LFS inode_info
static inline struct lfs_inode_info *LFS_I(const struct inode *inode)
{
    return container_of(inode, struct lfs_inode_info, vfs_inode);
}

#endif // LFS_INODE_H