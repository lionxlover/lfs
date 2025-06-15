# 🦁 LFS - The Lion File System

[![License](https://img.shields.io/badge/License-GPLv2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)
[![Kernel Version](https://img.shields.io/badge/Kernel-5.x+-orange.svg)](#)
[![Language](https://img.shields.io/badge/Language-C-informational.svg)](#)

Lion's File System (LFS) is a high-performance, from-scratch filesystem for Linux, built with a focus on **reliability, and deterministic behavior**. It implements a journal-first architecture to ensure metadata consistency, making it resilient against system crashes and power failures.

The core philosophy of LFS is to create a robust and predictable system using classical computer science principles, **strictly avoiding AI, ML, or non-deterministic heuristics.**

---

## ⚠️ Current Status: Alpha / Educational

**WARNING:** This filesystem is in an **alpha, educational stage**. It is an excellent tool for learning about kernel development and filesystem internals, but it has not undergone the rigorous testing required for a production environment.

**DO NOT USE LFS ON A SYSTEM WITH IMPORTANT DATA.** You risk data loss. Use it exclusively on loopback devices or dedicated, disposable test partitions.

---

## ✨ Core Features

| Feature                         | Description                                                                                             | Status |
| ------------------------------- | ------------------------------------------------------------------------------------------------------- | :----: |
| **🛡️ Metadata Journaling**      | A Write-Ahead Log ensures metadata consistency and fast recovery from power loss or system crashes.     |   ✅   |
| **🚀 Full Kernel Implementation** | Runs directly in the kernel (no FUSE) for maximum performance and full VFS integration.                 |   ✅   |
| **📦 Large File Support**       | A multi-level block mapping system (direct, indirect, double-indirect) supports files up to several GB. |   ✅   |
| **🔗 POSIX Compliance**          | Supports standard permissions (`rwx`), ownership (`uid`/`gid`), file types, and hard/symbolic links.      |   ✅   |
| **🛠️ Complete Toolset**          | Includes userland utilities (`mkfs`, `fsck`, `lfs-dump`) for managing the filesystem.                   |   ✅   |
| **📄 Deterministic by Design**   | All recovery and maintenance operations are based on predictable, verifiable algorithms.                |   ✅   |

---

## ⚙️ Getting Started & Building

You can test LFS by creating a loopback device, formatting it, and mounting it.

### 1. Prerequisites

You will need the kernel headers for your currently running kernel and standard build tools.

```bash
# On Debian/Ubuntu
sudo apt-get update
sudo apt-get install build-essential linux-headers-$(uname -r)

# On Fedora/CentOS/RHEL
sudo dnf install kernel-devel kernel-headers
sudo dnf groupinstall "Development Tools"
```

### 2. Build the Project

From the root project directory, simply run `make`. This will build the kernel module and all userland utilities.

```bash
make
```

After a successful build, you will find:
- `kernel/lfs.ko` - The kernel module.
- `userland/mkfs.lfs` - The formatting tool.
- `userland/fsck.lfs` - The check/repair tool.
- `userland/lfs-dump` - The debug tool.

### 3. Create and Mount a Filesystem

Follow these steps to create a 512MB image file and mount it as an LFS filesystem.

```bash
# 1. Create an empty image file (512MB)
dd if=/dev/zero of=lfs.img bs=1M count=512

# 2. Format the image with LFS
# This requires sudo for writing a random UUID.
sudo userland/mkfs.lfs lfs.img

# 3. Load the LFS kernel module
sudo insmod kernel/lfs.ko

# 4. Create a mount point
mkdir -p /mnt/lfs

# 5. Mount the filesystem
# Note: A "tainted kernel" message in dmesg is normal when loading out-of-tree modules.
sudo mount -t lfs lfs.img /mnt/lfs

# 6. Verify that it's mounted!
echo "Hello LFS!" | sudo tee /mnt/lfs/welcome.txt
cat /mnt/lfs/welcome.txt
df -h /mnt/lfs
```

### 4. Unmount and Cleanup

```bash
# Unmount the filesystem
sudo umount /mnt/lfs

# Unload the kernel module
sudo rmmod lfs

# You can now safely remove the image file and mount point
rm lfs.img
rmdir /mnt/lfs
```

---

## 🗺️ Project Roadmap

This is the plan for future development, building on the current stable foundation.

-   [ ] **🚀 Smart Defragmentation:** Implement an idle-time and manual `lfs-defrag` tool.
-   [ ] **🛡️ Block Checksums:** Add CRC32 checks to all metadata and data blocks for self-healing capabilities.
-   [ ] **📈 User/Group Quotas:** Integrate quota support into the VFS hooks.
-   [ ] **💨 Sparse File Support:** Optimize storage for files with large empty regions.
-   [ ] **🔒 Immutable & Append-Only Flags:** Implement `chattr`-like file flags for added security.
-   [ ] **⚡ Delayed Allocation:** Improve write performance and reduce fragmentation.

---

## 🤝 Contributing

Contributions are welcome! This project is a great place to learn about low-level systems programming.

1.  **Fork** the repository.
2.  Create a new **branch** (`git checkout -b feature/YourAmazingFeature`).
3.  **Commit** your changes (`git commit -m 'Add some YourAmazingFeature'`).
4.  **Push** to the branch (`git push origin feature/YourAmazingFeature`).
5.  Open a **Pull Request**.

Please open an issue first to discuss any major changes you would like to make.

## ⚖️ License

This project is licensed under the **GNU General Public License v2.0**. This license is required for modules to interact with the core Linux kernel subsystems. See the `LICENSE` file for more details.

## 👤 Author

**LFS** was created and designed by **Lion**.