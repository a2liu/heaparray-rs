//! Contains definition of `ThinPtrArray`, an array whose pointer is 1 word.
//!
//! This more similar to how arrays are defined in C or C++, and is less idiomatic
//! in Rust, but may improve performance depending on your use case. Thus, it is
//! not the standard implementation of `HeapArray`, but is still available for use
//! via `use heaparray::thin_array_ptr::*;
pub use crate::prelude::*;
use core::sync::atomic::{AtomicPtr, Ordering};

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
/// the array using the `ThinPtrArray::with_label` function:
///
/// ```rust
/// # use heaparray::thin_array_ptr::*;
/// struct MyLabel {
///     pub even: usize,
///     pub odd: usize,
/// }
///
/// let mut array = ThinPtrArray::with_label(
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
#[repr(C)]
pub struct ThinPtrArray<'a, E, L = ()>
where
    Self: 'a,
{
    data: ManuallyDrop<&'a mut TPArrayBlock<E, L>>,
}

impl<'a, E, L> ThinPtrArray<'a, E, L> {
    fn from_raw(ptr: *mut TPArrayBlock<E, L>) -> Self {
        Self {
            data: ManuallyDrop::new(unsafe { &mut *ptr }),
        }
    }
    fn get_ref<'b>(&self) -> &'b mut TPArrayBlock<E, L> {
        let ret = unsafe { mem::transmute_copy(&self.data) };
        ret
    }
    fn to_ref<'b>(self) -> &'b mut TPArrayBlock<E, L> {
        let ret = self.get_ref();
        mem::forget(self);
        ret
    }
    fn as_atomic(&self) -> AtomicPtr<TPArrayBlock<E, L>> {
        AtomicPtr::new(self.get_ref())
    }
}

impl<'a, E, L> UnsafeArrayRef for ThinPtrArray<'a, E, L> {
    unsafe fn null_ref() -> Self {
        Self {
            data: ManuallyDrop::new(&mut *(TPArrayBlock::null_ptr())),
        }
    }
}

impl<'a, E, L> Index<usize> for ThinPtrArray<'a, E, L> {
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        &self.data[idx]
    }
}

impl<'a, E, L> IndexMut<usize> for ThinPtrArray<'a, E, L> {
    fn index_mut(&mut self, idx: usize) -> &mut E {
        &mut self.data[idx]
    }
}

impl<'a, E, L> Clone for ThinPtrArray<'a, E, L>
where
    E: Clone,
    L: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: ManuallyDrop::new((*self.data).clone()),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        if source.len() != self.len() {
            *self = source.clone();
        } else {
            self.get_label_mut().clone_from(source.get_label());
            for i in 0..source.len() {
                self[i].clone_from(&source[i]);
            }
        }
    }
}

impl<'a, E, L> Drop for ThinPtrArray<'a, E, L> {
    fn drop(&mut self) {
        #[cfg(not(feature = "no-asserts"))]
        assert!(!self.is_null());
        let mut_ref = &mut self.data;
        unsafe { mut_ref.dealloc() };
        mem::forget(mut_ref);
    }
}

impl<'a, E, L> Container<(usize, E)> for ThinPtrArray<'a, E, L> {
    fn add(&mut self, elem: (usize, E)) {
        self[elem.0] = elem.1;
    }
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<'a, E, L> CopyMap<'a, usize, E, (usize, E)> for ThinPtrArray<'a, E, L> {
    fn get(&'a self, key: usize) -> Option<&'a E> {
        if key > self.len() {
            None
        } else {
            Some(&self[key])
        }
    }
    fn get_mut(&'a mut self, key: usize) -> Option<&'a mut E> {
        if key > self.len() {
            None
        } else {
            Some(&mut self[key])
        }
    }
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        if key > self.len() {
            None
        } else {
            Some(mem::replace(&mut self[key], value))
        }
    }
}

impl<'a, E, L> Array<'a, E> for ThinPtrArray<'a, E, L> {}

impl<'a, E> MakeArray<'a, E> for ThinPtrArray<'a, E, ()>
where
    E: 'a,
{
    fn new<F>(len: usize, mut func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self::with_label((), len, |_, idx| func(idx))
    }
}

impl<'a, E, L> LabelledArray<'a, E, L> for ThinPtrArray<'a, E, L> {
    fn with_label<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let block_ptr = TPArrayBlock::<E, L>::new_ptr(label, len, func);
        let new_obj = Self {
            data: ManuallyDrop::new(block_ptr),
        };
        new_obj
    }
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self {
        let new_ptr = TPArrayBlock::<E, L>::new_ptr_unsafe(label, len);
        Self {
            data: ManuallyDrop::new(new_ptr),
        }
    }
    fn get_label(&self) -> &L {
        &self.data.label
    }
    fn get_label_mut(&mut self) -> &mut L {
        &mut self.data.label
    }
    unsafe fn get_label_unsafe(&self) -> &mut L {
        self.data.get_label_unsafe()
    }
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E {
        self.data.unchecked_access(idx)
    }
}

impl<'a, E, L> DefaultLabelledArray<'a, E, L> for ThinPtrArray<'a, E, L>
where
    E: 'a + Default,
{
    fn with_len(label: L, len: usize) -> Self {
        Self {
            data: ManuallyDrop::new(TPArrayBlock::new_ptr_default(label, len)),
        }
    }
}

impl<'a, E, L> BaseArrayRef for ThinPtrArray<'a, E, L> {
    fn is_null(&self) -> bool {
        self.data.is_null()
    }
}

impl<'a, E, L> AtomicArrayRef for ThinPtrArray<'a, E, L> {
    fn compare_and_swap(&self, current: Self, new: Self, order: Ordering) -> Self {
        Self::from_raw(
            self.as_atomic()
                .compare_and_swap(current.to_ref(), new.to_ref(), order),
        )
    }
    fn compare_exchange(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        match self
            .as_atomic()
            .compare_exchange(current.to_ref(), new.to_ref(), success, failure)
        {
            Ok(data) => Ok(Self::from_raw(data)),
            Err(data) => Err(Self::from_raw(data)),
        }
    }
    fn compare_exchange_weak(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        match self.as_atomic().compare_exchange_weak(
            current.to_ref(),
            new.to_ref(),
            success,
            failure,
        ) {
            Ok(data) => Ok(Self::from_raw(data)),
            Err(data) => Err(Self::from_raw(data)),
        }
    }
    fn load(&self, order: Ordering) -> Self {
        Self::from_raw(self.as_atomic().load(order))
    }
    fn store(&self, ptr: Self, order: Ordering) {
        self.as_atomic().store(ptr.to_ref(), order)
    }
    fn swap(&self, ptr: Self, order: Ordering) -> Self {
        Self::from_raw(self.as_atomic().swap(ptr.to_ref(), order))
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     impl<'a> ArrayTest<'a> for ThinPtrArray<'a, Load, LabelLoad> {
//         fn get_self(len: usize) -> Self {
//             Self::with_len(LabelLoad::default(), len)
//         }
//     }
// }
