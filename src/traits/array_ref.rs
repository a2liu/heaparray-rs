use core::sync::atomic::Ordering;

/// A reference to a heap-allocated array.
///
/// Should be paired with exactly one of either `heaparray::UnsafeArrayRef`
/// or `heaparray::ArrayRef`.
pub trait BaseArrayRef {}

/// A reference to an array, whose clone points to the same data.
///
/// Allows for idiomatic cloning of array references:
///
/// ```rust
/// use heaparray::naive_rc::*;
/// let array_ref = FpRcArray::new(10, |_| 0);
/// let another_ref = ArrayRef::clone(&array_ref);
///
/// assert!(array_ref.len() == another_ref.len());
/// for i in 0..another_ref.len() {
///     let r1 = &array_ref[i] as *const i32;
///     let r2 = &another_ref[i] as *const i32;
///     assert!(r1 == r2);
/// }
/// ```
pub trait ArrayRef: Clone {
    /// Clones the array reference. Internally just calls its `.clone()`
    /// method.
    fn clone(ptr: &Self) -> Self {
        ptr.clone()
    }
}

/// Atomically modified array reference.
///
/// Guarrantees that all operations on the
/// array reference are atomic (i.e. all changes to the internal array pointer).
/// Additionally, guarrantees that all reads to a reference of this pointer use
/// atomic loads.
///
/// For more details on the expected behavior of these methods, see the
/// documentation for `core::sync::atomic::AtomicPtr`.
pub trait AtomicArrayRef: Sized {
    fn as_ref(&self) -> usize;
    /// Returns the previous value, and also the struct you passed in if the value
    /// wasn't updated
    fn compare_and_swap(
        &self,
        current: usize,
        new: Self,
        order: Ordering,
    ) -> Result<usize, (Self, usize)>;
    /// Returns the previous value, and also the struct you passed in if the value
    /// wasn't updated
    fn compare_exchange(
        &self,
        current: usize,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<usize, (Self, usize)>;
    /// Swaps in the passed-in reference if the internal reference matches `current`.
    /// Returns the previous value, and also the struct you passed in if the value
    /// wasn't updated
    fn compare_exchange_weak(
        &self,
        current: usize,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<usize, (Self, usize)>;
    /// Swaps in the specified array reference and returns the previous value
    fn swap(&self, ptr: Self, order: Ordering) -> Self;
}
