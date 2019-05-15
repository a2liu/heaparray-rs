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

pub struct TestLarge {
    a: Vec<u8>,
}

impl Default for TestLarge {
    fn default() -> Self {
        let data = Vec::with_capacity(100);
        Self { a: data }
    }
}
