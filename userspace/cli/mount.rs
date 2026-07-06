use std::env;
use fuser::MountOption;
use lionfs_core::disk::block_io::Disk;
use lionfs_core::fs::filesystem::LionFS;

fn main() {
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: mount-lfs <image_file> <mountpoint>");
        std::process::exit(1);
    }

    let image_file = &args[1];
    let mountpoint = &args[2];

    let disk = Disk::open(image_file).expect("Failed to open image file");
    let fs = LionFS::new(disk).expect("Failed to mount LionFS");

    let options = vec![
        MountOption::FSName("lionfs".to_string()),
        MountOption::RW,
        MountOption::DefaultPermissions,
    ];

    println!("Mounting {} to {}", image_file, mountpoint);
    fuser::mount2(fs, mountpoint, &options).unwrap();
}
