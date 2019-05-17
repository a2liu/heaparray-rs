//! # HeapArray
//! This crate aims to give people better control of how they want to allocate memory,
//! by providing a customizable way to allocate blocks of memory, that optionally contains
//! metadata about the block itself.
//!
//! It provides two main features that provide the foundation for the rest:
//!
//! - **Storing data next to an array:** From the
//!   [Rust documentation on exotically sized types](https://doc.rust-lang.org/nomicon/exotic-sizes.html), at the end of the section on dynamically-sized
//!   types:
//!
//!   > (Yes, custom DSTs are a largely half-baked feature for now.)
//!
//!   This crate aims to provide *some* of that functionality; the code that
//!   the docs give is the following:
//!
//!   ```rust
//!   struct MySuperSliceable<T: ?Sized> {
//!       info: u32,
//!       data: T
//!   }
//!
//!   fn main() {
//!       let sized: MySuperSliceable<[u8; 8]> = MySuperSliceable {
//!           info: 17,
//!           data: [0; 8],
//!       };
//!
//!       let dynamic: &MySuperSliceable<[u8]> = &sized;
//!
//!       // prints: "17 [0, 0, 0, 0, 0, 0, 0, 0]"
//!       println!("{} {:?}", dynamic.info, &dynamic.data);
//!   }
//!   ```
//!
//!   using this crate, the `MySuperSliceable<[u8]>` type would be
//!   implemented like this:
//!
//!   ```rust
//!   use heaparray::*;
//!
//!   fn main() {
//!       let dynamic = HeapArray::<u8,u32>::with_label(17, 8, |_,_| 0);
//!
//!       print!("{} [", dynamic.get_label());
//!       for i in 0..(dynamic.len()-1) {
//!           print!("{:?},", dynamic[i]);
//!       }
//!       println!("{:?}]", dynamic[dynamic.len()-1]);
//!   }
//!   ```
//!
//!   Note that a `Debug` implementation will be available soon, and make
//!   the above code much prettier.
//!
//! - **Thin pointer arrays:** in Rust, unsized structs are referenced with
//!   pointers that are stored with an associated length; these are called fat
//!   pointers. This behavior isn't always desired, so this crate provides
//!   both thin and fat pointer-referenced arrays, where the length is stored
//!   with the data instead of with the pointer in the thin pointer variant.
//!
//! ## Features
//! - Arrays are allocated on the heap, with optional extra space allocated for metadata
//! - 1-word and 2-word references to arrays
//! - Atomically reference-counted memory blocks of arbitrary size without using a `Vec`
//! - Swap owned objects in and out with `array.insert()`
//! - Arbitrarily sized objects using label and an array of bytes (`u8`)
//! - Atomic pointer comparison for the `heaparray::ArcArray` type.
//!
//! ## Examples
//! Creating an array:
//!
//! ```rust
//! use heaparray::*;
//! let len = 10;
//! let array = HeapArray::new(len, |idx| idx + 3);
//! assert!(array[1] == 4);
//! ```
//!
//! Indexing works as you would expect:
//!
//! ```rust
//! use heaparray::*;
//! let mut array = HeapArray::new(10, |_| 0);
//! array[3] = 2;
//! assert!(array[3] == 2);
//! ```
//!
//! You can take ownership of objects back from the container:
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
//! Additionally, you can customize what information should be stored alongside
//! the elements in the array using the `HeapArray::with_label` function:
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
//!
//! ## Use of `unsafe` Keyword
//! This library relies heavily on the use of the `unsafe` keyword to do both
//! reference counting and atomic operations; there are 14 instances total,
//! not including tests.
//!
//! ## Future Plans
//! Iteration, allocator customization, constant-sized array of arbitrary size,
//! i.e. `CArray`, with sizes managed by the type system (waiting on const
//! generics for this one).  See `TODO.md` in the repository for a full
//! list of planned features.

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
