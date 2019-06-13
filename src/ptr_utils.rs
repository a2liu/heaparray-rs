use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};

/// Trait representing an unsafe reference to an object. Should be the same size
/// as the underlying pointer
pub trait UnsafePtr<T>: Sized {
    /// Creates a new reference of this type
    fn new(ptr: *mut T) -> Self;
    /// Creates a new reference of this type without doing any checks
    unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self::new(ptr)
    }
    /// Returns whether or not this pointer is null
    fn is_null(&self) -> bool;
    /// Returns a reference to the underlying data that this pointer represents
    unsafe fn as_ref(&self) -> &T;
    /// Returns a mutable reference to the underlying data that this pointer
    /// represents
    unsafe fn as_mut(&mut self) -> &mut T;
    /// Casts this pointer to another value
    unsafe fn cast<E, P>(&self) -> P
    where
        P: UnsafePtr<E>,
    {
        P::new(self.as_ref() as *const T as *const E as *mut E)
    }
}

impl<T> UnsafePtr<T> for NonNull<T> {
    fn new(ptr: *mut T) -> Self {
        Self::new(ptr).unwrap()
    }
    unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self::new_unchecked(ptr)
    }
    fn is_null(&self) -> bool {
        self.as_ptr().is_null()
    }
    unsafe fn as_ref(&self) -> &T {
        self.as_ref()
    }
    unsafe fn as_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<T> UnsafePtr<T> for AtomicPtr<T> {
    fn new(ptr: *mut T) -> Self {
        Self::new(ptr)
    }
    fn is_null(&self) -> bool {
        self.load(Ordering::Acquire).is_null()
    }
    unsafe fn as_ref(&self) -> &T {
        &*self.load(Ordering::Acquire)
    }
    unsafe fn as_mut(&mut self) -> &mut T {
        &mut *self.load(Ordering::Acquire)
    }
}

impl<T> UnsafePtr<T> for *mut T {
    fn new(ptr: *mut T) -> Self {
        ptr
    }
    fn is_null(&self) -> bool {
        (*self).is_null()
    }
    unsafe fn as_ref(&self) -> &T {
        &**self
    }
    unsafe fn as_mut(&mut self) -> &mut T {
        &mut **self
    }
}
