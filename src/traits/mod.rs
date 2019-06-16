mod array_ref;
mod labelled_array;
mod make_array;
mod slice_array;

pub use labelled_array::*;
pub use make_array::*;
pub use slice_array::*;

pub(crate) mod rc {
    pub use super::array_ref::*;
}
