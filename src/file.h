#ifndef LFS_FILE_H
#define LFS_FILE_H

#include <linux/fs.h> // For struct file_operations and other filesystem-related structures
#include "lfs.h"      // For common LFS definitions

// LFS file structure for efficient file operations and state tracking
struct lfs_file {
    struct inode *inode;          // Associated inode (kernel object)
    struct file *file;            // VFS file pointer
    loff_t pos;                   // Current file position
    unsigned int flags;           // Open flags (O_RDONLY, O_WRONLY, etc.)
    spinlock_t lock;              // Per-file spinlock for SMP safety
};

// File operation prototypes (kernel-space, SMP-safe)
struct lfs_file *lfs_file_open(const char *path, int flags, umode_t mode);
ssize_t lfs_file_read(struct lfs_file *lfs_file, void *buf, size_t count, loff_t *pos);
ssize_t lfs_file_write(struct lfs_file *lfs_file, const void *buf, size_t count, loff_t *pos);
int lfs_file_close(struct lfs_file *lfs_file);
loff_t lfs_file_seek(struct lfs_file *lfs_file, loff_t offset, int whence);

#endif // LFS_FILE_H