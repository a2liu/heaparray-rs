//! Memory blocks that can hold arbitrary data on the heap.
//! Used to represent the data that all the other types point to.
//!
//! It's not recommended to use these directly; instead, use the pointer types
//! that refer to these, namely `HeapArray`, `FatPtrArray`, and `ThinPtrArray`.
//!
//! *NOTE:* `TPArrayBlock` is marked by the compiler as "Sized". This is incorrect,
//! and thus it's not suggested that you use this type directly. It's suggested
//! that you use one of either `FatPtrArray` or `ThinPtrArray`, cooresponding to
//! `FPArrayBlock` and `TPArrayBlock` respectively.
//!
//! # Invariants
//! These are assumptions that safe code follows, and are maintained by the safe
//! subset of the API to this struct. Please note that the unsafe API does NOT
//! maintain these invariants, but still assumes them to be true.
//!
//! - A valid memory block cannot have a `len` of `core::usize::MAX`.
//! - A valid memory block cannot have an valid index that overflows a `usize`
//! - A valid reference to a memory block cannot have the value of `core::usize::MAX`
//! - A valid memory block will not be zero-sized
//! - A valid thin-pointer memory block has a correctly set capacity.
//!
//! The invariants above are used to reduce the number of checks in the safe API,
//! as well as to have consistent definitions for null pointers. Again, note that
//! calls to unsafe functions do *NOT* check these invariants for you when doing
//! things like constructing new types.
use super::alloc_utils::*;
use core::mem;
use core::ops::{Index, IndexMut};
pub const NULL: usize = core::usize::MAX;
use core::mem::ManuallyDrop;

// TODO Make this function a const function when if statements are stabilized in
// const functions
/// Get the maximum length of a memory block, based on the types that it contains.
/// Maintains the invariants discussed above.
pub fn block_max_len<E, L>() -> usize {
    use core::usize::MAX as MAX_LEN;
    let elem_size = mem::size_of::<E>();
    let label_size = mem::size_of::<L>();
    if elem_size == 0 {
        MAX_LEN - 1
    } else {
        (MAX_LEN - label_size) / elem_size - 1
    }
}

/// This function checks that the given array block pointer is valid, and if it
/// isn't, it panics with an error message. This prevents unsafe code from seg-faulting
/// when corrupt memory is about to be access, but it's a comparison AND a memory
/// dereference, so it's only enabled during testing.
#[cfg(test)]
fn check_null_tp<E, L>(arr: &TPArrayBlock<E, L>, message: &'static str) {
    assert!(
        arr as *const TPArrayBlock<E, L> as usize != NULL && arr.len != NULL,
        message
    );
}

/// This function checks that the given array block pointer is valid, and if it
/// isn't, it panics with an error message. This prevents unsafe code from seg-faulting
/// when corrupt memory is about to be access, but it's two extra comparisons,
/// so it's only enabled during testing.
#[cfg(test)]
fn check_null_fp<E, L>(arr: &FPArrayBlock<E, L>, message: &'static str) {
    assert!(
        arr.elements.len() != NULL && (&arr.label as *const L as usize) != NULL,
        message
    );
}

/// An array block that keeps size information in the block itself.
/// Can additionally hold arbitrary information about the elements in the container,
/// through the `L` generic type.
///
/// TP stands for Thin Pointer, as the pointer to this block is a single pointer.
#[repr(C)]
pub struct TPArrayBlock<E, L = ()> {
    /// Metadata about the block
    pub label: L,
    /// Capacity of the block
    len: usize,
    /// First element in the block
    elements: ManuallyDrop<E>,
}

impl<E, L> TPArrayBlock<E, L> {
    /// Get size and alignment of the memory that this struct uses.
    pub fn memory_layout(len: usize) -> (usize, usize) {
        let l_layout = size_align::<Self>();
        if len <= 1 {
            l_layout
        } else {
            let d_layout = size_align_array::<E>(len - 1);
            size_align_multiple(&[l_layout, size_align::<usize>(), d_layout])
        }
    }

    /// Deallocates a reference to this struct, as well as all objects contained
    /// in it.
    pub unsafe fn dealloc<'a>(&'a mut self) {
        #[cfg(test)]
        check_null_tp(self, "TPArrayBlock::dealloc: Deallocating null pointer!");
        for i in 0..self.len {
            let val = mem::transmute_copy(&self[i]);
            mem::drop::<E>(val);
        }
        let (size, align) = Self::memory_layout(self.len);
        deallocate(self, size, align);
    }

    /// Get a mutable reference to a new block. Array elements are initialized to
    /// garbage (i.e. they are not initialized).
    pub unsafe fn new_ptr_unsafe<'a>(label: L, len: usize) -> &'a mut Self {
        let (size, align) = Self::memory_layout(len);
        let new_ptr = allocate::<Self>(size, align);
        new_ptr.label = label;
        new_ptr.len = len;
        #[cfg(test)]
        check_null_tp(
            new_ptr,
            "TPArrayBlock::new_ptr_unsafe: Allocated null pointer!",
        );
        new_ptr
    }

    /// Returns a null pointer to a memory block. Dereferencing it is undefined
    /// behavior.
    pub unsafe fn null_ptr() -> *mut Self {
        NULL as *mut Self
    }

    /// Uses the invariants discussed above to check whether a reference to a
    /// memory block is null or not. Shouldn't be necessary unless you're using
    /// the unsafe API.
    pub fn is_null(&self) -> bool {
        self as *const Self as usize == NULL
    }

    /// Create a new pointer to an array, using a function to initialize all the
    /// elements.
    pub fn new_ptr<'a, F>(label: L, len: usize, mut func: F) -> &'a mut Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        assert!(len <= block_max_len::<E, L>());
        let new_ptr = unsafe { Self::new_ptr_unsafe(label, len) };
        for i in 0..new_ptr.len {
            let item = func(&mut new_ptr.label, i);
            let garbage = mem::replace(new_ptr.get_mut(i), item);
            mem::forget(garbage);
            //*item_ref = item; // FIXME Malloc error happening right here. Why?
        }
        new_ptr
    }

    /// Get a reference to an element in this memory block.
    #[inline]
    pub fn get<'a>(&'a self, idx: usize) -> &'a E {
        #[cfg(test)]
        check_null_tp(self, "TPArrayBlock::get: Immutable access of null pointer!");
        assert!(idx < self.len);
        unsafe { self.unchecked_access(idx) }
    }

    /// Get a mutable reference to an element in this memory block.
    #[inline]
    pub fn get_mut<'a>(&'a mut self, idx: usize) -> &'a mut E {
        #[cfg(test)]
        check_null_tp(
            self,
            "TPArrayBlock::get_mut: Mutable access of null pointer!",
        );
        assert!(idx < self.len);
        unsafe { self.unchecked_access(idx) }
    }

    /// Unsafe access to an element at an index in the block.
    #[inline]
    pub unsafe fn unchecked_access<'a>(&'a self, idx: usize) -> &'a mut E {
        #[cfg(test)]
        check_null_tp(
            self,
            "TPArrayBlock::unchecked_access: Memory access on null pointer!",
        );
        let element = &*self.elements as *const E as *mut E;
        let element = element.add(idx);
        &mut *element
    }

    /// Unsafe access to the label of the block.
    #[inline]
    pub unsafe fn unchecked_access_label(&self) -> &mut L {
        #[cfg(test)]
        check_null_tp(
            self,
            "TPArrayBlock::unchecked_access_label: Memory access on null pointer!",
        );
        &mut *(&self.label as *const L as *mut L)
    }

    /// Get the capacity of this memory block
    #[inline]
    pub fn len(&self) -> usize {
        #[cfg(test)]
        check_null_tp(self, "TPArrayBlock::len: Length check of null pointer!");
        self.len
    }
}

impl<E, L> TPArrayBlock<E, L>
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

impl<E, L> Index<usize> for TPArrayBlock<E, L> {
    type Output = E;
    #[inline]
    fn index(&self, idx: usize) -> &E {
        self.get(idx)
    }
}

impl<E, L> IndexMut<usize> for TPArrayBlock<E, L> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut E {
        self.get_mut(index)
    }
}

impl<E, L> Clone for &mut TPArrayBlock<E, L>
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
/// Can additionally hold arbitrary information about the elements in the container,
/// through the `L` generic type.
///
/// FP stands for Fat Pointer, as the pointer to this block is a pointer and an
/// associated capacity.
#[repr(C)]
pub struct FPArrayBlock<E, L = ()> {
    /// Metadata about the block
    pub label: L,
    /// Slice of elememnts in the block
    pub elements: [E],
}

impl<E, L> FPArrayBlock<E, L> {
    /// Get size and alignment of the memory that this struct uses.
    fn memory_layout(len: usize) -> (usize, usize) {
        let l_layout = size_align::<L>();
        let d_layout = size_align_array::<E>(len);
        size_align_multiple(&[l_layout, d_layout])
    }

    /// Get a mutable reference to a new block. Array elements are initialized to
    /// garbage (i.e. they are not initialized).
    pub unsafe fn new_ptr_unsafe<'a>(label: L, len: usize) -> &'a mut Self {
        let (size, align) = Self::memory_layout(len);
        let new_ptr = allocate::<E>(size, align);
        let new_ptr = core::slice::from_raw_parts(new_ptr, len);
        let new_ptr = &mut *(new_ptr as *const [E] as *mut [E] as *mut Self);
        #[cfg(test)]
        check_null_fp(
            new_ptr,
            "FPArrayBlock::new_ptr_unsafe: Allocated null pointer!",
        );
        new_ptr.label = label;
        new_ptr
    }

    /// Deallocates a reference to this struct, as well as all objects contained
    /// in it.
    pub unsafe fn dealloc<'a>(&'a mut self) {
        #[cfg(test)]
        check_null_fp(self, "FPArrayBlock::dealloc: Deallocating null pointer!");

        for i in 0..self.elements.len() {
            let val = mem::transmute_copy(&self[i]);
            mem::drop::<E>(val);
        }
        let (size, align) = Self::memory_layout(self.elements.len());
        deallocate(&mut self.label, size, align);
    }

    /// Returns a null pointer to a memory block. Dereferencing it is undefined
    /// behavior.
    pub unsafe fn null_ptr() -> *mut Self {
        let new_ptr = core::slice::from_raw_parts(NULL as *const E, NULL);
        &mut *(new_ptr as *const [E] as *mut [E] as *mut Self)
    }

    /// Uses the invariants discussed above to check whether a reference to a
    /// memory block is null or not. Shouldn't be necessary unless you're using
    /// the unsafe API.
    pub fn is_null(&self) -> bool {
        (&self.label as *const L as usize) == NULL
    }

    /// Create a new pointer to an array, using a function to initialize all the
    /// elements
    pub fn new_ptr<'a, F>(label: L, len: usize, mut func: F) -> &'a mut Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        assert!(len <= block_max_len::<E, L>());
        let new_ptr = unsafe { Self::new_ptr_unsafe(label, len) };
        for i in 0..new_ptr.len() {
            let item = func(&mut new_ptr.label, i);
            let garbage = mem::replace(new_ptr.get_mut(i), item);
            mem::forget(garbage);
        }
        new_ptr
    }

    /// Get a reference to an element in this memory block.
    #[inline]
    pub fn get<'a>(&'a self, idx: usize) -> &'a E {
        #[cfg(test)]
        check_null_fp(self, "FPArrayBlock::get: Immutable access of null pointer!");
        assert!(idx < self.len());
        &self.elements[idx]
    }

    /// Get a mutable reference to an element in this memory block.
    #[inline]
    pub fn get_mut<'a>(&'a mut self, idx: usize) -> &'a mut E {
        #[cfg(test)]
        check_null_fp(
            self,
            "FPArrayBlock::get_mut: Mutable access of null pointer!",
        );
        assert!(idx < self.len());
        &mut self.elements[idx]
    }

    /// Unsafe access to an element at an index in the block.
    #[inline]
    pub unsafe fn unchecked_access(&self, idx: usize) -> &mut E {
        #[cfg(test)]
        check_null_fp(
            self,
            "FPArrayBlock::unchecked_access: Memory access of null pointer!",
        );
        let mut_self = &mut *(self as *const Self as *mut Self);
        mut_self.elements.get_unchecked_mut(idx)
    }

    /// Unsafe access to the label of the block.
    #[inline]
    pub unsafe fn unchecked_access_label(&self) -> &mut L {
        #[cfg(test)]
        check_null_fp(
            self,
            "FPArrayBlock::unchecked_access_label: Memory access of null pointer!",
        );
        &mut *(&self.label as *const L as *mut L)
    }

    /// Get the capacity of this memory block
    #[inline]
    pub fn len(&self) -> usize {
        #[cfg(test)]
        check_null_fp(self, "FPArrayBlock::len: Length check on null pointer!");
        self.elements.len()
    }
}

impl<E, L> FPArrayBlock<E, L>
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

impl<E, L> Index<usize> for FPArrayBlock<E, L> {
    type Output = E;
    #[inline]
    fn index(&self, index: usize) -> &E {
        assert!(index < self.len());
        self.get(index)
    }
}

impl<E, L> IndexMut<usize> for FPArrayBlock<E, L> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut E {
        self.get_mut(index)
    }
}

impl<E, L> Clone for &mut FPArrayBlock<E, L>
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
