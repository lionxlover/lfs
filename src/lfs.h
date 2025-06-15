#ifndef LFS_H
#define LFS_H

#include <linux/fs.h>
#include <linux/slab.h>
#include <linux/buffer_head.h>
#include <linux/errno.h>
#include <linux/types.h>
#include <linux/time.h>
#include "lfs_format.h" // Use canonical on-disk format for consistency

// Use canonical magic number (matches on-disk format)
#define LFS_MAGIC LFS_MAGIC

// Use the canonical superblock and inode structures from lfs_format.h
// This ensures consistency and future-proofing

// Kernel VFS integration helpers
#define LFS_SB(sb) ((struct lfs_superblock *)((sb)->s_fs_info))

// Function prototypes (kernel-space, SMP-safe, robust)
// Mount and unmount operations
int lfs_mount(struct super_block *sb, void *data, int silent);
void lfs_unmount(struct super_block *sb);

// Superblock helpers
struct lfs_superblock *lfs_get_superblock(struct super_block *sb);
void lfs_put_superblock(struct lfs_superblock *s);

// Inode helpers
struct lfs_inode *lfs_get_disk_inode(struct super_block *sb, unsigned long ino);
void lfs_put_disk_inode(struct lfs_inode *inode);

// Utility: Convert VFS inode to LFS inode_info (see inode.h)
struct lfs_inode_info *LFS_I(const struct inode *inode);

// Utility: Convert VFS superblock to LFS superblock
static inline struct lfs_superblock *LFS_SB_INFO(const struct super_block *sb)
{
    return (struct lfs_superblock *)sb->s_fs_info;
}

#endif // LFS_H