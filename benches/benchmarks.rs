#![feature(test)]

extern crate temporenc;
extern crate test;
extern crate rand;

use std::io::{Cursor};
use self::rand::Rng;
use self::rand::distributions::range::SampleRange;
use test::Bencher;
use temporenc::*;

const NUM_ITEMS: usize = 1000;

#[bench]
fn serialize_date_fixed(b: &mut Bencher) {
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
fn serialize_date_random(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * DateOnly::max_serialized_size());
    b.bytes = v.capacity() as u64;
    let mut r = RandomFieldSource::new(rand::weak_rng());
    b.iter(|| {
        for _ in 0..NUM_ITEMS {
            DateOnly::serialize_components(r.year(), r.month(), r.day(), &mut v).unwrap();
        }
        v.clear();
    })
}

#[bench]
fn serialize_time_fixed(b: &mut Bencher) {
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
fn serialize_time_random(b: &mut Bencher) {
    let mut v: Vec<u8> = Vec::with_capacity(NUM_ITEMS * TimeOnly::max_serialized_size());
    b.bytes = v.capacity() as u64;
    let mut r = RandomFieldSource::new(rand::weak_rng());
    b.iter(|| {
        for _ in 0..NUM_ITEMS {
            TimeOnly::serialize_components(r.hour(), r.minute(), r.second(), &mut v).unwrap();
        }
        v.clear();
    })
}

#[bench]
fn serialize_date_time_fixed(b: &mut Bencher) {
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
fn serialize_date_time_offset_fixed(b: &mut Bencher) {
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
fn serialize_date_time_subsecond_ns_fixed(b: &mut Bencher) {
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
fn serialize_date_time_subsecond_ns_offset_fixed(b: &mut Bencher) {
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

// copied from integration tests
struct RandomFieldSource<R: Rng> {
    rng: R,
}

impl<R: Rng> RandomFieldSource<R> {
    pub fn new(rng: R) -> RandomFieldSource<R> {
        RandomFieldSource {
            rng: rng
        }
    }

    pub fn year(&mut self) -> Option<u16> {
        self.none_or_range(YEAR_MIN, YEAR_MAX + 1)
    }
    pub fn month(&mut self) -> Option<u8> {
        self.none_or_range(MONTH_MIN, MONTH_MAX + 1)
    }
    pub fn day(&mut self) -> Option<u8> {
        self.none_or_range(DAY_MIN, DAY_MAX + 1)
    }

    pub fn hour(&mut self) -> Option<u8> {
        self.none_or_range(HOUR_MIN, HOUR_MAX + 1)
    }
    pub fn minute(&mut self) -> Option<u8> {
        self.none_or_range(MINUTE_MIN, MINUTE_MAX + 1)
    }
    pub fn second(&mut self) -> Option<u8> {
        self.none_or_range(SECOND_MIN, SECOND_MAX + 1)
    }
    //
    //    pub fn fractional_second(&mut self) -> FractionalSecond {
    //        match self.rng.gen_range(0, 4) {
    //            0 => FractionalSecond::None,
    //            1 => FractionalSecond::Milliseconds(self.rng.gen_range(MILLIS_MIN, MILLIS_MAX + 1)),
    //            2 => FractionalSecond::Microseconds(self.rng.gen_range(MICROS_MIN, MICROS_MAX + 1)),
    //            3 => FractionalSecond::Nanoseconds(self.rng.gen_range(NANOS_MIN, NANOS_MAX + 1)),
    //            _ => panic!("Impossible!")
    //        }
    //    }
    //
    //    pub fn offset(&mut self) -> OffsetValue {
    //        match self.rng.gen_range(0, 3) {
    //            0 => OffsetValue::None,
    //            1 => OffsetValue::SpecifiedElsewhere,
    //            2 => OffsetValue::UtcOffset(self.rng.gen_range(OFFSET_MIN / 15, OFFSET_MAX / 15 + 1) * 15),
    //            _ => panic!("Impossible!")
    //        }
    //    }

    fn none_or_range<T: PartialOrd + SampleRange>(&mut self, low: T, high: T) -> Option<T> {
        // arbitrarily using 10% of the time
        if self.rng.gen_range(0, 100) < 10 {
            None
        } else {
            Some(self.rng.gen_range(low, high))
        }
    }
}
