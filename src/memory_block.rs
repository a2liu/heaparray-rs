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
    /// Get size and alignment of the memory that this struct uses.
    pub fn memory_layout(len: usize) -> (usize, usize) {
        let l_layout = size_align::<L>();
        let d_layout = size_align_array::<E>(len);
        size_align_multiple(&[l_layout, size_align::<usize>(), d_layout])
    }

    /// Deallocates a reference to this struct.
    pub unsafe fn dealloc<'a>(&'a mut self) {
        let (size, align) = TPArrayBlock::<L, E>::memory_layout(self.len());
        deallocate(self, size, align);
    }

    /// Get a mutable reference to a new block. Array elements are initialized to
    /// garbage (i.e. they are not initialized).
    pub unsafe fn new_ptr_unsafe<'a>(label: L, len: usize) -> &'a mut Self {
        let (size, align) = Self::memory_layout(len);
        let new_ptr = allocate::<Self>(size, align);
        new_ptr.label = label;
        new_ptr.len = len;
        new_ptr
    }

    /// Create a new pointer to an array, using a function to initialize all the
    /// elements.
    pub fn new_ptr<'a, F>(label: L, len: usize, mut func: F) -> &'a mut Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let new_ptr = unsafe { Self::new_ptr_unsafe(label, len) };
        for i in 0..new_ptr.len() {
            new_ptr[i] = func(&mut new_ptr.label, i);
        }
        new_ptr
    }

    /// Get a reference to an element in this memory block.
    #[inline]
    pub fn get<'a>(&'a self, idx: usize) -> &'a E {
        assert!(idx < self.len());
        unsafe { self.unchecked_access(idx) }
    }

    /// Get a mutable reference to an element in this memory block.
    #[inline]
    pub fn get_mut<'a>(&'a mut self, idx: usize) -> &'a mut E {
        assert!(idx < self.len());
        unsafe { self.unchecked_access(idx) }
    }

    /// Unsafe access to an element at an index in the block.
    #[inline]
    pub unsafe fn unchecked_access<'a>(&'a self, idx: usize) -> &'a mut E {
        let element = &self.elements as *const E as *mut E;

        // TODO what if I make a buffer of u8 whose size overflows an isize?
        let element = element.offset(idx as isize);
        &mut *element
    }

    /// Get the capacity of this memory block
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<L, E> TPArrayBlock<L, E>
where
    E: Default,
{
    /// Get a mutable reference to a new block.
    #[inline]
    pub fn new_ptr_default<'a>(label: L, len: usize) -> &'a mut Self {
        let new_ptr = Self::new_ptr(label, len, |_, _| E::default());
        new_ptr
    }
}

impl<L, E> Index<usize> for TPArrayBlock<L, E> {
    type Output = E;
    #[inline]
    fn index(&self, index: usize) -> &E {
        assert!(index < self.len());
        self.get(index)
    }
}

impl<L, E> IndexMut<usize> for TPArrayBlock<L, E> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut E {
        assert!(index < self.len());
        self.get_mut(index)
    }
}

impl<L, E> Clone for &mut TPArrayBlock<L, E>
where
    L: Clone,
    E: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let new_ptr =
            TPArrayBlock::new_ptr(self.label.clone(), self.len(), |_, idx| self[idx].clone());
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
    /// Get a mutable reference to a new block. Array elements are initialized to
    /// garbage (i.e. they are not initialized).
    pub unsafe fn new_ptr_unsafe<'a>(label: L, len: usize) -> &'a mut Self {
        let l_layout = size_align::<L>();
        let d_layout = size_align_array::<E>(len);
        let (size, align) = size_align_multiple(&[l_layout, d_layout]);
        let new_ptr = allocate::<E>(size, align);
        let new_ptr = std::slice::from_raw_parts(new_ptr, len);
        let new_ptr = &mut *(new_ptr as *const [E] as *mut [E] as *mut Self);
        new_ptr.label = label;
        new_ptr
    }

    /// Create a new pointer to an array, using a function to initialize all the
    /// elements
    pub fn new_ptr<'a, F>(label: L, len: usize, mut func: F) -> &'a mut Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let new_ptr = unsafe { Self::new_ptr_unsafe(label, len) };
        for i in 0..new_ptr.len() {
            new_ptr[i] = func(&mut new_ptr.label, i);
        }
        new_ptr
    }

    /// Get a reference to an element in this memory block.
    #[inline]
    pub fn get<'a>(&'a self, idx: usize) -> &'a E {
        assert!(idx < self.len());
        &self.elements[idx]
    }

    /// Get a mutable reference to an element in this memory block.
    #[inline]
    pub fn get_mut<'a>(&'a mut self, idx: usize) -> &'a mut E {
        assert!(idx < self.len());
        &mut self.elements[idx]
    }

    /// Unsafe access to an element at an index in the block.
    #[inline]
    pub unsafe fn unchecked_access(&self, idx: usize) -> &mut E {
        let mut_self = &mut *(self as *const Self as *mut Self);
        mut_self.elements.get_unchecked_mut(idx)
    }

    /// Get the capacity of this memory block
    #[inline]
    pub fn len(&self) -> usize {
        self.elements.len()
    }
}

impl<L, E> FPArrayBlock<L, E>
where
    E: Default,
{
    /// Get a mutable reference to a new block, initialized to default values.
    #[inline]
    pub fn new_ptr_default<'a>(label: L, len: usize) -> &'a mut Self {
        let new_ptr = Self::new_ptr(label, len, |_, _| E::default());
        new_ptr
    }
}

impl<L, E> Index<usize> for FPArrayBlock<L, E> {
    type Output = E;
    #[inline]
    fn index(&self, index: usize) -> &E {
        assert!(index < self.len());
        self.get(index)
    }
}

impl<L, E> IndexMut<usize> for FPArrayBlock<L, E> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut E {
        self.get_mut(index)
    }
}

impl<L, E> Clone for &mut FPArrayBlock<L, E>
where
    L: Clone,
    E: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let new_ptr =
            FPArrayBlock::new_ptr(self.label.clone(), self.len(), |_, idx| self[idx].clone());
        new_ptr
    }
}
