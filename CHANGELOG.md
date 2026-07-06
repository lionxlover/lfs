# Changelog

All notable changes to the LionFS project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Extensive, highly-modular project directory structure.
- Initial Phase 1 Extent-based filesystem layout.
- `mkfs-lfs`, `mount-lfs`, `fsck-lfs`, and `debug-lfs` utilities.
- Zero-copy metadata deserialization via `bytemuck`.
- Basic free-space bitmap allocator.
- Inline extent storage inside 256-byte inodes.
- Directory schema with dynamic entry sizing and alignments.
- FUSE daemon enabling `mkdir`, `echo`, `cat`, `mv`, and `rm` capabilities natively on Linux.

### Changed
- Refactored `lionfs-core` to split `disk`, `inode`, `allocator`, `fs`, `dir`, and `file` logic into distinct, scalable submodules.

## [0.1.0] - Initial Prototype
### Added
- Proof-of-concept initialization for LionFS logic testing.
