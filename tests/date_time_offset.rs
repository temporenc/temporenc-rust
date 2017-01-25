extern crate temporenc;

mod common;

use std::iter::once;

use temporenc::*;

use common::range_iter;



#[test]
fn deser_dto_all_missing() {
    let bytes: Vec<u8> = vec!(0xDF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF);
    let d = DateTimeOffset::deserialize(bytes.as_slice()).unwrap();
    assert_eq!(None, d.year());
    assert_eq!(None, d.month());
    assert_eq!(None, d.day());
    assert_eq!(None, d.hour());
    assert_eq!(None, d.month());
    assert_eq!(None, d.second());
    assert_eq!(OffsetValue::None, d.offset());
}

#[test]
fn deser_dto_none_missing() {
    // TODO offset calculation
    let bytes = vec!(0xCF, 0x7E, 0x0E, 0x93, 0x26, 0x44);
    let d = DateTimeOffset::deserialize(bytes.as_slice()).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(OffsetValue::UtcOffset(60), d.offset());
}

#[test]
fn deser_dto_wrong_tag() {
    let bytes = vec!(0xFF);
    assert_eq!(DeserializationError::IncorrectTypeTag,
    DateTimeOffset::deserialize(bytes.as_slice()).unwrap_err());
}

#[test]
fn deser_dto_too_short() {
    let bytes = vec!(0xCF, 0xFF);
    assert_eq!(DeserializationError::EarlyEOF,
    DateTimeOffset::deserialize(bytes.as_slice()).unwrap_err());
}


#[test]
fn roundtrip_dto_all_year_month_day() {
    let mut vec = Vec::new();

    let hour = Some(4);
    let minute = Some(5);
    let second = Some(6);
    let offset = OffsetValue::UtcOffset(45);

    for year in once(None).chain((YEAR_MIN..(YEAR_MAX + 1)).map(|y| Some(y))) {
        for month in once(None).chain((MONTH_MIN..(MONTH_MAX + 1)).map(|m| Some(m))) {
            for day in once(None).chain((DAY_MIN..(DAY_MAX + 1)).map(|d| Some(d))) {
                serialize_and_check(year, month, day, hour, minute, second, offset, &mut vec);
            }
        }
    }
}

#[test]
fn roundtrip_dto_all_hour_minute_second() {
    let mut vec = Vec::new();

    let year = Some(8);
    let month = Some(9);
    let day = Some(10);

    for hour in once(None).chain((HOUR_MIN..(HOUR_MAX + 1)).map(|h| Some(h))) {
        for minute in once(None).chain((MINUTE_MIN..(MINUTE_MAX + 1)).map(|m| Some(m))) {
            for second in once(None).chain((SECOND_MIN..(SECOND_MAX + 1)).map(|s| Some(s))) {
                for offset in once(OffsetValue::None)
                    .chain(once(OffsetValue::SpecifiedElsewhere))
                    .chain(((OFFSET_MIN / 15)..((OFFSET_MAX + 1) / 15))
                        .map(|o| OffsetValue::UtcOffset(o * 15))) {
                    serialize_and_check(year, month, day, hour, minute, second, offset, &mut vec);
                }
            }
        }
    }
}

#[test]
fn roundtrip_dto_all_random() {
    let mut vec = Vec::new();

    let rounds = 5;

    // try a whole bunch of random values, plus the None value, in each field

    for year in once(None).chain(range_iter(YEAR_MIN, YEAR_MAX + 1)
        .take(rounds).map(|y| Some(y))) {
        for month in once(None).chain(range_iter(MONTH_MIN, MONTH_MAX + 1)
            .take(rounds).map(|m| Some(m))) {
            for day in once(None).chain(range_iter(DAY_MIN, DAY_MAX + 1)
                .take(rounds).map(|d| Some(d))) {
                for hour in once(None).chain(range_iter(HOUR_MIN, HOUR_MAX + 1)
                    .take(rounds).map(|h| Some(h))) {
                    for minute in once(None).chain(range_iter(MINUTE_MIN, MINUTE_MAX + 1)
                        .take(rounds).map(|m| Some(m))) {
                        for second in once(None).chain(range_iter(SECOND_MIN, SECOND_MAX + 1)
                            .take(rounds).map(|s| Some(s))) {
                            for offset in once(OffsetValue::None)
                                .chain(once(OffsetValue::SpecifiedElsewhere))
                                .chain(range_iter(OFFSET_MIN / 15, (OFFSET_MAX + 1) / 15)
                                    .take(rounds)
                                    .map(|o| OffsetValue::UtcOffset(o * 15))) {
                                    serialize_and_check(year, month, day, hour, minute, second, offset, &mut vec);
                                }
                        };
                    };
                }
            };
        };
    }
}


fn serialize_and_check(year: Option<u16>, month: Option<u8>, day: Option<u8>, hour: Option<u8>,
                       minute: Option<u8>, second: Option<u8>, offset: OffsetValue,
                       vec: &mut Vec<u8>) {
    vec.clear();
    assert_eq!(6, DateTimeOffset::serialize(year, month, day, hour, minute, second, offset, vec)
        .unwrap());
    let dt = DateTimeOffset::deserialize(vec.as_slice()).unwrap();

    assert_eq!(year, dt.year());
    assert_eq!(month, dt.month());
    assert_eq!(day, dt.day());

    assert_eq!(hour, dt.hour());
    assert_eq!(minute, dt.minute());
    assert_eq!(second, dt.second());

    assert_eq!(offset, dt.offset());
}
