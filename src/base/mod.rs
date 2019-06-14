//! Module for simple operations on memory.

mod alloc_utils;
mod base;
mod mem_block;
mod ptr_utils;

pub use crate::traits::AtomicArrayRef;
pub use base::{BaseArray, BaseArrayIter};
pub use mem_block::MemBlock;
