use super::mem_block::*;
pub use crate::prelude::*;
use crate::ptr_utils::UnsafePtr;
use core::marker::PhantomData;
use core::ptr::NonNull;

/// Base array that handles converting a memory block into a constructible object.
///
/// Doesn't store length information, but contains logic necessary to handle
/// allocation, deallocation, iteration, and slices given length. Holds
/// the bulk of the unsafe logic in this library.
#[repr(transparent)]
pub struct BaseArray<E, L, P = NonNull<MemBlock<E, L>>>
where
    P: UnsafePtr<MemBlock<E, L>>,
{
    data: P,
    phantom: PhantomData<(E, L)>,
}

pub struct BaseArrayIter<E, L, P = NonNull<MemBlock<E, L>>>
where
    P: UnsafePtr<MemBlock<E, L>>,
{
    array: BaseArray<E, L, P>,
    current: *mut E,
    end: *mut E,
}

impl<E, L, P> BaseArray<E, L, P>
where
    P: UnsafePtr<MemBlock<E, L>>,
{
    fn _mut(&mut self) -> &mut MemBlock<E, L> {
        unsafe { self.data.as_mut() }
    }

    fn _ref(&self) -> &MemBlock<E, L> {
        unsafe { self.data.as_ref() }
    }

    pub fn new<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        unsafe { Self::from_ptr(MemBlock::new_init(label, len, func).as_ptr()) }
    }

    /// Doesn't initialize the elements of the array, or check for null references.
    pub unsafe fn new_lazy(label: L, len: usize) -> Self {
        Self::from_ptr(MemBlock::new(label, len).as_ptr())
    }

    pub unsafe fn from_ptr(ptr: *mut MemBlock<E, L>) -> Self {
        Self::from_ref(P::new_unchecked(ptr))
    }

    pub unsafe fn from_ref(ptr: P) -> Self {
        Self {
            data: ptr,
            phantom: PhantomData,
        }
    }

    pub unsafe fn as_ptr(&self) -> *const MemBlock<E, L> {
        self.data.as_ref()
    }

    pub unsafe fn as_ptr_mut(&mut self) -> *mut MemBlock<E, L> {
        self.data.as_mut()
    }

    pub unsafe fn drop(&mut self, len: usize) {
        self._mut().dealloc(len)
    }

    pub unsafe fn drop_lazy(&mut self, len: usize) {
        self._mut().dealloc_lazy(len)
    }

    pub unsafe fn cast_into<T, Q>(self) -> BaseArray<T, L, Q>
    where
        Q: UnsafePtr<MemBlock<T, L>>,
    {
        let mut ptr = self.data.cast::<MemBlock<T, L>, Q>();
        BaseArray::<T, L, Q>::from_ptr(ptr.as_mut() as *mut MemBlock<T, L>)
    }

    pub unsafe fn cast_ref<T, Q>(&self) -> &BaseArray<T, L, Q>
    where
        Q: UnsafePtr<MemBlock<T, L>>,
    {
        &*(self as *const BaseArray<E, L, P> as *const BaseArray<T, L, Q>)
    }

    pub unsafe fn cast_mut<T, Q>(&mut self) -> &mut BaseArray<T, L, Q>
    where
        Q: UnsafePtr<MemBlock<T, L>>,
    {
        &mut *(self as *mut BaseArray<E, L, P> as *mut BaseArray<T, L, Q>)
    }

    pub fn get_ptr(&self, idx: usize) -> *const E {
        self._ref().get_ptr(idx)
    }

    pub fn get_ptr_mut(&mut self, idx: usize) -> *mut E {
        self._mut().get_ptr_mut(idx)
    }

    pub unsafe fn get(&self, idx: usize) -> &E {
        &*self.get_ptr(idx)
    }

    pub unsafe fn get_mut(&mut self, idx: usize) -> &mut E {
        &mut *self.get_ptr_mut(idx)
    }

    pub fn get_label(&self) -> &L {
        self._ref().get_label()
    }

    pub fn get_label_mut(&mut self) -> &mut L {
        self._mut().get_label_mut()
    }

    pub unsafe fn as_slice(&self, len: usize) -> &[E] {
        core::slice::from_raw_parts(self.get(0), len)
    }

    pub unsafe fn as_slice_mut(&mut self, len: usize) -> &mut [E] {
        core::slice::from_raw_parts_mut(self.get_mut(0), len)
    }

    pub unsafe fn into_iter(mut self, len: usize) -> BaseArrayIter<E, L, P> {
        let current = self.get_mut(0) as *mut E;
        let end = current.add(len);
        BaseArrayIter {
            array: self,
            current,
            end,
        }
    }
}

impl<E, L, P> BaseArray<E, L, P>
where
    E: Clone,
    L: Clone,
    P: UnsafePtr<MemBlock<E, L>>,
{
    pub unsafe fn clone(&self, len: usize) -> Self {
        Self::new(self.get_label().clone(), len, |_, i| self.get(i).clone())
    }
}

unsafe impl<E, L, P> Send for BaseArray<E, L, P>
where
    E: Send,
    L: Send,
    P: UnsafePtr<MemBlock<E, L>>,
{
}

unsafe impl<E, L, P> Sync for BaseArray<E, L, P>
where
    E: Sync,
    L: Sync,
    P: UnsafePtr<MemBlock<E, L>>,
{
}

impl<E, L, P> Iterator for BaseArrayIter<E, L, P>
where
    P: UnsafePtr<MemBlock<E, L>>,
{
    type Item = E;
    fn next(&mut self) -> Option<E> {
        if self.current == self.end {
            None
        } else {
            unsafe {
                let out = Some(ptr::read(self.current));
                self.current = self.current.add(1);
                out
            }
        }
    }
}

impl<E, L, P> Drop for BaseArrayIter<E, L, P>
where
    P: UnsafePtr<MemBlock<E, L>>,
{
    fn drop(&mut self) {
        let begin = self.array.get_ptr_mut(0) as usize;
        let len = ((self.end as usize) - begin) / mem::size_of::<E>();
        unsafe { self.array.drop(len) }
    }
}
