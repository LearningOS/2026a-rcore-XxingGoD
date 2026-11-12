//! Hard links with counts derived from persistent root directory entries.
// rCore Guide chapter6/4exercise.html. Disk inode layout is unchanged.
use super::*;

impl Inode {
    pub(super) fn insert_dirent(&self, name: &str, id: u32, fs: &mut MutexGuard<EasyFileSystem>) {
        self.modify_disk_inode(|dir| {
            let count = dir.size as usize / DIRENT_SZ;
            let mut entry = DirEntry::empty();
            let slot = (0..count).find(|i| {
                dir.read_at(*i * DIRENT_SZ, entry.as_bytes_mut(), &self.block_device);
                entry.name().is_empty()
            }).unwrap_or(count);
            if slot == count { self.increase_size(((count + 1) * DIRENT_SZ) as u32, dir, fs); }
            dir.write_at(slot * DIRENT_SZ, DirEntry::new(name, id).as_bytes(), &self.block_device);
        });
    }

    /// Add another name for an existing file in this directory.
    pub fn link(&self, old: &str, new: &str) -> bool {
        if old == new || new.is_empty() || new.len() > 27 || new.contains('/') { return false; }
        let mut fs = self.fs.lock();
        let id = match self.read_disk_inode(|dir| self.find_inode_id(old, dir)) { Some(id) => id, None => return false };
        if self.read_disk_inode(|dir| self.find_inode_id(new, dir)).is_some() { return false; }
        self.insert_dirent(new, id, &mut fs);
        block_cache_sync_all();
        true
    }

    /// Remove a name; the OS decides when an unlinked open inode can be reclaimed.
    pub fn unlink(&self, name: &str) -> Option<Arc<Inode>> {
        if name.is_empty() { return None; }
        let fs = self.fs.lock();
        let id = self.modify_disk_inode(|dir| {
            let mut entry = DirEntry::empty();
            for i in 0..dir.size as usize / DIRENT_SZ {
                dir.read_at(i * DIRENT_SZ, entry.as_bytes_mut(), &self.block_device);
                if entry.name() == name {
                    let id = entry.inode_id();
                    dir.write_at(i * DIRENT_SZ, DirEntry::empty().as_bytes(), &self.block_device);
                    return Some(id);
                }
            }
            None
        })?;
        let (block, offset) = fs.get_disk_inode_pos(id);
        block_cache_sync_all();
        Some(Arc::new(Inode::new(block, offset, self.fs.clone(), self.block_device.clone())))
    }

    /// Stable inode number and current link count without an on-disk format change.
    pub fn metadata(&self) -> (u64, u32, bool) {
        let fs = self.fs.lock();
        let id = fs.get_disk_inode_id(self.block_id as u32, self.block_offset);
        let directory = self.read_disk_inode(|inode| inode.is_dir());
        if directory { return (id as u64, 1, true); }
        let (block, offset) = fs.get_disk_inode_pos(0);
        let count = get_block_cache(block as usize, self.block_device.clone()).lock().read(offset, |root: &DiskInode| {
            let mut count = 0;
            let mut entry = DirEntry::empty();
            for i in 0..root.size as usize / DIRENT_SZ {
                root.read_at(i * DIRENT_SZ, entry.as_bytes_mut(), &self.block_device);
                if !entry.name().is_empty() && entry.inode_id() == id { count += 1; }
            }
            count
        });
        (id as u64, count, false)
    }

    /// Release an inode after its final link and open-file reference disappear.
    pub fn reclaim(&self) {
        self.clear();
        let fs = self.fs.lock();
        let id = fs.get_disk_inode_id(self.block_id as u32, self.block_offset);
        fs.inode_bitmap.dealloc(&self.block_device, id as usize);
        block_cache_sync_all();
    }
}
