<div align="center">
  <h1>🦁 LionFS</h1>
  <p><b>The Ultimate, Cryptographically-Secure, Self-Healing Filesystem for Linux</b></p>
  
  [![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
  [![FUSE](https://img.shields.io/badge/Filesystem-FUSE-blue?style=for-the-badge)](https://github.com/libfuse/libfuse)
  [![Security](https://img.shields.io/badge/Encryption-ChaCha20-red?style=for-the-badge)](#security)
  
  <p>
    Built with ❤️ by <a href="https://github.com/lionxlover">lion (lionxlover)</a>
  </p>
</div>

<hr>

## 📖 Table of Contents

- [Introduction](#-introduction)
- [Core Philosophy](#-core-philosophy)
- [Key Features](#-key-features)
- [Architecture & Design](#-architecture--design)
  - [Block Allocation](#block-allocation)
  - [Superblock Integrity](#superblock-integrity)
  - [Inode Structure](#inode-structure)
  - [POSIX Compliance](#posix-compliance)
- [Advanced Cryptography](#-advanced-cryptography)
- [Performance Optimization](#-performance-optimization)
- [Getting Started](#-getting-started)
  - [Prerequisites](#prerequisites)
  - [Building from Source](#building-from-source)
  - [Mounting the Filesystem](#mounting-the-filesystem)
  - [Unmounting](#unmounting)
- [Supported Operations](#-supported-operations)
- [Developer Guide](#-developer-guide)
- [Roadmap](#-roadmap)
- [Contributing](#-contributing)
- [License](#-license)
- [Acknowledgments](#-acknowledgments)

---

## 🌟 Introduction

Welcome to **LionFS**, the state-of-the-art, next-generation user-space filesystem built completely in Rust using FUSE (Filesystem in Userspace). Created specifically for modern Linux systems by **lion** (*GitHub: [lionxlover](https://github.com/lionxlover)*), LionFS brings enterprise-grade data protection to everyday workstations.

LionFS isn't just another filesystem. It is an exploration into merging the blazing speed of `ext4`, the advanced integrity checks of `ZFS`, and the transparent security of encrypted vaults into a single, cohesive, lightweight binary. 

Whether you are storing sensitive corporate data, running virtual machines, or just looking for a ridiculously over-engineered backend for your daily driver, LionFS is designed to ensure zero data corruption, zero security breaches, and zero headaches.

---

## 🧠 Core Philosophy

LionFS was built around three non-negotiable principles:

1. **Security is Not an Add-on:** It's baked into the very blocks on the disk. Every single byte written to LionFS is encrypted at rest using ChaCha20, with random nonces assigned at the block level.
2. **Data is Sacred:** Hardware fails. Disks get bit rot. Power outages happen. LionFS implements CRC32 integrity checks on every block and maintains a self-healing dual-superblock architecture.
3. **Safety Through Rust:** Written in 100% safe Rust where possible, eliminating buffer overflows, dangling pointers, and data races that plague legacy C-based filesystems.

---

## ✨ Key Features

- 🔒 **Transparent Encryption:** Instant, zero-configuration ChaCha20 stream-cipher encryption on all file contents and metadata.
- 🛡️ **Cryptographic Nonces:** The filesystem allocates 16-bytes per physical block exclusively for a true random nonce (`/dev/urandom`), completely preventing stream-cipher reuse attacks, even during in-place metadata modifications.
- 🧬 **Self-Healing Superblocks:** The filesystem maintains two isolated superblocks at the extreme boundaries of the disk image. If the primary superblock gets corrupted, LionFS automatically fails over and restores from the backup.
- 🔄 **Copy-on-Write (CoW):** Modifying a file doesn't overwrite its existing data. Data is written to new blocks, and atomic pointer swaps ensure you never lose data during a power failure.
- ✅ **CRC32 Checksums:** Every block pointer is paired with a CRC32 checksum. If a cosmic ray flips a bit on your SSD, LionFS knows about it immediately and will throw an `EIO` error rather than serving you corrupt data.
- 📁 **POSIX Compliance:** Supports creating files, making directories, truncating, `mv` renaming (both intra-directory and cross-directory), and symbolic links (`ln -s`).
- ⚡ **Zero-Overhead Truncation:** Highly optimized file truncation capabilities that cleanly free unused block bitmaps and avoid disk space leaks.

---

## 🏗 Architecture & Design

LionFS uses a highly customized block architecture mapped on top of a standard loopback image file.

### Block Allocation
The filesystem treats the underlying storage as a continuous array of `4096-byte` blocks. However, to accommodate cryptographic security, the *Payload Size* is restricted to `4080-bytes`. The first 16 bytes of every single physical block are reserved for internal filesystem mechanics (primarily the encryption nonce).

Blocks are allocated via an incredibly fast Bitmap Allocator occupying Block #1.

### Superblock Integrity
Block `0` is the Primary Superblock.
Block `N-1` is the Backup Superblock.
They contain the magic signature `0x4C494F4E` (LION), block sizes, and free-block tracking. Whenever a block is allocated or freed, both superblocks are kept in perfect atomic sync.

### Inode Structure
Each Inode in LionFS takes up exactly 256 bytes, allowing us to pack 15 Inodes into a single physical block.
An Inode keeps track of:
- Metadata (`mtime`, `ctime`, `atime`, `size`, `mode`, `uid`, `gid`)
- 12 Direct Block Pointers + 12 Checksums (for extreme speed on small files)
- 1 Indirect Block Pointer (for files larger than 48KB, pointing to a block of 510 extra pointers)

### POSIX Compliance
LionFS implements the `fuser` traits seamlessly. We handle directory parsing dynamically (at 63 `DirEntry` objects per payload block). File paths can be up to 56 bytes per segment, ensuring deep hierarchical directory trees.

---

## 🔐 Advanced Cryptography

Unlike standard filesystems which require complex tools like LUKS, LionFS encrypts *everything* internally using `ChaCha20`. 

During earlier iterations, a static block ID was used as a nonce. However, to guarantee absolute security against chosen-ciphertext and stream-cipher replay attacks on in-place metadata structures, **LionFS V5** revolutionized the physical layout:
1. Every time a block is written (be it a new data block or a modified inode table), 12 bytes are pulled from Linux's `/dev/urandom`.
2. This true-random data is stored at the physical head of the 4096-byte boundary.
3. The remaining 4080 bytes of payload are stream-ciphered using the generated nonce and the master key.
4. If a block is completely empty (zeroes), encryption is bypassed to allow the underlying SSD TRIM commands and sparse files to operate efficiently.

---

## 🏎 Performance Optimization

- **Single-Allocation CoW:** When modifying existing files, LionFS's intelligent `write` endpoint intercepts the Copy-on-Write flow, fetching data without allocating, and then explicitly allocating a single destination block to prevent dual-allocation overhead.
- **Lazy Checksumming:** Hashes are generated at the block level just before flushing to disk, maintaining high memory throughput.
- **Padded Structs:** All in-memory structs (`DiskInode`, `Superblock`) leverage the `bytemuck` crate, meaning memory representation perfectly matches disk representation. Deserialization overhead is completely non-existent (`O(1)` cost to cast bytes to structs).

---

## 🚀 Getting Started

### Prerequisites
- Linux OS (Ubuntu, Arch, Debian, Fedora, etc.)
- Rust Toolchain (`cargo`, `rustc` version 1.70+)
- `libfuse3-dev` or `fuse3` installed on your system.
- `pkg-config`

```bash
# Ubuntu/Debian
sudo apt update && sudo apt install -y fuse3 libfuse3-dev pkg-config

# Arch Linux
sudo pacman -S fuse3 pkgconf
```

### Building from Source

Clone the repository and build using Cargo:

```bash
git clone https://github.com/lionxlover/lionfs.git
cd lionfs
cargo build --release
```

### Mounting the Filesystem

You don't need to format a partition. LionFS operates on image files, meaning you can store a LionFS vault anywhere!

1. **Create a mount directory:**
   ```bash
   mkdir -p /tmp/mnt_lionfs
   ```

2. **Run the Daemon:**
   *(If the image file `lfs.img` does not exist, LionFS will automatically create it and format it for you!)*
   ```bash
   RUST_LOG=info cargo run --release -- lfs.img /tmp/mnt_lionfs
   ```

3. **Enjoy!**
   Open a new terminal and interact with your filesystem:
   ```bash
   cd /tmp/mnt_lionfs
   mkdir top_secret
   echo "LionFS is awesome!" > top_secret/note.txt
   cat top_secret/note.txt
   mv top_secret/note.txt top_secret/renamed.txt
   ln -s top_secret/renamed.txt my_link.txt
   ```

### Unmounting

Because LionFS runs in userspace, you unmount it using `fusermount` (or `fusermount3` on newer systems):

```bash
fusermount3 -u /tmp/mnt_lionfs
# or
fusermount -u /tmp/mnt_lionfs
```
*(The daemon will cleanly flush its state and exit).*

---

## 🛠 Supported Operations

Currently, LionFS fully supports the following FUSE operations:
- `lookup`: Path resolution and entry lookup.
- `getattr`: Reading file permissions, sizes, and timestamps.
- `setattr`: Chmod, Chown, and Truncation.
- `readdir`: Directory listing (supports standard `.` and `..` mappings).
- `create`: Touching and creating new files.
- `mkdir`: Creating new directories.
- `unlink`: Deleting files and freeing blocks/inodes.
- `rmdir`: Safely removing empty directories.
- `read`: Streaming file data with transparent decryption and CRC validation.
- `write`: Block-aligned and unaligned writes with Copy-on-Write and transparent encryption.
- `rename`: Moving files atomically.
- `symlink` & `readlink`: Creating soft-links.

---

## 👨‍💻 Developer Guide

The codebase is split into three main logical components:

- **`inode.rs`**: Defines the data structures that represent the filesystem on-disk. This includes `DiskInode`, `DirEntry`, and `IndirectBlock`. It handles the 4080-byte `PAYLOAD_SIZE` mathematics.
- **`disk.rs`**: The Block Device Abstraction Layer. This module handles all I/O to the `.img` file. It wraps read/writes with `ChaCha20`, handles block/inode bitmap allocation, and manages Superblock healing.
- **`fs.rs`**: The FUSE implementation layer. This handles POSIX translation, mapping FUSE requests (`readdir`, `lookup`, `write`) down into `DiskManager` operations.
- **`main.rs`**: Entry point and environment setup.

---

## 🗺 Roadmap

LionFS is continuously evolving. Here is what is planned for future major releases:

- [ ] **V6: Extended Attributes (XATTR)** - Support for SELinux contexts and extended metadata.
- [ ] **V7: Double-Indirect Blocks** - Increasing the maximum file size from ~2MB to multiple gigabytes by implementing recursive indirect block pointers.
- [ ] **V8: Hardware Acceleration** - Binding the ChaCha20 and CRC32 algorithms to AES-NI and SIMD instructions for extreme throughput on NVMe drives.
- [ ] **V9: Zstd Compression** - Adding transparent Zstandard compression before the encryption layer to dramatically increase storage density.
- [ ] **V10: Snapshots** - Btrfs-style atomic snapshots of the entire filesystem state.

---

## 🤝 Contributing

Contributions are heavily encouraged! Since LionFS is a solo-developer project by **lion**, pull requests, bug reports, and feature suggestions are the lifeblood of this repository.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request on [GitHub](https://github.com/lionxlover/lionfs)

---

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information. 
LionFS believes in open-source, unrestricted technological advancement.

---

## 🙏 Acknowledgments

- Tremendous thanks to the **Rust** community for building an ecosystem where writing filesystems doesn't require a Ph.D in debugging segmentation faults.
- Shoutout to the **FUSE (Filesystem in Userspace)** project for bringing kernel-level capabilities to user-space applications.
- Created, maintained, and heavily over-engineered by **lion** ([@lionxlover](https://github.com/lionxlover)).

<div align="center">
  <br>
  <i>"It's not just a filesystem. It's a fortress."</i> - <b>lion</b>
</div>
