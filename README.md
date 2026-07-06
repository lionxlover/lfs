# LionFS (Lion Filesystem)

![LionFS Logo](https://via.placeholder.com/800x200.png?text=LionFS+-+Modern+Rust+Filesystem)

**LionFS** is a modern, extent-based Linux filesystem written natively in Rust. Designed to scale from tiny NVMe instances up to exabyte-scale clusters, LionFS bridges the gap between everyday simplicity (ext4) and enterprise durability (ZFS).

## Phase 1 (Alpha)
LionFS is currently in **Phase 1: Core Foundation**. It is capable of mounting as a VFS under Linux (via FUSE), persisting basic extents, and providing raw directory capabilities. It is entirely functional as a prototype.

### Features (Phase 1)
- Extent-based contiguous block mapping
- Zero-copy native `bytemuck` deserialization
- Free-space bitmap allocation
- Fully aligned 256-byte Inodes
- FUSE Daemon (`mount-lfs`)
- Robust tooling (`mkfs-lfs`, `fsck-lfs`, `debug-lfs`)

## Getting Started

### Dependencies
- Rust Toolchain (`cargo`)
- `pkg-config`
- `libfuse3-dev`

### Formatting an Image
```bash
cargo run --bin mkfs-lfs -- disk.img 100
```

### Mounting the Filesystem
```bash
mkdir -p /mnt/lionfs
cargo run --bin mount-lfs -- disk.img /mnt/lionfs
```

## Documentation
Check out the `/docs` and `/specifications` folders in the repository to learn more about the on-disk format and modular architecture of LionFS.

## Author
Built by [lionxlover](https://github.com/lionxlover).
