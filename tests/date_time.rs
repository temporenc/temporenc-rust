extern crate temporenc;
extern crate rand;

mod common;

use std::iter::once;
use std::io::Cursor;
use temporenc::*;
use common::RandomFieldSource;

#[test]
fn deser_dt_all_missing() {
    let bytes: Vec<u8> = vec!(0x3F, 0xFF, 0xFF, 0xFF, 0xFF);
    let d = DateTime::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(None, d.year());
    assert_eq!(None, d.month());
    assert_eq!(None, d.day());
    assert_eq!(None, d.hour());
    assert_eq!(None, d.month());
    assert_eq!(None, d.second());

    let mut serialized = Vec::new();
    assert_eq!(d.serialized_size(), d.serialize(&mut serialized).unwrap());
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_dt_none_missing() {
    let bytes = vec!(0x1E, 0xFC, 0x1D, 0x26, 0x4c);
    let d = DateTime::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());

    let mut serialized = Vec::new();
    assert_eq!(d.serialized_size(), d.serialize(&mut serialized).unwrap());
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_dt_wrong_tag() {
    let bytes = vec!(0xAF, 0xFF, 0xFF, 0xFF, 0xFF);
    assert_eq!(DeserializationError::IncorrectTypeTag,
               DateTime::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap_err());
}

#[test]
fn deser_dt_too_short() {
    let bytes = vec!(0x3F, 0x7E);
    assert_eq!(DeserializationError::IoError,
               DateTime::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap_err());
}


#[test]
fn roundtrip_dt_all_year_month_day() {
    let mut vec = Vec::new();

    let hour = Some(4);
    let minute = Some(5);
    let second = Some(6);

    for year in once(None).chain((YEAR_MIN..(YEAR_MAX + 1)).map(|y| Some(y))) {
        for month in once(None).chain((MONTH_MIN..(MONTH_MAX + 1)).map(|m| Some(m))) {
            for day in once(None).chain((DAY_MIN..(DAY_MAX + 1)).map(|d| Some(d))) {
                serialize_and_check(year, month, day, hour, minute, second, &mut vec);
            }
        }
    }
}

#[test]
fn roundtrip_dt_all_hour_minute_second() {
    let mut vec = Vec::new();

    let year = Some(8);
    let month = Some(9);
    let day = Some(10);

    for hour in once(None).chain((HOUR_MIN..(HOUR_MAX + 1)).map(|h| Some(h))) {
        for minute in once(None).chain((MINUTE_MIN..(MINUTE_MAX + 1)).map(|m| Some(m))) {
            for second in once(None).chain((SECOND_MIN..(SECOND_MAX + 1)).map(|s| Some(s))) {
                serialize_and_check(year, month, day, hour, minute, second, &mut vec);
            }
        }
    }
}

#[test]
fn roundtrip_dt_all_random() {
    let mut vec = Vec::new();

    let mut random_fields = RandomFieldSource::new(rand::weak_rng());

    for _ in 0..1_000_000 {
        let year = random_fields.year();
        let month = random_fields.month();
        let day = random_fields.day();
        let hour = random_fields.hour();
        let minute = random_fields.minute();
        let second = random_fields.second();
        serialize_and_check(year, month, day, hour, minute, second, &mut vec);
    }
}

fn serialize_and_check(year: Option<u16>, month: Option<u8>, day: Option<u8>, hour: Option<u8>,
                       minute: Option<u8>, second: Option<u8>, vec: &mut Vec<u8>) {
    vec.clear();
    assert_eq!(5, DateTime::serialize_components(year, month, day, hour, minute, second, vec).unwrap());
    let dt = DateTime::deserialize(&mut Cursor::new(vec.as_slice())).unwrap();

    assert_eq!(year, dt.year());
    assert_eq!(month, dt.month());
    assert_eq!(day, dt.day());

    assert_eq!(hour, dt.hour());
    assert_eq!(minute, dt.minute());
    assert_eq!(second, dt.second());
}
