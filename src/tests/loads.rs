#[derive(Clone, Copy, Default)]
pub struct Light {
    _data: u8,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Test {
    pub a: usize,
    pub b: u8,
    pub c: u8,
}
