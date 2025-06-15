#ifndef LFS_SUPER_H
#define LFS_SUPER_H

#include <linux/fs.h>
#include <linux/types.h>
#include <linux/uuid.h>

// Magic number for LFS
#define LFS_MAGIC 0x4C465300 // "LFS\0"

// Superblock structure definition
struct lfs_super_block {
    __le32 s_magic;                // Magic number
    __le32 s_block_size;           // Size of blocks in bytes
    __le32 s_inode_count;          // Total number of inodes
    __le32 s_block_count;          // Total number of blocks
    __le32 s_free_blocks;          // Number of free blocks
    __le32 s_free_inodes;          // Number of free inodes
    __le32 s_first_data_block;     // First data block
    __le32 s_log_block_size;       // Block size for logging
    __le32 s_mount_count;           // Mount count
    __le32 s_max_mount_count;       // Maximum mount count before fsck
    __le32 s_state;                 // Filesystem state (clean/dirty)
    uuid_t s_uuid;                  // UUID of the filesystem
    char s_volume_name[16];         // Volume name
    // Additional fields can be added as needed
};

// Function prototypes for superblock management
struct lfs_super_block *lfs_get_super_block(struct super_block *sb);
int lfs_read_super_block(struct super_block *sb);
int lfs_write_super_block(struct super_block *sb);
void lfs_update_super_block(struct super_block *sb);
void lfs_mount(struct super_block *sb);
void lfs_unmount(struct super_block *sb);

#endif // LFS_SUPER_H