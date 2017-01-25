extern crate temporenc;
extern crate rand;

mod common;

use std::iter::once;
use temporenc::*;
use common::RandomFieldSource;


#[test]
fn deser_dtso_all_missing() {
    let bytes: Vec<u8> = vec!(0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xC0);
    let d = DateTimeSubSecondOffset::deserialize(bytes.as_slice()).unwrap();
    assert_eq!(None, d.year());
    assert_eq!(None, d.month());
    assert_eq!(None, d.day());
    assert_eq!(None, d.hour());
    assert_eq!(None, d.month());
    assert_eq!(None, d.second());
    assert_eq!(FractionalSecond::None, d.fractional_second());
    assert_eq!(OffsetValue::None, d.offset());

    let mut serialized = Vec::new();
    let _ = d.serialize(&mut serialized).unwrap();
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_dtso_all_no_subsec() {
    let bytes = vec!(0xFB, 0xDF, 0x83, 0xA4, 0xC9, 0x91, 0x00);
    let d = DateTimeSubSecondOffset::deserialize(bytes.as_slice()).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::None, d.fractional_second());
    assert_eq!(OffsetValue::UtcOffset(60), d.offset());

    let mut serialized = Vec::new();
    let _ = d.serialize(&mut serialized).unwrap();
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_dtso_all_ms() {
    let bytes = vec!(0xE3, 0xDF, 0x83, 0xA4, 0xC9, 0x83, 0xDC, 0x40);
    let d = DateTimeSubSecondOffset::deserialize(bytes.as_slice()).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::Milliseconds(123), d.fractional_second());
    assert_eq!(OffsetValue::UtcOffset(60), d.offset());

    let mut serialized = Vec::new();
    let _ = d.serialize(&mut serialized).unwrap();
    assert_eq!(bytes, serialized);
}


#[test]
fn deser_dtso_all_us() {
    let bytes = vec!(0xEB, 0xDF, 0x83, 0xA4, 0xC9, 0x83, 0xC4, 0x81, 0x10);
    let d = DateTimeSubSecondOffset::deserialize(bytes.as_slice()).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::Microseconds(123456), d.fractional_second());
    assert_eq!(OffsetValue::UtcOffset(60), d.offset());

    let mut serialized = Vec::new();
    let _ = d.serialize(&mut serialized).unwrap();
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_dtso_all_ns() {
    let bytes = vec!(0xF3, 0xDF, 0x83, 0xA4, 0xC9, 0x83, 0xAD, 0xE6, 0x8A, 0xC4);
    let d = DateTimeSubSecondOffset::deserialize(bytes.as_slice()).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::Nanoseconds(123456789), d.fractional_second());
    assert_eq!(OffsetValue::UtcOffset(60), d.offset());

    let mut serialized = Vec::new();
    let _ = d.serialize(&mut serialized).unwrap();
    assert_eq!(bytes, serialized);
}
#[test]
fn deser_dtso_wrong_tag() {
    let bytes = vec!(0x0F);
    assert_eq!(DeserializationError::IncorrectTypeTag,
    DateTimeSubSecondOffset::deserialize(bytes.as_slice()).unwrap_err());
}

#[test]
fn deser_dtso_too_short() {
    let bytes = vec!(0xFF, 0xFF);
    assert_eq!(DeserializationError::EarlyEOF,
    DateTimeSubSecondOffset::deserialize(bytes.as_slice()).unwrap_err());
}


#[test]
fn roundtrip_dtso_all_year_month_day() {
    let mut vec = Vec::new();

    for year in once(None).chain((YEAR_MIN..(YEAR_MAX + 1)).map(|y| Some(y))) {
        for month in once(None).chain((MONTH_MIN..(MONTH_MAX + 1)).map(|m| Some(m))) {
            for day in once(None).chain((DAY_MIN..(DAY_MAX + 1)).map(|d| Some(d))) {
                let hour = Some(4);
                let minute = Some(5);
                let second = Some(6);
                let frac_second = FractionalSecond::Microseconds(12345);
                let offset = OffsetValue::UtcOffset(45);

                serialize_and_check(year, month, day, hour, minute, second, frac_second, offset, &mut vec);
            }
        }
    }
}

#[test]
fn roundtrip_dtso_all_random() {
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
        let offset = random_fields.offset();
        serialize_and_check(year, month, day, hour, minute, second,
                            fractional_second, offset, &mut vec);
    }
}


fn serialize_and_check(year: Option<u16>, month: Option<u8>, day: Option<u8>, hour: Option<u8>,
                       minute: Option<u8>, second: Option<u8>, frac_second: FractionalSecond,
                       offset: OffsetValue, vec: &mut Vec<u8>) {
    vec.clear();
    assert_eq!(dtso_encoded_length(frac_second),
        DateTimeSubSecondOffset::serialize_components(year, month, day, hour, minute, second, frac_second,
                                                 offset, vec).unwrap());
    let dt = DateTimeSubSecondOffset::deserialize(vec.as_slice()).unwrap();

    assert_eq!(year, dt.year());
    assert_eq!(month, dt.month());
    assert_eq!(day, dt.day());

    assert_eq!(hour, dt.hour());
    assert_eq!(minute, dt.minute());
    assert_eq!(second, dt.second());

    assert_eq!(offset, dt.offset());
}


fn dtso_encoded_length(frac_second: FractionalSecond) -> usize {
    match frac_second {
        FractionalSecond::Milliseconds(_) => 8,
        FractionalSecond::Microseconds(_) => 9,
        FractionalSecond::Nanoseconds(_) => 10,
        FractionalSecond::None => 7,
    }
}
