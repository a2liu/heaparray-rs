use crate::*;
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
