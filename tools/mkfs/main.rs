use std::env;
use lionfs_core::disk::block_io::Disk;
use lionfs_core::ondisk::serialization::{Superblock, Inode, BLOCK_SIZE, LIONFS_MAGIC};


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: mkfs-lfs <image_file> <size_in_mb>");
        std::process::exit(1);
    }

    let image_file = &args[1];
    let size_mb: u64 = args[2].parse().expect("Invalid size");
    let total_blocks = (size_mb * 1024 * 1024) / BLOCK_SIZE as u64;

    if total_blocks < 10 {
        eprintln!("Size too small");
        std::process::exit(1);
    }

    // Calculate layout
    let bitmap_blocks = (total_blocks + (BLOCK_SIZE as u64 * 8) - 1) / (BLOCK_SIZE as u64 * 8);
    let inode_count = 1024; // Fixed for now
    let inodes_per_block = BLOCK_SIZE as u64 / 256;
    let inode_blocks = (inode_count + inodes_per_block - 1) / inodes_per_block;
    
    let bitmap_start = 1;
    let inode_table_start = bitmap_start + bitmap_blocks;
    let data_region_start = inode_table_start + inode_blocks;

    println!("Formatting {} with size {}MB ({} blocks)", image_file, size_mb, total_blocks);

    let mut disk = Disk::create(image_file, total_blocks * BLOCK_SIZE as u64).unwrap();

    let secondary_sb_1 = if total_blocks > 8192 { 8192 } else { 0 };
    let secondary_sb_2 = if total_blocks > 16384 { 16384 } else { 0 };
    let journal_start = data_region_start;
    let journal_blocks = 4096; // 16 MB flat journal for simplicity
    
    // Check if image is big enough for this layout
    if total_blocks < journal_start + journal_blocks + 100 {
        panic!("Disk image is too small for Phase 2 layout (requires at least {} blocks)", journal_start + journal_blocks + 100);
    }
    
    let mut sb = Superblock {
        magic: LIONFS_MAGIC,
        version: 1,
        block_size: BLOCK_SIZE as u32,
        total_blocks,
        free_blocks: total_blocks - (journal_start + journal_blocks), // Subtract metadata and journal
        inode_count,
        root_inode: 1,
        flags: 0,
        padding1: 0,
        bitmap_start: 1,
        inode_table_start,
        data_region_start: journal_start + journal_blocks, // Data region now starts AFTER journal
        generation: 1,
        checksum: 0,
        padding_csum: 0,
        journal_start,
        journal_blocks,
        secondary_sb_1: 8192,
        secondary_sb_2: 16384,
        block_group_count: 1,
        blocks_per_group: total_blocks as u32,
        inode_tree_root: 0,
        dir_tree_root: 0,
        extent_tree_root: 0,
        freespace_tree_root: 0,
        next_ino: 2,
        checksum_tree_root: 0,
        bad_blocks_root: 0,
        snapshot_tree_root: 0,
        clone_tree_root: 0,
        refcount_tree_root: 0,
        subvolume_tree_root: 0,
        space_map_root: 0,
        last_snapshot_generation: 0,
        dedupe_tree_root: 0,
        key_tree_root: 0,
        fs_features: 0,
        default_compression: 0,
        default_encryption: 0,
        padding_phase7: [0; 6],
        device_tree_root: 0,
        pool_uuid: [0; 16],
        raid_profile: 0,
        padding_raid: [0; 3],
        chunk_size: 0,
        padding2: [0; 3792],
    };

    // Calculate checksum
    use lionfs_core::utils::checksum::calculate_superblock_checksum;
    sb.checksum = calculate_superblock_checksum(&sb);

    println!("Writing primary superblock...");
    disk.write_block(0, bytemuck::bytes_of(&sb)).unwrap();

    if secondary_sb_1 != 0 {
        println!("Writing secondary superblock 1...");
        disk.write_block(secondary_sb_1, bytemuck::bytes_of(&sb)).unwrap();
    }
    
    if secondary_sb_2 != 0 {
        println!("Writing secondary superblock 2...");
        disk.write_block(secondary_sb_2, bytemuck::bytes_of(&sb)).unwrap();
    }

    // Init bitmap
    let mut bitmap_buf = [0u8; BLOCK_SIZE];
    // Mark blocks 0..data_region_start as used
    // This includes Superblock, Bitmaps, Inodes, and the Journal!
    for i in 0..sb.data_region_start {
        let byte_idx = (i / 8) as usize;
        let bit_idx = i % 8;
        bitmap_buf[byte_idx] |= 1 << bit_idx;
    }
    disk.write_block(bitmap_start, &bitmap_buf).unwrap();
    for i in 1..bitmap_blocks {
        disk.write_block(bitmap_start + i, &[0; BLOCK_SIZE]).unwrap();
    }

    // Init inodes
    for i in 0..inode_blocks {
        disk.write_block(inode_table_start + i, &[0; BLOCK_SIZE]).unwrap();
    }

    // Create Root Inode (ino 1)
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let root_inode = Inode {
        ino: 1,
        mode: libc::S_IFDIR as u32 | 0o755,
        uid: 1000,
        gid: 1000,
        links_count: 2,
        flags: 0,
        padding1: 0,
        size: 0,
        ctime: now,
        mtime: now,
        atime: now,
        extent_count: 0,
        compression_algo: 0,
        encryption_algo: 0,
        key_id: 0,
        extents: [lionfs_core::ondisk::serialization::Extent { logical_start: 0, physical_start: 0, length: 0 }; 7],
        checksum: 0,
        padding4: [0; 12],
    };
    
    let mut root_buf = [0u8; BLOCK_SIZE];
    root_buf[256..512].copy_from_slice(bytemuck::bytes_of(&root_inode)); // index 1 = offset 256
    disk.write_block(inode_table_start, &root_buf).unwrap();

    disk.sync().unwrap();
    println!("Format complete!");
}
