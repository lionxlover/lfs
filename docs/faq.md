# LFS - Frequently Asked Questions

This document answers common questions about the Lion File System (LFS). It is divided into sections for General, Technical, and Development-related questions.

---

## ❔ General Questions

#### Q1: What is LFS?
LFS (Lion File System) is a new, high-performance, and feature-rich filesystem for Linux. It is designed from the ground up to be extremely fast, reliable, and secure, incorporating modern features like snapshots, data checksumming, and transparent compression directly into its core design.

#### Q2: Why create another Linux filesystem?
LFS was created to address the compromises inherent in existing filesystems. For example, users often have to choose between the raw performance of XFS and the data integrity features of Btrfs/ZFS. LFS aims to provide **both** in a single, cohesive package, without trade-offs. Its core philosophy is to achieve this through superior, deterministic engineering rather than complex heuristics or AI.

#### Q3: Is LFS stable? Can I use it for my data?
**NO.** LFS is currently a development project. While it is being built to the highest standards of reliability, it has not yet undergone the years of real-world testing required for production use. **DO NOT use LFS for any critical data.** It is intended for testing, benchmarking, and development purposes only.

#### Q4: How is LFS licensed?
LFS is licensed under the **GNU General Public License v2.0 (GPLv2)**. This is necessary for it to be a part of the Linux kernel ecosystem.

---

## ⚙️ Technical Questions

#### Q1: What makes LFS fast?
LFS is designed for speed at every level of its architecture:
1.  **Concurrent Data Structures:** B+Trees are used for directories, allowing many processes to access them at once without blocking.
2.  **Hybrid Allocator:** An extent-based allocator drastically reduces file fragmentation, which is key for fast sequential reads.
3.  **Adaptive Caching:** A smart, deterministic engine tunes its caching and prefetching strategy based on real-time workload patterns.
4.  **Zero-Copy I/O:** A dedicated path for high-throughput applications to bypass kernel buffers and write directly to disk.

#### Q2: How does LFS protect against data corruption?
LFS has a multi-layered defense against data corruption, specifically "bit rot":
1.  **End-to-End Checksums:** LFS stores a checksum (e.g., CRC32c) for *every single data and metadata block*.
2.  **Read-Time Verification:** Whenever a block is read, its checksum is automatically recalculated and verified. If it doesn't match, an error is immediately returned, preventing corrupt data from ever reaching your applications.
3.  **Background Scrubbing:** A low-priority background process (`lfs-scrubd`) periodically scans the entire disk, verifying all checksums to proactively find and report silent errors.

#### Q3: What are "atomic snapshots"? How do they work?
Atomic snapshots are instantaneous, point-in-time images of your filesystem. LFS uses a **Copy-on-Write (COW)** mechanism. When you create a snapshot, LFS doesn't copy any data. Instead, when a block is about to be changed, the original data is preserved for the snapshot, and the new data is written to a new, free block. This makes creating snapshots instant and very space-efficient.

#### Q4: What does "deterministic, no AI/ML" mean in practice?
It means that every decision the filesystem makes is based on a clear, verifiable algorithm. For example, the adaptive cache follows a set of predefined rules: "IF the I/O stream is sequential for N blocks, THEN increase the readahead buffer to X." It is not "learning" or "predicting" in a non-deterministic way. This ensures that LFS behaves consistently and predictably, which is a critical attribute for core system software.

#### Q5: Can I resize an LFS filesystem?
Yes. LFS supports **online resizing**, meaning you can both grow and shrink a mounted filesystem without any downtime.

---

## 🧑‍💻 Development Questions

#### Q1: I want to contribute. Where do I start?
Welcome! We are excited to have you. The best way to start is:
1.  Read the `docs/design.md` and `docs/format.md` to get a deep understanding of the architecture.
2.  Follow the `docs/build.md` guide to get the project compiled and running on a test image.
3.  Look at the open issues on GitHub, especially those tagged with `good first issue` or `help wanted`.
4.  Fork the repository, create a dedicated branch for your feature or bugfix, and submit a pull request.

#### Q2: What are the coding standards for the project?
We follow the standard **Linux kernel coding style**. Before submitting code, please ensure it adheres to these conventions (indentation, naming, commenting, etc.). The script `scripts/checkpatch.pl` from the kernel source tree is a good tool for checking your code.

#### Q3. How do I test my changes?
The project includes a growing test suite in the `tests/` directory.
-   If you modify a specific subsystem (e.g., the allocator), add or update the relevant **unit tests**.
-   For larger changes, create a new **integration test** script that verifies the end-to-end functionality.
-   All changes must pass the full test suite (`./tools/test_runner.sh`) before a pull request will be merged.

#### Q4: Where can I ask more questions or discuss development ideas?
Please use the **Issues** tab on GitHub. You can open a new issue to ask a question, propose a new feature, or discuss an implementation detail. This keeps the conversation public and archived for others to reference.