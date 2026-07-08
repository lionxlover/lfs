use std::collections::HashMap;
use std::io::Result;
use crate::disk::block_io::Disk;

pub struct Transaction {
    pub id: u64,
    pub dirty_blocks: HashMap<u64, Vec<u8>>,
    pub timestamp: u64,
}

impl Transaction {
    pub fn new(id: u64, timestamp: u64) -> Self {
        Self {
            id,
            dirty_blocks: HashMap::new(),
            timestamp,
        }
    }

    pub fn add_block(&mut self, block_num: u64, data: Vec<u8>) {
        self.dirty_blocks.insert(block_num, data);
    }
}

use crate::cache::node_cache::NodeCache;

pub struct TxContext<'a> {
    pub disk: &'a Disk,
    pub tx: &'a mut Transaction,
    pub node_cache: Option<&'a NodeCache>,
}

impl<'a> TxContext<'a> {
    pub fn new(disk: &'a Disk, tx: &'a mut Transaction) -> Self {
        Self { disk, tx, node_cache: None }
    }

    pub fn with_cache(disk: &'a Disk, tx: &'a mut Transaction, node_cache: &'a NodeCache) -> Self {
        Self { disk, tx, node_cache: Some(node_cache) }
    }

    pub fn read_block(&mut self, block: u64, buf: &mut [u8]) -> Result<()> {
        if let Some(data) = self.tx.dirty_blocks.get(&block) {
            buf.copy_from_slice(data);
            Ok(())
        } else {
            self.disk.read_block(block, buf)
        }
    }

    pub fn write_block(&mut self, block: u64, buf: &[u8]) -> Result<()> {
        self.tx.add_block(block, buf.to_vec());
        Ok(())
    }
}
