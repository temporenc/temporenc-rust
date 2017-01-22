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
    /// UTC offset, if specified.
    /// The offset may be "elsewhere", meaning that this temporal value
    /// is not at UTC but the timezone is not specified here, or it may
    /// be specified as the number of 15-minute increments away from UTC
    /// plus 64 (to make it always positive). Thus, UTC would be 64 and
    /// UTC+2:00 would be 64 + 8 = 72.
    fn offset(&self) -> Option<OffsetData>;
}

pub enum OffsetData {
    SpecifiedElsewhere,
    UtcOffset(u8)
}

#[derive(Debug)]
pub struct DateOnly<'a> {
    // 3-bit tag, 12-bit year, 4-bit month, 5-bit day
    data: &'a [u8]
}

#[derive(Debug)]
pub struct TimeOnly<'a> {
    // 7-bit tag, 5-bit hour, 6-bit minute, 6-bit second
    data: &'a [u8]
}

pub struct DateTime {}

pub struct DateTimeOffset {}

pub struct DateTimeSecond {}

pub struct DateTimeSecondOffset {}

enum TypeTag {
    DateOnly,
    TimeOnly,
    DateTime,
    DateTimeOffset,
    DateTimeSubsecond,
    DateTimeSubsecondOffset
}

impl TypeTag {
    fn matches(&self, byte: u8) -> bool {
        let top_three_bits = 0b1110_0000;
        let top_two_bits = 0b1100_0000;
        let top_seven_bits = 0b1111_1110;
        match self {
            &TypeTag::DateOnly => byte & top_three_bits == 0b1000_0000,
            &TypeTag::TimeOnly => byte & top_seven_bits == 0b1010_0000,
            &TypeTag::DateTime => byte & top_two_bits == 0b0000_0000,
            &TypeTag::DateTimeOffset => byte & top_three_bits == top_two_bits,
            &TypeTag::DateTimeSubsecond => byte & top_two_bits == 0b0100_0000,
            &TypeTag::DateTimeSubsecondOffset => byte & top_three_bits == top_three_bits
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum DeserError {
    InputTooShort,
    WrongTag
}

impl<'a> DateOnly<'a> {
    pub fn from_slice(slice: &[u8]) -> Result<DateOnly, DeserError> {
        if slice.len() < 3 {
            return Err(DeserError::InputTooShort);
        }

        if !TypeTag::DateOnly.matches(slice[0]) {
            return Err(DeserError::WrongTag);
        }

        Ok(DateOnly {
            data: &slice[0..3]
        })
    }
}

impl<'a> Date for DateOnly<'a> {
    fn year(&self) -> Option<u16> {
        // bits 4-15
        let mut year = ((self.data[0] & 0x1F) as u16) << 7;
        year |= (self.data[1] as u16) >> 1;

        if year == 4095 {
            None
        } else {
            Some(year)
        }
    }

    fn month(&self) -> Option<u8> {
        // bits 16-19
        let mut month = (self.data[1] & 0x01) << 3;
        println!("month: {:02x}", month);
        month |= (self.data[2] & 0xE0) >> 5;
        println!("month: {:02x}", month);

        if month == 15 {
            None
        } else {
            Some(month + 1)
        }
    }

    fn day(&self) -> Option<u8> {
        // bits 20-24
        let day = self.data[2] & 0x1F;

        if day == 31 {
            None
        } else {
            Some(day + 1)
        }
    }
}

impl<'a> TimeOnly<'a> {

    pub fn from_slice(slice: &[u8]) -> Result<TimeOnly, DeserError> {
        if slice.len() < 3 {
            return Err(DeserError::InputTooShort);
        }

        if !TypeTag::TimeOnly.matches(slice[0]) {
            return Err(DeserError::WrongTag);
        }

        Ok(TimeOnly {
            data: &slice[0..3]
        })
    }}

impl<'a> Time for TimeOnly<'a> {
    fn hour(&self) -> Option<u8> {
        // bits 8-12
        let mut hour = self.data[0] << 4;
        hour |= (self.data[1] & 0xF0) >> 4;

        if hour == 31 {
            None
        } else {
            Some(hour)
        }
    }

    fn minute(&self) -> Option<u8> {
        // bits 13-18
        let mut minute = (self.data[1] & 0x0F) << 2;
        minute |= (self.data[2] & 0xC0) >> 6;

        if minute == 63 {
            None
        } else {
            Some(minute)
        }
    }

    fn second(&self) -> Option<u8> {
        // bits 19-24
        let seconds = self.data[2] & 0x3F;

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
        assert_eq!(DeserError::WrongTag, DateOnly::from_slice(bytes).unwrap_err());
    }

    #[test]
    fn deser_date_too_short() {
        let bytes = &[0xAF, 0xFF];
        assert_eq!(DeserError::InputTooShort, DateOnly::from_slice(bytes).unwrap_err());
    }

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
        assert_eq!(DeserError::WrongTag, TimeOnly::from_slice(bytes).unwrap_err());
    }

    #[test]
    fn deser_time_too_short() {
        let bytes = &[0xAF, 0xFF];
        assert_eq!(DeserError::InputTooShort, TimeOnly::from_slice(bytes).unwrap_err());
    }

}
