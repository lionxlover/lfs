use std::io::Result;
use crate::disk::block_io::Disk;
use crate::ondisk::serialization::{Superblock, JournalHeader, JournalRecordHeader, JournalFooter, JOURNAL_MAGIC, BLOCK_SIZE};
use crate::transaction::transaction::Transaction;
use crate::utils::crc::compute_checksum;
use bytemuck::bytes_of;

pub struct TransactionManager {
    pub current_tx_id: u64,
    pub next_journal_block: u64,
}

impl TransactionManager {
    pub fn new(sb: &Superblock) -> Self {
        Self {
            current_tx_id: sb.generation + 1,
            next_journal_block: 0, // This will wrap around journal_blocks
        }
    }

    pub fn begin(&mut self, timestamp: u64) -> Transaction {
        self.current_tx_id += 1;
        Transaction::new(self.current_tx_id, timestamp)
    }

    pub fn commit(&mut self, disk: &mut Disk, sb: &Superblock, tx: &Transaction) -> Result<()> {
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

        // Write header to journal
        let start_logical = self.next_journal_block;
        let mut current_j_block = start_logical;
        
        let header_p_block = sb.journal_start + (current_j_block % sb.journal_blocks);
        disk.write_block(header_p_block, bytes_of(&header))?;
        current_j_block += 1;

        // Write records
        for (&p_block, data) in &tx.dirty_blocks {
            let data_checksum = compute_checksum(data);
            let mut rec_header = JournalRecordHeader {
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

        // Now apply to actual disk locations
        for (&p_block, data) in &tx.dirty_blocks {
            disk.write_block(p_block, data)?;
        }

        // Flush final data
        disk.sync()?;

        self.next_journal_block = current_j_block % sb.journal_blocks;
        Ok(())
    }
}
