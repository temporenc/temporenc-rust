pub trait Date {
    /// If present, the year. In range [0, 4094].
    fn year(&self) -> Option<u16>;
    /// If present, the month. In range [1, 12].
    fn month(&self) -> Option<u8>;
    /// If present, the day. In range [1, 31].
    fn day(&self) -> Option<u8>;
}

pub trait Time {
    /// If present, the number of hours. In range [0, 23].
    fn hour(&self) -> Option<u8>;
    /// If present, the number of minutes. In range [0, 59].
    fn minute(&self) -> Option<u8>;
    /// If present, the number of seconds. In range [0, 60].
    fn second(&self) -> Option<u8>;
}

pub trait SubSecond {
    /// Milliseconds of fractional second, if specified.
    /// Other fractional second precisions may be specified if this is None.
    fn ms(&self) -> Option<u16>;
    /// Microseconds of fractional second, if specified.
    /// Other fractional second precisions may be specified if this is None.
    fn us(&self) -> Option<u32>;
    /// Nanoseconds of fractional second, if specified.
    /// Other fractional second precisions may be specified if this is None.
    fn ns(&self) -> Option<u32>;
}

pub trait Offset {
    fn offset(&self) -> Option<u8>;
}

pub struct DateOnly<'a> {
    data: &'a [u8; 3]
}

pub struct TimeOnly<'a> {
    data: &'a [u8; 3]
}

pub struct DateTime {}

pub struct DateTimeOffset {}

pub struct DateTimeSecond {}

pub struct DateTimeSecondOffset {}

impl<'a> Date for DateOnly<'a> {
    fn year(&self) -> Option<u16> {
        // first 12 BE bits
        let mut year = (self.data[0] as u16) << 4;
        year |= (self.data[1] as u16) >> 4;
        println!("year: {:X}", year);

        if year == 4095 {
            None
        } else {
            Some(year)
        }
    }

    fn month(&self) -> Option<u8> {
        // bits 13-16
        let month = self.data[1] & 0x0F;

        if month == 15 {
            None
        } else {
            Some(month + 1)
        }
    }

    fn day(&self) -> Option<u8> {
        let day = self.data[2] >> 3;

        if day == 31 {
            None
        } else {
            Some(day + 1)
        }
    }
}

impl<'a> Time for TimeOnly<'a> {
    fn hour(&self) -> Option<u8> {
        // first 5 bits
        let hour = self.data[0] >> 3;

        if hour == 31 {
            None
        } else {
            Some(hour)
        }
    }

    fn minute(&self) -> Option<u8> {
        // bits 6-11: bottom 3 of byte 0, top 3 of byte 1
        let mut minute = (self.data[0] & 0x07) << 3;
        minute |= self.data[1] >> 5;

        if minute == 63 {
            None
        } else {
            Some(minute)
        }
    }

    fn second(&self) -> Option<u8> {
        // bits 12-17: bottom 5 of byte 1, top 1 of byte 2
        let mut seconds = (self.data[1] & 0x1F) << 1;
        seconds |= self.data[2] >> 7;

        if seconds == 63 {
            None
        } else {
            Some(seconds)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deser_date_all_missing() {
        let bytes = &[0xFF, 0xFF, 0xFF];
        let d = DateOnly { data: bytes };
        assert_eq!(None, d.year());
        assert_eq!(None, d.month());
        assert_eq!(None, d.day());
    }

    #[test]
    fn deser_date_none_missing() {
        let bytes = &[0x7B, 0xF0, 0x70];
        let d = DateOnly { data: bytes };
        assert_eq!(Some(1983), d.year());
        assert_eq!(Some(1), d.month());
        assert_eq!(Some(15), d.day());
    }

    #[test]
    fn deser_time_all_missing() {
        let bytes = &[0xFF, 0xFF, 0xFF];
        let t = TimeOnly{ data: bytes };
        assert_eq!(None, t.hour());
        assert_eq!(None, t.minute());
        assert_eq!(None, t.second());
    }

    #[test]
    fn deser_time_none_missing() {
        let bytes = &[0x93, 0x26, 0x00];
        let t = TimeOnly { data: bytes };
        assert_eq!(Some(18), t.hour());
        assert_eq!(Some(25), t.minute());
        assert_eq!(Some(12), t.second());
    }
}
