use crate::naive_rc::*;

type TestArray<'a, E, L = ()> = FpRcArray<'a, E, L>;

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
    let t_0 = before_alloc();
    let mut ref_vec = Vec::with_capacity(10000);
    let first_ref = TestArray::<Load, ()>::with_len((), 1000);
    ref_vec.push(first_ref);
    let balloc = before_alloc().bytes_alloc;
    for _ in 0..LENGTH {
        let new_ref = ArrayRef::clone(ref_vec.last().unwrap());
        assert!(
            before_alloc().bytes_alloc == balloc,
            "Clone caused allocation"
        );
        ref_vec.push(new_ref);
    }
    let final_ref = ArrayRef::clone(&ref_vec[0]);
    mem::drop(ref_vec);
    assert!(before_alloc().bytes_alloc == balloc);
    after_alloc(final_ref, t_0);
}
