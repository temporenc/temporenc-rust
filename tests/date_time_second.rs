extern crate temporenc;
extern crate rand;

use temporenc::*;

use self::rand::Rng;
use self::rand::distributions::range::SampleRange;

use std::iter::once;

#[test]
fn deser_dts_all_missing() {
    let bytes = &[0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    let d = DateTimeSecond::from_slice(bytes).unwrap();
    assert_eq!(None, d.year());
    assert_eq!(None, d.month());
    assert_eq!(None, d.day());
    assert_eq!(None, d.hour());
    assert_eq!(None, d.minute());
    assert_eq!(None, d.second());
    assert_eq!(FractionalSecond::NoValue, d.fractional_second());
}

#[test]
fn deser_dts_all_no_subsec() {
    let bytes = &[0x77, 0xBF, 0x07, 0x49, 0x93, 0x00];
    let d = DateTimeSecond::from_slice(bytes).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::NoValue, d.fractional_second());
}


#[test]
fn deser_dts_all_ms() {
    let bytes = &[0x47, 0xBF, 0x07, 0x49, 0x93, 0x07, 0xB0];
    let d = DateTimeSecond::from_slice(bytes).unwrap();
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
    let bytes = &[0x57, 0xBF, 0x07, 0x49, 0x93, 0x07, 0x89, 0x00];
    let d = DateTimeSecond::from_slice(bytes).unwrap();
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
    let bytes = &[0x67, 0xBF, 0x07, 0x49, 0x93, 0x07, 0x5B, 0xCD, 0x15];
    let d = DateTimeSecond::from_slice(bytes).unwrap();
    assert_eq!(Some(1983), d.year());
    assert_eq!(Some(1), d.month());
    assert_eq!(Some(15), d.day());
    assert_eq!(Some(18), d.hour());
    assert_eq!(Some(25), d.minute());
    assert_eq!(Some(12), d.second());
    assert_eq!(FractionalSecond::Nanoseconds(123456789), d.fractional_second());
}

#[test]
fn roundtrip_dts_random() {
    let mut vec = Vec::new();

    let rounds = 10;

    for year in once(None).chain(range_iter(YEAR_MIN, YEAR_MAX + 1).take(rounds).map(|y| Some(y))) {
        for month in once(None).chain(range_iter(MONTH_MIN, MONTH_MAX + 1).take(rounds).map(|m| Some(m))) {
            for day in once(None).chain(range_iter(DAY_MIN, DAY_MAX + 1).take(rounds).map(|d| Some(d))) {
                for hour in once(None).chain(range_iter(HOUR_MIN, HOUR_MAX + 1).take(rounds).map(|h| Some(h))) {
                    for minute in once(None).chain(range_iter(MINUTE_MIN, MINUTE_MAX + 1).take(rounds).map(|m| Some(m))) {
                        for second in once(None).chain(range_iter(SECOND_MIN, SECOND_MAX + 1).take(rounds).map(|s| Some(s))) {
                            vec.clear();
                            assert_eq!(3, write_date_time_subsecond(year, month, day, hour, minute,
                                                                    second, &mut vec).unwrap());
                            let dts = DateTimeSecond::from_slice(&vec).unwrap();

                            assert_eq!(year, dts.year());
                            assert_eq!(month, dts.month());
                            assert_eq!(day, dts.day());

                            assert_eq!(hour, dts.hour());
                            assert_eq!(minute, dts.minute());
                            assert_eq!(second, dts.second());


                        };
                    };
                }
            };
        };
    }
}

fn range_iter<T: PartialOrd + SampleRange + Copy>(low: T, high: T) -> RngRangeIterator<T, self::rand::XorShiftRng> {
    RngRangeIterator {
        low: low,
        high: high,
        rng: rand::weak_rng()
    }
}

struct RngRangeIterator<T: PartialOrd + SampleRange + Copy, R: Rng> {
    low: T,
    high: T,
    rng: R
}

impl<T: PartialOrd + SampleRange + Copy, R: Rng> Iterator for RngRangeIterator<T, R> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        Some(self.rng.gen_range(self.low, self.high))
    }
}
