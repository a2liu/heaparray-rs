use crate::prelude::*;
use core::ptr::NonNull;

#[repr(transparent)]
pub struct BaseArray<E, L> {
    data: NonNull<MemBlock<E, L>>,
}

pub struct BaseArrayIter<E, L> {
    array: BaseArray<E, L>,
    current: *mut E,
    end: *mut E,
}

impl<E, L> BaseArray<E, L> {
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
        let data = NonNull::new(MemBlock::new_init(label, len, func)).unwrap();
        Self { data }
    }

    pub unsafe fn new_lazy(label: L, len: usize) -> Self {
        Self::from_ptr(MemBlock::new(label, len))
    }

    pub unsafe fn from_ptr(ptr: *mut MemBlock<E, L>) -> Self {
        Self {
            data: NonNull::new_unchecked(ptr),
        }
    }

    pub unsafe fn as_ptr(&self) -> *const MemBlock<E, L> {
        self.data.as_ptr()
    }

    pub unsafe fn as_ptr_mut(&mut self) -> *mut MemBlock<E, L> {
        self.data.as_ptr()
    }

    pub unsafe fn drop(&mut self, len: usize) {
        self._mut().dealloc(len)
    }

    pub unsafe fn drop_lazy(&mut self, len: usize) {
        self._mut().dealloc_lazy(len)
    }

    pub unsafe fn cast_into<T>(self) -> BaseArray<T, L> {
        let ptr = self.data.cast::<MemBlock<T, L>>();
        BaseArray { data: ptr }
    }

    pub unsafe fn cast_ref<T>(&self) -> &BaseArray<T, L> {
        &*(self as *const BaseArray<E, L> as *const BaseArray<T, L>)
    }

    pub unsafe fn cast_mut<T>(&mut self) -> &mut BaseArray<T, L> {
        &mut *(self as *mut BaseArray<E, L> as *mut BaseArray<T, L>)
    }

    pub fn get_ptr(&self, idx: usize) -> *const E {
        self._ref().get_ptr(idx)
    }

    pub fn get_ptr_mut(&mut self, idx: usize) -> *mut E {
        self._mut().get_ptr(idx)
    }

    pub unsafe fn get(&self, idx: usize) -> &E {
        &*self.get_ptr(idx)
    }

    pub unsafe fn get_mut(&mut self, idx: usize) -> &mut E {
        &mut *self.get_ptr_mut(idx)
    }

    pub fn get_label(&self) -> &L {
        &self._ref().label
    }

    pub fn get_label_mut(&mut self) -> &mut L {
        &mut self._mut().label
    }

    pub unsafe fn as_slice(&self, len: usize) -> &[E] {
        core::slice::from_raw_parts(self.get(0), len)
    }

    pub unsafe fn as_slice_mut(&mut self, len: usize) -> &mut [E] {
        core::slice::from_raw_parts_mut(self.get_mut(0), len)
    }

    pub unsafe fn into_iter(mut self, len: usize) -> BaseArrayIter<E, L> {
        let current = self.get_mut(0) as *mut E;
        let end = current.add(len);
        BaseArrayIter {
            array: self,
            current,
            end,
        }
    }
}

impl<E, L> BaseArray<E, L>
where
    E: Clone,
    L: Clone,
{
    pub unsafe fn clone(&self, len: usize) -> Self {
        Self::new(self.get_label().clone(), len, |_, i| self.get(i).clone())
    }
}

impl<E, L> Iterator for BaseArrayIter<E, L> {
    type Item = E;
    fn next(&mut self) -> Option<E> {
        if self.current == self.end {
            None
        } else {
            unsafe {
                let out = ptr::read(self.current);
                self.current = self.current.add(1);
                Some(out)
            }
        }
    }
}

impl<E, L> Drop for BaseArrayIter<E, L> {
    fn drop(&mut self) {
        let begin = self.array.get_ptr(0) as usize;
        let len = ((self.end as usize) - begin) / mem::size_of::<E>();
        unsafe { self.array.drop(len) }
    }
}
