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

/// An iterator that returns each item by reference.
///
/// ```rust
/// # use heaparray::base::*;
/// let array = ThinPtrArray::with_label((), 100, |_,i| i);
/// let mut idx = 0;
/// for elem in &array {
///     println!("{}", elem);
///     assert!(elem == &idx);
///     idx += 1;
/// }
/// ```
#[repr(transparent)]
pub struct ThinPtrArrayIterRef<'a, E, L>(pub(crate) MemBlockIterRef<'a, E, LenLabel<L>>);

impl<'a, E, L> Iterator for ThinPtrArrayIterRef<'a, E, L> {
    type Item = &'a E;
    fn next(&mut self) -> Option<&'a E> {
        self.0.next()
    }
}

/// An iterator that returns each item by mutable reference.
///
/// ```rust
/// # use heaparray::base::*;
/// let mut array = ThinPtrArray::with_label((), 100, |_,i| i);
/// let mut idx = 100;
/// {
///     for elem in &mut array {
///         println!("{}", elem);
///         *elem = idx;
///         idx -= 1;
///     }
/// }
/// let mut idx = 100;
/// for elem in array {
///     assert!(elem == idx);
///     idx -= 1;
/// }
/// ```
#[repr(transparent)]
pub struct ThinPtrArrayIterMut<'a, E, L>(pub(crate) MemBlockIterMut<'a, E, LenLabel<L>>);

impl<'a, E, L> Iterator for ThinPtrArrayIterMut<'a, E, L> {
    type Item = &'a mut E;
    fn next(&mut self) -> Option<&'a mut E> {
        self.0.next()
    }
}
