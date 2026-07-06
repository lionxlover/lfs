use std::io::{Result, Error, ErrorKind};
use crate::transaction::transaction::TxContext;
use crate::ondisk::serialization::{Inode, BLOCK_SIZE};

pub const INODES_PER_BLOCK: u64 = (BLOCK_SIZE / 256) as u64;

pub struct InodeManager;

impl InodeManager {
    pub fn read_inode(ctx: &mut TxContext, inode_table_start: u64, ino: u64) -> Result<Inode> {
        if ino == 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Inode 0 is reserved"));
        }
        let block_offset = ino / INODES_PER_BLOCK;
        let inode_offset = (ino % INODES_PER_BLOCK) as usize * 256;

        let mut buf = [0u8; BLOCK_SIZE];
        ctx.read_block(inode_table_start + block_offset, &mut buf)?;

        let inode_bytes = &buf[inode_offset..inode_offset + 256];
        let inode: Inode = *bytemuck::from_bytes(inode_bytes);
        
        Ok(inode)
    }

    pub fn write_inode(ctx: &mut TxContext, inode_table_start: u64, inode: &Inode) -> Result<()> {
        let ino = inode.ino;
        if ino == 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Inode 0 is reserved"));
        }
        let block_offset = ino / INODES_PER_BLOCK;
        let inode_offset = (ino % INODES_PER_BLOCK) as usize * 256;

        let mut buf = [0u8; BLOCK_SIZE];
        ctx.read_block(inode_table_start + block_offset, &mut buf)?;

        let inode_bytes = bytemuck::bytes_of(inode);
        buf[inode_offset..inode_offset + 256].copy_from_slice(inode_bytes);

        ctx.write_block(inode_table_start + block_offset, &buf)?;
        Ok(())
    }

    pub fn allocate_inode(ctx: &mut TxContext, inode_table_start: u64, max_inodes: u64) -> Result<u64> {
        let mut buf = [0u8; BLOCK_SIZE];
        let total_blocks = (max_inodes + INODES_PER_BLOCK - 1) / INODES_PER_BLOCK;

        for block_idx in 0..total_blocks {
            ctx.read_block(inode_table_start + block_idx, &mut buf)?;
            for i in 0..INODES_PER_BLOCK {
                let offset = i as usize * 256;
                let inode_bytes = &buf[offset..offset + 256];
                let inode: Inode = *bytemuck::from_bytes(inode_bytes);
                
                let current_ino = block_idx * INODES_PER_BLOCK + i;
                if current_ino == 0 { continue; }

                if inode.links_count == 0 && inode.mode == 0 {
                    return Ok(current_ino);
                }
            }
        }
        Err(Error::new(ErrorKind::OutOfMemory, "No free inodes"))
    }
}
