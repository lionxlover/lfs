use std::io::{Result, Error, ErrorKind};
use std::cmp::min;
use crate::transaction::transaction::TxContext;
use crate::ondisk::serialization::{Inode, Extent, BLOCK_SIZE, MAX_INLINE_EXTENTS, BlockGroupDescriptor};
use crate::allocator::bitmap::Allocator;

use crate::integrity::algorithms::{ChecksumAlgorithm, calculate_checksum, verify_checksum};
use crate::integrity::checksum_tree::{ChecksumTree, ChecksumTreeKey, ChecksumTreeValue};
use crate::integrity::bad_blocks::{BadBlockManager};

pub struct FileManager;

impl FileManager {
    pub fn read_file(ctx: &mut TxContext, checksum_tree_root: u64, bad_blocks_root: u64, inode: &mut Inode, offset: u64, size: u64) -> Result<Vec<u8>> {
        if offset >= inode.size {
            return Ok(Vec::new());
        }
        
        let read_size = min(size, inode.size - offset);
        let mut data = vec![0u8; read_size as usize];
        let mut data_pos = 0;
        let mut current_offset = offset;
        
        while data_pos < read_size {
            let logical_block = current_offset / BLOCK_SIZE as u64;
            let block_offset = (current_offset % BLOCK_SIZE as u64) as usize;
            
            let physical_block = Self::get_physical_block(inode, logical_block)?;
            let mut buf = [0u8; BLOCK_SIZE];
            
            if physical_block != 0 {
                ctx.read_block(physical_block, &mut buf)?;
                
                // Verify Checksum
                if checksum_tree_root != 0 {
                    let csum_tree = ChecksumTree::new(checksum_tree_root);
                    let key = ChecksumTreeKey { object_id: inode.ino, logical_block };
                    if let Ok(Some(val)) = csum_tree.lookup_checksum(ctx, &key) {
                        let algo = ChecksumAlgorithm::from_u8(val.algorithm_id);
                        if !verify_checksum(algo, &buf, &val.checksum_bytes) {
                            // Corruption detected!
                            eprintln!("CORRUPTION DETECTED: Inode {}, Logical Block {}", inode.ino, logical_block);
                            if bad_blocks_root != 0 {
                                let mut bb_mgr = BadBlockManager::new(bad_blocks_root);
                                let mut dummy_allocator = |_ctx: &mut TxContext| -> Result<u64> {
                                    Err(Error::other("Should not allocate during bad block marking in read"))
                                };
                                let _ = bb_mgr.mark_bad_block(ctx, physical_block, inode.ino, &mut dummy_allocator);
                            }
                            // Do not return corrupted data
                            return Err(Error::new(ErrorKind::InvalidData, "Checksum mismatch on read"));
                        }
                    }
                }
            } // if 0, it's a hole, leave buf as 0s
            
            let chunk_size = min(
                (BLOCK_SIZE - block_offset) as u64,
                read_size - data_pos
            ) as usize;
            
            data[data_pos as usize..data_pos as usize + chunk_size]
                .copy_from_slice(&buf[block_offset..block_offset + chunk_size]);
                
            data_pos += chunk_size as u64;
            current_offset += chunk_size as u64;
        }
        
        Ok(data)
    }

    pub fn write_file(ctx: &mut TxContext, bg_desc: &BlockGroupDescriptor, blocks_per_group: u32, checksum_tree_root: u64, inode: &mut Inode, offset: u64, data: &[u8]) -> Result<()> {
        let mut data_pos = 0;
        let mut current_offset = offset;
        
        while data_pos < data.len() {
            let logical_block = current_offset / BLOCK_SIZE as u64;
            let block_offset = (current_offset % BLOCK_SIZE as u64) as usize;
            
            let mut physical_block = Self::get_physical_block(inode, logical_block).unwrap_or(0);
            
            if physical_block == 0 {
                // We need to allocate a new block
                physical_block = Allocator::allocate_extents(ctx, bg_desc, blocks_per_group, 1)?;
                Self::add_extent(inode, logical_block, physical_block, 1)?;
            }
            
            let mut buf = [0u8; BLOCK_SIZE];
            
            let chunk_size = min(
                BLOCK_SIZE - block_offset ,
                data.len() - data_pos
            );
            
            // Read-modify-write if partial block
            if chunk_size < BLOCK_SIZE && physical_block != 0 {
                // PHASE 6 Data CoW Infrastructure:
                // If we had the refcount_tree_root here, we would check if this physical_block
                // has a refcount > 1. If it does, we must NOT modify it in place.
                // We would allocate a new block, copy the existing data into the new block,
                // decrement the refcount of the old block, and update the Inode's extent list.
                // For now, we perform in-place modification.
                ctx.read_block(physical_block, &mut buf)?;
            }
            
            buf[block_offset..block_offset + chunk_size]
                .copy_from_slice(&data[data_pos..data_pos + chunk_size]);
                
            ctx.write_block(physical_block, &buf)?;
            
            // Calculate and store checksum
            if checksum_tree_root != 0 {
                let algo = ChecksumAlgorithm::XxHash64;
                let csum_bytes = calculate_checksum(algo, &buf);
                let mut csum_tree = ChecksumTree::new(checksum_tree_root);
                let key = ChecksumTreeKey { object_id: inode.ino, logical_block };
                let val = ChecksumTreeValue {
                    physical_block,
                    checksum_bytes: csum_bytes,
                    generation: 1,
                    algorithm_id: algo as u8,
                    verification_status: 1, // Verified (just written)
                    padding: [0; 6],
                };
                
                let mut dummy_allocator = |_ctx: &mut TxContext| -> Result<u64> {
                    Err(Error::other("Not expecting allocation in ChecksumTree overwrite for now"))
                };
                // NOTE: Proper allocation lambda needed if tree grows
                let _ = csum_tree.insert_checksum(ctx, key, val, &mut dummy_allocator);
            }
            
            data_pos += chunk_size;
            current_offset += chunk_size as u64;
        }
        
        if offset + data.len() as u64 > inode.size {
            inode.size = offset + data.len() as u64;
        }
        
        Ok(())
    }

    pub fn truncate_file(ctx: &mut TxContext, bg_desc: &BlockGroupDescriptor, inode: &mut Inode, new_size: u64) -> Result<()> {
        if new_size >= inode.size {
            // Expansion is handled via write or explicit fallocate, ignore for now
            inode.size = new_size;
            return Ok(());
        }
        
        let new_blocks = new_size.div_ceil(BLOCK_SIZE as u64);
        
        // Remove and free extents that are fully beyond new_blocks
        let mut new_extent_count = 0;
        for i in 0..inode.extent_count as usize {
            let extent = &mut inode.extents[i];
            
            if extent.logical_start >= new_blocks {
                // Free the whole extent
                Allocator::free_extents(ctx, bg_desc, extent.physical_start, extent.length)?;
                extent.logical_start = 0;
                extent.physical_start = 0;
                extent.length = 0;
            } else if extent.logical_start + extent.length > new_blocks {
                // Partial truncate of extent
                let keep_blocks = new_blocks - extent.logical_start;
                let free_blocks = extent.length - keep_blocks;
                
                Allocator::free_extents(ctx, bg_desc, extent.physical_start + keep_blocks, free_blocks)?;
                extent.length = keep_blocks;
                new_extent_count += 1;
            } else {
                new_extent_count += 1;
            }
        }
        inode.extent_count = new_extent_count;
        inode.size = new_size;
        Ok(())
    }

    fn get_physical_block(inode: &Inode, logical_block: u64) -> Result<u64> {
        for i in 0..inode.extent_count as usize {
            let extent = &inode.extents[i];
            if logical_block >= extent.logical_start && logical_block < extent.logical_start + extent.length {
                return Ok(extent.physical_start + (logical_block - extent.logical_start));
            }
        }
        Ok(0) // 0 implies hole
    }
    
    fn add_extent(inode: &mut Inode, logical_block: u64, physical_block: u64, length: u64) -> Result<()> {
        // Try to merge with adjacent extent
        for i in 0..inode.extent_count as usize {
            let extent = &mut inode.extents[i];
            
            if extent.logical_start + extent.length == logical_block && extent.physical_start + extent.length == physical_block {
                extent.length += length;
                return Ok(());
            }
            if logical_block + length == extent.logical_start && physical_block + length == extent.physical_start {
                extent.logical_start = logical_block;
                extent.physical_start = physical_block;
                extent.length += length;
                return Ok(());
            }
        }
        
        // Cannot merge, append new extent
        if (inode.extent_count as usize) < MAX_INLINE_EXTENTS {
            inode.extents[inode.extent_count as usize] = Extent {
                logical_start: logical_block,
                physical_start: physical_block,
                length,
            };
            inode.extent_count += 1;
            Ok(())
        } else {
            eprintln!("Max inline extents reached! ino: {}, logical_block: {}, physical_block: {}, length: {}", inode.ino, logical_block, physical_block, length);
            for i in 0..inode.extent_count as usize {
                let e = &inode.extents[i];
                eprintln!("  extent {}: log_start: {}, phys_start: {}, len: {}", i, e.logical_start, e.physical_start, e.length);
            }
            Err(Error::new(ErrorKind::OutOfMemory, "Max inline extents reached (Phase 1 limit)"))
        }
    }
}
