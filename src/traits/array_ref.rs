// #[cfg(test)]
// use super::tests::*;
use core::sync::atomic::Ordering;

/// A basic reference to a heap-allocated array. Should be paired with exactly
/// one of either `heaparray::UnsafeArrayRef` or `heaparray::ArrayRef`.
pub trait BaseArrayRef {
    /// Returns whether the array pointer that this contains is null.
    fn is_null(&self) -> bool;
}

/// A reference to a heap-allocated array whose safe API guarrantees it to
/// always be non-null.
pub trait UnsafeArrayRef<'a, B>: BaseArrayRef
where
    B: ?Sized,
{
    /// Creates a new array from a raw pointer to a memory block.
    unsafe fn from_raw_parts(ptr: &'a mut B) -> Self;
    /// Sets the internal pointer to null, without deallocating it, and
    /// returns a reference to the associated memory block.
    /// Causes all sorts of undefined behavior, use with caution.
    unsafe fn to_null<'b>(&mut self) -> &'b mut B;
    /// Creates a null array. All kinds of UB associated with this, use
    /// with caution.
    unsafe fn null_ref() -> Self;
}

/// A reference to an array, whose clone points to the same data.
///
/// Allows for idiomatic cloning of array references:
///
/// ```rust
/// # use heaparray::naive_rc::*;
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
pub trait ArrayRef: BaseArrayRef + Clone {
    // Should this be stricter? It really shouldn't be implemented by other
    // crates, but the type system could definitely make that somewhat of a
    // guarrantee without making the usage of this trait any less ergonomic.
    /// Clones the array reference. Internally just calls its `.clone()`
    /// method.
    fn clone(ptr: &Self) -> Self {
        ptr.clone()
    }
    /// Set this pointer to null.
    fn to_null(&mut self);
    /// Get a null reference of this pointer type.
    fn null_ref() -> Self;
}

// #[trait_tests]
// pub trait RefTest<'a>: ArrayRef + ArrayTest<'a> {
//     fn clone_test() {
//         let first_ref = Self::get_self(LENGTH);
//         let second_ref = ArrayRef::clone(&first_ref);
//         assert!(first_ref.len() == second_ref.len());
//         for i in 0..second_ref.len() {
//             let r1 = &first_ref[i] as *const Load;
//             let r2 = &second_ref[i] as *const Load;
//             assert!(r1 == r2);
//         }
//     }
//     fn ref_counting_test() {
//         let mut ref_vec = Vec::with_capacity(2 * LENGTH);
//         let t_0 = before_alloc();
//         let balloc = t_0.bytes_alloc;
//         let first_ref = Self::get_self(LENGTH);
//         ref_vec.push(first_ref);
//         for _ in 0..LENGTH {
//             let new_ref = ArrayRef::clone(ref_vec.last().unwrap());
//             assert!(before_alloc().bytes_alloc == balloc);
//             ref_vec.push(new_ref);
//         }
//         let final_ref = ArrayRef::clone(&ref_vec[0]);
//         mem::drop(ref_vec);
//         assert!(before_alloc().bytes_alloc == balloc);
//         after_alloc(final_ref, t_0);
//     }
// }

/// Atomically modified array reference. Guarrantees that all operations on the
/// array reference are atomic (i.e. all changes to the internal array pointer).
///
/// For mor details on the expected behavior of these methods, see the
/// documentation for `std::sync::atomic::AtomicPtr`
pub trait AtomicArrayRef: BaseArrayRef + Sized {
    fn compare_and_swap(&self, current: Self, new: Self, order: Ordering) -> Self;
    fn compare_exchange(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self>;
    fn compare_exchange_weak(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self>;
    fn load(&self, order: Ordering) -> Self;
    fn store(&self, ptr: Self, order: Ordering);
    fn swap(&self, ptr: Self, order: Ordering) -> Self;
}
