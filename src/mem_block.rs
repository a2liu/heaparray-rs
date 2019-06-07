//! Contains the struct `MemBlock`, which handles pointer math and very low-level
//! interactions with memory.

use super::alloc_utils::*;
use crate::const_utils::{cond, max, safe_div};
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr;

/// Get the maximum length of a memory block, based on the types that it contains.
/// Prevents blocks from allocating more than `core::isize::MAX` bytes.
pub const fn block_max_len<E, L>() -> usize {
    let max_len = core::isize::MAX as usize;
    let max_len_calc = {
        let (esize, ealign) = size_align::<E>();
        let lsize = aligned_size::<L>(ealign);
        safe_div(max_len - lsize, esize)
    };
    cond(mem::size_of::<E>() == 0, max_len, max_len_calc)
}

/// An array block that can hold arbitrary information, and cannot be
/// constructed on the stack. The `E` type can be repeated an arbitrary
/// number of times, and the `L` type can be repeated exactly once.
///
/// It's not recommended to use this directly; instead, use the pointer
/// types that refer to these, namely `BaseArray`, `HeapArray`, `FatPtrArray`, and
/// `ThinPtrArray`.
///
/// # Safety
/// The functions below are safe as long as the following conditions hold:
/// - `core::mem::drop` is never called
///
/// # Invariants
/// These conditions will hold as long as you hold a reference to an instance of
/// `MemBlock` that you haven't deallocated yet.
///
/// - The public field `label` will always be initialized
/// - The memory block allocated will always have a size (in bytes) less than
///   or equal to `core::isize::MAX`
#[repr(C)]
pub struct MemBlock<E, L = ()> {
    /// Metadata about the block. Always safe to access.
    pub label: ManuallyDrop<L>,
    /// First element in the block. May not be initialized.
    elements: ManuallyDrop<E>,
}

impl<E, L> MemBlock<E, L> {
    /// Get size and alignment of the memory that a block of length `len`
    /// would need.
    pub const fn memory_layout(len: usize) -> (usize, usize) {
        let (l_size, l_align) = size_align::<L>();
        let (calc_size, calc_align) = {
            let (dsize, dalign) = size_align_array::<E>(len);
            let (l_size, align) = ensure_align(l_size, max(dalign, l_align));
            (l_size + dsize, align)
        };
        (
            cond(len == 0, l_size, calc_size),
            cond(len == 0, l_align, calc_align),
        )
    }

    /// Returns a `*mut` pointer to an object at index `idx`.
    pub fn get_ptr(&self, idx: usize) -> *mut E {
        #[cfg(not(feature = "no-asserts"))]
        assert!(
            idx < block_max_len::<E, L>(),
            "Index {} is invalid: Block cannot be bigger than core::isize::MAX bytes ({} elements)",
            idx,
            block_max_len::<E, L>()
        );

        // let element = (&*self.elements) as *const E as *mut E;
        let e_align = mem::align_of::<E>();
        let lsize = aligned_size::<L>(e_align);
        let element = unsafe { (&*self.label as *const L as *const u8).add(lsize) as *mut E };
        unsafe { element.add(idx) }
    }

    /// Deallocates a reference to this struct, as well as all objects
    /// contained in it.
    ///
    /// # Safety
    /// This method is safe given that the following preconditions hold:
    ///
    /// - This reference hasn't been deallocated
    /// - The operation of dereferencing an element at index `i`, where
    ///   `i < len`, accesses allocated memory that has been properly initialized.
    ///
    /// Upon being deallocated, this reference is no longer valid.
    pub unsafe fn dealloc(&mut self, len: usize) {
        ManuallyDrop::drop(&mut self.label);
        for i in 0..len {
            ptr::drop_in_place(self.get_ptr(i));
        }
        self.dealloc_lazy(len);
    }

    /// Deallocates a reference to this struct, without running the
    /// destructor of the label or elements it contains.
    ///
    /// **NOTE:** This method will leak memory if your objects have destructors
    /// that deallocate memory themselves, and you forget to call them before
    /// deallocating. If that is not the intended behavior, consider using the
    /// `dealloc` function instead.
    ///
    /// # Safety
    /// This method is safe given that the following preconditions hold:
    ///
    /// - This reference hasn't been deallocated previously
    ///
    pub unsafe fn dealloc_lazy(&mut self, len: usize) {
        let (size, align) = Self::memory_layout(len);
        deallocate(self, size, align);
    }

    /// Returns a pointer to a new memory block on the heap with an
    /// initialized label. Does not initialize memory, so use with care.
    ///
    /// If you use this function, remember to prevent the compiler from
    /// running the destructor for the memory that wasn't initialized. i.e.
    /// something like this:
    ///
    /// ```rust
    /// use heaparray::mem_block::MemBlock;
    /// use core::ptr;
    /// let len = 100;
    /// let block = unsafe { &mut *MemBlock::<usize, ()>::new((), len) };
    /// for i in 0..len {
    ///     let item = i * i;
    ///     unsafe {
    ///         ptr::write(block.get_ptr(i), item);
    ///     }
    /// }
    /// ```
    pub unsafe fn new<'a>(label: L, len: usize) -> *mut Self {
        #[cfg(not(feature = "no-asserts"))]
        assert!(
            len <= block_max_len::<E, L>(),
            "New array of length {} is invalid: Cannot allocate a block larger than core::isize::MAX bytes ({} elements)",
            len,
            block_max_len::<E, L>()
        );

        let (size, align) = Self::memory_layout(len);
        let block = allocate::<Self>(size, align);
        #[cfg(not(feature = "no-asserts"))]
        assert!(
            !block.is_null(),
            "Allocated a null pointer; You may be out of memory."
        );
        let block = &mut *block;

        ptr::write(&mut block.label, ManuallyDrop::new(label));
        block
    }

    /// Returns a pointer to a labelled memory block, with elements initialized
    /// using the provided function. Function is safe, because the following
    /// invariants will always hold:
    ///
    /// - A pointer returned by `block.get_ptr(i)` where `i < len` will always
    ///   point to a valid, aligned instance of `E`
    /// - A memory access `block.label` will always be valid.
    ///
    /// However, note that *deallocating* the resulting block can *never*
    /// be safe; there is not guarrantee provided by the type system
    /// that the block you deallocate will have the length that you assume
    /// it has.
    pub fn new_init<F>(label: L, len: usize, mut func: F) -> *mut Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let block = unsafe { &mut *Self::new(label, len) };
        for i in 0..len {
            let item = func(&mut block.label, i);
            unsafe { ptr::write(block.get_ptr(i), item) }
        }
        block
    }
}
