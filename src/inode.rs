use bytemuck::{Pod, Zeroable};
use fuser::{FileAttr, FileType};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const PAYLOAD_SIZE: usize = 4080;
pub const DIRECT_BLOCKS: usize = 12;

pub const KIND_FILE: u32 = 1;
pub const KIND_DIR: u32 = 2;
pub const KIND_SYMLINK: u32 = 3;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct DiskInode {
    pub ino: u64,
    pub parent: u64,
    pub size: u64,
    pub ctime: u64,
    pub mtime: u64,
    pub atime: u64,
    pub uid: u32,
    pub gid: u32,
    pub nlink: u32,
    pub kind: u32,
    pub mode: u16,
    pub padding1: u16,
    pub direct_blocks: [u32; DIRECT_BLOCKS],
    pub direct_checksums: [u32; DIRECT_BLOCKS],
    pub indirect_block: u32,
    pub flags: u32,
    pub padding: [u8; 84],
}

impl DiskInode {
    pub fn new(ino: u64, parent: u64, kind: u32, mode: u16, uid: u32, gid: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            ino,
            parent,
            size: 0,
            ctime: now,
            mtime: now,
            atime: now,
            uid,
            gid,
            nlink: if kind == KIND_DIR { 2 } else { 1 },
            kind,
            mode,
            padding1: 0,
            direct_blocks: [0; DIRECT_BLOCKS],
            direct_checksums: [0; DIRECT_BLOCKS],
            indirect_block: 0,
            flags: 0,
            padding: [0; 84],
        }
    }

    pub fn attr(&self) -> FileAttr {
        FileAttr {
            ino: fuser::INodeNo(self.ino),
            size: self.size,
            blocks: (self.size + 511) / 512,
            atime: UNIX_EPOCH + Duration::from_secs(self.atime),
            mtime: UNIX_EPOCH + Duration::from_secs(self.mtime),
            ctime: UNIX_EPOCH + Duration::from_secs(self.ctime),
            crtime: UNIX_EPOCH + Duration::from_secs(self.ctime),
            kind: match self.kind {
                KIND_DIR => FileType::Directory,
                KIND_SYMLINK => FileType::Symlink,
                _ => FileType::RegularFile,
            },
            perm: self.mode,
            nlink: self.nlink,
            uid: self.uid,
            gid: self.gid,
            rdev: 0,
            blksize: 4080,
            flags: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct IndirectBlock {
    pub blocks: [u32; 510],
    pub checksums: [u32; 510],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct DirEntry {
    pub ino: u64,
    pub name: [u8; 56],
}
