use crate::base::ThinPtrArray;
use crate::tests::prelude::*;

type TestArray<'a, E, L = ()> = ThinPtrArray<'a, E, L>;

#[test]
fn make_array() {
    let _array = TestArray::with_label(LabelLoad::default(), LENGTH, |_, _| Load::default());
}

#[test]
#[should_panic]
fn bounds_check() {
    let fat = TestArray::with_label(LabelLoad::default(), LENGTH, |_, _| Load::default());
    println!("{:?}", fat[LENGTH]);
}

#[test]
fn data_check() {
    let arr = TestArray::with_label(LabelLoad::default(), LENGTH, |_, _| Load::default());
    let default = LabelLoad::default();
    for i in 0..arr.len() {
        assert!(default == arr[i]);
    }
}

#[test]
fn swap_exchange() {
    let mut arr = TestArray::with_label(LabelLoad::default(), LENGTH, |_, i| Medium {
        a: i,
        b: i as u32,
        c: i as u32,
    });

    let mut default = Medium::default();
    let len = arr.len();
    for i in 0..len {
        default = match arr.insert(i, default) {
            Some(x) => x,
            None => panic!("should not return None"),
        }
    }
    assert!(Medium::default() == arr[0]);
    for i in 1..arr.len() {
        assert!(arr[i].a == i - 1);
        assert!(arr[i].b == i as u32 - 1);
        assert!(arr[i].c == i as u32 - 1);
    }
}

#[test]
fn check_drop() {
    let bfr = before_alloc();
    let arr = TestArray::new(LENGTH, |_| Vec::<usize>::new());
    after_alloc(arr, bfr);
}
