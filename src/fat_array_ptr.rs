use crate::prelude::*;

pub struct FatPtrArray<'a, E, L = ()>
where
    Self: 'a,
{
    data: &'a mut FPArrayBlock<E, L>,
}

impl<'a, E> FatPtrArray<'a, E> {
    /// Create a new array, with values initialized using a provided function.
    #[inline]
    pub fn new<F>(len: usize, mut func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self::new_labelled((), len, |_, idx| func(idx))
    }
}

impl<'a, E, L> FatPtrArray<'a, E, L> {
    /// Create a new array, with values initialized using a provided function, and label
    /// initialized to a provided value.
    #[inline]
    pub fn new_labelled<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        Self {
            data: FPArrayBlock::<E, L>::new_ptr(label, len, func),
        }
    }

    /// Create a new array, without initializing the values in it.
    #[inline]
    pub unsafe fn new_labelled_unsafe(label: L, len: usize) -> Self {
        let new_ptr = FPArrayBlock::<E, L>::new_ptr_unsafe(label, len);
        Self { data: new_ptr }
    }

    /// Unsafe access to an element at an index in the array.
    #[inline]
    pub unsafe fn unchecked_access(&'a self, idx: usize) -> &'a mut E {
        self.data.unchecked_access(idx)
    }
}

impl<'a, E> FatPtrArray<'a, E>
where
    E: Default,
{
    /// Get a new array, initialized to default values.
    #[inline]
    pub fn new_default(len: usize) -> Self {
        Self::new_default_labelled((), len)
    }
}

impl<'a, E, L> FatPtrArray<'a, E, L>
where
    E: Default,
{
    /// Get a new array, initialized to default values.
    #[inline]
    pub fn new_default_labelled(label: L, len: usize) -> Self {
        Self {
            data: FPArrayBlock::new_ptr_default(label, len),
        }
    }
}
impl<'a, E, L> LabelledArray<E, L> for FatPtrArray<'a, E, L> {
    /// Get a reference to the label of the array.
    #[inline]
    fn get_label(&self) -> &L {
        &self.data.label
    }

    /// Get a mutable reference to the label of the array.
    #[inline]
    fn get_label_mut(&mut self) -> &mut L {
        &mut self.data.label
    }
}

impl<'a, E, L> Index<usize> for FatPtrArray<'a, E, L> {
    type Output = E;
    #[inline]
    fn index(&self, idx: usize) -> &E {
        &self.data[idx]
    }
}

impl<'a, E, L> IndexMut<usize> for FatPtrArray<'a, E, L> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut E {
        &mut self.data[idx]
    }
}

impl<'a, E, L> Clone for FatPtrArray<'a, E, L>
where
    L: Clone,
    E: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<'a, E, L> Container<(usize, E)> for FatPtrArray<'a, E, L> {
    #[inline]
    fn add(&mut self, elem: (usize, E)) {
        self[elem.0] = elem.1;
    }
    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<'a, E, L> CopyMap<usize, E> for FatPtrArray<'a, E, L> {
    #[inline]
    fn get(&self, key: usize) -> Option<&E> {
        if key > self.len() {
            None
        } else {
            Some(&self[key])
        }
    }
    #[inline]
    fn get_mut(&mut self, key: usize) -> Option<&mut E> {
        if key > self.len() {
            None
        } else {
            Some(&mut self[key])
        }
    }
    #[inline]
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        if key > self.len() {
            None
        } else {
            // This took so long to figure out but it was totally worth it to
            // mimic java functionality
            let ret = unsafe { std::mem::transmute_copy::<E, E>(&self[key]) };
            let value_ref = (&mut self[key]) as *mut E as *mut ManuallyDrop<E>;
            unsafe {
                *value_ref = ManuallyDrop::new(value);
            }
            Some(ret)
        }
    }
}

impl<'a, E, L> Array<E> for FatPtrArray<'a, E, L> {}
