//! Contains the struct `MemBlock`, which handles pointer math and very low-level
//! interactions with memory.

use crate::alloc_utils::*;
use crate::const_utils::{cond, max, safe_div};
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr;
use core::ptr::NonNull;

/// An array block that can hold arbitrary information, and cannot be
/// constructed on the stack.
///
/// The label type, `L`, and element type, `E`, are both held in the same block;
/// i.e. this block holds exactly one instance of `L`, and some arbitrary number
/// of instances of `E`.
///
/// It's not recommended to use this type directly; instead, use the safe pointer
/// types that refer to these, namely `HeapArray`, `FatPtrArray`, and
/// `ThinPtrArray`. If you need more low level control of how to initialize your
/// data, try using the `BaseArray` class first.
///
/// # Invariants
/// These conditions will hold as long as you hold a reference to an instance of
/// `MemBlock` that you haven't deallocated yet.
///
/// 1. The public field `label` will always be initialized
/// 2. The memory block allocated will always have a size (in bytes) less than
///    or equal to `core::isize::MAX`
/// 3. Pointers to valid memory blocks cannot be null
///
/// Additional guarrantees are provided by the instantiation functions, `new`
/// and `new_init`.
///
/// ### Invalidation
/// Some crate features invalidate the invariants above. Namely:
/// - TODO
///
/// # Safety of Deallocating References
/// Deallocation methods on `MemBlock` take a `len` argument as a parameter
/// describing the number of instances of `E` that the block stores. In general,
/// deallocation methods on some reference `let r: &mut MemBlock<E,L>` are safe
/// if the following conditions hold, in addition to the invariants discussed above:
///
/// 1. The memory pointed to by `r` has not already been deallocated
/// 2. `r` was allocated with a size, in bytes, large enough to hold `len` many
///    elements; this means that its size is at least the size of `L` aligned
///    to the alignment of `E`, plus the size of `E` times `len`, i.e.
///    `size_of(L).aligned_to(E) + size_of(E) * len`
/// 3. The elements of `r` have all been initialized; i.e. the element pointed to
///    `r.get_ptr(i)` for all `i < len` is initialized to a valid instance of `E`
///
/// The above are sufficient for a memory block to be safely deallocated; depending
/// on the invariants your codebase holds, they may not be necessary.
#[repr(C)]
pub struct MemBlock<E, L = ()> {
    /// Metadata about the block. Will always be initialized on a valid `MemBlock`
    /// instance, as discussed in the invarants section above.
    pub label: ManuallyDrop<L>,
    /// First element in the block. May not be initialized.
    elements: ManuallyDrop<E>,
}

impl<E, L> MemBlock<E, L> {
    /// Get the maximum length of a `MemBlock`, based on the types that it contains.
    ///
    /// This function is used to maintain the invariant that all `MemBlock` instances
    /// are of size (in bytes) less than or equal to `core::isize::MAX`.
    #[cfg(all(not(feature = "mem-block-no-check"), not(release)))]
    pub const fn block_max_len() -> usize {
        let max_len = core::isize::MAX as usize;
        let max_len_calc = {
            let (esize, ealign) = size_align::<E>();
            let lsize = aligned_size::<L>(ealign);
            safe_div(max_len - lsize, esize)
        };
        cond(mem::size_of::<E>() == 0, max_len, max_len_calc)
    }

    /// Get size and alignment of the memory that a block of length `len`
    /// would need.
    ///
    /// Returns a tuple in the form `(size, align)`
    pub const fn memory_layout(len: usize) -> (usize, usize) {
        let (l_size, l_align) = size_align::<L>();
        let (calc_size, calc_align) = {
            let (dsize, dalign) = size_align_array::<E>(len);
            let l_size = aligned_size::<L>(dalign);
            (l_size + dsize, max(l_align, dalign))
        };
        (
            cond(len == 0, l_size, calc_size),
            cond(len == 0, l_align, calc_align),
        )
    }

    /// Returns a `*mut` pointer to an object at index `idx`.
    ///
    /// # Safety
    /// The following must hold to safely dereference the pointer `r.get_ptr(idx)`
    /// for some `let r: &MemBlock<E,L>`:
    ///
    /// 1. The memory pointed to by `r` has not already been deallocated
    /// 2. `r` was allocated with a size, in bytes, large enough to hold at least
    ///     `idx + 1` many elements; this means that its size is at least the
    ///     size of `L` aligned to the alignment of `E`, plus the size of `E`
    ///     times `idx + 1`, i.e. `size_of(L).aligned_to(E) + size_of(E) * (idx + 1)`
    /// 3.  The element pointed to `r.get_ptr(idx)`
    pub fn get_ptr(&self, idx: usize) -> *mut E {
        #[cfg(all(not(feature = "mem-block-no-check"), not(release)))]
        assert!(
            idx < Self::block_max_len(),
            "Index {} is invalid: Block cannot be bigger than core::isize::MAX bytes ({} elements)",
            idx,
            Self::block_max_len()
        );

        // let element = (&*self.elements) as *const E as *mut E;
        let e_align = mem::align_of::<E>();
        let lsize = aligned_size::<L>(e_align);
        let element = unsafe { (self as *const _ as *const u8).add(lsize) as *mut E };
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
    pub unsafe fn new<'a>(label: L, len: usize) -> NonNull<Self> {
        #[cfg(all(not(feature = "mem-block-no-check")))]
        assert!(
            len <= Self::block_max_len(),
            "New array of length {} is invalid: Cannot allocate a block larger than core::isize::MAX bytes ({} elements)",
            len,
            Self::block_max_len()
        );

        let (size, align) = Self::memory_layout(len);

        #[cfg(not(feature = "mem-block-fast-alloc"))]
        let mut block = NonNull::new(allocate::<Self>(size, align))
            .expect("Allocated a null pointer. You may be out of memory.");

        #[cfg(feature = "mem-block-fast-alloc")]
        let mut block = NonNull::new_unchecked(allocate::<Self>(size, align));

        ptr::write(&mut block.as_mut().label, ManuallyDrop::new(label));
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
    pub fn new_init<F>(label: L, len: usize, mut func: F) -> NonNull<Self>
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let mut block = unsafe { Self::new(label, len) };
        let block_ref = unsafe { block.as_mut() };
        for i in 0..len {
            let item = func(&mut block_ref.label, i);
            unsafe { ptr::write(block_ref.get_ptr(i), item) }
        }
        block
    }
}
