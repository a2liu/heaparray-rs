/// Array that returns a slice into its contents
pub trait SliceArray<E> {
    /// Returns a reference to a slice into the elements of this array.
    fn as_slice(&self) -> &[E];
}

/// Array that returns a mutable slice into its contents
pub trait SliceArrayMut<E> {
    /// Returns a mutable reference to a slice into the elements of this array.
    fn as_slice_mut(&mut self) -> &mut [E];
}

/*
/// Array reference that can return a slice into its contents.
pub trait SliceArrayRef<E> {
    /// Returns a reference to a slice into this array.
    fn as_slice(&self) -> &[E];

    /// Returns a mutable reference to a slice into this array.
    fn as_slice_mut(&mut self) -> Option<&mut [E]>;
}*/
