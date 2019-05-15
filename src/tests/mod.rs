#[allow(dead_code)]
pub(crate) mod monitor;

mod fat_array_ptr;
mod thin_array_ptr;

mod prelude {
    pub use crate::prelude::*;
    pub(crate) use core::mem;
}
