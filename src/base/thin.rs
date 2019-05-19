//! Contains definition of `ThinPtrArray`, an array whose pointer is 1 word.
//!
//! This more similar to how arrays are defined in C or C++, and is less idiomatic
//! in Rust, but may improve performance depending on your use case. Thus, it is
//! not the standard implementation of `HeapArray`, but is still available for use
//! via `use heaparray::base::*;
use super::iter::ThinPtrArrayIter;
use crate::prelude::*;
// use core::sync::atomic::{AtomicPtr, Ordering};

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
pub struct ThinPtrArray<'a, E, L = ()>
where
    Self: 'a,
{
    data: Data<'a, E, L>,
}

type Block<E, L> = MemBlock<E, LenLabel<L>>;
type Data<'a, E, L> = ManuallyDrop<&'a mut Block<E, L>>;

#[derive(Clone)]
pub(crate) struct LenLabel<L> {
    len: usize,
    label: L,
}

// impl<'a, E, L> ThinPtrArray<'a, E, L>
// where
//     E: 'a,
//     L: 'a,
// {
//     fn from_raw(ptr: *mut Block<E, L>) -> Self {
//         Self {
//             data: ManuallyDrop::new(unsafe { &mut *ptr }),
//         }
//     }
//     fn get_ref<'b>(&self) -> &'b mut Block<E, L> {
//         let ret = unsafe { mem::transmute_copy(&self.data) };
//         ret
//     }
//     fn into_ref<'b>(self) -> &'b mut Block<E, L> {
//         let ret = self.get_ref();
//         mem::forget(self);
//         ret
//     }
//     fn as_atomic(&self) -> AtomicPtr<Block<E, L>> {
//         AtomicPtr::new(self.get_ref())
//     }
//     fn usize_to_ref<'b>(ptr: usize) -> &'b mut Block<E, L> {
//         Self::from_raw(ptr as *const Block<E, L> as *mut Block<E, L>).into_ref()
//     }
// }

impl<'a, E, L> Clone for ThinPtrArray<'a, E, L>
where
    E: Clone,
    L: Clone,
{
    fn clone(&self) -> Self {
        let new_ptr = unsafe { (*self.data).clone(self.len()) };
        Self {
            data: ManuallyDrop::new(new_ptr),
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
        let len = self.len();
        let mut_ref = &mut self.data;
        unsafe { mut_ref.dealloc(len) };
        mem::forget(mut_ref);
    }
}

impl<'a, E, L> Container for ThinPtrArray<'a, E, L> {
    fn len(&self) -> usize {
        self.data.label.len
    }
}

impl<'a, E, L> CopyMap<usize, E> for ThinPtrArray<'a, E, L> {
    fn get(&self, key: usize) -> Option<&E> {
        if key > self.len() {
            None
        } else {
            Some(unsafe { self.data.get(key) })
        }
    }
    fn get_mut(&mut self, key: usize) -> Option<&mut E> {
        if key > self.len() {
            None
        } else {
            Some(unsafe { self.data.get(key) })
        }
    }
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        match self.get_mut(key) {
            Some(slot) => Some(mem::replace(slot, value)),
            None => None,
        }
    }
}

impl<'a, E, L> Index<usize> for ThinPtrArray<'a, E, L> {
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        self.get(idx).unwrap()
    }
}

impl<'a, E, L> IndexMut<usize> for ThinPtrArray<'a, E, L> {
    fn index_mut(&mut self, idx: usize) -> &mut E {
        self.get_mut(idx).unwrap()
    }
}

impl<'a, E> MakeArray<E> for ThinPtrArray<'a, E, ()> where {
    fn new<F>(len: usize, mut func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self::with_label((), len, |_, idx| func(idx))
    }
}

impl<'a, E, L> LabelledArray<E, L> for ThinPtrArray<'a, E, L> {
    fn with_label<F>(label: L, len: usize, mut func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let block_ptr = Block::new_init(LenLabel { len, label }, len, |lbl, idx| {
            func(&mut lbl.label, idx)
        });
        let new_obj = Self {
            data: ManuallyDrop::new(block_ptr),
        };
        new_obj
    }
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self {
        let new_ptr = Block::new(LenLabel { len, label }, len);
        Self {
            data: ManuallyDrop::new(new_ptr),
        }
    }
    fn get_label(&self) -> &L {
        &self.data.label.label
    }
    unsafe fn get_label_unsafe(&self) -> &mut L {
        &mut self.data.get_label().label
    }
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E {
        self.data.get(idx)
    }
}

impl<'a, E, L> LabelledArrayMut<E, L> for ThinPtrArray<'a, E, L> {
    fn get_label_mut(&mut self) -> &mut L {
        &mut self.data.label.label
    }
}

impl<'a, E, L> DefaultLabelledArray<E, L> for ThinPtrArray<'a, E, L>
where
    E: Default,
{
    fn with_len(label: L, len: usize) -> Self {
        Self::with_label(label, len, |_, _| E::default())
    }
}

impl<'a, E, L> BaseArrayRef for ThinPtrArray<'a, E, L> {}

impl<'a, E, L> IntoIterator for ThinPtrArray<'a, E, L> {
    type Item = E;
    type IntoIter = ThinPtrArrayIter<'a, E, L>;
    fn into_iter(self) -> Self::IntoIter {
        let iter = unsafe { mem::transmute_copy(&self.data.iter(self.len())) };
        mem::forget(self);
        iter
    }
}

impl<'a, E, L> SliceArray<E> for ThinPtrArray<'a, E, L> {
    fn as_slice(&self) -> &[E] {
        unsafe { self.data.as_slice(self.len()) }
    }
    fn as_slice_mut(&mut self) -> &mut [E] {
        unsafe { self.data.as_slice(self.len()) }
    }
}

impl<'a, 'b, E, L> IntoIterator for &'b ThinPtrArray<'a, E, L> {
    type Item = &'b E;
    type IntoIter = core::slice::Iter<'b, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().into_iter()
    }
}

impl<'a, 'b, E, L> IntoIterator for &'b mut ThinPtrArray<'a, E, L> {
    type Item = &'b mut E;
    type IntoIter = core::slice::IterMut<'b, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice_mut().into_iter()
    }
}

impl<'a, E, L> fmt::Debug for ThinPtrArray<'a, E, L>
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

//impl<'a, E, L> AtomicArrayRef for ThinPtrArray<'a, E, L> {
//    fn as_ref(&self) -> usize {
//        self.get_ref() as *const Block<E, L> as usize
//    }
//    fn compare_and_swap(&self, current: usize, new: Self, order: Ordering) -> Self {
//        Self::from_raw(self.as_atomic().compare_and_swap(
//            Self::usize_to_ref(current),
//            new.into_ref(),
//            order,
//        ))
//    }
//    fn compare_exchange(
//        &self,
//        current: usize,
//        new: Self,
//        success: Ordering,
//        failure: Ordering,
//    ) -> Result<Self, Self> {
//        match self.as_atomic().compare_exchange(
//            Self::usize_to_ref(current),
//            new.into_ref(),
//            success,
//            failure,
//        ) {
//            Ok(data) => Ok(Self::from_raw(data)),
//            Err(data) => Err(Self::from_raw(data)),
//        }
//    }
//    fn compare_exchange_weak(
//        &self,
//        current: usize,
//        new: Self,
//        success: Ordering,
//        failure: Ordering,
//    ) -> Result<Self, Self> {
//        match self.as_atomic().compare_exchange_weak(
//            Self::usize_to_ref(current),
//            new.into_ref(),
//            success,
//            failure,
//        ) {
//            Ok(data) => Ok(Self::from_raw(data)),
//            Err(data) => Err(Self::from_raw(data)),
//        }
//    }
//    fn load(&self, order: Ordering) -> Self {
//        Self::from_raw(self.as_atomic().load(order))
//    }
//    fn store(&self, ptr: Self, order: Ordering) {
//        self.as_atomic().store(ptr.into_ref(), order)
//    }
//    fn swap(&self, ptr: Self, order: Ordering) -> Self {
//        Self::from_raw(self.as_atomic().swap(ptr.into_ref(), order))
//    }
//}
