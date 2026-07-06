use bytemuck::{Pod, Zeroable};

pub const BLOCK_SIZE: usize = 4096;
pub const LIONFS_MAGIC: u64 = 0x4C494F4E46533130; // "LIONFS10"
pub const MAX_INLINE_EXTENTS: usize = 7;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Superblock {
    pub magic: u64,
    pub version: u32,
    pub block_size: u32,
    pub total_blocks: u64,
    pub free_blocks: u64,
    pub inode_count: u64,
    pub root_inode: u64,
    pub flags: u32,
    pub padding1: u32,
    pub bitmap_start: u64,
    pub inode_table_start: u64,
    pub data_region_start: u64,
    pub generation: u64,
    pub checksum: u32,
    pub padding_csum: u32,
    pub journal_start: u64,
    pub journal_blocks: u64,
    pub secondary_sb_1: u64,
    pub secondary_sb_2: u64,
    pub padding2: [u8; BLOCK_SIZE - 128], // Ensures Superblock is exactly 4096 bytes
}

pub const JOURNAL_MAGIC: u64 = 0x4A4F55524E414C31; // "JOURNAL1"

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct JournalHeader {
    pub magic: u64,
    pub version: u32,
    pub entry_count: u32,
    pub tx_id: u64,
    pub timestamp: u64,
    pub checksum: u32,
    pub padding_csum: u32,
    pub padding: [u8; BLOCK_SIZE - 40], // Header is a full block
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct JournalRecordHeader {
    pub tx_id: u64,
    pub physical_block: u64, // Where this block belongs
    pub checksum: u32,       // Checksum of the 4096-byte data block that follows this record header
    pub padding: u32,
    // The actual 4096-byte data block is written immediately after this header (which we will pad to 1 block or just write inline).
    // Actually, to make it simple: each journaled block takes exactly 2 blocks in the journal: 
    // 1 block for JournalRecordHeader (with padding) + 1 block for the Data itself.
    // Let's just make JournalRecordHeader exactly 4096 bytes.
    pub padding2: [u8; BLOCK_SIZE - 24],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct JournalFooter {
    pub magic: u64, // Same magic or "JCOMMIT1"
    pub tx_id: u64,
    pub total_records: u32,
    pub checksum: u32,
    pub padding: [u8; BLOCK_SIZE - 24], // Full block
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Extent {
    pub logical_start: u64,
    pub physical_start: u64,
    pub length: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Inode {
    pub ino: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub links_count: u32,
    pub flags: u32,
    pub padding1: u32,
    pub size: u64,
    pub ctime: i64,
    pub mtime: i64,
    pub atime: i64,
    pub extent_count: u16,
    pub padding2: u16,
    pub padding3: u32,
    pub extents: [Extent; MAX_INLINE_EXTENTS],
    pub checksum: u32,
    pub padding4: [u8; 12], // Pads Inode to exactly 256 bytes
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct DirEntryHeader {
    pub ino: u64,
    pub rec_len: u16,
    pub name_len: u8,
    pub file_type: u8,
    pub padding: u32,
}
