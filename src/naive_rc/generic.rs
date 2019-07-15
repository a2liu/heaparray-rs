//! Contains definition for `RcArray`, which is an implementation-agnositc,
//! reference-counted array.

use super::ref_counters::*;
pub use crate::api_prelude_rc::*;
use crate::prelude::*;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr;

/// `RcArray` is a generic, implementation-agnositc array. It contains
/// logic for enforcing type safety.
///
/// The type parameters are, in order:
///
/// ```text
/// A: A struct that acts as a reference to an array.
/// R: A reference-counting structure, that wraps the label. Note that this
///    *can be the label itself*. It just needs to have defined ways of incrementing
///    and decrementing its reference count.
/// E: The elements that this array contains.
/// L: The label that is associated with this array.
/// ```
#[repr(transparent)]
pub struct RcArray<A, R, E, L = ()>
where
    A: LabelledArray<E, R>,
    R: RefCounter<L>,
{
    data: ManuallyDrop<A>,
    phantom: PhantomData<(R, E, L)>,
}

impl<A, R, E, L> RcArray<A, R, E, L>
where
    A: LabelledArray<E, R>,
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
    /// Returns the reference count of the data this `RcArray` points to.
    pub fn ref_count(&self) -> usize {
        self.data.get_label().counter()
    }
    /// Returns an owned version of this array if the caller has exclusive access,
    /// or returns back this reference otherwise.
    pub fn to_owned(self) -> Result<A, Self> {
        if self.ref_count() > 1 {
            Err(self)
        } else {
            Ok(self.to_ref())
        }
    }
    /// Returns a mutable reference to the array if the caller has exclusive access,
    /// or `None` otherwise.
    pub fn to_mut(&mut self) -> Option<&mut A> {
        if self.ref_count() > 1 {
            None
        } else {
            Some(&mut *self.data)
        }
    }
}

impl<A, R, E, L> RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + Clone,
    R: RefCounter<L>,
{
    /// Returns an owned version of this array if the caller has exclusive access,
    /// or copies the data otherwise.
    pub fn make_owned(self) -> A {
        if self.ref_count() > 1 {
            (*self.data).clone()
        } else {
            self.to_ref()
        }
    }
    /// Returns a clone of the data in this array.
    ///
    /// Behaves differently from `ArrayRef::clone()`, as the result of this operation
    /// **does not** point to the same data.
    pub fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            phantom: PhantomData,
        }
    }
    /// Returns a mutable reference to the array if the caller has exclusive access,
    /// or copies the data otherwise.
    pub fn make_mut(&mut self) -> &mut A {
        if self.ref_count() > 1 {
            *self = Self::from_ref((*self.data).clone());
        }
        &mut *self.data
    }
}

impl<A, R, E, L> Clone for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R>,
    R: RefCounter<L>,
{
    fn clone(&self) -> Self {
        (*self.data).get_label().increment();
        let ret = unsafe { mem::transmute_copy(self) };
        ret
    }
}

impl<A, R, E, L> ArrayRef for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R>,
    R: RefCounter<L>,
{
}

impl<A, R, E, L> Index<usize> for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + Index<usize, Output = E>,
    R: RefCounter<L>,
{
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        &self.data[idx]
    }
}

impl<A, R, E, L> Drop for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R>,
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
    A: LabelledArray<E, R>,
    R: RefCounter<L>,
{
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<A, R, E, L> CopyMap<usize, E> for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R>,
    R: RefCounter<L>,
{
    /// Get a reference into this array. Returns `None` if and only if:
    ///
    /// - The index given is out-of-bounds
    fn get(&self, key: usize) -> Option<&E> {
        self.data.get(key)
    }
    /// Get a mutable reference into this array. Returns `None` if and only if:
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
    /// Insert an element into this array, returning the previous element. Returns
    /// `None` if and only if:
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
    A: LabelledArray<E, R>,
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
    fn get_label(&self) -> &L {
        self.data.get_label().get_data()
    }
    unsafe fn get_unchecked(&self, idx: usize) -> &E {
        self.data.get_unchecked(idx)
    }
}

impl<A, R, E> MakeArray<E> for RcArray<A, R, E, ()>
where
    A: LabelledArray<E, R>,
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
    A: DefaultLabelledArray<E, R> + LabelledArray<E, R>,
    R: RefCounter<L>,
    E: Default,
{
    fn with_len(label: L, len: usize) -> Self {
        Self::from_ref(A::with_len(R::new(label), len))
    }
}

impl<A, R, E, L> SliceArray<E> for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + SliceArray<E>,
    R: RefCounter<L>,
{
    fn as_slice(&self) -> &[E] {
        self.data.as_slice()
    }
}

impl<A, R, E, L> Index<Range<usize>> for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + SliceArray<E>,
    R: RefCounter<L>,
{
    type Output = [E];
    fn index(&self, idx: Range<usize>) -> &[E] {
        &self.as_slice()[idx]
    }
}

impl<'b, A, R, E, L> IntoIterator for &'b RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + SliceArray<E>,
    R: RefCounter<L>,
{
    type Item = &'b E;
    type IntoIter = core::slice::Iter<'b, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().into_iter()
    }
}

impl<'a, A, R, E, L, A2, R2, E2, L2> PartialEq<RcArray<A2, R2, E2, L2>> for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + SliceArray<E> + PartialEq<A2>,
    R: RefCounter<L>,
    A2: LabelledArray<E2, R2> + SliceArray<E2>,
    R2: RefCounter<L2>,
{
    fn eq(&self, other: &RcArray<A2, R2, E2, L2>) -> bool {
        self.data.eq(&other.data)
    }
}

impl<'a, A, R, E, L> Eq for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + SliceArray<E> + Eq,
    R: RefCounter<L>,
{
}

impl<A, R, E, L> fmt::Debug for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + SliceArray<E>,
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
    A: LabelledArray<E, R> + Send + Sync,
    R: RefCounter<L> + Send + Sync,
    E: Send + Sync,
    L: Send + Sync,
{
}

unsafe impl<A, R, E, L> Sync for RcArray<A, R, E, L>
where
    A: LabelledArray<E, R> + Send + Sync,
    R: RefCounter<L> + Send + Sync,
    E: Send + Sync,
    L: Send + Sync,
{
}
