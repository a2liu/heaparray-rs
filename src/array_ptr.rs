use super::memory_block::*;

#[repr(C)]
pub struct TPRawArray<'a, L, E>
where
    Self: 'a,
    L: Clone,
{
    data: &'a mut TPArrayBlock<L, E>,
}

#[repr(C)]
pub struct FPRawArray<'a, L, E>
where
    Self: 'a,
    L: Clone,
{
    data: &'a mut FPArrayBlock<L, E>,
}
