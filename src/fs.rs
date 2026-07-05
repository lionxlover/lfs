use crate::disk::DiskManager;
use crate::inode::{
    DirEntry, DiskInode, IndirectBlock, DIRECT_BLOCKS, KIND_DIR, KIND_FILE, KIND_SYMLINK, PAYLOAD_SIZE,
};
use crc32fast::Hasher;
use fuser::{
    BsdFileFlags, Errno, FileHandle, FileType, FopenFlags, Generation, INodeNo, LockOwner, OpenFlags,
    ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyWrite, Request,
    WriteFlags, Filesystem, RenameFlags,
};
use std::ffi::OsStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TTL: Duration = Duration::from_secs(1);
const DIR_ENTRIES_PER_BLOCK: usize = 63;
const INDIRECT_BLOCK_PTRS: usize = 510;

pub struct LionFS {
    pub disk: DiskManager,
}

impl LionFS {
    pub fn new(image_path: &str) -> Self {
        let disk = DiskManager::new(image_path);
        if disk.read_inode(1).is_none() {
            let mut root_inode = DiskInode::new(1, 1, KIND_DIR, 0o777, 1000, 1000);
            if let Some(block) = disk.allocate_block() {
                root_inode.direct_blocks[0] = block;
                disk.write_inode(&root_inode);
            }
        }
        Self { disk }
    }

    fn get_dir_entries(&self, inode: &DiskInode) -> Vec<DirEntry> {
        let mut entries = Vec::new();
        for &block in &inode.direct_blocks {
            if block == 0 {
                continue;
            }
            let mut buf = [0u8; PAYLOAD_SIZE];
            self.disk.read_block(block, &mut buf);
            for i in 0..DIR_ENTRIES_PER_BLOCK {
                let offset = i * 64;
                let entry: &DirEntry = bytemuck::from_bytes(&buf[offset..offset + 64]);
                if entry.ino != 0 {
                    entries.push(*entry);
                }
            }
        }
        if inode.indirect_block != 0 {
            let mut ind_buf = [0u8; PAYLOAD_SIZE];
            self.disk.read_block(inode.indirect_block, &mut ind_buf);
            let indirect: &IndirectBlock = bytemuck::from_bytes(&ind_buf);
            for &block in &indirect.blocks {
                if block == 0 {
                    continue;
                }
                let mut buf = [0u8; PAYLOAD_SIZE];
                self.disk.read_block(block, &mut buf);
                for i in 0..DIR_ENTRIES_PER_BLOCK {
                    let offset = i * 64;
                    let entry: &DirEntry = bytemuck::from_bytes(&buf[offset..offset + 64]);
                    if entry.ino != 0 {
                        entries.push(*entry);
                    }
                }
            }
        }
        entries
    }

    fn add_dir_entry(&self, inode: &mut DiskInode, name: &str, ino: u64) -> Result<(), Errno> {
        let mut name_bytes = [0u8; 56];
        let name_bytes_slice = name.as_bytes();
        if name_bytes_slice.len() > 56 {
            return Err(Errno::ENAMETOOLONG);
        }
        name_bytes[..name_bytes_slice.len()].copy_from_slice(name_bytes_slice);

        let new_entry = DirEntry {
            ino,
            name: name_bytes,
        };
        let new_entry_bytes = bytemuck::bytes_of(&new_entry);

        for (idx, &block) in inode.direct_blocks.iter().enumerate() {
            if block == 0 {
                if let Some(new_block) = self.disk.allocate_block() {
                    inode.direct_blocks[idx] = new_block;
                    inode.size += PAYLOAD_SIZE as u64;
                    let mut buf = [0u8; PAYLOAD_SIZE];
                    buf[0..64].copy_from_slice(new_entry_bytes);
                    self.disk.write_block(new_block, &buf);
                    self.disk.write_inode(inode);
                    return Ok(());
                } else {
                    return Err(Errno::ENOSPC);
                }
            }

            let mut buf = [0u8; PAYLOAD_SIZE];
            self.disk.read_block(block, &mut buf);
            for i in 0..DIR_ENTRIES_PER_BLOCK {
                let offset = i * 64;
                let entry: &DirEntry = bytemuck::from_bytes(&buf[offset..offset + 64]);
                if entry.ino == 0 {
                    buf[offset..offset + 64].copy_from_slice(new_entry_bytes);
                    self.disk.write_block(block, &buf);
                    self.disk.write_inode(inode);
                    return Ok(());
                }
            }
        }
        Err(Errno::ENOSPC)
    }

    fn remove_dir_entry(&self, inode: &DiskInode, name: &str) -> Option<u64> {
        let name_bytes_slice = name.as_bytes();
        for &block in &inode.direct_blocks {
            if block == 0 {
                continue;
            }
            let mut buf = [0u8; PAYLOAD_SIZE];
            self.disk.read_block(block, &mut buf);
            for i in 0..DIR_ENTRIES_PER_BLOCK {
                let offset = i * 64;
                let entry: &mut DirEntry = bytemuck::from_bytes_mut(&mut buf[offset..offset + 64]);
                if entry.ino != 0 {
                    let len = entry.name.iter().position(|&c| c == 0).unwrap_or(56);
                    if &entry.name[..len] == name_bytes_slice {
                        let target_ino = entry.ino;
                        entry.ino = 0;
                        self.disk.write_block(block, &buf);
                        return Some(target_ino);
                    }
                }
            }
        }
        None
    }

    fn get_file_block(
        &self,
        inode: &mut DiskInode,
        idx: usize,
        allocate: bool,
    ) -> Result<(u32, u32), Errno> {
        if idx < DIRECT_BLOCKS {
            let mut block = inode.direct_blocks[idx];
            if block == 0 {
                if allocate {
                    if let Some(new_block) = self.disk.allocate_block() {
                        inode.direct_blocks[idx] = new_block;
                        block = new_block;
                    } else {
                        return Err(Errno::ENOSPC);
                    }
                } else {
                    return Ok((0, 0));
                }
            }
            return Ok((block, inode.direct_checksums[idx]));
        }

        let indirect_idx = idx - DIRECT_BLOCKS;
        if indirect_idx >= INDIRECT_BLOCK_PTRS {
            return Err(Errno::EFBIG);
        }

        if inode.indirect_block == 0 {
            if allocate {
                if let Some(new_block) = self.disk.allocate_block() {
                    inode.indirect_block = new_block;
                } else {
                    return Err(Errno::ENOSPC);
                }
            } else {
                return Ok((0, 0));
            }
        }

        let mut ind_buf = [0u8; PAYLOAD_SIZE];
        self.disk.read_block(inode.indirect_block, &mut ind_buf);

        let mut block = {
            let indirect: &IndirectBlock = bytemuck::from_bytes(&ind_buf);
            indirect.blocks[indirect_idx]
        };

        if block == 0 {
            if allocate {
                if let Some(new_block) = self.disk.allocate_block() {
                    {
                        let indirect: &mut IndirectBlock = bytemuck::from_bytes_mut(&mut ind_buf);
                        indirect.blocks[indirect_idx] = new_block;
                    }
                    block = new_block;
                    self.disk.write_block(inode.indirect_block, &ind_buf);
                } else {
                    return Err(Errno::ENOSPC);
                }
            } else {
                return Ok((0, 0));
            }
        }

        let checksum = {
            let indirect: &IndirectBlock = bytemuck::from_bytes(&ind_buf);
            indirect.checksums[indirect_idx]
        };
        Ok((block, checksum))
    }

    fn set_file_block_checksum(&self, inode: &mut DiskInode, idx: usize, checksum: u32) {
        if idx < DIRECT_BLOCKS {
            inode.direct_checksums[idx] = checksum;
        } else {
            let indirect_idx = idx - DIRECT_BLOCKS;
            if inode.indirect_block != 0 {
                let mut ind_buf = [0u8; PAYLOAD_SIZE];
                self.disk.read_block(inode.indirect_block, &mut ind_buf);
                let indirect: &mut IndirectBlock = bytemuck::from_bytes_mut(&mut ind_buf);
                indirect.checksums[indirect_idx] = checksum;
                self.disk.write_block(inode.indirect_block, &ind_buf);
            }
        }
    }

    fn set_file_block_id(&self, inode: &mut DiskInode, idx: usize, block_id: u32) {
        if idx < DIRECT_BLOCKS {
            inode.direct_blocks[idx] = block_id;
        } else {
            let indirect_idx = idx - DIRECT_BLOCKS;
            if inode.indirect_block != 0 {
                let mut ind_buf = [0u8; PAYLOAD_SIZE];
                self.disk.read_block(inode.indirect_block, &mut ind_buf);
                let indirect: &mut IndirectBlock = bytemuck::from_bytes_mut(&mut ind_buf);
                indirect.blocks[indirect_idx] = block_id;
                self.disk.write_block(inode.indirect_block, &ind_buf);
            }
        }
    }

    fn free_file_blocks(&self, inode: &mut DiskInode, start_idx: usize) {
        for idx in start_idx..DIRECT_BLOCKS {
            if inode.direct_blocks[idx] != 0 {
                self.disk.free_block(inode.direct_blocks[idx]);
                inode.direct_blocks[idx] = 0;
            }
        }

        if inode.indirect_block != 0 && start_idx < DIRECT_BLOCKS + INDIRECT_BLOCK_PTRS {
            let mut ind_buf = [0u8; PAYLOAD_SIZE];
            self.disk.read_block(inode.indirect_block, &mut ind_buf);
            let indirect: &mut IndirectBlock = bytemuck::from_bytes_mut(&mut ind_buf);

            let ind_start = if start_idx > DIRECT_BLOCKS {
                start_idx - DIRECT_BLOCKS
            } else {
                0
            };
            let mut modified = false;

            for i in ind_start..INDIRECT_BLOCK_PTRS {
                if indirect.blocks[i] != 0 {
                    self.disk.free_block(indirect.blocks[i]);
                    indirect.blocks[i] = 0;
                    modified = true;
                }
            }
            if modified {
                self.disk.write_block(inode.indirect_block, &ind_buf);
            }

            if start_idx <= DIRECT_BLOCKS {
                self.disk.free_block(inode.indirect_block);
                inode.indirect_block = 0;
            }
        }
    }
}

impl Filesystem for LionFS {
    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        if let Some(parent_inode) = self.disk.read_inode(parent.into()) {
            if parent_inode.kind == KIND_DIR {
                let name_str = name.to_str().unwrap();
                for entry in self.get_dir_entries(&parent_inode) {
                    let len = entry.name.iter().position(|&c| c == 0).unwrap_or(56);
                    let entry_name = std::str::from_utf8(&entry.name[..len]).unwrap_or("");
                    if entry_name == name_str {
                        if let Some(inode) = self.disk.read_inode(entry.ino) {
                            reply.entry(&TTL, &inode.attr(), Generation(0));
                            return;
                        }
                    }
                }
            }
        }
        reply.error(Errno::ENOENT);
    }

    fn getattr(&self, _req: &Request, ino: INodeNo, _fh: Option<FileHandle>, reply: ReplyAttr) {
        if let Some(inode) = self.disk.read_inode(ino.into()) {
            reply.attr(&TTL, &inode.attr());
        } else {
            reply.error(Errno::ENOENT);
        }
    }

    fn setattr(
        &self,
        _req: &Request,
        ino: INodeNo,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<fuser::TimeOrNow>,
        _mtime: Option<fuser::TimeOrNow>,
        _ctime: Option<std::time::SystemTime>,
        _fh: Option<FileHandle>,
        _crtime: Option<std::time::SystemTime>,
        _chgtime: Option<std::time::SystemTime>,
        _bkuptime: Option<std::time::SystemTime>,
        _flags: Option<BsdFileFlags>,
        reply: ReplyAttr,
    ) {
        if let Some(mut inode) = self.disk.read_inode(ino.into()) {
            if let Some(new_size) = size {
                if new_size < inode.size {
                    let start_idx = ((new_size + PAYLOAD_SIZE as u64 - 1) / PAYLOAD_SIZE as u64) as usize;
                    self.free_file_blocks(&mut inode, start_idx);
                }
                inode.size = new_size;
            }
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            inode.mtime = now;
            self.disk.write_inode(&inode);
            reply.attr(&TTL, &inode.attr());
        } else {
            reply.error(Errno::ENOENT);
        }
    }

    fn readdir(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        mut reply: ReplyDirectory,
    ) {
        if let Some(inode) = self.disk.read_inode(ino.into()) {
            if inode.kind == KIND_DIR {
                let mut entries = vec![
                    (inode.ino, FileType::Directory, ".".to_string()),
                    (inode.parent, FileType::Directory, "..".to_string()),
                ];

                for entry in self.get_dir_entries(&inode) {
                    if let Some(child_inode) = self.disk.read_inode(entry.ino) {
                        let child_type = match child_inode.kind {
                            KIND_DIR => FileType::Directory,
                            KIND_SYMLINK => FileType::Symlink,
                            _ => FileType::RegularFile,
                        };
                        let len = entry.name.iter().position(|&c| c == 0).unwrap_or(56);
                        let name = String::from_utf8_lossy(&entry.name[..len]).into_owned();
                        entries.push((entry.ino, child_type, name));
                    }
                }

                for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
                    if reply.add(INodeNo(entry.0), (i + 1) as u64, entry.1, &entry.2) {
                        break;
                    }
                }
                reply.ok();
                return;
            }
        }
        reply.error(Errno::ENOENT);
    }

    fn create(
        &self,
        req: &Request,
        parent: INodeNo,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        _flags: i32,
        reply: ReplyCreate,
    ) {
        let name_str = name.to_str().unwrap();
        if let Some(mut parent_inode) = self.disk.read_inode(parent.into()) {
            if let Some(ino) = self.disk.allocate_inode() {
                let new_inode = DiskInode::new(
                    ino,
                    parent.into(),
                    KIND_FILE,
                    mode as u16,
                    req.uid(),
                    req.gid(),
                );
                if let Err(e) = self.add_dir_entry(&mut parent_inode, name_str, ino) {
                    self.disk.free_inode(ino);
                    reply.error(e);
                    return;
                }
                self.disk.write_inode(&new_inode);
                reply.created(
                    &TTL,
                    &new_inode.attr(),
                    Generation(0),
                    FileHandle(0),
                    FopenFlags::empty(),
                );
                return;
            } else {
                reply.error(Errno::ENOSPC);
                return;
            }
        }
        reply.error(Errno::ENOENT);
    }

    fn mkdir(
        &self,
        req: &Request,
        parent: INodeNo,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        let name_str = name.to_str().unwrap();
        if let Some(mut parent_inode) = self.disk.read_inode(parent.into()) {
            if let Some(ino) = self.disk.allocate_inode() {
                let mut new_inode = DiskInode::new(
                    ino,
                    parent.into(),
                    KIND_DIR,
                    mode as u16,
                    req.uid(),
                    req.gid(),
                );
                if let Some(block) = self.disk.allocate_block() {
                    new_inode.direct_blocks[0] = block;
                } else {
                    self.disk.free_inode(ino);
                    reply.error(Errno::ENOSPC);
                    return;
                }

                if let Err(e) = self.add_dir_entry(&mut parent_inode, name_str, ino) {
                    self.disk.free_block(new_inode.direct_blocks[0]);
                    self.disk.free_inode(ino);
                    reply.error(e);
                    return;
                }

                parent_inode.nlink += 1;
                self.disk.write_inode(&parent_inode);
                self.disk.write_inode(&new_inode);
                reply.entry(&TTL, &new_inode.attr(), Generation(0));
                return;
            } else {
                reply.error(Errno::ENOSPC);
                return;
            }
        }
        reply.error(Errno::ENOENT);
    }

    fn unlink(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        if let Some(parent_inode) = self.disk.read_inode(parent.into()) {
            let name_str = name.to_str().unwrap();
            if let Some(target_ino) = self.remove_dir_entry(&parent_inode, name_str) {
                if let Some(mut target_inode) = self.disk.read_inode(target_ino) {
                    self.free_file_blocks(&mut target_inode, 0);
                    self.disk.free_inode(target_ino);
                }
                reply.ok();
                return;
            }
        }
        reply.error(Errno::ENOENT);
    }

    fn rmdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        if let Some(mut parent_inode) = self.disk.read_inode(parent.into()) {
            let name_str = name.to_str().unwrap();
            let entries = self.get_dir_entries(&parent_inode);
            let target = entries.iter().find(|e| {
                let len = e.name.iter().position(|&c| c == 0).unwrap_or(56);
                std::str::from_utf8(&e.name[..len]).unwrap_or("") == name_str
            });

            if let Some(target_entry) = target {
                if let Some(mut target_inode) = self.disk.read_inode(target_entry.ino) {
                    let children = self.get_dir_entries(&target_inode);
                    if !children.is_empty() {
                        reply.error(Errno::ENOTEMPTY);
                        return;
                    }
                    self.remove_dir_entry(&parent_inode, name_str);
                    parent_inode.nlink -= 1;
                    self.disk.write_inode(&parent_inode);

                    self.free_file_blocks(&mut target_inode, 0);
                    self.disk.free_inode(target_entry.ino);
                    reply.ok();
                    return;
                }
            }
        }
        reply.error(Errno::ENOENT);
    }

    fn rename(
        &self,
        _req: &Request,
        parent: INodeNo,
        name: &OsStr,
        newparent: INodeNo,
        newname: &OsStr,
        _flags: RenameFlags,
        reply: ReplyEmpty,
    ) {
        let name_str = name.to_str().unwrap();
        let newname_str = newname.to_str().unwrap();
        
        let p_ino: u64 = parent.into();
        let np_ino: u64 = newparent.into();

        if p_ino == np_ino {
            if let Some(mut p_inode) = self.disk.read_inode(p_ino) {
                if let Some(target_ino) = self.remove_dir_entry(&p_inode, name_str) {
                    let _ = self.remove_dir_entry(&p_inode, newname_str); // Overwrite if exists
                    if let Err(e) = self.add_dir_entry(&mut p_inode, newname_str, target_ino) {
                        reply.error(e);
                    } else {
                        reply.ok();
                    }
                    return;
                }
            }
        } else {
            if let (Some(mut old_p), Some(mut new_p)) = (
                self.disk.read_inode(p_ino),
                self.disk.read_inode(np_ino),
            ) {
                if let Some(target_ino) = self.remove_dir_entry(&old_p, name_str) {
                    self.disk.write_inode(&old_p);
                    let _ = self.remove_dir_entry(&new_p, newname_str); // Overwrite if exists
                    if let Err(e) = self.add_dir_entry(&mut new_p, newname_str, target_ino) {
                        let _ = self.add_dir_entry(&mut old_p, name_str, target_ino);
                        reply.error(e);
                    } else {
                        if let Some(mut target_inode) = self.disk.read_inode(target_ino) {
                            target_inode.parent = np_ino;
                            self.disk.write_inode(&target_inode);
                        }
                        reply.ok();
                    }
                    return;
                }
            }
        }
        reply.error(Errno::ENOENT);
    }

    fn symlink(
        &self,
        req: &Request,
        parent: INodeNo,
        name: &OsStr,
        link: &std::path::Path,
        reply: ReplyEntry,
    ) {
        let name_str = name.to_str().unwrap();
        if let Some(mut parent_inode) = self.disk.read_inode(parent.into()) {
            if let Some(ino) = self.disk.allocate_inode() {
                let mut new_inode = DiskInode::new(
                    ino,
                    parent.into(),
                    KIND_SYMLINK,
                    0o777,
                    req.uid(),
                    req.gid(),
                );
                let link_str = link.to_str().unwrap().as_bytes();
                new_inode.size = link_str.len() as u64;

                if let Some(block) = self.disk.allocate_block() {
                    new_inode.direct_blocks[0] = block;
                    let mut buf = [0u8; PAYLOAD_SIZE];
                    buf[..link_str.len()].copy_from_slice(link_str);
                    self.disk.write_block(block, &buf);
                } else {
                    self.disk.free_inode(ino);
                    reply.error(Errno::ENOSPC);
                    return;
                }

                if let Err(e) = self.add_dir_entry(&mut parent_inode, name_str, ino) {
                    self.disk.free_block(new_inode.direct_blocks[0]);
                    self.disk.free_inode(ino);
                    reply.error(e);
                    return;
                }

                self.disk.write_inode(&new_inode);
                reply.entry(&TTL, &new_inode.attr(), Generation(0));
                return;
            } else {
                reply.error(Errno::ENOSPC);
                return;
            }
        }
        reply.error(Errno::ENOENT);
    }

    fn readlink(&self, _req: &Request, ino: INodeNo, reply: ReplyData) {
        if let Some(inode) = self.disk.read_inode(ino.into()) {
            if inode.kind == KIND_SYMLINK {
                let block = inode.direct_blocks[0];
                if block != 0 {
                    let mut buf = [0u8; PAYLOAD_SIZE];
                    self.disk.read_block(block, &mut buf);
                    reply.data(&buf[..inode.size as usize]);
                    return;
                }
            }
        }
        reply.error(Errno::ENOENT);
    }

    fn read(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        size: u32,
        _flags: OpenFlags,
        _lock_owner: Option<LockOwner>,
        reply: ReplyData,
    ) {
        if let Some(mut inode) = self.disk.read_inode(ino.into()) {
            if offset >= inode.size {
                reply.data(&[]);
                return;
            }

            let mut read_size = size as u64;
            if offset + read_size > inode.size {
                read_size = inode.size - offset;
            }

            let start_block_idx = (offset / PAYLOAD_SIZE as u64) as usize;
            let end_block_idx = ((offset + read_size - 1) / PAYLOAD_SIZE as u64) as usize;

            let mut data = Vec::with_capacity(read_size as usize);
            let mut current_offset = offset;

            for idx in start_block_idx..=end_block_idx {
                match self.get_file_block(&mut inode, idx, false) {
                    Ok((block, expected_crc)) => {
                        if block == 0 {
                            break;
                        }
                        let mut buf = [0u8; PAYLOAD_SIZE];
                        self.disk.read_block(block, &mut buf);

                        if expected_crc != 0 {
                            let mut hasher = Hasher::new();
                            hasher.update(&buf);
                            if hasher.finalize() != expected_crc {
                                log::error!(
                                    "DATA CORRUPTION DETECTED in inode {} at block index {}",
                                    inode.ino,
                                    idx
                                );
                                reply.error(Errno::EIO);
                                return;
                            }
                        }

                        let block_offset = (current_offset % PAYLOAD_SIZE as u64) as usize;
                        let chunk_size = std::cmp::min(
                            PAYLOAD_SIZE - block_offset,
                            (offset + read_size - current_offset) as usize,
                        );

                        data.extend_from_slice(&buf[block_offset..block_offset + chunk_size]);
                        current_offset += chunk_size as u64;
                    }
                    Err(e) => {
                        reply.error(e);
                        return;
                    }
                }
            }
            reply.data(&data);
            return;
        }
        reply.error(Errno::ENOENT);
    }

    fn write(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        data: &[u8],
        _write_flags: WriteFlags,
        _flags: OpenFlags,
        _lock_owner: Option<LockOwner>,
        reply: ReplyWrite,
    ) {
        if let Some(mut inode) = self.disk.read_inode(ino.into()) {
            let start_block_idx = (offset / PAYLOAD_SIZE as u64) as usize;
            let end_block_idx = ((offset + data.len() as u64 - 1) / PAYLOAD_SIZE as u64) as usize;

            let mut data_offset = 0;
            let mut current_fs_offset = offset;

            for idx in start_block_idx..=end_block_idx {
                match self.get_file_block(&mut inode, idx, false) {
                    Ok((old_block, _)) => {
                        let mut buf = [0u8; PAYLOAD_SIZE];
                        let block_offset = (current_fs_offset % PAYLOAD_SIZE as u64) as usize;
                        let chunk_size = std::cmp::min(
                            PAYLOAD_SIZE - block_offset,
                            data.len() - data_offset,
                        );

                        if chunk_size < PAYLOAD_SIZE && old_block != 0 {
                            self.disk.read_block(old_block, &mut buf);
                        }

                        buf[block_offset..block_offset + chunk_size]
                            .copy_from_slice(&data[data_offset..data_offset + chunk_size]);

                        // Efficient CoW: Only allocate ONE block
                        let new_block = self.disk.allocate_block().unwrap_or(0);
                        if new_block == 0 {
                            reply.error(Errno::ENOSPC);
                            return;
                        }
                        
                        self.disk.write_block(new_block, &buf);
                        self.set_file_block_id(&mut inode, idx, new_block);

                        let mut hasher = Hasher::new();
                        hasher.update(&buf);
                        self.set_file_block_checksum(&mut inode, idx, hasher.finalize());

                        if old_block != 0 {
                            self.disk.free_block(old_block);
                        }

                        data_offset += chunk_size;
                        current_fs_offset += chunk_size as u64;
                    }
                    Err(e) => {
                        reply.error(e);
                        return;
                    }
                }
            }

            if offset + data.len() as u64 > inode.size {
                inode.size = offset + data.len() as u64;
            }
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            inode.mtime = now;
            self.disk.write_inode(&inode);
            reply.written(data.len() as u32);
            return;
        }
        reply.error(Errno::ENOENT);
    }
}
