//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

// Specification: rCore Guide chapter4/7exercise.html.
/// Return time through checked, possibly non-contiguous user pages.
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = crate::timer::get_time_us();
    let mut bytes = [0u8; core::mem::size_of::<TimeVal>()];
    bytes[..8].copy_from_slice(&(us / 1_000_000).to_ne_bytes());
    bytes[8..].copy_from_slice(&(us % 1_000_000).to_ne_bytes());
    if crate::mm::copy_to_user(crate::task::current_user_token(), ts as usize, &bytes) { 0 } else { -1 }
}

/// Trace this task's memory or syscall history.
pub fn sys_trace(request: usize, id: usize, data: usize) -> isize {
    match request {
        0 | 1 => {
            match crate::mm::user_byte(crate::task::current_user_token(), id, request == 1) {
                Some(pa) if request == 0 => unsafe { (pa.0 as *const u8).read() as isize },
                Some(pa) => { unsafe { (pa.0 as *mut u8).write(data as u8); } 0 }
                None => -1,
            }
        }
        2 => crate::task::get_syscall_count(id) as isize,
        _ => -1,
    }
}

/// Map anonymous user memory.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    crate::task::mmap(start, len, prot)
}

/// Remove user memory mappings.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    crate::task::munmap(start, len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
