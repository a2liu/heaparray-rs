//! Module for simple operations on memory.

mod base;
mod mem_block;

pub use crate::traits::AtomicArrayRef;
pub use base::{BaseArray, BaseArrayIter};
pub use mem_block::MemBlock;
