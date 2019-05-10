use crate::prelude::*;
use core::ops::{Index, IndexMut};

pub struct FatPtrArray<'a, L, E>
where
    Self: 'a,
{
    data: &'a mut FPArrayBlock<L, E>,
}

impl<'a, L, E> FatPtrArray<'a, L, E> {
    /// Create a new array, with values initialized using a provided function.
    #[inline]
    pub fn new<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        Self {
            data: FPArrayBlock::<L, E>::new_ptr(label, len, func),
        }
    }

    /// Create a new array, without initializing the values in it.
    #[inline]
    pub unsafe fn new_unsafe(label: L, len: usize) -> Self {
        let new_ptr = FPArrayBlock::<L, E>::new_ptr_unsafe(label, len);
        Self { data: new_ptr }
    }

    /// Unsafe access to an element at an index in the array.
    #[inline]
    pub unsafe fn unchecked_access(&'a self, idx: usize) -> &'a mut E {
        self.data.unchecked_access(idx)
    }
}

impl<'a, L, E> FatPtrArray<'a, L, E>
where
    E: Default,
{
    /// Get a new array, initialized to default values.
    #[inline]
    pub fn new_default(label: L, len: usize) -> Self {
        Self {
            data: FPArrayBlock::new_ptr_default(label, len),
        }
    }
}

impl<'a, L, E> LabelledArray<L, E> for FatPtrArray<'a, L, E> {
    /// Get a reference to the label of the array.
    #[inline]
    fn get_label(&self) -> &L {
        &self.data.label
    }

    /// Get a mutable reference to the label of the array.
    #[inline]
    fn get_label_mut(&mut self) -> &mut L {
        &mut self.data.label
    }
}

impl<'a, L, E> Index<usize> for FatPtrArray<'a, L, E> {
    type Output = E;
    #[inline]
    fn index(&self, idx: usize) -> &E {
        &self.data[idx]
    }
}

impl<'a, L, E> IndexMut<usize> for FatPtrArray<'a, L, E> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut E {
        &mut self.data[idx]
    }
}

impl<'a, L, E> Clone for FatPtrArray<'a, L, E>
where
    L: Clone,
    E: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<'a, L, E> Container<(usize, E)> for FatPtrArray<'a, L, E> {
    #[inline]
    fn add(&mut self, elem: (usize, E)) {
        self[elem.0] = elem.1;
    }
    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<'a, L, E> CopyMap<usize, E> for FatPtrArray<'a, L, E> {
    #[inline]
    fn get(&self, key: usize) -> Option<&E> {
        if key > self.len() {
            None
        } else {
            Some(&self[key])
        }
    }
    #[inline]
    fn get_mut(&mut self, key: usize) -> Option<&mut E> {
        if key > self.len() {
            None
        } else {
            Some(&mut self[key])
        }
    }
    #[inline]
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        let ret = Some(unsafe { std::mem::transmute_copy::<E, E>(&self[key]) });

        // TODO Need to check that this doesn't drop the value after
        // assigning
        self[key] = value;
        ret
    }
}

impl<'a, L, E> Array<E> for FatPtrArray<'a, L, E> {}
