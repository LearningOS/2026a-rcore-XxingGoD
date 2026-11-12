//! Process-local resource accounting and reduction-based deadlock detection.
// Specification: rCore Guide chapter8/5exercise.html.
use alloc::vec::Vec;

/// One resource family: mutexes or semaphores, checked independently.
#[derive(Default)]
pub struct ResourceTracker {
    available: Vec<usize>,
    allocation: Vec<Vec<usize>>,
    // A thread can be blocked in only one acquisition syscall at a time.
    need: Vec<Option<usize>>,
    work: Vec<usize>,
    finish: Vec<bool>,
}

impl ResourceTracker {
    /// Register a new resource, including resources created after threads start.
    pub fn register(&mut self, id: usize, capacity: usize) {
        if id == self.available.len() {
            self.available.push(capacity);
            for row in &mut self.allocation { row.push(0); }
        } else {
            self.available[id] = capacity;
            for row in &mut self.allocation { row[id] = 0; }
        }
    }

    fn ensure_thread(&mut self, tid: usize) {
        while self.allocation.len() <= tid {
            self.allocation.push(alloc::vec![0; self.available.len()]);
            self.need.push(None);
        }
    }

    /// Announce demand before blocking; rejection leaves no pending request.
    pub fn request(&mut self, tid: usize, resource: usize, enabled: bool) -> bool {
        self.ensure_thread(tid);
        self.need[tid] = Some(resource);
        if enabled && !self.safe() {
            self.need[tid] = None;
            false
        } else { true }
    }

    fn safe(&mut self) -> bool {
        self.work.clone_from(&self.available);
        self.finish.resize(self.allocation.len(), false);
        self.finish.fill(false);
        loop {
            let mut progress = false;
            for tid in 0..self.allocation.len() {
                if self.finish[tid] { continue; }
                if self.need[tid].map_or(false, |id| self.work[id] == 0) { continue; }
                for (free, held) in self.work.iter_mut().zip(&self.allocation[tid]) { *free += *held; }
                self.finish[tid] = true;
                progress = true;
            }
            if !progress { return self.finish.iter().all(|done| *done); }
        }
    }

    /// Record actual acquisition, after a blocked or spinning primitive returns.
    pub fn acquired(&mut self, tid: usize, resource: usize) {
        self.ensure_thread(tid);
        self.need[tid] = None;
        self.available[resource] -= 1;
        self.allocation[tid][resource] += 1;
    }

    /// Release a resource; semaphore producers may up without a preceding down.
    pub fn release(&mut self, tid: usize, resource: usize) {
        self.ensure_thread(tid);
        if self.allocation[tid][resource] > 0 { self.allocation[tid][resource] -= 1; }
        self.available[resource] += 1;
    }

    /// Exited threads cannot be simulated as releasing still-held resources.
    pub fn retire(&mut self, tid: usize) {
        if let Some(row) = self.allocation.get_mut(tid) { row.fill(0); self.need[tid] = None; }
    }
}

/// Detection is opt-in, while accounting remains current across enable/disable.
#[derive(Default)]
pub struct DeadlockState {
    /// Whether new acquisition requests undergo safety checking.
    pub enabled: bool,
    /// Independently tracked mutual-exclusion resources.
    pub mutexes: ResourceTracker,
    /// Independently tracked counting resources.
    pub semaphores: ResourceTracker,
}
