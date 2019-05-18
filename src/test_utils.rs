pub use core::mem;

pub const LENGTH: usize = 10;
pub type Load = Large;
pub type LabelLoad = Large;

pub trait ArrayTest: containers::Array<Load> + Sized {
    fn get_self(len: usize) -> Self;
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct Light {
    _data: u8,
}

#[derive(Clone, Debug, Copy, Default, Eq, PartialEq)]
pub struct Medium {
    pub a: usize,
    pub b: u32,
    pub c: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Large {
    a: Vec<u8>,
}

impl Default for Large {
    fn default() -> Self {
        let data = Vec::with_capacity(100);
        Self { a: data }
    }
}

pub fn before_alloc() -> interloc::AllocInfo {
    crate::TEST_MONITOR.local_info()
}

pub fn after_alloc<T>(obj: T, before: interloc::AllocInfo) {
    mem::drop(obj);
    let diff = crate::TEST_MONITOR.local_info().relative_to(&before);
    assert!(
        diff.bytes_alloc == diff.bytes_dealloc,
        "Diff is {:#?}",
        diff
    );
}
