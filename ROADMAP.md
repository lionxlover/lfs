# LionFS Roadmap

LionFS is currently in **Phase 1: Core Foundation**. This document details the long-term vision and timeline for integrating advanced storage features into LionFS.

## Phase 1: Core Foundation (Current)
- [x] Basic Extent-Based Layout
- [x] Little-Endian On-Disk Format
- [x] Contiguous Block Allocator
- [x] 256-Byte Inodes with Inline Extents
- [x] FUSE Daemon Integration
- [x] `mkfs-lfs`, `fsck-lfs`, `debug-lfs`

## Phase 2: Structural Scaling
- [ ] **B-Tree Extent Management**: Expand beyond the 7 inline extents limit to support highly fragmented and massive files via a B-Tree structure.
- [ ] **Journaling**: Implement Write-Ahead Logging (WAL) to prevent corruption during unexpected shutdowns.
- [ ] **Cross-Directory Renames**: Implement a safe, atomic locking schema for moving files across directories.
- [ ] **Extended Attributes (xattr)**: Add support for ACLs and standard POSIX attributes natively within inodes or overflow blocks.

## Phase 3: Modern Data Safety
- [ ] **Snapshots (Copy-on-Write)**: Transition metadata structures to support zero-cost point-in-time snapshots.
- [ ] **Data Checksumming**: End-to-end CRC32/xxHash validation of all metadata and file blocks.
- [ ] **Compression**: Transparent Zstandard (Zstd) compression at the block level.
- [ ] **Encryption**: Built-in ChaCha20 / AES-XTS block-level encryption utilizing physical nonces.

## Phase 4: Enterprise Scalability
- [ ] **De-duplication**: Offline and optionally online deduplication of matching extents.
- [ ] **Multi-Device Support**: Native RAID levels (mirroring, striping) spanning across multiple physical block devices.
- [ ] **Self-Healing Scrubbing**: Background routines to repair degraded/corrupted extents automatically.
- [ ] **Kernel Module**: Transition the stable `lionfs-core` to a Linux VFS-compatible Kernel Module (`.ko`) for extreme performance.
