#ifndef LFS_FORMAT_H
#define LFS_FORMAT_H

#include <linux/types.h> // Use kernel types for kernel code

// Magic number to identify the LFS filesystem ("LFS\0")
#define LFS_MAGIC 0x4C465300U

// Filesystem state flags
#define LFS_STATE_CLEAN 0x00
#define LFS_STATE_DIRTY 0x01

// Maximum filename length (for fixed-size dir entries, variable for others)
#define LFS_NAME_MAX 255

// Inode block pointers
#define LFS_NDIR_BLOCKS 12
#define LFS_IND_BLOCK   12
#define LFS_DIND_BLOCK  13
#define LFS_N_BLOCKS    14 // 12 direct, 1 indirect, 1 double-indirect

// Superblock structure (packed for on-disk layout)
struct lfs_superblock {
    __le32 magic;             // Magic number
    __le32 version;           // Filesystem version
    __le32 block_size;        // Size of each block in bytes
    __le32 total_blocks;      // Total number of blocks
    __le32 free_blocks;       // Number of free blocks
    __le32 total_inodes;      // Total number of inodes
    __le32 free_inodes;       // Number of free inodes
    __le64 journal_start;     // Start block of the journal
    __le32 journal_size;      // Size of the journal in blocks
    __le32 state;             // Clean/dirty state
    __u8   uuid[16];          // Unique identifier (UUID)
    __le32 checksum;          // CRC32 of superblock (excluding this field)
    __le32 reserved[13];      // Padding for future expansion (64B aligned)
} __attribute__((packed));

// Inode structure (packed for on-disk layout)
struct lfs_inode {
    __le16 mode;              // File mode (permissions + type)
    __le16 flags;             // Immutable, append-only, etc.
    __le32 uid;               // Owner user ID
    __le32 gid;               // Owner group ID
    __le64 size;              // File size in bytes
    __le64 atime;             // Last access time (UNIX timestamp)
    __le64 mtime;             // Last modification time
    __le64 ctime;             // Last status change time
    __le32 blocks[LFS_N_BLOCKS]; // Block pointers (direct/indirect)
    __le32 links_count;       // Hard link count
    __le32 generation;        // Inode generation (for NFS, etc.)
    __le32 checksum;          // CRC32 of inode (excluding this field)
    __le32 reserved[6];       // Padding for future expansion
} __attribute__((packed));

// Directory entry structure (packed, variable-length name)
struct lfs_dir_entry {
    __le32 inode;             // Inode number
    __le16 rec_len;           // Directory entry length
    __u8   name_len;          // Length of file name
    __u8   file_type;         // File type (regular, dir, symlink, etc.)
    char   name[LFS_NAME_MAX];// File name (not null-terminated)
} __attribute__((packed));

// File types for directory entries
#define LFS_FT_UNKNOWN  0
#define LFS_FT_REG_FILE 1
#define LFS_FT_DIR      2
#define LFS_FT_SYMLINK  3

// Function prototypes for reading/writing superblock (kernel-space)
int lfs_read_superblock(struct super_block *sb, struct lfs_superblock *disk_sb);
int lfs_write_superblock(struct super_block *sb, const struct lfs_superblock *disk_sb);

// Function prototypes for inode management (kernel-space)
int lfs_read_inode(struct super_block *sb, uint32_t ino, struct lfs_inode *disk_inode);
int lfs_write_inode(struct super_block *sb, uint32_t ino, const struct lfs_inode *disk_inode);

#endif // LFS_FORMAT_H