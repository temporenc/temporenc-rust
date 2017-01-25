extern crate temporenc;

mod common;

use std::iter::once;

use temporenc::*;

use common::range_iter;

#[test]
fn deser_dts_all_missing() {
    let bytes = vec!(0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF);
    let d = DateTimeSubSecond::deserialize(bytes.as_slice()).unwrap();
    assert_eq!(None, d.year());
    assert_eq!(None, d.month());
    assert_eq!(None, d.day());
    assert_eq!(None, d.hour());
    assert_eq!(None, d.minute());
    assert_eq!(None, d.second());
    assert_eq!(FractionalSecond::None, d.fractional_second());
}

#[test]
fn deser_dts_all_no_subsec() {
    let bytes = vec!(0x77, 0xBF, 0x07, 0x49, 0x93, 0x00);
    let d = DateTimeSubSecond::deserialize(bytes.as_slice()).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::None, d.fractional_second());
}


#[test]
fn deser_dts_all_ms() {
    let bytes = vec!(0x47, 0xBF, 0x07, 0x49, 0x93, 0x07, 0xB0);
    let d = DateTimeSubSecond::deserialize(bytes.as_slice()).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::Milliseconds(123), d.fractional_second());
}

#[test]
fn deser_dts_all_us() {
    let bytes = vec!(0x57, 0xBF, 0x07, 0x49, 0x93, 0x07, 0x89, 0x00);
    let d = DateTimeSubSecond::deserialize(bytes.as_slice()).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::Microseconds(123456), d.fractional_second());
}

#[test]
fn deser_dts_all_ns() {
    let bytes = vec!(0x67, 0xBF, 0x07, 0x49, 0x93, 0x07, 0x5B, 0xCD, 0x15);
    let d = DateTimeSubSecond::deserialize(bytes.as_slice()).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::Nanoseconds(123456789), d.fractional_second());
}

#[test]
fn roundtrip_dts_all_random() {
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
                            for frac_second in once(FractionalSecond::None)
                                .chain(range_iter(MILLIS_MIN, MILLIS_MAX + 1).take(rounds).map(|s| FractionalSecond::Milliseconds(s)))
                                .chain(range_iter(MICROS_MIN, MICROS_MAX + 1).take(rounds).map(|s| FractionalSecond::Microseconds(s)))
                                .chain(range_iter(NANOS_MIN, NANOS_MAX + 1).take(rounds).map(|s| FractionalSecond::Nanoseconds(s)))
                                {
                                    serialize_and_check(year, month, day, hour, minute, second, frac_second, &mut vec);
                                }
                        };
                    };
                }
            };
        };
    }
}

#[test]
fn roundtrip_dts_all_year_month_day() {
    let mut vec = Vec::new();

    let hour = Some(4);
    let minute = Some(5);
    let second = Some(6);
    let frac_second = FractionalSecond::Milliseconds(7);

    for year in once(None).chain((YEAR_MIN..(YEAR_MAX + 1)).map(|y| Some(y))) {
        for month in once(None).chain((MONTH_MIN..(MONTH_MAX + 1)).map(|m| Some(m))) {
            for day in once(None).chain((DAY_MIN..(DAY_MAX + 1)).map(|d| Some(d))) {
                serialize_and_check(year, month, day, hour, minute, second, frac_second, &mut vec);
            }
        }
    }
}

#[test]
fn roundtrip_dts_all_hour_minute_second() {
    let mut vec = Vec::new();

    let year = Some(8);
    let month = Some(9);
    let day = Some(10);
    let frac_second = FractionalSecond::Microseconds(987654);

    for hour in once(None).chain((HOUR_MIN..(HOUR_MAX + 1)).map(|h| Some(h))) {
        for minute in once(None).chain((MINUTE_MIN..(MINUTE_MAX + 1)).map(|m| Some(m))) {
            for second in once(None).chain((SECOND_MIN..(SECOND_MAX + 1)).map(|s| Some(s))) {
                serialize_and_check(year, month, day, hour, minute, second, frac_second, &mut vec);
            }
        }
    }
}


#[test]
fn roundtrip_dts_random_fractional() {
    let mut vec = Vec::new();

    let year = Some(8);
    let month = Some(9);
    let day = Some(10);
    let hour = Some(11);
    let minute = Some(12);
    let second = Some(13);

    let rounds = 200_000;

    for frac_second in once(FractionalSecond::None)
        .chain(range_iter(MILLIS_MIN, MILLIS_MAX + 1).take(rounds).map(|s| FractionalSecond::Milliseconds(s)))
        .chain(range_iter(MICROS_MIN, MICROS_MAX + 1).take(rounds).map(|s| FractionalSecond::Microseconds(s)))
        .chain(range_iter(NANOS_MIN, NANOS_MAX + 1).take(rounds).map(|s| FractionalSecond::Nanoseconds(s))) {
        serialize_and_check(year, month, day, hour, minute, second, frac_second, &mut vec);
    }
}

fn dts_encoded_length(frac_second: FractionalSecond) -> usize {
    match frac_second {
        FractionalSecond::Milliseconds(_) => 7,
        FractionalSecond::Microseconds(_) => 8,
        FractionalSecond::Nanoseconds(_) => 9,
        FractionalSecond::None => 6,
    }
}

fn serialize_and_check(year: Option<u16>, month: Option<u8>, day: Option<u8>, hour: Option<u8>,
                       minute: Option<u8>, second: Option<u8>, frac_second: FractionalSecond,
                       vec: &mut Vec<u8>) {
    let expected_length = dts_encoded_length(frac_second);

    vec.clear();
    assert_eq!(expected_length,
        DateTimeSubSecond::serialize(year, month, day, hour, minute, second, frac_second, vec).unwrap());
    let dts = DateTimeSubSecond::deserialize(vec.as_slice()).unwrap();

    assert_eq!(year, dts.year());
    assert_eq!(month, dts.month());
    assert_eq!(day, dts.day());

    assert_eq!(hour, dts.hour());
    assert_eq!(minute, dts.minute());
    assert_eq!(second, dts.second());

    assert_eq!(frac_second, dts.fractional_second());
}
