# Lion's File System (LFS) - On-Disk Format Specification v1.0

## 1. Overview

This document specifies the definitive on-disk layout for the Lion File System (LFS). All multi-byte fields are stored in **little-endian** format, using standard Linux kernel types like `__le32` and `__le64` for clarity. The design prioritizes performance, reliability, and simplicity.

The default block size is **4096 bytes**.

## 2. Filesystem Layout

The filesystem is organized into a series of contiguous sections. The superblock, located at a fixed position, contains pointers to the dynamic sections (bitmaps, inode table).

```
+--------------------------------------------------------------------------+
| Block 0     | Block 1       | Journal Area        | Inode     | Block     |
| Boot Sector | Superblock    | (Size defined in    | Bitmap    | Bitmap    | ...
| (Unused)    | (Primary)     | the Superblock)     |           |           |
+--------------------------------------------------------------------------+
                                     |
                                     V
... +----------------------------------------------------------------------+
    | Inode Table                                     | Data Blocks        |
    | (Array of Inode structures)                     | (Remaining Space)  |
    +----------------------------------------------------------------------+
```

| Section | Location (Block Number) | Description |
| :--- | :--- | :--- |
| **Boot Sector** | Block 0 | Reserved for bootloaders. Unused by LFS. |
| **Superblock** | Block 1 | Contains critical filesystem metadata and pointers. |
| **Journal Area** | Starts at `s_journal_start_block` | A circular write-ahead log for metadata operations. |
| **Inode Bitmap** | Located at `s_inode_bitmap_block` | Tracks allocated/free inodes (1 bit per inode). |
| **Block Bitmap** | Located at `s_block_bitmap_block` | Tracks allocated/free data blocks (1 bit per block). |
| **Inode Table** | Starts at `s_inode_table_block` | A contiguous array of all inode structures. |
| **Data Blocks** | The remaining space on the volume | Stores file content and directory data. |

---

## 3. The Superblock (`struct lfs_super_block`)

The superblock is the root of the filesystem, located at the fixed **Block 1**. It is padded to 1024 bytes.

| Offset (bytes) | Size (bytes) | Field Name | Description |
| :--- | :--- | :--- | :--- |
| 0 | 4 | `s_magic` | Magic number: **`0x1F51F510`** ("LFS LION"). |
| 4 | 4 | `s_version` | Filesystem version (starts at 1). |
| 8 | 4 | `s_block_size` | Block size in bytes (e.g., 4096). |
| 12 | 8 | `s_blocks_count` | `__le64` Total number of blocks in the filesystem. |
| 20 | 8 | `s_inodes_count` | `__le64` Total number of inodes in the filesystem. |
| 28 | 8 | `s_free_blocks_count` | `__le64` Count of free blocks. |
| 36 | 8 | `s_free_inodes_count` | `__le64` Count of free inodes. |
| 44 | 4 | `s_state` | `__le32` Filesystem state: `1` (Clean), `2` (Dirty). |
| 48 | 16 | `s_uuid` | `__u8[16]` A 128-bit unique identifier for the volume. |
| 64 | 8 | `s_journal_start_block`| `__le64` Starting block of the journal area. |
| 72 | 4 | `s_journal_blocks` | `__le32` Number of blocks dedicated to the journal. |
| 76 | 8 | `s_inode_bitmap_block`| `__le64` Block number of the inode bitmap. |
| 84 | 8 | `s_block_bitmap_block`| `__le64` Block number of the block bitmap. |
| 92 | 8 | `s_inode_table_block` | `__le64` Starting block number of the inode table. |
| 100 | 4 | `s_inode_size` | `__le32` Size of a single inode structure (e.g., 256). |
| 104 | 920 | `s_padding` | Reserved for future use. Padded to 1024 bytes. |

---

## 4. Inodes and the Inode Table

The Inode Table is a simple, contiguous array of `lfs_inode` structures. An inode number is its 1-based index into this table.
- **Inode 1:** Reserved (traditionally for bad blocks).
- **Inode 2:** The root directory (`/`).

### Inode Structure (`struct lfs_inode`)
Each inode is 256 bytes. It contains all metadata for a file except its name.

| Offset | Size | Field Name | Description |
| :--- | :--- | :--- | :--- |
| 0 | 2 | `i_mode` | `__le16` File type and POSIX permissions (rwx). |
| 2 | 2 | `i_links_count` | `__le16` Number of hard links to this inode. |
| 4 | 4 | `i_uid` | `__le32` User ID of the owner. |
| 8 | 4 | `i_gid` | `__le32` Group ID of the owner. |
| 12 | 8 | `i_size` | `__le64` File size in bytes. |
| 20 | 8 | `i_atime` | `__le64` Last access time (seconds since epoch). |
| 28 | 8 | `i_ctime` | `__le64` Inode change time. |
| 36 | 8 | `i_mtime` | `__le64` File modification time. |
| 44 | 8 | `i_dtime` | `__le64` Deletion time (0 if not deleted). |
| 52 | 4 | `i_flags` | `__le32` File flags (e.g., immutable, append-only). |
| 56 | 8 | `i_blocks_count`| `__le64` Number of blocks allocated to this file. |
| 64 | 60 | `i_block[15]` | `__le32[15]` Block pointers (12 direct, 1 indirect, 1 double-indirect, 1 triple-indirect). |
| 124 | 132 | `i_padding` | Reserved. Padded to 256 bytes. |

### Inode Block Pointer Logic

The `i_block` array provides a tiered system to locate file data, enabling both small file efficiency and large file support.

```
lfs_inode.i_block[]
  +----------------------------------------------------------------------+
  | [0] ... [11]       | [12]               | [13]                       |
  | (Direct Pointers)  | (Indirect Pointer) | (Double Indirect Pointer)  |
  +--------------------+--------------------+----------------------------+
            |                    |                    |
            |                    |                    V
            |                    |           [Block of Indirect Pointers]
            |                    |                    |
            |                    |                    V
            |                    |           [Block of Data Pointers]
            |                    |                    |
            |                    |                    V
            |                    |               [Data Block]
            |                    |
            |                    V
            |           [Block of Data Pointers]
            |                    |
            |                    V
            |               [Data Block]
            |
            V
       [Data Block]
```
---

## 5. Directory Entries (`struct lfs_dir_entry`)

Directories are special files whose data blocks contain a linked list of `lfs_dir_entry` structures. This structure links a filename to an inode number.

| Offset | Size | Field Name | Description |
| :--- | :--- | :--- | :--- |
| 0 | 8 | `inode` | `__le64` Inode number for this entry (0 if unused). |
| 8 | 2 | `rec_len` | `__le16` Total length of this entry. Points to the start of the next entry. |
| 10 | 1 | `name_len` | `__u8` Length of the file name in bytes. |
| 11 | 1 | `file_type` | `__u8` File type (matches inode mode, for convenience). |
| 12 | 255 | `name` | `char[255]` File name (null-terminated). |

The `rec_len` field makes it easy to traverse the directory and skip over deleted entries, whose `inode` number is set to 0.

---

## 6. The Journal

The journal is a **circular write-ahead log** used to guarantee metadata consistency. Changes are written to the journal first; only after a transaction is committed is the metadata written to its final location.

### Journal Logic

The journal operates like a circular buffer with a `head` and `tail` pointer. New transactions are written at the `tail`. The `head` points to the oldest transaction that still needs to be committed to disk.

```
       <-- Log wraps around <--
+--------------------------------------------------------------------------+
|  | Committed Tx | Committed Tx | Oldest Live Tx (Head) | ... | New Tx (Tail) | Free Space |
+--------------------------------------------------------------------------+
   ^                                                                       |
   |_________________________________ Moves forward _______________________|
```

### Journal Transaction Structure

A single transaction in the journal consists of:
1.  A **Descriptor Block**: Marks the start of a transaction and lists the final on-disk locations of all metadata blocks that will be changed.
2.  **Metadata Block Copies**: One or more blocks containing the new versions of the metadata (e.g., a modified inode table block, a changed bitmap block).
3.  A **Commit Block**: A single, final block that marks the transaction as complete. If a crash occurs before the commit block is written, the entire transaction is considered invalid and is discarded during recovery.
