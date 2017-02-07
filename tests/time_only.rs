extern crate temporenc;

use temporenc::*;

use std::iter::once;
use std::io::Cursor;

#[test]
fn deser_time_all_missing() {
    let bytes = vec!(0xA1, 0xFF, 0xFF);
    let d = TimeOnly::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(None, d.hour());
    assert_eq!(None, d.minute());
    assert_eq!(None, d.second());

    let mut serialized = Vec::new();
    assert_eq!(d.serialized_size(), d.serialize(&mut serialized).unwrap());
    assert_eq!(d.serialized_size(), serialized.len());
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_time_none_missing() {
    let bytes = vec!(0xA1, 0x26, 0x4C);
    let d = TimeOnly::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());

    let mut serialized = Vec::new();
    assert_eq!(d.serialized_size(), d.serialize(&mut serialized).unwrap());
    assert_eq!(d.serialized_size(), serialized.len());
    assert_eq!(bytes, serialized);
}

#[test]
fn deser_time_wrong_tag() {
    let bytes = vec!(0xA3, 0xFF, 0xFF);
    assert_eq!(DeserializationError::IncorrectTypeTag,
        TimeOnly::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap_err());
}

#[test]
fn deser_time_too_short() {
    let bytes = vec!(0xA1, 0xFF);
    assert_eq!(DeserializationError::IoError,
        TimeOnly::deserialize(&mut Cursor::new(bytes.as_slice())).unwrap_err());
}

#[test]
fn time_roundtrip_struct() {
    let mut vec = Vec::new();

    for hour in once(None).chain((HOUR_MIN..(HOUR_MAX + 1)).map(|h| Some(h))) {
        for minute in once(None).chain((MINUTE_MIN..(MINUTE_MAX + 1)).map(|m| Some(m))) {
            for second in once(None).chain((SECOND_MIN..(SECOND_MAX + 1)).map(|s| Some(s))) {
                vec.clear();
                let new = TimeOnly::new(hour, minute, second).unwrap();
                let bytes_written = new.serialize(&mut vec).unwrap();
                assert_eq!(3, bytes_written);
                assert_eq!(3, new.serialized_size());
                assert_eq!(3, vec.len());
                let deser = TimeOnly::deserialize(&mut Cursor::new(vec.as_slice())).unwrap();

                assert_eq!(hour, deser.hour());
                assert_eq!(minute, deser.minute());
                assert_eq!(second, deser.second());

                assert_eq!(new, deser);
            };
        };
    }
}
