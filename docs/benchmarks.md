# LionFS 1.0 Benchmarking Methodology

LionFS treats performance as a foundational requirement, equal in importance to data integrity. To guarantee that regressions are never merged into stable code, we have developed a dual-layer benchmarking architecture.

## 1. Microbenchmarking (Internal)
For internal engine performance, LionFS utilizes the `criterion` framework. This operates on a micro-scale to measure nanosecond and picosecond latency profiles of internal structs.

**Key Coverage Areas:**
- **B+Tree Operations (`btree_bench.rs`)**: Measures raw insertion, node splitting, and recursive rebalancing iterations. Tests ensure that memory-aligned nodes (`std::ptr::copy_nonoverlapping`) maintain zero false-sharing during concurrent multi-threaded writes.
- **Allocation & Caching (`allocator_bench.rs`)**: Validates the ring-buffer speeds of the `PerCpuAllocatorCache`. It specifically measures lock-contention latency when 256+ threads rapidly request free-space block mappings simultaneously.
- **Asynchronous I/O (`io_bench.rs`)**: Simulates the `write_blocks_parallel` multi-threaded async dispatch pipeline, guaranteeing that our batched `FileExt` (pread/pwrite) mechanisms maintain dynamic queue depths at minimal overhead.

**How to Run:**
```bash
# Run the internal benchmark suite
cargo bench
```

## 2. Macrobenchmarking (CLI Tools)
For real-world storage hardware profiling, LionFS ships with `lfs_benchmark`.

Unlike `fio` or `sysbench` which run on top of the VFS layer, `lfs_benchmark` communicates directly with the LionFS engine. It measures physical IOPS and Latency against the storage block device using the LionFS transaction manager natively.

**Features of `lfs_benchmark`:**
- Evaluates raw NVMe SSD performance via direct block alignments.
- Tests multi-device scaling algorithms (RAID 0 Striping latency vs. RAID 1 Mirroring latency).
- Emits fully deterministic JSON output designed specifically for CI/CD Grafana pipelines.

**How to Run:**
```bash
# Execute a simulated random-write test on a mounted volume
sudo lfs_benchmark --mode=randwrite --block-size=4k --threads=64 /mnt/lion
```

## Continuous Profiling
In addition to the static benchmarks, LionFS utilizes `lfs_profile` to attach eBPF tracing and generate flame graphs on-the-fly, giving enterprise administrators unprecedented visibility into memory bottlenecks and kernel VFS integration times.
