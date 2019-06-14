use crate::base::mem_block::*;
use crate::ptr_utils::UnsafePtr;
use crate::traits::AtomicArrayRef;
use atomic_types::*;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::sync::atomic::Ordering;
use core::{mem, ptr};

/// Base array that handles converting a memory block into a constructible object.
///
/// Doesn't store length information, but contains logic necessary to handle
/// allocation, deallocation, iteration, and slices given length. Holds
/// the bulk of the unsafe logic in this library.
///
/// # Invariants
/// This struct follows some of the invariants as discussed in
/// [`heaparray::base::mem_block::MemBlock`](mem_block/struct.MemBlock.html),
/// as it internally is just a pointer to a `MemBlock`. Specifically, it maintains
/// the invariant that the memory block allocated will always have a size (in bytes)
/// less than or equal to `core::isize::MAX`. However, note that the internal
/// pointer isn't necessarily valid; while the associated functions `new` and
/// `new_lazy` do uphold this invariant, this type can be constructed without
/// allocating anything (e.g. `BaseArray::<u8, ()>::from_ptr(core::ptr::null())`).
///
/// # Safety
/// Generally, the functions of this struct are safe given that the length that
/// you provide to the function is less than or equal to that of the underlying
/// memory. Functions with more restrictive requirements will describe those
/// requirements in more detail.
#[repr(transparent)]
pub struct BaseArray<E, L, P = NonNull<MemBlock<E, L>>>
where
    P: UnsafePtr<MemBlock<E, L>>,
{
    data: P,
    phantom: PhantomData<(E, L, *mut u8)>,
}

/// Iterator for an instance of `BaseArray` that takes ownership of the array
///
/// `BaseArray` can't be safely iterated over, so this object can only be constructed
/// via the unsafe method `BaseArray::into_iter`, which takes as a parameter an
/// associated length.
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
    /// Get a mutable reference to this struct's memory block
    fn _mut(&mut self) -> &mut MemBlock<E, L> {
        unsafe { self.data.as_mut() }
    }

    /// Get a reference to this struct's memory block
    fn _ref(&self) -> &MemBlock<E, L> {
        unsafe { self.data.as_ref() }
    }

    /// Construct an instance of this struct from a raw pointer; doesn't do any
    /// checking for validity of the pointer.
    pub unsafe fn from_ptr(ptr: *mut MemBlock<E, L>) -> Self {
        Self::from_ref(P::new_unchecked(ptr))
    }

    /// Construct an instance of this struct from an instance of the pointer type
    /// `P`.
    pub unsafe fn from_ref(ptr: P) -> Self {
        Self {
            data: ptr,
            phantom: PhantomData,
        }
    }

    /// Creates a new array of size `len`.
    ///
    /// Initializes all elements using the given function, and initializes the
    /// label with the provided value.
    pub fn new<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        unsafe { Self::from_ptr(MemBlock::new_init(label, len, func).as_ptr()) }
    }

    /// Doesn't initialize anything in the array. Just allocates a block of memory.
    pub unsafe fn alloc(len: usize) -> Self {
        Self::from_ptr(MemBlock::alloc(len).as_ptr())
    }

    /// Doesn't initialize the elements of the array.
    pub unsafe fn new_lazy(label: L, len: usize) -> Self {
        Self::from_ptr(MemBlock::new(label, len).as_ptr())
    }

    /// Returns the internal pointer of this array as a pointer to a `MemBlock`
    /// instance.
    pub unsafe fn as_block_ptr(&self) -> *const MemBlock<E, L> {
        self.data.as_ref()
    }

    /// Returns the internal pointer of this array as a mutable pointer to a
    /// `MemBlock` instance.
    pub unsafe fn as_block_ptr_mut(&mut self) -> *mut MemBlock<E, L> {
        self.data.as_mut()
    }

    /// Runs destructor code for elements and for label, then deallocates block.
    pub unsafe fn drop(&mut self, len: usize) {
        self._mut().dealloc(len)
    }

    /// Deallocates block without running destructor code for elements or label.
    pub unsafe fn drop_lazy(&mut self, len: usize) {
        self._mut().dealloc_lazy(len)
    }

    /// Cast this array into a different array.
    pub unsafe fn cast_into<T, Q>(self) -> BaseArray<T, L, Q>
    where
        Q: UnsafePtr<MemBlock<T, L>>,
    {
        let mut ptr = self.data.cast::<MemBlock<T, L>, Q>();
        BaseArray::<T, L, Q>::from_ptr(ptr.as_mut() as *mut MemBlock<T, L>)
    }

    /// Cast a reference to this array into a reference to a different array.
    pub unsafe fn cast_ref<T, Q>(&self) -> &BaseArray<T, L, Q>
    where
        Q: UnsafePtr<MemBlock<T, L>>,
    {
        &*(self as *const BaseArray<E, L, P> as *const BaseArray<T, L, Q>)
    }

    /// Cast a mutable reference to this array into a mutable reference to a
    /// different array.
    pub unsafe fn cast_mut<T, Q>(&mut self) -> &mut BaseArray<T, L, Q>
    where
        Q: UnsafePtr<MemBlock<T, L>>,
    {
        &mut *(self as *mut BaseArray<E, L, P> as *mut BaseArray<T, L, Q>)
    }

    /// Returns a pointer to the element at the index `idx`
    pub fn get_ptr(&self, idx: usize) -> *const E {
        self._ref().get_ptr(idx)
    }

    /// Returns a mutable pointer to the element at the index `idx`
    pub fn get_ptr_mut(&mut self, idx: usize) -> *mut E {
        self._mut().get_ptr_mut(idx)
    }

    /// Returns whether or not the internal pointer in this array is null
    pub fn is_null(&self) -> bool {
        self.data.is_null()
    }

    /// Returns a reference to the element at the index `idx`
    pub unsafe fn get(&self, idx: usize) -> &E {
        &*self.get_ptr(idx)
    }

    /// Returns a mutable reference to the element at the index `idx`
    pub unsafe fn get_mut(&mut self, idx: usize) -> &mut E {
        &mut *self.get_ptr_mut(idx)
    }

    /// Returns a reference to the label
    pub fn get_label(&self) -> &L {
        self._ref().get_label()
    }

    /// Returns a mutable reference to the label
    pub fn get_label_mut(&mut self) -> &mut L {
        self._mut().get_label_mut()
    }

    /// Returns a reference to a slice into this array
    ///
    /// The slice is from element 0 to `len - 1` inclusive.
    pub unsafe fn as_slice(&self, len: usize) -> &[E] {
        core::slice::from_raw_parts(self.get(0), len)
    }

    /// Returns a mutable reference to a slice into this array
    ///
    /// The slice is from element 0 to `len - 1` inclusive.
    pub unsafe fn as_slice_mut(&mut self, len: usize) -> &mut [E] {
        core::slice::from_raw_parts_mut(self.get_mut(0), len)
    }

    /// Returns an iterator into this array, consuming the array in the process
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
    /// Clones the elements and label of this array into a new array of the same
    /// size
    pub unsafe fn clone(&self, len: usize) -> Self {
        Self::new(self.get_label().clone(), len, |_, i| self.get(i).clone())
    }
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

impl<E, L, P> AtomicArrayRef for BaseArray<E, L, P>
where
    P: UnsafePtr<MemBlock<E, L>> + Atomic<*mut MemBlock<E, L>>,
{
    fn as_ref(&self) -> usize {
        self._ref() as *const MemBlock<E, L> as usize
    }
    fn compare_and_swap(
        &self,
        current: usize,
        mut new: Self,
        order: Ordering,
    ) -> Result<usize, (Self, usize)> {
        let current = current as *mut MemBlock<E, L>;
        let new_ref = new._mut() as *mut MemBlock<E, L>;
        let actual = self.data.compare_and_swap(current, new_ref, order);
        if actual == current {
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
        let current = current as *mut MemBlock<E, L>;
        let new_ref = new.as_ref() as *const MemBlock<E, L> as *mut MemBlock<E, L>;
        match self
            .data
            .compare_exchange(current, new_ref, success, failure)
        {
            Ok(ptr) => Ok(ptr as usize),
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
        let current = current as *mut MemBlock<E, L>;
        let new_ref = new.as_ref() as *const MemBlock<E, L> as *mut MemBlock<E, L>;
        match self
            .data
            .compare_exchange_weak(current, new_ref, success, failure)
        {
            Ok(ptr) => Ok(ptr as usize),
            Err(ptr) => Err((new, ptr as usize)),
        }
    }
    fn swap(&self, mut ptr: Self, order: Ordering) -> Self {
        unsafe {
            Self::from_ref(<P as UnsafePtr<MemBlock<E, L>>>::new(
                self.data.swap(ptr._mut(), order),
            ))
        }
    }
}
