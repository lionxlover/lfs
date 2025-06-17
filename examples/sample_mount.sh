#!/bin/bash

# ===================================================================
# LFS Sample Mount Script
#
# This script demonstrates how to mount an LFS filesystem with
# various common and advanced mount options.
#
# Usage:
#   ./sample_mount.sh <device> <mount_point>
#
# Example:
#   sudo ./sample_mount.sh /dev/sdb1 /mnt/lfs_data
#   sudo ./sample_mount.sh lfs.img /mnt/lfs_image
# ===================================================================

# --- Configuration ---

# The device or image file to mount.
# Passed as the first argument to the script.
DEVICE="$1"

# The directory where the filesystem will be mounted.
# Passed as the second argument to the script.
MOUNT_POINT="$2"

# --- Script Logic ---

# Function to print usage information and exit.
print_usage() {
    echo "Usage: $0 <device_or_image_file> <mount_point>"
    echo "Example: sudo $0 /dev/sdc1 /data"
    exit 1
}

# 1. Validate Input
if [ -z "$DEVICE" ] || [ -z "$MOUNT_POINT" ]; then
    echo "Error: Both a device and a mount point must be provided."
    print_usage
fi

if [ "$EUID" -ne 0 ]; then
  echo "Error: This script must be run as root (or with sudo)."
  exit 1
fi

if [ ! -e "$DEVICE" ]; then
    echo "Error: Device or image file '$DEVICE' not found."
    exit 1
fi

# 2. Ensure the mount point exists
if [ ! -d "$MOUNT_POINT" ]; then
    echo "Mount point '$MOUNT_POINT' not found. Creating it..."
    mkdir -p "$MOUNT_POINT"
    if [ $? -ne 0 ]; then
        echo "Error: Failed to create mount point."
        exit 1
    fi
fi

# 3. Define Mount Options
# The `-o` flag is used to pass a comma-separated list of options.
# Here are some examples of LFS-specific and standard mount options.
# Uncomment or combine the options you need.

# Option Set 1: High-Performance Default
# - noatime: Disables writing last-access times. A huge performance win.
# - data=writeback: Metadata is journaled, but data is written out later. Fast and safe for most uses.
MOUNT_OPTIONS="noatime,data=writeback"

# Option Set 2: Maximum Data Integrity
# - data=journal: Journals all data and metadata. Slower writes, but guarantees data content after a crash.
# - sync: All writes are committed to disk immediately. Very slow, but required for some databases.
# MOUNT_OPTIONS="data=journal,sync"

# Option Set 3: Enabling or Disabling Features at Mount Time
# - compress=lz4: Enables LZ4 compression for all new files.
# - nocompress: Disables compression, even if the feature is enabled on the filesystem.
# - discard: Enables TRIM/discard commands for SSDs to manage free space.
# MOUNT_OPTIONS="noatime,compress=lz4,discard"

# Option Set 4: Read-Only for Maintenance or Safety
# - ro: Mounts the filesystem in read-only mode.
# MOUNT_OPTIONS="ro"


# 4. Execute the Mount Command
echo "Mounting '$DEVICE' on '$MOUNT_POINT' with options: '$MOUNT_OPTIONS'..."

mount -t lfs -o "$MOUNT_OPTIONS" "$DEVICE" "$MOUNT_POINT"

# 5. Verify the Mount
if [ $? -eq 0 ]; then
    echo "✅ Successfully mounted LFS filesystem."
    echo "---"
    df -hT "$MOUNT_POINT"
    echo "---"
    # Use lfs-info to see detailed LFS-specific mount info
    # /path/to/lfs-info "$MOUNT_POINT"
else
    echo "❌ Error: Failed to mount LFS filesystem."
    echo "Check dmesg for kernel messages that might indicate the problem."
    dmesg | tail -n 20
    exit 1
fi

exit 0