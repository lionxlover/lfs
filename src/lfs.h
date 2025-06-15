#ifndef LFS_H
#define LFS_H

#include <linux/fs.h>
#include <linux/slab.h>
#include <linux/buffer_head.h>
#include <linux/errno.h>
#include <linux/types.h>
#include <linux/time.h>

// Define the magic number for LFS
#define LFS_MAGIC 0x4C4653 // "LFS"

// Superblock structure
struct lfs_super_block {
    __le32 s_magic;          // Magic number
    __le32 s_block_size;     // Size of blocks
    __le32 s_inode_count;     // Total number of inodes
    __le32 s_block_count;     // Total number of blocks
    __le32 s_free_blocks;     // Number of free blocks
    __le32 s_free_inodes;     // Number of free inodes
    __le32 s_dirty;           // Dirty flag for journal recovery
    char s_uuid[16];          // UUID for the filesystem
    // Additional fields can be added as needed
};

// Inode structure
struct lfs_inode {
    __le32 i_mode;           // File mode
    __le32 i_uid;            // Owner UID
    __le32 i_gid;            // Group ID
    __le32 i_size;           // Size of the file
    __le32 i_blocks;         // Number of blocks allocated
    __le32 i_atime;          // Last access time
    __le32 i_mtime;          // Last modification time
    __le32 i_ctime;          // Last status change time
    // Additional fields can be added as needed
};

// Function prototypes
int lfs_mount(struct super_block *sb);
void lfs_unmount(struct super_block *sb);
struct lfs_super_block *lfs_get_super_block(struct super_block *sb);
void lfs_put_super_block(struct lfs_super_block *s);
struct lfs_inode *lfs_get_inode(struct super_block *sb, unsigned long ino);
void lfs_put_inode(struct lfs_inode *inode);

#endif // LFS_H