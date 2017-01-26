#![feature(test)]

extern crate temporenc;
extern crate test;

use test::Bencher;
use temporenc::*;

const NUM_ITEMS: usize = 1000;

#[bench]
fn serialize_date_only(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateOnly::max_serialized_size());
    b.bytes = v.capacity() as u64;
    b.iter(|| {
        for _ in 0..NUM_ITEMS {
            DateOnly::serialize_components(Some(1000), Some(6), Some(15), &mut v).unwrap();
        }
        v.clear();
    })
}

#[bench]
fn serialize_time_only(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * TimeOnly::max_serialized_size());
    b.bytes = v.capacity() as u64;
    b.iter(|| {
        for _ in 0..NUM_ITEMS {
            TimeOnly::serialize_components(Some(12), Some(30), Some(60), &mut v).unwrap();
        }
        v.clear();
    })
}

#[bench]
fn serialize_date_time(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTime::max_serialized_size());
    b.bytes = v.capacity() as u64;
    b.iter(|| {
        for _ in 0..NUM_ITEMS {
            DateTime::serialize_components(Some(2001), Some(6), Some(15), Some(12), Some(30),
                                           Some(60), &mut v).unwrap();
        }
        v.clear();
    })
}

#[bench]
fn serialize_date_time_offset(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeOffset::max_serialized_size());
    b.bytes = v.capacity() as u64;
    b.iter(|| {
        for _ in 0..NUM_ITEMS {
            DateTimeOffset::serialize_components(Some(2001), Some(6), Some(15), Some(12), Some(30),
                                                 Some(60), OffsetValue::UtcOffset(120), &mut v).unwrap();
        }
        v.clear();
    })
}

#[bench]
fn serialize_date_time_subsecond_ns(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeSubSecond::max_serialized_size());
    b.bytes = v.capacity() as u64;
    b.iter(|| {
        for _ in 0..NUM_ITEMS {
            DateTimeSubSecond::serialize_components(Some(2001), Some(6), Some(15), Some(12),
                                                    Some(30), Some(60),
                                                    FractionalSecond::Nanoseconds(123456789),
                                                    &mut v).unwrap();
        }
        v.clear();
    })
}

#[bench]
fn serialize_date_time_subsecond_ns_offset(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeSubSecondOffset::max_serialized_size());
    b.bytes = v.capacity() as u64;
    b.iter(|| {
        for _ in 0..NUM_ITEMS {
            DateTimeSubSecondOffset::serialize_components(Some(2001), Some(6), Some(15), Some(12),
                                                          Some(30), Some(60),
                                                          FractionalSecond::Nanoseconds(123456789),
                                                          OffsetValue::UtcOffset(-120),
                                                          &mut v).unwrap();
        }
        v.clear();
    })
}
