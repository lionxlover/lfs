lfs/
├── kernel/
│   ├── lfs.h                     # Core FS structs and constants
│   ├── super.c                   # Superblock handling logic
│   ├── inode.c                   # Inode structure and operations
│   ├── dir_tree.c                # Directory B+Tree logic
│   ├── block.c                   # Block I/O operations
│   ├── alloc/
│   │   ├── alloc.c               # Hybrid extent + bitmap allocator
│   │   ├── bitmap.c              # Block/inode bitmap handling
│   │   └── extent_tree.c         # Extent tree logic
│   ├── journal/
│   │   ├── journal.c             # Write-ahead logging
│   │   ├── replay.c              # Journal replay engine
│   │   └── journal.h             # Journal API and types
│   ├── defrag/
│   │   ├── defrag.c              # Manual and background defrag
│   │   └── defrag_policy.c       # Non-AI fragmentation strategy
│   ├── scrub/
│   │   ├── scrub.c               # Background scrubbing
│   │   └── checksum.c            # CRC32/SHA256 checksum logic
│   ├── cache/
│   │   ├── cache.c               # Read/write caching engine
│   │   └── prefetch.c            # Adaptive prefetch system
│   ├── compress/
│   │   ├── compress.c            # Optional LZ4/Zstd support
│   │   └── api.h                 # Abstract compression API
│   ├── snap.c                    # Snapshot & rollback logic
│   ├── quota.c                   # Quota engine
│   ├── resize.c                  # Online resizing
│   ├── raid.c                    # Optional RAID features
│   └── fs.c                      # Entry point, registration hooks
│
├── include/
│   ├── lfs_fs.h                  # Filesystem-wide types and macros
│   ├── lfs_config.h              # Compile-time options
│   └── kernel_compat.h           # Linux kernel compatibility macros
│
├── userland/
│   ├── mkfs/
│   │   └── mkfs.lfs.c            # Format tool
│   ├── fsck/
│   │   └── fsck.lfs.c            # Check and repair tool
│   ├── defrag/
│   │   ├── lfs-defrag.c          # Manual defragmenter
│   │   └── lfs-defragd.c         # Background daemon
│   ├── info/
│   │   └── lfs-info.c            # Filesystem stats
│   ├── dump/
│   │   └── lfs-dump.c            # Dump raw filesystem structures
│   ├── recover/
│   │   └── lfs-recover.c         # Manual journal recovery
│   ├── snapshot/
│   │   └── lfs-snapshot.c        # Create/rollback snapshot
│   ├── resize/
│   │   └── lfs-resize.c          # Resize utility
│   ├── scrub/
│   │   └── lfs-scrub.c           # Trigger integrity check
│   └── raid/
│       └── lfs-raid.c            # Manage software RAID mode
│
├── tools/
│   ├── test_runner.sh            # Automate test suites
│   ├── mount_image.sh            # Loop-mount helper
│   └── mkdisk.sh                 # Create blank disk image
│
├── docs/
│   ├── format.md                 # Complete on-disk layout spec
│   ├── build.md                  # Kernel + userland build instructions
│   ├── design.md                 # High-level architecture
│   ├── perf_targets.md           # Benchmark goals & results
│   └── faq.md                    # Technical Q&A
│
├── tests/
│   ├── unit/
│   │   ├── test_inode.c          # Inode API tests
│   │   ├── test_alloc.c          # Allocation strategy tests
│   │   └── test_journal.c        # Journal write/replay tests
│   ├── integration/
│   │   ├── fs_create_mount.sh    # Full loop-mount lifecycle
│   │   └── defrag_benchmark.sh   # Defrag timing test
│   └── fuzz/
│       └── fuzz_blocks.c         # Fuzz tests for data corruption
│
├── examples/
│   ├── sample_config.json        # Config options template
│   └── sample_mount.sh           # Mount LFS with options
│
├── Makefile                      # Build system (kernel + userland)
├── Kbuild                        # Kernel module integration
└── README.md                     # Project overview
