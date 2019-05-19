pub use crate::prelude::*;

pub use crate::base::FatPtrArray as HeapArray;
pub use crate::naive_rc::FpRcArray as RcArray;
/// Atomically reference counted array, referenced using a raw pointer.
///
/// See the documentation for `heaparray::naive_rc::generic::RcArray`
/// for more information on API. Note that this implementation satisfies
/// the trait bound requirements for `AtomicArrayRef`, and so you can
/// alter its pointer atomically:
///
/// ```rust
/// use heaparray::*;
/// use core::sync::atomic::Ordering;
/// let array = ArcArray::new(100, |_| 12);
/// let null = ArcArray::null_ref();
/// let null_ref = null.as_ref();
/// let null_ref = array.compare_and_swap(null_ref, null, Ordering::Relaxed);
/// ```
pub use crate::naive_rc::TpArcArray as ArcArray;
