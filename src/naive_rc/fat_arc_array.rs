//! Contains definition for `FpArcArray`, which is a fat pointer to an atomically reference-counted
//! array, and the implementation for `heaparray::naive_rc::ArcArray`.
pub use crate::naive_rc::prelude::*;

type PtrType<'a, E, L> = FatPtrArray<'a, E, ArcStruct<L>>;
type DataType<'a, E, L> = ManuallyDrop<PtrType<'a, E, L>>;

/// A fat-pointer to an atomically reference-counted array. Has the same API as the `HeapArray`, but implements `Send` for all types.
///
/// Additionally implements the marker trait
/// `ArrayRef`, so you can clone references to existing arrays:
///
/// ```rust
/// use heaparray::naive_rc::fat_arc_array::*;
/// let array_ref = FpArcArray::<i32, (&str, &str)>::new_labelled(("hello", "world"), 10, |_,_| 0);
/// let array_ref_2 = ArrayRef::clone(&array_ref);
///
/// assert!(array_ref.len() == array_ref_2.len());
/// for i in 0..array_ref_2.len() {
///     let r1 = &array_ref[i] as *const i32;
///     let r2 = &array_ref_2[i] as *const i32;
///     assert!(r1 == r2);
/// }
/// ```
pub struct FpArcArray<'a, E, L = ()> {
    data: DataType<'a, E, L>,
}

impl<'a, E> FpArcArray<'a, E> {
    /// Create a new reference-counted array, with values initialized using a provided function.
    #[inline]
    pub fn new<F>(len: usize, mut func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self::new_labelled((), len, |_, idx| func(idx))
    }
}

impl<'a, E, L> FpArcArray<'a, E, L> {
    /// Create a new reference-counted array, with values initialized using a provided function, and label
    /// initialized to a provided value.
    #[inline]
    pub fn new_labelled<F>(label: L, len: usize, mut func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let new_ptr = PtrType::new_labelled(ArcStruct::new(label), len, |rc_struct, idx| {
            func(&mut rc_struct.data, idx)
        });
        Self {
            data: ManuallyDrop::new(new_ptr),
        }
    }

    /// Create a new reference-counted array, without initializing the values in it.
    #[inline]
    pub unsafe fn new_labelled_unsafe(label: L, len: usize) -> Self {
        let new_ptr = PtrType::new_labelled_unsafe(ArcStruct::new(label), len);

        Self {
            data: ManuallyDrop::new(new_ptr),
        }
    }

    /// Unsafe access to an element at an index in the array. Does NOT check
    /// for null reference first.
    #[inline]
    pub unsafe fn unchecked_access(&'a self, idx: usize) -> &'a mut E {
        self.data.unchecked_access(idx)
    }

    /// Make this a null reference. This is safe, because the API for this
    /// object doesn't assume that the reference is valid before dereferencing it.
    #[inline]
    pub fn to_null(&mut self) {
        if self.is_null() {
            return;
        }

        let ref_count = self.data.get_label_mut().decrement();
        if ref_count == 0 {
            let to_drop = mem::replace(&mut *self.data, unsafe { PtrType::null_ref() });
            mem::drop(to_drop);
        }
    }

    /// Checks whether the array reference contained by this struct is
    /// valid.
    #[inline]
    pub fn is_null(&'a self) -> bool {
        self.data.is_null()
    }

    /// Creates a new null pointer reference.
    #[inline]
    pub fn null_ref() -> Self {
        Self {
            data: ManuallyDrop::new(unsafe { PtrType::null_ref() }),
        }
    }
}

impl<'a, E> FpArcArray<'a, E>
where
    E: Default,
{
    /// Get a new reference-counted array, initialized to default values.
    #[inline]
    pub fn new_default(len: usize) -> Self {
        Self::new_default_labelled((), len)
    }
}

impl<'a, E, L> FpArcArray<'a, E, L>
where
    E: Default,
{
    /// Get a new reference-counted array, initialized to default values.
    #[inline]
    pub fn new_default_labelled(label: L, len: usize) -> Self {
        Self {
            data: ManuallyDrop::new(FatPtrArray::new_default_labelled(
                ArcStruct::new(label),
                len,
            )),
        }
    }
}

unsafe impl<'a, E, L> Send for FpArcArray<'a, E, L>
where
    E: Send + Sync,
    L: Send + Sync,
{
}

unsafe impl<'a, E, L> Sync for FpArcArray<'a, E, L>
where
    E: Send + Sync,
    L: Send + Sync,
{
}

impl<'a, E, L> Index<usize> for FpArcArray<'a, E, L> {
    type Output = E;
    #[inline]
    fn index(&self, idx: usize) -> &E {
        if self.is_null() {
            panic!("Null dereference of heaparray::FpArcArray");
        }
        &self.data[idx]
    }
}

impl<'a, E, L> IndexMut<usize> for FpArcArray<'a, E, L> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut E {
        if self.is_null() {
            panic!("Null dereference of heaparray::FpArcArray");
        }
        &mut self.data[idx]
    }
}

impl<'a, E, L> Clone for FpArcArray<'a, E, L> {
    #[inline]
    fn clone(&self) -> Self {
        if self.is_null() {
            Self::null_ref()
        } else {
            (*self.data).get_label().increment();
            unsafe { mem::transmute_copy(self) }
        }
    }
}

impl<'a, E, L> Drop for FpArcArray<'a, E, L> {
    fn drop(&mut self) {
        self.to_null();
    }
}

impl<'a, E, L> Container<(usize, E)> for FpArcArray<'a, E, L> {
    #[inline]
    fn add(&mut self, elem: (usize, E)) {
        if self.is_null() {
            panic!("Null dereference of heaparray::FpArcArray");
        }
        self[elem.0] = elem.1;
    }
    #[inline]
    fn len(&self) -> usize {
        if self.is_null() {
            panic!("Null dereference of heaparray::FpArcArray");
        }
        self.data.len()
    }
}

impl<'a, E, L> CopyMap<'a, usize, E> for FpArcArray<'a, E, L>
where
    E: 'a,
{
    #[inline]
    fn get(&'a self, key: usize) -> Option<&'a E> {
        if self.is_null() {
            panic!("Null dereference of heaparray::FpArcArray");
        } else if key > self.len() {
            None
        } else {
            Some(&self[key])
        }
    }
    #[inline]
    fn get_mut(&'a mut self, key: usize) -> Option<&'a mut E> {
        if self.is_null() {
            panic!("Null dereference of heaparray::FpArcArray");
        } else if key > self.len() {
            None
        } else {
            Some(&mut self[key])
        }
    }
    #[inline]
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        if self.is_null() {
            panic!("Null dereference of heaparray::FpArcArray");
        } else if key > self.len() {
            None
        } else {
            Some(mem::replace(&mut self[key], value))
        }
    }
}

impl<'a, E, L> Array<'a, E> for FpArcArray<'a, E, L> where E: 'a {}

impl<'a, E, L> LabelledArray<'a, E, L> for FpArcArray<'a, E, L>
where
    E: 'a,
{
    /// Get a reference to the label of the array.
    #[inline]
    fn get_label(&self) -> &L {
        if self.is_null() {
            panic!("Null dereference of heaparray::FpArcArray");
        }
        &self.data.get_label().data
    }

    /// Get a mutable reference to the label of the array.
    #[inline]
    fn get_label_mut(&mut self) -> &mut L {
        if self.is_null() {
            panic!("Null dereference of heaparray::FpArcArray");
        }
        &mut self.data.get_label_mut().data
    }
}

impl<'a, E, L> ArrayRef for FpArcArray<'a, E, L> {}
