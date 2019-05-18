//! Contains definition for `RcArray`, which is an implementation-agnositc,
//! reference-counted array.
use super::ref_counters::*;
use crate::prelude::*;
use core::marker::PhantomData;
use core::sync::atomic::Ordering;

// A is the array reference that this type wraps
// R is the reference counter for the label of this type
// L is the label of this type
// E is the element that this type is an array of
// B is the type that A dereferences to using its `.to_null()` method.
/// `RcArray` is a generic, implementation-agnositc array. It uses traits to
/// handle literally everything.
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
/// Since this struct is generic, feel free to go ham with implementation details.
#[repr(C)]
pub struct RcArray<'a, A, R, E, L = ()>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef,
    R: 'a + RefCounter<L>,
    E: 'a,
    L: 'a,
{
    data: ManuallyDrop<A>,
    phantom: PhantomData<(&'a E, R, L)>,
}

impl<'a, A, R, E, L> RcArray<'a, A, R, E, L>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef,
    R: 'a + RefCounter<L>,
    E: 'a,
    L: 'a,
{
    fn check_null(&self) {
        #[cfg(not(feature = "no-asserts"))]
        assert!(
            !self.is_null(),
            "Null dereference of reference-counted array."
        );
    }
    fn from_ref(ptr: A) -> Self {
        Self {
            data: ManuallyDrop::new(ptr),
            phantom: PhantomData,
        }
    }
    // This would be unsafe if the `A` were returned to caller, but since it
    // isn't, everything should be fiiiiiiiine
    fn to_ref(self) -> A {
        let ret = unsafe { mem::transmute_copy(&self.data) };
        mem::forget(self);
        ret
    }
    pub fn ref_count(&self) -> usize {
        self.data.get_label().counter()
    }
}

impl<'a, A, R, E, L> BaseArrayRef for RcArray<'a, A, R, E, L>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef,
    R: 'a + RefCounter<L>,
    E: 'a,
    L: 'a,
{
    #[inline]
    fn is_null(&self) -> bool {
        (*self.data).is_null()
    }
}

impl<'a, A, R, E, L> Clone for RcArray<'a, A, R, E, L>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef,
    R: 'a + RefCounter<L>,
    E: 'a,
    L: 'a,
{
    fn clone(&self) -> Self {
        if self.is_null() {
            Self::null_ref()
        } else {
            (*self.data).get_label().increment();
            unsafe { mem::transmute_copy(self) }
        }
    }
}

impl<'a, A, R, E, L> ArrayRef for RcArray<'a, A, R, E, L>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef,
    R: 'a + RefCounter<L>,
    E: 'a,
    L: 'a,
{
    fn to_null(&mut self) {
        if self.is_null() {
            return;
        } else {
            let ref_count = self.data.get_label().decrement();

            // Set this reference to null
            let other = mem::replace(self, Self::null_ref());

            if ref_count == 0 {
                let to_drop: A = unsafe { mem::transmute_copy(&*other.data) };
                mem::drop(to_drop); // Drop is here to be explicit about intent
            }
            mem::forget(other);
        }
    }

    fn null_ref() -> Self {
        Self::from_ref(unsafe { A::null_ref() })
    }
}

impl<'a, A, R, E, L> Index<usize> for RcArray<'a, A, R, E, L>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef + Index<usize, Output = E>,
    R: 'a + RefCounter<L>,
    E: 'a,
    L: 'a,
{
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        self.check_null();
        &self.data[idx]
    }
}

impl<'a, A, R, E, L> Drop for RcArray<'a, A, R, E, L>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef,
    R: 'a + RefCounter<L>,
    E: 'a,
    L: 'a,
{
    fn drop(&mut self) {
        self.to_null();
        mem::forget(self);
    }
}

impl<'a, A, R, E, L> Container for RcArray<'a, A, R, E, L>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef,
    R: 'a + RefCounter<L>,
    E: 'a,
    L: 'a,
{
    fn len(&self) -> usize {
        self.check_null();
        self.data.len()
    }
}

impl<'a, A, R, E, L> CopyMap<usize, E> for RcArray<'a, A, R, E, L>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef,
    R: 'a + RefCounter<L>,
    E: 'a,
    L: 'a,
{
    fn get(&self, key: usize) -> Option<&E> {
        self.check_null();
        self.data.get(key)
    }
    fn get_mut(&mut self, key: usize) -> Option<&mut E> {
        self.check_null();
        if self.data.get_label().counter() == 1 {
            self.data.get_mut(key)
        } else {
            None
        }
    }
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        self.check_null();
        if self.data.get_label().counter() == 1 {
            self.data.insert(key, value)
        } else {
            None
        }
    }
}

impl<'a, A, R, E, L> LabelledArray<E, L> for RcArray<'a, A, R, E, L>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef,
    R: 'a + RefCounter<L>,
    E: 'a,
    L: 'a,
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
        self.check_null();
        self.data.get_label().get_data()
    }
    unsafe fn get_label_unsafe(&self) -> &mut L {
        self.data.get_label_unsafe().get_data_mut()
    }
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E {
        self.data.get_unsafe(idx)
    }
}

impl<'a, A, R, E, L> LabelledArrayRefMut<E, L> for RcArray<'a, A, R, E, L>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef + LabelledArrayMut<E, R>,
    R: 'a + RefCounter<L>,
    E: 'a,
    L: 'a,
{
    fn get_label_mut(&mut self) -> Option<&mut L> {
        self.check_null();
        if self.data.get_label().counter() == 1 {
            Some(self.data.get_label_mut().get_data_mut())
        } else {
            None
        }
    }
}

impl<'a, A, R, E> MakeArray<E> for RcArray<'a, A, R, E, ()>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef,
    R: 'a + RefCounter<()>,
    E: 'a,
{
    fn new<F>(len: usize, mut func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self::with_label((), len, |_, idx| func(idx))
    }
}

impl<'a, A, R, E, L> DefaultLabelledArray<E, L> for RcArray<'a, A, R, E, L>
where
    A: 'a + DefaultLabelledArray<E, R> + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef,
    R: 'a + RefCounter<L>,
    E: 'a + Default,
    L: 'a,
{
    fn with_len(label: L, len: usize) -> Self {
        Self::from_ref(A::with_len(R::new(label), len))
    }
}

unsafe impl<'a, A, R, E, L> Send for RcArray<'a, A, R, E, L>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef,
    R: 'a + RefCounter<L> + Send + Sync,
    E: 'a + Send + Sync,
    L: 'a + Send + Sync,
{
}

unsafe impl<'a, A, R, E, L> Sync for RcArray<'a, A, R, E, L>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef,
    R: 'a + RefCounter<L> + Send + Sync,
    E: 'a + Send + Sync,
    L: 'a + Send + Sync,
{
}

impl<'a, A, R, E, L> AtomicArrayRef for RcArray<'a, A, R, E, L>
where
    A: 'a + LabelledArray<E, R> + BaseArrayRef + UnsafeArrayRef + AtomicArrayRef,
    R: 'a + RefCounter<L>,
    E: 'a,
    L: 'a,
{
    fn compare_and_swap(&self, current: Self, new: Self, order: Ordering) -> Self {
        Self::from_ref((*self.data).compare_and_swap(current.to_ref(), new.to_ref(), order))
    }
    fn compare_exchange(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        match (*self.data).compare_exchange(current.to_ref(), new.to_ref(), success, failure) {
            Ok(data) => Ok(Self::from_ref(data)),
            Err(data) => Err(Self::from_ref(data)),
        }
    }
    fn compare_exchange_weak(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        match (*self.data).compare_exchange_weak(current.to_ref(), new.to_ref(), success, failure) {
            Ok(data) => Ok(Self::from_ref(data)),
            Err(data) => Err(Self::from_ref(data)),
        }
    }
    fn load(&self, order: Ordering) -> Self {
        Self::from_ref((*self.data).load(order))
    }
    fn store(&self, ptr: Self, order: Ordering) {
        (*self.data).store(ptr.to_ref(), order);
    }
    fn swap(&self, ptr: Self, order: Ordering) -> Self {
        Self::from_ref((*self.data).swap(ptr.to_ref(), order))
    }
}
