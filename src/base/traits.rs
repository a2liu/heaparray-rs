//! Defines `BaseArrayPtr`, the interface `BaseArray` uses when defining methods.

/// Trait representing an unsafe reference to an array.
///
/// Should be the same size as the underlying pointer.
///
/// # Implementing this Type
/// Let `a` be an instance of `A`, which is a concrete implementation of
/// `BaseArrayPtr<E, L>`. The following must hold on `a`:
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
/// - The difference in addresses between `a.elem_ptr(idx + 1)` and `a.elem_ptr(idx)`
///   is exactly `core::mem::size_of::<E>()`; i.e. the objects that `a.elem_ptr`
///   points to are in an array
/// - `A::alloc(len).lbl_ptr()` returns a pointer to allocated memory
/// - `a._init()` is safe to call on the result of `A::alloc(len)`
/// - `a._drop()` is safe to call on any result of `A::alloc(len)` for which
///   `_init()` has been called exactly once
/// - `A::from_ptr(A::alloc(len).as_ptr())` is safe; i.e. `A::from_ptr` and
///   `A::as_ptr` must agree on the raw pointer representation of `A`
///
/// # Use of API by `BaseArray`
/// Let `A` be a concrete implementation of `BaseArrayPtr`. At initialization via
/// `BaseArray::new`, `BaseArray::new_lazy`, or `BaseArray::alloc`, `BaseArray`
/// does the following:
///
/// 1. Call `A::alloc(len)`
/// 2. Call `a._init()` on the newly created instance
/// 3. Optionally call constructor methods (depending on which method)
///    1. Label is initialized first by calling `a.lbl_ptr()` and writing to it
///    2. Elements are initialized by calling `a.elem_ptr(idx)` and writing to it
///       for each `idx < len`
///
/// At destruction via `BaseArray::drop` or `BaseArray::drop_lazy`, `BaseArray`
/// does the following:
///
/// 1. Optionally call destructors (depending on which method)
///    1. Label is destructed first, in place
///    2. Elements are destructed in ascending order
/// 2. Call `a._drop()`
/// 3. Call `a.dealloc()`
///
/// On accessing an element, `BaseArray` calls `elem_ptr`, and on accessing the
/// label, `BaseArray` calls `lbl_ptr`
pub unsafe trait BaseArrayPtr<E, L>: Sized {
    /// Allocate the memory necessary for a new instance of `len` elements, without
    /// initializing it
    unsafe fn alloc(len: usize) -> Self;

    /// Deallocate the memory for an instance of `len` elements, without running
    /// destructors
    unsafe fn dealloc(&mut self, len: usize);

    /// Creates a new reference of this type without doing any checks.
    ///
    /// # Safety
    /// This function is not *ever* guarranteed to be safe.
    unsafe fn from_ptr(ptr: *mut u8) -> Self;

    /// Returns the value of the internal raw pointer in this array pointer
    ///
    /// Dereferencing this raw pointer isn't safe unless the original array pointer
    /// pointed to valid memory.
    fn as_ptr(&self) -> *mut u8;

    /// Returns whether or not this pointer is null
    fn is_null(&self) -> bool;

    /// Returns a raw pointer to the label associated with this array
    ///
    /// Dereferencing this raw pointer isn't safe unless the original array pointer
    /// pointed to valid memory.
    fn lbl_ptr(&self) -> *mut L;

    /// Returns a raw pointer to the element at `idx`
    ///
    /// Dereferencing this pointer is only safe if there actually is a properly
    /// initialized element at that location.
    fn elem_ptr(&self, idx: usize) -> *mut E;

    /// Initializes fields at construction
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

    /// Runs destructors right before deallocating the buffer
    ///
    /// In `BaseArray` this will run *after* all other destructors; this means that
    /// the memory this method has access to is almost entirely uninitialized.
    ///
    /// # Safety
    /// Almost all accesses are unsafe. Tread with caution.
    unsafe fn _drop(&mut self) {}

    /// Casts this pointer to another value, by transferring the internal pointer
    /// to its constructor. Super unsafe.
    unsafe fn cast<T, Q, P>(&self) -> P
    where
        P: BaseArrayPtr<T, Q>,
    {
        P::from_ptr(self.as_ptr() as *mut u8)
    }
}
