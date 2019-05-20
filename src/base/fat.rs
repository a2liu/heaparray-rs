//! Contains definition of `FatPtrArray`, an array whose pointer is 2 words.
//!
//! This is the typical representation of unsized references in Rust,
//! and is thus also the default implementation of `HeapArray` as imported by `use heaparray::*;`
use super::iter::FatPtrArrayIter;
pub use crate::prelude::*;
use ptr::NonNull;

/// Heap-allocated array, with array size stored with the pointer to the memory.
///
/// ## Examples
///
/// Creating an array:
/// ```rust
/// use heaparray::base::*;
/// let len = 10;
/// let array = FatPtrArray::new(len, |idx| idx + 3);
/// ```
///
/// Indexing works as you would expect:
/// ```rust
/// # use heaparray::base::*;
/// # let mut array = FatPtrArray::new(10, |idx| idx + 3);
/// array[3] = 2;
/// assert!(array[3] == 2);
/// ```
///
/// Notably, you can take ownership of objects back from the container:
///
/// ```rust
/// # use heaparray::base::*;
/// let mut array = FatPtrArray::new(10, |_| Vec::<u8>::new());
/// let replacement_object = Vec::new();
/// let owned_object = array.insert(0, replacement_object);
/// ```
///
/// but you need to give the array a replacement object to fill its slot with.
///
/// Additionally, you can customize what information should be stored alongside the elements in
/// the array using the `FatPtrArray::with_label` function:
///
/// ```rust
/// # use heaparray::base::*;
/// struct MyLabel {
///     pub even: usize,
///     pub odd: usize,
/// }
///
/// let mut array = FatPtrArray::with_label(
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
/// # Invariants
/// This struct follows the same invariants as mentioned in `heaparray::mem_block`,
/// and does not check for pointer validity; you should use this struct in the same
/// way you would use a raw array or slice.
#[repr(C)]
pub struct FatPtrArray<E, L = ()> {
    data: NonNull<MemBlock<E, L>>,
    len: usize,
}

impl<E, L> BaseArrayRef for FatPtrArray<E, L> {}

impl<E, L> Clone for FatPtrArray<E, L>
where
    E: Clone,
    L: Clone,
{
    fn clone(&self) -> Self {
        let new_ptr = unsafe { NonNull::new_unchecked(self.data.as_ref().clone(self.len())) };
        Self {
            data: new_ptr,
            len: self.len,
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

impl<E, L> Drop for FatPtrArray<E, L> {
    fn drop(&mut self) {
        let len = self.len;
        unsafe { self.data.as_mut().dealloc(len) };
    }
}

impl<E, L> Container for FatPtrArray<E, L> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<E, L> CopyMap<usize, E> for FatPtrArray<E, L> {
    fn get(&self, key: usize) -> Option<&E> {
        if key >= self.len() {
            None
        } else {
            Some(unsafe { self.data.as_ref().get(key) })
        }
    }
    fn get_mut(&mut self, key: usize) -> Option<&mut E> {
        if key >= self.len() {
            None
        } else {
            Some(unsafe { self.data.as_mut().get(key) })
        }
    }
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        match self.get_mut(key) {
            Some(slot) => Some(mem::replace(slot, value)),
            None => None,
        }
    }
}

impl<E, L> Index<usize> for FatPtrArray<E, L> {
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        self.get(idx).unwrap()
    }
}

impl<E, L> IndexMut<usize> for FatPtrArray<E, L> {
    fn index_mut(&mut self, idx: usize) -> &mut E {
        self.get_mut(idx).unwrap()
    }
}

impl<E> MakeArray<E> for FatPtrArray<E, ()> {
    fn new<F>(len: usize, mut func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self::with_label((), len, |_, idx| func(idx))
    }
}

impl<E, L> LabelledArray<E, L> for FatPtrArray<E, L> {
    fn with_label<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        Self {
            data: NonNull::new(MemBlock::<E, L>::new_init(label, len, func)).unwrap(),
            len,
        }
    }
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self {
        let new_ptr = NonNull::new_unchecked(MemBlock::new(label, len));
        Self { data: new_ptr, len }
    }
    fn get_label(&self) -> &L {
        unsafe { self.data.as_ref().get_label() }
    }
    unsafe fn get_label_unsafe(&self) -> &mut L {
        self.data.as_ref().get_label()
    }
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E {
        self.data.as_ref().get(idx)
    }
}

impl<E, L> LabelledArrayMut<E, L> for FatPtrArray<E, L> {
    fn get_label_mut(&mut self) -> &mut L {
        unsafe { self.data.as_mut().get_label() }
    }
}

impl<E, L> DefaultLabelledArray<E, L> for FatPtrArray<E, L>
where
    E: Default,
{
    fn with_len(label: L, len: usize) -> Self {
        Self::with_label(label, len, |_, _| E::default())
    }
}

impl<E, L> IntoIterator for FatPtrArray<E, L> {
    type Item = E;
    type IntoIter = FatPtrArrayIter<E, L>;
    fn into_iter(mut self) -> Self::IntoIter {
        let len = self.len();
        let iter = unsafe { self.data.as_mut().iter(len) };
        mem::forget(self);
        FatPtrArrayIter(iter)
    }
}

impl<E, L> SliceArray<E> for FatPtrArray<E, L> {
    fn as_slice(&self) -> &[E] {
        let len = self.len();
        unsafe { self.data.as_ref().as_slice(len) }
    }
    fn as_slice_mut(&mut self) -> &mut [E] {
        let len = self.len();
        unsafe { self.data.as_mut().as_slice(len) }
    }
}

impl<'b, E, L> IntoIterator for &'b FatPtrArray<E, L> {
    type Item = &'b E;
    type IntoIter = core::slice::Iter<'b, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().into_iter()
    }
}

impl<'b, E, L> IntoIterator for &'b mut FatPtrArray<E, L> {
    type Item = &'b mut E;
    type IntoIter = core::slice::IterMut<'b, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice_mut().into_iter()
    }
}

impl<E, L> fmt::Debug for FatPtrArray<E, L>
where
    E: fmt::Debug,
    L: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("FatPtrArray")
            .field("label", &self.get_label())
            .field("len", &self.len())
            .field("elements", &self.as_slice())
            .finish()
    }
}

unsafe impl<E, L> Send for FatPtrArray<E, L>
where
    E: Send,
    L: Send,
{
}

unsafe impl<E, L> Sync for FatPtrArray<E, L>
where
    E: Sync,
    L: Sync,
{
}
