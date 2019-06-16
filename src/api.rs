//! Dictates what is imported by the line `use heaparray::*;`

pub use crate::api_prelude::*;
pub use crate::impls::FatPtrArray as HeapArray;

// pub use crate::naive_rc::ArcArray;
// pub use crate::naive_rc::FpRcArray as RcArray;
