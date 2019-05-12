mod array_ptr;

#[derive(Eq, PartialEq)]
pub struct Test {
    pub a: usize,
    pub b: u8,
    pub c: u8,
}

impl Default for Test {
    fn default() -> Test {
        Self { a: 100, b: 2, c: 2 }
    }
}

pub(self) mod prelude {
    pub use super::Test;
    pub use crate::fat_array_ptr::FatPtrArray;
    pub use crate::thin_array_ptr::ThinPtrArray;
    pub use crate::*;
}
