mod memory_model;

extern crate containers_rs as containers;
extern crate heaparray;

mod prelude {
    pub use crate::memory_model::test_utils::*;
    pub use heaparray::*;
}

#[cfg(not(bench))]
extern crate interloc;

#[cfg(not(bench))]
use interloc::*;

#[cfg(not(bench))]
use memory_model::monitor::*;

#[cfg(not(bench))]
use std::alloc::System;

#[cfg(not(bench))]
static TEST_MONITOR: TestMonitor = TestMonitor::new();

#[cfg(not(bench))]
#[global_allocator]
static GLOBAL: InterAlloc<System, TestMonitor> = InterAlloc {
    inner: System,
    monitor: &TEST_MONITOR,
};
