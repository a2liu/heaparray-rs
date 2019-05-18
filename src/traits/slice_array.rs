/// Array that returns slices into its contents
pub trait SliceArray<E> {
    fn as_slice(&self) -> &[E];
    fn as_slice_mut(&mut self) -> &mut [E];
}
