pub use core::alloc::{GlobalAlloc, Layout};

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

#[derive(Clone, Copy, Eq, PartialEq)]
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
    pub inner: T,
    pub monitor_struct: &'a F,
}

impl<'a, T, F> InterAlloc<'a, T, F>
where
    T: GlobalAlloc,
    F: AllocMonitor,
{
    pub fn new(base: T, monitor: &'a F) -> Self {
        Self {
            inner: base,
            monitor_struct: monitor,
        }
    }

    fn monitor(&self, layout: Layout, act: AllocAction) {
        self.monitor_struct.monitor(layout, act);
    }
}

unsafe impl<'a, T, F> GlobalAlloc for InterAlloc<'a, T, F>
where
    T: GlobalAlloc,
    F: AllocMonitor,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.monitor(layout, AllocAction::Alloc);
        let ptr = self.inner.alloc(layout);
        self.monitor(layout, AllocAction::AllocResult { ptr });
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.monitor(layout, AllocAction::Dealloc { ptr });
        self.inner.dealloc(ptr, layout);
        self.monitor(layout, AllocAction::DeallocResult);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.monitor(layout, AllocAction::AllocZeroed);
        let ptr = self.inner.alloc_zeroed(layout);
        self.monitor(layout, AllocAction::AllocZeroedResult { ptr });
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.monitor(layout, AllocAction::Realloc { ptr, new_size });
        let ptr = self.inner.realloc(ptr, layout, new_size);
        self.monitor(layout, AllocAction::Realloc { ptr, new_size });
        ptr
    }
}

pub trait AllocMonitor {
    fn monitor(&self, layout: Layout, act: AllocAction);
}
