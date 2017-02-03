#![feature(test)]

extern crate temporenc;
extern crate test;
extern crate rand;

#[allow(dead_code)]
mod common;

use test::Bencher;
use common::{NUM_ITEMS, RandomFieldSource};
use temporenc::*;

#[bench]
fn serialize_random_struct_date(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateOnly::max_serialized_size());
    b.bytes = v.capacity() as u64;
    let mut r = RandomFieldSource::new(rand::weak_rng());
    b.iter(|| {
        let year = r.year();
        let month = r.month();
        let day = r.day();
        let d = DateOnly::new(year, month, day).unwrap();
        for _ in 0..NUM_ITEMS {
            d.serialize(&mut v).unwrap();
        }
        v.clear();
    })
}

#[bench]
fn serialize_random_struct_time(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * TimeOnly::max_serialized_size());
    b.bytes = v.capacity() as u64;
    let mut r = RandomFieldSource::new(rand::weak_rng());

    b.iter(|| {
        let hour = r.hour();
        let minute = r.minute();
        let second = r.second();
        let d = TimeOnly::new(hour, minute, second).unwrap();
        for _ in 0..NUM_ITEMS {
            d.serialize(&mut v).unwrap();
        };
        v.clear();
    });
}

#[bench]
fn serialize_random_struct_date_time(b: &mut Bencher) {
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
        let d = DateTime::new(year, month, day, hour, minute, second).unwrap();
        for _ in 0..NUM_ITEMS {
            d.serialize(&mut v).unwrap();
        }
        v.clear();
    })
}

#[bench]
fn serialize_random_struct_date_time_offset(b: &mut Bencher) {
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
        let d = DateTimeOffset::new(year, month, day, hour, minute, second, offset)
            .unwrap();
        for _ in 0..NUM_ITEMS {
            d.serialize(&mut v).unwrap();
        };
        v.clear();
    })
}

#[bench]
fn serialize_random_struct_date_time_subsecond(b: &mut Bencher) {
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
        let d = DateTimeSubSecond::new(year, month, day, hour, minute, second,
                                       frac_second).unwrap();
        for _ in 0..NUM_ITEMS {
            d.serialize(&mut v).unwrap();
        };
        v.clear();
    })
}

#[bench]
fn serialize_random_struct_date_time_subsecond_offset(b: &mut Bencher) {
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
        let d = DateTimeSubSecondOffset::new(year, month, day, hour, minute, second,
                                             frac_second, offset).unwrap();
        for _ in 0..NUM_ITEMS {
            d.serialize(&mut v).unwrap();
        };
        v.clear();
    })
}
