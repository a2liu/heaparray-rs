use crate::naive_rc::*;

type TestArray<'a, E, L = ()> = FpRcArray<'a, E, L>;

#[test]
fn null_test() {
    let null = TestArray::<Load, LabelLoad>::null_ref();
    assert!(null.is_null());
}

#[test]
fn clone_test() {
    let first_ref = TestArray::<Load, LabelLoad>::with_len(LabelLoad::default(), 1000);
    let second_ref = ArrayRef::clone(&first_ref);
    assert!(first_ref.len() == second_ref.len());
    for i in 0..second_ref.len() {
        let r1 = &first_ref[i] as *const Load;
        let r2 = &second_ref[i] as *const Load;
        assert!(r1 == r2);
    }
}

#[test]
fn small_ref_counting() {
    let bfr = before_alloc();
    let first_ref = TestArray::<Load, LabelLoad>::with_len(LabelLoad::default(), LENGTH);
    after_alloc(first_ref, bfr);
}

#[test]
fn ref_counting_test() {
    let mut ref_vec = Vec::with_capacity(10000);
    let t_0 = before_alloc();
    let first_ref = TestArray::<Load, LabelLoad>::with_len(LabelLoad::default(), 1000);
    ref_vec.push(first_ref);
    let balloc = before_alloc().bytes_alloc;
    for _ in 0..LENGTH {
        let new_ref = ArrayRef::clone(ref_vec.last().unwrap());
        assert!(before_alloc().bytes_alloc == balloc);
        ref_vec.push(new_ref);
    }
    let final_ref = ArrayRef::clone(&ref_vec[0]);
    for arr_ref in &mut ref_vec {
        arr_ref.to_null();
    }
    assert!(before_alloc().bytes_alloc == balloc);
    after_alloc(final_ref, t_0);
}
