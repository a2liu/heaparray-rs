//! Contains definition of `AtomicPtrArray`, an array reference whose pointer is
//! 1 word and atomically loaded/stored.
use super::base::BaseArray;
use super::iter::ThinPtrArrayIter;
use super::mem_block::MemBlock;
use super::thin::LenLabel;
use crate::prelude::*;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicPtr, Ordering};

/// Heap-allocated array, with array size stored alongside the memory block
/// itself. Doesn't implement `Sync` because CAS operations on the pointer create
/// a race condition between the time the pointer is read and dereferenced. This
/// can be fixed using reference counting.
///
/// ## Examples
///
/// Creating an array:
/// ```rust
/// use heaparray::base::*;
/// let len = 10;
/// let array = AtomicPtrArray::new(len, |idx| idx + 3);
/// ```
///
/// Indexing works as you would expect:
/// ```rust
/// # use heaparray::base::*;
/// # let mut array = AtomicPtrArray::new(10, |idx| idx + 3);
/// array[3] = 2;
/// assert!(array[3] == 2);
/// ```
///
/// Additionally, you can customize what information should be stored alongside the elements in
/// the array using the `AtomicPtrArray::with_label` function:
///
/// ```rust
/// # use heaparray::base::*;
/// struct MyLabel {
///     pub even: usize,
///     pub odd: usize,
/// }
///
/// let mut array = AtomicPtrArray::with_label(
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
pub struct AtomicPtrArray<E, L = ()> {
    data: Data<E, L>,
    phantom: PhantomData<*mut u8>,
}

type Block<E, L> = MemBlock<E, LenLabel<L>>;
type Data<E, L> = AtomicPtr<Block<E, L>>;

impl<E, L> AtomicPtrArray<E, L> {
    fn as_ref(&self) -> &Block<E, L> {
        unsafe { &*self.data.load(Ordering::Acquire) }
    }
    fn as_mut(&mut self) -> &mut Block<E, L> {
        unsafe { &mut *self.data.load(Ordering::SeqCst) }
    }
    fn to_ref(mut self) -> *mut Block<E, L> {
        let ret = self.as_mut() as *mut Block<E, L>;
        mem::forget(self);
        ret
    }
    fn from_ref(ptr: *mut Block<E, L>) -> Self {
        Self {
            data: AtomicPtr::new(ptr),
            phantom: PhantomData,
        }
    }
    /// Returns a null reference. This function is `unsafe` because none of this
    /// struct's API checks for null references before dereferencing.
    pub unsafe fn null_ref() -> Self {
        Self {
            data: AtomicPtr::new(ptr::null_mut()),
            phantom: PhantomData,
        }
    }
    /// Returns true if the internal pointer in this struct is null.
    pub fn is_null(&self) -> bool {
        self.data.load(Ordering::Acquire).is_null()
    }
}

impl<E, L> Container for AtomicPtrArray<E, L> {
    fn len(&self) -> usize {
        self.as_ref().label.len
    }
}

impl<E, L> CopyMap<usize, E> for AtomicPtrArray<E, L> {
    fn get(&self, key: usize) -> Option<&E> {
        if key >= self.len() {
            None
        } else {
            Some(unsafe { &*self.as_ref().get_ptr(key) })
        }
    }
    fn get_mut(&mut self, key: usize) -> Option<&mut E> {
        if key >= self.len() {
            None
        } else {
            Some(unsafe { &mut *(self.as_mut().get_ptr(key) as *mut E) })
        }
    }
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        match self.get_mut(key) {
            Some(slot) => Some(mem::replace(slot, value)),
            None => None,
        }
    }
}

impl<E, L> Index<usize> for AtomicPtrArray<E, L> {
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        self.get(idx).unwrap()
    }
}

impl<E, L> IndexMut<usize> for AtomicPtrArray<E, L> {
    fn index_mut(&mut self, idx: usize) -> &mut E {
        self.get_mut(idx).unwrap()
    }
}

impl<E, L> LabelledArray<E, L> for AtomicPtrArray<E, L> {
    fn with_label<F>(label: L, len: usize, mut func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let block_ptr = Block::new_init(LenLabel { len, label }, len, |lbl, idx| {
            func(&mut lbl.label, idx)
        })
        .as_ptr();
        let new_obj = Self {
            data: AtomicPtr::new(block_ptr),
            phantom: PhantomData,
        };
        new_obj
    }
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self {
        let new_ptr = Block::new(LenLabel { len, label }, len).as_ptr();
        Self {
            data: AtomicPtr::new(new_ptr),
            phantom: PhantomData,
        }
    }
    fn get_label(&self) -> &L {
        &self.as_ref().label.label
    }
    unsafe fn get_unchecked(&self, idx: usize) -> &E {
        &mut *(self.as_ref().get_ptr(idx) as *mut E)
    }
}

impl<E, L> DefaultLabelledArray<E, L> for AtomicPtrArray<E, L>
where
    E: Default,
{
    fn with_len(label: L, len: usize) -> Self {
        Self::with_label(label, len, |_, _| E::default())
    }
}

impl<E, L> Clone for AtomicPtrArray<E, L>
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

impl<E, L> Drop for AtomicPtrArray<E, L> {
    fn drop(&mut self) {
        let len = self.len();
        unsafe { self.as_mut().dealloc(len) };
    }
}

impl<E, L> LabelledArrayMut<E, L> for AtomicPtrArray<E, L> {
    fn get_label_mut(&mut self) -> &mut L {
        &mut (*self.as_mut().label).label
    }
    unsafe fn get_mut_unchecked(&mut self, idx: usize) -> &mut E {
        &mut *self.as_mut().get_ptr_mut(idx)
    }
}

impl<E> MakeArray<E> for AtomicPtrArray<E, ()> where {
    fn new<F>(len: usize, mut func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self::with_label((), len, |_, idx| func(idx))
    }
}

impl<E, L> BaseArrayRef for AtomicPtrArray<E, L> {}

impl<E, L> IntoIterator for AtomicPtrArray<E, L> {
    type Item = E;
    type IntoIter = ThinPtrArrayIter<E, L>;
    fn into_iter(mut self) -> Self::IntoIter {
        let len = self.len();
        let iter = unsafe { BaseArray::from_ptr(self.as_mut()).into_iter(len) };
        mem::forget(self);
        ThinPtrArrayIter(iter)
    }
}

impl<E, L> SliceArray<E> for AtomicPtrArray<E, L> {
    fn as_slice(&self) -> &[E] {
        let len = self.len();
        unsafe { core::slice::from_raw_parts(&self[0], len) }
    }
    fn as_slice_mut(&mut self) -> &mut [E] {
        let len = self.len();
        unsafe { core::slice::from_raw_parts_mut(&mut self[0], len) }
    }
}

impl<'b, E, L> IntoIterator for &'b AtomicPtrArray<E, L> {
    type Item = &'b E;
    type IntoIter = core::slice::Iter<'b, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().into_iter()
    }
}

impl<'b, E, L> IntoIterator for &'b mut AtomicPtrArray<E, L> {
    type Item = &'b mut E;
    type IntoIter = core::slice::IterMut<'b, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice_mut().into_iter()
    }
}

impl<E, L> fmt::Debug for AtomicPtrArray<E, L>
where
    E: fmt::Debug,
    L: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("AtomicPtrArray")
            .field("label", &self.get_label())
            .field("len", &self.len())
            .field("elements", &self.as_slice())
            .finish()
    }
}

impl<E, L> AtomicArrayRef for AtomicPtrArray<E, L> {
    fn as_ref(&self) -> usize {
        self.as_ref() as *const Block<E, L> as usize
    }
    fn compare_and_swap(
        &self,
        current: usize,
        new: Self,
        order: Ordering,
    ) -> Result<usize, (Self, usize)> {
        let current = current as *mut Block<E, L>;
        let new_ref = new.as_ref() as *const Block<E, L> as *mut Block<E, L>;
        let actual = self.data.compare_and_swap(current, new_ref, order);
        if actual == current {
            mem::forget(new);
            Ok(current as usize)
        } else {
            Err((new, current as usize))
        }
    }
    fn compare_exchange(
        &self,
        current: usize,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<usize, (Self, usize)> {
        let current = current as *mut Block<E, L>;
        let new_ref = new.as_ref() as *const Block<E, L> as *mut Block<E, L>;
        match self
            .data
            .compare_exchange(current, new_ref, success, failure)
        {
            Ok(ptr) => {
                mem::forget(new);
                Ok(ptr as usize)
            }
            Err(ptr) => Err((new, ptr as usize)),
        }
    }
    fn compare_exchange_weak(
        &self,
        current: usize,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<usize, (Self, usize)> {
        let current = current as *mut Block<E, L>;
        let new_ref = new.as_ref() as *const Block<E, L> as *mut Block<E, L>;
        match self
            .data
            .compare_exchange_weak(current, new_ref, success, failure)
        {
            Ok(ptr) => {
                mem::forget(new);
                Ok(ptr as usize)
            }
            Err(ptr) => Err((new, ptr as usize)),
        }
    }
    fn swap(&self, ptr: Self, order: Ordering) -> Self {
        Self::from_ref(self.data.swap(ptr.to_ref(), order))
    }
}

unsafe impl<E, L> Send for AtomicPtrArray<E, L>
where
    E: Send,
    L: Send,
{
}
