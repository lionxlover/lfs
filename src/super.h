#ifndef LFS_SUPER_H
#define LFS_SUPER_H

#include <linux/fs.h>
#include <linux/types.h>
#include <linux/uuid.h>
#include "lfs_format.h" // Use canonical on-disk format for consistency

// Use canonical magic number and superblock structure from lfs_format.h
#define LFS_MAGIC LFS_MAGIC

// Use the canonical superblock structure for all kernel operations
// This ensures consistency, future-proofing, and compatibility with userland tools

// Kernel VFS integration helper
#define LFS_SB(sb) ((struct lfs_superblock *)((sb)->s_fs_info))

// Function prototypes for superblock management (kernel-space, SMP-safe)
// Read superblock from disk into memory
int lfs_read_superblock(struct super_block *sb, struct lfs_superblock *disk_sb);

// Write superblock from memory to disk
int lfs_write_superblock(struct super_block *sb, const struct lfs_superblock *disk_sb);

// Update in-memory superblock and write to disk (atomic update)
int lfs_update_superblock(struct super_block *sb);

// Mount and unmount operations (should match lfs.h)
int lfs_mount(struct super_block *sb, void *data, int silent);
void lfs_unmount(struct super_block *sb);

#endif // LFS_SUPER_H