use std::time::Instant;
use lionfs_core::disk::block_io::Disk;
use lionfs_core::transaction::manager::TransactionManager;
use lionfs_core::transaction::transaction::TxContext;
use lionfs_core::btree::tree::BTree;
use lionfs_core::ondisk::serialization::Superblock;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct BenchKey(u64);
unsafe impl bytemuck::Zeroable for BenchKey {}
unsafe impl bytemuck::Pod for BenchKey {}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BenchValue(u64);
unsafe impl bytemuck::Zeroable for BenchValue {}
unsafe impl bytemuck::Pod for BenchValue {}

fn main() {
    println!("LionFS Phase 4 - B+ Tree Benchmark Suite");
    
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("bench_btree.img");
    let mut disk = Disk::create(&path, 1024 * 1024 * 50).unwrap();
    
    let sb = Superblock {
        magic: 0, version: 0, block_size: 4096, total_blocks: 10240, free_blocks: 0, inode_count: 0,
        root_inode: 0, flags: 0, padding1: 0, bitmap_start: 0, inode_table_start: 0, data_region_start: 0,
        generation: 0, checksum: 0, padding_csum: 0, journal_start: 1, journal_blocks: 10,
        secondary_sb_1: 0, secondary_sb_2: 0, block_group_count: 0, blocks_per_group: 0,
        inode_tree_root: 0, dir_tree_root: 0, extent_tree_root: 0, freespace_tree_root: 0,
        next_ino: 2, padding2: [0; 3920],
    };
    disk.write_block(0, bytemuck::bytes_of(&sb)).unwrap();

    let mut tm = TransactionManager::new(&sb);
    let mut tx = tm.begin(0);
    let mut ctx = TxContext::new(&mut disk, &mut tx);

    BTree::<BenchKey, BenchValue>::init_empty(&mut ctx, 100, 1).unwrap();
    let mut btree = BTree::<BenchKey, BenchValue>::new(100, 1);

    let mut next_block = 101;
    let mut allocator = |_: &mut TxContext| {
        let b = next_block;
        next_block += 1;
        Ok(b)
    };

    let num_items = 50_000;
    
    println!("Benchmarking B+ Tree Insertion ({} items)...", num_items);
    let start_insert = Instant::now();
    for i in 0..num_items {
        btree.insert(&mut ctx, BenchKey(i), BenchValue(i * 2), &mut allocator).unwrap();
    }
    let duration_insert = start_insert.elapsed();
    println!("Completed {} insertions in {:?}", num_items, duration_insert);
    println!("Insert Throughput: {:.2} ops/sec", (num_items as f64) / duration_insert.as_secs_f64());

    println!("Benchmarking B+ Tree Lookup ({} items)...", num_items);
    let start_lookup = Instant::now();
    for i in 0..num_items {
        let _ = btree.lookup(&mut ctx, &BenchKey(i)).unwrap();
    }
    let duration_lookup = start_lookup.elapsed();
    println!("Completed {} lookups in {:?}", num_items, duration_lookup);
    println!("Lookup Throughput: {:.2} ops/sec", (num_items as f64) / duration_lookup.as_secs_f64());
}
