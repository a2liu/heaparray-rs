pub use crate::prelude::*;

pub use crate::base::FatPtrArray as HeapArray;
pub use crate::naive_rc::FpRcArray as RcArray;
/// Atomically reference counted array, referenced using a raw pointer.
///
/// See the documentation for `heaparray::naive_rc::generic::RcArray`
/// for more information on API.
pub use crate::naive_rc::TpArcArray as ArcArray;

// Note that this implementation satisfies the trait bound requirements for
// AtomicArrayRef, so you can alter its pointer atomically:
//
// ```rust
// use heaparray::*;
// use core::sync::atomic::Ordering;
// let array = ArcArray::new(100, |_| 12);
// let other = ArcArray::new(100, |_| 13);
// let array_ref = array.as_ref();
// let result = array.compare_and_swap(array_ref, other, Ordering::Relaxed);
// ```
