use super::iter::HeapArrayIterOwned;
use crate::base::FatPtrArray;
use crate::prelude::*;

/// Heap-allocated array, with array size stored with the pointer to the memory.
///
/// ## Examples
///
/// Creating an array:
/// ```rust
/// use heaparray::*;
/// let len = 10;
/// let array = HeapArray::new(len, |idx| idx + 3);
/// ```
///
/// Indexing works as you would expect:
/// ```rust
/// # use heaparray::*;
/// # let mut array = HeapArray::new(10, |idx| idx + 3);
/// array[3] = 2;
/// assert!(array[3] == 2);
/// ```
///
/// Notably, you can take ownership of objects back from the container:
///
/// ```rust
/// # use heaparray::*;
/// let mut array = HeapArray::new(10, |_| Vec::<u8>::new());
/// let replacement_object = Vec::new();
/// let owned_object = array.insert(0, replacement_object);
/// ```
///
/// but you need to give the array a replacement object to fill its slot with.
///
/// Additionally, you can customize what information should be stored alongside the elements in
/// the array using the HeapArray::with_label function:
///
/// ```rust
/// # use heaparray::*;
/// struct MyLabel {
///     pub even: usize,
///     pub odd: usize,
/// }
///
/// let mut array = HeapArray::with_label(
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
pub struct HeapArray<'a, E, L = ()>(Inner<'a, E, L>);
type Inner<'a, E, L> = FatPtrArray<'a, E, L>;

impl<'a, E, L> BaseArrayRef for HeapArray<'a, E, L> {
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
impl<'a, E, L> Clone for HeapArray<'a, E, L>
where
    E: Clone,
    L: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<'a, E, L> UnsafeArrayRef for HeapArray<'a, E, L> {
    unsafe fn null_ref() -> Self {
        Self(Inner::null_ref())
    }
}
impl<'a, E, L> Index<usize> for HeapArray<'a, E, L> {
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        self.0.index(idx)
    }
}
impl<'a, E, L> IndexMut<usize> for HeapArray<'a, E, L> {
    fn index_mut(&mut self, idx: usize) -> &mut E {
        self.0.index_mut(idx)
    }
}

impl<'a, E, L> Container for HeapArray<'a, E, L> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, E, L> CopyMap<usize, E> for HeapArray<'a, E, L> {
    fn get(&self, key: usize) -> Option<&E> {
        self.0.get(key)
    }
    fn get_mut(&mut self, key: usize) -> Option<&mut E> {
        self.0.get_mut(key)
    }
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        self.0.insert(key, value)
    }
}

impl<'a, E, L> LabelledArray<E, L> for HeapArray<'a, E, L> {
    fn with_label<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        Self(Inner::with_label(label, len, func))
    }
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self {
        Self(Inner::with_label_unsafe(label, len))
    }
    fn get_label(&self) -> &L {
        self.0.get_label()
    }
    unsafe fn get_label_unsafe(&self) -> &mut L {
        self.0.get_label_unsafe()
    }
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E {
        self.0.get_unsafe(idx)
    }
}

impl<'a, E, L> LabelledArrayMut<E, L> for HeapArray<'a, E, L> {
    fn get_label_mut(&mut self) -> &mut L {
        self.0.get_label_mut()
    }
}

impl<'a, E> MakeArray<E> for HeapArray<'a, E, ()>
where
    E: 'a,
{
    fn new<F>(len: usize, func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self(Inner::new(len, func))
    }
}

impl<'a, E, L> DefaultLabelledArray<E, L> for HeapArray<'a, E, L>
where
    E: Default,
{
    fn with_len(label: L, len: usize) -> Self {
        Self(Inner::with_len(label, len))
    }
}

impl<'a, E, L> IntoIterator for HeapArray<'a, E, L> {
    type Item = E;
    type IntoIter = HeapArrayIterOwned<'a, E, L>;
    fn into_iter(self) -> Self::IntoIter {
        HeapArrayIterOwned(self.0.into_iter())
    }
}

unsafe impl<'a, E, L> Send for HeapArray<'a, E, L> where Inner<'a, E, L>: Send {}
unsafe impl<'a, E, L> Sync for HeapArray<'a, E, L> where Inner<'a, E, L>: Sync {}
