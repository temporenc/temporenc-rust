#![feature(test)]

extern crate temporenc;
extern crate test;
extern crate rand;

mod common;

use test::Bencher;
use common::{NUM_ITEMS, RandomFieldSource};
use temporenc::*;

#[bench]
fn serialize_random_date(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateOnly::max_serialized_size());
    b.bytes = v.capacity() as u64;
    let mut r = RandomFieldSource::new(rand::weak_rng());
    b.iter(|| {
        let year = r.year();
        let month = r.month();
        let day = r.day();
        for _ in 0..NUM_ITEMS {
            DateOnly::serialize_components(year, month, day, &mut v).unwrap();
        }
        v.clear();
    })
}

#[bench]
fn serialize_random_time(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * TimeOnly::max_serialized_size());
    b.bytes = v.capacity() as u64;
    let mut r = RandomFieldSource::new(rand::weak_rng());
    b.iter(|| {
        let hour = r.hour();
        let minute = r.minute();
        let second = r.second();
        for _ in 0..NUM_ITEMS {
            TimeOnly::serialize_components(hour, minute, second, &mut v).unwrap();
        }
        v.clear();
    })
}

#[bench]
fn serialize_random_date_time(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTime::max_serialized_size());
    b.bytes = v.capacity() as u64;
    let mut r = RandomFieldSource::new(rand::weak_rng());
    b.iter(|| {
        let year = r.year();
        let month = r.month();
        let day = r.day();
        let hour = r.hour();
        let minute = r.minute();
        let second = r.second();
        for _ in 0..NUM_ITEMS {
            DateTime::serialize_components(year, month, day, hour, minute, second, &mut v).unwrap();
        }
        v.clear();
    })
}

#[bench]
fn serialize_random_date_time_offset(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeOffset::max_serialized_size());
    b.bytes = v.capacity() as u64;
    let mut r = RandomFieldSource::new(rand::weak_rng());
    b.iter(|| {
        let year = r.year();
        let month = r.month();
        let day = r.day();
        let hour = r.hour();
        let minute = r.minute();
        let second = r.second();
        let offset = r.offset();
        for _ in 0..NUM_ITEMS {
            DateTimeOffset::serialize_components(year, month, day, hour, minute, second, offset,
                                                 &mut v).unwrap();
        };
        v.clear();
    })
}

#[bench]
fn serialize_random_date_time_subsecond(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeSubSecond::max_serialized_size());
    let mut r = RandomFieldSource::new(rand::weak_rng());
    b.iter(|| {
        let year = r.year();
        let month = r.month();
        let day = r.day();
        let hour = r.hour();
        let minute = r.minute();
        let second = r.second();
        let frac_second = r.fractional_second();
        for _ in 0..NUM_ITEMS {
            DateTimeSubSecond::serialize_components(year, month, day, hour, minute, second,
                                                          frac_second, &mut v).unwrap();
        };
        v.clear();
    })
}

#[bench]
fn serialize_random_date_time_subsecond_offset(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateTimeSubSecondOffset::max_serialized_size());
    let mut r = RandomFieldSource::new(rand::weak_rng());
    b.iter(|| {
        let year = r.year();
        let month = r.month();
        let day = r.day();
        let hour = r.hour();
        let minute = r.minute();
        let second = r.second();
        let offset = r.offset();
        let frac_second = r.fractional_second();
        for _ in 0..NUM_ITEMS {
            DateTimeSubSecondOffset::serialize_components(year, month, day, hour, minute, second,
                                                          frac_second, offset, &mut v).unwrap();
        };
        v.clear();
    })
}
