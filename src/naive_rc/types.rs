use super::generic::RcArray;
use super::ref_counters::{ArcStruct, RcStruct};
use crate::base::FatPtrArray;
use crate::base::ThinPtrArray;

/// Atomically reference counted array, referenced using a fat pointer.
///
/// See the documentation for `heaparray::naive_rc::generic::RcArray`
/// for more information on API.
pub type FpArcArray<'a, E, L> = RcArray<'a, FatPtrArray<'a, E, ArcStruct<L>>, ArcStruct<L>, E, L>;

/// Reference counted array, referenced using a fat pointer.
///
/// See the documentation for `heaparray::naive_rc::generic::RcArray`
/// for more information on API.
pub type FpRcArray<'a, E, L> = RcArray<'a, FatPtrArray<'a, E, RcStruct<L>>, RcStruct<L>, E, L>;

/// Atomically reference counted array, referenced using a raw pointer.
///
/// See the documentation for `heaparray::naive_rc::generic::RcArray`
/// for more information on API.
pub type TpArcArray<'a, E, L> = RcArray<'a, ThinPtrArray<'a, E, ArcStruct<L>>, ArcStruct<L>, E, L>;

/// Reference counted array, referenced using a raw pointer.
///
/// See the documentation for `heaparray::naive_rc::generic::RcArray`
/// for more information on API.
pub type TpRcArray<'a, E, L> = RcArray<'a, ThinPtrArray<'a, E, RcStruct<L>>, RcStruct<L>, E, L>;

// Note that this implementation satisfies
// the trait bound requirements for `AtomicArrayRef`, and so you can
// alter its pointer atomically:
//
// ```rust
// use heaparray::naive_rc::*;
// use core::sync::atomic::Ordering;
// let array = TpArcArray::new(100, |_| 12);
// let other = TpArcArray::new(100, |_| 13);
// let array_ref = array.as_ref();
// let result = array.compare_and_swap(array_ref, other, Ordering::Relaxed);
// ```
