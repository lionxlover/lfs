#include <linux/fs.h>
#include <linux/slab.h>
#include <linux/string.h>
#include <linux/uaccess.h>
#include "lfs.h"
#include "dir.h"
#include "inode.h"
#include "journal.h"

// Create a new directory entry
int lfs_create_dir(struct inode *parent_inode, const char *name) {
    struct inode *new_inode;
    struct dir_entry *new_entry;
    int err;

    // Allocate a new inode for the directory
    new_inode = lfs_alloc_inode();
    if (!new_inode) {
        return -ENOMEM;
    }

    // Initialize the new inode as a directory
    new_inode->i_mode = S_IFDIR | 0755; // Directory with rwxr-xr-x permissions
    new_inode->i_size = 0;
    new_inode->i_blocks = 0;
    new_inode->i_atime = new_inode->i_mtime = new_inode->i_ctime = current_time(new_inode);

    // Add the new directory entry to the parent directory
    new_entry = kmalloc(sizeof(struct dir_entry), GFP_KERNEL);
    if (!new_entry) {
        lfs_free_inode(new_inode);
        return -ENOMEM;
    }

    strncpy(new_entry->name, name, NAME_MAX);
    new_entry->inode_no = new_inode->i_ino;

    err = lfs_add_entry(parent_inode, new_entry);
    if (err) {
        kfree(new_entry);
        lfs_free_inode(new_inode);
        return err;
    }

    // Update the journal with the new directory creation
    lfs_journal_add_entry(new_inode, new_entry);

    return 0;
}

// Read directory entries
int lfs_read_dir(struct inode *dir_inode, struct dir_entry *entries, int max_entries) {
    int count = 0;
    struct dir_entry *entry;

    // Iterate through the directory entries
    for (entry = dir_inode->i_dir_entries; entry && count < max_entries; entry = entry->next) {
        entries[count++] = *entry; // Copy entry to the output array
    }

    return count; // Return the number of entries read
}

// Remove a directory entry
int lfs_remove_dir(struct inode *parent_inode, const char *name) {
    struct dir_entry *entry;
    int err;

    // Find the directory entry to remove
    entry = lfs_find_entry(parent_inode, name);
    if (!entry) {
        return -ENOENT; // Entry not found
    }

    // Remove the entry from the parent directory
    err = lfs_remove_entry(parent_inode, entry);
    if (err) {
        return err;
    }

    // Free the inode associated with the entry
    lfs_free_inode(entry->inode_no);
    kfree(entry);

    // Update the journal with the directory removal
    lfs_journal_remove_entry(parent_inode, entry);

    return 0;
}

// Traverse the directory
struct dir_entry *lfs_find_entry(struct inode *dir_inode, const char *name) {
    struct dir_entry *entry;

    // Iterate through the directory entries to find the specified name
    for (entry = dir_inode->i_dir_entries; entry; entry = entry->next) {
        if (strcmp(entry->name, name) == 0) {
            return entry; // Entry found
        }
    }

    return NULL; // Entry not found
}