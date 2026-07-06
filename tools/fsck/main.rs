use std::env;
use lionfs_core::disk::block_io::Disk;
use lionfs_core::ondisk::serialization::{Superblock, BLOCK_SIZE, LIONFS_MAGIC};

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
    println!("Inode count: {}", sb.inode_count);
    
    // Check root inode
    let mut root_buf = [0u8; BLOCK_SIZE];
    disk.read_block(sb.inode_table_start, &mut root_buf).unwrap();
    let root_inode: lionfs_core::ondisk::serialization::Inode = *bytemuck::from_bytes(&root_buf[256..512]);
    if root_inode.ino != 1 {
        eprintln!("ERROR: Root inode not found at index 1!");
        std::process::exit(1);
    }
    println!("Root inode OK.");
}
