#![feature(test)]

extern crate temporenc;
extern crate test;
extern crate rand;

#[allow(dead_code)]
mod common;

use std::io::Cursor;
use test::Bencher;
use common::{bb, NUM_ITEMS};
use temporenc::*;

#[bench]
fn deserialize_fixed_date_time_subsecond_ms(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeSubSecond::max_serialized_size() - 2);
    let mut structs = Vec::with_capacity(NUM_ITEMS);

    let year = bb(Some(1000));
    let month = bb(Some(6));
    let day = bb(Some(15));
    let hour = bb(Some(12));
    let minute = bb(Some(30));
    let second = bb(Some(60));
    let frac_second = bb(FractionalSecond::Milliseconds(123));

    for _ in 0..NUM_ITEMS {
        DateTimeSubSecond::serialize_components(year, month, day, hour, minute, second,
                                                frac_second, &mut v).unwrap();
    };

    b.bytes = v.len() as u64;

    b.iter(|| {
        let mut cursor = Cursor::new(v.as_slice());
        for _ in 0..NUM_ITEMS {
            structs.push(DateTimeSubSecond::deserialize(&mut cursor).unwrap());
        }
        structs.clear();
    })
}
