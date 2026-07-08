use std::io::Result;
use crate::disk::block_io::Disk;
use crate::ondisk::serialization::{Superblock, JournalHeader, JournalRecordHeader, JournalFooter, JOURNAL_MAGIC, BLOCK_SIZE};
use crate::transaction::transaction::Transaction;
use crate::utils::crc::compute_checksum;
use bytemuck::bytes_of;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

pub struct TransactionManager {
    pub current_tx_id: AtomicU64,
    pub next_journal_block: RwLock<u64>,
}

impl TransactionManager {
    pub fn new(sb: &Superblock) -> Self {
        Self {
            current_tx_id: AtomicU64::new(sb.generation + 1),
            next_journal_block: RwLock::new(0), // This will wrap around journal_blocks
        }
    }

    pub fn begin(&self, timestamp: u64) -> Transaction {
        let tx_id = self.current_tx_id.fetch_add(1, Ordering::SeqCst);
        Transaction::new(tx_id, timestamp)
    }

    pub fn commit(&self, disk: &mut Disk, sb: &Superblock, tx: &Transaction) -> Result<()> {
        if tx.dirty_blocks.is_empty() {
            return Ok(());
        }

        let entry_count = tx.dirty_blocks.len() as u32;
        
        let mut header = JournalHeader {
            magic: JOURNAL_MAGIC,
            version: 1,
            entry_count,
            tx_id: tx.id,
            timestamp: tx.timestamp,
            checksum: 0,
            padding_csum: 0,
            padding: [0; BLOCK_SIZE - 40],
        };

        header.checksum = compute_checksum(bytes_of(&header));

        // Lock journal offset for sequential write
        let mut j_block_guard = self.next_journal_block.write().unwrap();
        let start_logical = *j_block_guard;
        let mut current_j_block = start_logical;
        
        let header_p_block = sb.journal_start + (current_j_block % sb.journal_blocks);
        disk.write_block(header_p_block, bytes_of(&header))?;
        current_j_block += 1;

        // Write records
        for (&p_block, data) in &tx.dirty_blocks {
            let data_checksum = compute_checksum(data);
            let rec_header = JournalRecordHeader {
                tx_id: tx.id,
                physical_block: p_block,
                checksum: data_checksum,
                padding: 0,
                padding2: [0; BLOCK_SIZE - 24],
            };
            
            let rec_p_block = sb.journal_start + (current_j_block % sb.journal_blocks);
            disk.write_block(rec_p_block, bytes_of(&rec_header))?;
            current_j_block += 1;
            
            let data_j_block = sb.journal_start + (current_j_block % sb.journal_blocks);
            disk.write_block(data_j_block, data)?;
            current_j_block += 1;
        }

        // Flush journal before writing footer
        disk.sync()?;

        let mut footer = JournalFooter {
            magic: JOURNAL_MAGIC,
            tx_id: tx.id,
            total_records: entry_count,
            checksum: 0,
            padding: [0; BLOCK_SIZE - 24],
        };
        footer.checksum = compute_checksum(bytes_of(&footer));

        let footer_p_block = sb.journal_start + (current_j_block % sb.journal_blocks);
        disk.write_block(footer_p_block, bytes_of(&footer))?;
        current_j_block += 1;

        // Flush footer so transaction is firmly committed in WAL
        disk.sync()?;

        *j_block_guard = current_j_block % sb.journal_blocks;
        drop(j_block_guard); // Release lock early before applying to actual disk locations

        // Now apply to actual disk locations
        for (&p_block, data) in &tx.dirty_blocks {
            disk.write_block(p_block, data)?;
        }

        // Flush final data
        disk.sync()?;

        Ok(())
    }
}
