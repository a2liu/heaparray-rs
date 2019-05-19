//! Contains the struct `MemBlock`, which handles pointer math and very low-level
//! interactions with memory.
use super::alloc_utils::*;
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr;

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
    assert!(!arr.is_null(), message);
}

/// An array block that can hold arbitrary information, and cannot be
/// constructed on the stack. The `E` type can be repeated an arbitrary
/// number of times, and the `L` type can be repeated exactly once.
///
/// It's not recommended to use this directly; instead, use the pointer
/// types that refer to these, namely `HeapArray`, `FatPtrArray`, and
/// `ThinPtrArray`.
///
/// # Invariants
/// The code for this struct does little to no runtime checks for validity;
/// thus, you need to do them yourself. Length checks, runtime validity
/// checks, etc. all need to be done at runtime before using the methods
/// associated with this type, as forgetting to do so leads to undefined
/// behavior (likely a segmentation fault or an invalid pointer
/// dereference). To avoid these errors, you can instead use the safe API
/// provided by the `ThinPtrArray` and `FatPtrArray` structs, or by the
/// `HeapArray` struct, which internally is implemented as a `FatPtrArray`.
#[repr(C)]
pub struct MemBlock<E, L = ()> {
    /// Metadata about the block
    pub label: L,
    /// First element in the block
    elements: ManuallyDrop<E>,
}

impl<E, L> MemBlock<E, L> {
    /// The value of null. Nullified pointers to memory blocks are overwritten
    /// with this value.
    pub const NULL: usize = core::usize::MAX;

    /// Get size and alignment of the memory that a block of length `len`
    /// would need.
    pub fn memory_layout(len: usize) -> (usize, usize) {
        let l_layout = size_align::<Self>();
        if len <= 1 {
            l_layout
        } else {
            let d_layout = size_align_array::<E>(len - 1);
            size_align_multiple(&[l_layout, size_align::<usize>(), d_layout])
        }
    }

    /// Deallocates a reference to this struct, as well as all objects
    /// contained in it. This function is safe given that the following
    /// preconditions hold:
    ///
    /// - The reference `&self` is valid, and points to a valid instance
    ///   of this type
    /// - The memory block pointed to by `&self` was initialized with
    ///   length of `len`
    /// - The operation of dereferencing an element at index `i`, where
    ///   `0 <= i < len`, accesses valid memory that has been
    ///   properly initialized. This is NOT checked at runtime.
    pub unsafe fn dealloc<'a>(&'a mut self, len: usize) {
        #[cfg(test)]
        check_null_ref(self, "MemBlock::dealloc: Deallocating null pointer!");
        let lbl = mem::transmute_copy::<L, L>(&self.label);
        mem::drop(lbl);
        for i in 0..len {
            let val = mem::transmute_copy(self.get(i));
            mem::drop::<E>(val);
        }
        self.dealloc_lazy(len);
    }

    /// Deallocates a reference to this struct, without running the
    /// destructor of the elements it contains. This function is safe
    /// given that the following preconditions hold:
    ///
    /// - The reference `&self` is valid, and points to a valid instance
    ///   of this type
    /// - The memory block pointed to by `&self` was initialized with length
    ///   of `len`
    ///
    /// This function *may* leak memory; be sure to run destructors on
    /// all initialized elements in this block before calling this method,
    /// as they may in accessible afterwards if you don't.
    pub unsafe fn dealloc_lazy<'a>(&'a mut self, len: usize) {
        #[cfg(test)]
        check_null_ref(self, "MemBlock::dealloc: Deallocating null pointer!");

        let (size, align) = Self::memory_layout(len);
        deallocate(self, size, align);
    }

    /// Returns a null pointer to a memory block. Dereferencing it is undefined
    /// behavior, and is by definition unsafe.
    pub unsafe fn null_ref() -> *mut Self {
        Self::NULL as *mut Self
    }

    /// Returns a pointer to a new memory block on the heap with an
    /// initialized label. Does not initialize memory, so use with care.
    ///
    /// If you use this function, remember to prevent the compiler from
    /// running the destructor for the memory wasn't initialized. i.e.
    /// something like this:
    ///
    /// ```rust
    /// use heaparray::mem_block::MemBlock;
    /// use core::mem;
    /// let len = 100;
    /// let block = unsafe { MemBlock::<usize, ()>::new((), len) };
    /// for i in 0..len {
    ///     let item = i * i;
    ///     let garbage = mem::replace(unsafe { block.get(i) }, item);
    ///     mem::forget(garbage);
    /// }
    /// ```
    pub unsafe fn new<'a>(label: L, len: usize) -> &'a mut Self {
        #[cfg(not(feature = "no-asserts"))]
        assert!(len <= block_max_len::<E, L>());
        let (size, align) = Self::memory_layout(len);
        let new_ptr = allocate::<Self>(size, align);
        let garbage = mem::replace(&mut new_ptr.label, label);
        mem::forget(garbage);
        new_ptr
    }

    /// Returns a pointer to a labelled memory block, with elements initialized
    /// using the provided function. Function is safe, because the following
    /// invariants will always hold:
    ///
    /// - A memory access `block.get(i)` where `0 <= i < len` will always
    ///   be valid.
    /// - A memory access `block.label` will always be valid.
    ///
    /// However, note that *deallocating* the resulting block can *never*
    /// be safe; there is not guarrantee provided by the type system
    /// that the block you deallocate will have the length that you assume
    /// it has.
    pub fn new_init<'a, F>(label: L, len: usize, mut func: F) -> &'a mut Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let new_ptr = unsafe { Self::new(label, len) };
        for i in 0..len {
            let item = func(&mut new_ptr.label, i);
            let garbage = mem::replace(unsafe { new_ptr.get(i) }, item);
            mem::forget(garbage);
        }
        new_ptr
    }

    /// Gets a mutable reference to the element at the index `idx` in this
    /// memory block. This function will return a valid reference given
    /// that the following preconditions hold:
    ///
    /// - The reference `&self` is valid, and points to
    ///   a valid instance of this type
    /// - The memory block pointed to by `&self` was initialized with a
    ///   `len` of at least `idx + 1`
    /// - The element at index `idx` was initialized; this can be acheived
    ///   by calling `MemBlock::new_init(label, len, || { /* closure */ })`.
    ///   or by initializing the values yourself.
    pub unsafe fn get<'a>(&'a self, idx: usize) -> &'a mut E {
        #[cfg(test)]
        check_null_ref(self, "MemBlock::get: Indexing on null pointer!");
        let element = &*self.elements as *const E as *mut E;
        let element = element.add(idx);
        &mut *element
    }

    pub unsafe fn get_label<'a>(&'a self) -> &'a mut L {
        #[cfg(test)]
        check_null_ref(self, "MemBlock::get_label: Indexing on null pointer!");
        &mut *(&self.label as *const L as *mut L)
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
    pub unsafe fn iter<'a>(&'a self, len: usize) -> MemBlockIter<'a, E, L> {
        #[cfg(test)]
        check_null_ref(self, "MemBlock::iter: Getting iterator on null pointer!");
        MemBlockIter {
            block: &mut *(self as *const Self as *mut Self),
            end: &mut *(self.get(len) as *mut E),
            current: &mut *(self.get(0) as *mut E),
        }
    }

    /// Generates a slice into this memory block. The following invariants
    /// must hold:
    ///
    /// - Mutual exclusivity of mutable slices; for every element in the
    ///   block, there can be at most one mutable reference to it at
    ///   any given time.
    /// - `start` must be less than or equal to `end`. This is checked
    ///   at runtime unless this crate is build with the feature
    ///   `no-asserts`.
    /// - The operation of dereferencing an element at index `i`, where
    ///   `start <= i < end`, accesses valid memory that has been
    ///   properly initialized. This is NOT checked at runtime.
    pub unsafe fn get_slice<'a>(&'a self, start: usize, end: usize) -> &'a mut [E] {
        #[cfg(test)]
        check_null_ref(
            self,
            "MemBlock::get_slice: Getting slice into null pointer!",
        );
        #[cfg(not(feature = "no-asserts"))]
        assert!(start <= end);
        core::slice::from_raw_parts_mut(self.get(start) as *mut E, end - start)
    }

    /// Generates a slice into this memory block. The following invariants
    /// must hold:
    ///
    /// - The operation of dereferencing an element at index `i`, where
    ///   `0 <= i < len`, accesses valid memory that has been
    ///   properly initialized. This is NOT checked at runtime.
    pub unsafe fn as_slice<'a>(&'a self, len: usize) -> &'a mut [E] {
        self.get_slice(0, len)
    }

    /// Since this struct isn't a reference to contigous memory, but rather
    /// the contiguous memory itself, it doesn't not implement the trait
    /// `heaparray::BaseArrayRef`. However, it provides the same
    /// functionality through this method.
    pub fn is_null(&self) -> bool {
        use crate::black_box::black_box;
        let ret = unsafe {
            // Rust does some funky optimizations behind the scenes that
            // in most cases would be useful, but because we don't
            // maintain the typical invariants of Rust references in this
            // codebase, we need to wrap some of the values in black
            // boxes.
            //
            // I don't know whether it's better to use this system or
            // to use the provided function, `core::ptr::null()`. My
            // current rationale is that `usize::MAX` is a reasonable
            // value for "never going to be seen in the wild so if we
            // do it's an error". But I could definitely change it very
            // easily, and I'm very willing to.
            ptr::eq(
                black_box(self) as *const Self,
                black_box(black_box(Self::NULL) as *const Self),
            )
        };
        ret
    }
}

impl<E, L> MemBlock<E, L>
where
    E: Clone,
    L: Clone,
{
    /// Clones all the elements in this memory block, as well as the
    /// label. This function is safe given that the following preconditions
    /// are true:
    ///
    /// - The reference `&self` is valid, and points to a valid instance
    ///   of this type
    /// - The memory block pointed to by `&self` was initialized with length
    ///   of `len`
    /// - The element at index `i` was initialized, where `0 <= i < len`,
    ///   for all `i`
    /// - The operation of dereferencing an element at index `i`, where
    ///   `0 <= i < len`, accesses valid memory that has been
    ///   properly initialized. This is NOT checked at runtime.
    pub unsafe fn clone<'a, 'b>(&'a self, len: usize) -> &'b mut Self {
        Self::new_init(self.label.clone(), len, |_, i| self.get(i).clone())
    }
}

/// A struct that allows for iteration over a `MemBlock`. Uses safe
/// functions because its construction is unsafe, so once you've
/// constructed it you purportedly know what you're doing.
pub struct MemBlockIter<'a, E, L> {
    block: &'a mut MemBlock<E, L>,
    end: &'a mut E,
    current: &'a mut E,
}

impl<'a, E, L> MemBlockIter<'a, E, L> {
    /// Creates an owned version of this iterator that takes ownership
    /// of the memory block it has a reference to and deallocates it
    /// when done iterating.
    pub fn to_owned(self) -> MemBlockIterOwned<'a, E, L> {
        MemBlockIterOwned { iter: self }
    }
    /// Creates a borrow version of this iterator that returns references
    /// to the stuff inside of it.
    pub fn to_ref(self) -> MemBlockIterRef<'a, E, L> {
        MemBlockIterRef { iter: self }
    }
    /// Creates a mutable borrow version of this iterator that returns
    /// mutable references to the stuff inside of it.
    pub fn to_mut(self) -> MemBlockIterMut<'a, E, L> {
        MemBlockIterMut { iter: self }
    }
}

/// Owned version of `MemBlockIter` that returns the items in the memory
/// block by value and then deallocates the block once this iterator
/// goes out of scope.
#[repr(transparent)]
pub struct MemBlockIterOwned<'a, E, L> {
    iter: MemBlockIter<'a, E, L>,
}

impl<'a, E, L> Iterator for MemBlockIterOwned<'a, E, L> {
    type Item = E;
    fn next(&mut self) -> Option<E> {
        let curr = self.iter.current as *mut E;
        let end = self.iter.end as *mut E;
        if curr == end {
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

impl<'a, E, L> Drop for MemBlockIterOwned<'a, E, L> {
    fn drop(&mut self) {
        let end = self.iter.end as *mut E;
        let begin = &mut *self.iter.block.elements as *mut E as usize;
        let len = (end as usize) - begin;
        unsafe { self.iter.block.dealloc_lazy(len) };
    }
}

/// Borrow version of `MemBlockIter` that returns references to
/// the elements in the block.
#[repr(transparent)]
pub struct MemBlockIterRef<'a, E, L> {
    iter: MemBlockIter<'a, E, L>,
}

impl<'a, E, L> Iterator for MemBlockIterRef<'a, E, L> {
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

/// Mutable borrow version of `MemBlockIter` that returns mutable
/// references to the elements in the block.
#[repr(transparent)]
pub struct MemBlockIterMut<'a, E, L> {
    iter: MemBlockIter<'a, E, L>,
}

impl<'a, E, L> Iterator for MemBlockIterMut<'a, E, L> {
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
