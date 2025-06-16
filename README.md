# 🦁 LFS - The Lion File System

[![License](https://img.shields.io/badge/License-GPLv2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)
[![Kernel Version](https://img.shields.io/badge/Kernel-5.x+-orange.svg)](#)
[![Language](https://img.shields.io/badge/Language-C-informational.svg)](#)

**LFS (Lion File System)** is a next-generation, high-performance, and resilient filesystem for Linux, engineered to surpass traditional filesystems like ext4, XFS, and Btrfs. It achieves this through a hyper-optimized, modular architecture and a complete suite of integrated, advanced features.

The core philosophy of LFS is achieving superior performance and absolute reliability through **deterministic, verifiable engineering**—strictly avoiding any AI, ML, or non-deterministic heuristics.

---

## 🚀 Unique Value Proposition

LFS is engineered to be the fastest, most reliable, and most versatile filesystem available through its integrated feature set:

*   **Adaptive I/O Optimization**: A real-time engine tunes caching, prefetch, and write-back based on deterministic workload analysis, maximizing throughput for any task.
*   **Multi-Tier Journaling**: A high-speed metadata journal operates alongside an optional, high-throughput data journal, ensuring tunable levels of data integrity without compromising speed.
*   **Self-Tuning Allocation**: The hybrid extent and bitmap allocator dynamically adjusts its strategy to minimize fragmentation for both large and small files.
*   **Zero-Copy Direct I/O**: A first-class I/O path allows high-throughput applications like databases to bypass the kernel page cache entirely.
*   **Integrated Data Integrity**: Background scrubbing, multi-level checksums, and redundant metadata are not afterthoughts—they are fundamental to the design.
*   **Complete Feature Suite**: Snapshots, transparent compression, online resizing, and quotas are all built-in, modular, and can be managed at mount-time.

---

## ✨ Integrated Feature Set

LFS is a feature-complete system. All components are designed to work in concert to deliver unparalleled performance and reliability.

| Category | Feature | Status |
| :--- | :--- | :---: |
| **Core Architecture** | Hybrid Extent + Bitmap Allocator | ✅ |
| | Concurrent Directory B+Tree | ✅ |
| | Configurable Block Size (512B-64KB) | ✅ |
| **Data Integrity** | Multi-Tier WAL Journaling (Metadata + Data) | ✅ |
| | Background Data Scrubbing & Checksums (CRC32/SHA256) | ✅ |
| | Atomic, Copy-on-Write Snapshots & Rollback | ✅ |
| | Redundant Superblocks & Metadata Logs | ✅ |
| **Performance** | Adaptive Caching & Predictive Prefetch Engine | ✅ |
| | Zero-Copy Direct I/O Path | ✅ |
| | Live, Policy-Driven Defragmentation & Reclustering | ✅ |
| **Management** | Online Filesystem Resizing (Grow & Shrink) | ✅ |
| | User, Group, and Project Quotas | ✅ |
| | Transparent Compression (LZ4/Zstd per-file/dir) | ✅ |
| | Multi-Device Aggregation (RAID-0/1/5) | ✅ |

---

## 🔗 System Architecture

LFS employs a clean, layered architecture where the VFS glue code interacts with a powerful set of core modules. All features are managed through a unified config and mount-time flag system.

```
[ VFS Layer ]
    | mount/umount
    v
[ mount.c ] --> parses options, sets feature flags
    |
    +--> [ super.c ]  --> loads primary or backup superblock
    +--> [ config.c ] --> applies runtime configuration
    v
+----------------------- [ CORE LFS ENGINE ] ----------------------+
|                                                                  |
|  - inode.c    (Metadata Caching)  - quota.c      (Quotas)        |
|  - dir_tree.c (B+Tree Directories) - snap.c       (Snapshots)     |
|  - alloc.c    (Hybrid Allocator)    - compress.c   (Compression)   |
|  - block.c    (I/O Scheduler)     - resize.c     (Online Resizing) |
|  - journal.c  (Multi-Tier WAL)      - raid.c       (RAID Logic)      |
|  - cache.c    (Adaptive Cache)      - scrub.c      (Scrubbing)       |
|  - defrag.c   (Defragmentation)                                    |
|                                                                  |
+------------------------------------------------------------------+
```

---

## 🛠️ Userland Toolchain

LFS is managed by a complete and powerful set of command-line utilities.

| Tool | Description |
| :--- | :--- |
| **`mkfs.lfs`** | Formats a device with a highly configurable LFS layout. |
| **`fsck.lfs`** | Performs fast journal recovery and deep integrity verification. |
| **`lfs-info`** | Displays detailed statistics, fragmentation levels, and feature flags. |
| **`lfs-dump`** | Provides a low-level structural dump for debugging. |
| **`lfs-defrag`** | Manually triggers or configures the background defragmentation daemon. |
| **`lfs-scrub`** | Manages the background data scrubbing process to ensure integrity. |
| **`lfs-snapshot`** | Creates, lists, mounts, and rolls back atomic filesystem snapshots. |
| **`lfs-resize`** | Grows or shrinks a mounted LFS filesystem. |
| **`lfs-raid`** | Configures and manages multi-device LFS volumes. |

---

## ⚙️ Getting Started (Development & Testing)

This guide is for developers looking to build and test the LFS suite.

### 1. Prerequisites
You need the kernel headers for your running kernel and standard build tools.
```bash
# On Debian/Ubuntu
sudo apt-get update && sudo apt-get install build-essential linux-headers-$(uname -r)

# On Fedora/CentOS
sudo dnf install kernel-devel kernel-headers && sudo dnf groupinstall "Development Tools"
```

### 2. Build from Source
From the root project directory, a single `make` command builds everything.
```bash
make
```

### 3. Test on a Loopback Device (Safe Method)
**Never test on a real partition unless you intend to wipe it.**
```bash
# 1. Use a helper script to create a disk image (e.g., 1GB)
tools/mkdisk.sh lfs.img 1G

# 2. Format the image with LFS (enable compression for this example)
sudo userland/mkfs.lfs --features=compress,checksums lfs.img

# 3. Load the LFS kernel module
sudo insmod kernel/lfs.ko

# 4. Mount the filesystem
sudo tools/mount_image.sh lfs.img /mnt/lfs

# 5. Verify and use it!
df -hT /mnt/lfs
sudo userland/lfs-info /mnt/lfs
echo "Welcome to the Lion File System!" | sudo tee /mnt/lfs/welcome.txt

# 6. Unmount and unload when done
sudo umount /mnt/lfs
sudo rmmod lfs
```
---

## 🤝 Contributing

We are actively looking for contributors who are passionate about systems programming, performance, and building robust software. If you're interested, please:
1.  Read the `docs/design.md` and `docs/format.md` to understand the architecture.
2.  Fork the repository and create a feature branch.
3.  Open an issue to discuss your proposed changes or bug fixes.
4.  Submit a pull request with clean, well-commented code and accompanying tests.

## ⚖️ License

This project is licensed under the **GNU General Public License v2.0**, in compliance with the Linux kernel's license.