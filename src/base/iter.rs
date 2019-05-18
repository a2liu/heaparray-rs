use super::thin::LenLabel;
use crate::mem_block::*;

/// An iterator that that returns each item by ownership
///
/// ```rust
/// # use heaparray::base::*;
/// let array = ThinPtrArray::with_label((), 100, |_,i| i);
/// let mut idx = 0;
/// for elem in array {
///     println!("{}",elem);
///     assert!(elem == idx);
///     idx += 1;
/// }
/// ```
#[repr(transparent)]
pub struct ThinPtrArrayIterOwned<'a, E, L>(pub(crate) MemBlockIterOwned<'a, E, LenLabel<L>>);

impl<'a, E, L> Iterator for ThinPtrArrayIterOwned<'a, E, L> {
    type Item = E;
    fn next(&mut self) -> Option<E> {
        self.0.next()
    }
}
