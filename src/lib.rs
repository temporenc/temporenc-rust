use std::io::{Read, Write, Error};

pub trait Serializable {
    /// The largest encoded size of any instance of the type
    fn max_serialized_size() -> usize;
    /// The encoded size of this instance
    fn serialized_size(&self) -> usize;
}

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

#[derive(Debug, PartialEq, Eq)]
pub enum CreationError {
    InvalidFieldValue,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeserializationError {
    InvalidFieldValue,
    IoError,
    IncorrectTypeTag,
    IncorrectPrecisionTag,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SerializationError {
    IoError
}

#[derive(Debug, PartialEq, Eq)]
pub enum ComponentSerializationError {
    InvalidFieldValue,
    IoError
}

#[derive(Debug, PartialEq, Eq)]
enum PrecisionTag {
    Milli,
    Micro,
    Nano,
    None
}

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
pub const MINUTE_MAX: u8 = 59;
pub const SECOND_MIN: u8 = 0;
pub const SECOND_MAX: u8 = 60;
pub const MILLIS_MIN: u16 = 0;
pub const MILLIS_MAX: u16 = 999;
pub const MICROS_MIN: u32 = 0;
pub const MICROS_MAX: u32 = 999_999;
pub const NANOS_MIN: u32 = 0;
pub const NANOS_MAX: u32 = 999_999_999;
pub const OFFSET_MIN: i16 = -(64 * 15);
pub const OFFSET_MAX: i16 = (125 - 64) * 15;

// for when encoded form has different limits
const MONTH_RAW_MIN: u8 = 0;
const MONTH_RAW_MAX: u8 = 11;

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

// As of 1.14, yields a ~25% perf improvement when there are multiple things in the benchmark
// that end up calling this (namely, all benchmarks that have > 1 type deserialized).
// With (always), benchmarks perform the same as they do when all other functions in the file
// are commented out. With merely #[inline], it has no effect vs no inline at all.
#[inline(always)]
fn read_exact<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<(), DeserializationError> {
    reader.read_exact(buf).map_err(|_| DeserializationError::IoError)
}

fn write_array_map_err<W: Write>(bytes: &[u8], writer: &mut W) -> Result<usize, Error> {
    writer.write_all(bytes).map(|_| bytes.len())
}

fn check_option_in_range<T: PartialOrd>(val: Option<T>, min: T, max: T)
                                        -> Result<(), ComponentSerializationError> {
    if let Some(v) = val {
        return check_in_range(v, min, max);
    }

    Ok(())
}

fn check_in_range<T: PartialOrd>(v: T, min: T, max: T)
                                 -> Result<(), ComponentSerializationError> {
    if v < min || v > max {
        return Err(ComponentSerializationError::InvalidFieldValue)
    }

    Ok(())
}

fn check_new_option_in_range<T: PartialOrd>(val: Option<T>, min: T, max: T)
                                            -> Result<(), CreationError> {
    if let Some(v) = val {
        return check_new_in_range(v, min, max);
    }

    Ok(())
}
fn check_new_in_range<T: PartialOrd>(v: T, min: T, max: T)
                                     -> Result<(), CreationError> {
    if v < min || v > max {
        return Err(CreationError::InvalidFieldValue)
    }

    Ok(())
}

fn check_deser_in_range<T: PartialOrd + std::fmt::Debug>(v: T, min: T, max: T)
                                      -> Result<(), DeserializationError> {
    if v < min || v > max {
        return Err(DeserializationError::InvalidFieldValue)
    }

    Ok(())
}

fn check_deser_in_range_or_none<T: PartialOrd>(v: T, min: T, max: T, none: T) -> Result<(), DeserializationError> {
    if (v >= min && v <= max) || v == none {
        Ok(())
    } else {
        Err(DeserializationError::InvalidFieldValue)
    }
}

// As of 1.14, 3x speed boost on serialization benchmarks with this inline
#[inline]
fn encode_offset_num(offset: OffsetValue) -> Result<u8, ComponentSerializationError> {
    match offset {
        OffsetValue::None => Ok(OFFSET_RAW_NONE),
        OffsetValue::SpecifiedElsewhere => Ok(OFFSET_RAW_ELSEWHERE),
        OffsetValue::UtcOffset(o) => {
            check_in_range(o, OFFSET_MIN, OFFSET_MAX)?;

            if o % 15 != 0 {
                return Err(ComponentSerializationError::InvalidFieldValue);
            };

            Ok(((o / 15) + 64) as u8)
        }
    }
}

// convert Option back to numeric form

#[inline]
fn year_num(year: Option<u16>) -> u16 {
    year.unwrap_or(YEAR_RAW_NONE)
}

#[inline]
fn month_num(month: Option<u8>) -> u8 {
    month.map(|m| m - 1).unwrap_or(MONTH_RAW_NONE)
}

#[inline]
fn day_num(day: Option<u8>) -> u8 {
    day.map(|m| m - 1).unwrap_or(DAY_RAW_NONE)
}

#[inline]
fn hour_num(hour: Option<u8>) -> u8 {
    hour.unwrap_or(HOUR_RAW_NONE)
}

#[inline]
fn minute_num(minute: Option<u8>) -> u8 {
    minute.unwrap_or(MINUTE_RAW_NONE)
}

#[inline]
fn second_num(second: Option<u8>) -> u8 {
    second.unwrap_or(SECOND_RAW_NONE)
}
