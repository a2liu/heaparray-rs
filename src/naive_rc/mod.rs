//! This module contains naively reference counted arrays, both as atomic and
//! regular versions; i.e. if you're not careful, you could make a cycle that
//! never gets deallocated.

pub mod generic;
pub mod ref_counters;
mod types;

pub use crate::prelude::*;
pub use types::*;
