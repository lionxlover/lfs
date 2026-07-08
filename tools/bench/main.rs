use std::time::Instant;
use lionfs_core::disk::block_io::Disk;
use lionfs_core::transaction::manager::TransactionManager;
use lionfs_core::transaction::transaction::TxContext;
use lionfs_core::btree::tree::BTree;
use lionfs_core::inode::tree::INODE_TREE_NODE_TYPE;
use lionfs_core::ondisk::serialization::{Superblock, Inode, Extent, MAX_INLINE_EXTENTS};
use std::fs;

fn main() {
    let test_file = "test_bench.img";
    let _ = fs::remove_file(test_file);
    let mut disk = Disk::create(test_file, 1024 * 1024 * 100).unwrap(); // 100MB
    
    let sb = Superblock {
        magic: 0, version: 0, block_size: 4096, total_blocks: 10240, free_blocks: 0, inode_count: 0,
        root_inode: 0, flags: 0, padding1: 0, bitmap_start: 0, inode_table_start: 0, data_region_start: 0,
        generation: 0, checksum: 0, padding_csum: 0, journal_start: 1, journal_blocks: 10,
        secondary_sb_1: 0, secondary_sb_2: 0, block_group_count: 0, blocks_per_group: 0,
        inode_tree_root: 0, dir_tree_root: 0, extent_tree_root: 0, freespace_tree_root: 0,
        next_ino: 2, checksum_tree_root: 0, bad_blocks_root: 0, 
        snapshot_tree_root: 0, clone_tree_root: 0, refcount_tree_root: 0, subvolume_tree_root: 0, space_map_root: 0, last_snapshot_generation: 0,
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
    disk.write_block(0, bytemuck::bytes_of(&sb)).unwrap();

    let mut tm = TransactionManager::new(&sb);
    let mut tx = tm.begin(0);
    let mut ctx = TxContext::new(&mut disk, &mut tx);
    BTree::<u64, Inode>::init_empty(&mut ctx, 12, INODE_TREE_NODE_TYPE).unwrap();
    let mut tree = BTree::<u64, Inode>::new(12, INODE_TREE_NODE_TYPE);

    let mut mock_allocator = |_ctx: &mut TxContext| -> std::io::Result<u64> {
        Ok(999) 
    };

    let start = Instant::now();
    let count = 10_000;
    for i in 0..count {
        let inode = Inode {
            ino: i,
            mode: 0, uid: 0, gid: 0, links_count: 0, flags: 0, padding1: 0,
            size: 0, ctime: 0, mtime: 0, atime: 0, extent_count: 0,
            compression_algo: 0, encryption_algo: 0, key_id: 0,
            extents: [Extent { logical_start: 0, physical_start: 0, length: 0 }; MAX_INLINE_EXTENTS],
            checksum: 0, padding4: [0; 12],
        };
        tree.insert(&mut ctx, i, inode, &mut mock_allocator).unwrap();
    }
    
    let duration = start.elapsed();
    println!("Inserted {} inodes in {:?}", count, duration);
    println!("Ops/sec: {:.2}", count as f64 / duration.as_secs_f64());

    let _ = fs::remove_file(test_file);
}
