use std::env;
use lionfs_core::disk::block_io::Disk;
use lionfs_core::ondisk::serialization::{Superblock, LIONFS_MAGIC, BLOCK_SIZE};
use lionfs_core::transaction::manager::TransactionManager;
use lionfs_core::transaction::transaction::TxContext;
use lionfs_core::btree::tree::BTree;
use lionfs_core::integrity::bad_blocks::{BadBlockKey, BadBlockValue, BAD_BLOCKS_TREE_NODE_TYPE};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mount_point = if args.len() > 1 { &args[1] } else { "/mnt/lion" };
    let health_file = format!("{}/.lfs_health", mount_point);
    
    match std::fs::read_to_string(&health_file) {
        Ok(report) => print!("{}", report),
        Err(e) => {
            eprintln!("Failed to read health report from {} (is LionFS mounted?): {}", health_file, e);
            std::process::exit(1);
        }
    }
}
