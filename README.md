# LionFS (LFS) 🦁

Welcome to **LionFS (LFS)** — a next-generation, high-performance, extent-based Linux filesystem written entirely in **Rust**. Built by **lion** (GitHub: [@lionxlover](https://github.com/lionxlover)), LionFS is designed to combine the crash-safe reliability of modern journaling filesystems, the petabyte-scale capability of enterprise storage solutions, and the memory safety guarantees of Rust.

[![Build Status](https://github.com/lionxlover/lfs/actions/workflows/rust.yml/badge.svg)](https://github.com/lionxlover/lfs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)

---

## 🌟 Vision & Design Philosophy

LionFS was born from a desire to rethink filesystem architecture from the ground up, avoiding the legacy technical debt of older filesystems while learning from the architectural triumphs of ZFS, ReFS, and ext4.

By implementing LionFS in Rust, we completely eliminate entire classes of memory safety bugs—buffer overflows, use-after-free errors, and data races—which have historically plagued kernel-space filesystem implementations in C.

### Key Goals:
1. **Safety First**: Utilize Rust's borrow checker to ensure memory safety and concurrency safety.
2. **Crash-Safe Reliability**: Implement robust Write-Ahead Logging (WAL) and metadata journaling. A sudden power loss should never corrupt the filesystem metadata.
3. **Massive Scalability**: Scale seamlessly to petabytes of storage and billions of files through Allocation Groups (AGs) and extent-based tracking.
4. **Blazing Performance**: Leverage high-performance concurrent metadata caches (`moka`), lock-free data structures, and asynchronous background flushers.
5. **Modern Architecture**: Incorporate advanced concepts like delayed allocation, inline extents, and adaptive block groups.

---

## 🚀 Key Features

### 📦 1. Extent-Based Allocation
Instead of mapping every single block individually, LionFS uses **extents** (contiguous runs of blocks). This dramatically reduces metadata overhead for large files and significantly improves sequential read/write speeds, particularly on HDDs and NVMe drives.

### 🛡️ 2. Crash-Safe Metadata Journaling
LionFS implements a full **Write-Ahead Log (WAL) Transaction Manager**. All metadata operations are atomic. If the system panics or loses power mid-write, LionFS recovers the journal in milliseconds upon remount, ensuring the filesystem is always consistent.

### ⚡ 3. High-Performance Caching Layer
LionFS features a sophisticated caching architecture:
- **Inode Cache**: Caches active inodes for O(1) metadata access.
- **Directory Cache**: Caches directory entries for instant pathname resolution.
- **Extent Cache**: Caches block mappings to prevent costly on-disk lookups during sequential reads.

### 🧩 4. Allocation Groups (AGs)
The filesystem is divided into independent **Allocation Groups**. This localization minimizes lock contention during parallel allocations and keeps related inodes and data blocks close together on disk, drastically reducing seek times on rotational media and maximizing parallelism on NVMe arrays.

### 🔄 5. Delayed Allocation & Background Flushing
Writes can be aggressively cached in memory and flushed asynchronously by the `BackgroundFlusher` worker threads. This allows LionFS to intelligently merge and organize writes into larger, contiguous extents before committing them to the physical disk.

### 🛠️ 6. Userspace (FUSE) & Kernel Ready
Currently, LionFS runs as a high-performance **FUSE (Filesystem in Userspace)** module for ease of development, testing, and debugging. The core storage engine is completely decoupled from FUSE, paving the way for a future native Linux Kernel Module (`vfs`) implementation.

---

## 🏗️ Architecture Overview

The LionFS architecture is cleanly separated into distinct layers:

```text
+-------------------------------------------------------------+
|                      VFS / FUSE Layer                       |
|   (Translates OS requests into LionFS operations)           |
+-------------------------------------------------------------+
                              |
+-------------------------------------------------------------+
|                   Cache & Concurrency Layer                 |
|   (Moka-backed InodeCache, DirCache, ExtentCache)           |
+-------------------------------------------------------------+
                              |
+-------------------------------------------------------------+
|                  Transaction & Journal Layer                |
|   (Atomic Commits, WAL, Crash Recovery, Checkpointing)      |
+-------------------------------------------------------------+
                              |
+-------------------------------------------------------------+
|                 Storage & Allocation Engine                 |
|   (BlockGroupDescriptors, Extent Allocator, Bitmap)         |
+-------------------------------------------------------------+
                              |
+-------------------------------------------------------------+
|                      Physical Disk I/O                      |
|                 (Raw Block Device Access)                   |
+-------------------------------------------------------------+
```

### On-Disk Format
LionFS uses a stable, endian-independent on-disk format defined using `bytemuck`:
- **Superblock**: Contains filesystem geometry, magic number (`0x4C494F4E`), and UUIDs. Replicated across AGs.
- **Block Group Descriptors**: Tracks free space, inodes, and directories per AG.
- **Inodes**: 256-byte structures containing metadata (size, timestamps, ownership) and inline extent arrays.
- **Directories**: Variable-length hashed entries for fast lookups.

---

## 🛠️ Getting Started

### Prerequisites
- **Rust**: `rustc` and `cargo` (1.70 or newer).
- **FUSE**: `libfuse3-dev` (Linux) or `osxfuse` (macOS).

### Installation

Clone the repository:
```bash
git clone https://github.com/lionxlover/lfs.git
cd lfs
```

Build the project (includes the FS driver, `mkfs`, and `fsck`):
```bash
cargo build --release
```

### Formatting a Volume (`mkfs-lfs`)
Before mounting, you must format a block device or a sparse file with the LionFS format.

```bash
# Create a 1GB sparse loopback file for testing
dd if=/dev/zero of=lion.img bs=1M count=1024

# Format it as LionFS
cargo run --release --bin mkfs-lfs -- lion.img
```

### Mounting the Filesystem
Mount the formatted image to a mountpoint:

```bash
# Create mountpoint
mkdir -p /mnt/lion

# Mount the filesystem via FUSE
cargo run --release --bin lfs -- lion.img /mnt/lion
```

You can now read, write, and manipulate files in `/mnt/lion` just like any standard filesystem!

### Checking the Filesystem (`fsck-lfs`)
To verify filesystem integrity and check for errors:

```bash
# Ensure the filesystem is UNMOUNTED before running fsck
cargo run --release --bin fsck-lfs -- lion.img
```

---

## 📈 Benchmarks

LionFS includes a custom benchmarking suite (`bench-lfs`) to measure allocation throughput, cache hit rates, and raw I/O performance.

```bash
cargo run --release --bin bench-lfs
```

*Preliminary internal benchmarks show LionFS metadata operations via the Moka cache layer exceeding **26 million ops/sec** on modern hardware.*

---

## 🗺️ Roadmap & Future Phases

LionFS is under active development. The evolution is structured into distinct phases:

- [x] **Phase 1: Core Foundation**
  - Superblock, Extents, Block Allocator, Inodes, Directory Entries, FUSE Integration.
- [x] **Phase 2: Crash Recovery**
  - Transaction Manager, Metadata Journaling, Atomic Commits, Log Replay.
- [x] **Phase 3: High-Performance Engine**
  - Allocation Groups (AGs), Memory Caching (`moka`), Background Flushers, Multithreading.
- [ ] **Phase 4: Advanced Features (ZFS/ReFS-like capabilities)**
  - Copy-on-Write (CoW) Snapshots
  - Transparent Data Compression (LZ4/ZSTD)
  - Inline Data Deduplication
  - B-Tree based Extent Mapping for highly fragmented files
  - Software RAID / Multi-device pooling

---

## 🤝 Contributing

Contributions are highly welcome! Whether you're fixing bugs, optimizing the allocator, writing tests, or adding documentation, your help makes LionFS better.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

---

## 📜 License

Distributed under the MIT License. See `LICENSE` for more information.

---

## 👤 Author

**lion**
- GitHub: [@lionxlover](https://github.com/lionxlover)

*"Building the future of Linux storage, one block at a time."*
