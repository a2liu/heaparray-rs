use crate::naive_rc::generic::RcArray as GenRcArray;
use crate::naive_rc::ref_counters::*;
use crate::prelude::*;
use crate::thin_array_ptr::ThinPtrArray as TpArr;

type RC<L> = ArcStruct<L>;
type ArrPtr<'a, E, L> = TpArr<'a, E, RC<L>>;
type Inner<'a, E, L> = GenRcArray<'a, ArrPtr<'a, E, L>, RC<L>, E, L>;

/// Atomically reference counted array.
#[repr(C)]
pub struct ArcArray<'a, E, L = ()>(Inner<'a, E, L>);

impl<'a, E, L> BaseArrayRef for ArcArray<'a, E, L> {
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
impl<'a, E, L> Clone for ArcArray<'a, E, L> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<'a, E, L> ArrayRef for ArcArray<'a, E, L> {
    fn to_null(&mut self) {
        self.0.to_null()
    }
    fn null_ref() -> Self {
        Self(Inner::null_ref())
    }
}
impl<'a, E, L> Index<usize> for ArcArray<'a, E, L> {
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        self.0.index(idx)
    }
}
impl<'a, E, L> IndexMut<usize> for ArcArray<'a, E, L> {
    fn index_mut(&mut self, idx: usize) -> &mut E {
        self.0.index_mut(idx)
    }
}

impl<'a, E, L> Container<(usize, E)> for ArcArray<'a, E, L> {
    fn add(&mut self, elem: (usize, E)) {
        self.0.add(elem)
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, E, L> CopyMap<'a, usize, E> for ArcArray<'a, E, L> {
    fn get(&'a self, key: usize) -> Option<&'a E> {
        self.0.get(key)
    }
    fn get_mut(&'a mut self, key: usize) -> Option<&'a mut E> {
        self.0.get_mut(key)
    }
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        self.0.insert(key, value)
    }
}

impl<'a, E, L> Array<'a, E> for ArcArray<'a, E, L> {}

impl<'a, E, L> LabelledArray<'a, E, L> for ArcArray<'a, E, L> {
    fn with_label<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        Self(Inner::with_label(label, len, func))
    }
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self {
        Self(Inner::with_label_unsafe(label, len))
    }
    fn get_label(&self) -> &L {
        self.0.get_label()
    }
    fn get_label_mut(&mut self) -> &mut L {
        self.0.get_label_mut()
    }
    unsafe fn get_label_unsafe(&self) -> &mut L {
        self.0.get_label_unsafe()
    }
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E {
        self.0.get_unsafe(idx)
    }
}

impl<'a, E> MakeArray<'a, E> for ArcArray<'a, E, ()>
where
    E: 'a,
{
    fn new<F>(len: usize, func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self(Inner::new(len, func))
    }
}

impl<'a, E, L> DefaultLabelledArray<'a, E, L> for ArcArray<'a, E, L>
where
    E: Default,
{
    fn with_len(label: L, len: usize) -> Self {
        Self(Inner::with_len(label, len))
    }
}

unsafe impl<'a, E, L> Send for ArcArray<'a, E, L> where Inner<'a, E, L>: Send {}
unsafe impl<'a, E, L> Sync for ArcArray<'a, E, L> where Inner<'a, E, L>: Sync {}
