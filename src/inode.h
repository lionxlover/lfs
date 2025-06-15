#ifndef LFS_INODE_H
#define LFS_INODE_H

#include <linux/types.h>
#include <linux/fs.h>
#include <linux/time.h>

// Define the maximum number of direct blocks an inode can have
#define LFS_DIRECT_BLOCKS 12

// Define the inode structure
struct lfs_inode {
    __le32 i_mode;                // File mode (permissions)
    __le32 i_uid;                 // Owner's user ID
    __le32 i_gid;                 // Owner's group ID
    __le32 i_size;                // Size of the file in bytes
    __le32 i_atime;               // Last access time
    __le32 i_mtime;               // Last modification time
    __le32 i_ctime;               // Last status change time
    __le32 i_blocks;              // Number of blocks allocated
    __le32 i_block[LFS_DIRECT_BLOCKS + 2]; // Pointers to data blocks (direct + indirect)
    __le32 i_flags;               // File flags (e.g., immutable)
};

// Function prototypes for inode management
struct lfs_inode *lfs_get_inode(struct super_block *sb, unsigned long ino);
void lfs_put_inode(struct lfs_inode *inode);
int lfs_create_inode(struct super_block *sb, unsigned long ino, umode_t mode);
void lfs_delete_inode(struct lfs_inode *inode);
int lfs_update_inode(struct lfs_inode *inode);

#endif // LFS_INODE_H