//! Dictates what is imported by the line `use heaparray::*;`

pub use crate::api_prelude_rc::*;
pub use crate::impls::FatPtrArray as HeapArray;

pub use crate::naive_rc::FpArcArray as ArcArray;
pub use crate::naive_rc::FpRcArray as RcArray;
