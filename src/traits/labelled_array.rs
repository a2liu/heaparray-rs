/// Array with an optional label struct stored next to the data.
pub trait LabelledArray<E, L>: containers::CopyMap<usize, E> {
    /// Create a new array, with values initialized using a provided
    /// function, and label initialized to a provided value.
    fn with_label<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E;

    /// Get a reference to the label.
    fn get_label(&self) -> &L;

    /// Get a reference to the element at a specified index.
    /// Implementations of this method shouldn't do any safety checks.
    unsafe fn get_unchecked(&self, idx: usize) -> &E;
}

/// Array with optional label struct stored next to the data that can
/// be mutated
pub trait LabelledArrayMut<E, L>: LabelledArray<E, L> {
    /// Get mutable reference to the label.
    fn get_label_mut(&mut self) -> &mut L;

    /// Get a mutable reference to the element at a specified index.
    /// Implementations of this method shouldn't do any safety checks.
    unsafe fn get_mut_unchecked(&mut self, idx: usize) -> &mut E;
}

/// Array with optional label struct stored next to the data that can
/// be conditionally mutated.
pub trait LabelledArrayRefMut<E, L>: LabelledArray<E, L> {
    /// Get mutable reference to the label.
    fn get_label_mut(&mut self) -> Option<&mut L>;
}

/// Trait for a labelled array with a default value.
pub trait DefaultLabelledArray<E, L>: LabelledArray<E, L>
where
    E: Default,
{
    /// Create a new array, initialized to default values.
    fn with_len(label: L, len: usize) -> Self;
}
