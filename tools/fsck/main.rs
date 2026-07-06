use std::env;
use lionfs_core::disk::block_io::Disk;
use lionfs_core::ondisk::serialization::{Superblock, BLOCK_SIZE, LIONFS_MAGIC};
use lionfs_core::transaction::manager::TransactionManager;
use lionfs_core::transaction::transaction::TxContext;
use lionfs_core::btree::tree::BTree;
use lionfs_core::ondisk::serialization::Inode;
use lionfs_core::directory::tree::DirTreeValue;
use lionfs_core::extents::tree::ExtentTreeValue;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: fsck-lfs <image_file>");
        std::process::exit(1);
    }

    let image_file = &args[1];
    let mut disk = Disk::open(image_file).expect("Failed to open image file");
    
    let mut buf = [0u8; BLOCK_SIZE];
    disk.read_block(0, &mut buf).expect("Failed to read superblock");
    
    let sb: Superblock = *bytemuck::from_bytes(&buf);
    
    if sb.magic != LIONFS_MAGIC {
        eprintln!("ERROR: Invalid magic number!");
        std::process::exit(1);
    }
    
    println!("LionFS volume OK.");
    println!("Version: {}", sb.version);
    println!("Total blocks: {}", sb.total_blocks);
    println!("Free blocks: {}", sb.free_blocks);
    
    let mut tm = TransactionManager::new(&sb);
    let mut tx = tm.begin(0);
    let mut ctx = TxContext::new(&mut disk, &mut tx);

    if sb.inode_tree_root != 0 {
        let inode_tree = BTree::<u64, Inode>::new(sb.inode_tree_root, lionfs_core::inode::tree::INODE_TREE_NODE_TYPE);
        match inode_tree.validate(&mut ctx) {
            Ok(count) => println!("InodeTree valid. Validated {} nodes.", count),
            Err(e) => eprintln!("ERROR: InodeTree validation failed: {}", e),
        }
    }

    if sb.dir_tree_root != 0 {
        let dir_tree = BTree::<u64, DirTreeValue>::new(sb.dir_tree_root, lionfs_core::directory::tree::DIR_TREE_NODE_TYPE);
        match dir_tree.validate(&mut ctx) {
            Ok(count) => println!("DirectoryTree valid. Validated {} nodes.", count),
            Err(e) => eprintln!("ERROR: DirectoryTree validation failed: {}", e),
        }
    }

    if sb.extent_tree_root != 0 {
        let ext_tree = BTree::<u64, ExtentTreeValue>::new(sb.extent_tree_root, lionfs_core::extents::tree::EXTENT_TREE_NODE_TYPE);
        match ext_tree.validate(&mut ctx) {
            Ok(count) => println!("ExtentTree valid. Validated {} nodes.", count),
            Err(e) => eprintln!("ERROR: ExtentTree validation failed: {}", e),
        }
    }

    if sb.freespace_tree_root != 0 {
        let free_tree = BTree::<u64, u64>::new(sb.freespace_tree_root, lionfs_core::allocator::tree::FREESPACE_TREE_NODE_TYPE);
        match free_tree.validate(&mut ctx) {
            Ok(count) => println!("FreeSpaceTree valid. Validated {} nodes.", count),
            Err(e) => eprintln!("ERROR: FreeSpaceTree validation failed: {}", e),
        }
    }
}
