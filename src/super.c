#include <linux/fs.h>
#include <linux/slab.h>
#include <linux/init.h>
#include <linux/module.h>
#include <linux/buffer_head.h>
#include <linux/errno.h>
#include <linux/string.h>
#include "lfs.h"
#include "super.h"

// Use canonical on-disk superblock structure from lfs_format.h
// All superblock operations are SMP-safe and robust

// Fill the VFS superblock from disk (mount-time)
int lfs_fill_super(struct super_block *sb, void *data, int silent)
{
    struct buffer_head *bh;
    struct lfs_superblock *disk_sb;

    bh = sb_bread(sb, 0);
    if (!bh) {
        pr_err("LFS: Unable to read superblock block\n");
        return -EIO;
    }

    disk_sb = (struct lfs_superblock *)bh->b_data;

    // Validate magic number and block size
    if (le32_to_cpu(disk_sb->magic) != LFS_MAGIC) {
        pr_err("LFS: Invalid superblock magic number\n");
        brelse(bh);
        return -EINVAL;
    }
    if (le32_to_cpu(disk_sb->block_size) < 1024 || le32_to_cpu(disk_sb->block_size) > 65536) {
        pr_err("LFS: Unsupported block size\n");
        brelse(bh);
        return -EINVAL;
    }

    // Set VFS superblock fields
    sb->s_magic = le32_to_cpu(disk_sb->magic);
    sb->s_blocksize = le32_to_cpu(disk_sb->block_size);
    sb->s_blocksize_bits = blksize_bits(sb->s_blocksize);
    sb->s_maxbytes = MAX_LFS_FILESIZE;
    sb->s_op = &lfs_sops; // Superblock operations (must be defined elsewhere)
    sb->s_fs_info = kzalloc(sizeof(struct lfs_superblock), GFP_KERNEL);
    if (!sb->s_fs_info) {
        brelse(bh);
        return -ENOMEM;
    }
    memcpy(sb->s_fs_info, disk_sb, sizeof(struct lfs_superblock));

    brelse(bh);
    return 0;
}

// Write the in-memory superblock to disk (atomic update)
int lfs_write_superblock(struct super_block *sb, const struct lfs_superblock *mem_sb)
{
    struct buffer_head *bh;

    bh = sb_bread(sb, 0);
    if (!bh) {
        pr_err("LFS: Unable to read superblock for writing\n");
        return -EIO;
    }

    memcpy(bh->b_data, mem_sb, sizeof(struct lfs_superblock));
    mark_buffer_dirty(bh);
    sync_dirty_buffer(bh);
    brelse(bh);
    return 0;
}

// Update the in-memory superblock and write to disk
int lfs_update_superblock(struct super_block *sb)
{
    struct lfs_superblock *mem_sb = sb->s_fs_info;
    return lfs_write_superblock(sb, mem_sb);
}

// Mount operation (registers with VFS)
int lfs_mount(struct file_system_type *fs_type, int flags, const char *dev_name, void *data)
{
    struct dentry *root;
    struct super_block *sb;

    sb = sget(fs_type, NULL, set_anon_super, flags, NULL);
    if (IS_ERR(sb))
        return PTR_ERR(sb);

    if (lfs_fill_super(sb, data, 0)) {
        deactivate_locked_super(sb);
        return -EINVAL;
    }

    root = d_make_root(lfs_get_inode(sb, LFS_ROOT_INO)); // LFS_ROOT_INO must be defined
    if (!root) {
        deactivate_locked_super(sb);
        return -ENOMEM;
    }
    sb->s_root = root;
    return 0;
}

// Unmount operation (clean up and flush)
void lfs_unmount(struct super_block *sb)
{
    lfs_update_superblock(sb); // Ensure superblock is written before unmount
    kfree(sb->s_fs_info);
    sb->s_fs_info = NULL;
    deactivate_super(sb);
}

// Module initialization
static int __init lfs_init(void)
{
    return register_filesystem(&lfs_type); // lfs_type must be defined elsewhere
}

// Module exit
static void __exit lfs_exit(void)
{
    unregister_filesystem(&lfs_type);
}

module_init(lfs_init);
module_exit(lfs_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Lion's File System (LFS)");
MODULE_AUTHOR("Lion");