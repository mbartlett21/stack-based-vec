#![no_main]

use libfuzzer_sys::fuzz_target;
use stack_based_vec::ArrayVec;

const CAPACITY: usize = 32;

#[derive(arbitrary::Arbitrary, Debug)]
struct Data {
    extend_from_copyable_slice: Vec<i32>,
    push: i32,
    truncate: usize
}

fuzz_target!(|data: Data| {
    let mut v: ArrayVec<i32, CAPACITY> = ArrayVec::new();

    extend_from_copyable_slice(&data, &mut v);
    push(&data, &mut v);
    truncate(&data, &mut v);
});

fn extend_from_copyable_slice(data: &Data, v: &mut ArrayVec<i32, CAPACITY>) {
    let upper_bound = if v.extend_from_copyable_slice(&data.extend_from_copyable_slice).is_ok() {
        data.extend_from_copyable_slice.len()
    }
    else {
        CAPACITY
    };
    assert_eq!(v.as_slice(), &data.extend_from_copyable_slice[0..upper_bound]);
}

fn push(data: &Data, v: &mut ArrayVec<i32, CAPACITY>) {
    let idx = v.len();
    let _ = v.push(data.push);
    if let Some(rslt) = v.get(idx) {
        assert_eq!(*rslt, data.push);
    }
}

fn truncate(data: &Data, v: &mut ArrayVec<i32, CAPACITY>) {
    let original = *v;
    v.truncate(data.truncate);
    if let Some(rslt) = v.get(0..data.truncate) {
        assert_eq!(rslt, &original[0..data.truncate]);
    }
}