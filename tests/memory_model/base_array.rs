use crate::prelude::*;
use core::ptr::NonNull;
use heaparray::base::{BaseArray, MemBlock};

type Array<E, L> = BaseArray<E, L, NonNull<MemBlock<E, L>>>;

#[test]
fn new() {
    let info = before_alloc();
    let mut array = Array::new(Vec::<u8>::with_capacity(10), 100, |_, _| {
        Vec::<u8>::with_capacity(10)
    });
    unsafe {
        array.drop(100);
    }
    after_alloc(array, info);
}

#[test]
fn label_element_access() {
    for _ in 0..1000 {
        let mut array = Array::new(Vec::<u8>::with_capacity(10), 100, |_, _| {
            Vec::<u8>::with_capacity(10)
        });
        array.get_label_mut().push(10);
        for i in 0..100 {
            // Changing this to a 101 should result in a seg fault
            unsafe {
                array.get_mut(i).push(10);
            }
        }
    }
}
