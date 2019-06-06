//! Contains definition of `ThinPtrArray`, an array whose pointer is 1 word.
//!
//! This more similar to how arrays are defined in C or C++, and is less idiomatic
//! in Rust, but may improve performance depending on your use case. Thus, it is
//! not the standard implementation of `HeapArray`, but is still available for use
//! via `use heaparray::base::*;
use super::{Array, ArrayIter};
use crate::prelude::*;

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
#[repr(transparent)]
pub struct ThinPtrArray<E, L = ()> {
    data: Data<E, L>,
}

type Data<E, L> = Array<E, LenLabel<L>>;

#[derive(Clone)]
pub(crate) struct LenLabel<L> {
    pub(crate) len: usize,
    pub(crate) label: L,
}

impl<E, L> Clone for ThinPtrArray<E, L>
where
    E: Clone,
    L: Clone,
{
    fn clone(&self) -> Self {
        Self::with_label(self.get_label().clone(), self.len(), |_, i| self[i].clone())
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
        unsafe { self.data.drop(len) };
    }
}

impl<E, L> Container for ThinPtrArray<E, L> {
    fn len(&self) -> usize {
        self.data.get_label().len
    }
}

impl<E, L> CopyMap<usize, E> for ThinPtrArray<E, L> {
    fn get(&self, key: usize) -> Option<&E> {
        if key >= self.len() {
            None
        } else {
            Some(unsafe { self.data.get(key) })
        }
    }
    fn get_mut(&mut self, key: usize) -> Option<&mut E> {
        if key >= self.len() {
            None
        } else {
            Some(unsafe { self.data.get_mut(key) })
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
        Self {
            data: Array::new(LenLabel { len, label }, len, |lbl, idx| {
                func(&mut lbl.label, idx)
            }),
        }
    }
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self {
        Self {
            data: Array::new_lazy(LenLabel { len, label }, len),
        }
    }
    fn get_label(&self) -> &L {
        unsafe { self.get_label_unsafe() }
    }
    unsafe fn get_label_unsafe(&self) -> &mut L {
        &mut *(&self.data.get_label().label as *const L as *mut L)
    }
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E {
        &mut *(self.data.get_ptr(idx) as *mut E)
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
    type IntoIter = ArrayIter<E, LenLabel<L>>;
    fn into_iter(mut self) -> Self::IntoIter {
        let len = self.len();
        let iter = unsafe {
            mem::transmute_copy::<Array<E, LenLabel<L>>, Array<E, LenLabel<L>>>(&self.data)
                .into_iter(len)
        };
        mem::forget(self);
        iter
    }
}

impl<E, L> SliceArray<E> for ThinPtrArray<E, L> {
    fn as_slice(&self) -> &[E] {
        let len = self.len();
        unsafe { self.data.as_slice(len) }
    }
    fn as_slice_mut(&mut self) -> &mut [E] {
        let len = self.len();
        unsafe { self.data.as_slice_mut(len) }
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
