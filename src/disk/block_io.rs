use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write, Result};
use std::path::Path;

use crate::ondisk::serialization::BLOCK_SIZE;

pub struct Disk {
    file: File,
}

impl Disk {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        Ok(Self { file })
    }

    pub fn create<P: AsRef<Path>>(path: P, size_bytes: u64) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        file.set_len(size_bytes)?;
        Ok(Self { file })
    }

    pub fn read_block(&mut self, block_num: u64, buf: &mut [u8]) -> Result<()> {
        self.file.seek(SeekFrom::Start(block_num * BLOCK_SIZE as u64))?;
        self.file.read_exact(buf)?;
        Ok(())
    }

    pub fn write_block(&mut self, block_num: u64, buf: &[u8]) -> Result<()> {
        self.file.seek(SeekFrom::Start(block_num * BLOCK_SIZE as u64))?;
        self.file.write_all(buf)?;
        Ok(())
    }

    pub fn sync(&mut self) -> Result<()> {
        self.file.sync_all()
    }
}
