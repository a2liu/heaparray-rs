use super::alloc::*;
use lock_api::{RawMutex as RawMutexTrait, RawRwLock as RawRwLockTrait};
use parking_lot::{RawMutex, RawRwLock};
use std::thread::{current, ThreadId};

/// Information about allocs by the allocator
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct AllocInfo {
    // Taken directly from https://github.com/neoeinstein/stats_alloc, or stats_alloc
    // on crates.io - all credit to the original writer of this struct, the user
    // neoeinstein on GitHub.
    /// Number of calls to alloc
    pub allocs: usize,
    /// Number of calls to dealloc
    pub deallocs: usize,
    /// Number of calls to realloc
    pub reallocs: usize,
    /// Total bytes allocated
    pub bytes_alloc: usize,
    /// Total bytes deallocated
    pub bytes_dealloc: usize,
    /// Total bytes reallocated
    pub bytes_realloc: isize,
}

impl AllocInfo {
    pub fn relative_to(&self, origin: &Self) -> Self {
        Self {
            allocs: self.allocs - origin.allocs,
            deallocs: self.deallocs - origin.deallocs,
            reallocs: self.reallocs - origin.reallocs,
            bytes_alloc: self.bytes_alloc - origin.bytes_alloc,
            bytes_dealloc: self.bytes_dealloc - origin.bytes_dealloc,
            bytes_realloc: self.bytes_realloc - origin.bytes_realloc,
        }
    }
}

pub struct TestMonitor {
    to_writer: RawMutex,
    authorize: RawRwLock,
    data: Option<ThreadId>,
    allocate: RawRwLock,
}

impl TestMonitor {
    pub const fn new() -> Self {
        Self {
            to_writer: RawMutex::INIT,
            authorize: RawRwLock::INIT,
            data: None,
            allocate: RawRwLock::INIT,
        }
    }

    pub fn mem_lock(&self) {
        use std::sync::atomic;
        self.to_writer.lock();
        atomic::fence(atomic::Ordering::SeqCst);
        self.allocate.lock_exclusive();
        atomic::fence(atomic::Ordering::SeqCst);
        self.authorize.lock_exclusive();
        let data = &self.data as *const Option<ThreadId> as *mut Option<ThreadId>;
        atomic::fence(atomic::Ordering::SeqCst);
        unsafe {
            *data = Some(current().id());
        }
        atomic::fence(atomic::Ordering::SeqCst);
        self.authorize.unlock_exclusive();
    }

    pub fn mem_unlock(&self) {
        use std::sync::atomic;
        self.allocate.unlock_exclusive();
        atomic::fence(atomic::Ordering::SeqCst);
        self.authorize.lock_exclusive();
        let data = &self.data as *const Option<ThreadId> as *mut Option<ThreadId>;
        atomic::fence(atomic::Ordering::SeqCst);
        unsafe {
            *data = None;
        }
        atomic::fence(atomic::Ordering::SeqCst);
        self.authorize.unlock_exclusive();
        atomic::fence(atomic::Ordering::SeqCst);
        self.to_writer.unlock();
    }

    // This RwLock  is overflowing:
    //
    // thread '<unnamed>' panicked at 'RwLock reader count overflow', src/libcore/option.rs:1038:5
    //
    // Which is weird because the source code uses an AtomicUsize.
    //
    // Man concurrency is like hard or something.
    pub fn monitor_mem_lock(&self, act: AllocAction) {
        use std::sync::atomic;
        atomic::fence(atomic::Ordering::SeqCst);
        self.authorize.lock_shared();
        atomic::fence(atomic::Ordering::SeqCst);
        if let Some(id) = self.data {
            atomic::fence(atomic::Ordering::SeqCst);
            self.authorize.unlock_shared();
            atomic::fence(atomic::Ordering::SeqCst);
            if id == current().id() {
                return;
            }
        } else {
            atomic::fence(atomic::Ordering::SeqCst);
            self.authorize.unlock_shared();
        }
        atomic::fence(atomic::Ordering::SeqCst);
        if act.relation() == AllocRel::After {
            atomic::fence(atomic::Ordering::SeqCst);
            self.allocate.lock_shared();
        } else {
            atomic::fence(atomic::Ordering::SeqCst);
            self.allocate.unlock_shared();
        }
    }
}

impl AllocMonitor for TestMonitor {
    #[inline]
    fn monitor(&self, _layout: Layout, _act: AllocAction) {
        // self.monitor_mem_lock(act)
    }
}
