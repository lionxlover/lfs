use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Mutex;
use bytemuck::{Pod, Zeroable};
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;

use crate::inode::{DiskInode, PAYLOAD_SIZE};

pub const BLOCK_SIZE: usize = 4096;
pub const TOTAL_BLOCKS: u32 = 32640;
pub const MAGIC: u32 = 0x4C494F4E; // "LION"

// Static Master Key for Encryption
const MASTER_KEY: [u8; 32] = *b"LionFSMasterKeyForEncryption32B!";

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct Superblock {
    pub magic: u32,
    pub block_size: u32,
    pub total_blocks: u32,
    pub free_blocks: u32,
    pub root_ino: u64,
    pub padding: [u8; PAYLOAD_SIZE - 24],
}

pub struct DiskManager {
    pub file: Mutex<File>,
}

impl DiskManager {
    pub fn new(path: &str) -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();

        let metadata = file.metadata().unwrap();
        if metadata.len() < (TOTAL_BLOCKS as u64 * BLOCK_SIZE as u64) {
            file.set_len(TOTAL_BLOCKS as u64 * BLOCK_SIZE as u64).unwrap();
            let disk = Self { file: Mutex::new(file) };
            disk.format();
            disk
        } else {
            let disk = Self { file: Mutex::new(file) };
            
            // Self-Healing Superblock Logic
            let mut sb_buf = [0u8; PAYLOAD_SIZE];
            disk.read_block(0, &mut sb_buf);
            let sb: &Superblock = bytemuck::from_bytes(&sb_buf);
            
            if sb.magic != MAGIC {
                log::warn!("Superblock 0 corrupted! Attempting recovery from Backup Superblock...");
                let mut backup_buf = [0u8; PAYLOAD_SIZE];
                disk.read_block(TOTAL_BLOCKS - 1, &mut backup_buf);
                let backup_sb: &Superblock = bytemuck::from_bytes(&backup_buf);
                
                if backup_sb.magic == MAGIC {
                    log::warn!("Backup Superblock valid! Restoring Block 0.");
                    disk.write_block(0, &backup_buf);
                } else {
                    log::error!("Backup Superblock also corrupted! Fatal error.");
                }
            }
            
            disk
        }
    }

    pub fn read_block(&self, block_id: u32, payload: &mut [u8; PAYLOAD_SIZE]) {
        let mut file = self.file.lock().unwrap();
        file.seek(SeekFrom::Start((block_id as u64) * (BLOCK_SIZE as u64))).unwrap();
        let mut block = [0u8; BLOCK_SIZE];
        file.read_exact(&mut block).unwrap_or(());

        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&block[0..12]);
        payload.copy_from_slice(&block[16..4096]);

        if nonce != [0u8; 12] {
            let mut cipher = ChaCha20::new(&MASTER_KEY.into(), &nonce.into());
            cipher.apply_keystream(payload);
        }
    }

    pub fn write_block(&self, block_id: u32, payload: &[u8; PAYLOAD_SIZE]) {
        let mut encrypted_payload = *payload;
        
        let mut nonce = [0u8; 12];
        let mut is_empty = true;
        for b in payload.iter() {
            if *b != 0 {
                is_empty = false;
                break;
            }
        }
        
        // Don't encrypt completely zeroed blocks
        if !is_empty {
            std::fs::File::open("/dev/urandom").unwrap().read_exact(&mut nonce).unwrap();
            let mut cipher = ChaCha20::new(&MASTER_KEY.into(), &nonce.into());
            cipher.apply_keystream(&mut encrypted_payload);
        }

        let mut block = [0u8; BLOCK_SIZE];
        block[0..12].copy_from_slice(&nonce);
        block[16..4096].copy_from_slice(&encrypted_payload);

        let mut file = self.file.lock().unwrap();
        file.seek(SeekFrom::Start((block_id as u64) * (BLOCK_SIZE as u64))).unwrap();
        file.write_all(&block).unwrap();
    }

    fn format(&self) {
        let sb = Superblock {
            magic: MAGIC,
            block_size: BLOCK_SIZE as u32,
            total_blocks: TOTAL_BLOCKS,
            free_blocks: TOTAL_BLOCKS - 35,
            root_ino: 1,
            padding: [0; PAYLOAD_SIZE - 24],
        };

        let mut sb_bytes = [0u8; PAYLOAD_SIZE];
        sb_bytes.copy_from_slice(bytemuck::bytes_of(&sb));
        self.write_block(0, &sb_bytes);
        // Backup superblock
        self.write_block(TOTAL_BLOCKS - 1, &sb_bytes);

        // Block 1 is the block bitmap
        let mut bitmap = [0u8; PAYLOAD_SIZE];
        // Mark blocks 0 to 33 and TOTAL_BLOCKS - 1 as used
        for i in 0..34 {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            bitmap[byte_idx] |= 1 << bit_idx;
        }
        let backup_idx = (TOTAL_BLOCKS - 1) as usize;
        bitmap[backup_idx / 8] |= 1 << (backup_idx % 8);
        self.write_block(1, &bitmap);

        // Blocks 2 to 33 are the inode table
        let zero_block = [0u8; PAYLOAD_SIZE];
        for i in 2..34 {
            self.write_block(i, &zero_block);
        }
    }

    pub fn read_inode(&self, ino: u64) -> Option<DiskInode> {
        if ino == 0 || ino >= 1024 {
            return None;
        }
        let block_id = 2 + (ino / 15) as u32; // 15 inodes per block
        let offset = ((ino % 15) * 256) as usize;
        let mut buf = [0u8; PAYLOAD_SIZE];
        self.read_block(block_id, &mut buf);
        let inode: DiskInode = *bytemuck::from_bytes(&buf[offset..offset + 256]);
        if inode.kind != 0 {
            Some(inode)
        } else {
            None
        }
    }

    pub fn write_inode(&self, inode: &DiskInode) {
        let ino = inode.ino;
        let block_id = 2 + (ino / 15) as u32;
        let offset = ((ino % 15) * 256) as usize;
        let mut buf = [0u8; PAYLOAD_SIZE];
        self.read_block(block_id, &mut buf);
        buf[offset..offset + 256].copy_from_slice(bytemuck::bytes_of(inode));
        self.write_block(block_id, &buf);
    }

    pub fn allocate_inode(&self) -> Option<u64> {
        for block_id in 2..34 {
            let mut buf = [0u8; PAYLOAD_SIZE];
            self.read_block(block_id, &mut buf);
            for i in 0..15 {
                let offset = i * 256;
                let inode: &DiskInode = bytemuck::from_bytes(&buf[offset..offset + 256]);
                if inode.kind == 0 {
                    let ino = ((block_id - 2) * 15 + i as u32) as u64;
                    if ino > 0 {
                        return Some(ino);
                    }
                }
            }
        }
        None
    }

    pub fn free_inode(&self, ino: u64) {
        if ino == 0 || ino >= 1024 {
            return;
        }
        let block_id = 2 + (ino / 15) as u32;
        let offset = ((ino % 15) * 256) as usize;
        let mut buf = [0u8; PAYLOAD_SIZE];
        self.read_block(block_id, &mut buf);
        let zero_inode = [0u8; 256];
        buf[offset..offset + 256].copy_from_slice(&zero_inode);
        self.write_block(block_id, &buf);
    }

    pub fn allocate_block(&self) -> Option<u32> {
        let mut bitmap = [0u8; PAYLOAD_SIZE];
        self.read_block(1, &mut bitmap);
        for byte_idx in 0..PAYLOAD_SIZE {
            if bitmap[byte_idx] != 0xFF {
                for bit_idx in 0..8 {
                    if (bitmap[byte_idx] & (1 << bit_idx)) == 0 {
                        bitmap[byte_idx] |= 1 << bit_idx;
                        self.write_block(1, &bitmap);
                        
                        let mut sb_buf = [0u8; PAYLOAD_SIZE];
                        self.read_block(0, &mut sb_buf);
                        let sb: &mut Superblock = bytemuck::from_bytes_mut(&mut sb_buf);
                        sb.free_blocks -= 1;
                        self.write_block(0, &sb_buf);
                        self.write_block(TOTAL_BLOCKS - 1, &sb_buf);
                        
                        let block_id = (byte_idx * 8 + bit_idx) as u32;
                        // Zero out the newly allocated block
                        let zero_block = [0u8; PAYLOAD_SIZE];
                        self.write_block(block_id, &zero_block);
                        return Some(block_id);
                    }
                }
            }
        }
        None
    }

    pub fn free_block(&self, block_id: u32) {
        if block_id == 0 || block_id >= TOTAL_BLOCKS {
            return;
        }
        let byte_idx = (block_id / 8) as usize;
        let bit_idx = block_id % 8;
        let mut bitmap = [0u8; PAYLOAD_SIZE];
        self.read_block(1, &mut bitmap);
        
        if (bitmap[byte_idx] & (1 << bit_idx)) != 0 {
            bitmap[byte_idx] &= !(1 << bit_idx);
            self.write_block(1, &bitmap);
            
            let mut sb_buf = [0u8; PAYLOAD_SIZE];
            self.read_block(0, &mut sb_buf);
            let sb: &mut Superblock = bytemuck::from_bytes_mut(&mut sb_buf);
            sb.free_blocks += 1;
            self.write_block(0, &sb_buf);
            self.write_block(TOTAL_BLOCKS - 1, &sb_buf);
        }
    }
}
