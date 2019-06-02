//! This crate aims to give people better control of how they allocate memory,
//! by providing a customizable way to allocate blocks of memory, that optionally
//! contains metadata about the block itself. This makes it much easier to implement
//! Dynamically-Sized Types (DSTs), and also reduces the number of pointer
//! indirections necessary to share data between threads.
//!
//! It has two main features that provide the foundation for the rest:
//!
//! - **Storing data next to an array:** From the
//!   [Rust documentation on exotically sized types](https://doc.rust-lang.org/nomicon/exotic-sizes.html),
//!   at the end of the section on dynamically-sized types:
//!
//!   > Currently the only properly supported way to create a custom DST is by
//!   > making your type generic and performing an unsizing coercion
//!   > ...
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
//!   type MySuperSliceable = HeapArray<u8, u32>;
//!
//!   fn main() {
//!       let dynamic = MySuperSliceable::with_label(17, 8, |_,_| 0);
//!       println!("{:?}", dynamic);
//!   }
//!   ```
//!
//! - **Thin pointer arrays:** in Rust, unsized structs are referenced with
//!   pointers that are stored with an associated length.
//!   This behavior isn't always desired, so this crate provides
//!   both thin and fat pointer-referenced arrays, where the length is stored
//!   with the data instead of with the pointer in the thin pointer variant.
//!
//! ## Features
//! - Arrays are allocated on the heap, with optional extra space allocated for metadata
//! - Support for 1-word and 2-word pointers
//! - Atomically reference-counted memory blocks of arbitrary size without using a `Vec`;
//!   this means you can access reference-counted memory with only a single pointer
//!   indirection.
//! - Swap owned objects in and out with `array.insert()`
//! - Arbitrarily sized objects using label and an array of bytes (`u8`)
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
//! ## Use of `unsafe` Keyword
//! This library relies heavily on the use of the `unsafe` keyword to do both
//! reference counting and atomic operations; there are 40 instances total,
//! not including tests.
//!
//! ## Customizability
//! All of the implementation details of this crate are public and documented;
//! if you'd like to implement your own version of the tools available through
//! this crate, note that you don't need to reinvent the wheel; many of the types in
//! this crate are generic over certain traits, so you might not need to do that much.

extern crate containers_rs as containers;

pub mod alloc_utils;
mod api;
pub mod base;
pub mod mem_block;
pub mod naive_rc;
mod traits;

mod prelude {
    pub(crate) use super::mem_block::*;
    pub use super::traits::*;
    pub use containers::{Container, CopyMap};
    pub(crate) use core::fmt;
    pub(crate) use core::mem;
    pub(crate) use core::mem::ManuallyDrop;
    pub(crate) use core::ops::{Index, IndexMut};
    pub(crate) use core::ptr;
}

pub use api::*;
