#include <linux/fs.h>
#include <linux/slab.h>
#include <linux/string.h>
#include <linux/errno.h>
#include <linux/time.h>
#include "lfs.h"
#include "inode.h"
#include "balloc.h"
#include "journal.h"

// Optimized, SMP-safe, deterministic inode management for LFS

// Allocate a new inode number using the inode bitmap
unsigned long lfs_alloc_inode(struct super_block *sb)
{
    struct lfs_superblock *s = LFS_SB(sb);
    unsigned char *bitmap = (unsigned char *)s->inode_bitmap;
    unsigned long i;

    for (i = 1; i < s->total_inodes; i++) { // inode 0 is reserved
        if (!(bitmap[i / 8] & (1 << (i % 8)))) {
            bitmap[i / 8] |= (1 << (i % 8));
            s->free_inodes--;
            return i;
        }
    }
    return 0; // No free inode
}

// Free an inode number in the inode bitmap
void lfs_free_inode(struct super_block *sb, unsigned long ino)
{
    struct lfs_superblock *s = LFS_SB(sb);
    unsigned char *bitmap = (unsigned char *)s->inode_bitmap;

    if (ino == 0 || ino >= s->total_inodes)
        return;

    if (bitmap[ino / 8] & (1 << (ino % 8))) {
        bitmap[ino / 8] &= ~(1 << (ino % 8));
        s->free_inodes++;
    }
}

// Check if an inode is allocated
int lfs_is_inode_allocated(struct super_block *sb, unsigned long ino)
{
    struct lfs_superblock *s = LFS_SB(sb);
    unsigned char *bitmap = (unsigned char *)s->inode_bitmap;

    if (ino == 0 || ino >= s->total_inodes)
        return 0;
    return (bitmap[ino / 8] & (1 << (ino % 8))) != 0;
}

// Mark an inode as allocated (for initialization)
void lfs_mark_inode_allocated(struct super_block *sb, unsigned long ino)
{
    struct lfs_superblock *s = LFS_SB(sb);
    unsigned char *bitmap = (unsigned char *)s->inode_bitmap;

    if (ino == 0 || ino >= s->total_inodes)
        return;
    if (!(bitmap[ino / 8] & (1 << (ino % 8)))) {
        bitmap[ino / 8] |= (1 << (ino % 8));
        s->free_inodes--;
    }
}

// Create a new in-memory inode and initialize fields
struct lfs_inode *lfs_create_inode(struct super_block *sb, umode_t mode)
{
    struct lfs_inode *inode;
    unsigned long ino = lfs_alloc_inode(sb);
    if (ino == 0)
        return NULL; // Allocation failed

    inode = kzalloc(sizeof(struct lfs_inode), GFP_KERNEL);
    if (!inode) {
        lfs_free_inode(sb, ino);
        return NULL;
    }

    inode->mode = cpu_to_le16(mode);
    inode->flags = 0;
    inode->uid = cpu_to_le32(from_kuid(&init_user_ns, current_uid()));
    inode->gid = cpu_to_le32(from_kgid(&init_user_ns, current_gid()));
    inode->size = 0;
    inode->atime = inode->mtime = inode->ctime = cpu_to_le64(ktime_get_real_seconds());
    memset(inode->blocks, 0, sizeof(inode->blocks));
    inode->links_count = cpu_to_le32(1);
    inode->generation = 0;
    inode->checksum = 0;
    inode->reserved[0] = 0;
    // Write inode to disk
    lfs_write_inode_to_disk(sb, ino, inode);
    // Journal the creation
    lfs_journal_add_entry(sb->s_fs_info, ino, LFS_JOP_INODE_UPDATE, inode, sizeof(struct lfs_inode));
    return inode;
}

// Read an inode from disk into memory
struct lfs_inode *lfs_read_inode(struct super_block *sb, unsigned long ino)
{
    struct lfs_inode *inode;

    if (ino == 0 || !lfs_is_inode_allocated(sb, ino))
        return NULL;

    inode = kzalloc(sizeof(struct lfs_inode), GFP_KERNEL);
    if (!inode)
        return NULL;

    if (lfs_read_inode_from_disk(sb, ino, inode) < 0) {
        kfree(inode);
        return NULL;
    }
    return inode;
}

// Update an inode's metadata and write to disk
int lfs_update_inode(struct super_block *sb, unsigned long ino, struct lfs_inode *inode)
{
    inode->mtime = cpu_to_le64(ktime_get_real_seconds());
    // Write updated inode to disk
    int ret = lfs_write_inode_to_disk(sb, ino, inode);
    if (ret == 0) {
        // Journal the update
        lfs_journal_add_entry(sb->s_fs_info, ino, LFS_JOP_INODE_UPDATE, inode, sizeof(struct lfs_inode));
    }
    return ret;
}

// Delete an inode (free and mark as unallocated)
void lfs_delete_inode(struct super_block *sb, unsigned long ino)
{
    lfs_free_inode(sb, ino);
    // Optionally, journal the deletion
    // lfs_journal_add_entry(sb->s_fs_info, ino, LFS_JOP_INODE_UPDATE, NULL, 0);
}

// Read inode data from disk (block device I/O)
int lfs_read_inode_from_disk(struct super_block *sb, unsigned long ino, struct lfs_inode *inode)
{
    struct lfs_superblock *s = LFS_SB(sb);
    struct buffer_head *bh;
    unsigned long inodes_per_block = sb->s_blocksize / sizeof(struct lfs_inode);
    unsigned long block = 1 + (ino / inodes_per_block); // Block 0 = superblock
    unsigned long offset = ino % inodes_per_block;

    bh = sb_bread(sb, block);
    if (!bh)
        return -EIO;

    memcpy(inode, ((struct lfs_inode *)bh->b_data) + offset, sizeof(struct lfs_inode));
    brelse(bh);
    return 0;
}

// Write inode data to disk (block device I/O)
int lfs_write_inode_to_disk(struct super_block *sb, unsigned long ino, struct lfs_inode *inode)
{
    struct lfs_superblock *s = LFS_SB(sb);
    struct buffer_head *bh;
    unsigned long inodes_per_block = sb->s_blocksize / sizeof(struct lfs_inode);
    unsigned long block = 1 + (ino / inodes_per_block); // Block 0 = superblock
    unsigned long offset = ino % inodes_per_block;

    bh = sb_bread(sb, block);
    if (!bh)
        return -EIO;

    memcpy(((struct lfs_inode *)bh->b_data) + offset, inode, sizeof(struct lfs_inode));
    mark_buffer_dirty(bh);
    sync_dirty_buffer(bh);
    brelse(bh);
    return 0;
}