#![feature(test)]

extern crate temporenc;
extern crate test;
extern crate rand;

mod common;

use std::io::Cursor;
use test::Bencher;
use common::{NUM_ITEMS, RandomFieldSource};
use temporenc::*;

#[bench]
fn deserialize_random_date_only(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateOnly::max_serialized_size());
    let mut structs = Vec::with_capacity(NUM_ITEMS);

    let mut r = RandomFieldSource::new(rand::weak_rng());
    for _ in 0..NUM_ITEMS {
        DateOnly::serialize_components(r.year(), r.month(), r.day(), &mut v).unwrap();
    }

    b.bytes = v.len() as u64;

    b.iter(|| {
        let mut cursor = Cursor::new(v.as_slice());
        for _ in 0..NUM_ITEMS {
            structs.push(DateOnly::deserialize(&mut cursor).unwrap());
        }
        structs.clear();
    })
}

#[bench]
fn deserialize_random_time_only(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * TimeOnly::max_serialized_size());
    let mut structs = Vec::with_capacity(NUM_ITEMS);

    let mut r = RandomFieldSource::new(rand::weak_rng());
    for _ in 0..NUM_ITEMS {
        TimeOnly::serialize_components(r.hour(), r.minute(), r.second(), &mut v).unwrap();
    }

    b.bytes = v.len() as u64;

    b.iter(|| {
        let mut cursor = Cursor::new(v.as_slice());
        for _ in 0..NUM_ITEMS {
            structs.push(TimeOnly::deserialize(&mut cursor).unwrap());
        }
        structs.clear();
    })
}

#[bench]
fn deserialize_random_date_time(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTime::max_serialized_size());
    let mut structs = Vec::with_capacity(NUM_ITEMS);

    let mut r = RandomFieldSource::new(rand::weak_rng());
    for _ in 0..NUM_ITEMS {
        DateTime::serialize_components(r.year(), r.month(), r.day(), r.hour(), r.minute(),
                                       r.second(), &mut v).unwrap();
    }

    b.bytes = v.len() as u64;

    b.iter(|| {
        let mut cursor = Cursor::new(v.as_slice());
        for _ in 0..NUM_ITEMS {
            structs.push(DateTime::deserialize(&mut cursor).unwrap());
        }
        structs.clear();
    })
}

#[bench]
fn deserialize_random_date_time_offset(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeOffset::max_serialized_size());
    let mut structs = Vec::with_capacity(NUM_ITEMS);

    let mut r = RandomFieldSource::new(rand::weak_rng());
    for _ in 0..NUM_ITEMS {
        DateTimeOffset::serialize_components(r.year(), r.month(), r.day(), r.hour(), r.minute(),
                                             r.second(), r.offset(), &mut v).unwrap();
    }

    b.bytes = v.len() as u64;

    b.iter(|| {
        let mut cursor = Cursor::new(v.as_slice());
        for _ in 0..NUM_ITEMS {
            structs.push(DateTimeOffset::deserialize(&mut cursor).unwrap());
        }
        structs.clear();
    })
}

#[bench]
fn deserialize_random_date_time_subsecond(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeSubSecond::max_serialized_size());
    let mut structs = Vec::with_capacity(NUM_ITEMS);

    let mut r = RandomFieldSource::new(rand::weak_rng());
    for _ in 0..NUM_ITEMS {
        DateTimeSubSecond::serialize_components(r.year(), r.month(), r.day(), r.hour(),
                                                r.minute(), r.second(), r.fractional_second(),
                                                &mut v).unwrap();
    }

    b.bytes = v.len() as u64;

    b.iter(|| {
        let mut cursor = Cursor::new(v.as_slice());
        for _ in 0..NUM_ITEMS {
            structs.push(DateTimeSubSecond::deserialize(&mut cursor).unwrap());
        }
        structs.clear();
    })
}

#[bench]
fn deserialize_random_date_time_subsecond_offset(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeSubSecondOffset::max_serialized_size());
    let mut structs = Vec::with_capacity(NUM_ITEMS);

    let mut r = RandomFieldSource::new(rand::weak_rng());
    for _ in 0..NUM_ITEMS {
        DateTimeSubSecondOffset::serialize_components(r.year(), r.month(), r.day(), r.hour(),
                                                      r.minute(), r.second(), r.fractional_second(),
                                                      r.offset(), &mut v).unwrap();
    }

    b.bytes = v.len() as u64;

    b.iter(|| {
        let mut cursor = Cursor::new(v.as_slice());
        for _ in 0..NUM_ITEMS {
            structs.push(DateTimeSubSecondOffset::deserialize(&mut cursor).unwrap());
        }
        structs.clear();
    })
}
