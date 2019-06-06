//! Contains the struct `MemBlock`, which handles pointer math and very low-level
//! interactions with memory.
use super::alloc_utils::*;
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr;

// TODO Make this function a const function when if statements are stabilized in
// const functions
/// Get the maximum length of a memory block, based on the types that it contains.
/// Returns a length that ensures that no more than
pub fn block_max_len<E, L>() -> usize {
    let max_len = core::isize::MAX as usize;
    if mem::size_of::<E>() == 0 {
        max_len - 1
    } else {
        let (lsize, lalign) = size_align::<L>();
        let (esize, ealign) = size_align::<E>();
        let align = core::cmp::max(lalign, ealign);
        let ((lsize, _), (esize, _)) = (ensure_align(lsize, align), ensure_align(esize, align));
        (max_len - lsize) / esize - 1
    }
}

/// An array block that can hold arbitrary information, and cannot be
/// constructed on the stack. The `E` type can be repeated an arbitrary
/// number of times, and the `L` type can be repeated exactly once.
///
/// It's not recommended to use this directly; instead, use the pointer
/// types that refer to these, namely `HeapArray`, `FatPtrArray`, and
/// `ThinPtrArray`.
///
/// # Invariants
/// The code for this struct does little to no runtime checks for validity;
/// thus, you need to do them yourself. Length checks, runtime validity
/// checks, etc. all need to be done at runtime before using the methods
/// associated with this type, as forgetting to do so leads to undefined
/// behavior (likely a segmentation fault or an invalid pointer
/// dereference). To avoid these errors, you can instead use the safe API
/// provided by the `ThinPtrArray` and `FatPtrArray` structs, or by the
/// `HeapArray` struct, which internally is implemented as a `FatPtrArray`.
#[repr(C)]
pub struct MemBlock<E, L = ()> {
    /// Metadata about the block
    pub label: ManuallyDrop<L>,
    /// First element in the block
    elements: ManuallyDrop<E>,
}

impl<E, L> MemBlock<E, L> {
    /// Get size and alignment of the memory that a block of length `len`
    /// would need.
    pub fn memory_layout(len: usize) -> (usize, usize) {
        let l_layout = size_align::<L>();
        if len == 0 {
            l_layout
        } else {
            let d_layout = size_align_array::<E>(len);
            size_align_multiple(&[l_layout, d_layout])
        }
    }

    /// Deallocates a reference to this struct, as well as all objects
    /// contained in it. This function is safe given that the following
    /// preconditions hold:
    ///
    /// - The reference `&self` is valid, and points to a valid instance
    ///   of this type
    /// - The memory block pointed to by `&self` was initialized with
    ///   length of `len`
    /// - The operation of dereferencing an element at index `i`, where
    ///   `0 <= i < len`, accesses valid memory that has been
    ///   properly initialized. This is NOT checked at runtime.
    pub unsafe fn dealloc(&mut self, len: usize) {
        ptr::drop_in_place(&mut *self.label);
        for i in 0..len {
            ptr::drop_in_place(self.get_ptr(i));
        }
        self.dealloc_lazy(len);
    }

    /// Deallocates a reference to this struct, without running the
    /// destructor of the elements it contains. This function is safe
    /// given that the following preconditions hold:
    ///
    /// - The reference `&self` is valid, and points to a valid instance
    ///   of this type
    /// - The memory block pointed to by `&self` was initialized with length
    ///   of `len`
    ///
    /// This function *may* leak memory; be sure to run destructors on
    /// all initialized elements in this block before calling this method,
    /// as they may in accessible afterwards if you don't.
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
    /// use core::mem;
    /// let len = 100;
    /// let block = unsafe { &mut *MemBlock::<usize, ()>::new((), len) };
    /// for i in 0..len {
    ///     let item = i * i;
    ///     let garbage = mem::replace(unsafe { &mut *block.get_ptr(i) }, item);
    ///     mem::forget(garbage);
    /// }
    /// ```
    pub unsafe fn new<'a>(label: L, len: usize) -> *mut Self {
        #[cfg(not(feature = "no-asserts"))]
        assert!(len <= block_max_len::<E, L>());

        let (size, align) = Self::memory_layout(len);
        let new_ptr = allocate::<Self>(size, align);
        ptr::write(&mut (&mut *new_ptr).label, ManuallyDrop::new(label));
        new_ptr
    }

    /// Returns a pointer to a labelled memory block, with elements initialized
    /// using the provided function. Function is safe, because the following
    /// invariants will always hold:
    ///
    /// - A memory access `block.get(i)` where `0 <= i < len` will always
    ///   be valid.
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
        let new_ptr = unsafe { &mut *Self::new(label, len) };
        for i in 0..len {
            let item = func(&mut new_ptr.label, i);
            let garbage = mem::replace(unsafe { &mut *new_ptr.get_ptr(i) }, item);
            mem::forget(garbage);
        }
        new_ptr
    }

    pub fn get_ptr(&self, idx: usize) -> *mut E {
        let element = (&*self.elements) as *const E as *mut E;
        unsafe { element.add(idx) }
    }
}
