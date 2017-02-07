//! An implementation of [Temporenc](https://temporenc.org/) v1, a compact binary format for
//! temporal data. This is (intentionally) not a full-featured time library; it is focused just on
//! handling the Temporenc format.
//!
//! Temporenc has the concept of "components" like date or time, and "types", which are a
//! composition of one ore more components like "date + time" or just "time". Components are
//! repesented in this library by traits (`Time`, `Date`, etc) and types by structs
//! (`TimeOnly` which only implements `Time`, `DateTime` which implements `Date` and `Time`, etc).
//!
//! All of the fields in a component ("day" in `Date`, "minute" in `Time`, etc) are optional, so
//! the accessors expose `Option<T>` or an enum with a `None` variant.
//!
//! All of the structs implement `Serializable` and `Deserializable` which, surprisingly enough,
//! provide methods related to serialization and deserialization.
//!
//! ```
//! use temporenc::*;
//! use std::io::Cursor;
//!
//! let mut vec = Vec::new();
//!
//! // Serialize a date
//! // 2017-01-15
//! let date = DateOnly::new(Some(2017), Some(01), Some(15)).unwrap();
//!
//! let date_bytes_written = date.serialize(&mut vec).unwrap();
//! assert_eq!(date_bytes_written, date.serialized_size());
//! // Date is not variable precision, so the serialized size is always the max size.
//! assert_eq!(DateOnly::max_serialized_size(), date.serialized_size());
//!
//! // Serialize a date + time + subsecond precision + offset
//! // 2017-01-15T18:45:30.123456+02:15
//! let dtso = DateTimeSubSecondOffset::new(Some(2017), Some(01), Some(15),
//!     Some(18), Some(45), Some(30), FractionalSecond::Microseconds(123456),
//!     OffsetValue::UtcOffset(135)).unwrap();
//!
//! let dtso_bytes_written = dtso.serialize(&mut vec).unwrap();
//! assert_eq!(dtso.serialized_size(), dtso_bytes_written);
//! // This one is only microseconds, not nanoseconds, so it doesn't have the full size
//! assert_eq!(DateTimeSubSecondOffset::max_serialized_size() - 1,
//!     dtso.serialized_size());
//!
//! // Deserialize the two items
//! assert_eq!(date.serialized_size() + dtso.serialized_size(), vec.len());
//! let mut cursor = Cursor::new(vec.as_slice());
//!
//! let deser_date = DateOnly::deserialize(&mut cursor).unwrap();
//! assert_eq!(date, deser_date);
//!
//! let deser_dtso =
//!     DateTimeSubSecondOffset::deserialize(&mut cursor).unwrap();
//! assert_eq!(dtso, deser_dtso);
//! ```

use std::io::{Read, Write, Error};

/// Serialize into the Temporenc binary format.
pub trait Serializable {
    /// The largest encoded size of any instance of the type. Some types have variable precision,
    /// and instances with higher precision will use more bytes than those with lower precision.
    fn max_serialized_size() -> usize;
    /// The encoded size of this instance. No larger than `max_serialized_size()`.
    fn serialized_size(&self) -> usize;
    /// Serialize into the provided writer with the Temporenc format. Returns the number of bytes
    /// written, which will be the same as `serialized_size()`.
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError>;
}

/// Deserialize from the Temporenc binary format.
pub trait Deserializable: Sized {
    /// Deserialize from the provided reader with the Temporenc format.
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, DeserializationError>;
}

/// Represents the Temporenc "Date" component.
pub trait Date {
    /// If present, the year. In range [0, 4094].
    fn year(&self) -> Option<u16>;
    /// If present, the month. In range [1, 12].
    fn month(&self) -> Option<u8>;
    /// If present, the day. In range [1, 31].
    fn day(&self) -> Option<u8>;
}

/// Represents the Temporenc "Time" component.
pub trait Time {
    /// If present, the number of hours. In range [0, 23].
    fn hour(&self) -> Option<u8>;
    /// If present, the number of minutes. In range [0, 59].
    fn minute(&self) -> Option<u8>;
    /// If present, the number of seconds. In range [0, 60].
    fn second(&self) -> Option<u8>;
}

/// Represents the Temporenc "Sub-second precision" component.
pub trait SubSecond {
    fn fractional_second(&self) -> FractionalSecond;
}

/// Represents the Temporenc "UTC Offset" component.
pub trait Offset {
    fn offset(&self) -> OffsetValue;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OffsetValue {
    /// Offset not specified.
    None,
    /// Temporal value is not at UTC, but the timezone is not specified here
    SpecifiedElsewhere,
    /// Temporal value is offset from UTC by the specified number of minutes.
    /// The number of minutes must be a multiple of 15.
    UtcOffset(i16)
}

mod date_only;
mod time_only;
mod date_time;
mod date_time_offset;
mod date_time_subsecond;
mod date_time_subsecond_offset;
mod frac_second;

pub use date_only::DateOnly;
pub use time_only::TimeOnly;
pub use date_time::DateTime;
pub use date_time_offset::DateTimeOffset;
pub use date_time_subsecond::DateTimeSubSecond;
pub use date_time_subsecond_offset::DateTimeSubSecondOffset;
pub use frac_second::FractionalSecond;

/// Used when creating a struct via `::new()`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CreationError {
    InvalidFieldValue,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeserializationError {
    InvalidFieldValue,
    IoError,
    IncorrectTypeTag,
    IncorrectPrecisionTag,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SerializationError {
    IoError
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

fn check_option_in_range<T: PartialOrd>(val: Option<T>, min: T, max: T, none: T) -> Result<T, CreationError> {
    if let Some(v) = val {
        return check_in_range(v, min, max, CreationError::InvalidFieldValue);
    }

    Ok(none)
}

fn check_in_range<T: PartialOrd, E: Copy>(v: T, min: T, max: T, err_val: E)
                                 -> Result<T, E> {
    if v < min || v > max {
        return Err(err_val)
    }

    Ok(v)
}

fn check_deser_in_range_or_none<T: PartialOrd>(v: T, min: T, max: T, none: T) -> Result<(), DeserializationError> {
    if (v >= min && v <= max) || v == none {
        Ok(())
    } else {
        Err(DeserializationError::InvalidFieldValue)
    }
}

#[inline]
fn year_num(year: Option<u16>) -> Result<u16, CreationError> {
    check_option_in_range(year, YEAR_MIN, YEAR_MAX, YEAR_RAW_NONE)
}

#[inline]
fn month_num(month: Option<u8>) -> Result<u8, CreationError> {
    if let Some(m) = month {
        // will never underflow because min = 1
        return check_in_range(m, MONTH_MIN, MONTH_MAX, CreationError::InvalidFieldValue).map(|m| m - 1);
    }

    Ok(MONTH_RAW_NONE)
}

#[inline]
fn day_num(day: Option<u8>) -> Result<u8, CreationError> {
    if let Some(d) = day {
        // will never underflow because min = 1
        return check_in_range(d, DAY_MIN, DAY_MAX, CreationError::InvalidFieldValue).map(|d| d - 1);
    }

    Ok(DAY_RAW_NONE)
}

#[inline]
fn hour_num(hour: Option<u8>) -> Result<u8, CreationError> {
    check_option_in_range(hour, HOUR_MIN, HOUR_MAX, HOUR_RAW_NONE)
}

#[inline]
fn minute_num(minute: Option<u8>) -> Result<u8, CreationError> {
    check_option_in_range(minute, MINUTE_MIN, MINUTE_MAX, MINUTE_RAW_NONE)
}

#[inline]
fn second_num(second: Option<u8>) -> Result<u8, CreationError> {
    check_option_in_range(second, SECOND_MIN, SECOND_MAX, SECOND_RAW_NONE)
}

#[inline]
fn offset_num(offset: OffsetValue) -> Result<u8, CreationError> {
    match offset {
        OffsetValue::None => Ok(OFFSET_RAW_NONE),
        OffsetValue::SpecifiedElsewhere => Ok(OFFSET_RAW_ELSEWHERE),
        OffsetValue::UtcOffset(o) => {
            check_in_range(o, OFFSET_MIN, OFFSET_MAX, CreationError::InvalidFieldValue)?;

            if o % 15 != 0 {
                return Err(CreationError::InvalidFieldValue);
            };

            Ok(((o / 15) + 64) as u8)
        }
    }
}

fn check_frac_second(frac_second: FractionalSecond) -> Result<(), CreationError> {
    match frac_second {
        FractionalSecond::None => {},
        FractionalSecond::Milliseconds(ms) => {
            check_in_range(ms, MILLIS_MIN, MILLIS_MAX, CreationError::InvalidFieldValue)?;
        },
        FractionalSecond::Microseconds(us) => {
            check_in_range(us, MICROS_MIN, MICROS_MAX, CreationError::InvalidFieldValue)?;
        },
        FractionalSecond::Nanoseconds(ns) => {
            check_in_range(ns, NANOS_MIN, NANOS_MAX, CreationError::InvalidFieldValue)?;
        }
    }

    Ok(())
}
