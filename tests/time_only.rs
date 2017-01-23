extern crate temporenc;

use temporenc::*;

use std::iter::once;


#[test]
fn deser_time_all_missing() {
    let bytes = &[0xA1, 0xFF, 0xFF];
    let t = TimeOnly::from_slice(bytes).unwrap();
    assert_eq!(None, t.hour());
    assert_eq!(None, t.minute());
    assert_eq!(None, t.second());
}

#[test]
fn deser_time_none_missing() {
    let bytes = &[0xA1, 0x26, 0x4C];
    let t = TimeOnly::from_slice(bytes).unwrap();
    assert_eq!(Some(18), t.hour());
    assert_eq!(Some(25), t.minute());
    assert_eq!(Some(12), t.second());
}

#[test]
fn deser_time_wrong_tag() {
    let bytes = &[0xA3, 0xFF, 0xFF];
    assert_eq!(DeserializationError::WrongTag, TimeOnly::from_slice(bytes).unwrap_err());
}

#[test]
fn deser_time_too_short() {
    let bytes = &[0xAF, 0xFF];
    assert_eq!(DeserializationError::InputTooShort, TimeOnly::from_slice(bytes).unwrap_err());
}

#[test]
fn time_roundtrip() {
    let mut vec = Vec::new();

    for hour in once(None).chain((HOUR_MIN..(HOUR_MAX + 1)).map(|y| Some(y))) {
        for minute in once(None).chain((MINUTE_MIN..(MINUTE_MAX + 1)).map(|y| Some(y))) {
            for second in once(None).chain((SECOND_MIN..(SECOND_MAX + 1)).map(|y| Some(y))) {
                vec.clear();
                assert_eq!(3, write_time(hour, minute, second, &mut vec).unwrap());
                let time = TimeOnly::from_slice(&vec).unwrap();

                assert_eq!(hour, time.hour());
                assert_eq!(minute, time.minute());
                assert_eq!(second, time.second());
            };
        };
    }
}
