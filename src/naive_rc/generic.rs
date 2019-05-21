//! Contains definition for `RcArray`, which is an implementation-agnositc,
//! reference-counted array.

use super::ref_counters::*;
pub use crate::prelude::*;
use core::marker::PhantomData;
use core::sync::atomic::Ordering;

/// `RcArray` is a generic, implementation-agnositc array. It contains
/// logic for enforcing type safety.
///
/// The type parameters are, in order:
///
/// ```text
/// A: A struct that acts as a reference to an array.
/// R: A reference-counting structure, that wraps the label.
/// E: The elements that this array contains.
/// L: The label that is associated with this array.
/// ```
///
/// # Implementation
/// Reference counting is done by a struct stored directly next to the data;
/// this means that this struct is *a single pointer indirection* from its underlying
/// data.
#[repr(transparent)]
pub struct RcArray<A, R, E, L = ()>
where
    A: LabelledArray<E, R> + BaseArrayRef,
    R: RefCounter<L>,
{
    data: ManuallyDrop<A>,
    phantom: PhantomData<(R, E, L)>,
}

impl<A, R, E, L> RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef,
    R: RefCounter<L>,
{
    fn from_ref(ptr: A) -> Self {
        Self {
            data: ManuallyDrop::new(ptr),
            phantom: PhantomData,
        }
    }
    fn to_ref(self) -> A {
        let ret = unsafe { mem::transmute_copy(&self.data) };
        mem::forget(self);
        ret
    }
    pub fn ref_count(&self) -> usize {
        self.data.get_label().counter()
    }
    pub fn to_owned(self) -> Result<A, Self> {
        if self.ref_count() > 1 {
            Err(self)
        } else {
            Ok(self.to_ref())
        }
    }
}

impl<A, R, E, L> RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef + Clone,
    R: RefCounter<L>,
{
    pub fn make_owned(&self) -> A {
        (*self.data).clone()
    }
}

impl<A, R, E, L> BaseArrayRef for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef,
    R: RefCounter<L>,
{
}

impl<A, R, E, L> Clone for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef,
    R: RefCounter<L>,
{
    fn clone(&self) -> Self {
        (*self.data).get_label().increment();
        unsafe { mem::transmute_copy(self) }
    }
}

impl<A, R, E, L> ArrayRef for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef,
    R: RefCounter<L>,
{
}

impl<A, R, E, L> Index<usize> for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef + Index<usize, Output = E>,
    R: RefCounter<L>,
{
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        &self.data[idx]
    }
}

impl<A, R, E, L> Drop for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef,
    R: RefCounter<L>,
{
    fn drop(&mut self) {
        let ref_count = self.data.get_label().decrement();

        if ref_count == 0 {
            unsafe {
                ptr::drop_in_place(&mut *self.data);
            }
        }
    }
}

impl<A, R, E, L> Container for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef,
    R: RefCounter<L>,
{
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<A, R, E, L> CopyMap<usize, E> for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef,
    R: RefCounter<L>,
{
    /// Get a reference into this array. Returns `None` if:
    ///
    /// - The index given is out-of-bounds
    fn get(&self, key: usize) -> Option<&E> {
        self.data.get(key)
    }
    /// Get a mutable reference into this array. Returns `None` if:
    ///
    /// - The array is referenced by another pointer
    /// - The index given is out-of-bounds
    fn get_mut(&mut self, key: usize) -> Option<&mut E> {
        if self.data.get_label().counter() == 1 {
            self.data.get_mut(key)
        } else {
            None
        }
    }
    /// Insert an element into this array. Returns `None` if:
    ///
    /// - The array is referenced by another pointer
    /// - The index given is out-of-bounds
    /// - There was nothing in the slot previously
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        if self.data.get_label().counter() == 1 {
            self.data.insert(key, value)
        } else {
            None
        }
    }
}

impl<A, R, E, L> LabelledArray<E, L> for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef,
    R: RefCounter<L>,
{
    fn with_label<F>(label: L, len: usize, mut func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let new_ptr = A::with_label(R::new(label), len, |rc_struct, idx| {
            func(&mut rc_struct.get_data_mut(), idx)
        });
        Self::from_ref(new_ptr)
    }
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self {
        Self::from_ref(A::with_label_unsafe(R::new(label), len))
    }
    fn get_label(&self) -> &L {
        self.data.get_label().get_data()
    }
    unsafe fn get_label_unsafe(&self) -> &mut L {
        self.data.get_label_unsafe().get_data_mut()
    }
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E {
        self.data.get_unsafe(idx)
    }
}

impl<A, R, E, L> LabelledArrayRefMut<E, L> for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef + LabelledArrayMut<E, R>,
    R: RefCounter<L>,
{
    fn get_label_mut(&mut self) -> Option<&mut L> {
        if self.data.get_label().counter() == 1 {
            Some(self.data.get_label_mut().get_data_mut())
        } else {
            None
        }
    }
}

impl<A, R, E> MakeArray<E> for RcArray<A, R, E, ()>
where
    A: LabelledArray<E, R> + BaseArrayRef,
    R: RefCounter<()>,
{
    fn new<F>(len: usize, mut func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self::with_label((), len, |_, idx| func(idx))
    }
}

impl<A, R, E, L> DefaultLabelledArray<E, L> for RcArray<A, R, E, L>
where
    A: DefaultLabelledArray<E, R> + LabelledArray<E, R> + BaseArrayRef,
    R: RefCounter<L>,
    E: Default,
{
    fn with_len(label: L, len: usize) -> Self {
        Self::from_ref(A::with_len(R::new(label), len))
    }
}

impl<A, R, E, L> SliceArrayRef<E> for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef + SliceArray<E>,
    R: RefCounter<L>,
{
    fn as_slice(&self) -> &[E] {
        self.data.as_slice()
    }
    fn as_slice_mut(&mut self) -> Option<&mut [E]> {
        if self.ref_count() > 1 {
            None
        } else {
            Some(self.data.as_slice_mut())
        }
    }
}

impl<'b, A, R, E, L> IntoIterator for &'b RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef + SliceArray<E>,
    R: RefCounter<L>,
{
    type Item = &'b E;
    type IntoIter = core::slice::Iter<'b, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().into_iter()
    }
}

impl<A, R, E, L> fmt::Debug for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef + SliceArray<E>,
    R: RefCounter<L>,
    E: fmt::Debug,
    L: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        // maybe change this when const generics become stable? I.e. change the
        // name of the struct.
        formatter
            .debug_struct("RcArray")
            .field("label", &self.get_label())
            .field("ref_count", &self.ref_count())
            .field("len", &self.len())
            .field("elements", &self.as_slice())
            .finish()
    }
}

unsafe impl<A, R, E, L> Send for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef,
    R: RefCounter<L> + Send + Sync,
    E: Send + Sync,
    L: Send + Sync,
{
}

unsafe impl<A, R, E, L> Sync for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef,
    R: RefCounter<L> + Send + Sync,
    E: Send + Sync,
    L: Send + Sync,
{
}

impl<A, R, E, L> AtomicArrayRef for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + BaseArrayRef + AtomicArrayRef,
    R: RefCounter<L>,
{
    fn as_ref(&self) -> usize {
        self.data.as_ref()
    }
    fn compare_and_swap(&self, current: usize, new: Self, order: Ordering) -> Self {
        Self::from_ref((*self.data).compare_and_swap(current, new.to_ref(), order))
    }
    fn compare_exchange(
        &self,
        current: usize,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        match (*self.data).compare_exchange(current, new.to_ref(), success, failure) {
            Ok(data) => Ok(Self::from_ref(data)),
            Err(data) => Err(Self::from_ref(data)),
        }
    }
    fn compare_exchange_weak(
        &self,
        current: usize,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        match (*self.data).compare_exchange_weak(current, new.to_ref(), success, failure) {
            Ok(data) => Ok(Self::from_ref(data)),
            Err(data) => Err(Self::from_ref(data)),
        }
    }
    fn swap(&self, ptr: Self, order: Ordering) -> Self {
        Self::from_ref((*self.data).swap(ptr.to_ref(), order))
    }
}
