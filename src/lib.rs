extern crate containers_rs as containers;

/// Array with an optional label struct stored next to the data.
pub trait LabelledArray<L, E>: containers::Array<E> {
    /// Get immutable access to the label. Causes undefined behavior if
    /// L is a zero-sized type.
    fn get_label(&self) -> &L;
    /// Get mutable reference to the label. Causes undefined behavior if
    /// L is a zero-sized type.
    fn get_label_mut(&mut self) -> &mut L;
}

mod alloc;
pub mod fat_array_ptr;
mod memory_block;
pub mod thin_array_ptr;

mod prelude {
    pub(crate) use super::memory_block::*;
    pub use super::LabelledArray;
    pub use containers::{Array, Container, CopyMap};
    pub(crate) use core::mem::ManuallyDrop;
    pub use core::ops::{Index, IndexMut};
}

/// Heap-allocated array.
///
/// ## Examples
///
/// Creating an array:
/// ```rust
/// use heaparray::*;
/// let len = 10;
/// let label = ();
/// let array = HeapArray::new(label, len, |_label, idx| idx + 3);
/// ```
///
/// Indexing works as you would expect:
/// ```rust
/// # use heaparray::*;
/// # let mut array = HeapArray::new((), 10, |_label, idx| idx + 3);
/// array[3] = 2;
/// assert!(array[3] == 2);
/// ```
///
///
/// Notably, you can take ownership of objects back from the container:
///
/// ```rust
/// # use heaparray::*;
/// # let mut array = HeapArray::new((), 10, |_,_| Vec::<u8>::new());
/// let replacement_object = Vec::new();
/// let owned_object = array.insert(0, replacement_object);
/// ```
///
/// but you need to give the array a replacement object to fill its slot with.
pub use fat_array_ptr::FatPtrArray as HeapArray;

pub use fat_array_ptr::*;
pub use prelude::*;
pub use thin_array_ptr::*;

#[cfg(test)]
pub mod tests;
