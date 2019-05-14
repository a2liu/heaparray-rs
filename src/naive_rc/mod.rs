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

/// A reference to an array, whose clone points to the same data.
///
/// Allows for idiomatic cloning of array references:
///
/// ```rust
/// # use heaparray::naive_rc::*;
/// let array_ref = RcArray::new(10, |_| 0);
/// let another_ref = ArrayRef::clone(&array_ref);
///
/// assert!(array_ref.len() == another_ref.len());
/// for i in 0..another_ref.len() {
///     let r1 = &array_ref[i] as *const i32;
///     let r2 = &another_ref[i] as *const i32;
///     assert!(r1 == r2);
/// }
/// ```
pub trait ArrayRef: Clone {
    // Should this be stricter? It really shouldn't be implemented by other
    // crates, but the type system could definitely make that somewhat of a
    // guarrantee without making the usage of this trait any less ergonomic.
    fn clone(ptr: &Self) -> Self {
        ptr.clone()
    }
}

pub mod fat_arc_array;
pub mod fat_rc_array;
pub mod ref_counters;
pub mod thin_arc_array;
pub mod thin_rc_array;

pub use crate::prelude::*;
pub use fat_arc_array::FpArcArray as ArcArray;
pub use fat_rc_array::FpRcArray as RcArray;

/// publicly include this before every reference counting module, as it contains
/// all the traits necessary to use the structures correctly.
pub(crate) mod prelude {
    pub(crate) use super::ref_counters::*;
    pub use super::ArrayRef;
    pub(crate) use crate::fat_array_ptr::FatPtrArray;
    pub use crate::prelude::*;
    // pub(crate) use crate::thin_array_ptr::ThinPtrArray;
}
