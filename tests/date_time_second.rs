extern crate rand;

use self::rand::Rng;
use self::rand::distributions::range::SampleRange;

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
