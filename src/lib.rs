//! This crate holds a struct, `HeapArray`, that internally points to a
//! contiguous block of memory. It also supports storing arbitrary data
//! adjacent to the block of memory.
//!
//! ## Examples
//!
//! Creating an array:
//! ```rust
//! use heaparray::*;
//! let len = 10;
//! let array = HeapArray::new(len, |idx| idx + 3);
//! assert!(array[1] == 4);
//! ```
//!
//! Indexing works as you would expect:
//! ```rust
//! # use heaparray::*;
//! # let mut array = HeapArray::new(10, |idx| idx + 3);
//! array[3] = 2;
//! assert!(array[3] == 2);
//! ```
//!
//! Notably, you can take ownership of objects back from the container:
//!
//! ```rust
//! # use heaparray::*;
//! let mut array = HeapArray::new(10, |_| Vec::<u8>::new());
//! let replacement_object = Vec::new();
//! let owned_object = array.insert(0, replacement_object);
//! ```
//!
//! but you need to give the array a replacement object to fill its slot with.
//!
//! Additionally, you can customize what information should be stored alongside the elements in
//! the array using the `HeapArray::with_label` function:
//!
//! ```rust
//! # use heaparray::*;
//! struct MyLabel {
//!     pub even: usize,
//!     pub odd: usize,
//! }
//!
//! let mut array = HeapArray::with_label(
//!     MyLabel { even: 0, odd: 0 },
//!     100,
//!     |label, index| {
//!         if index % 2 == 0 {
//!             label.even += 1;
//!             index
//!         } else {
//!             label.odd += 1;
//!             index
//!         }
//!     });
//! ```

// TODO uncomment this when the alloc crate hits stable
// extern crate alloc;
extern crate containers_rs as containers;

pub mod alloc_utils;
mod api;
pub mod fat_array_ptr;
pub mod memory_block;
pub mod naive_rc;
pub mod thin_array_ptr;
mod traits;

mod prelude {
    pub(crate) use super::memory_block::*;
    #[cfg(test)]
    pub(crate) use super::test_utils::*;
    pub use super::traits::*;
    pub use containers::{Array, Container, CopyMap};
    pub(crate) use core::mem;
    pub(crate) use core::mem::ManuallyDrop;
    pub(crate) use core::ops::{Index, IndexMut};
}

pub use api::*;

#[cfg(all(test, not(bench)))]
extern crate interloc;

#[cfg(all(test, not(bench)))]
use interloc::*;

#[cfg(all(test, not(bench)))]
use tests::monitor::*;

#[cfg(all(test, not(bench)))]
use std::alloc::System;

#[cfg(all(test, not(bench)))]
static TEST_MONITOR: TestMonitor = TestMonitor::new();

#[cfg(all(test, not(bench)))]
#[global_allocator]
static GLOBAL: InterAlloc<System, TestMonitor> = InterAlloc {
    inner: System,
    monitor: &TEST_MONITOR,
};

#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod tests;
