//! This module contains naively reference counted arrays, both as atomic and
//! regular versions; i.e. if you're not careful, you could make a cycle that
//! never gets deallocated.
//!
//! The API for `ArcArray` and `RcArray` is the same as for `HeapArray`, with
//! the caveat that `ArcArray::clone()` and `RcArray::clone()` only copy the
//! *reference* to the data, and leave the data itself untouched.
//!
//! Additionally, it is more idiomatic to perform reference clones using the
//! `ArrayRef` trait:
//!
//! ```rust
//! # use heaparray::naive_rc::*;
//! let array_ref = FpRcArray::new(10, |_| 0);
//! let another_ref = ArrayRef::clone(&array_ref);
//! ```

pub mod generic;
pub mod ref_counters;
mod types;

pub use crate::prelude::*;
pub use types::*;
