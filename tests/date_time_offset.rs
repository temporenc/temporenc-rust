extern crate temporenc;
extern crate rand;

mod common;

use std::iter::once;
use std::io::Cursor;
use temporenc::*;
use common::RandomFieldSource;

#[test]
fn deser_dto_all_missing() {
    let bytes: Vec<u8> = vec!(0xDF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF);
    let d = DateTimeOffset::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(None, d.year());
    assert_eq!(None, d.month());
    assert_eq!(None, d.day());
    assert_eq!(None, d.hour());
    assert_eq!(None, d.month());
    assert_eq!(None, d.second());
    assert_eq!(OffsetValue::None, d.offset());

    let mut serialized = Vec::new();
    assert_eq!(d.serialized_size(), d.serialize(&mut serialized).unwrap());
    assert_eq!(d.serialized_size(), serialized.len());
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_dto_none_missing() {
    let bytes = vec!(0xCF, 0x7E, 0x0E, 0x93, 0x26, 0x44);
    let d = DateTimeOffset::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(OffsetValue::UtcOffset(60), d.offset());

    let mut serialized = Vec::new();
    assert_eq!(d.serialized_size(), d.serialize(&mut serialized).unwrap());
    assert_eq!(d.serialized_size(), serialized.len());
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_dto_wrong_tag() {
    let bytes = vec!(0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF);
    assert_eq!(DeserializationError::IncorrectTypeTag,
    DateTimeOffset::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap_err());
}

#[test]
fn deser_dto_too_short() {
    let bytes = vec!(0xCF, 0xFF);
    assert_eq!(DeserializationError::IoError,
        DateTimeOffset::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap_err());
}

#[test]
fn roundtrip_dto_all_year_month_day() {
    let mut vec = Vec::new();
    let mut random_fields = RandomFieldSource::new(rand::weak_rng());

    for year in once(None).chain((YEAR_MIN..(YEAR_MAX + 1)).map(|y| Some(y))) {
        for month in once(None).chain((MONTH_MIN..(MONTH_MAX + 1)).map(|m| Some(m))) {
            for day in once(None).chain((DAY_MIN..(DAY_MAX + 1)).map(|d| Some(d))) {
                let hour = random_fields.hour();
                let minute = random_fields.minute();
                let second = random_fields.second();
                let offset = random_fields.offset();

                serialize_struct_and_check(year, month, day, hour, minute, second, offset, &mut vec);
            }
        }
    }
}

#[test]
fn roundtrip_components_dto_all_random() {
    let mut vec = Vec::new();

    let mut random_fields = RandomFieldSource::new(rand::weak_rng());

    for _ in 0..1_000_000 {
        let year = random_fields.year();
        let month = random_fields.month();
        let day = random_fields.day();
        let hour = random_fields.hour();
        let minute = random_fields.minute();
        let second = random_fields.second();
        let offset = random_fields.offset();
        serialize_struct_and_check(year, month, day, hour, minute, second, offset, &mut vec);
    }
}

#[test]
fn roundtrip_struct_dto_all_random() {
    let mut vec = Vec::new();

    let mut random_fields = RandomFieldSource::new(rand::weak_rng());

    for _ in 0..1_000_000 {
        let year = random_fields.year();
        let month = random_fields.month();
        let day = random_fields.day();
        let hour = random_fields.hour();
        let minute = random_fields.minute();
        let second = random_fields.second();
        let offset = random_fields.offset();
        serialize_struct_and_check(year, month, day, hour, minute, second, offset, &mut vec);
    }
}

fn serialize_struct_and_check(year: Option<u16>, month: Option<u8>, day: Option<u8>, hour: Option<u8>,
                                  minute: Option<u8>, second: Option<u8>, offset: OffsetValue,
                                  vec: &mut Vec<u8>) {
    vec.clear();
    let new = DateTimeOffset::new(year, month, day, hour, minute, second, offset)
        .unwrap();
    assert_eq!(6, new.serialized_size());
    assert_eq!(6, new.serialize(vec).unwrap());
    assert_eq!(6, vec.len());
    let deser = DateTimeOffset::deserialize(&mut Cursor::new(vec.as_slice())).unwrap();

    assert_eq!(year, deser.year());
    assert_eq!(month, deser.month());
    assert_eq!(day, deser.day());

    assert_eq!(hour, deser.hour());
    assert_eq!(minute, deser.minute());
    assert_eq!(second, deser.second());

    assert_eq!(offset, deser.offset());

    assert_eq!(new, deser);
}
