#[allow(dead_code)]
pub(crate) mod monitor;

mod array_ref;
mod fat_array_ptr;
mod test_utils;
mod thin_array_ptr;

mod prelude {
    use crate::test_utils::*;
}

#[cfg(not(bench))]
extern crate interloc;

#[cfg(not(bench))]
use interloc::*;

#[cfg(not(bench))]
use tests::monitor::*;

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
