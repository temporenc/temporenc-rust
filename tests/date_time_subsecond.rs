extern crate temporenc;
extern crate rand;

mod common;

use std::iter::once;
use std::io::Cursor;
use temporenc::*;
use common::RandomFieldSource;

#[test]
fn deser_dts_all_missing() {
    let bytes = vec!(0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xC0);
    let d = DateTimeSubSecond::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(None, d.year());
    assert_eq!(None, d.month());
    assert_eq!(None, d.day());
    assert_eq!(None, d.hour());
    assert_eq!(None, d.minute());
    assert_eq!(None, d.second());
    assert_eq!(FractionalSecond::None, d.fractional_second());

    let mut serialized = Vec::new();
    assert_eq!(d.serialized_size(), d.serialize(&mut serialized).unwrap());
    assert_eq!(d.serialized_size(), serialized.len());
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_dts_all_no_subsec() {
    let bytes = vec!(0x77, 0xBF, 0x07, 0x49, 0x93, 0x00);
    let d = DateTimeSubSecond::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::None, d.fractional_second());

    let mut serialized = Vec::new();
    assert_eq!(d.serialized_size(), d.serialize(&mut serialized).unwrap());
    assert_eq!(d.serialized_size(), serialized.len());
    assert_eq!(bytes, serialized);
}


#[test]
fn deser_dts_all_ms() {
    let bytes = vec!(0x47, 0xBF, 0x07, 0x49, 0x93, 0x07, 0xB0);
    let d = DateTimeSubSecond::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::Milliseconds(123), d.fractional_second());

    let mut serialized = Vec::new();
    assert_eq!(d.serialized_size(), d.serialize(&mut serialized).unwrap());
    assert_eq!(d.serialized_size(), serialized.len());
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_dts_all_us() {
    let bytes = vec!(0x57, 0xBF, 0x07, 0x49, 0x93, 0x07, 0x89, 0x00);
    let d = DateTimeSubSecond::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::Microseconds(123456), d.fractional_second());

    let mut serialized = Vec::new();
    assert_eq!(d.serialized_size(), d.serialize(&mut serialized).unwrap());
    assert_eq!(d.serialized_size(), serialized.len());
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_dts_all_ns() {
    let bytes = vec!(0x67, 0xBF, 0x07, 0x49, 0x93, 0x07, 0x5B, 0xCD, 0x15);
    let d = DateTimeSubSecond::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::Nanoseconds(123456789), d.fractional_second());

    let mut serialized = Vec::new();
    assert_eq!(d.serialized_size(), d.serialize(&mut serialized).unwrap());
    assert_eq!(d.serialized_size(), serialized.len());
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_dts_too_short() {
    let bytes = vec!(0x77, 0xBF, 0x07, 0x49);
    assert_eq!(DeserializationError::IoError,
        DateTimeSubSecond::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap_err());
}

#[test]
fn deser_dts_wrong_type_tag() {
    let bytes = vec!(0xF7, 0xBF, 0x07, 0x49, 0x93, 0x00);
    assert_eq!(DeserializationError::IncorrectTypeTag,
    DateTimeSubSecond::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap_err());
}

#[test]
fn roundtrip_dts_all_year_month_day() {
    let mut vec = Vec::new();
    let mut random_fields = RandomFieldSource::new(rand::weak_rng());

    for year in once(None).chain((YEAR_MIN..(YEAR_MAX + 1)).map(|y| Some(y))) {
        for month in once(None).chain((MONTH_MIN..(MONTH_MAX + 1)).map(|m| Some(m))) {
            for day in once(None).chain((DAY_MIN..(DAY_MAX + 1)).map(|d| Some(d))) {
                let hour = random_fields.hour();
                let minute = random_fields.minute();
                let second = random_fields.second();
                let frac_second = random_fields.fractional_second();

                serialize_struct_and_check(year, month, day, hour, minute, second, frac_second, &mut vec);
            }
        }
    }
}

#[test]
fn roundtrip_dts_all_hour_minute_second() {
    let mut vec = Vec::new();
    let mut random_fields = RandomFieldSource::new(rand::weak_rng());

    let year = random_fields.year();
    let month = random_fields.month();
    let day = random_fields.day();
    let frac_second = random_fields.fractional_second();

    for hour in once(None).chain((HOUR_MIN..(HOUR_MAX + 1)).map(|h| Some(h))) {
        for minute in once(None).chain((MINUTE_MIN..(MINUTE_MAX + 1)).map(|m| Some(m))) {
            for second in once(None).chain((SECOND_MIN..(SECOND_MAX + 1)).map(|s| Some(s))) {
                serialize_struct_and_check(year, month, day, hour, minute, second, frac_second, &mut vec);
            }
        }
    }
}

#[test]
fn roundtrip_struct_dts_all_random() {
    let mut vec = Vec::new();
    let mut random_fields = RandomFieldSource::new(rand::weak_rng());

    for _ in 0..1_000_000 {
        let year = random_fields.year();
        let month = random_fields.month();
        let day = random_fields.day();
        let hour = random_fields.hour();
        let minute = random_fields.minute();
        let second = random_fields.second();
        let fractional_second = random_fields.fractional_second();
        serialize_struct_and_check(year, month, day, hour, minute, second,
                                       fractional_second, &mut vec);
    }
}

fn serialize_struct_and_check(year: Option<u16>, month: Option<u8>, day: Option<u8>, hour: Option<u8>,
                       minute: Option<u8>, second: Option<u8>, frac_second: FractionalSecond,
                       vec: &mut Vec<u8>) {

    vec.clear();
    let new = DateTimeSubSecond::new(year, month, day, hour, minute, second, frac_second)
        .unwrap();
    let expected_length = new.serialized_size();
    assert_eq!(expected_length, new.serialize(vec).unwrap());
    assert_eq!(expected_length, vec.len());
    let deser = DateTimeSubSecond::deserialize(&mut Cursor::new(vec.as_slice())).unwrap();

    assert_eq!(year, deser.year());
    assert_eq!(month, deser.month());
    assert_eq!(day, deser.day());

    assert_eq!(hour, deser.hour());
    assert_eq!(minute, deser.minute());
    assert_eq!(second, deser.second());

    assert_eq!(frac_second, deser.fractional_second());

    assert_eq!(new, deser);
}
