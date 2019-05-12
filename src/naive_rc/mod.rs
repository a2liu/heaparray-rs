//! This module contains naively reference counted arrays, both as atomic and
//! regular versions.
//!
pub mod fat_arc_array_ptr;
pub mod fat_rc_array_ptr;
pub(crate) mod prelude;
pub mod thin_arc_array_ptr;
pub mod thin_rc_array_ptr;

pub use crate::prelude::*;
pub use fat_arc_array_ptr::FpArcArray as ArcArray;
pub use fat_rc_array_ptr::FpRcArray as RcArray;
