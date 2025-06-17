# Building and Testing the Lion File System (LFS)

This guide provides all the necessary steps to compile the LFS kernel module and its complete userland toolchain from source. It also outlines the recommended procedure for safely testing the filesystem on a loopback device.

## 1. Prerequisites

To build LFS, you need a Linux environment with the standard C development toolchain and the kernel headers corresponding to your currently running kernel.

### 1.1. Install Required Packages

**On Debian / Ubuntu / Mint:**
```bash
sudo apt-get update
sudo apt-get install build-essential linux-headers-$(uname -r) git
```

**On Fedora / CentOS / RHEL:**
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install kernel-devel kernel-headers git
```

### 1.2. Verify Your Setup
Ensure your kernel headers are correctly installed. The following command should execute without errors:
```bash
ls /lib/modules/$(uname -r)/build
```

## 2. Compiling the Project

The project uses a top-level `Makefile` that orchestrates the build process for both the kernel module and all userland utilities.

### 2.1. Clone the Repository
First, clone the official LFS repository:
```bash
git clone https://github.com/your-username/lfs.git
cd lfs
```

### 2.2. Run the Build
From the root of the project directory, execute `make`:
```bash
make
```

This single command will:
1.  Recursively descend into the `kernel/` directory and build the `lfs.ko` kernel module.
2.  Recursively descend into the `userland/` directory and build all associated tools (`mkfs.lfs`, `fsck.lfs`, etc.).

### 2.3. Build Output
After a successful build, the compiled artifacts will be located in their respective source directories:
*   **Kernel Module:** `kernel/lfs.ko`
*   **Userland Tools:** `userland/mkfs/mkfs.lfs`, `userland/fsck/fsck.lfs`, and so on.

### 2.4. Cleaning the Build
To remove all compiled objects and executables, run:
```bash
make clean
```

## 3. Safe Testing Methodology

**WARNING:** Never test a development filesystem on a physical partition with important data. The recommended and safest method is to use a **loopback device**, which is an image file that the kernel treats as a block device.

We provide helper scripts in the `tools/` directory to simplify this process.

### Step 1: Create a Disk Image
Use the `mkdisk.sh` script to create a blank image file. This example creates a 2 GB image.
```bash
./tools/mkdisk.sh lfs.img 2G
```

### Step 2: Format the Image with LFS
Use the newly compiled `mkfs.lfs` tool to format the image. This command enables the `checksums` and `compression` features as an example.
```bash
sudo userland/mkfs/mkfs.lfs --features=checksums,compression lfs.img
```

### Step 3: Load the Kernel Module
Insert the compiled `lfs.ko` module into the kernel:
```bash
sudo insmod kernel/lfs.ko
```
You can verify it's loaded with `lsmod | grep lfs`.

### Step 4: Mount the Filesystem
Use the `mount_image.sh` helper script, which handles the `losetup` and `mount` commands for you.
```bash
# Create a mount point first
mkdir -p /mnt/lfs

# Mount the image
sudo ./tools/mount_image.sh lfs.img /mnt/lfs
```

### Step 5: Use and Verify
The filesystem is now live. You can interact with it and use the info tools.
```bash
# Check mount status
df -hT /mnt/lfs

# Check LFS-specific features
sudo userland/info/lfs-info /mnt/lfs

# Standard file operations
echo "LFS is running!" | sudo tee /mnt/lfs/test.txt
cat /mnt/lfs/test.txt
```

### Step 6: Unmount and Cleanup
When you are finished, unmount the filesystem and unload the module.
```bash
# Unmount
sudo umount /mnt/lfs

# Unload the kernel module
sudo rmmod lfs

# You can now safely remove the image file if desired
# rm lfs.img
```

## 4. Advanced Build Options & Debugging

The build system can be configured for debugging purposes.

### Building with Debug Symbols
To include debugging information (`-g` flag) in the kernel module and userland tools, use the `DEBUG` flag.
```bash
make DEBUG=1
```
This is useful for analyzing kernel oops messages or using `gdb` on the userland tools.

### Running a Full Test Suite
The project includes a comprehensive test suite. To run all unit and integration tests, execute:
```bash
./tools/test_runner.sh
```
This script will automatically build the necessary components and run a series of automated checks to verify the filesystem's integrity and functionality.