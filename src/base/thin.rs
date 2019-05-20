//! Contains definition of `ThinPtrArray`, an array whose pointer is 1 word.
//!
//! This more similar to how arrays are defined in C or C++, and is less idiomatic
//! in Rust, but may improve performance depending on your use case. Thus, it is
//! not the standard implementation of `HeapArray`, but is still available for use
//! via `use heaparray::base::*;
use super::iter::ThinPtrArrayIter;
use crate::prelude::*;
use core::ptr::NonNull;

/// Heap-allocated array, with array size stored alongside the memory block
/// itself.
///
/// ## Examples
///
/// Creating an array:
/// ```rust
/// use heaparray::base::*;
/// let len = 10;
/// let array = ThinPtrArray::new(len, |idx| idx + 3);
/// ```
///
/// Indexing works as you would expect:
/// ```rust
/// # use heaparray::base::*;
/// # let mut array = ThinPtrArray::new(10, |idx| idx + 3);
/// array[3] = 2;
/// assert!(array[3] == 2);
/// ```
///
/// Notably, you can take ownership of objects back from the container:
///
/// ```rust
/// # use heaparray::base::*;
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
/// # use heaparray::base::*;
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
/// This struct follows the same invariants as mentioned in `heaparray::mem_block`,
/// and does not check for pointer validity; you should use this struct in the same
/// way you would use a raw array or slice.
#[repr(transparent)]
pub struct ThinPtrArray<E, L = ()> {
    data: Data<E, L>,
}

type Block<E, L> = MemBlock<E, LenLabel<L>>;
type Data<E, L> = NonNull<Block<E, L>>;

#[derive(Clone)]
pub(crate) struct LenLabel<L> {
    len: usize,
    label: L,
}

impl<E, L> Clone for ThinPtrArray<E, L>
where
    E: Clone,
    L: Clone,
{
    fn clone(&self) -> Self {
        let new_ptr = unsafe { NonNull::new_unchecked(self.data.as_ref().clone(self.len())) };
        Self { data: new_ptr }
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

impl<E, L> Drop for ThinPtrArray<E, L> {
    fn drop(&mut self) {
        let len = self.len();
        unsafe { self.data.as_mut().dealloc(len) };
    }
}

impl<E, L> Container for ThinPtrArray<E, L> {
    fn len(&self) -> usize {
        unsafe { self.data.as_ref().get_label().len }
    }
}

impl<E, L> CopyMap<usize, E> for ThinPtrArray<E, L> {
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

impl<E, L> Index<usize> for ThinPtrArray<E, L> {
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        self.get(idx).unwrap()
    }
}

impl<E, L> IndexMut<usize> for ThinPtrArray<E, L> {
    fn index_mut(&mut self, idx: usize) -> &mut E {
        self.get_mut(idx).unwrap()
    }
}

impl<E> MakeArray<E> for ThinPtrArray<E, ()> where {
    fn new<F>(len: usize, mut func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self::with_label((), len, |_, idx| func(idx))
    }
}

impl<E, L> LabelledArray<E, L> for ThinPtrArray<E, L> {
    fn with_label<F>(label: L, len: usize, mut func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let block_ptr = Block::new_init(LenLabel { len, label }, len, |lbl, idx| {
            func(&mut lbl.label, idx)
        });
        let new_obj = Self {
            data: NonNull::new(block_ptr).unwrap(),
        };
        new_obj
    }
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self {
        let new_ptr = Block::new(LenLabel { len, label }, len);
        Self {
            data: NonNull::new_unchecked(new_ptr),
        }
    }
    fn get_label(&self) -> &L {
        unsafe { self.get_label_unsafe() }
    }
    unsafe fn get_label_unsafe(&self) -> &mut L {
        &mut self.data.as_ref().get_label().label
    }
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E {
        self.data.as_ref().get(idx)
    }
}

impl<E, L> LabelledArrayMut<E, L> for ThinPtrArray<E, L> {
    fn get_label_mut(&mut self) -> &mut L {
        unsafe { self.get_label_unsafe() }
    }
}

impl<E, L> DefaultLabelledArray<E, L> for ThinPtrArray<E, L>
where
    E: Default,
{
    fn with_len(label: L, len: usize) -> Self {
        Self::with_label(label, len, |_, _| E::default())
    }
}

impl<E, L> BaseArrayRef for ThinPtrArray<E, L> {}

impl<E, L> IntoIterator for ThinPtrArray<E, L> {
    type Item = E;
    type IntoIter = ThinPtrArrayIter<E, L>;
    fn into_iter(mut self) -> Self::IntoIter {
        let len = self.len();
        let iter = unsafe { self.data.as_mut().iter(len) };
        mem::forget(self);
        ThinPtrArrayIter(iter)
    }
}

impl<E, L> SliceArray<E> for ThinPtrArray<E, L> {
    fn as_slice(&self) -> &[E] {
        let len = self.len();
        unsafe { self.data.as_ref().as_slice(len) }
    }
    fn as_slice_mut(&mut self) -> &mut [E] {
        let len = self.len();
        unsafe { self.data.as_mut().as_slice(len) }
    }
}

impl<'b, E, L> IntoIterator for &'b ThinPtrArray<E, L> {
    type Item = &'b E;
    type IntoIter = core::slice::Iter<'b, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().into_iter()
    }
}

impl<'b, E, L> IntoIterator for &'b mut ThinPtrArray<E, L> {
    type Item = &'b mut E;
    type IntoIter = core::slice::IterMut<'b, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice_mut().into_iter()
    }
}

impl<E, L> fmt::Debug for ThinPtrArray<E, L>
where
    E: fmt::Debug,
    L: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("ThinPtrArray")
            .field("label", &self.get_label())
            .field("len", &self.len())
            .field("elements", &self.as_slice())
            .finish()
    }
}

unsafe impl<E, L> Send for ThinPtrArray<E, L>
where
    E: Send,
    L: Send,
{
}

unsafe impl<E, L> Sync for ThinPtrArray<E, L>
where
    E: Sync,
    L: Sync,
{
}
