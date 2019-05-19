use super::generic::RcArray;
use super::ref_counters::{ArcStruct, RcStruct};
use crate::base::FatPtrArray;
use crate::base::ThinPtrArray;

pub type FpArcArray<'a, E, L> = RcArray<'a, FatPtrArray<'a, E, ArcStruct<L>>, ArcStruct<L>, E, L>;

/// test docs
pub type FpRcArray<'a, E, L> = RcArray<'a, FatPtrArray<'a, E, RcStruct<L>>, RcStruct<L>, E, L>;

pub type TpArcArray<'a, E, L> = RcArray<'a, ThinPtrArray<'a, E, ArcStruct<L>>, ArcStruct<L>, E, L>;

pub type TpRcArray<'a, E, L> = RcArray<'a, ThinPtrArray<'a, E, RcStruct<L>>, RcStruct<L>, E, L>;
