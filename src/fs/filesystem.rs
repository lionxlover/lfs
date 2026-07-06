use std::ffi::OsStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use fuser::{
    Filesystem, Request, ReplyEntry, ReplyAttr, ReplyDirectory, ReplyData, ReplyWrite,
    ReplyCreate, ReplyEmpty, FileAttr, FileType,
};
use libc::{ENOENT, EIO, ENOSPC, ENOTEMPTY, EEXIST};
use crate::disk::block_io::Disk;
use crate::ondisk::serialization::{Superblock, Inode, BLOCK_SIZE, LIONFS_MAGIC};
use bytemuck::{from_bytes, bytes_of};
use crate::transaction::manager::TransactionManager;
use crate::transaction::transaction::{Transaction, TxContext};
use crate::inode::manager::InodeManager;
use crate::directory::entries::DirManager;
use crate::file::writer::FileManager;

const TTL: Duration = Duration::from_secs(1);

pub struct LionFS {
    disk: Disk,
    pub superblock: Superblock,
    tx_manager: TransactionManager,
}

impl LionFS {
    pub fn new(mut disk: Disk) -> std::io::Result<Self> {
        let mut buffer = [0u8; BLOCK_SIZE];
        let mut best_sb: Option<Superblock> = None;
        
        let candidate_locations = [0, 8192, 16384];
        for &loc in &candidate_locations {
            if disk.read_block(loc, &mut buffer).is_ok() {
                let sb: Superblock = *from_bytes(&buffer);
                if sb.magic == LIONFS_MAGIC {
                    let mut sb_copy = sb;
                    let saved_checksum = sb_copy.checksum;
                    sb_copy.checksum = 0;
                    if crate::utils::crc::compute_checksum(bytes_of(&sb_copy)) == saved_checksum {
                        if let Some(current_best) = &best_sb {
                            if sb.generation > current_best.generation {
                                best_sb = Some(sb);
                            }
                        } else {
                            best_sb = Some(sb);
                        }
                    }
                }
            }
        }

        let superblock = match best_sb {
            Some(sb) => sb,
            None => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "No valid superblock found or all checksums failed")),
        };

        // Recover journal if any
        let highest_tx = crate::recovery::recovery::RecoveryManager::recover(&mut disk, &superblock)?;
        let mut tx_manager = TransactionManager::new(&superblock);
        if highest_tx > tx_manager.current_tx_id {
            tx_manager.current_tx_id = highest_tx;
        }
        
        Ok(Self { disk, superblock, tx_manager })
    }
    
    fn get_inode(&mut self, ino: u64) -> std::io::Result<Inode> {
        let mut tx = self.tx_manager.begin(0);
        let mut ctx = TxContext::new(&mut self.disk, &mut tx);
        crate::inode::manager::InodeManager::read_inode(&mut ctx, self.superblock.inode_table_start, ino)
    }
    
    fn write_inode(&mut self, inode: &Inode) -> std::io::Result<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut tx = self.tx_manager.begin(now);
        {
            let mut ctx = TxContext::new(&mut self.disk, &mut tx);
            crate::inode::manager::InodeManager::write_inode(&mut ctx, self.superblock.inode_table_start, inode)?;
        }
        self.tx_manager.commit(&mut self.disk, &self.superblock, &tx)
    }

    fn to_file_attr(&self, inode: &Inode) -> FileAttr {
        let kind = if inode.mode & libc::S_IFMT as u32 == libc::S_IFDIR as u32 {
            FileType::Directory
        } else {
            FileType::RegularFile
        };
        
        FileAttr {
            ino: inode.ino,
            size: inode.size,
            blocks: (inode.size + BLOCK_SIZE as u64 - 1) / BLOCK_SIZE as u64,
            atime: UNIX_EPOCH + Duration::from_secs(inode.atime as u64),
            mtime: UNIX_EPOCH + Duration::from_secs(inode.mtime as u64),
            ctime: UNIX_EPOCH + Duration::from_secs(inode.ctime as u64),
            crtime: UNIX_EPOCH + Duration::from_secs(inode.ctime as u64),
            kind,
            perm: (inode.mode & 0o777) as u16,
            nlink: inode.links_count,
            uid: inode.uid,
            gid: inode.gid,
            rdev: 0,
            blksize: BLOCK_SIZE as u32,
            flags: inode.flags,
        }
    }
}

impl Filesystem for LionFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let mut tx = self.tx_manager.begin(0);
        let mut ctx = TxContext::new(&mut self.disk, &mut tx);
        
        if let Ok(parent_inode) = crate::inode::manager::InodeManager::read_inode(&mut ctx, self.superblock.inode_table_start, parent) {
            if let Ok(entries) = crate::directory::entries::DirManager::read_entries(&mut ctx, &parent_inode) {
                let name_str = name.to_string_lossy();
                for entry in entries {
                    if entry.name == name_str {
                        if let Ok(inode) = crate::inode::manager::InodeManager::read_inode(&mut ctx, self.superblock.inode_table_start, entry.ino) {
                            reply.entry(&TTL, &self.to_file_attr(&inode), 0);
                            return;
                        }
                    }
                }
            }
        }
        reply.error(ENOENT);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        if let Ok(inode) = self.get_inode(ino) {
            reply.attr(&TTL, &self.to_file_attr(&inode));
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        let mut tx = self.tx_manager.begin(0);
        let mut ctx = TxContext::new(&mut self.disk, &mut tx);
        
        if let Ok(inode) = crate::inode::manager::InodeManager::read_inode(&mut ctx, self.superblock.inode_table_start, ino) {
            if let Ok(entries) = crate::directory::entries::DirManager::read_entries(&mut ctx, &inode) {
                let mut dir_entries = vec![
                    (inode.ino, FileType::Directory, ".".to_string()),
                    (inode.ino, FileType::Directory, "..".to_string()), // Simplified parent for Phase 1
                ];
                
                for entry in entries {
                    let kind = if entry.file_type == 2 { FileType::Directory } else { FileType::RegularFile };
                    dir_entries.push((entry.ino, kind, entry.name));
                }
                
                for (i, entry) in dir_entries.into_iter().enumerate().skip(offset as usize) {
                    if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                        break;
                    }
                }
                reply.ok();
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, size: u32, _flags: i32, _lock_owner: Option<u64>, reply: ReplyData) {
        let mut tx = self.tx_manager.begin(0);
        let mut ctx = TxContext::new(&mut self.disk, &mut tx);
        if let Ok(inode) = crate::inode::manager::InodeManager::read_inode(&mut ctx, self.superblock.inode_table_start, ino) {
            if let Ok(data) = crate::file::writer::FileManager::read_file(&mut ctx, &inode, offset as u64, size as u64) {
                reply.data(&data);
                return;
            }
        }
        reply.error(EIO);
    }

    fn write(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, data: &[u8], _write_flags: u32, _flags: i32, _lock_owner: Option<u64>, reply: ReplyWrite) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut tx = self.tx_manager.begin(now);
        let mut success = false;
        let mut bytes_written = 0;
        
        {
            let mut ctx = TxContext::new(&mut self.disk, &mut tx);
            if let Ok(mut inode) = crate::inode::manager::InodeManager::read_inode(&mut ctx, self.superblock.inode_table_start, ino) {
                if let Ok(()) = crate::file::writer::FileManager::write_file(&mut ctx, self.superblock.bitmap_start, self.superblock.total_blocks, &mut inode, offset as u64, data) {
                    inode.mtime = now as i64;
                    let _ = crate::inode::manager::InodeManager::write_inode(&mut ctx, self.superblock.inode_table_start, &inode);
                    success = true;
                    bytes_written = data.len() as u32;
                }
            }
        }
        
        if success {
            if self.tx_manager.commit(&mut self.disk, &self.superblock, &tx).is_ok() {
                reply.written(bytes_written);
                return;
            }
        }
        reply.error(EIO);
    }

    fn mkdir(&mut self, req: &Request, parent: u64, name: &OsStr, mode: u32, _umask: u32, reply: ReplyEntry) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut tx = self.tx_manager.begin(now);
        let mut success = false;
        let mut final_inode = None;
        
        {
            let mut ctx = TxContext::new(&mut self.disk, &mut tx);
            if let Ok(mut parent_inode) = crate::inode::manager::InodeManager::read_inode(&mut ctx, self.superblock.inode_table_start, parent) {
                if let Ok(new_ino) = crate::inode::manager::InodeManager::allocate_inode(&mut ctx, self.superblock.inode_table_start, self.superblock.inode_count) {
                    let new_inode = Inode {
                        ino: new_ino,
                        mode: mode | libc::S_IFDIR,
                        uid: req.uid(),
                        gid: req.gid(),
                        links_count: 2,
                        flags: 0,
                        padding1: 0,
                        size: 0,
                        ctime: now as i64,
                        mtime: now as i64,
                        atime: now as i64,
                        extent_count: 0,
                        padding2: 0,
                        padding3: 0,
                        extents: [crate::ondisk::serialization::Extent { logical_start: 0, physical_start: 0, length: 0 }; 7],
                        checksum: 0,
                        padding4: [0; 12],
                    };
                    
                    if crate::inode::manager::InodeManager::write_inode(&mut ctx, self.superblock.inode_table_start, &new_inode).is_ok() {
                        if crate::directory::entries::DirManager::add_entry(&mut ctx, self.superblock.bitmap_start, self.superblock.total_blocks, &mut parent_inode, name, new_ino, 2).is_ok() {
                            parent_inode.mtime = now as i64;
                            let _ = crate::inode::manager::InodeManager::write_inode(&mut ctx, self.superblock.inode_table_start, &parent_inode);
                            
                            // Also need to add . and .. to new dir
                            let _ = crate::directory::entries::DirManager::add_entry(&mut ctx, self.superblock.bitmap_start, self.superblock.total_blocks, &mut new_inode.clone(), std::ffi::OsStr::new("."), new_ino, 2);
                            let _ = crate::directory::entries::DirManager::add_entry(&mut ctx, self.superblock.bitmap_start, self.superblock.total_blocks, &mut new_inode.clone(), std::ffi::OsStr::new(".."), parent, 2);
                            
                            success = true;
                            final_inode = Some(new_inode);
                        }
                    }
                }
            }
        }
        
        if success {
            if self.tx_manager.commit(&mut self.disk, &self.superblock, &tx).is_ok() {
                reply.entry(&TTL, &self.to_file_attr(&final_inode.unwrap()), 0);
                return;
            }
        }
        reply.error(EIO);
    }

    fn create(&mut self, req: &Request, parent: u64, name: &OsStr, mode: u32, _umask: u32, _flags: i32, reply: ReplyCreate) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut tx = self.tx_manager.begin(now);
        let mut success = false;
        let mut final_inode = None;

        {
            let mut ctx = TxContext::new(&mut self.disk, &mut tx);
            if let Ok(mut parent_inode) = crate::inode::manager::InodeManager::read_inode(&mut ctx, self.superblock.inode_table_start, parent) {
                if let Ok(new_ino) = crate::inode::manager::InodeManager::allocate_inode(&mut ctx, self.superblock.inode_table_start, self.superblock.inode_count) {
                    let new_inode = Inode {
                        ino: new_ino,
                        mode: mode | libc::S_IFREG,
                        uid: req.uid(),
                        gid: req.gid(),
                        links_count: 1,
                        flags: 0,
                        padding1: 0,
                        size: 0,
                        ctime: now as i64,
                        mtime: now as i64,
                        atime: now as i64,
                        extent_count: 0,
                        padding2: 0,
                        padding3: 0,
                        extents: [crate::ondisk::serialization::Extent { logical_start: 0, physical_start: 0, length: 0 }; 7],
                        checksum: 0,
                        padding4: [0; 12],
                    };
                    
                    if crate::inode::manager::InodeManager::write_inode(&mut ctx, self.superblock.inode_table_start, &new_inode).is_ok() {
                        if crate::directory::entries::DirManager::add_entry(&mut ctx, self.superblock.bitmap_start, self.superblock.total_blocks, &mut parent_inode, name, new_ino, 1).is_ok() {
                            parent_inode.mtime = now as i64;
                            let _ = crate::inode::manager::InodeManager::write_inode(&mut ctx, self.superblock.inode_table_start, &parent_inode);
                            
                            success = true;
                            final_inode = Some(new_inode);
                        }
                    }
                }
            }
        }
        
        if success {
            if self.tx_manager.commit(&mut self.disk, &self.superblock, &tx).is_ok() {
                reply.created(&TTL, &self.to_file_attr(&final_inode.unwrap()), 0, 0, 0);
                return;
            }
        }
        reply.error(EIO);
    }

    fn unlink(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut tx = self.tx_manager.begin(now);
        let mut success = false;
        
        {
            let mut ctx = TxContext::new(&mut self.disk, &mut tx);
            if let Ok(mut parent_inode) = crate::inode::manager::InodeManager::read_inode(&mut ctx, self.superblock.inode_table_start, parent) {
                if let Ok(Some(target_ino)) = crate::directory::entries::DirManager::remove_entry(&mut ctx, self.superblock.bitmap_start, self.superblock.total_blocks, &mut parent_inode, name) {
                    if let Ok(mut target_inode) = crate::inode::manager::InodeManager::read_inode(&mut ctx, self.superblock.inode_table_start, target_ino) {
                        target_inode.links_count -= 1;
                        if target_inode.links_count == 0 {
                            target_inode.mode = 0; // free inode
                            let _ = crate::inode::manager::InodeManager::write_inode(&mut ctx, self.superblock.inode_table_start, &target_inode);
                        } else {
                            let _ = crate::inode::manager::InodeManager::write_inode(&mut ctx, self.superblock.inode_table_start, &target_inode);
                        }
                        
                        parent_inode.mtime = now as i64;
                        let _ = crate::inode::manager::InodeManager::write_inode(&mut ctx, self.superblock.inode_table_start, &parent_inode);
                        success = true;
                    }
                }
            }
        }
        
        if success {
            if self.tx_manager.commit(&mut self.disk, &self.superblock, &tx).is_ok() {
                reply.ok();
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn rename(&mut self, _req: &Request, parent: u64, name: &OsStr, newparent: u64, newname: &OsStr, _flags: u32, reply: ReplyEmpty) {
        if parent == newparent {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let mut tx = self.tx_manager.begin(now);
            let mut success = false;
            
            {
                let mut ctx = TxContext::new(&mut self.disk, &mut tx);
                if let Ok(mut p_inode) = crate::inode::manager::InodeManager::read_inode(&mut ctx, self.superblock.inode_table_start, parent) {
                    if let Ok(Some(target_ino)) = crate::directory::entries::DirManager::remove_entry(&mut ctx, self.superblock.bitmap_start, self.superblock.total_blocks, &mut p_inode, name) {
                        if let Ok(target_inode) = crate::inode::manager::InodeManager::read_inode(&mut ctx, self.superblock.inode_table_start, target_ino) {
                            let file_type = if (target_inode.mode & libc::S_IFDIR) != 0 { 2 } else { 1 };
                            if crate::directory::entries::DirManager::add_entry(&mut ctx, self.superblock.bitmap_start, self.superblock.total_blocks, &mut p_inode, newname, target_ino, file_type).is_ok() {
                                p_inode.mtime = now as i64;
                                let _ = crate::inode::manager::InodeManager::write_inode(&mut ctx, self.superblock.inode_table_start, &p_inode);
                                success = true;
                            }
                        }
                    }
                }
            }
            
            if success {
                if self.tx_manager.commit(&mut self.disk, &self.superblock, &tx).is_ok() {
                    reply.ok();
                    return;
                }
            }
        }
        reply.error(ENOENT);
    }
}
