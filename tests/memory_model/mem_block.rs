use crate::prelude::*;
use heaparray::base::MemBlock;

#[test]
pub fn ref_no_dealloc() {
    let blk = unsafe { MemBlock::<Vec<u8>, ()>::alloc(200) };
    let info = before_alloc();
    after_alloc(blk, info);
}

#[test]
pub fn ref_dealloc_lazy() {
    let info = before_alloc();
    let mut blk = unsafe { MemBlock::<Vec<u8>, ()>::alloc(200) };
    unsafe { blk.as_mut().dealloc_lazy(200) };
    after_alloc(blk, info);
}

#[test]
pub fn ref_dealloc_normal() {
    let info = before_alloc();
    let mut blk = MemBlock::<Vec<u8>, ()>::new_init((), 200, |_, _| Vec::with_capacity(10));
    unsafe { blk.as_mut().dealloc(200) };
    after_alloc(blk, info);
}
