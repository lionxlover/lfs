#ifndef LFS_FILE_H
#define LFS_FILE_H

#include <linux/fs.h> // For struct file_operations and other filesystem-related structures
#include "lfs.h"      // For common LFS definitions

// Define the file structure for LFS
struct lfs_file {
    struct inode *inode;          // Pointer to the associated inode
    struct file *file;            // Pointer to the file structure
    loff_t pos;                   // Current file position
};

// Function prototypes for file operations
struct lfs_file *lfs_file_open(const char *path, int flags);
ssize_t lfs_file_read(struct lfs_file *lfs_file, void *buf, size_t count);
ssize_t lfs_file_write(struct lfs_file *lfs_file, const void *buf, size_t count);
int lfs_file_close(struct lfs_file *lfs_file);
int lfs_file_seek(struct lfs_file *lfs_file, loff_t offset, int whence);

#endif // LFS_FILE_H