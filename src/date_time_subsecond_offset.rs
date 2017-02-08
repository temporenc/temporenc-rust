use std::io::{Read, Write};

use super::*;
use super::frac_second;

/// A Date and Time with subsecond precision and UTC offset.
#[derive(Debug, PartialEq)]
pub struct DateTimeSubSecondOffset {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    frac_second_fw: u32,
    offset: u8
}

impl DateTimeSubSecondOffset {

    /// Returns an error if any of the arguments have invalid values, like a month of 18.
    #[inline]
    pub fn new(year: Option<u16>, month: Option<u8>, day: Option<u8>, hour: Option<u8>,
               minute: Option<u8>, second: Option<u8>, frac_second: FractionalSecond,
               offset: OffsetValue) -> Result<DateTimeSubSecondOffset, CreationError> {
        check_frac_second(frac_second)?;

        Ok(DateTimeSubSecondOffset {
            year: year_num(year)?,
            month: month_num(month)?,
            day: day_num(day)?,
            hour: hour_num(hour)?,
            minute: minute_num(minute)?,
            second: second_num(second)?,
            frac_second_fw: frac_second::encode_fixed_width(&frac_second),
            offset: offset_num(offset)?
        })
    }
}

impl Date for DateTimeSubSecondOffset {
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

impl Time for DateTimeSubSecondOffset {
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

impl SubSecond for DateTimeSubSecondOffset {
    fn fractional_second(&self) -> FractionalSecond {
        frac_second::decode_fixed_width(self.frac_second_fw)
    }
}

impl Offset for DateTimeSubSecondOffset {
    fn offset(&self) -> OffsetValue {
        match self.offset {
            127 => OffsetValue::None,
            126 => OffsetValue::SpecifiedElsewhere,
            x => OffsetValue::UtcOffset(((x as i16) - 64) * 15)
        }
    }
}

impl Serializable for DateTimeSubSecondOffset {
    fn max_serialized_size() -> usize {
        MAX_SERIALIZED_SIZE
    }

    fn serialized_size(&self) -> usize {
        match frac_second::decode_fixed_width(self.frac_second_fw) {
            FractionalSecond::Milliseconds(_) => 8,
            FractionalSecond::Microseconds(_) => 9,
            FractionalSecond::Nanoseconds(_) => MAX_SERIALIZED_SIZE,
            FractionalSecond::None => MIN_SERIALIZED_SIZE,
        }
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        let b0_partial = DATE_TIME_SUBSECOND_OFFSET_TAG | (self.year >> 9) as u8;
        let b1 = (self.year >> 1) as u8;
        let b2 = (self.year << 7) as u8 | (self.month << 3) | (self.day >> 2);
        let b3 = (self.day << 6) | (self.hour << 1) | (self.minute >> 5);
        let b4 = (self.minute << 3) | (self.second >> 3);
        let b5_partial = self.second << 5;

        let mut buf = [0, b1, b2, b3, b4, 0, 0, 0, 0, 0];

        let frac_prefix = frac_second::FRAC_SECOND_FIXED_WIDTH_PREFIX_MASK & self.frac_second_fw;
        let frac_value = frac_second::FRAC_SECOND_FIXED_WIDTH_VALUE_MASK & self.frac_second_fw;

        let slice_end_index = match frac_prefix {
            frac_second::FRAC_SECOND_FIXED_WIDTH_NONE => {
                buf[0] = b0_partial | PRECISION_DTSO_NONE_TAG;
                buf[5] = b5_partial | self.offset >> 2;
                buf[6] = self.offset << 6;
                7
            },
            frac_second::FRAC_SECOND_FIXED_WIDTH_MILLI => {
                buf[0] = b0_partial | PRECISION_DTSO_MILLIS_TAG;
                buf[5] = b5_partial | (frac_value >> 5) as u8;
                buf[6] = ((frac_value << 3) as u8) | (self.offset >> 4);
                buf[7] = self.offset << 4;
                8
            },
            frac_second::FRAC_SECOND_FIXED_WIDTH_MICRO => {
                buf[0] = b0_partial | PRECISION_DTSO_MICROS_TAG;
                buf[5] = b5_partial | (frac_value >> 15) as u8;
                buf[6] = (frac_value >> 7) as u8;
                buf[7] = ((frac_value << 1) as u8) | self.offset >> 6;
                buf[8] = self.offset << 2;
                9
            },
            frac_second::FRAC_SECOND_FIXED_WIDTH_NANO => {
                buf[0] = b0_partial | PRECISION_DTSO_NANOS_TAG;
                buf[5] = b5_partial | (frac_value >> 25) as u8;
                buf[6] = (frac_value >> 17) as u8;
                buf[7] = (frac_value >> 9) as u8;
                buf[8] = (frac_value >> 1) as u8;
                buf[9] = (frac_value << 7) as u8 | self.offset;
                10
            },
            _ => panic!("Corrupt fixed width encoded fractional second")
        };

        write_array_map_err(&buf[0..slice_end_index], writer)
            .map_err(|_| SerializationError::IoError)
    }
}

impl Deserializable for DateTimeSubSecondOffset {
    fn deserialize<R: Read>(reader: &mut R) -> Result<DateTimeSubSecondOffset, DeserializationError> {
        let mut buf = [0; MAX_SERIALIZED_SIZE];
        read_exact(reader, &mut buf[0..MIN_SERIALIZED_SIZE])?;

        let byte0 = buf[0];

        if byte0 & 0b1110_0000 != DATE_TIME_SUBSECOND_OFFSET_TAG {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        // 3-bit tag, 2-bit subsecond precision tag, 12-bit year, 4-bit month, 5-bit day, 5-bit hour,
        // 6-bit minute, 6-bit second, (0, 10, 20, or 30)-bit fractional second, 7-bit offset
        // TTTP PYYY | YYYY YYYY | YMMM MDDD | DDHH HHHM | MMMM MSSS
        // SSSF FFFF | FFFF FOOO | OOOO ____ [millis]
        // SSSF FFFF | FFFF FFFF | FFFF FFFO | OOOO OO__ [micros]
        // SSSF FFFF | FFFF FFFF | FFFF FFFF | FFFF FFFF | FOOO OOOO [nanos]
        // SSSO OOOO | OO__ ____ [none]

        let byte1 = buf[1];
        let byte2 = buf[2];
        let mut raw_year = ((byte0 & 0x07) as u16) << 9;
        raw_year |= (byte1 as u16) << 1;
        raw_year |= ((byte2 as u16) & 0x80) >> 7;

        let raw_month = (byte2 & 0x78) >> 3;

        let byte3 = buf[3];
        let raw_day = ((byte2 & 0x07) << 2) | (byte3 >> 6);

        let raw_hour = (byte3 & 0x3E) >> 1;

        let byte4 = buf[4];
        let raw_minute = ((byte3 & 0x01) << 5) | (byte4 >> 3);

        let byte5 = buf[5];
        let raw_second = ((byte4 & 0x07) << 3) | (byte5 >> 5);

        let (frac_second_fw, raw_offset) = match byte0 & PRECISION_DTSO_MASK {
            PRECISION_DTSO_MILLIS_TAG => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..(MIN_SERIALIZED_SIZE + 1)])?;
                let mut ms = ((byte5 & 0x1F) as u16) << 5;
                let byte6 = buf[6];
                ms |= (byte6 >> 3) as u16;

                check_in_range(ms, MILLIS_MIN, MILLIS_MAX,
                               DeserializationError::InvalidFieldValue)?;

                let raw_offset = ((byte6 & 0x07) << 4) | (buf[7] >> 4);
                (frac_second::encode_millis(ms), raw_offset)
            }
            PRECISION_DTSO_MICROS_TAG => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..(MIN_SERIALIZED_SIZE + 2)])?;
                let mut us = ((byte5 & 0x1F) as u32) << 15;
                us |= (buf[6] as u32) << 7;
                let byte7 = buf[7];
                us |= (byte7 >> 1) as u32;

                check_in_range(us, MICROS_MIN, MICROS_MAX,
                               DeserializationError::InvalidFieldValue)?;

                let raw_offset = ((byte7 & 0x01) << 6) | (buf[8] >> 2);

                (frac_second::encode_micros(us), raw_offset)
            }
            PRECISION_DTSO_NANOS_TAG => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..MAX_SERIALIZED_SIZE])?;
                let mut ns = ((byte5 & 0x1F) as u32) << 25;
                ns |= (buf[6] as u32) << 17;
                ns |= (buf[7] as u32) << 9;
                ns |= (buf[8] as u32) << 1;
                let byte9 = buf[9];
                ns |= (byte9 >> 7) as u32;

                check_in_range(ns, NANOS_MIN, NANOS_MAX,
                               DeserializationError::InvalidFieldValue)?;

                let raw_offset = byte9 & 0x7F;
                (frac_second::encode_nanos(ns), raw_offset)
            },
            PRECISION_DTSO_NONE_TAG => {
                let raw_offset = ((byte5 & 0x1F) << 2) | (buf[6] >> 6);
                (frac_second::encode_none(), raw_offset)
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
        // no need to check offset as every possible number is a valid offset

        Ok(DateTimeSubSecondOffset {
            year: raw_year,
            month: raw_month,
            day: raw_day,
            hour: raw_hour,
            minute: raw_minute,
            second: raw_second,
            frac_second_fw: frac_second_fw,
            offset: raw_offset
        })
    }
}

const MIN_SERIALIZED_SIZE: usize = 7;
const MAX_SERIALIZED_SIZE: usize = 10;
