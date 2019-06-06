use crate::prelude::*;
use core::ptr::NonNull;

#[repr(transparent)]
pub struct Array<E, L> {
    data: NonNull<MemBlock<E, L>>,
}

pub struct ArrayIter<E, L> {
    array: Array<E, L>,
    current: *mut E,
    end: *mut E,
}

impl<E, L> Array<E, L> {
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
        let data = unsafe { NonNull::new_unchecked(MemBlock::<E, L>::new_init(label, len, func)) };
        Self { data }
    }

    pub unsafe fn new_lazy(label: L, len: usize) -> Self {
        let data = NonNull::new(MemBlock::<E, L>::new(label, len)).unwrap();
        Self { data }
    }

    pub unsafe fn drop(&mut self, len: usize) {
        self._mut().dealloc(len)
    }

    pub unsafe fn drop_lazy(&mut self, len: usize) {
        self._mut().dealloc_lazy(len)
    }

    pub fn get_ptr(&self, idx: usize) -> *const E {
        self._ref().get_ptr(idx)
    }

    pub fn get_ptr_mut(&mut self, idx: usize) -> *mut E {
        self._mut().get_ptr(idx) as *mut E
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

    pub unsafe fn into_iter(mut self, len: usize) -> ArrayIter<E, L> {
        let current = self.get_mut(0) as *mut E;
        let end = current.add(len);
        ArrayIter {
            array: self,
            current,
            end,
        }
    }
}

impl<E, L> Array<E, L>
where
    E: Clone,
    L: Clone,
{
    pub unsafe fn clone(&self, len: usize) -> Self {
        Self::new(self.get_label().clone(), len, |_, i| self.get(i).clone())
    }
}

impl<E, L> Iterator for ArrayIter<E, L> {
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

impl<E, L> Drop for ArrayIter<E, L> {
    fn drop(&mut self) {
        let begin = self.array.get_ptr(0) as usize;
        let len = ((self.end as usize) - begin) / mem::size_of::<E>();
        unsafe { self.array.drop(len) }
    }
}
