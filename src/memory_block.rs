//! Memory blocks that can be created on the heap to hold an arbitrary amount
//! of data.
//!
//! *NOTE:* `TPArrayBlock` is marked by the compiler as "Sized". This is incorrect,
//! and thus it's not suggested that you use this type directly.
//!
use super::alloc::*;
use core::ops::{Index, IndexMut};

/// An array block that keeps size information in the block itself.
/// Can additionally hold arbitrary information about the elements in the container,
/// through the `L` generic type.
///
/// TP stands for Thin Pointer, as the pointer to this block is a single pointer.
#[repr(C)]
pub struct TPArrayBlock<L, E> {
    /// Metadata about the block
    pub label: L,
    /// Capacity of the block
    len: usize,
    /// First element in the block
    elements: E,
}

impl<L, E> TPArrayBlock<L, E> {
    /// Get a mutable reference to a new block.
    pub fn new_ptr<'a>(label: L, len: usize) -> &'a mut Self {
        let l_layout = size_align::<L>();
        let d_layout = size_align_array::<E>(len);
        let (size, align) = size_align_multiple(&[l_layout, size_align::<usize>(), d_layout]);
        let new_ptr = unsafe { allocate::<Self>(size, align) };
        new_ptr.label = label;
        new_ptr
    }

    /// Unsafe access to an element at an index in the block.
    #[inline]
    pub unsafe fn unchecked_access<'a>(&'a self, index: usize) -> &'a mut E {
        let element = &self.elements as *const E as *mut E;
        let element = element.offset(index as isize);
        &mut *element
    }

    /// Get the capacity of this memory block
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<L, E> Index<usize> for TPArrayBlock<L, E> {
    type Output = E;
    #[inline]
    fn index(&self, index: usize) -> &E {
        assert!(index < self.len());
        unsafe { self.unchecked_access(index) }
    }
}

impl<L, E> IndexMut<usize> for TPArrayBlock<L, E> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut E {
        assert!(index < self.len());
        unsafe { self.unchecked_access(index) }
    }
}

impl<L, E> Clone for &mut TPArrayBlock<L, E>
where
    L: Clone,
    E: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let new_ptr = TPArrayBlock::new_ptr(self.label.clone(), self.len());
        for i in 0..self.len() {
            new_ptr[i] = self[i].clone();
        }
        new_ptr
    }
}

/// An array block that keeps size information in the pointer to the block.
/// Can additionally hold arbitrary information about this elements in the container,
/// through the `L` generic type.
///
/// FP stands for Fat Pointer, as the pointer to this block is a pointer and an
/// associated capacity.
#[repr(C)]
pub struct FPArrayBlock<L, E> {
    /// Metadata about the block
    pub label: L,
    /// Slice of elememnts in the block
    pub elements: [E],
}

impl<L, E> FPArrayBlock<L, E> {
    /// Get a mutable reference to a new block.
    pub fn new_ptr<'a>(label: L, len: usize) -> &'a mut Self {
        let l_layout = size_align::<L>();
        let d_layout = size_align_array::<E>(len);
        let (size, align) = size_align_multiple(&[l_layout, d_layout]);
        let new_ptr = unsafe {
            let new_ptr = allocate::<E>(size, align);
            let new_ptr = std::slice::from_raw_parts(new_ptr, len);
            &mut *(new_ptr as *const [E] as *mut [E] as *mut Self)
        };
        new_ptr.label = label;
        new_ptr
    }

    /// Unsafe access to an element at an index in the block.
    #[inline]
    pub unsafe fn unchecked_access(&self, index: usize) -> &mut E {
        let elements = &self.elements[index] as *const E as *mut E;
        &mut *elements
    }

    /// Get the capacity of this memory block
    #[inline]
    pub fn len(&self) -> usize {
        self.elements.len()
    }
}

impl<L, E> Index<usize> for FPArrayBlock<L, E> {
    type Output = E;
    #[inline]
    fn index(&self, index: usize) -> &E {
        assert!(index < self.len());
        unsafe { self.unchecked_access(index) }
    }
}

impl<L, E> IndexMut<usize> for FPArrayBlock<L, E> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut E {
        assert!(index < self.len());
        unsafe { self.unchecked_access(index) }
    }
}

impl<L, E> Clone for &mut FPArrayBlock<L, E>
where
    L: Clone,
    E: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let new_ptr = FPArrayBlock::new_ptr(self.label.clone(), self.len());
        for i in 0..self.len() {
            new_ptr[i] = self[i].clone();
        }
        new_ptr
    }
}
