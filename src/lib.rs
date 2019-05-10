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
/// let generator = |_label, idx| idx + 3;
/// let array = HeapArray::new(label, len, generator);
/// ```
///
/// Indexing works as you would expect:
/// ```rust
/// let d = array[i];
/// array[i] = 2;
/// ```
pub use fat_array_ptr::FatPtrArray as HeapArray;
pub use fat_array_ptr::*;
pub use prelude::*;
pub use thin_array_ptr::*;

#[cfg(test)]
pub mod tests;
