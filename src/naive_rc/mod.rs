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
//! let array_ref = ArcArray::new(10, |_| 0);
//! let another_ref = ArrayRef::clone(&array_ref);
//! ```

pub mod fat_rc_array;
mod generic;
pub mod ref_counters;
pub mod thin_arc_array;

pub use crate::prelude::*;
pub use fat_rc_array::FpRcArray as RcArray;
pub use thin_arc_array::TpArcArray as ArcArray;

/// publicly include this before every reference counting module, as it contains
/// all the traits necessary to use the structures correctly.
pub(crate) mod prelude {
    pub(crate) use super::ref_counters::*;
    pub(crate) use crate::fat_array_ptr::FatPtrArray;
    pub use crate::prelude::*;
    pub(crate) use crate::thin_array_ptr::ThinPtrArray;
}
