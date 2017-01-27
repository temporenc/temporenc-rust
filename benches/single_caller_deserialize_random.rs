#![feature(test)]

extern crate temporenc;
extern crate test;
extern crate rand;

mod common;

use std::io::{Cursor};
use test::Bencher;
use common::{NUM_ITEMS, RandomFieldSource};
use temporenc::*;

#[bench]
fn deserialize_date_only_random(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateOnly::max_serialized_size());

    let mut r = RandomFieldSource::new(rand::weak_rng());
    for _ in 0..NUM_ITEMS {
        DateOnly::serialize_components(r.year(), r.month(), r.day(), &mut v).unwrap();
    }

    b.bytes = v.len() as u64;

    b.iter(|| {
        let mut cursor = Cursor::new(v.as_slice());
        for _ in 0..NUM_ITEMS {
            DateOnly::deserialize(&mut cursor).unwrap();
        }
    })
}

#[bench]
fn deserialize_time_only_random(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * TimeOnly::max_serialized_size());

    let mut r = RandomFieldSource::new(rand::weak_rng());
    for _ in 0..NUM_ITEMS {
        TimeOnly::serialize_components(r.hour(), r.minute(), r.second(), &mut v).unwrap();
    }

    b.bytes = v.len() as u64;

    b.iter(|| {
        let mut cursor = Cursor::new(v.as_slice());
        for _ in 0..NUM_ITEMS {
            TimeOnly::deserialize(&mut cursor).unwrap();
        }
    })
}

#[bench]
fn deserialize_date_time_random(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTime::max_serialized_size());

    let mut r = RandomFieldSource::new(rand::weak_rng());
    for _ in 0..NUM_ITEMS {
        DateTime::serialize_components(r.year(), r.month(), r.day(), r.hour(), r.minute(),
                                       r.second(), &mut v).unwrap();
    }

    b.bytes = v.len() as u64;

    b.iter(|| {
        let mut cursor = Cursor::new(v.as_slice());
        for _ in 0..NUM_ITEMS {
            DateTime::deserialize(&mut cursor).unwrap();
        }
    })
}

#[bench]
fn deserialize_date_time_offset_random(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeOffset::max_serialized_size());

    let mut r = RandomFieldSource::new(rand::weak_rng());
    for _ in 0..NUM_ITEMS {
        DateTimeOffset::serialize_components(r.year(), r.month(), r.day(), r.hour(), r.minute(),
                                             r.second(), r.offset(), &mut v).unwrap();
    }

    b.bytes = v.len() as u64;

    b.iter(|| {
        let mut cursor = Cursor::new(v.as_slice());
        for _ in 0..NUM_ITEMS {
            DateTimeOffset::deserialize(&mut cursor).unwrap();
        }
    })
}

#[bench]
fn deserialize_date_time_subsecond_random(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeSubSecond::max_serialized_size());

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
            DateTimeSubSecond::deserialize(&mut cursor).unwrap();
        }
    })
}

#[bench]
fn deserialize_date_time_subsecond_offset_random(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeSubSecondOffset::max_serialized_size());

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
            DateTimeSubSecondOffset::deserialize(&mut cursor).unwrap();
        }
    })
}
