# LFS Performance Targets & Benchmarks

This document outlines the performance goals for the Lion File System (LFS). The primary objective of LFS is to deliver superior, predictable performance that surpasses existing filesystems across a wide range of workloads. All targets are set for a standard, high-performance NVMe SSD.

## 1. Core Philosophy

Performance in LFS is not just about achieving the highest possible number in a single, specific benchmark. It is about delivering **balanced, sustained, and predictable throughput and latency** under real-world conditions. Our targets reflect this philosophy.

## 2. Key Performance Indicators (KPIs)

These are the high-level metrics that define the performance profile of LFS.

| Metric | Target | Competitive Target | Notes |
| :--- | :--- | :--- | :--- |
| **Sequential Read** | **> 10 GB/s** | Surpass XFS | Large file reads, media streaming, backups. |
| **Sequential Write** | **> 6 GB/s** | Surpass XFS | Large file writes, video capture, data ingestion. |
| **Random Read 4K IOPS** | **> 1,500,000** | Surpass ext4/XFS | Database lookups, virtualization, small file access. |
| **Random Write 4K IOPS** | **> 1,000,000** | Surpass ext4/XFS | Database transactions, logging, metadata-heavy work. |
| **Metadata Operations** | **> 300,000 ops/s** | Surpass Btrfs/ext4 | File creation, deletion, `stat`, `ls -l` on large directories. |
| **`fsck` (Journal Replay)** | **< 500 ms** | Surpass ext4 | Time to recover a "dirty" 1TB filesystem after a simulated crash. |
| **Cache Hit Rate** | **> 98%** | N/A | Target for mixed workloads, measured internally. |

## 3. Detailed Performance Targets

### 3.1. Throughput (Large Files)
*   **Target:** Achieve near-hardware-saturation speeds for large, multi-gigabyte sequential I/O.
*   **Test:** `fio` benchmark with `bs=1M`, `iodepth=128`, `rw=read/write`.
*   **Goal:** Demonstrate the efficiency of the extent-based allocator and the I/O engine's ability to cluster writes.

### 3.2. Latency (Small Files & Metadata)
*   **Target:** Minimize the time-to-completion for small, random I/O and metadata operations.
*   **Test:** `fio` with `bs=4k`, `iodepth=1`, `rw=randread/randwrite`.
*   **Goal:** Keep 99.9th percentile latency below **100 microseconds (µs)** for random reads on NVMe. This showcases the efficiency of the B+Tree structures and the low-overhead I/O path.

### 3.3. Scalability (Directory Operations)
*   **Target:** Maintain high performance even with massive directories.
*   **Test:** Create 1 million files in a single directory. Measure the time to create, `stat`, and delete these files.
*   **Goal:** The time to `stat` the 1,000,000th file should be nearly identical to the time to `stat` the 1st file, demonstrating the O(log n) efficiency of the directory B+Tree.

### 3.4. Feature Performance
*   **Snapshot Creation:** Must be effectively instantaneous (< 1 ms), regardless of filesystem size.
*   **Defragmentation Throughput:** The online defragmenter should achieve a re-clustering speed of **> 1 GB/s** on an idle system.
*   **Compression/Decompression:** For compressible data, achieve a throughput of **> 2 GB/s** using LZ4, showcasing minimal CPU overhead.

## 4. Benchmarking Methodology

To ensure fair and reproducible results, all official LFS benchmarks will adhere to the following methodology:

1.  **Hardware:** A standardized test bench with a high-performance Gen4 NVMe SSD (e.g., Samsung 980 Pro) and a modern multi-core CPU.
2.  **Tooling:** The primary tool for I/O benchmarking will be `fio`. Metadata benchmarks will use tools like `mdtest`.
3.  **Filesystem State:** Tests will be run on a filesystem that is 75% full to simulate realistic usage and fragmentation.
4.  **Comparison:** LFS will be benchmarked against the latest stable versions of `ext4`, `xfs`, and `btrfs` on the exact same hardware with their default mount options.
5.  **Transparency:** All benchmark scripts, `fio` profiles, and raw results will be published in the `tests/benchmarks` directory of the repository.

## 5. Current Performance Status (Template)

This section will be updated as the project matures and official benchmarks are run.

*(This is a template for future results)*

| Benchmark | LFS Result | ext4 Result | XFS Result | Btrfs Result | Status |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Seq Read (GB/s)** | 9.8 | 9.5 | **10.1** | 9.2 | 🟡 **Nearly Met** |
| **Rand 4K Read IOPS**| **1.6M** | 1.2M | 1.4M | 1.1M | 🟢 **Target Met** |
| **Metadata Ops/s**| 280k | 250k | 220k | 180k | 🟡 **Nearly Met** |
| **`fsck` time (ms)** | **410** | 480 | 450 | N/A | 🟢 **Target Met** |