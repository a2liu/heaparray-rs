use crate::base::*;
use crate::prelude::*;

/// Array pointer that also knows what its length is.
///
/// In addition to the invariants discussed in
/// [`BaseArrayPtr`](trait.BaseArrayPtr.html), implementors of this trait need
/// to maintain the following for any instance created via
/// `let array = Self::alloc(len)`:
/// - The method `array.set_len(len)`, and only that method, can change the result
///   of `array.get_len()`, and after calling `array.set_len(len)`, `array.get_len()`
///   will return the value that was set
/// - Any pointer returned by `array.get_ptr(i)` or `array.get_ptr_mut()` where
///   `i < len` points to aligned, allocated memory
/// - both `array.get_label()` and `array.get_label_mut()` point to allocated, aligned
///   memory as well.
pub unsafe trait SafeArrayPtr<E, L>: BaseArrayPtr<E, L> {
    /// Set the length of this array
    fn set_len(&mut self, len: usize);

    /// Get the length of this array
    fn get_len(&self) -> usize;
}

/// Safe, generic interface to [`BaseArray`](../base/struct.BaseArray.html).
///
/// Uses length information to guarrantee memory safety, and excludes operations
/// that cannot be performed safely from its API.
#[repr(transparent)]
pub struct SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
{
    data: BaseArray<E, L, P>,
}

impl<E, L, P> Container for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
{
    fn len(&self) -> usize {
        self.data.as_ptr().get_len()
    }
}

impl<E, L, P> Drop for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
{
    fn drop(&mut self) {
        let len = self.len();
        unsafe { self.data.drop(len) };
    }
}

impl<E, L, P> CopyMap<usize, E> for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
{
    fn get(&self, key: usize) -> Option<&E> {
        if key >= self.len() {
            None
        } else {
            Some(unsafe { &*self.data.get(key) })
        }
    }
    fn get_mut(&mut self, key: usize) -> Option<&mut E> {
        if key >= self.len() {
            None
        } else {
            Some(unsafe { &mut *self.data.get_mut(key) })
        }
    }
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        match self.get_mut(key) {
            Some(slot) => Some(mem::replace(slot, value)),
            None => None,
        }
    }
}

impl<E, L, P> LabelledArray<E, L> for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
{
    fn with_label<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let mut out = Self {
            data: BaseArray::new(label, len, func),
        };
        out.data.as_ptr_mut().set_len(len);
        out
    }
    fn get_label(&self) -> &L {
        self.data.get_label()
    }
    unsafe fn get_unchecked(&self, idx: usize) -> &E {
        self.data.get(idx)
    }
}

impl<E, L, P> LabelledArrayMut<E, L> for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
{
    fn get_label_mut(&mut self) -> &mut L {
        self.data.get_label_mut()
    }
    unsafe fn get_mut_unchecked(&mut self, idx: usize) -> &mut E {
        self.data.get_mut(idx)
    }
}

impl<E, P> MakeArray<E> for SafeArray<E, (), P>
where
    P: SafeArrayPtr<E, ()>,
{
    fn new<F>(len: usize, mut func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self::with_label((), len, |_, idx| func(idx))
    }
}

impl<E, L, P> DefaultLabelledArray<E, L> for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
    E: Default,
{
    fn with_len(label: L, len: usize) -> Self {
        Self::with_label(label, len, |_, _| E::default())
    }
}

impl<E, L, P> Clone for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
    E: Clone,
    L: Clone,
{
    fn clone(&self) -> Self {
        Self::with_label(self.get_label().clone(), self.len(), |_, i| self[i].clone())
    }
    fn clone_from(&mut self, source: &Self) {
        if source.len() != self.len() {
            *self = source.clone();
        } else {
            self.get_label_mut().clone_from(source.get_label());
            for i in 0..source.len() {
                self[i].clone_from(&source[i]);
            }
        }
    }
}

impl<E, L, P> Index<usize> for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
{
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        self.get(idx).unwrap()
    }
}

impl<E, L, P> IndexMut<usize> for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
{
    fn index_mut(&mut self, idx: usize) -> &mut E {
        self.get_mut(idx).unwrap()
    }
}

impl<E, L, P> IntoIterator for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
{
    type Item = E;
    type IntoIter = BaseArrayIter<E, L, P>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        let iter = unsafe { core::ptr::read(&self.data).into_iter(len) };
        mem::forget(self);
        iter
    }
}

impl<E, L, P> SliceArray<E> for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
{
    fn as_slice(&self) -> &[E] {
        let len = self.len();
        unsafe { self.data.as_slice(len) }
    }
}

impl<E, L, P> SliceArrayMut<E> for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
{
    fn as_slice_mut(&mut self) -> &mut [E] {
        let len = self.len();
        unsafe { self.data.as_slice_mut(len) }
    }
}

impl<'a, E, L, P> IntoIterator for &'a SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
{
    type Item = &'a E;
    type IntoIter = core::slice::Iter<'a, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().into_iter()
    }
}

impl<'a, E, L, P> IntoIterator for &'a mut SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
{
    type Item = &'a mut E;
    type IntoIter = core::slice::IterMut<'a, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice_mut().into_iter()
    }
}

impl<E, L, P> fmt::Debug for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
    E: fmt::Debug,
    L: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("Array")
            .field("label", &self.get_label())
            .field("len", &self.len())
            .field("elements", &self.as_slice())
            .finish()
    }
}

unsafe impl<E, L, P> Send for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
    E: Send,
    L: Send,
{
}

unsafe impl<E, L, P> Sync for SafeArray<E, L, P>
where
    P: SafeArrayPtr<E, L>,
    E: Sync,
    L: Sync,
{
}
