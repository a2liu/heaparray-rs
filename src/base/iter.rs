//! Iterators for the arrays in `heaparray::base`

use super::base::BaseArrayIter;
use super::thin::LenLabel;

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
/// assert!(idx == 100);
/// ```
#[repr(transparent)]
pub struct ThinPtrArrayIter<E, L>(pub(crate) BaseArrayIter<E, LenLabel<L>>);

impl<E, L> Iterator for ThinPtrArrayIter<E, L> {
    type Item = E;
    fn next(&mut self) -> Option<E> {
        self.0.next()
    }
}
