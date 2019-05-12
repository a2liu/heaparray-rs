use crate::tests::prelude::*;

#[test]
fn make_arrays() {
    let _thin = ThinPtrArray::new_labelled(Test::default(), 10, |_, i| i);
}

#[test]
#[should_panic]
fn bounds_check_fat() {
    let fat = FatPtrArray::new_labelled(Test::default(), 10, |_, i| i);
    println!("{}", fat[10]);
}

#[test]
#[should_panic]
fn bounds_check_thin() {
    let thin = ThinPtrArray::new_labelled(Test::default(), 10, |_, i| i);
    println!("{}", thin[10]);
}

#[test]
fn data_check_fat() {
    let arr = FatPtrArray::new_labelled(Test::default(), 100, |_, _| Test::default());
    let default = Test::default();
    for i in 0..arr.len() {
        assert!(default == arr[i]);
    }
}

#[test]
fn data_check_thin() {
    let arr = ThinPtrArray::new_labelled(Test::default(), 100, |_, _| Test::default());
    let default = Test::default();
    for i in 0..arr.len() {
        assert!(default == arr[i]);
    }
}

#[test]
fn swap_exchange_thin() {
    let mut arr = ThinPtrArray::new_labelled(Test::default(), 100, |_, i| Test {
        a: i,
        b: i as u8,
        c: i as u8,
    });

    let mut default = Test::default();
    let len = arr.len();
    for i in 0..len {
        default = match arr.insert(i, default) {
            Some(x) => x,
            None => panic!("should not return None"),
        }
    }
    assert!(Test::default() == arr[0]);
    for i in 1..arr.len() {
        assert!(arr[i].a == i - 1);
        assert!(arr[i].b == i as u8 - 1);
        assert!(arr[i].c == i as u8 - 1);
    }
}

#[test]
fn swap_exchange_fat() {
    let mut arr = FatPtrArray::new_labelled(Test::default(), 100, |_, i| Test {
        a: i,
        b: i as u8,
        c: i as u8,
    });

    let mut default = Test::default();
    let len = arr.len();
    for i in 0..len {
        default = match arr.insert(i, default) {
            Some(x) => x,
            None => panic!("should not return None"),
        }
    }
    assert!(Test::default() == arr[0]);
    for i in 1..arr.len() {
        assert!(arr[i].a == i - 1);
        assert!(arr[i].b == i as u8 - 1);
        assert!(arr[i].c == i as u8 - 1);
    }
}
