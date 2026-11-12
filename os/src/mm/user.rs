//! Checked user-memory access for syscall results.
// Based on the rCore chapter 4 user-address permission requirements.
use super::{PageTable, PTEFlags, PhysAddr, VirtAddr};
use crate::config::PAGE_SIZE;

/// Translate one readable or writable user byte without dereferencing user VA.
pub fn user_byte(token: usize, address: usize, write: bool) -> Option<PhysAddr> {
    if address >= 1usize << 38 { return None; }
    let va = VirtAddr::from(address);
    let pte = PageTable::from_token(token).translate(va.floor())?;
    let required = PTEFlags::V | PTEFlags::U | if write { PTEFlags::W } else { PTEFlags::R };
    if !pte.flags().contains(required) { return None; }
    Some(PhysAddr::from((pte.ppn().0 << 12) + va.page_offset()))
}

/// Copy an initialized byte slice to user memory, including cross-page results.
pub fn copy_to_user(token: usize, address: usize, bytes: &[u8]) -> bool {
    let end = match address.checked_add(bytes.len()) { Some(end) => end, None => return false };
    let mut cursor = address;
    while cursor < end {
        if user_byte(token, cursor, true).is_none() { return false; }
        cursor += (PAGE_SIZE - cursor % PAGE_SIZE).min(end - cursor);
    }
    let mut offset = 0;
    while offset < bytes.len() {
        let cursor = address + offset;
        let count = (PAGE_SIZE - cursor % PAGE_SIZE).min(bytes.len() - offset);
        let pa = user_byte(token, cursor, true).unwrap();
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr().add(offset), pa.0 as *mut u8, count);
        }
        offset += count;
    }
    true
}
