use super::alloc_utils::*;
use core::mem;
use core::mem::ManuallyDrop;

/// The value of null. Nullified pointers to memory blocks are overwritten
/// with this value.
const NULL: usize = core::usize::MAX;

// TODO Make this function a const function when if statements are stabilized in
// const functions
/// Get the maximum length of a memory block, based on the types that it contains.
/// Maintains the invariants discussed above.
pub fn block_max_len<E, L>() -> usize {
    use core::usize::MAX as MAX_LEN;
    let elem_size = mem::size_of::<E>();
    let label_size = mem::size_of::<L>();
    if elem_size == 0 {
        MAX_LEN - 1
    } else {
        (MAX_LEN - label_size) / elem_size - 1
    }
}

#[cfg(test)]
pub fn check_null_ref<E, L>(arr: &MemBlock<E, L>, message: &'static str) {
    assert!(arr as *const _ as usize != NULL, message);
}

/// An array block that can hold arbitrary information, and cannot be
/// constructed on the stack. The `E` type can be repeated an arbitrary
/// number of times, and the `L` type can be repeated exactly once.
#[repr(C)]
pub struct MemBlock<E, L> {
    /// Metadata about the block
    pub label: L,
    /// First element in the block
    elements: ManuallyDrop<E>,
}

impl<E, L> MemBlock<E, L> {
    /// Get size and alignment of the memory that this memory block uses.
    pub fn memory_layout(len: usize) -> (usize, usize) {
        let l_layout = size_align::<Self>();
        if len <= 1 {
            l_layout
        } else {
            let d_layout = size_align_array::<E>(len - 1);
            size_align_multiple(&[l_layout, size_align::<usize>(), d_layout])
        }
    }

    /// Deallocates a reference to this struct, as well as all objects contained
    /// in it.
    pub unsafe fn dealloc<'a>(&'a mut self, len: usize) {
        #[cfg(test)]
        check_null_ref(self, "MemBlock::dealloc: Deallocating null pointer!");
        for i in 0..len {
            let val = mem::transmute_copy(self.get(i));
            mem::drop::<E>(val);
        }
        self.dealloc_lazy(len);
    }

    pub unsafe fn dealloc_lazy<'a>(&'a mut self, len: usize) {
        #[cfg(test)]
        check_null_ref(self, "MemBlock::dealloc: Deallocating null pointer!");

        let (size, align) = Self::memory_layout(len);
        deallocate(self, size, align);
    }

    /// Returns a null pointer to a memory block. Dereferencing it is undefined
    /// behavior.
    pub unsafe fn null_ptr() -> *mut Self {
        NULL as *mut Self
    }

    pub unsafe fn new<'a>(label: L, len: usize) -> &'a mut Self {
        #[cfg(not(feature = "no-asserts"))]
        assert!(len <= block_max_len::<E, L>());
        let (size, align) = Self::memory_layout(len);
        let new_ptr = allocate::<Self>(size, align);
        new_ptr.label = label;
        new_ptr
    }

    pub unsafe fn new_init<'a, F>(label: L, len: usize, mut func: F) -> &'a mut Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let new_ptr = Self::new(label, len);
        for i in 0..len {
            let item = func(&mut new_ptr.label, i);
            let garbage = mem::replace(new_ptr.get(i), item);
            mem::forget(garbage);
        }
        new_ptr
    }

    pub unsafe fn get<'a>(&'a self, idx: usize) -> &'a mut E {
        #[cfg(test)]
        check_null_ref(self, "MemBlock::get: Indexing on null pointer!");
        let element = &*self.elements as *const E as *mut E;
        let element = element.add(idx);
        &mut *element
    }

    /// Generates an iterator from this memory block, which is inclusive
    /// on the `start` index and exclusive on the `end` index.
    /// The iterator operates on the preconditions that:
    ///
    /// - The reference `self` is not `NULL`. This is NOT checked at
    ///   runtime.
    /// - The operation of dereferencing an element at index `i`, where
    ///   `0 <= i < len`, accesses valid memory that has been
    ///   properly initialized. This is NOT checked at runtime.
    ///
    /// This function is unsafe because not meeting any of the above
    /// conditions results in undefined behavior. Additionally, the
    /// iterator that's created can potentially take ownership, and
    /// it's your job to prove that doing so is a valid operation.
    pub unsafe fn iter<'a>(&'a self, len: usize) -> MemBlockIter<E, L> {
        #[cfg(test)]
        check_null_ref(self, "MemBlock::iter: Getting iterator on null pointer!");
        MemBlockIter {
            block: &mut *(self as *const Self as *mut Self),
            end: self.get(len),
            current: self.get(0),
        }
    }
}

impl<'a, E, L> crate::traits::BaseArrayRef for &'a mut MemBlock<E, L> {
    fn is_null(&self) -> bool {
        self as *const Self as usize == NULL
    }
}

pub struct MemBlockIter<'a, E, L> {
    block: &'a mut MemBlock<E, L>,
    end: &'a mut E,
    current: &'a mut E,
}

impl<'a, E, L> MemBlockIter<'a, E, L> {
    pub fn to_owned(self) -> MbIterOwned<'a, E, L> {
        MbIterOwned { iter: self }
    }
    pub fn to_ref(self) -> MbIterRef<'a, E, L> {
        MbIterRef { iter: self }
    }
    pub fn to_mut(self) -> MbIterRefMut<'a, E, L> {
        MbIterRefMut { iter: self }
    }
}

#[repr(transparent)]
pub struct MbIterOwned<'a, E, L> {
    iter: MemBlockIter<'a, E, L>,
}

impl<'a, E, L> Iterator for MbIterOwned<'a, E, L> {
    type Item = E;
    fn next(&mut self) -> Option<E> {
        let curr = self.iter.current as *mut E;
        let end = self.iter.end as *mut E;
        if curr == end {
            let begin = &mut *self.iter.block.elements as *mut E as usize;
            let len = (end as usize) - begin;
            unsafe { self.iter.block.dealloc_lazy(len) };
            None
        } else {
            unsafe {
                let out = mem::transmute_copy(self.iter.current);
                self.iter.current = &mut *curr.add(1);
                Some(out)
            }
        }
    }
}

#[repr(transparent)]
pub struct MbIterRef<'a, E, L> {
    iter: MemBlockIter<'a, E, L>,
}

impl<'a, E, L> Iterator for MbIterRef<'a, E, L> {
    type Item = &'a E;
    fn next(&mut self) -> Option<&'a E> {
        let curr = self.iter.current as *mut E;
        let end = self.iter.end as *mut E;
        if curr == end {
            None
        } else {
            unsafe {
                let out = self.iter.current as *mut E;
                self.iter.current = &mut *curr.add(1);
                Some(&*out)
            }
        }
    }
}

#[repr(transparent)]
pub struct MbIterRefMut<'a, E, L> {
    iter: MemBlockIter<'a, E, L>,
}

impl<'a, E, L> Iterator for MbIterRefMut<'a, E, L> {
    type Item = &'a mut E;
    fn next(&mut self) -> Option<&'a mut E> {
        let curr = self.iter.current as *mut E;
        let end = self.iter.end as *mut E;
        if curr == end {
            None
        } else {
            unsafe {
                let out = self.iter.current as *mut E;
                self.iter.current = &mut *curr.add(1);
                Some(&mut *out)
            }
        }
    }
}
