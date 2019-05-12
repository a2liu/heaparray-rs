//! This module contains naively reference counted arrays, both as atomic and
//! regular versions. i.e. if you're not careful, you could make a cycle that
//! never gets deallocated.
//!
//! # Examples

pub mod fat_arc_array_ptr;
pub mod fat_rc_array_ptr;
pub mod ref_counters;
// pub mod thin_arc_array_ptr;
// pub mod thin_rc_array_ptr;

pub use crate::prelude::*;
pub use fat_arc_array_ptr::FpArcArray as ArcArray;
pub use fat_rc_array_ptr::FpRcArray as RcArray;

pub(crate) mod prelude {
    pub(crate) use super::ref_counters::*;
    pub(crate) use crate::fat_array_ptr::FatPtrArray;
    pub use crate::prelude::*;
    // pub(crate) use crate::thin_array_ptr::ThinPtrArray;
}
