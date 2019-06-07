//! Contains pointer math and allocation utilities.
use core::alloc::Layout;
use core::mem::{align_of, size_of};
use std::alloc::{alloc, dealloc};

/// Allocate a block of memory, and then coerce it to type `T`
pub unsafe fn allocate<T>(size: usize, align: usize) -> *mut T {
    let layout = Layout::from_size_align(size, align).unwrap();
    &mut *(alloc(layout) as *mut T)
}

/// Deallocate a block of memory using the given size and alignment information.
/// Completely ignores the type of the input pointer, so the size and
/// align need to be correct.
pub unsafe fn deallocate<T>(ptr: *mut T, size: usize, align: usize) {
    let layout = Layout::from_size_align(size, align).unwrap();
    dealloc(ptr as *mut u8, layout);
}

/// Aligns everything and potentially slightly overestimates the amount of space necessary
pub fn size_align_multiple(alignments: &[(usize, usize)]) -> (usize, usize) {
    let mut max_align: usize = 0;
    for (_, align) in alignments {
        max_align = core::cmp::max(max_align, *align);
    }
    let mut total_size = 0;
    for (size, _) in alignments {
        total_size += ensure_align(*size, max_align).0;
    }
    (total_size, max_align)
}

/// Returns the (size, alignment) of an array of elements with capacity T
pub const fn size_align_array<T>(capacity: usize) -> (usize, usize) {
    let (size, align) = size_align::<T>();
    (size * capacity, align)
}

/// Get the size and alignment of a type in bytes
pub const fn size_align<T>() -> (usize, usize) {
    let align = align_of::<T>();
    let size = size_of::<T>();
    (size, align)
}

///
pub const fn aligned_size<T>(align: usize) -> usize {
    let size = size_of::<T>();
    ensure_align(size, align).0
}

/// Ensure size is a multiple of align.
pub const fn ensure_align(size: usize, align: usize) -> (usize, usize) {
    let off_by = size % align;
    let adjusted_size = size - off_by + align;
    (cond(off_by == 0, size, adjusted_size), align)
}

// TODO move this to separate crate
/// Returns 1 if `n` is zero and 0 if `n` is greater than zero
///
/// ```rust
/// use heaparray::alloc_utils::is_zero;
/// assert!(is_zero(0) == 1);
/// assert!(is_zero(1) == 0);
/// assert!(is_zero(2) == 0);
/// assert!(is_zero(3) == 0);
/// assert!(is_zero(4) == 0);
/// assert!(is_zero(5) == 0);
/// ```
pub const fn is_zero(n: usize) -> usize {
    (n == 0) as usize
}

/// Returns 1 if `n` is zero and 0 if `n` is greater than zero
pub const fn not_zero(n: usize) -> usize {
    is_zero(is_zero(n))
}

/// Returns `if_true` if cond is true, and otherwise returns `if_false`
///
/// ```rust
/// use heaparray::alloc_utils::cond;
/// assert!(cond(true, 33, 121) == 33);
/// assert!(cond(false, 33, 121) == 121);
/// ```
pub const fn cond(cond: bool, if_true: usize, if_false: usize) -> usize {
    (cond as usize) * if_true + (!cond as usize) * if_false
}

/// Returns `dividend - divisor` if divisor isn't zero, and `core::usize::MAX`
/// otherwise.
pub const fn safe_div(dividend: usize, divisor: usize) -> usize {
    let val = dividend / (divisor + is_zero(divisor));
    cond(divisor == 0, core::usize::MAX, val)
}

pub const fn max(a: usize, b: usize) -> usize {
    cond(a > b, a, b)
}

pub const fn min(a: usize, b: usize) -> usize {
    cond(a > b, b, a)
}
