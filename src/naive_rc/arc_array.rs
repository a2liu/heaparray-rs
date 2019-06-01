//! Contains definition for `ArcArray`, which is an atomically reference counted
//! array that can be atomically initialized after construction.

use super::ref_counters::*;
use crate::base::AtomicPtrArray;
pub use crate::prelude::*;
use core::sync::atomic::Ordering;

#[repr(transparent)]
pub struct ArcArray<E, L = ()> {
    data: ManuallyDrop<AtomicPtrArray<E, ArcStruct<L>>>,
}

impl<E, L> ArcArray<E, L> {
    fn from_ref(data: AtomicPtrArray<E, ArcStruct<L>>) -> Self {
        Self {
            data: ManuallyDrop::new(data),
        }
    }
    fn to_ref(self) -> AtomicPtrArray<E, ArcStruct<L>> {
        let data = unsafe { mem::transmute_copy(&self.data) };
        mem::forget(self);
        data
    }
    fn check_null(&self, message: &'static str) {
        if cfg!(not(feature = "no-asserts")) {
            core::sync::atomic::fence(Ordering::Acquire);
            assert!(
                !self.is_null(),
                format!(
                    "heaparray::naive_rc::ArcArray null pointer dereference in `{}`",
                    message
                )
            );
        }
    }
    pub fn null_ref() -> Self {
        Self::from_ref(unsafe { AtomicPtrArray::null_ref() })
    }
    pub fn is_null(&self) -> bool {
        self.data.is_null()
    }
    pub fn ref_count(&self) -> usize {
        self.check_null("ref_count");
        self.data.get_label().counter()
    }
    pub fn initialize<F>(&self, label: L, len: usize, mut func: F) -> Result<(), Self>
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let data = AtomicPtrArray::with_label(ArcStruct::new(label), len, |l, i| {
            func(l.get_data_mut(), i)
        });

        match self.data.compare_and_swap(
            unsafe { AtomicPtrArray::<E, L>::null_ref().as_ref() },
            data,
            Ordering::AcqRel,
        ) {
            Ok(_) => Ok(()),
            Err((data, _)) => Err(Self::from_ref(data)),
        }
    }
    pub fn to_owned(self) -> Result<AtomicPtrArray<E, ArcStruct<L>>, Self> {
        self.check_null("to_owned");
        if self.ref_count() > 1 {
            Err(self)
        } else {
            Ok(self.to_ref())
        }
    }
    pub fn to_mut(&mut self) -> Result<&mut AtomicPtrArray<E, ArcStruct<L>>, ()> {
        self.check_null("to_mut");
        if self.ref_count() > 1 {
            Err(())
        } else {
            Ok(&mut *self.data)
        }
    }
}

impl<E, L> ArcArray<E, L>
where
    E: Clone,
    L: Clone,
{
    pub fn make_owned(self) -> AtomicPtrArray<E, ArcStruct<L>> {
        self.check_null("make_owned");
        if self.ref_count() > 1 {
            (*self.data).clone()
        } else {
            self.to_ref()
        }
    }
    pub fn make_mut(&mut self) -> &mut AtomicPtrArray<E, ArcStruct<L>> {
        self.check_null("make_mut");
        if self.ref_count() > 1 {
            *self = Self::from_ref((*self.data).clone());
        }
        &mut *self.data
    }
}

impl<E, L> BaseArrayRef for ArcArray<E, L> where {}

impl<E, L> Clone for ArcArray<E, L>
where
    E: Clone,
    L: Clone,
{
    fn clone(&self) -> Self {
        if !self.is_null() {
            (*self.data).get_label().increment();
        }
        unsafe { mem::transmute_copy(self) }
    }
}

impl<E, L> ArrayRef for ArcArray<E, L>
where
    E: Clone,
    L: Clone,
{
}

impl<E, L> Index<usize> for ArcArray<E, L> {
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        self.check_null("index");
        &self.data[idx]
    }
}

impl<E, L> Drop for ArcArray<E, L> {
    fn drop(&mut self) {
        if self.is_null() {
            return;
        } else {
            let ref_count = self.data.get_label().decrement();
            if ref_count == 0 {
                unsafe {
                    ptr::drop_in_place(&mut *self.data);
                }
            }
        }
    }
}

impl<E, L> Container for ArcArray<E, L> {
    fn len(&self) -> usize {
        self.check_null("len");
        self.data.len()
    }
}

impl<E, L> CopyMap<usize, E> for ArcArray<E, L> {
    /// Get a reference into this array. Returns `None` if:
    ///
    /// - The index given is out-of-bounds
    fn get(&self, key: usize) -> Option<&E> {
        self.check_null("get");
        self.data.get(key)
    }
    /// Get a mutable reference into this array. Returns `None` if:
    ///
    /// - The array is referenced by another pointer
    /// - The index given is out-of-bounds
    fn get_mut(&mut self, key: usize) -> Option<&mut E> {
        self.check_null("get_mut");
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
        self.check_null("insert");
        if self.data.get_label().counter() == 1 {
            self.data.insert(key, value)
        } else {
            None
        }
    }
}

impl<E, L> LabelledArray<E, L> for ArcArray<E, L> {
    fn with_label<F>(label: L, len: usize, mut func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let new_ptr = AtomicPtrArray::with_label(ArcStruct::new(label), len, |rc_struct, idx| {
            func(&mut rc_struct.get_data_mut(), idx)
        });
        Self::from_ref(new_ptr)
    }
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self {
        Self::from_ref(AtomicPtrArray::with_label_unsafe(
            ArcStruct::new(label),
            len,
        ))
    }
    fn get_label(&self) -> &L {
        self.check_null("get_label");
        self.data.get_label().get_data()
    }
    unsafe fn get_label_unsafe(&self) -> &mut L {
        self.data.get_label_unsafe().get_data_mut()
    }
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E {
        self.data.get_unsafe(idx)
    }
}

impl<E, L> LabelledArrayRefMut<E, L> for ArcArray<E, L> {
    fn get_label_mut(&mut self) -> Option<&mut L> {
        self.check_null("get_label_mut");
        if self.data.get_label().counter() == 1 {
            Some(self.data.get_label_mut().get_data_mut())
        } else {
            None
        }
    }
}

impl<E> MakeArray<E> for ArcArray<E, ()> {
    fn new<F>(len: usize, mut func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self::with_label((), len, |_, idx| func(idx))
    }
}

impl<E, L> DefaultLabelledArray<E, L> for ArcArray<E, L>
where
    E: Default,
{
    fn with_len(label: L, len: usize) -> Self {
        Self::from_ref(AtomicPtrArray::with_len(ArcStruct::new(label), len))
    }
}

impl<E, L> SliceArrayRef<E> for ArcArray<E, L> {
    fn as_slice(&self) -> &[E] {
        self.check_null("as_slice");
        self.data.as_slice()
    }
    fn as_slice_mut(&mut self) -> Option<&mut [E]> {
        self.check_null("as_slice_mut");
        if self.ref_count() > 1 {
            None
        } else {
            Some(self.data.as_slice_mut())
        }
    }
}

impl<'b, E, L> IntoIterator for &'b ArcArray<E, L> {
    type Item = &'b E;
    type IntoIter = core::slice::Iter<'b, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.check_null("into_iter");
        self.as_slice().into_iter()
    }
}

impl<E, L> fmt::Debug for ArcArray<E, L>
where
    E: fmt::Debug,
    L: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.is_null() {
            formatter
                .debug_struct("ArcArray")
                .field("label", &"null")
                .field("ref_count", &"null")
                .field("elements", &"null")
                .finish()
        } else {
            formatter
                .debug_struct("ArcArray")
                .field("label", &self.get_label())
                .field("ref_count", &self.ref_count())
                .field("len", &self.len())
                .field("elements", &self.as_slice())
                .finish()
        }
    }
}

unsafe impl<E, L> Send for ArcArray<E, L>
where
    E: Send + Sync,
    L: Send + Sync,
{
}

unsafe impl<E, L> Sync for ArcArray<E, L>
where
    E: Send + Sync,
    L: Send + Sync,
{
}
