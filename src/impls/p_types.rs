use super::generic::*;
use crate::base::*;
use core::ptr::NonNull;

/// 1-word reference to an array on the heap that takes ownership of its contained
/// data.
pub type ThinPtrArray<E, L> = SafeArray<E, L, ThinArrayPtr<E, L>>;

/// 2-word reference to an array on the heap that takes ownership of its contained
/// data.
pub type FatPtrArray<E, L> = SafeArray<E, L, FatArrayPtr<E, L>>;

struct LenLabel<L> {
    len: usize,
    label: L,
}

type ThinPtr<E, L> = NonNull<MemBlock<E, LenLabel<L>>>;

/// Thin pointer to a memory block, that implements the `BaseArrayPtr` and
/// `SafeArrayPtr` traits.
#[repr(transparent)]
pub struct ThinArrayPtr<E, L> {
    data: ThinPtr<E, L>,
}

unsafe impl<E, L> BaseArrayPtr<E, L> for ThinArrayPtr<E, L> {
    unsafe fn alloc(len: usize) -> Self {
        Self {
            data: ThinPtr::alloc(len),
        }
    }

    unsafe fn dealloc(&mut self, len: usize) {
        self.data.dealloc(len)
    }

    unsafe fn from_ptr(ptr: *mut u8) -> Self {
        Self {
            data: ThinPtr::from_ptr(ptr),
        }
    }

    fn as_ptr(&self) -> *mut u8 {
        (&self.data).as_ptr()
    }

    fn is_null(&self) -> bool {
        self.data.is_null()
    }

    fn lbl_ptr(&self) -> *mut L {
        unsafe { &mut (&mut *self.data.lbl_ptr()).label }
    }

    fn elem_ptr(&self, idx: usize) -> *mut E {
        self.data.elem_ptr(idx)
    }
}

unsafe impl<E, L> SafeArrayPtr<E, L> for ThinArrayPtr<E, L> {
    fn set_len(&mut self, len: usize) {
        unsafe { (&mut *self.data.lbl_ptr()).len = len }
    }
    fn get_len(&self) -> usize {
        unsafe { (*self.data.lbl_ptr()).len }
    }
}

/// Fat pointer to a memory block, that implements the `BaseArrayPtr` and
/// `SafeArrayPtr` traits.
pub struct FatArrayPtr<E, L> {
    data: NonNull<MemBlock<E, L>>,
    len: usize,
}

unsafe impl<E, L> BaseArrayPtr<E, L> for FatArrayPtr<E, L> {
    unsafe fn alloc(len: usize) -> Self {
        Self {
            data: NonNull::alloc(len),
            len: len,
        }
    }

    unsafe fn dealloc(&mut self, len: usize) {
        self.data.dealloc(len)
    }

    unsafe fn from_ptr(ptr: *mut u8) -> Self {
        Self {
            data: NonNull::from_ptr(ptr),
            len: 0,
        }
    }

    fn as_ptr(&self) -> *mut u8 {
        (&self.data).as_ptr()
    }

    fn is_null(&self) -> bool {
        self.data.is_null()
    }

    fn lbl_ptr(&self) -> *mut L {
        self.data.lbl_ptr()
    }

    fn elem_ptr(&self, idx: usize) -> *mut E {
        self.data.elem_ptr(idx)
    }
}

unsafe impl<E, L> SafeArrayPtr<E, L> for FatArrayPtr<E, L> {
    fn set_len(&mut self, len: usize) {
        self.len = len;
    }
    fn get_len(&self) -> usize {
        self.len
    }
}
