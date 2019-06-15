use crate::prelude::*;
use heaparray::base::MemBlock as HeapArrayMemBlock;

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

#[test]
pub fn ref_dealloc_lazy_leak() {
    let vec = Vec::with_capacity(10);
    let info = before_alloc();
    let mut blk = unsafe { MemBlock::<Vec<u8>, Vec<u8>>::new(vec, 200) };
    unsafe {
        blk.as_mut().dealloc_lazy(200);
    }
    after_alloc(blk, info);
}

#[test]
pub fn ref_alloc_efficient() {
    use core::mem::size_of;

    let alloc_size = 200 * size_of::<Vec<()>>();
    let info = before_alloc();

    let mut blk = MemBlock::<Vec<()>, ()>::new_init((), 200, |_, _| Vec::new());
    let info_2 = before_alloc();
    let info_diff = info_2.relative_to(&info);

    assert!(
        info_diff.bytes_alloc == alloc_size,
        "Allocation had incorrect size;\n\
         Stats are {:#?}",
        info_diff
    );

    assert!(
        info_diff.bytes_dealloc == 0,
        "Deallocated during allocation!\n\
         Stats are {:#?}",
        info_diff
    );

    unsafe {
        blk.as_mut().dealloc(200);
    }
    let info_diff = before_alloc().relative_to(&info_2);

    assert!(
        info_diff.bytes_dealloc == alloc_size,
        "Deallocation had incorrect size;\n\
         Stats are {:#?}",
        info_diff
    );

    assert!(
        info_diff.bytes_alloc == 0,
        "Allocated during deallocation!\n\
         Stats are {:#?}",
        info_diff
    );
}

// #[test]
// pub fn block_alignment() {
//     let blk = MemBlock::<(), Vec<
// }
