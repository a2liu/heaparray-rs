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
    pub label: ManuallyDrop<L>,
    /// First element in the block
    elements: ManuallyDrop<E>,
}

impl<E, L> MemBlock<E, L> {
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
    pub unsafe fn dealloc(&mut self, len: usize) {
        ptr::drop_in_place(&mut *self.label);
        for i in 0..len {
            ptr::drop_in_place(self.get(i));
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
    pub unsafe fn dealloc_lazy(&mut self, len: usize) {
        let (size, align) = Self::memory_layout(len);
        deallocate(self, size, align);
    }

    /// Returns a pointer to a new memory block on the heap with an
    /// initialized label. Does not initialize memory, so use with care.
    ///
    /// If you use this function, remember to prevent the compiler from
    /// running the destructor for the memory that wasn't initialized. i.e.
    /// something like this:
    ///
    /// ```rust
    /// use heaparray::mem_block::MemBlock;
    /// use core::mem;
    /// let len = 100;
    /// let block = unsafe { &mut *MemBlock::<usize, ()>::new((), len) };
    /// for i in 0..len {
    ///     let item = i * i;
    ///     let garbage = mem::replace(unsafe { block.get(i) }, item);
    ///     mem::forget(garbage);
    /// }
    /// ```
    pub unsafe fn new<'a>(label: L, len: usize) -> *mut Self {
        #[cfg(not(feature = "no-asserts"))]
        assert!(len <= block_max_len::<E, L>());

        let (size, align) = Self::memory_layout(len);
        let new_ptr = allocate::<Self>(size, align);
        ptr::write(&mut (&mut *new_ptr).label, ManuallyDrop::new(label));
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
    pub fn new_init<F>(label: L, len: usize, mut func: F) -> *mut Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let new_ptr = unsafe { &mut *Self::new(label, len) };
        for i in 0..len {
            let item = func(&mut new_ptr.label, i);
            let garbage = mem::replace(unsafe { new_ptr.get(i) }, item);
            mem::forget(garbage);
        }
        new_ptr
    }

    pub fn get_ptr(&self, idx: usize) -> *const E {
        let element = (&*self.elements) as *const E;
        unsafe { element.add(idx) }
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
        let element = &*self.elements as *const E as *mut E;
        let element = element.add(idx);
        &mut *element
    }

    /// Get the label associated with this memory block.
    pub unsafe fn get_label<'a>(&'a self) -> &'a mut L {
        &mut *(&*self.label as *const L as *mut L)
    }

    /// Generates an iterator from this memory block, which is inclusive
    /// on the `start` index and exclusive on the `end` index.
    /// The iterator operates on the preconditions that:
    ///
    /// - The operation of dereferencing an element at index `i`, where
    ///   `0 <= i < len`, accesses valid memory that has been
    ///   properly initialized. This is NOT checked at runtime.
    ///
    /// This function is unsafe because not meeting any of the above
    /// conditions results in undefined behavior. Additionally, the
    /// iterator that's created can potentially take ownership, and
    /// it's your job to prove that doing so is a valid operation.
    pub unsafe fn iter(&self, len: usize) -> MemBlockIter<E, L> {
        let beginning = &*self.elements as *const E as *mut E;
        MemBlockIter {
            block: self as *const Self as *mut Self,
            current: beginning,
            end: beginning.add(len),
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
        &mut *Self::new_init((*self.label).clone(), len, |_, i| self.get(i).clone())
    }
}

/// A struct that allows for iteration over a `MemBlock`. Uses safe
/// functions because its construction is unsafe, so once you've
/// constructed it you purportedly know what you're doing.
pub struct MemBlockIter<E, L> {
    block: *mut MemBlock<E, L>,
    current: *mut E,
    end: *mut E,
}

impl<E, L> Iterator for MemBlockIter<E, L> {
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

impl<E, L> Drop for MemBlockIter<E, L> {
    fn drop(&mut self) {
        unsafe {
            let block_ref = &mut *self.block;
            ptr::drop_in_place(&mut *block_ref.label);
            let len = ((self.end as usize) - (&*block_ref.elements as *const E as usize))
                / mem::size_of::<E>();
            (&mut *self.block).dealloc_lazy(len);
        }
    }
}
