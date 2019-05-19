pub use crate::prelude::*;

pub use crate::base::FatPtrArray as HeapArray;
pub use crate::naive_rc::FpRcArray as RcArray;
/// Atomically reference counted array, referenced using a raw pointer.
///
/// See the documentation for `heaparray::naive_rc::generic::RcArray`
/// for more information on API.
pub use crate::naive_rc::TpArcArray as ArcArray;
