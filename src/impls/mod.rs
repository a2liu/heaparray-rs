/*!
Implementations of safe APIs to the `BaseArray` struct.

`BaseArray` is defined in [`heaparray::base`](../base/index.html)
*/

mod generic;

pub use crate::api_prelude::*;
pub use crate::base::BaseArrayPtr;
pub use generic::*;
