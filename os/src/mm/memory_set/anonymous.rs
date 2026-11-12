//! Anonymous user mappings shared by the process laboratories.
// Specification: rCore Guide chapter4/7exercise.html.
use super::*;

fn page_range(start: usize, len: usize) -> Option<VPNRange> {
    if start % PAGE_SIZE != 0 { return None; }
    let rounded = start.checked_add(len)?.checked_add(PAGE_SIZE - 1)?;
    if rounded >= 1usize << 38 { return None; }
    Some(VPNRange::new(VirtAddr::from(start).floor(), VirtAddr::from(rounded / PAGE_SIZE * PAGE_SIZE).floor()))
}

impl MemorySet {
    /// Map zeroed user pages, rejecting conflicts before changing the address space.
    pub fn mmap(&mut self, start: usize, len: usize, prot: usize) -> isize {
        if prot == 0 || prot & !7 != 0 { return -1; }
        let range = match page_range(start, len) { Some(range) => range, None => return -1 };
        if range.into_iter().any(|vpn| self.page_table.translate(vpn).map_or(false, |p| p.is_valid())) { return -1; }
        if len == 0 { return 0; }
        let permission = MapPermission::from_bits((prot as u8) << 1).unwrap() | MapPermission::U;
        let mut area = MapArea::new(start.into(), VirtAddr::from(range.get_end()), MapType::Framed, permission);
        for vpn in range {
            let frame = match frame_alloc() {
                Some(frame) => frame,
                None => {
                    for mapped in area.data_frames.keys() { self.page_table.unmap(*mapped); }
                    return -1;
                }
            };
            if !self.page_table.try_map(vpn, frame.ppn, PTEFlags::from_bits(permission.bits).unwrap()) {
                for mapped in area.data_frames.keys() { self.page_table.unmap(*mapped); }
                return -1;
            }
            area.data_frames.insert(vpn, frame);
        }
        self.areas.push(area);
        0
    }

    /// Remove an interval and preserve ownership of the remaining area fragments.
    pub fn munmap(&mut self, start: usize, len: usize) -> isize {
        let range = match page_range(start, len) { Some(range) => range, None => return -1 };
        if range.into_iter().any(|vpn| self.page_table.translate(vpn).map_or(true, |p| !p.is_valid() || !p.flags().contains(PTEFlags::U))) { return -1; }
        if len == 0 { return 0; }
        let first = range.get_start();
        let last = range.get_end();
        let mut i = 0;
        while i < self.areas.len() {
            let lo = self.areas[i].vpn_range.get_start();
            let hi = self.areas[i].vpn_range.get_end();
            if lo >= last || hi <= first { i += 1; continue; }
            let cut_lo = lo.max(first);
            let cut_hi = hi.min(last);
            let mut area = self.areas.remove(i);
            if cut_hi < hi {
                let right = MapArea {
                    vpn_range: VPNRange::new(cut_hi, hi),
                    data_frames: area.data_frames.split_off(&cut_hi),
                    map_type: area.map_type,
                    map_perm: area.map_perm,
                };
                self.areas.insert(i, right);
                i += 1;
            }
            for vpn in VPNRange::new(cut_lo, cut_hi) { area.unmap_one(&mut self.page_table, vpn); }
            if lo < cut_lo {
                area.vpn_range = VPNRange::new(lo, cut_lo);
                self.areas.insert(i, area);
                i += 1;
            }
        }
        0
    }
}
