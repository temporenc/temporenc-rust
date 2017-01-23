use std::io::{Read, Write, Bytes};
use std::iter::Iterator;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FractionalSecond {
    Milliseconds(u16),
    Microseconds(u32),
    Nanoseconds(u32),
    None
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
pub struct DateOnly {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>
}

impl DateOnly {
    pub fn deserialize<R: Read>(reader: R) -> Result<DateOnly, DeserializationError> {
        let mut bytes = reader.bytes();
        let byte0 = next_byte(&mut bytes)?;

        if !TypeTag::DateOnly.matches(byte0) {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        // 3-bit tag, 12-bit year, 4-bit month, 5-bit day
        // TTTY YYYY YYYY YYYM MMMD DDDD

        // bits 4-15
        let mut raw_year = ((byte0 & 0x1F) as u16) << 7;
        let byte1 = next_byte(&mut bytes)?;
        raw_year |= (byte1 as u16) >> 1;

        let year = if raw_year == YEAR_RAW_NONE {
            None
        } else {
            Some(raw_year)
        };

        // bits 16-19
        let mut raw_month = (byte1 & 0x01) << 3;
        let byte2 = next_byte(&mut bytes)?;
        raw_month |= (byte2 & 0xE0) >> 5;

        let month = if raw_month == MONTH_RAW_NONE {
            None
        } else {
            Some(raw_month + 1)
        };

        // bits 20-24
        let raw_day = byte2 & 0x1F;

        let day = if raw_day == DAY_RAW_NONE {
            None
        } else {
            Some(raw_day + 1)
        };

        Ok(DateOnly {
            year: year,
            month: month,
            day: day
        })
    }

    pub fn serialize<W: Write>(year: Option<u16>, month: Option<u8>, day: Option<u8>, writer: &mut W)
                               -> Result<usize, SerializationError> {
        check_option_outside_range(year, YEAR_MIN, YEAR_MAX, TemporalField::Year)?;
        check_option_outside_range(month, MONTH_MIN, MONTH_MAX, TemporalField::Month)?;
        check_option_outside_range(day, DAY_MIN, DAY_MAX, TemporalField::Day)?;

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
}

impl Date for DateOnly {
    fn year(&self) -> Option<u16> {
        self.year
    }

    fn month(&self) -> Option<u8> {
        self.month
    }

    fn day(&self) -> Option<u8> {
        self.day
    }
}

#[derive(Debug)]
pub struct TimeOnly {
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
}

impl TimeOnly {
    pub fn deserialize<R: Read>(reader: R) -> Result<TimeOnly, DeserializationError> {
        let mut bytes = reader.bytes();

        let byte0 = next_byte(&mut bytes)?;
        if !TypeTag::TimeOnly.matches(byte0) {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        // 7-bit tag, 5-bit hour, 6-bit minute, 6-bit second
        // TTTT TTTH HHHH MMMM MMSS SSSS

        // bits 8-12
        let mut raw_hour = byte0 << 4;
        let byte1 = next_byte(&mut bytes)?;
        raw_hour |= (byte1 & 0xF0) >> 4;

        let hour = if raw_hour == HOUR_RAW_NONE {
            None
        } else {
            Some(raw_hour)
        };

        // bits 13-18
        let mut raw_minute = (byte1 & 0x0F) << 2;
        let byte2 = next_byte(&mut bytes)?;
        raw_minute |= (byte2 & 0xC0) >> 6;

        let minute = if raw_minute == MINUTE_RAW_NONE {
            None
        } else {
            Some(raw_minute)
        };

        // bits 19-24
        let raw_second = byte2 & 0x3F;

        let second = if raw_second == SECOND_RAW_NONE {
            None
        } else {
            Some(raw_second)
        };

        Ok(TimeOnly {
            hour: hour,
            minute: minute,
            second: second
        })
    }

    pub fn serialize<W: Write>(hour: Option<u8>, minute: Option<u8>, second: Option<u8>, writer: &mut W)
                               -> Result<usize, SerializationError> {
        check_option_outside_range(hour, HOUR_MIN, HOUR_MAX, TemporalField::Hour)?;
        check_option_outside_range(minute, MINUTE_MIN, MINUTE_MAX, TemporalField::Minute)?;
        check_option_outside_range(second, SECOND_MIN, SECOND_MAX, TemporalField::Second)?;

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
}

impl Time for TimeOnly {
    fn hour(&self) -> Option<u8> {
        self.hour
    }

    fn minute(&self) -> Option<u8> {
        self.minute
    }

    fn second(&self) -> Option<u8> {
        self.second
    }
}

#[derive(Debug)]
pub struct DateTime {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
}

impl DateTime {
    pub fn deserialize<R: Read>(reader: R) -> Result<DateTime, DeserializationError> {
        let mut bytes = reader.bytes();
        let byte0 = next_byte(&mut bytes)?;

        if !TypeTag::DateTime.matches(byte0) {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        let precision = match byte0 & PRECISION_DTS_MASK {
            PRECISION_DTS_MILLIS_TAG => PrecisionTag::Milli,
            PRECISION_DTS_MICROS_TAG => PrecisionTag::Micro,
            PRECISION_DTS_NANOS_TAG => PrecisionTag::Nano,
            PRECISION_DTS_NONE_TAG => PrecisionTag::None,
            _ => {
                return Err(DeserializationError::IncorrectPrecisionTag);
            }
        };

        // 2-bit tag, 12-bit year, 4-bit month, 5-bit day, 5-bit hour, 6-bit minute, 6-bit second
        // TTYY YYYY | YYYY YYMM | MMDD DDDH | HHHH MMMM | MMSS SSSS

        // bits 3-14
        let byte1 = next_byte(&mut bytes)?;
        let mut raw_year = ((byte0 & 0x3F) as u16) << 6;
        raw_year |= (byte1 >> 2) as u16;
        let year = if raw_year == YEAR_RAW_NONE {
            None
        } else {
            Some(raw_year)
        };

        // bits 15-18
        let byte2 = next_byte(&mut bytes)?;
        let raw_month = ((byte1 & 0x03) << 2) | (byte2 >> 6);
        let month = if raw_month == MONTH_RAW_NONE {
            None
        } else {
            Some(raw_month + 1)
        };

        // bits 19-23
        let raw_day = (byte2 & 0x3E) >> 1;
        let day = if raw_day == DAY_RAW_NONE {
            None
        } else {
            Some(raw_day + 1)
        };

        // bits 24-28
        let byte3 = next_byte(&mut bytes)?;
        let raw_hour = ((byte2 & 0x01) << 4) | (byte3 >> 4);
        let hour = if raw_hour == HOUR_RAW_NONE {
            None
        } else {
            Some(raw_hour)
        };

        // bits 29-34
        let byte4 = next_byte(&mut bytes)?;
        let raw_minute = ((byte3 & 0x0F) << 2) | (byte4 >> 6);
        let minute = if raw_minute == MINUTE_RAW_NONE {
            None
        } else {
            Some(raw_minute)
        };

        // bits 35-40
        let raw_second = byte4 & 0x3F;
        let second = if raw_second == SECOND_RAW_NONE {
            None
        } else {
            Some(raw_second)
        };

        Ok(DateTime {
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
        })
    }

    pub fn serialize<W: Write>(year: Option<u16>, month: Option<u8>, day: Option<u8>,
                               hour: Option<u8>, minute: Option<u8>, second: Option<u8>,
                               writer: &mut W)
                               -> Result<usize, SerializationError> {
        check_option_outside_range(year, YEAR_MIN, YEAR_MAX, TemporalField::Year)?;
        check_option_outside_range(month, MONTH_MIN, MONTH_MAX, TemporalField::Month)?;
        check_option_outside_range(day, DAY_MIN, DAY_MAX, TemporalField::Day)?;
        check_option_outside_range(hour, HOUR_MIN, HOUR_MAX, TemporalField::Hour)?;
        check_option_outside_range(minute, MINUTE_MIN, MINUTE_MAX, TemporalField::Minute)?;
        check_option_outside_range(second, SECOND_MIN, SECOND_MAX, TemporalField::Second)?;

        let year_num = year.unwrap_or(YEAR_RAW_NONE);
        let month_num = month.map(|m| m - 1).unwrap_or(MONTH_RAW_NONE);
        let day_num = day.map(|d| d - 1).unwrap_or(DAY_RAW_NONE);
        let hour_num = hour.unwrap_or(HOUR_RAW_NONE);
        let minute_num = minute.unwrap_or(MINUTE_RAW_NONE);
        let second_num = second.unwrap_or(SECOND_RAW_NONE);

        let mut bytes_written = write_map_err(DATE_TIME_TAG | (year_num >> 6) as u8, writer)?;
        bytes_written += write_map_err(((year_num << 2) as u8) | (month_num >> 2), writer)?;
        bytes_written += write_map_err((month_num << 6) | (day_num << 1) | (hour_num >> 4), writer)?;
        bytes_written += write_map_err((hour_num << 4) | (minute_num >> 2), writer)?;
        bytes_written += write_map_err((minute_num << 6) | second_num, writer)?;

        Ok(bytes_written)
    }
}

impl Date for DateTime {
    fn year(&self) -> Option<u16> {
        self.year
    }

    fn month(&self) -> Option<u8> {
        self.month
    }

    fn day(&self) -> Option<u8> {
        self.day
    }
}

impl Time for DateTime {
    fn hour(&self) -> Option<u8> {
        self.hour
    }

    fn minute(&self) -> Option<u8> {
        self.minute
    }

    fn second(&self) -> Option<u8> {
        self.second
    }
}

pub struct DateTimeOffset {}

pub struct DateTimeSubSecond {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
    frac_second: FractionalSecond
}

impl DateTimeSubSecond {
    pub fn deserialize<R: Read>(reader: R) -> Result<DateTimeSubSecond, DeserializationError> {
        let mut bytes = reader.bytes();
        let byte0 = next_byte(&mut bytes)?;

        if !TypeTag::DateTimeSubSecond.matches(byte0) {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        let precision = match byte0 & PRECISION_DTS_MASK {
            PRECISION_DTS_MILLIS_TAG => PrecisionTag::Milli,
            PRECISION_DTS_MICROS_TAG => PrecisionTag::Micro,
            PRECISION_DTS_NANOS_TAG => PrecisionTag::Nano,
            PRECISION_DTS_NONE_TAG => PrecisionTag::None,
            _ => {
                return Err(DeserializationError::IncorrectPrecisionTag);
            }
        };

        // 2-bit tag, 2-bit subsecond precision tag, 12-bit year, 4-bit month, 5-bit day, 5-bit hour,
        // 6-bit minute, 6-bit second, and 0, 10, 20, or 30-bit subsecond value (as V in bit diagram)
        // TTPP YYYY | YYYY YYYY | MMMM DDDD | DHHH HHMM
        // MMMM SSSS | SSVV VVVV | [0, 1, 2, or 3 subsecond bytes]

        // bits 5-16
        let byte1 = next_byte(&mut bytes)?;
        let mut raw_year = ((byte0 & 0x0F) as u16) << 8;
        raw_year |= byte1 as u16;

        let year = if raw_year == YEAR_RAW_NONE {
            None
        } else {
            Some(raw_year)
        };

        // bits 17-20
        let byte2 = next_byte(&mut bytes)?;
        let raw_month = byte2 >> 4;

        let month = if raw_month == MONTH_RAW_NONE {
            None
        } else {
            Some(raw_month + 1)
        };

        // bits 21-25
        let byte3 = next_byte(&mut bytes)?;
        let raw_day = ((byte2 & 0x0F) << 1) | (byte3 >> 7);

        let day = if raw_day == DAY_RAW_NONE {
            None
        } else {
            Some(raw_day + 1)
        };

        // bits 26-30
        let raw_hour = (byte3 & 0x7C) >> 2;

        let hour = if raw_hour == HOUR_RAW_NONE {
            None
        } else {
            Some(raw_hour)
        };

        // bits 31-36
        let byte4 = next_byte(&mut bytes)?;
        let raw_minute = ((byte3 & 0x03) << 4) | ((byte4 & 0xF0) >> 4);

        let minute = if raw_minute == MINUTE_RAW_NONE {
            None
        } else {
            Some(raw_minute)
        };

        // bits 37-42
        let byte5 = next_byte(&mut bytes)?;
        let raw_second = ((byte4 & 0x0F) << 2) | ((byte5 & 0xC0) >> 6);

        let second = if raw_second == SECOND_RAW_NONE {
            None
        } else {
            Some(raw_second)
        };

        let frac_second = match precision {
            PrecisionTag::None => FractionalSecond::None,
            PrecisionTag::Milli => {
                // bits 43-52
                let mut ms = ((byte5 & 0x3F) as u16) << 4;
                ms |= (next_byte(&mut bytes)? >> 4) as u16;

                FractionalSecond::Milliseconds(ms)
            }
            PrecisionTag::Micro => {
                // bits 43-62
                let mut us = ((byte5 & 0x3F) as u32) << 14;
                us |= (next_byte(&mut bytes)? as u32) << 6;
                us |= ((next_byte(&mut bytes)? & 0xFC) >> 2) as u32;

                FractionalSecond::Microseconds(us)
            }
            PrecisionTag::Nano => {
                // bits 43-72
                let mut ns = ((byte5 & 0x3F) as u32) << 24;
                ns |= (next_byte(&mut bytes)? as u32) << 16;
                ns |= (next_byte(&mut bytes)? as u32) << 8;
                ns |= next_byte(&mut bytes)? as u32;

                FractionalSecond::Nanoseconds(ns)
            }
        };

        Ok(DateTimeSubSecond {
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
            frac_second: frac_second
        })
    }

    pub fn serialize<W: Write>(year: Option<u16>, month: Option<u8>, day: Option<u8>,
                               hour: Option<u8>, minute: Option<u8>, second: Option<u8>,
                               fractional_second: FractionalSecond, writer: &mut W)
                               -> Result<usize, SerializationError> {
        check_option_outside_range(year, YEAR_MIN, YEAR_MAX, TemporalField::Year)?;
        check_option_outside_range(month, MONTH_MIN, MONTH_MAX, TemporalField::Month)?;
        check_option_outside_range(day, DAY_MIN, DAY_MAX, TemporalField::Day)?;
        check_option_outside_range(hour, HOUR_MIN, HOUR_MAX, TemporalField::Hour)?;
        check_option_outside_range(minute, MINUTE_MIN, MINUTE_MAX, TemporalField::Minute)?;
        check_option_outside_range(second, SECOND_MIN, SECOND_MAX, TemporalField::Second)?;

        let (precision_tag, first_subsecond_byte_fragment) = match fractional_second {
            FractionalSecond::None => (PRECISION_DTS_NONE_TAG, 0x0),
            FractionalSecond::Milliseconds(ms) => {
                check_outside_range(ms, MILLIS_MIN, MILLIS_MAX, TemporalField::FractionalSecond)?;
                (PRECISION_DTS_MILLIS_TAG, (ms >> 4) as u8)
            },
            FractionalSecond::Microseconds(us) => {
                check_outside_range(us, MICROS_MIN, MICROS_MAX, TemporalField::FractionalSecond)?;
                (PRECISION_DTS_MICROS_TAG, (us >> 14) as u8)
            },
            FractionalSecond::Nanoseconds(ns) => {
                check_outside_range(ns, NANOS_MIN, NANOS_MAX, TemporalField::FractionalSecond)?;
                (PRECISION_DTS_NANOS_TAG, (ns >> 24) as u8)
            }
        };

        let year_num = year.unwrap_or(YEAR_RAW_NONE);
        let month_num = month.map(|m| m - 1).unwrap_or(MONTH_RAW_NONE);
        let day_num = day.map(|d| d - 1).unwrap_or(DAY_RAW_NONE);
        let hour_num = hour.unwrap_or(HOUR_RAW_NONE);
        let minute_num = minute.unwrap_or(MINUTE_RAW_NONE);
        let second_num = second.unwrap_or(SECOND_RAW_NONE);

        let mut bytes_written = write_map_err(DATE_TIME_SUBSECOND_TAG | precision_tag | (year_num >> 8) as u8,
                                              writer)?;
        bytes_written += write_map_err(year_num as u8, writer)?;
        bytes_written += write_map_err((month_num << 4) | (day_num >> 1), writer)?;
        bytes_written += write_map_err((day_num << 7) | (hour_num << 2) | (minute_num >> 4),
                                       writer)?;
        bytes_written += write_map_err((minute_num << 4) | (second_num >> 2), writer)?;
        bytes_written += write_map_err((second_num << 6) | first_subsecond_byte_fragment, writer)?;

        // write variable length fractinoal second
        match fractional_second {
            FractionalSecond::None => {},
            FractionalSecond::Milliseconds(ms) => {
                bytes_written += write_map_err((ms << 4) as u8, writer)?;
            },
            FractionalSecond::Microseconds(us) => {
                bytes_written += write_map_err((us >> 6) as u8, writer)?;
                bytes_written += write_map_err((us << 2) as u8, writer)?;
            },
            FractionalSecond::Nanoseconds(ns) => {
                bytes_written += write_map_err((ns >> 16) as u8, writer)?;
                bytes_written += write_map_err((ns >> 8) as u8, writer)?;
                bytes_written += write_map_err(ns as u8, writer)?;
            }
        }

        Ok(bytes_written)
    }
}

impl Date for DateTimeSubSecond {
    fn year(&self) -> Option<u16> {
        self.year
    }

    fn month(&self) -> Option<u8> {
        self.month
    }

    fn day(&self) -> Option<u8> {
        self.day
    }
}

impl Time for DateTimeSubSecond {
    fn hour(&self) -> Option<u8> {
        self.hour
    }

    fn minute(&self) -> Option<u8> {
        self.minute
    }

    fn second(&self) -> Option<u8> {
        self.second
    }
}

impl SubSecond for DateTimeSubSecond {
    fn fractional_second(&self) -> FractionalSecond {
        self.frac_second
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

#[derive(Debug, PartialEq, Eq)]
enum PrecisionTag {
    Milli,
    Micro,
    Nano,
    None
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeserializationError {
    IoError,
    EarlyEOF,
    IncorrectTypeTag,
    IncorrectPrecisionTag
}

#[derive(Debug, PartialEq, Eq)]
pub enum SerializationError {
    FieldValueOutOfRange(TemporalField),
    IoError
}

#[derive(Debug, PartialEq, Eq)]
pub enum TemporalField {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
    FractionalSecond,
    Offset
}

// type tags, expanded to include the rest of the byte
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

// precision tags: 2 bits in positions 3 and 4, expanded into a byte, as it would be in a DTS
const PRECISION_DTS_MASK: u8 = 0b0011_0000;
const PRECISION_DTS_MILLIS_TAG: u8 = 0x0;
const PRECISION_DTS_MICROS_TAG: u8 = 0b0001_0000;
const PRECISION_DTS_NANOS_TAG: u8 = 0b0010_0000;
const PRECISION_DTS_NONE_TAG: u8 = 0b0011_0000;

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
pub const MILLIS_MIN: u16 = 0;
pub const MILLIS_MAX: u16 = 1_000;
pub const MICROS_MIN: u32 = 0;
pub const MICROS_MAX: u32 = 1_000_000;
pub const NANOS_MIN: u32 = 0;
pub const NANOS_MAX: u32 = 1_000_000_000;

fn next_byte<S: Sized + Read>(bytes: &mut Bytes<S>) -> Result<u8, DeserializationError> {
    bytes.next()
        .map(|r| r.map_err(|_| DeserializationError::IoError))
        .unwrap_or(Err(DeserializationError::EarlyEOF))
}

fn write_map_err<W: Write>(byte: u8, writer: &mut W) -> Result<usize, SerializationError> {
    writer.write(&[byte]).map_err(|_| SerializationError::IoError)
}

fn check_option_outside_range<T: PartialOrd>(val: Option<T>, min: T, max: T, field: TemporalField)
                                             -> Result<(), SerializationError> {
    if let Some(v) = val {
        check_outside_range(v, min, max, field)?;
    }

    Ok(())
}

fn check_outside_range<T: PartialOrd>(v: T, min: T, max: T, field: TemporalField)
                                      -> Result<(), SerializationError> {
    if v < min || v > max {
        return Err(SerializationError::FieldValueOutOfRange(field))
    }

    Ok(())
}
