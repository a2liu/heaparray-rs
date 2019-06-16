//! Contains the struct `MemBlock`, which handles pointer math and very low-level
//! interactions with memory.

use super::alloc_utils::*;
use super::traits::*;
use const_utils::{cond, max, safe_div};
use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};

/// An array block that can hold arbitrary information, and cannot be
/// constructed on the stack.
///
/// The label type, `L`, and element type, `E`, are both held in the same block;
/// i.e. this block holds exactly one instance of `L`, and some arbitrary number
/// of instances of `E`.
///
/// A raw pointer to a `MemBlock`, i.e. `*mut MemBlock`, correctly implements the
/// [`BaseArrayPtr`](trait.BaseArrayPtr.html) trait, and can thus be used as the
/// underlying type for a `BaseArray`. Additionally, `AtomicPtr<MemBlock>`
/// and `NonNull<MemBlock>` also correctly implement `BaseArrayPtr`.
///
/// # Invariants
/// The implementation of `MemBlock` holds the necessary invariants to be a valid
/// implementation of `BaseArrayPtr<E, L>`. That means that for mutable raw
/// pointers to a `MemBlock`, the following is true (`a` is a `*mut` pointer to
/// a `MemBlock` and `A` is the class `*mut MemBlock`):
///
/// - `a.dealloc(len)` is safe to call on the result of `A::alloc(len)`
/// - `a.elem_ptr(idx)` and `a.lbl_ptr()` must return properly aligned pointers
///   for the types `E` and `L` respectively
/// - `a.lbl_ptr()` must return the same value for the lifetime
///   of `a` for all `let a = A::alloc(len)`, or at least until `a.dealloc()`
///   is called.
/// - `a.elem_ptr(idx)` must return the same value for each value of `idx` for the
///   lifetime of `a` for all `let a = A::alloc(len)`, or at least until `a.dealloc()`
///   is called.
/// - `A::alloc(len).elem_ptr(idx)` returns a pointer to allocated memory for all
///   `idx < len`
/// - `A::alloc(len).lbl_ptr()` returns a pointer to allocated memory
/// - `a._init()` is safe to call on the result of `A::alloc(len)`
/// - `a._drop()` is safe to call on any result of `A::alloc(len)` for which
///   `_init()` has been called exactly once
///
/// Additionally, `MemBlock` provides additional guarrantees:
///
/// - A `MemBlock` cannot be larger than `core::isize::MAX` bytes
/// - Allocation functions panic instead of returning null pointers
///
/// However, note that the above invariants can be disabled for better performance,
/// as discussed below.
///
/// ### Invariant Invalidation
/// Some crate features invalidate the invariants above. Namely:
///
/// - **`mem-block-skip-size-check`** prevents size checks of the array being
///   created or accessed. This can cause undefined behavior with pointer arithmetic
///   when accessing elements.
/// - **`mem-block-skip-layout-check`** prevents checking whether or not the memory
///   layout of the block you try to allocate is valid on the platform you're
///   allocating it on
/// - **`mem-block-skip-ptr-check`** prevents checking for null pointer allocations
/// - **`mem-block-skip-all`** enables `mem-block-skip-layout-check`,
///   `mem-block-skip-ptr-check`, and `mem-block-skip-size-check`
///
/// Use all of the above with caution, as their behavior is inherently undefined.
#[repr(transparent)]
pub struct MemBlock<E, L = ()> {
    _placeholder: u8,
    _phantom: PhantomData<(E, L, *mut u8)>,
}

type MutMB<E, L> = *mut MemBlock<E, L>;

impl<E, L> MemBlock<E, L> {
    /// Get the maximum length of a `MemBlock`, based on the types that it contains.
    ///
    /// This function is used to maintain the invariant that all `MemBlock` instances
    /// are of size (in bytes) less than or equal to `core::isize::MAX`.
    pub const fn max_len() -> usize {
        let max_len = core::isize::MAX as usize;
        let max_len_calc = {
            let (esize, ealign) = size_align::<E>(1);
            let lsize = aligned_size::<L>(ealign);
            safe_div(max_len - lsize, esize)
        };
        cond(mem::size_of::<E>() == 0, max_len, max_len_calc)
    }

    /// Get size and alignment of the memory that a block of length `len` would need.
    ///
    /// Returns a tuple in the form `(size, align)`
    pub const fn memory_layout(len: usize) -> (usize, usize) {
        let (l_size, l_align) = size_align::<L>(1);
        let (calc_size, calc_align) = {
            let (dsize, dalign) = size_align::<E>(len);
            let l_size = aligned_size::<L>(dalign);
            (l_size + dsize, max(l_align, dalign))
        };
        (
            cond(len == 0, l_size, calc_size),
            cond(len == 0, l_align, calc_align),
        )
    }
}

/// Make sure that a `MemBlock<E, L>` of length `len` isn't too big
fn check_len<E, L>(len: usize) {
    if cfg!(not(feature = "mem-block-skip-size-check")) && len > MemBlock::<E, L>::max_len() {
        panic!(
            "Length {} is invalid: Block cannot be bigger than\
             core::isize::MAX bytes ({} elements)",
            len,
            MemBlock::<E, L>::max_len()
        );
    }
}

/// Get the memory layout of a `MemBlock<E, L>` of length `len`
fn get_layout<E, L>(len: usize) -> Layout {
    check_len::<E, L>(len);
    let (size, align) = MemBlock::<E, L>::memory_layout(len);
    if cfg!(feature = "mem-block-skip-layout-check") {
        unsafe { Layout::from_size_align_unchecked(size, align) }
    } else {
        match Layout::from_size_align(size, align) {
            Ok(layout) => layout,
            Err(err) => {
                panic!(
                    "MemBlock of length {} is invalid for this platform;\n\
                     it has (size, align) = ({}, {}), causing error\n{:#?}",
                    len, size, align, err
                );
            }
        }
    }
}

unsafe impl<E, L> BaseArrayPtr<E, L> for *mut MemBlock<E, L> {
    unsafe fn alloc(len: usize) -> Self {
        let layout = get_layout::<E, L>(len);
        let ptr = allocate(layout);
        if cfg!(feature = "mem-block-skip-ptr-check") {
            ptr
        } else {
            assert!(
                !ptr.is_null(),
                "Allocated a null pointer.\
                 You may be out of memory.",
            );
            ptr
        }
    }
    unsafe fn dealloc(&mut self, len: usize) {
        let layout = get_layout::<E, L>(len);
        deallocate(*self, layout);
    }
    unsafe fn from_ptr(ptr: *mut u8) -> Self {
        ptr as *mut MemBlock<E, L>
    }
    fn as_ptr(&self) -> *mut u8 {
        self.clone() as *const u8 as *mut u8
    }
    fn is_null(&self) -> bool {
        self.clone().is_null()
    }
    fn lbl_ptr(&self) -> *mut L {
        *self as *mut L
    }
    fn elem_ptr(&self, idx: usize) -> *mut E {
        check_len::<E, L>(idx + 1);
        let e_align = mem::align_of::<E>();
        let lsize = aligned_size::<L>(e_align);
        let element = unsafe { (*self as *mut u8).add(lsize) as *mut E };
        unsafe { element.add(idx) }
    }
}

unsafe impl<E, L> BaseArrayPtr<E, L> for NonNull<MemBlock<E, L>> {
    unsafe fn alloc(len: usize) -> Self {
        NonNull::new_unchecked(MutMB::alloc(len))
    }
    unsafe fn dealloc(&mut self, len: usize) {
        self.clone().as_ptr().dealloc(len)
    }
    unsafe fn from_ptr(ptr: *mut u8) -> Self {
        NonNull::new_unchecked(MutMB::from_ptr(ptr))
    }
    fn as_ptr(&self) -> *mut u8 {
        (*self).cast::<u8>().as_ptr()
    }
    fn is_null(&self) -> bool {
        (*self).as_ptr().is_null()
    }
    fn lbl_ptr(&self) -> *mut L {
        (*self).as_ptr().lbl_ptr()
    }
    fn elem_ptr(&self, idx: usize) -> *mut E {
        (*self).as_ptr().elem_ptr(idx)
    }
}

unsafe impl<E, L> BaseArrayPtr<E, L> for AtomicPtr<MemBlock<E, L>> {
    unsafe fn alloc(len: usize) -> Self {
        AtomicPtr::new(MutMB::alloc(len))
    }
    unsafe fn dealloc(&mut self, len: usize) {
        self.load(Ordering::Acquire).dealloc(len)
    }
    unsafe fn from_ptr(ptr: *mut u8) -> Self {
        AtomicPtr::new(MutMB::from_ptr(ptr))
    }
    fn as_ptr(&self) -> *mut u8 {
        self.load(Ordering::Acquire) as *mut u8
    }
    fn is_null(&self) -> bool {
        self.load(Ordering::Acquire).is_null()
    }
    fn lbl_ptr(&self) -> *mut L {
        self.load(Ordering::Acquire).lbl_ptr()
    }
    fn elem_ptr(&self, idx: usize) -> *mut E {
        self.load(Ordering::Acquire).elem_ptr(idx)
    }
}
