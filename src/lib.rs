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
//! the array using the `HeapArray::new_labelled` function:
//!
//! ```rust
//! # use heaparray::*;
//! struct MyLabel {
//!     pub even: usize,
//!     pub odd: usize,
//! }
//!
//! let mut array = HeapArray::new_labelled(
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

extern crate containers_rs as containers;

pub mod alloc;
mod api;
pub mod fat_array_ptr;
pub mod memory_block;
pub mod naive_rc;
pub mod thin_array_ptr;

mod prelude {
    /// Array with an optional label struct stored next to the data.
    pub trait LabelledArray<'a, E, L>: containers::Array<'a, E>
    where
        E: 'a,
    {
        /// Get immutable access to the label.
        fn get_label(&self) -> &L;
        /// Get mutable reference to the label.
        fn get_label_mut(&mut self) -> &mut L;
    }

    pub(crate) use super::memory_block::*;
    pub use containers::{Array, Container, CopyMap};
    pub(crate) use core::mem;
    pub(crate) use core::mem::ManuallyDrop;
    pub(crate) use core::ops::{Index, IndexMut};
}

pub use api::*;

#[cfg(all(test, not(bench)))]
extern crate parking_lot;

#[cfg(all(test, not(bench)))]
extern crate lock_api;

#[cfg(all(test, not(bench)))]
use tests::alloc::*;

#[cfg(all(test, not(bench)))]
use tests::monitor::*;

#[cfg(all(test, not(bench)))]
use std::alloc::System;

#[cfg(all(test, not(bench)))]
static TEST_MONITOR: TestMonitor = TestMonitor::new();

#[cfg(all(test, not(bench)))]
#[global_allocator]
static GLOBAL: InterAlloc<System, TestMonitor> = InterAlloc {
    allocs: AtomicUsize::new(0),
    deallocs: AtomicUsize::new(0),
    reallocs: AtomicUsize::new(0),
    bytes_alloc: AtomicUsize::new(0),
    bytes_dealloc: AtomicUsize::new(0),
    bytes_realloc: AtomicIsize::new(0),
    inner: System,
    monitor_struct: &TEST_MONITOR,
    info_lock: AtomicBool::new(false),
};

#[cfg(test)]
pub mod tests;
