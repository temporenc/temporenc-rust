use std::io::{Read, Write};

use super::*;
use super::frac_second;

/// A Date and Time with subsecond precision.
#[derive(Debug, PartialEq)]
pub struct DateTimeSubSecond {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    frac_second_fw: u32
}

impl DateTimeSubSecond {

    /// Returns an error if any of the arguments have invalid values, like a month of 18.
    #[inline]
    pub fn new(year: Option<u16>, month: Option<u8>, day: Option<u8>, hour: Option<u8>,
               minute: Option<u8>, second: Option<u8>, frac_second: FractionalSecond) -> Result<DateTimeSubSecond, CreationError> {
        check_frac_second(frac_second)?;

        Ok(DateTimeSubSecond {
            year: year_num(year)?,
            month: month_num(month)?,
            day: day_num(day)?,
            hour: hour_num(hour)?,
            minute: minute_num(minute)?,
            second: second_num(second)?,
            frac_second_fw: frac_second::encode_fixed_width(&frac_second)
        })
    }
}

impl Date for DateTimeSubSecond {
    fn year(&self) -> Option<u16> {
        if self.year == YEAR_RAW_NONE {
            None
        } else {
            Some(self.year)
        }
    }

    fn month(&self) -> Option<u8> {
        if self.month == MONTH_RAW_NONE {
            None
        } else {
            Some(self.month + 1)
        }
    }

    fn day(&self) -> Option<u8> {
        if self.day == DAY_RAW_NONE {
            None
        } else {
            Some(self.day + 1)
        }
    }
}

impl Time for DateTimeSubSecond {
    fn hour(&self) -> Option<u8> {
        if self.hour == HOUR_RAW_NONE {
            None
        } else {
            Some(self.hour)
        }
    }

    fn minute(&self) -> Option<u8> {
        if self.minute == MINUTE_RAW_NONE {
            None
        } else {
            Some(self.minute)
        }
    }

    fn second(&self) -> Option<u8> {
        if self.second == SECOND_RAW_NONE {
            None
        } else {
            Some(self.second)
        }
    }
}

impl SubSecond for DateTimeSubSecond {
    fn fractional_second(&self) -> FractionalSecond {
        frac_second::decode_fixed_width(self.frac_second_fw)
    }
}

impl Serializable for DateTimeSubSecond {
    fn max_serialized_size() -> usize {
        MAX_SERIALIZED_SIZE
    }

    fn serialized_size(&self) -> usize {
        match frac_second::decode_fixed_width(self.frac_second_fw) {
            FractionalSecond::Milliseconds(_) => 7,
            FractionalSecond::Microseconds(_) => 8,
            FractionalSecond::Nanoseconds(_) => MAX_SERIALIZED_SIZE,
            FractionalSecond::None => MIN_SERIALIZED_SIZE,
        }
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        let b0_partial = DATE_TIME_SUBSECOND_TAG | (self.year >> 8) as u8;

        let b1 = self.year as u8;
        let b2 = (self.month << 4) | (self.day >> 1);
        let b3 = (self.day << 7) | (self.hour << 2) | (self.minute >> 4);
        let b4 = (self.minute << 4) | (self.second >> 2);
        let b5_partial = self.second << 6;

        let mut buf = [0, b1, b2, b3, b4, 0, 0, 0, 0];

        let frac_prefix = frac_second::FRAC_SECOND_FIXED_WIDTH_PREFIX_MASK & self.frac_second_fw;
        let frac_value = frac_second::FRAC_SECOND_FIXED_WIDTH_VALUE_MASK & self.frac_second_fw;

        let slice_end_index = match frac_prefix {
            frac_second::FRAC_SECOND_FIXED_WIDTH_NONE => {
                buf[0] = b0_partial | PRECISION_DTS_NONE_TAG;
                buf[5] = b5_partial;
                6
            },
            frac_second::FRAC_SECOND_FIXED_WIDTH_MILLI => {
                buf[0] = b0_partial | PRECISION_DTS_MILLIS_TAG;
                buf[5] = b5_partial | (frac_value >> 4) as u8;
                buf[6] = (frac_value << 4) as u8;
                7
            },
            frac_second::FRAC_SECOND_FIXED_WIDTH_MICRO => {
                buf[0] = b0_partial | PRECISION_DTS_MICROS_TAG;
                buf[5] = b5_partial | (frac_value >> 14) as u8;
                buf[6] = (frac_value >> 6) as u8;
                buf[7] = (frac_value << 2) as u8;
                8
            },
            frac_second::FRAC_SECOND_FIXED_WIDTH_NANO => {
                buf[0] = b0_partial | PRECISION_DTS_NANOS_TAG;
                buf[5] = b5_partial | (frac_value >> 24) as u8;
                buf[6] = (frac_value >> 16) as u8;
                buf[7] = (frac_value >> 8) as u8;
                buf[8] = frac_value as u8;
                9
            },
            _ => panic!("Corrupt fixed width encoded fractional second")
        };

        write_array_map_err(&buf[0..slice_end_index], writer)
            .map_err(|_| SerializationError::IoError)
    }
}

impl Deserializable for DateTimeSubSecond {
    fn deserialize<R: Read>(reader: &mut R) -> Result<DateTimeSubSecond, DeserializationError> {
        let mut buf = [0; MAX_SERIALIZED_SIZE];
        read_exact(reader, &mut buf[0..MIN_SERIALIZED_SIZE])?;

        let byte0 = buf[0];

        if byte0 & 0b1100_0000 != DATE_TIME_SUBSECOND_TAG {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        // 2-bit tag, 2-bit subsecond precision tag, 12-bit year, 4-bit month, 5-bit day, 5-bit hour,
        // 6-bit minute, 6-bit second, and 0, 10, 20, or 30-bit fractional second
        // TTPP YYYY | YYYY YYYY | MMMM DDDD | DHHH HHMM
        // MMMM SSSS | SSFF FFFF | [0, 1, 2, or 3 subsecond bytes]

        let byte1 = buf[1];
        let mut raw_year = ((byte0 & 0x0F) as u16) << 8;
        raw_year |= byte1 as u16;

        let byte2 = buf[2];
        let raw_month = byte2 >> 4;

        let byte3 = buf[3];
        let raw_day = ((byte2 & 0x0F) << 1) | (byte3 >> 7);

        let raw_hour = (byte3 & 0x7C) >> 2;

        let byte4 = buf[4];
        let raw_minute = ((byte3 & 0x03) << 4) | (byte4 >> 4);

        let byte5 = buf[5];
        let raw_second = ((byte4 & 0x0F) << 2) | ((byte5 & 0xC0) >> 6);

        let frac_second_fw = match byte0 & PRECISION_DTS_MASK {
            PRECISION_DTS_NONE_TAG => frac_second::encode_none(),
            PRECISION_DTS_MILLIS_TAG => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..(MIN_SERIALIZED_SIZE + 1)])?;
                let mut ms = ((byte5 & 0x3F) as u16) << 4;
                ms |= (buf[6] >> 4) as u16;

                check_in_range(ms, MILLIS_MIN, MILLIS_MAX,
                               DeserializationError::InvalidFieldValue)?;
                frac_second::encode_millis(ms)
            }
            PRECISION_DTS_MICROS_TAG => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..(MIN_SERIALIZED_SIZE + 2)])?;
                let mut us = ((byte5 & 0x3F) as u32) << 14;
                us |= (buf[6] as u32) << 6;
                us |= (buf[7] >> 2) as u32;

                check_in_range(us, MICROS_MIN, MICROS_MAX,
                               DeserializationError::InvalidFieldValue)?;
                frac_second::encode_micros(us)
            }
            PRECISION_DTS_NANOS_TAG  => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..MAX_SERIALIZED_SIZE])?;
                let mut ns = ((byte5 & 0x3F) as u32) << 24;
                ns |= (buf[6] as u32) << 16;
                ns |= (buf[7] as u32) << 8;
                ns |= buf[8] as u32;

                check_in_range(ns, NANOS_MIN, NANOS_MAX,
                               DeserializationError::InvalidFieldValue)?;
                frac_second::encode_nanos(ns)
            },
            _ => {
                return Err(DeserializationError::IncorrectPrecisionTag);
            }
        };

        // no need to check year as every possible number is a valid year
        check_deser_in_range_or_none(raw_month, MONTH_RAW_MIN, MONTH_RAW_MAX, MONTH_RAW_NONE)?;
        // no need to check day as every possible number is a valid day
        check_deser_in_range_or_none(raw_hour, HOUR_MIN, HOUR_MAX, HOUR_RAW_NONE)?;
        check_deser_in_range_or_none(raw_minute, MINUTE_MIN, MINUTE_MAX, MINUTE_RAW_NONE)?;
        check_deser_in_range_or_none(raw_second, SECOND_MIN, SECOND_MAX, SECOND_RAW_NONE)?;

        Ok(DateTimeSubSecond {
            year: raw_year,
            month: raw_month,
            day: raw_day,
            hour: raw_hour,
            minute: raw_minute,
            second: raw_second,
            frac_second_fw: frac_second_fw
        })
    }
}

const MIN_SERIALIZED_SIZE: usize = 6;
const MAX_SERIALIZED_SIZE: usize = 9;
