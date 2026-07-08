# LionFS 1.0 Architecture Overview

LionFS is a high-performance, extent-based filesystem written in Rust, designed for extreme scalability, data integrity, and multi-device capabilities. 

## 1. Core Principles
- **End-to-End Data Integrity**: Uses hardware-accelerated CRC32C / BLAKE3 checksums on all Superblocks, Inodes, and Extent payloads to detect and heal silent data corruption automatically.
- **Copy-on-Write (CoW)**: Fully implements CoW semantics using B+Tree node replication (root-node replication), facilitating instantaneous zero-cost snapshots and writable clones.
- **Crash-Safe Transactions**: A robust dual-journaling pipeline handles metadata and data writes, ensuring complete crash consistency via atomic checkpointing.
- **Micro-Optimized Performance**: 1.0 Stable brings lock-free asynchronous I/O parallel pipelines (via `rayon`), per-CPU slab allocator caching, and fine-grained concurrent metadata handling to achieve unparalleled throughput.

## 2. B+Tree Metadata Engine
At its heart, LionFS relies on a unified, high-performance B+Tree engine for metadata isolation:
- **Inode Tree**: Manages file attributes and metadata lookups.
- **Directory Tree**: Scales to millions of entries inside a single directory without degradation.
- **Extent Tree**: Maps physical clusters on disk to logical offsets in files.
- **Allocation Tree**: Handles block bitmaps and high-speed allocation lookups.
- **Checksum Tree**: Isolates CRC maps for on-the-fly verification.

Each tree is strictly memory-aligned to hardware cache boundaries, eliminating false-sharing and ensuring rapid iterations during heavy transactional loads.

## 3. Storage Pools and Volume Management (RAID)
LionFS moves beyond single-disk constraints by integrating multi-device block awareness seamlessly at the I/O layer:
- Support for **Single, RAID 0, RAID 1, and RAID 10**.
- Read mappings utilize round-robin striping for high IOPS.
- Write mappings mirror blocks concurrently using the `write_blocks_parallel` multi-threaded async dispatcher, pushing hardware queues to maximum depths on NVMe devices.
- Devices can be dynamically added or removed with background balancing.

## 4. Sub-systems
- **AI-Assisted Optimization Engine**: Lightweight statistical models in userspace monitor file access patterns to proactively cache sequential streams before they are explicitly requested.
- **Compression & Deduplication**: Real-time multi-threaded Zstd and LZ4 compression.
- **Encryption**: AES-256-GCM native hardware-accelerated block encryption natively embedded in the VFS pipeline.

## 5. Security & Upgradability
- Implements `FeatureNegotiator` for rolling, backwards-compatible deployments using `RO_COMPAT` and `INCOMPAT` flags.
- Strong permission enforcement modeled directly against POSIX.

By synthesizing these technologies, LionFS offers enterprise-class reliability with the sheer speed of raw block processing.
