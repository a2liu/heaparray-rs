use crate::base::iter::FatPtrArrayIterOwned;

/// An iterator that that returns each item by ownership
///
/// ```rust
/// # use heaparray::*;
/// let array = HeapArray::with_label((), 100, |_,i| i);
/// let mut idx = 0;
/// for elem in array {
///     println!("{}",elem);
///     assert!(elem == idx);
///     idx += 1;
/// }
/// ```
#[repr(transparent)]
pub struct HeapArrayIterOwned<'a, E, L>(pub(crate) FatPtrArrayIterOwned<'a, E, L>);

impl<'a, E, L> Iterator for HeapArrayIterOwned<'a, E, L> {
    type Item = E;
    fn next(&mut self) -> Option<E> {
        self.0.next()
    }
}
