#![allow(deprecated)]
use core::alloc::Layout;
use std::alloc::{alloc, dealloc};
use std::mem::{align_of, size_of};

pub unsafe fn allocate<'a, T>(size: usize, align: usize) -> &'a mut T {
    let layout = Layout::from_size_align(size, align).unwrap();
    &mut *(alloc(layout) as *mut T)
}

pub unsafe fn deallocate<T>(ptr: &mut T, size: usize, align: usize) {
    let layout = Layout::from_size_align(size, align).unwrap();
    dealloc(ptr as *mut T as *mut u8, layout);
}

/// Aligns everything and potentially slightly overestimates the amount of space necessary
pub fn size_align_multiple(alignments: &[(usize, usize)]) -> (usize, usize) {
    let mut max_align: usize = 0;
    for (_, align) in alignments {
        if max_align < *align {
            max_align = *align;
        }
    }
    let mut total_size = 0;
    for (size, _) in alignments {
        total_size += ensure_align(*size, max_align).0;
    }
    (total_size, max_align)
}

// TODO Use std::mem::align_of and std::mem::size_of to get the size and alignment
// of the array, and then create the layout with Layout::from_size_align(size, align)

/// Returns the (size, alignment) of an array of elements with capacity T
#[inline]
pub fn size_align_array<T>(capacity: usize) -> (usize, usize) {
    let (size, align) = size_align::<T>();
    let (size, align) = ensure_align(size, align);
    (size * capacity, align)
}

/// Get the size and alignment of a type in bytes
#[inline]
pub const fn size_align<T>() -> (usize, usize) {
    let align = align_of::<T>();
    let size = size_of::<T>();

    (size, align)
}

/// Ensure size is a multiple of align.
#[inline]
pub fn ensure_align(mut size: usize, align: usize) -> (usize, usize) {
    let off_by = size % align;
    if off_by != 0 {
        size -= off_by;
        size += align;
    }
    (size, align)
}
