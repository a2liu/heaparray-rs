//! Contains definition for `ArcArray`, which is an atomically reference counted
//! array that can be atomically initialized after construction.

use super::ref_counters::*;
use crate::base::AtomicPtrArray;
pub use crate::prelude::*;
use core::sync::atomic::Ordering;

pub struct ArcArray<E, L = ()> {
    data: AtomicPtrArray<E, L>,
}

impl<E, L> ArcArray<E, L> {
    pub fn null_ref() -> Self {
        Self {
            data: unsafe { AtomicPtrArray::null_ref() },
        }
    }
    pub fn is_null(&self) -> bool {
        self.data.is_null()
    }
    pub fn initialize<F>(&self, label: L, len: usize, func: F)
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let data = AtomicPtrArray::with_label(label, len, func);
        mem::forget(self.data.compare_and_swap(
            unsafe { AtomicPtrArray::<E, L>::null_ref().as_ref() },
            data,
            Ordering::AcqRel,
        ))
    }
}
