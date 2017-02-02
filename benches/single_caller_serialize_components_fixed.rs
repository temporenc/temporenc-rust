#![feature(test)]

extern crate temporenc;
extern crate test;
extern crate rand;

#[allow(dead_code)]
mod common;

use test::Bencher;
use common::{bb, NUM_ITEMS};
use temporenc::*;

// Because there is only a single caller of each serialize function, LLVM will optimize
// heavily.

#[bench]
fn serialize_components_fixed_date(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateOnly::max_serialized_size());
    b.bytes = v.capacity() as u64;

    b.iter(|| {
        let year = bb(Some(1000));
        let month = bb(Some(6));
        let day = bb(Some(15));

        for _ in 0..NUM_ITEMS {
            DateOnly::serialize_components(year, month, day, &mut v).unwrap();
        };
        v.clear();
    });
}

#[bench]
fn serialize_components_fixed_time(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * TimeOnly::max_serialized_size());
    b.bytes = v.capacity() as u64;

    b.iter(|| {
        let hour = bb(Some(12));
        let minute = bb(Some(30));
        let second = bb(Some(60));

        for _ in 0..NUM_ITEMS {
            TimeOnly::serialize_components(hour, minute, second, &mut v).unwrap();
        };
        v.clear();
    });
}

#[bench]
fn serialize_components_fixed_date_time(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTime::max_serialized_size());
    b.bytes = v.capacity() as u64;

    b.iter(|| {
        let year = bb(Some(1000));
        let month = bb(Some(6));
        let day = bb(Some(15));
        let hour = bb(Some(12));
        let minute = bb(Some(30));
        let second = bb(Some(60));

        for _ in 0..NUM_ITEMS {
            DateTime::serialize_components(year, month, day, hour, minute, second, &mut v).unwrap();
        };
        v.clear();
    });
}

#[bench]
fn serialize_components_fixed_date_time_offset(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeOffset::max_serialized_size());
    b.bytes = v.capacity() as u64;

    b.iter(|| {
        let year = bb(Some(1000));
        let month = bb(Some(6));
        let day = bb(Some(15));
        let hour = bb(Some(12));
        let minute = bb(Some(30));
        let second = bb(Some(60));
        let offset = bb(OffsetValue::UtcOffset(120));

        for _ in 0..NUM_ITEMS {
            DateTimeOffset::serialize_components(year, month, day, hour, minute, second, offset,
                                                 &mut v).unwrap();
        }
        v.clear();
    })
}

#[bench]
fn serialize_components_fixed_date_time_subsecond_ns(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeSubSecond::max_serialized_size());
    b.bytes = v.capacity() as u64;

    b.iter(|| {
        let year = bb(Some(1000));
        let month = bb(Some(6));
        let day = bb(Some(15));
        let hour = bb(Some(12));
        let minute = bb(Some(30));
        let second = bb(Some(60));
        let frac_second = bb(FractionalSecond::Nanoseconds(123456789));

        for _ in 0..NUM_ITEMS {
            DateTimeSubSecond::serialize_components(year, month, day, hour, minute, second,
                                                    frac_second, &mut v).unwrap();
        };
        v.clear();
    });
}

#[bench]
fn serialize_components_fixed_date_time_subsecond_ns_offset(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeSubSecondOffset::max_serialized_size());
    b.bytes = v.capacity() as u64;

    b.iter(|| {
        let year = bb(Some(1000));
        let month = bb(Some(6));
        let day = bb(Some(15));
        let hour = bb(Some(12));
        let minute = bb(Some(30));
        let second = bb(Some(60));
        let frac_second = bb(FractionalSecond::Nanoseconds(123456789));
        let offset = bb(OffsetValue::UtcOffset(120));

        for _ in 0..NUM_ITEMS {
            DateTimeSubSecondOffset::serialize_components(year, month, day, hour, minute, second,
                                                          frac_second, offset, &mut v).unwrap();
        };
        v.clear();
    });
}

