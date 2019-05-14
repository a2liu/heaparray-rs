pub use core::sync::atomic::Ordering;

pub trait MakeArray<'a, E>: containers::Array<'a, E>
where
    E: 'a,
{
    /// Create a new array, with values initialized using a provided function.
    fn new<F>(len: usize, func: F) -> Self
    where
        F: FnMut(usize) -> E;
}

/// Array with an optional label struct stored next to the data.
pub trait LabelledArray<'a, E, L>: containers::Array<'a, E>
// MakeArray<'a,E>
where
    E: 'a,
{
    /// Create a new array, with values initialized using a provided function, and label
    /// initialized to a provided value.
    fn with_label<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E;
    /// Create a new array, without initializing the values in it.
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self;

    /// Get immutable access to the label.
    fn get_label(&self) -> &L;

    /// Get mutable reference to the label.
    fn get_label_mut(&mut self) -> &mut L;

    /// Get a mutable reference to the label. Implementations of this
    /// method shouldn't do any safety checks.
    unsafe fn get_label_unsafe(&self) -> &mut L;

    /// Get a mutable reference to the element at a specified index.
    /// Implementations of this method shouldn't do any safety checks.
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E;
}

/// Trait for a labelled array with a default value.
pub trait DefaultLabelledArray<'a, E, L>: LabelledArray<'a, E, L>
where
    E: 'a + Default,
{
    /// Create a new array, initialized to default values.
    fn with_len(label: L, len: usize) -> Self;
}

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
    /// Sets the internal pointer to null, without deallocating it, and returns
    /// reference to the associated memory block.
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
/// let array_ref = RcArray::new(10, |_| 0);
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
    fn clone(ptr: &Self) -> Self {
        ptr.clone()
    }
    fn to_null(&mut self);
    fn null_ref() -> Self;
}

/// Atomically modified array reference. Guarrantees that all operations on the
/// array reference are atomic (i.e. all changes to the internal array pointer).
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
