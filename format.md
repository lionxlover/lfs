# Lion File System (LFS) - On-Disk Format Specification v1.0

## 1. Overview

This document specifies the on-disk layout for the Lion File System (LFS). All multi-byte integer fields are stored in **little-endian** format. The design prioritizes extreme performance, scalability, and data integrity.

The default block size is **4096 bytes**, but it is configurable at format time.

## 2. Global Filesystem Layout

LFS divides the storage device into several key metadata regions and a large data region. The superblock, at a fixed location, acts as the primary anchor, containing pointers to the roots of all other data structures.

```
+-------------------------------------------------------------------------------------------------+
| Block 0     | Block 1       | Backup SBs    | Feature/Config | Journal (Meta) | Journal (Data)   |
| Boot Sector | Superblock    | (Redundant)   | Block          | (Fast, Small)  | (Optional, Large)|
+-------------------------------------------------------------------------------------------------+
                                     |
                                     V
... +---------------------------------------------------------------------------------------------+
    | Inode Bitmap | Block Bitmap | Free Extent Tree | Inode Table | Checksum Tree | Snapshot Meta  |
+-------------------------------------------------------------------------------------------------+
                                     |
                                     V
... +---------------------------------------------------------------------------------------------+
    | B+Tree Nodes (Directories, Extents)             | Data Blocks (File Contents)               |
    | (Dynamically allocated)                         | (The rest of the volume)                  |
    +---------------------------------------------------------------------------------------------+
```

## 3. The Superblock (`lfs_superblock_t`)

Located at a fixed offset (e.g., 4KB, Block 1). Contains pointers to the roots of all major filesystem structures. Backup copies are stored at fixed intervals for redundancy.

| Offset | Size | Field | Description |
| :--- | :--- | :--- | :--- |
| 0 | 4 | `s_magic` | Magic number: **`0x1F510F5D`** ("LIONFSD"). |
| 4 | 4 | `s_version` | `__le32` Filesystem format version. |
| 8 | 4 | `s_block_size` | `__le32` Block size in bytes. |
| 12 | 8 | `s_feature_flags` | `__le64` Bitmask of enabled features (checksums, snapshots, etc.). |
| 20 | 8 | `s_blocks_count` | `__le64` Total number of blocks. |
| 28 | 8 | `s_inodes_count` | `__le64` Total number of inodes. |
| 36 | 16 | `s_uuid` | `__u8[16]` Unique 128-bit identifier for the volume. |
| 52 | 4 | `s_state` | `__le32` Filesystem state: `1` (Clean), `2` (Dirty), `3` (Replaying). |
| 56 | 8 | `s_root_dir_block` | `__le64` Block address of the root of the Directory B+Tree. |
| 64 | 8 | `s_inode_bitmap_block` | `__le64` Start block of the inode allocation bitmap. |
| 72 | 8 | `s_block_bitmap_block` | `__le64` Start block of the block allocation bitmap. |
| 80 | 8 | `s_free_extent_tree_root` | `__le64` Block address of the root of the Free Extent B+Tree. |
| 88 | 8 | `s_inode_table_block` | `__le64` Start block of the main inode table. |
| 96 | 8 | `s_journal_meta_start` | `__le64` Start block of the high-speed metadata journal. |
| 104 | 4 | `s_journal_meta_blocks` | `__le32` Size of the metadata journal. |
| 108 | 8 | `s_journal_data_start` | `__le64` Start block of the optional data journal. |
| 116 | 4 | `s_journal_data_blocks` | `__le32` Size of the data journal. |
| 120 | 8 | `s_snapshot_meta_block` | `__le64` Start block of the snapshot metadata area. |
| 128 | 8 | `s_checksum_tree_root` | `__le64` Block address of the root of the global checksum tree. |
| ... | ... | `s_padding` | Reserved for future pointers. Padded to block size. |

## 4. Inode and Extent-Based Allocation

LFS uses an **extent-based** model for file allocation, which is highly efficient for both large and small files. An extent is a contiguous range of blocks.

### Inode Structure (`lfs_inode_t`)
Contains all metadata for a file object. If a file has more extents than can fit in the inode, `i_extent_root` points to the root of a dedicated B+Tree for that file's extents.

| Field | Type | Description |
| :--- | :--- | :--- |
| `i_mode` | `__le16` | File type and POSIX permissions. |
| `i_links_count` | `__le16` | Hard link count. |
| `i_uid`, `i_gid` | `__le32` | User and Group ID. |
| `i_size` | `__le64` | File size in bytes. |
| `i_atime`, `mtime`, `ctime`, `btime` | `__le64` | All four standard timestamps. |
| `i_flags` | `__le64` | Flags: `COMPRESSED`, `IMMUTABLE`, `ENCRYPTED`, `NO_SCRUB`, etc. |
| `i_blocks_count` | `__le64` | Number of blocks allocated. |
| `i_checksum` | `__le32` | CRC32 checksum of this inode structure itself. |
| `i_extent_count` | `__le32` | Number of extents stored directly in the inode. |
| `i_direct_extents` | `lfs_extent_t[4]` | Space for 4 direct extents for small files. |
| `i_extent_root` | `__le64` | Block address of the root of this inode's extent B+Tree. |

### Extent Structure (`lfs_extent_t`)
A simple structure representing a contiguous run of blocks.
```c
typedef struct lfs_extent {
    __le64  ex_logical_start;   // First logical block in the file.
    __le64  ex_physical_start;  // First physical block on disk.
    __le32  ex_length;          // Number of blocks in this extent.
    __le32  ex_flags;           // Flags (e.g., preallocated, unwritten).
} lfs_extent_t;
```

## 5. Directory B+Tree

To handle directories with millions of entries efficiently, LFS uses a B+Tree instead of a linear list.

*   **Internal Nodes**: Contain keys (hashes of filenames) and pointers to child nodes.
*   **Leaf Nodes**: Contain the actual directory entries, sorted by filename hash.

### Directory Entry (`lfs_dir_entry_t`)
Located in the leaf nodes of the B+Tree.
```c
typedef struct lfs_dir_entry {
    __le64  d_inode_num;      // Inode number of the entry.
    __u8    d_name_len;       // Length of the filename.
    __u8    d_file_type;      // File type (for convenience).
    char    d_name[];         // Variable-length filename.
} lfs_dir_entry_t;
```

## 6. Journaling and Recovery

LFS features a **multi-tier write-ahead journal** to provide tunable data consistency guarantees.

*   **Metadata Journal**: A small, extremely fast circular log for all metadata changes (inodes, bitmaps, extents). This is always active.
*   **Data Journal**: An optional, larger circular log that can be enabled (`data=journal`) for full data and metadata journaling, guaranteeing data content at the cost of performance.

### Journal Transaction Header (`lfs_txn_header_t`)
Every transaction in either journal begins with this header.
```c
typedef struct lfs_txn_header {
    __le32  th_magic;         // Transaction magic number.
    __le32  th_type;          // Type: START, COMMIT, REVOKE.
    __le64  th_tid;           // Unique Transaction ID.
    __le32  th_num_blocks;    // Number of data blocks following this header.
    __le32  th_checksum;      // CRC32 of the entire transaction.
} lfs_txn_header_t;
```
A transaction is only considered valid if a `COMMIT` block with a matching `th_tid` is present.

## 7. Data Integrity: Checksums & Scrubbing

LFS integrates data integrity at a fundamental level. A global, COW B+Tree (`s_checksum_tree_root`) stores checksums for all data blocks in the filesystem.

*   **On Write**: When a data block is written, its checksum (e.g., CRC32c or SHA256) is calculated and stored in the checksum tree.
*   **On Read**: The block is read, its checksum is re-calculated and verified against the stored value. A mismatch triggers an I/O error and can trigger a recovery action if redundancy (RAID) is available.
*   **Scrubbing**: A background process (`lfs-scrubd`) periodically reads blocks and verifies their checksums to detect silent corruption.

## 8. Snapshots

LFS supports instantaneous, copy-on-write (COW) snapshots.

*   When a snapshot is created, the current root of the filesystem metadata (superblock, inode table, etc.) is preserved.
*   When a metadata or data block is modified, instead of overwriting it, a new copy is written to a free block (COW). The parent structures are updated to point to the new copy.
*   The original block is kept, referenced only by the snapshot.
*   Snapshot metadata is stored in a dedicated log starting at `s_snapshot_meta_block`, containing information like snapshot name, timestamp, and a pointer to its root metadata block.