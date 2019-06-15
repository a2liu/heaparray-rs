//! Defines `BaseArrayPtr`, the interface `BaseArray` uses when defining methods.

/// Trait representing an unsafe reference to an array.
///
/// Should be the same size as the underlying pointer.
///
/// # Implementation
/// - Destructors for the label and elements are run by the callee; that means that
///   any implementation of `Drop` that you write no longer has access to the data
///   that the array contained. To run code before the buffer is deallocated,
///   use the `_drop()` function.
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
    unsafe fn _init(&mut self) {}

    /// Runs destructors right before deallocating the buffer.
    ///
    /// In `BaseArray` this will run *after* all other destructors; this means that
    /// the memory this method has access to is almost entirely uninitialized.
    ///
    /// # Safety
    /// Almost all accesses are unsafe. Tread with caution.
    unsafe fn _drop(&mut self) {}

    /// Returns a raw pointer to the element at `idx`
    ///
    /// Dereferencing this pointer is only safe if there actually is a properly
    /// initialized element at that location
    fn elem_ptr(&self, idx: usize) -> *mut E;

    /// Casts this pointer to another value, by transferring the internal pointer
    /// to its constructor. Super unsafe
    unsafe fn cast<T, Q, P>(&self) -> P
    where
        P: BaseArrayPtr<T, Q>,
    {
        P::from_ptr(self.as_ptr() as *mut u8)
    }
}
