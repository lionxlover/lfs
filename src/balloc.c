#include <linux/kernel.h>
#include <linux/slab.h>
#include <linux/fs.h>
#include <linux/buffer_head.h>
#include "lfs.h"
#include "balloc.h"
#include "lfs_format.h"

// Efficient block bitmap allocation for LFS
// All bitmap operations are atomic and lock-protected for SMP

// Fine-grained spinlock for bitmap operations (SMP-safe)
static DEFINE_SPINLOCK(lfs_balloc_lock);

// Helper: Mark a block as allocated in the bitmap
static inline void lfs_set_block_used(unsigned char *bitmap, unsigned long block)
{
    bitmap[block / 8] |= (1 << (block % 8));
}

// Helper: Mark a block as free in the bitmap
static inline void lfs_set_block_free(unsigned char *bitmap, unsigned long block)
{
    bitmap[block / 8] &= ~(1 << (block % 8));
}

// Helper: Test if a block is allocated
static inline int lfs_is_block_used(const unsigned char *bitmap, unsigned long block)
{
    return (bitmap[block / 8] & (1 << (block % 8))) != 0;
}

// Allocate a free block, mark it as used in the bitmap, and return its number
int lfs_allocate_block(struct super_block *sb, unsigned long *block_num)
{
    struct lfs_superblock *s = sb->s_fs_info;
    unsigned char *bitmap = (unsigned char *)s->block_bitmap;
    unsigned long i;
    int found = 0;

    spin_lock(&lfs_balloc_lock);
    for (i = 0; i < s->total_blocks; i++) {
        if (!lfs_is_block_used(bitmap, i)) {
            lfs_set_block_used(bitmap, i);
            if (likely(s->free_blocks > 0))
                s->free_blocks--;
            *block_num = i;
            found = 1;
            break;
        }
    }
    spin_unlock(&lfs_balloc_lock);

    return found ? 0 : -ENOSPC;
}

// Free a previously allocated block
void lfs_free_block(struct super_block *sb, unsigned long block_num)
{
    struct lfs_superblock *s = sb->s_fs_info;
    unsigned char *bitmap = (unsigned char *)s->block_bitmap;

    spin_lock(&lfs_balloc_lock);
    if (lfs_is_block_used(bitmap, block_num)) {
        lfs_set_block_free(bitmap, block_num);
        s->free_blocks++;
    }
    spin_unlock(&lfs_balloc_lock);
}

// Initialize the block bitmap (all blocks free except reserved ones)
int lfs_init_block_bitmap(struct super_block *sb)
{
    struct lfs_superblock *s = sb->s_fs_info;
    unsigned long bitmap_size = DIV_ROUND_UP(s->total_blocks, 8);

    s->block_bitmap = kzalloc(bitmap_size, GFP_KERNEL | __GFP_ZERO);
    if (!s->block_bitmap)
        return -ENOMEM;

    // Mark reserved blocks (superblock, bitmaps, journal, etc.) as allocated
    unsigned long reserved = 1; // superblock
    reserved += DIV_ROUND_UP(bitmap_size, s->block_size); // bitmap blocks
    reserved += s->journal_size; // journal blocks

    // Ensure reserved does not exceed total_blocks
    if (reserved > s->total_blocks)
        reserved = s->total_blocks;

    for (unsigned long i = 0; i < reserved; i++)
        lfs_set_block_used((unsigned char *)s->block_bitmap, i);

    if (s->free_blocks > reserved)
        s->free_blocks -= reserved;
    else
        s->free_blocks = 0;

    return 0;
}

// Clean up the block bitmap
void lfs_cleanup_block_bitmap(struct super_block *sb)
{
    struct lfs_superblock *s = sb->s_fs_info;
    if (s->block_bitmap) {
        kfree(s->block_bitmap);
        s->block_bitmap = NULL;
    }
}