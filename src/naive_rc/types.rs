use super::generic::RcArray;
use super::ref_counters::{ArcStruct, RcStruct};
use crate::base::{FatPtrArray, ThinPtrArray};

/// Atomically reference counted array, referenced using a fat pointer.
///
/// See the documentation for `heaparray::naive_rc::generic::RcArray`
/// for more information on API.
pub type FpArcArray<E, L> = RcArray<FatPtrArray<E, ArcStruct<L>>, ArcStruct<L>, E, L>;

/// Reference counted array, referenced using a fat pointer.
///
/// See the documentation for `heaparray::naive_rc::generic::RcArray`
/// for more information on API.
pub type FpRcArray<E, L> = RcArray<FatPtrArray<E, RcStruct<L>>, RcStruct<L>, E, L>;

/// Atomically reference counted array, referenced using a raw pointer.
///
/// See the documentation for `heaparray::naive_rc::generic::RcArray`
/// for more information on API.
pub type TpArcArray<E, L> = RcArray<ThinPtrArray<E, ArcStruct<L>>, ArcStruct<L>, E, L>;

/// Reference counted array, referenced using a raw pointer.
///
/// See the documentation for `heaparray::naive_rc::generic::RcArray`
/// for more information on API.
pub type TpRcArray<E, L> = RcArray<ThinPtrArray<E, RcStruct<L>>, RcStruct<L>, E, L>;
