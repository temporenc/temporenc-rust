use std::io::Write;

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
    fn fractional_second(&self) -> FractionalSecond;
}

#[derive(Debug, PartialEq)]
pub enum FractionalSecond {
    Milliseconds(u16),
    Microseconds(u32),
    Nanoseconds(u32),
    NoValue
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
    // TTTY YYYY YYYY YYYM MMMD DDDD
    data: &'a [u8]
}

pub fn write_date<W: Write>(year: Option<u16>, month: Option<u8>, day: Option<u8>, writer: &mut W)
                            -> Result<usize, SerializationError> {
    check_outside_range(year, YEAR_MIN, YEAR_MAX, TemporalField::Year)?;
    check_outside_range(month, MONTH_MIN, MONTH_MAX, TemporalField::Month)?;
    check_outside_range(day, DAY_MIN, DAY_MAX, TemporalField::Day)?;

    let year_num = year.unwrap_or(YEAR_RAW_NONE);
    let month_num = month.map(|m| m - 1).unwrap_or(MONTH_RAW_NONE);
    let day_num = day.map(|d| d - 1).unwrap_or(DAY_RAW_NONE);

    let b1 = DATE_TAG | ((year_num >> 7) as u8);
    let mut bytes_written = write_map_err(b1, writer)?;
    let b2 = ((year_num << 1) as u8) | (month_num >> 3);
    bytes_written += write_map_err(b2, writer)?;
    let b3 = (month_num << 5) | day_num;
    bytes_written += write_map_err(b3, writer)?;

    Ok(bytes_written)
}

impl<'a> DateOnly<'a> {
    pub fn from_slice(slice: &[u8]) -> Result<DateOnly, DeserializationError> {
        if slice.len() < DATE_LEN {
            return Err(DeserializationError::InputTooShort);
        }

        if !TypeTag::DateOnly.matches(slice[0]) {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        Ok(DateOnly {
            data: &slice[0..DATE_LEN]
        })
    }

    pub fn serialize<W: Write>(&self, mut writer: W) -> std::io::Result<usize> {
        writer.write_all(self.data).map(|_| self.data.len())
    }
}

impl<'a> Date for DateOnly<'a> {
    fn year(&self) -> Option<u16> {
        // bits 4-15
        let mut year = ((self.data[0] & 0x1F) as u16) << 7;
        year |= (self.data[1] as u16) >> 1;

        if year == YEAR_RAW_NONE {
            None
        } else {
            Some(year)
        }
    }

    fn month(&self) -> Option<u8> {
        // bits 16-19
        let mut month = (self.data[1] & 0x01) << 3;
        //        println!("month: {:02x}", month);
        month |= (self.data[2] & 0xE0) >> 5;
        //        println!("month: {:02x}", month);

        if month == MONTH_RAW_NONE {
            None
        } else {
            Some(month + 1)
        }
    }

    fn day(&self) -> Option<u8> {
        // bits 20-24
        let day = self.data[2] & 0x1F;

        if day == DAY_RAW_NONE {
            None
        } else {
            Some(day + 1)
        }
    }
}

#[derive(Debug)]
pub struct TimeOnly<'a> {
    // 7-bit tag, 5-bit hour, 6-bit minute, 6-bit second
    // TTTT TTTH HHHH MMMM MMSS SSSS
    data: &'a [u8]
}

pub fn write_time<W: Write>(hour: Option<u8>, minute: Option<u8>, second: Option<u8>, writer: &mut W)
                            -> Result<usize, SerializationError> {
    check_outside_range(hour, HOUR_MIN, HOUR_MAX, TemporalField::Hour)?;
    check_outside_range(minute, MINUTE_MIN, MINUTE_MAX, TemporalField::Minute)?;
    check_outside_range(second, SECOND_MIN, SECOND_MAX, TemporalField::Second)?;

    let hour_num = hour.unwrap_or(HOUR_RAW_NONE);
    let minute_num = minute.unwrap_or(MINUTE_RAW_NONE);
    let second_num = second.unwrap_or(SECOND_RAW_NONE);

    let b1 = TIME_TAG | hour_num >> 4;
    let mut bytes_written = write_map_err(b1, writer)?;
    let b2 = (hour_num << 4) | (minute_num >> 2);
    bytes_written += write_map_err(b2, writer)?;
    let b3 = (minute_num << 6) | (second_num);
    bytes_written += write_map_err(b3, writer)?;

    Ok(bytes_written)
}

impl<'a> TimeOnly<'a> {
    pub fn from_slice(slice: &[u8]) -> Result<TimeOnly, DeserializationError> {
        if slice.len() < TIME_LEN {
            return Err(DeserializationError::InputTooShort);
        }

        if !TypeTag::TimeOnly.matches(slice[0]) {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        Ok(TimeOnly {
            data: &slice[0..TIME_LEN]
        })
    }
}

impl<'a> Time for TimeOnly<'a> {
    fn hour(&self) -> Option<u8> {
        // bits 8-12
        let mut hour = self.data[0] << 4;
        hour |= (self.data[1] & 0xF0) >> 4;

        if hour == HOUR_RAW_NONE {
            None
        } else {
            Some(hour)
        }
    }

    fn minute(&self) -> Option<u8> {
        // bits 13-18
        let mut minute = (self.data[1] & 0x0F) << 2;
        minute |= (self.data[2] & 0xC0) >> 6;

        if minute == MINUTE_RAW_NONE {
            None
        } else {
            Some(minute)
        }
    }

    fn second(&self) -> Option<u8> {
        // bits 19-24
        let seconds = self.data[2] & 0x3F;

        if seconds == SECOND_RAW_NONE {
            None
        } else {
            Some(seconds)
        }
    }
}

pub struct DateTime {}

pub struct DateTimeOffset {}

pub struct DateTimeSecond<'a> {
    // 2-bit tag, 2-bit subsecond precision tag, 12-bit year, 4-bit month, 5-bit day, 5-bit hour,
    // 6-bit minute, 6-bit second, and 0, 10, 20, or 30-bit subsecond value (as V in bit diagram)
    // TTPP YYYY | YYYY YYYY | MMMM DDDD | DHHH HHMM
    // MMMM SSSS | SSVV VVVV | [0, 1, 2, or 3 subsecond bytes]
    data: &'a [u8],
    precision: PrecisionTag
}

pub fn write_date_time_subsecond<W: Write>(year: Option<u16>, month: Option<u8>, day: Option<u8>,
                                           hour: Option<u8>, minute: Option<u8>, second: Option<u8>,
                                           writer: &mut W)
                                           -> Result<usize, SerializationError> {
    check_outside_range(year, YEAR_MIN, YEAR_MAX, TemporalField::Year)?;
    check_outside_range(month, MONTH_MIN, MONTH_MAX, TemporalField::Month)?;
    check_outside_range(day, DAY_MIN, DAY_MAX, TemporalField::Day)?;
    check_outside_range(hour, HOUR_MIN, HOUR_MAX, TemporalField::Hour)?;
    check_outside_range(minute, MINUTE_MIN, MINUTE_MAX, TemporalField::Minute)?;
    check_outside_range(second, SECOND_MIN, SECOND_MAX, TemporalField::Second)?;

    let year_num = year.unwrap_or(YEAR_RAW_NONE);
    let month_num = month.map(|m| m - 1).unwrap_or(MONTH_RAW_NONE);
    let day_num = day.map(|d| d - 1).unwrap_or(DAY_RAW_NONE);
    let hour_num = hour.unwrap_or(HOUR_RAW_NONE);
    let minute_num = minute.unwrap_or(MINUTE_RAW_NONE);
    let second_num = second.unwrap_or(SECOND_RAW_NONE);

    // TODO

    Ok(0)
}

impl<'a> DateTimeSecond<'a> {
    pub fn from_slice(slice: &[u8]) -> Result<DateTimeSecond, DeserializationError> {
        if slice.len() < 1 {
            return Err(DeserializationError::InputTooShort);
        }

        let first_byte = slice[0];
        if !TypeTag::DateTimeSubSecond.matches(first_byte) {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        let precision;
        let len;
        match first_byte & 0x30 {
            0 => {
                precision = PrecisionTag::Milli;
                len = 7;
            }
            0b0001_0000 => {
                precision = PrecisionTag::Micro;
                len = 8;
            }
            0b0010_0000 => {
                precision = PrecisionTag::Nano;
                len = 9;
            }
            0b0011_0000 => {
                precision = PrecisionTag::NoValue;
                len = 6;
            }
            _ => {
                return Err(DeserializationError::IncorrectPrecisionTag);
            }
        }

        if slice.len() < len {
            return Err(DeserializationError::InputTooShort);
        }

        Ok(DateTimeSecond {
            data: &slice[0..len],
            precision: precision
        })
    }
}


impl<'a> Date for DateTimeSecond<'a> {
    fn year(&self) -> Option<u16> {
        // bits 5-16
        let mut year = ((self.data[0] & 0x0F) as u16) << 8;
        year |= self.data[1] as u16;

        if year == YEAR_RAW_NONE {
            None
        } else {
            Some(year)
        }
    }

    fn month(&self) -> Option<u8> {
        // bits 17-20
        let month = self.data[2] >> 4;

        if month == MONTH_RAW_NONE {
            None
        } else {
            Some(month + 1)
        }
    }

    fn day(&self) -> Option<u8> {
        // bits 21-25
        let day = ((self.data[2] & 0x0F) << 1) | (self.data[3] >> 7);

        if day == DAY_RAW_NONE {
            None
        } else {
            Some(day + 1)
        }
    }
}

impl<'a> Time for DateTimeSecond<'a> {
    fn hour(&self) -> Option<u8> {
        // bits 26-30
        let hour = (self.data[3] & 0x7C) >> 2;

        if hour == HOUR_RAW_NONE {
            None
        } else {
            Some(hour)
        }
    }

    fn minute(&self) -> Option<u8> {
        // bits 31-36
        let minute = ((self.data[3] & 0x03) << 4) | ((self.data[4] & 0xF0) >> 4);

        if minute == MINUTE_RAW_NONE {
            None
        } else {
            Some(minute)
        }
    }

    fn second(&self) -> Option<u8> {
        // bits 37-42
        let seconds = ((self.data[4] & 0x0F) << 2) | ((self.data[5] & 0xC0) >> 6);

        if seconds == SECOND_RAW_NONE {
            None
        } else {
            Some(seconds)
        }
    }
}

impl<'a> SubSecond for DateTimeSecond<'a> {
    fn fractional_second(&self) -> FractionalSecond {
        match self.precision {
            PrecisionTag::NoValue => FractionalSecond::NoValue,
            PrecisionTag::Milli => {
                // bits 44-52
                let mut ms = ((self.data[5] & 0x3F) as u16) << 4;
                ms |= (self.data[6] >> 4) as u16;

                FractionalSecond::Milliseconds(ms)
            }
            PrecisionTag::Micro => {
                // bits 44-62
                let mut us = ((self.data[5] & 0x3F) as u32) << 14;
                us |= (self.data[6] as u32) << 6;
                us |= ((self.data[7] & 0xFC) >> 2) as u32;

                FractionalSecond::Microseconds(us)
            }
            PrecisionTag::Nano => {
                // bits 44-72
                let mut ns = ((self.data[5] & 0x3F) as u32) << 24;
                ns |= (self.data[6] as u32) << 16;
                ns |= (self.data[7] as u32) << 8;
                ns |= self.data[8] as u32;

                FractionalSecond::Nanoseconds(ns)
            }
        }
    }
}

pub struct DateTimeSecondOffset {}

enum TypeTag {
    DateOnly,
    TimeOnly,
    DateTime,
    DateTimeOffset,
    DateTimeSubSecond,
    DateTimeSubSecondOffset
}

impl TypeTag {
    fn matches(&self, byte: u8) -> bool {
        let top_three_bits = 0b1110_0000;
        let top_two_bits = 0b1100_0000;
        let top_seven_bits = 0b1111_1110;
        match self {
            &TypeTag::DateOnly => byte & top_three_bits == DATE_TAG,
            &TypeTag::TimeOnly => byte & top_seven_bits == TIME_TAG,
            &TypeTag::DateTime => byte & top_two_bits == DATE_TIME_TAG,
            &TypeTag::DateTimeOffset => byte & top_three_bits == DATE_TIME_OFFSET_TAG,
            &TypeTag::DateTimeSubSecond => byte & top_two_bits == DATE_TIME_SUBSECOND_TAG,
            &TypeTag::DateTimeSubSecondOffset => byte & top_three_bits == DATE_TIME_SUBSECOND_OFFSET_TAG
        }
    }
}

#[derive(Debug, PartialEq)]
enum PrecisionTag {
    Milli,
    Micro,
    Nano,
    NoValue
}

#[derive(Debug, PartialEq)]
pub enum DeserializationError {
    InputTooShort,
    IncorrectTypeTag,
    IncorrectPrecisionTag
}

#[derive(Debug, PartialEq)]
pub enum SerializationError {
    FieldOutOfRange(TemporalField),
    IoError
}

#[derive(Debug, PartialEq)]
pub enum TemporalField {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
    Offset
}

// tags, expanded to include the rest of the byte
// 3 bits
const DATE_TAG: u8 = 0b1000_0000;
// 7 bits
const TIME_TAG: u8 = 0b1010_0000;
// 2 bits
const DATE_TIME_TAG: u8 = 0b0000_0000;
// 3 bits
const DATE_TIME_OFFSET_TAG: u8 = 0b1100_0000;
// 2 bits
const DATE_TIME_SUBSECOND_TAG: u8 = 0b0100_0000;
// 3 bits
const DATE_TIME_SUBSECOND_OFFSET_TAG: u8 = 0b1110_0000;

const DATE_LEN: usize = 3;
const TIME_LEN: usize = 3;

// encoded forms of "no value"
const YEAR_RAW_NONE: u16 = 4095;
const MONTH_RAW_NONE: u8 = 15;
const DAY_RAW_NONE: u8 = 31;
const HOUR_RAW_NONE: u8 = 31;
const MINUTE_RAW_NONE: u8 = 63;
const SECOND_RAW_NONE: u8 = 63;

// human-visible range ends (not necessarily internal encoding)
pub const YEAR_MIN: u16 = 0;
pub const YEAR_MAX: u16 = 4094;
pub const MONTH_MIN: u8 = 1;
pub const MONTH_MAX: u8 = 12;
pub const DAY_MIN: u8 = 1;
pub const DAY_MAX: u8 = 31;
pub const HOUR_MIN: u8 = 0;
pub const HOUR_MAX: u8 = 23;
pub const MINUTE_MIN: u8 = 0;
pub const MINUTE_MAX: u8 = 60;
pub const SECOND_MIN: u8 = 0;
pub const SECOND_MAX: u8 = 60;

fn write_map_err<W: Write>(byte: u8, writer: &mut W) -> Result<usize, SerializationError> {
    writer.write(&[byte]).map_err(|_| SerializationError::IoError)
}

fn check_outside_range<T: PartialOrd>(val: Option<T>, min: T, max: T, field: TemporalField)
                                      -> Result<(), SerializationError> {
    if let Some(v) = val {
        if v < min || v > max {
            return Err(SerializationError::FieldOutOfRange(field))
        }
    }

    Ok(())
}
