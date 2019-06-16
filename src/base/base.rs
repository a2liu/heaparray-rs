use super::mem_block::*;
use super::traits::*;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::{mem, ptr};

/// Base array that handles converting a memory block into a constructible object.
///
/// Doesn't store length information, but contains logic necessary to handle
/// allocation, deallocation, iteration, and slices given length. Holds
/// the bulk of the unsafe logic in this library.
///
/// Requires the user to specify the internal structure that handles allocation
/// and pointer math, which can be done through implementing the
/// [`BaseArrayPtr`](trait.BaseArrayPtr.html) trait.
///
/// # Memory Leaks
/// This struct doesn't perform memory cleanup automatically; it must be done manually
/// with methods `drop` or `drop_lazy`.
#[repr(transparent)]
pub struct BaseArray<E, L, P = NonNull<MemBlock<E, L>>>
where
    P: BaseArrayPtr<E, L>,
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
    P: BaseArrayPtr<E, L>,
{
    array: BaseArray<E, L, P>,
    current: *mut E,
    end: *mut E,
}

impl<E, L, P> BaseArray<E, L, P>
where
    P: BaseArrayPtr<E, L>,
{
    /// Construct an instance of this struct from an instance of the pointer type
    /// `P`.
    pub unsafe fn from_ptr(ptr: P) -> Self {
        Self {
            data: ptr,
            phantom: PhantomData,
        }
    }

    /// Returns a reference to the underlying pointer of this base array.
    pub fn as_ptr(&self) -> &P {
        &self.data
    }

    /// Returns a mutable reference to the underlying pointer of this base array.
    pub fn as_ptr_mut(&mut self) -> &mut P {
        &mut self.data
    }

    /// Doesn't initialize anything in the array. Just allocates a block of memory.
    pub unsafe fn alloc(len: usize) -> Self {
        let mut array = Self::from_ptr(P::alloc(len));
        array.data._init();
        array
    }

    /// Doesn't initialize the elements of the array.
    pub unsafe fn new_lazy(label: L, len: usize) -> Self {
        let mut array = Self::alloc(len);
        ptr::write(array.get_label_mut(), label);
        array
    }

    /// Creates a new array of size `len`.
    ///
    /// Initializes all elements using the given function, and initializes the
    /// label with the provided value.
    pub fn new<F>(label: L, len: usize, mut func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let array = unsafe { Self::new_lazy(label, len) };
        for i in 0..len {
            unsafe {
                ptr::write(array.data.elem_ptr(i), func(&mut *array.data.lbl_ptr(), i));
            }
        }
        array
    }

    /// Runs destructor code for elements and for label, then deallocates block.
    ///
    /// # Safety
    /// Function is safe as long as the underlying array is at least length `len`,
    /// and the elements in the array have been initialized.
    pub unsafe fn drop(&mut self, len: usize) {
        ptr::drop_in_place(self.get_label_mut());
        for i in 0..len {
            ptr::drop_in_place(self.data.elem_ptr(i));
        }
        self.drop_lazy(len);
    }

    /// Deallocates block without running destructor code for elements or label.
    ///
    /// # Safety
    /// Function is safe as long as the underlying array is at least length `len`.
    pub unsafe fn drop_lazy(&mut self, len: usize) {
        self.data._drop();
        self.data.dealloc(len);
    }

    /// Cast this array into a different array.
    ///
    /// Doesn't alter the length information of the array at all, or perform
    /// alignment/reference checks.
    pub unsafe fn cast_into<T, Q>(self) -> BaseArray<T, L, Q>
    where
        Q: BaseArrayPtr<T, L>,
    {
        BaseArray::<T, L, Q>::from_ptr(self.data.cast::<T, L, Q>())
    }

    /// Cast a reference to this array into a reference to a different array.
    ///
    /// Doesn't alter the length information of the array at all, or perform
    /// alignment/reference checks.
    pub unsafe fn cast_ref<T, Q>(&self) -> &BaseArray<T, L, Q>
    where
        Q: BaseArrayPtr<T, L>,
    {
        &*(self as *const BaseArray<E, L, P> as *const BaseArray<T, L, Q>)
    }

    /// Cast a mutable reference to this array into a mutable reference to a
    /// different array.
    ///
    /// Doesn't alter the length information of the array at all, or perform
    /// alignment/reference checks.
    pub unsafe fn cast_mut<T, Q>(&mut self) -> &mut BaseArray<T, L, Q>
    where
        Q: BaseArrayPtr<T, L>,
    {
        &mut *(self as *mut BaseArray<E, L, P> as *mut BaseArray<T, L, Q>)
    }

    /// Returns a pointer to the element at the index `idx`.
    ///
    /// # Safety
    /// Pointer is safe to dereference as long as the underlying array has a
    /// length greater than `idx`, and the element at `idx` has already been
    /// initialized.
    pub fn get_ptr(&self, idx: usize) -> *const E {
        self.data.elem_ptr(idx)
    }

    /// Returns a mutable pointer to the element at the index `idx`.
    ///
    /// # Safety
    /// Pointer is safe to dereference as long as the underlying array has a
    /// length greater than `idx`, and the element at `idx` has already been
    /// initialized.
    pub fn get_ptr_mut(&mut self, idx: usize) -> *mut E {
        self.data.elem_ptr(idx)
    }

    /// Returns whether or not the internal pointer in this array is null.
    pub fn is_null(&self) -> bool {
        self.data.is_null()
    }

    /// Returns a reference to the element at the index `idx`.
    ///
    /// # Safety
    /// Safe as long as the underlying array has a length greater than `idx`, and
    /// the element at `idx` has already been initialized.
    pub unsafe fn get(&self, idx: usize) -> &E {
        &*self.get_ptr(idx)
    }

    /// Returns a mutable reference to the element at the index `idx`.
    ///
    /// # Safety
    /// Safe as long as the underlying array has a length greater than `idx`, and
    /// the element at `idx` has already been initialized.
    pub unsafe fn get_mut(&mut self, idx: usize) -> &mut E {
        &mut *self.get_ptr_mut(idx)
    }

    /// Returns a reference to the label.
    pub fn get_label(&self) -> &L {
        unsafe { &*self.data.lbl_ptr() }
    }

    /// Returns a mutable reference to the label.
    pub fn get_label_mut(&mut self) -> &mut L {
        unsafe { &mut *self.data.lbl_ptr() }
    }

    /// Returns a reference to a slice into this array.
    ///
    /// The slice is from element 0 to `len - 1` inclusive.
    pub unsafe fn as_slice(&self, len: usize) -> &[E] {
        core::slice::from_raw_parts(self.get(0), len)
    }

    /// Returns a mutable reference to a slice into this array.
    ///
    /// The slice is from element 0 to `len - 1` inclusive.
    pub unsafe fn as_slice_mut(&mut self, len: usize) -> &mut [E] {
        core::slice::from_raw_parts_mut(self.get_mut(0), len)
    }

    /// Returns an iterator into this array, consuming the array in the process.
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
    P: BaseArrayPtr<E, L>,
{
    /// Clones the elements and label of this array into a new array of the same
    /// size.
    pub unsafe fn clone(&self, len: usize) -> Self {
        Self::new(self.get_label().clone(), len, |_, i| self.get(i).clone())
    }
}

impl<E, L, P> Iterator for BaseArrayIter<E, L, P>
where
    P: BaseArrayPtr<E, L>,
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
    P: BaseArrayPtr<E, L>,
{
    fn drop(&mut self) {
        let begin = self.array.get_ptr_mut(0) as usize;
        let len = ((self.end as usize) - begin) / mem::size_of::<E>();
        unsafe { self.array.drop(len) }
    }
}
