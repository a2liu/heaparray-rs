use crate::base::BaseArrayPtr;

/// Array pointer that also knows what its length is.
///
/// Implementors of this trait need
/// to maintain the invariant that for any instance created via
/// `let array = Self::alloc(len)`, the method `array.get_len` and `array.set_len`
/// agree with each other, any pointer returned by `array.get_ptr(i)` or
/// `array.get_ptr_mut()` where `i < len` points to aligned, allocated memory, and
/// both `array.get_label()` and `array.get_label_mut()` point to allocated, aligned
/// memory as well.
pub unsafe trait SafeArrayPtr<E, L>: BaseArrayPtr<E, L> {
    /// Set the length of this array
    unsafe fn set_len(&mut self, len: usize);

    /// Get the length of this array
    fn get_len(&self) -> usize;
}
