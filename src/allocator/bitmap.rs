use std::io::{Result, Error, ErrorKind};
use crate::transaction::transaction::TxContext;
use crate::ondisk::serialization::BLOCK_SIZE;

pub struct Allocator;

impl Allocator {
    /// Allocates `count` contiguous blocks. Returns the starting physical block number.
    /// For Phase 1, we do a simple linear scan over the bitmap blocks.
    pub fn allocate_extents(ctx: &mut TxContext, bitmap_start: u64, total_blocks: u64, count: u64) -> Result<u64> {
        if count == 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Cannot allocate 0 blocks"));
        }

        let total_bitmap_blocks = (total_blocks + (BLOCK_SIZE as u64 * 8) - 1) / (BLOCK_SIZE as u64 * 8);
        let mut buf = [0u8; BLOCK_SIZE];

        let mut current_run = 0;
        let mut run_start_block = 0;

        for bm_idx in 0..total_bitmap_blocks {
            ctx.read_block(bitmap_start + bm_idx, &mut buf)?;
            for byte_idx in 0..BLOCK_SIZE {
                let byte = buf[byte_idx];
                if byte == 0xFF {
                    current_run = 0; // run broken
                    continue;
                }

                for bit_idx in 0..8 {
                    let absolute_block = (bm_idx * BLOCK_SIZE as u64 * 8) + (byte_idx as u64 * 8) + bit_idx as u64;
                    if absolute_block >= total_blocks {
                        break;
                    }

                    if (byte & (1 << bit_idx)) == 0 {
                        // Free bit
                        if current_run == 0 {
                            run_start_block = absolute_block;
                        }
                        current_run += 1;

                        if current_run == count {
                            // We found a contiguous run! Mark them as used.
                            Self::mark_blocks_used(ctx, bitmap_start, run_start_block, count)?;
                            return Ok(run_start_block);
                        }
                    } else {
                        // Used bit
                        current_run = 0;
                    }
                }
            }
        }
        Err(Error::new(ErrorKind::OutOfMemory, "No contiguous free space found"))
    }

    pub fn free_extents(ctx: &mut TxContext, bitmap_start: u64, start: u64, count: u64) -> Result<()> {
        Self::mark_blocks_free(ctx, bitmap_start, start, count)
    }

    fn mark_blocks_used(ctx: &mut TxContext, bitmap_start: u64, start: u64, count: u64) -> Result<()> {
        Self::modify_blocks(ctx, bitmap_start, start, count, true)
    }

    fn mark_blocks_free(ctx: &mut TxContext, bitmap_start: u64, start: u64, count: u64) -> Result<()> {
        Self::modify_blocks(ctx, bitmap_start, start, count, false)
    }

    fn modify_blocks(ctx: &mut TxContext, bitmap_start: u64, start: u64, count: u64, set: bool) -> Result<()> {
        let mut current_bm_idx = u64::MAX;
        let mut buf = [0u8; BLOCK_SIZE];
        let mut modified = false;

        for i in 0..count {
            let block = start + i;
            let bm_idx = block / (BLOCK_SIZE as u64 * 8);
            let byte_idx = ((block % (BLOCK_SIZE as u64 * 8)) / 8) as usize;
            let bit_idx = block % 8;

            if bm_idx != current_bm_idx {
                if modified {
                    ctx.write_block(bitmap_start + current_bm_idx, &buf)?;
                }
                ctx.read_block(bitmap_start + bm_idx, &mut buf)?;
                current_bm_idx = bm_idx;
                modified = false;
            }

            if set {
                buf[byte_idx] |= 1 << bit_idx;
            } else {
                buf[byte_idx] &= !(1 << bit_idx);
            }
            modified = true;
        }

        if modified {
            ctx.write_block(bitmap_start + current_bm_idx, &buf)?;
        }
        Ok(())
    }
}
