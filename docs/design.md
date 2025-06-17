# LFS Architecture & Design

This document provides a high-level overview of the Lion File System (LFS) architecture. It is intended for developers who wish to understand the internal workings of the filesystem, its core subsystems, and the design principles that govern their interactions.

## 1. Core Philosophy: Deterministic High Performance

The fundamental principle of LFS is that peak performance and absolute reliability can be achieved through superior, deterministic engineering, without resorting to complex heuristics or AI. Every architectural choice is a reflection of this philosophy.

This has led to a design that is:
*   **Modular:** Subsystems are independent and communicate through well-defined APIs.
*   **Concurrent:** Built from the ground up for modern multi-core CPUs and high-IOPS NVMe devices.
*   **Resilient:** Data integrity is not a feature but a prerequisite, integrated at every layer.
*   **Scalable:** Data structures are chosen to scale from small embedded devices to multi-petabyte storage arrays.

## 2. System Architecture Diagram

LFS can be visualized as a series of interacting layers, with the Linux VFS at the top and the block device at the bottom. The core engine is a collection of powerful, specialized subsystems.

```
+-------------------------------------------------------------+
|                     User Application (ls, cp, etc.)         |
+-------------------------------------------------------------+
|                   Linux Virtual Filesystem (VFS)            |
+--------------------------+----------------------------------+
                           | (VFS API Calls)
                           v
+--------------------------+----------------------------------+
|                   LFS VFS Glue Layer (fs.c)                   |
| (Translates VFS calls into LFS-specific operations)         |
+-------------------------------------------------------------+
|                LFS Core Engine & Subsystems                 |
|                                                             |
| +-----------+  +-----------+  +-----------+  +------------+ |
| |  INODE    |  |  DIR B+TREE|  |  ALLOC    |  |  JOURNAL   | |
| |  MANAGER  |->|  MANAGER  |->|  (Hybrid) |->|  (Multi-Tier)| |
| +-----------+  +-----------+  +-----------+  +------------+ |
|      ^              ^              ^              ^          |
|      |              |              |              |          |
| +----+------+  +----+------+  +----+------+  +----+-------+ |
| | ADAPTIVE  |  |   BLOCK   |  |  SCRUB &  |  | SNAPSHOT   | |
| |  CACHE &  |<-|    I/O    |<-|  CHECKSUM |<-|  ENGINE    | |
| | PREFETCH  |  |  ENGINE   |  |  MANAGER  |  |  (COW)     | |
| +-----------+  +-----------+  +-----------+  +------------+ |
|                                                             |
+-------------------------------------------------------------+
                           | (Block I/O Requests)
                           v
+--------------------------+----------------------------------+
|                  Linux Block Layer / I/O Scheduler          |
+-------------------------------------------------------------+
|                       Physical Block Device                   |
+-------------------------------------------------------------+
```

## 3. Core Subsystems Explained

### 3.1. The Allocator (`alloc/`)
*   **Goal:** Fast, intelligent, and fragmentation-resistant allocation of blocks and inodes.
*   **Design:** LFS uses a **hybrid extent and bitmap allocator**.
    *   **Bitmap:** A simple, fast bitmap (`bitmap.c`) tracks the overall state (free/used) of every block and inode. This is used for quick checks of space availability.
    *   **Free Extent Tree:** For allocating blocks for file data, LFS uses a B+Tree (`extent_tree.c`) that indexes contiguous free-space regions (extents). This allows the allocator to instantly find the best-fit contiguous block range for a write, drastically reducing fragmentation for large files.
    *   **Hybrid Logic:** For very small allocations, or when the disk is heavily fragmented, the allocator can fall back to a simpler first-available strategy using the block bitmap.

### 3.2. Directory Management (`dir_tree.c`)
*   **Goal:** Support massive directories (millions of entries) with high-speed, concurrent lookups, creations, and deletions.
*   **Design:** A custom, highly-concurrent **B+Tree** implementation.
    *   **Keys:** Keys in the tree are 64-bit hashes of filenames, ensuring balanced distribution.
    *   **Leaf Nodes:** Contain the actual `lfs_dir_entry` structures, sorted by hash. This allows for fast lookups and efficient directory traversals (`readdir`).
    *   **Concurrency:** Fine-grained locking on B+Tree nodes allows multiple processes to read from and write to the same directory simultaneously with minimal contention, a significant advantage over traditional linear directory structures.

### 3.3. Journaling & Recovery (`journal/`)
*   **Goal:** Guarantee metadata consistency and provide tunable data consistency without significant performance loss.
*   **Design:** A **multi-tier, circular Write-Ahead Log (WAL)**.
    *   **Metadata Journal:** A small, dedicated region of the disk for logging changes to inodes, bitmaps, and extent trees. It is optimized for extremely low latency and is always active.
    *   **Data Journal (Optional):** A larger journal region that can be enabled at mount-time (`data=journal`). When active, it logs both metadata and file data content, providing the same level of safety as ext3's data journaling mode.
    *   **Atomic Transactions:** All related changes are wrapped in a transaction with a `START` and `COMMIT` record. Recovery (`replay.c`) is a simple and fast process of scanning the journal for complete transactions and applying them.

### 3.4. Data Integrity (`scrub/`, `checksum.c`)
*   **Goal:** Provide absolute protection against silent data corruption (bit rot).
*   **Design:** A unified **checksum and scrubbing framework**.
    *   **Checksum Tree:** A global B+Tree stores a checksum (configurable CRC32c or SHA256) for every allocated data block in the filesystem. This is a metadata overhead, but it provides end-to-end integrity.
    *   **On-the-Fly Verification:** On every read, the block's checksum is recalculated and verified. A mismatch returns an error, preventing corrupt data from ever reaching an application.
    *   **Background Scrubbing:** A low-priority background daemon (`lfs-scrubd`) systematically reads all data on the disk and verifies it against the checksum tree, proactively finding and reporting errors.

### 3.5. Snapshot Engine (`snap.c`)
*   **Goal:** Provide instantaneous, low-overhead, point-in-time snapshots of the entire filesystem.
*   **Design:** An integrated **Copy-on-Write (COW)** mechanism at the block level.
    *   **On Snapshot:** A snapshot command simply records the current root of the filesystem metadata and creates a new snapshot entry. This operation is nearly instantaneous.
    *   **On Write:** When a data or metadata block is about to be modified, LFS first checks if it is part of a previous snapshot. If so, it writes the changes to a *new* block (COW) and updates the parent pointers. The original block is left untouched, preserved for the snapshot.
    *   **Space Efficiency:** Snapshots only consume space for the blocks that have changed since the snapshot was taken.

### 3.6. Adaptive Cache (`cache/`, `prefetch.c`)
*   **Goal:** Intelligently use system memory to minimize physical I/O and accelerate performance.
*   **Design:** A **deterministic, policy-based adaptive cache**.
    *   **Workload Analysis:** LFS monitors I/O streams to classify them (e.g., highly sequential, small random, large random).
    *   **Dynamic Prefetching:** For sequential streams, the prefetcher aggressively reads ahead. For random streams, it reduces or disables prefetching to avoid polluting the cache with useless data.
    *   **Cache Prioritization:** Blocks are prioritized in the cache based on access frequency and recentness, ensuring that "hot" data remains in memory. This entire process is rule-based, not heuristic.