extern crate temporenc;

use temporenc::*;

use std::iter::once;
use std::io::Cursor;

#[test]
fn deser_d_all_missing() {
    let bytes: Vec<u8> = vec!(0x9F, 0xFF, 0xFF);
    let d = DateOnly::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(None, d.year());
    assert_eq!(None, d.month());
    assert_eq!(None, d.day());

    let mut serialized = Vec::new();
    assert_eq!(d.serialized_size(), d.serialize(&mut serialized).unwrap());
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_d_none_missing() {
    let bytes = vec!(0x8F, 0x7E, 0x0E);
    let d = DateOnly::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());

    let mut serialized = Vec::new();
    assert_eq!(d.serialized_size(), d.serialize(&mut serialized).unwrap());
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_d_wrong_tag() {
    let bytes = vec!(0xAF, 0xFF, 0xFF);
    assert_eq!(DeserializationError::IncorrectTypeTag,
               DateOnly::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap_err());
}

#[test]
fn deser_d_too_short() {
    let bytes = vec!(0x8F, 0x7E);
    assert_eq!(DeserializationError::IoError,
               DateOnly::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap_err());
}

#[test]
fn date_roundtrip_components() {
    let mut vec = Vec::new();

    for year in once(None).chain((YEAR_MIN..(YEAR_MAX + 1)).map(|y| Some(y))) {
        for month in once(None).chain((MONTH_MIN..(MONTH_MAX + 1)).map(|m| Some(m))) {
            for day in once(None).chain((DAY_MIN..(DAY_MAX + 1)).map(|d| Some(d))) {
                vec.clear();
                let bytes_written = DateOnly::serialize_components(year, month, day, &mut vec).unwrap();
                assert_eq!(3, bytes_written);
                assert_eq!(bytes_written, vec.len());
                let deser = DateOnly::deserialize(&mut Cursor::new(vec.as_slice())).unwrap();

                assert_eq!(year, deser.year());
                assert_eq!(month, deser.month());
                assert_eq!(day, deser.day());
            };
        };
    }
}

#[test]
fn date_roundtrip_struct() {
    let mut vec = Vec::new();

    for year in once(None).chain((YEAR_MIN..(YEAR_MAX + 1)).map(|y| Some(y))) {
        for month in once(None).chain((MONTH_MIN..(MONTH_MAX + 1)).map(|m| Some(m))) {
            for day in once(None).chain((DAY_MIN..(DAY_MAX + 1)).map(|d| Some(d))) {
                vec.clear();
                let new_date = DateOnly::new(year, month, day).unwrap();
                let bytes_written = new_date.serialize(&mut vec).unwrap();
                assert_eq!(3, bytes_written);
                assert_eq!(bytes_written, vec.len());
                assert_eq!(new_date.serialized_size(), vec.len());
                let deser = DateOnly::deserialize(&mut Cursor::new(vec.as_slice())).unwrap();

                assert_eq!(year, deser.year());
                assert_eq!(month, deser.month());
                assert_eq!(day, deser.day());
            };
        };
    }
}

#[test]
fn date_serialize_struct_matches_components() {
    let mut vec_components = Vec::new();
    let mut vec_struct = Vec::new();

    for year in once(None).chain((YEAR_MIN..(YEAR_MAX + 1)).map(|y| Some(y))) {
        for month in once(None).chain((MONTH_MIN..(MONTH_MAX + 1)).map(|m| Some(m))) {
            for day in once(None).chain((DAY_MIN..(DAY_MAX + 1)).map(|d| Some(d))) {
                vec_components.clear();
                DateOnly::serialize_components(year, month, day, &mut vec_components).unwrap();

                vec_struct.clear();
                DateOnly::new(year, month, day).unwrap().serialize(&mut vec_struct).unwrap();

                assert_eq!(vec_components, vec_struct);
            };
        };
    }
}
