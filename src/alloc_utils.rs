//! Contains pointer math and allocation utilities.
use core::alloc::Layout;
use core::mem::{align_of, size_of};

// TODO change this to use the alloc crate when stabilized
use std::alloc::{alloc, dealloc};

/// Allocate a block of memory, and then coerce it to type `T`
pub unsafe fn allocate<'a, T>(size: usize, align: usize) -> &'a mut T {
    let layout = Layout::from_size_align(size, align).unwrap();
    &mut *(alloc(layout) as *mut T)
}

/// Deallocate a block of memory using the given size and alignment information.
/// Completely ignores the type of the input pointer, so the size and
/// align need to be correct.
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
