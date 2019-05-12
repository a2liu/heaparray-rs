use super::alloc::*;
use lock_api::{RawMutex as RawMutexTrait, RawRwLock as RawRwLockTrait};
use parking_lot::{RawMutex, RawRwLock};
use std::thread::{current, ThreadId};

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
    // Man concurrency is like hard or something.
    pub fn monitor_accesses(&self, act: AllocAction) {
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
    fn monitor(&self, _info: AllocInfo, _layout: Layout, act: AllocAction) {
        // self.monitor_accesses(act)
    }
}
