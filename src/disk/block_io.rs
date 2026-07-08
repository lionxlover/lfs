use std::fs::{File, OpenOptions};
use std::io::Result;
use std::path::Path;

use crate::ondisk::serialization::BLOCK_SIZE;

use crate::pool::raid::{RaidEngine, RaidProfile};

use std::os::unix::fs::FileExt;
use rayon::prelude::*;
use std::sync::Arc;

pub struct Disk {
    files: Vec<Arc<File>>,
    pub raid_engine: RaidEngine,
}

impl Disk {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        Ok(Self { 
            files: vec![Arc::new(file)],
            raid_engine: RaidEngine::new(RaidProfile::Single, 0, 1),
        })
    }

    pub fn create<P: AsRef<Path>>(path: P, size_bytes: u64) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        file.set_len(size_bytes)?;
        Ok(Self { 
            files: vec![Arc::new(file)],
            raid_engine: RaidEngine::new(RaidProfile::Single, 0, 1),
        })
    }

    pub fn read_block(&self, block_num: u64, buf: &mut [u8]) -> Result<()> {
        let maps = self.raid_engine.map_read(block_num);
        let (dev_idx, physical_block) = maps[0];
        
        self.files[dev_idx].read_at(buf, physical_block * BLOCK_SIZE as u64)?;
        Ok(())
    }

    pub fn write_block(&self, block_num: u64, buf: &[u8]) -> Result<()> {
        let maps = self.raid_engine.map_write(block_num);
        for (dev_idx, physical_block) in maps {
            self.files[dev_idx].write_at(buf, physical_block * BLOCK_SIZE as u64)?;
        }
        Ok(())
    }

    pub fn write_blocks_parallel(&self, blocks: &[(u64, &[u8])]) -> Result<()> {
        let errors: Vec<_> = blocks.par_iter().filter_map(|(block_num, buf)| {
            self.write_block(*block_num, buf).err()
        }).collect();
        
        if let Some(err) = errors.into_iter().next() {
            return Err(err);
        }
        Ok(())
    }

    pub fn sync(&self) -> Result<()> {
        for file in &self.files {
            file.sync_all()?;
        }
        Ok(())
    }
}
