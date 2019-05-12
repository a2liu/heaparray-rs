//! Contains definition of `ThinPtrArray`, an array whose pointer is 1 word.
//!
//! This more similar to how arrays are defined in C or C++, and is less idiomatic
//! in Rust, but may improve performance depending on your use case. Thus, it is
//! not the standard implementation of `HeapArray`, but is still available for use
//! via `use heaparray::*;`.
pub use crate::prelude::*;

/// Heap-allocated array, with array size stored alongside the memory block
/// itself.
///
/// ## Examples
///
/// Creating an array:
/// ```rust
/// use heaparray::thin_array_ptr::*;
/// let len = 10;
/// let array = ThinPtrArray::new(len, |idx| idx + 3);
/// ```
///
/// Indexing works as you would expect:
/// ```rust
/// # use heaparray::thin_array_ptr::*;
/// # let mut array = ThinPtrArray::new(10, |idx| idx + 3);
/// array[3] = 2;
/// assert!(array[3] == 2);
/// ```
///
/// Notably, you can take ownership of objects back from the container:
///
/// ```rust
/// # use heaparray::thin_array_ptr::*;
/// let mut array = ThinPtrArray::new(10, |_| Vec::<u8>::new());
/// let replacement_object = Vec::new();
/// let owned_object = array.insert(0, replacement_object);
/// ```
///
/// but you need to give the array a replacement object to fill its slot with.
///
/// Additionally, you can customize what information should be stored alongside the elements in
/// the array using the ThinPtrArray::new_labelled function:
///
/// ```rust
/// # use heaparray::thin_array_ptr::*;
/// struct MyLabel {
///     pub even: usize,
///     pub odd: usize,
/// }
///
/// let mut array = ThinPtrArray::new_labelled(
///     MyLabel { even: 0, odd: 0 },
///     100,
///     |label, index| {
///         if index % 2 == 0 {
///             label.even += 1;
///             index
///         } else {
///             label.odd += 1;
///             index
///         }
///     });
/// ```
///
/// # Invariants
/// This struct follows the same invariants as mentioned in `crate::memory_block`,
/// and does not check for pointer validity; you should use this struct in the same
/// way you would use a raw array or slice.
pub struct ThinPtrArray<'a, E, L = ()>
where
    Self: 'a,
{
    data: ManuallyDrop<&'a mut TPArrayBlock<E, L>>,
}

impl<'a, E> ThinPtrArray<'a, E> {
    /// Create a new array, with values initialized using a provided function.
    #[inline]
    pub fn new<F>(len: usize, mut func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self::new_labelled((), len, |_, idx| func(idx))
    }
}

impl<'a, E, L> ThinPtrArray<'a, E, L> {
    /// Create a new array, with values initialized using a provided function, and label
    /// initialized to a provided value.
    #[inline]
    pub fn new_labelled<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        Self {
            data: ManuallyDrop::new(TPArrayBlock::<E, L>::new_ptr(label, len, func)),
        }
    }

    /// Create a new array, without initializing the values in it.
    #[inline]
    pub unsafe fn new_labelled_unsafe(label: L, len: usize) -> Self {
        let new_ptr = TPArrayBlock::<E, L>::new_ptr_unsafe(label, len);
        Self {
            data: ManuallyDrop::new(new_ptr),
        }
    }

    /// Creates a new array from a raw pointer to a memory block.
    #[inline]
    pub unsafe fn from_raw_parts(ptr: &'a mut TPArrayBlock<E, L>) -> Self {
        Self {
            data: ManuallyDrop::new(ptr),
        }
    }

    /// Unsafe access to an element at an index in the array.
    #[inline]
    pub unsafe fn unchecked_access(&'a self, idx: usize) -> &'a mut E {
        self.data.unchecked_access(idx)
    }

    /// Sets the internal pointer to null, without deallocating it, and returns
    /// reference to the associated memory block.
    /// Causes all sorts of undefined behavior, use with caution.
    pub unsafe fn to_null(&mut self) -> &mut TPArrayBlock<E, L> {
        let old = mem::replace(&mut *self, Self::null_ref());
        let ptr = mem::transmute_copy(*old.data);
        // We want to prevent the deallocation from being run, so we
        // need to forget the old version of this struct
        mem::forget(old);
        ptr
    }

    /// Creates a null array. All kinds of UB associated with this, use
    /// with caution.
    pub unsafe fn null_ref() -> Self {
        Self {
            data: ManuallyDrop::new(&mut *(TPArrayBlock::null_ptr())),
        }
    }

    /// Returns whether the internal pointer of this struct is null. Should always
    /// return false unless you use the unsafe API.
    pub fn is_null(&self) -> bool {
        self.data.is_null()
    }
}

impl<'a, E> ThinPtrArray<'a, E>
where
    E: Default,
{
    /// Get a new array, initialized to default values.
    #[inline]
    pub fn new_default(len: usize) -> Self {
        Self::new_default_labelled((), len)
    }
}

impl<'a, E, L> ThinPtrArray<'a, E, L>
where
    E: Default,
{
    /// Get a new array, initialized to default values.
    #[inline]
    pub fn new_default_labelled(label: L, len: usize) -> Self {
        Self {
            data: ManuallyDrop::new(TPArrayBlock::new_ptr_default(label, len)),
        }
    }
}

impl<'a, E, L> LabelledArray<'a, E, L> for ThinPtrArray<'a, E, L> {
    /// Get a reference to the label of the array.
    #[inline]
    fn get_label(&self) -> &L {
        &self.data.label
    }

    /// Get a mutable reference to the label of the array.
    #[inline]
    fn get_label_mut(&mut self) -> &mut L {
        &mut self.data.label
    }
}

impl<'a, E, L> Index<usize> for ThinPtrArray<'a, E, L> {
    type Output = E;
    #[inline]
    fn index(&self, idx: usize) -> &E {
        &self.data[idx]
    }
}

impl<'a, E, L> IndexMut<usize> for ThinPtrArray<'a, E, L> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut E {
        &mut self.data[idx]
    }
}

impl<'a, E, L> Clone for ThinPtrArray<'a, E, L>
where
    L: Clone,
    E: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            data: ManuallyDrop::new((*self.data).clone()),
        }
    }
}

impl<'a, E, L> Drop for ThinPtrArray<'a, E, L> {
    #[inline]
    fn drop(&mut self) {
        let mut_ref = &mut self.data;
        unsafe { mut_ref.dealloc() };
        core::mem::forget(mut_ref);
    }
}

impl<'a, E, L> Container<(usize, E)> for ThinPtrArray<'a, E, L> {
    #[inline]
    fn add(&mut self, elem: (usize, E)) {
        self[elem.0] = elem.1;
    }
    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<'a, E, L> CopyMap<'a, usize, E, (usize, E)> for ThinPtrArray<'a, E, L> {
    #[inline]
    fn get(&'a self, key: usize) -> Option<&'a E> {
        if key > self.len() {
            None
        } else {
            Some(&self[key])
        }
    }
    #[inline]
    fn get_mut(&'a mut self, key: usize) -> Option<&'a mut E> {
        if key > self.len() {
            None
        } else {
            Some(&mut self[key])
        }
    }
    #[inline]
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        if key > self.len() {
            None
        } else {
            Some(std::mem::replace(&mut self[key], value))
        }
    }
}

impl<'a, E, L> Array<'a, E> for ThinPtrArray<'a, E, L> {}
