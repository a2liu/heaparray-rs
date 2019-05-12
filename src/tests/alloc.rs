pub use core::alloc::{GlobalAlloc, Layout};
pub use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};

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

pub enum AllocAction {
    Alloc,
    AllocResult { ptr: *mut u8 },
    AllocZeroed,
    AllocZeroedResult { ptr: *mut u8 },
    Dealloc { ptr: *mut u8 },
    DeallocResult,
    Realloc { ptr: *mut u8, new_size: usize },
    ReallocResult { ptr: *mut u8, new_size: usize },
}

#[derive(Eq, PartialEq)]
pub enum AllocRel {
    Before,
    After,
}

impl AllocAction {
    pub fn relation(&self) -> AllocRel {
        use AllocAction::*;
        match self {
            Alloc | AllocZeroed => AllocRel::Before,
            Dealloc { ptr: _ } => AllocRel::Before,
            Realloc {
                ptr: _,
                new_size: _,
            } => AllocRel::Before,
            _ => AllocRel::After,
        }
    }
}

pub struct InterAlloc<'a, T, F>
where
    T: GlobalAlloc,
    F: AllocMonitor,
{
    // Taken directly from https://github.com/neoeinstein/stats_alloc, or stats_alloc
    // on crates.io - all credit to the original writer of this struct, the user
    // neoeinstein on GitHub.
    /// Number of calls to alloc
    pub allocs: AtomicUsize,
    /// Number of calls to dealloc
    pub deallocs: AtomicUsize,
    /// Number of calls to realloc
    pub reallocs: AtomicUsize,
    /// Total bytes allocated
    pub bytes_alloc: AtomicUsize,
    /// Total bytes deallocated
    pub bytes_dealloc: AtomicUsize,
    /// Total bytes reallocated
    pub bytes_realloc: AtomicIsize,
    pub inner: T,
    pub monitor_struct: &'a F,
    pub info_lock: AtomicBool,
}

impl<'a, T, F> InterAlloc<'a, T, F>
where
    T: GlobalAlloc,
    F: AllocMonitor,
{
    pub fn new(base: T, monitor: &'a F) -> Self {
        Self {
            allocs: AtomicUsize::new(0),
            deallocs: AtomicUsize::new(0),
            reallocs: AtomicUsize::new(0),
            bytes_alloc: AtomicUsize::new(0),
            bytes_dealloc: AtomicUsize::new(0),
            bytes_realloc: AtomicIsize::new(0),
            inner: base,
            monitor_struct: monitor,
            info_lock: AtomicBool::new(false),
        }
    }

    fn monitor(&self, layout: Layout, act: AllocAction) {
        self.monitor_struct.monitor(self.info(), layout, act);
    }

    fn lock(&self) {
        while self
            .info_lock
            .compare_and_swap(false, true, Ordering::Acquire)
        {
            std::thread::sleep(std::time::Duration::new(0, 1));
        }
    }

    fn unlock(&self) {
        self.info_lock
            .compare_and_swap(true, false, Ordering::Release);
    }
    pub fn info(&self) -> AllocInfo {
        self.lock();
        let info = AllocInfo {
            allocs: self.allocs.load(Ordering::SeqCst),
            deallocs: self.deallocs.load(Ordering::SeqCst),
            reallocs: self.reallocs.load(Ordering::SeqCst),
            bytes_alloc: self.bytes_alloc.load(Ordering::SeqCst),
            bytes_dealloc: self.bytes_dealloc.load(Ordering::SeqCst),
            bytes_realloc: self.bytes_realloc.load(Ordering::SeqCst),
        };
        self.unlock();
        info
    }
}

unsafe impl<'a, T, F> GlobalAlloc for InterAlloc<'a, T, F>
where
    T: GlobalAlloc,
    F: AllocMonitor,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.monitor(layout, AllocAction::Alloc);
        self.lock();
        self.allocs.fetch_add(1, Ordering::SeqCst);
        self.bytes_alloc.fetch_add(layout.size(), Ordering::SeqCst);
        self.unlock();
        let ptr = self.inner.alloc(layout);
        self.monitor(layout, AllocAction::AllocResult { ptr });
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.monitor(layout, AllocAction::Dealloc { ptr });
        self.lock();
        self.deallocs.fetch_add(1, Ordering::SeqCst);
        self.bytes_dealloc
            .fetch_add(layout.size(), Ordering::SeqCst);
        self.unlock();
        self.inner.dealloc(ptr, layout);
        self.monitor(layout, AllocAction::DeallocResult);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.monitor(layout, AllocAction::AllocZeroed);
        self.lock();
        self.allocs.fetch_add(1, Ordering::SeqCst);
        self.bytes_alloc.fetch_add(layout.size(), Ordering::SeqCst);
        self.unlock();
        let ptr = self.inner.alloc_zeroed(layout);
        self.monitor(layout, AllocAction::AllocZeroedResult { ptr });
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.monitor(layout, AllocAction::Realloc { ptr, new_size });
        self.lock();
        self.reallocs.fetch_add(1, Ordering::SeqCst);
        if new_size > layout.size() {
            let difference = new_size - layout.size();
            self.bytes_alloc.fetch_add(difference, Ordering::SeqCst);
        } else if new_size < layout.size() {
            let difference = layout.size() - new_size;
            self.bytes_dealloc.fetch_add(difference, Ordering::SeqCst);
        }
        self.bytes_realloc.fetch_add(
            new_size.wrapping_sub(layout.size()) as isize,
            Ordering::SeqCst,
        );
        self.unlock();
        let ptr = self.inner.realloc(ptr, layout, new_size);
        self.monitor(layout, AllocAction::Realloc { ptr, new_size });
        ptr
    }
}

pub trait AllocMonitor {
    fn monitor(&self, info: AllocInfo, layout: Layout, act: AllocAction);
}
