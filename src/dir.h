#ifndef LFS_DIR_H
#define LFS_DIR_H

#include <linux/types.h>
#include <linux/fs.h>
#include "lfs_format.h" // For on-disk dir entry structure and constants

// In-memory directory entry (matches on-disk, but with null-terminated name for safety)
struct lfs_dir_entry_mem {
    __le32 inode;                  // Inode number of the file
    __u8   file_type;              // File type (see lfs_format.h)
    char   name[LFS_NAME_MAX + 1]; // Null-terminated file name
};

// In-memory directory structure
struct lfs_dir {
    struct inode *inode;                // Pointer to the directory's inode
    struct lfs_dir_entry_mem *entries;  // Array of directory entries
    unsigned int entry_count;           // Number of entries in the directory
};

// Directory management API (POSIX-compliant, SMP-safe)
// Create a new directory under parent_inode with the given name
int lfs_create_dir(struct inode *parent_inode, const char *name, umode_t mode);

// Remove a directory entry by name from dir_inode (must be empty for rmdir)
int lfs_remove_dir(struct inode *dir_inode, const char *name);

// Read all directory entries into memory (caller must free with lfs_free_dir)
struct lfs_dir *lfs_read_dir(struct inode *dir_inode);

// Free memory allocated for an in-memory directory structure
void lfs_free_dir(struct lfs_dir *dir);

// Add a directory entry (file, dir, symlink) to dir_inode
int lfs_add_dir_entry(struct inode *dir_inode, const char *name, __le32 inode, __u8 file_type);

// Remove a directory entry by name (does not remove inode)
int lfs_del_dir_entry(struct inode *dir_inode, const char *name);

// Lookup a directory entry by name, returns inode number or 0 if not found
__le32 lfs_lookup_dir_entry(struct inode *dir_inode, const char *name, __u8 *file_type);

#endif // LFS_DIR_H