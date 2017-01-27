extern crate temporenc;
extern crate rand;
extern crate test;

use self::rand::Rng;
use self::rand::distributions::range::SampleRange;
use temporenc::*;

pub const NUM_ITEMS: usize = 100;

// copied from integration tests
pub struct RandomFieldSource<R: Rng> {
    rng: R,
}

impl<R: Rng> RandomFieldSource<R> {
    pub fn new(rng: R) -> RandomFieldSource<R> {
        RandomFieldSource {
            rng: rng
        }
    }

    pub fn year(&mut self) -> Option<u16> {
        self.none_or_range(YEAR_MIN, YEAR_MAX + 1)
    }
    pub fn month(&mut self) -> Option<u8> {
        self.none_or_range(MONTH_MIN, MONTH_MAX + 1)
    }
    pub fn day(&mut self) -> Option<u8> {
        self.none_or_range(DAY_MIN, DAY_MAX + 1)
    }

    pub fn hour(&mut self) -> Option<u8> {
        self.none_or_range(HOUR_MIN, HOUR_MAX + 1)
    }
    pub fn minute(&mut self) -> Option<u8> {
        self.none_or_range(MINUTE_MIN, MINUTE_MAX + 1)
    }
    pub fn second(&mut self) -> Option<u8> {
        self.none_or_range(SECOND_MIN, SECOND_MAX + 1)
    }

    pub fn fractional_second(&mut self) -> FractionalSecond {
        bb(match self.rng.gen_range(0, 4) {
            0 => FractionalSecond::None,
            1 => FractionalSecond::Milliseconds(self.rng.gen_range(MILLIS_MIN, MILLIS_MAX + 1)),
            2 => FractionalSecond::Microseconds(self.rng.gen_range(MICROS_MIN, MICROS_MAX + 1)),
            3 => FractionalSecond::Nanoseconds(self.rng.gen_range(NANOS_MIN, NANOS_MAX + 1)),
            _ => panic!("Impossible!")
        })
    }

    pub fn offset(&mut self) -> OffsetValue {
        bb(match self.rng.gen_range(0, 3) {
            0 => OffsetValue::None,
            1 => OffsetValue::SpecifiedElsewhere,
            2 => OffsetValue::UtcOffset(self.rng.gen_range(OFFSET_MIN / 15, OFFSET_MAX / 15 + 1) * 15),
            _ => panic!("Impossible!")
        })
    }

    fn none_or_range<T: PartialOrd + SampleRange>(&mut self, low: T, high: T) -> Option<T> {
        // arbitrarily using 10% of the time
        if self.rng.gen_range(0, 100) < 10 {
            bb(None)
        } else {
            bb(Some(self.rng.gen_range(low, high)))
        }
    }
}

#[inline]
pub fn bb<T>(t: T) -> T {
    test::black_box(t)
}
