#[cfg(not(bench))]
pub mod memory_model;

extern crate containers_rs as containers;
extern crate heaparray;
extern crate interloc;

use interloc::*;
use std::alloc::System;

mod prelude {
    pub use crate::memory_model::test_utils::*;
    pub use heaparray::base::*;
}

pub struct TestMonitor {
    local: ThreadMonitor,
}

impl TestMonitor {
    // This needs to be const to be usable in static functions
    pub const fn new() -> Self {
        Self {
            local: ThreadMonitor::new(),
        }
    }

    pub fn local_info(&self) -> AllocInfo {
        self.local.info()
    }
}

impl AllocMonitor for TestMonitor {
    fn monitor(&self, layout: Layout, action: AllocAction) {
        self.local.monitor(layout, action);
    }
}

static TEST_MONITOR: TestMonitor = TestMonitor::new();

#[global_allocator]
static GLOBAL: InterAlloc<System, TestMonitor> = InterAlloc {
    inner: System,
    monitor: &TEST_MONITOR,
};
