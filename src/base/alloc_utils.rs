//! Contains pointer math and allocation utilities.
#[cfg(feature = "no-std")]
use crate::alloc::alloc;
use const_utils::cond;
use core::alloc::Layout;
use core::mem::{align_of, size_of};
#[cfg(not(feature = "no-std"))]
use std::alloc;

/// Represents the global allocator. A hack while the allocator APIs are still
/// very small.
pub struct Global;

unsafe impl alloc::GlobalAlloc for Global {
    unsafe fn alloc(&self, layout: alloc::Layout) -> *mut u8 {
        alloc::alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: alloc::Layout) {
        alloc::dealloc(ptr, layout);
    }
}

/// Allocate a block of memory, and then coerce it to type `T`
pub unsafe fn allocate<T>(layout: Layout, allocator: impl alloc::GlobalAlloc) -> *mut T {
    allocator.alloc(layout) as *mut T
}

/// Deallocate a block of memory using the given size and alignment information.
///
/// Completely ignores the type of the input pointer, so the layout
/// needs to be correct.
pub unsafe fn deallocate<T>(ptr: *mut T, layout: Layout, allocator: impl alloc::GlobalAlloc) {
    allocator.dealloc(ptr as *mut u8, layout);
}

/// Get the size and alignment, in bytes, of a type repeated `repeat` many times.
pub const fn size_align<T>(repeat: usize) -> (usize, usize) {
    let align = align_of::<T>();
    let size = size_of::<T>();
    (size * repeat, align)
}

/// Gets the aligned size of a type given a specific alignment
pub const fn aligned_size<T>(align: usize) -> usize {
    let size = size_of::<T>();
    let off_by = size % align;
    let adjusted_size = size + align - off_by;
    cond(off_by == 0, size, adjusted_size)
}
