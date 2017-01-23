extern crate temporenc;

use temporenc::*;

use std::iter::once;

#[test]
fn deser_date_all_missing() {
    let bytes = &[0x9F, 0xFF, 0xFF];
    let d = DateOnly::from_slice(bytes).unwrap();
    assert_eq!(None, d.year());
    assert_eq!(None, d.month());
    assert_eq!(None, d.day());
}

#[test]
fn deser_date_none_missing() {
    let bytes = &[0x8F, 0x7E, 0x0E];
    let d = DateOnly::from_slice(bytes).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
}

#[test]
fn deser_date_wrong_tag() {
    let bytes = &[0xAF, 0xFF, 0xFF];
    assert_eq!(DeserializationError::IncorrectTypeTag, DateOnly::from_slice(bytes).unwrap_err());
}

#[test]
fn deser_date_too_short() {
    let bytes = &[0xAF, 0xFF];
    assert_eq!(DeserializationError::InputTooShort, DateOnly::from_slice(bytes).unwrap_err());
}

#[test]
fn date_roundtrip() {
    let mut vec = Vec::new();

    for year in once(None).chain((YEAR_MIN..(YEAR_MAX + 1)).map(|y| Some(y))) {
        for month in once(None).chain((MONTH_MIN..(MONTH_MAX + 1)).map(|m| Some(m))) {
            for day in once(None).chain((DAY_MIN..(DAY_MAX + 1)).map(|d| Some(d))) {
                vec.clear();
                assert_eq!(3, write_date(year, month, day, &mut vec).unwrap());
                let date = DateOnly::from_slice(&vec).unwrap();

                assert_eq!(year, date.year());
                assert_eq!(month, date.month());
                assert_eq!(day, date.day());
            };
        };
    }
}
