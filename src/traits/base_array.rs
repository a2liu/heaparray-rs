/// Trait representing an unsafe reference to an object. Should be the same size
/// as the underlying pointer
pub trait UnsafeArrayPtr<E, L>: Sized {
    /// Allocate the memory necessary for a new instance of `len` elements, without
    /// initializing it
    unsafe fn alloc(len: usize) -> Self;

    /// Creates a new reference of this type without doing any checks
    unsafe fn from_ptr(ptr: *mut u8) -> Self;

    /// Returns the value of the internal raw pointer in this array pointer
    fn as_ptr(&self) -> *mut u8;

    /// Returns whether or not this pointer is null
    fn is_null(&self) -> bool;

    /// Returns a raw pointer to the label associated with this array
    unsafe fn lbl_ptr(&self) -> *mut L;

    /// Returns a raw pointer to the element at `idx`.
    ///
    /// Dereferencing this pointer is only safe if there actually is a properly
    /// initialized element at that location
    fn elem_ptr(&self, idx: usize) -> *mut E;

    /// Casts this pointer to another value, by transferring the internal pointer
    /// to its constructor. Super unsafe.
    unsafe fn cast<T, Q, P>(&self) -> P
    where
        P: UnsafeArrayPtr<T, Q>,
    {
        P::from_ptr(self.as_ptr() as *mut u8)
    }
}

/// Pointer that also knows what its length is. Implementors of this trait need
/// to maintain the invariant that for any instance created via
/// `let array = Self::alloc(len)`, the method `array.get_len` and `array.set_len`
/// agree with each other, any pointer returned by `array.get_ptr(i)` or
/// `array.get_ptr_mut()` where `i < len` points to aligned, allocated memory, and
/// both `array.get_label()` and `array.get_label_mut()` point to allocated, aligned
/// memory as well.
pub unsafe trait LabelledArrayPtr<E, L>: UnsafeArrayPtr<E, L> {
    /// Set the length of this array
    unsafe fn set_len(&mut self, len: usize);

    /// Get the length of this array
    fn get_len(&self) -> usize;
}
