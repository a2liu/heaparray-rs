//! Module for simple heap arrays. `ThinPtrArray` and `AtomicPtrArray` are
//! a single word on the stack, whereas `FatPtrArray` is a 2-word struct.

// mod atomic;
mod base;
// mod fat;
// pub mod iter;
pub mod mem_block;
// mod thin;

pub use crate::traits::AtomicArrayRef;
// pub use atomic::AtomicPtrArray;
pub use base::{BaseArray, BaseArrayIter};
// pub use fat::FatPtrArray;
// pub use thin::ThinPtrArray;
