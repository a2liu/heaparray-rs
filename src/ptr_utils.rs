use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};

/// Trait representing an unsafe reference to an object
pub trait UnsafeRef<T> {
    /// Returns a reference to the underlying data that this pointer represents
    unsafe fn as_ref(&self) -> &T;
    /// Returns a mutable reference to the underlying data that this pointer
    /// represents
    unsafe fn as_mut(&mut self) -> &mut T;
}

impl<T> UnsafeRef<T> for NonNull<T> {
    unsafe fn as_ref(&self) -> &T {
        self.as_ref()
    }
    unsafe fn as_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<T> UnsafeRef<T> for AtomicPtr<T> {
    unsafe fn as_ref(&self) -> &T {
        &*self.load(Ordering::Acquire)
    }
    unsafe fn as_mut(&mut self) -> &mut T {
        &mut *self.load(Ordering::Acquire)
    }
}

impl<T> UnsafeRef<T> for *mut T {
    unsafe fn as_ref(&self) -> &T {
        &**self
    }
    unsafe fn as_mut(&mut self) -> &mut T {
        &mut **self
    }
}
