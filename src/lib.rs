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
    fn offset(&self) -> OffsetValue;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OffsetValue {
    /// Offset not specified.
    None,
    /// Temporal value is not at UTC, but the timezone is not specified here
    SpecifiedElsewhere,
    /// Temporal value is offset from UTC by the specified number of minutes
    UtcOffset(i16)
}

mod date_only;
mod time_only;
mod date_time;
mod date_time_offset;
mod date_time_subsecond;
mod date_time_subsecond_offset;

pub use date_only::DateOnly;
pub use time_only::TimeOnly;
pub use date_time::DateTime;
pub use date_time_offset::DateTimeOffset;
pub use date_time_subsecond::DateTimeSubSecond;
pub use date_time_subsecond_offset::DateTimeSubSecondOffset;

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
    InvalidFieldValue(TemporalField),
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

// precision tags: 2 bits in positions 3 and 4, expanded into a byte, as it would be in a DTSO
const PRECISION_DTSO_MASK: u8 = PRECISION_DTS_MASK >> 1;
const PRECISION_DTSO_MILLIS_TAG: u8 = PRECISION_DTS_MILLIS_TAG >> 1;
const PRECISION_DTSO_MICROS_TAG: u8 = PRECISION_DTS_MICROS_TAG >> 1;
const PRECISION_DTSO_NANOS_TAG: u8 = PRECISION_DTS_NANOS_TAG >> 1;
const PRECISION_DTSO_NONE_TAG: u8 = PRECISION_DTS_NONE_TAG >> 1;

// encoded forms of "no value"
const YEAR_RAW_NONE: u16 = 4095;
const MONTH_RAW_NONE: u8 = 15;
const DAY_RAW_NONE: u8 = 31;
const HOUR_RAW_NONE: u8 = 31;
const MINUTE_RAW_NONE: u8 = 63;
const SECOND_RAW_NONE: u8 = 63;
const OFFSET_RAW_NONE: u8 = 127;
const OFFSET_RAW_ELSEWHERE: u8 = 126;

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
pub const OFFSET_MIN: i16 = -(64 * 15);
pub const OFFSET_MAX: i16 = (125 - 64) * 15;

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
        return Err(SerializationError::InvalidFieldValue(field))
    }

    Ok(())
}
