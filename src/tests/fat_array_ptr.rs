use crate::tests::prelude::*;

#[test]
fn make_array() {
    let _array = FatPtrArray::new_labelled(Test::default(), 10, |_, i| i);
}

#[test]
#[should_panic]
fn bounds_check() {
    let fat = FatPtrArray::new_labelled(Test::default(), 10, |_, i| i);
    println!("{}", fat[10]);
}

#[test]
fn data_check() {
    let arr = FatPtrArray::new_labelled(Test::default(), 100, |_, _| Test::default());
    let default = Test::default();
    for i in 0..arr.len() {
        assert!(default == arr[i]);
    }
}

#[test]
fn swap_exchange() {
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

#[test]
fn check_drop() {
    let monitor = &crate::TEST_MONITOR;
    let origin = monitor.local_info();
    let arr = FatPtrArray::new(100, |_| Vec::<usize>::new());
    mem::drop(arr);
    let diff = monitor.local_info().relative_to(&origin);
    assert!(
        diff.bytes_alloc == diff.bytes_dealloc,
        "Diff is {:#?}",
        diff
    );
}

#[should_panic]
#[test]
fn check_null() {
    let null_ptr = unsafe { FatPtrArray::<u8, ()>::null_ref() };
    mem::drop(null_ptr);
}
