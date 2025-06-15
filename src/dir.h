#ifndef LFS_DIR_H
#define LFS_DIR_H

#include <linux/types.h>
#include <linux/fs.h>

// Directory entry structure
struct lfs_dir_entry {
    __le32 inode;          // Inode number of the file
    char name[256];       // File name (up to 255 characters + null terminator)
};

// Directory structure
struct lfs_dir {
    struct inode *inode;   // Pointer to the inode of the directory
    struct lfs_dir_entry *entries; // Pointer to the directory entries
    unsigned int entry_count; // Number of entries in the directory
};

// Function prototypes for directory management
int lfs_create_dir(struct inode *parent_inode, const char *name);
int lfs_remove_dir(struct inode *dir_inode, const char *name);
struct lfs_dir *lfs_read_dir(struct inode *dir_inode);
void lfs_free_dir(struct lfs_dir *dir);

#endif // LFS_DIR_H