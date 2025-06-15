#ifndef LFS_BALLOC_H
#define LFS_BALLOC_H

#include <linux/types.h>
#include <linux/fs.h>

// Block allocation is managed via the superblock's bitmap for global consistency.
// No fixed MAX_BLOCKS; use dynamic sizing based on superblock geometry.

// SMP-safe, kernel-optimized block allocation API for LFS

// Allocate a free block, mark as used, and return its number (0-based)
// Returns 0 on success, -ENOSPC if no free block is available
int lfs_allocate_block(struct super_block *sb, unsigned long *block_num);

// Free a previously allocated block (mark as free in bitmap)
void lfs_free_block(struct super_block *sb, unsigned long block_num);

// Initialize the block bitmap (allocates and zeroes memory)
int lfs_init_block_bitmap(struct super_block *sb);

// Free the block bitmap memory
void lfs_cleanup_block_bitmap(struct super_block *sb);

#endif // LFS_BALLOC_H