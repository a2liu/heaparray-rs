//! Defines `BaseArrayPtr`, the interface `BaseArray` uses when defining methods.

use super::mem_block::MemBlock;
use crate::traits::LabelWrapper;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};

/// Trait representing an unsafe reference to an array.
///
/// Should be the same size as the underlying pointer.
///
/// # Implementation
/// - Destructors for the label and elements are run by the callee;
///   don't implement `drop` unless you have internal state besides the label
///   and elements that needs to be destructed.
/// - Constructors are *also* run by the callee; don't try to initialize elements,
///   as it might result in a memory leak.
pub unsafe trait BaseArrayPtr<E, L>: Sized {
    /// Allocate the memory necessary for a new instance of `len` elements, without
    /// initializing it
    unsafe fn alloc(len: usize) -> Self;

    /// Deallocate the memory for an instance of `len` elements, without running
    /// destructors
    unsafe fn dealloc(&mut self, len: usize);

    /// Creates a new reference of this type without doing any checks
    unsafe fn from_ptr(ptr: *mut u8) -> Self;

    /// Returns the value of the internal raw pointer in this array pointer
    fn as_ptr(&self) -> *mut u8;

    /// Returns whether or not this pointer is null
    fn is_null(&self) -> bool;

    /// Returns a raw pointer to the label associated with this array
    fn lbl_ptr(&self) -> *mut L;

    /// Initializes fields at construction.
    ///
    /// Note that in `BaseArray` this will be run *before* any other initialization
    /// tasks; this means that the memory this method has access to is almost entirely
    /// uninitialized.
    ///
    /// # Safety
    /// Initializing memory that is accessible by dereferencing `lbl_ptr`
    /// or `elem_ptr` is safe, but may potentially result in a memory leak. However,
    /// the memory accessed in this function is not initialized, so reading memory
    /// in this function causes undefined behavior.
    unsafe fn init(&mut self) {}

    /// Returns a raw pointer to the element at `idx`
    ///
    /// Dereferencing this pointer is only safe if there actually is a properly
    /// initialized element at that location
    fn elem_ptr(&self, idx: usize) -> *mut E;

    unsafe fn new_lazy(lbl: L, len: usize) -> Self {
        let obj = Self::alloc(len);
        core::ptr::write(obj.lbl_ptr(), lbl);
        obj
    }

    /// Casts this pointer to another value, by transferring the internal pointer
    /// to its constructor. Super unsafe
    unsafe fn cast<T, Q, P>(&self) -> P
    where
        P: BaseArrayPtr<T, Q>,
    {
        P::from_ptr(self.as_ptr() as *mut u8)
    }
}

unsafe impl<E, L, W> BaseArrayPtr<E, L> for NonNull<MemBlock<E, W>>
where
    W: LabelWrapper<L>,
{
    unsafe fn alloc(len: usize) -> Self {
        MemBlock::<E, W>::alloc(len)
    }

    unsafe fn dealloc(&mut self, len: usize) {
        self.as_mut().dealloc_lazy(len)
    }

    unsafe fn from_ptr(ptr: *mut u8) -> Self {
        NonNull::new_unchecked(ptr as *mut MemBlock<E, W>)
    }

    fn as_ptr(&self) -> *mut u8 {
        self.clone().cast::<u8>().as_ptr()
    }

    fn is_null(&self) -> bool {
        self.as_ptr().is_null()
    }

    fn lbl_ptr(&self) -> *mut L {
        unsafe { self.as_ref().get_label().get_inner() as *const L as *mut L }
    }

    fn elem_ptr(&self, idx: usize) -> *mut E {
        unsafe { self.as_ref().get_ptr(idx) as *mut E }
    }
}

unsafe impl<E, L, W> BaseArrayPtr<E, L> for *mut MemBlock<E, W>
where
    W: LabelWrapper<L>,
{
    unsafe fn alloc(len: usize) -> Self {
        MemBlock::<E, W>::alloc(len).as_ptr()
    }

    unsafe fn dealloc(&mut self, len: usize) {
        (&mut **self).dealloc_lazy(len)
    }

    unsafe fn from_ptr(ptr: *mut u8) -> Self {
        ptr as *mut MemBlock<E, W>
    }

    fn as_ptr(&self) -> *mut u8 {
        self.clone() as *const u8 as *mut u8
    }

    fn is_null(&self) -> bool {
        self.clone().is_null()
    }

    fn lbl_ptr(&self) -> *mut L {
        unsafe { (&mut **self).get_label_mut().get_inner_mut() as *mut L }
    }

    fn elem_ptr(&self, idx: usize) -> *mut E {
        unsafe { (&**self).get_ptr(idx) as *mut E }
    }
}

unsafe impl<E, L, W> BaseArrayPtr<E, L> for AtomicPtr<MemBlock<E, W>>
where
    W: LabelWrapper<L>,
{
    unsafe fn alloc(len: usize) -> Self {
        AtomicPtr::new(MemBlock::<E, W>::alloc(len).as_ptr())
    }

    unsafe fn dealloc(&mut self, len: usize) {
        (&mut *self.load(Ordering::Acquire)).dealloc_lazy(len)
    }

    unsafe fn from_ptr(ptr: *mut u8) -> Self {
        AtomicPtr::new(ptr as *mut MemBlock<E, W>)
    }

    fn as_ptr(&self) -> *mut u8 {
        self.load(Ordering::Acquire) as *mut u8
    }

    fn is_null(&self) -> bool {
        self.load(Ordering::Acquire).is_null()
    }

    fn lbl_ptr(&self) -> *mut L {
        unsafe {
            (&mut *self.load(Ordering::Acquire))
                .get_label_mut()
                .get_inner_mut() as *mut L
        }
    }

    fn elem_ptr(&self, idx: usize) -> *mut E {
        unsafe { (&*self.load(Ordering::Acquire)).get_ptr(idx) as *mut E }
    }
}
