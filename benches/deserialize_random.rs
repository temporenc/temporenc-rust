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

    let mut buf = Vec::new();
    for _ in 0..DateOnly::max_serialized_size() {
        buf.push(0);
    }

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

    let mut buf = Vec::new();
    for _ in 0..TimeOnly::max_serialized_size() {
        buf.push(0);
    }

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

    let mut buf = Vec::new();
    for _ in 0..DateTime::max_serialized_size() {
        buf.push(0);
    }

    b.iter(|| {
        let mut cursor = Cursor::new(v.as_slice());
        for _ in 0..NUM_ITEMS {
            DateTime::deserialize(&mut cursor).unwrap();
        }
    })
}

