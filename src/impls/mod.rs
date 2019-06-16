/*!
Implementations of safe APIs to the `BaseArray` struct.

`BaseArray` is defined in [`heaparray::base`](../base/index.html).
*/

mod generic;
mod p_types;

pub use crate::api_prelude::*;
pub use generic::*;
pub use p_types::{FatPtrArray, ThinPtrArray};
