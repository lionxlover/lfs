mod disk;
mod fs;
mod inode;

use env_logger;
use fuser::{Config, MountOption};
use log::info;
use std::env;

fn main() {
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <image_file> <mountpoint>", args[0]);
        std::process::exit(1);
    }
    let image_file = args[1].clone();
    let mountpoint = &args[2];

    let mut options = Config::default();
    options.mount_options = vec![
        MountOption::FSName("lionfs".to_string()),
        MountOption::RW,
    ];

    info!("Mounting LionFS Block Allocator at {} backed by {}", mountpoint, image_file);
    
    let filesystem = fs::LionFS::new(&image_file);
    
    fuser::mount2(filesystem, mountpoint, &options).unwrap();
}
