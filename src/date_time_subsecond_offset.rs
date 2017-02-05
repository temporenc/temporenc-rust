use std::io::{Read, Write};

use super::*;
use super::frac_second;

#[derive(Debug)]
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
    #[inline]
    pub fn new(year: Option<u16>, month: Option<u8>, day: Option<u8>, hour: Option<u8>,
               minute: Option<u8>, second: Option<u8>, frac_second: FractionalSecond,
               offset: OffsetValue) -> Result<DateTimeSubSecondOffset, CreationError> {
        let err_val = CreationError::InvalidFieldValue;

        check_frac_second(frac_second, err_val)?;

        Ok(DateTimeSubSecondOffset {
            year: year_num(year, err_val)?,
            month: month_num(month, err_val)?,
            day: day_num(day, err_val)?,
            hour: hour_num(hour, err_val)?,
            minute: minute_num(minute, err_val)?,
            second: second_num(second, err_val)?,
            frac_second_fw: frac_second::encode_fixed_width(&frac_second),
            offset: offset_num(offset, err_val)?
        })
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<DateTimeSubSecondOffset, DeserializationError> {
        let mut buf = [0; MAX_SERIALIZED_SIZE];
        read_exact(reader, &mut buf[0..MIN_SERIALIZED_SIZE])?;

        let byte0 = buf[0];

        if byte0 & 0b1110_0000 != DATE_TIME_SUBSECOND_OFFSET_TAG {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        let precision = match byte0 & PRECISION_DTSO_MASK {
            PRECISION_DTSO_MILLIS_TAG => PrecisionTag::Milli,
            PRECISION_DTSO_MICROS_TAG => PrecisionTag::Micro,
            PRECISION_DTSO_NANOS_TAG => PrecisionTag::Nano,
            PRECISION_DTSO_NONE_TAG => PrecisionTag::None,
            _ => {
                return Err(DeserializationError::IncorrectPrecisionTag);
            }
        };

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

        let (frac_second_fw, last_variable_byte) = match precision {
            PrecisionTag::Milli => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..(MIN_SERIALIZED_SIZE + 1)])?;
                let mut ms = ((byte5 & 0x1F) as u16) << 5;
                let byte6 = buf[6];
                ms |= (byte6 >> 3) as u16;

                check_in_range(ms, MILLIS_MIN, MILLIS_MAX,
                               DeserializationError::InvalidFieldValue)?;
                (frac_second::encode_millis(ms), byte6)
            }
            PrecisionTag::Micro => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..(MIN_SERIALIZED_SIZE + 2)])?;
                let mut us = ((byte5 & 0x1F) as u32) << 15;
                us |= (buf[6] as u32) << 7;
                let byte7 = buf[7];
                us |= (byte7 >> 1) as u32;

                check_in_range(us, MICROS_MIN, MICROS_MAX,
                               DeserializationError::InvalidFieldValue)?;
                (frac_second::encode_micros(us), byte7)
            }
            PrecisionTag::Nano => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..MAX_SERIALIZED_SIZE])?;
                let mut ns = ((byte5 & 0x1F) as u32) << 25;
                ns |= (buf[6] as u32) << 17;
                ns |= (buf[7] as u32) << 9;
                ns |= (buf[8] as u32) << 1;
                let byte9 = buf[9];
                ns |= (byte9 >> 7) as u32;

                check_in_range(ns, NANOS_MIN, NANOS_MAX,
                               DeserializationError::InvalidFieldValue)?;
                (frac_second::encode_nanos(ns), byte9)
            },
            PrecisionTag::None => (frac_second::encode_none(), byte5),
        };

        let raw_offset = match precision {
            PrecisionTag::Milli => ((last_variable_byte & 0x07) << 4) | (buf[7] >> 4),
            PrecisionTag::Micro => ((last_variable_byte & 0x01) << 6) | (buf[8] >> 2),
            PrecisionTag::Nano => last_variable_byte & 0x7F,
            PrecisionTag::None => ((last_variable_byte & 0x1F) << 2) | (buf[6] >> 6),
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

    pub fn serialize_components<W: Write>(year: Option<u16>, month: Option<u8>,
                                          day: Option<u8>, hour: Option<u8>,
                                          minute: Option<u8>, second: Option<u8>,
                                          frac_second: FractionalSecond,
                                          offset: OffsetValue, writer: &mut W)
                                          -> Result<usize, ComponentSerializationError> {
        let err_val = ComponentSerializationError::InvalidFieldValue;
        let offset_num = offset_num(offset, err_val)?;

        let (precision_tag, first_var_length_byte_fragment) = match frac_second {
            FractionalSecond::Milliseconds(ms) => {
                check_in_range(ms, MILLIS_MIN, MILLIS_MAX, err_val)?;
                (PRECISION_DTSO_MILLIS_TAG, (ms >> 5) as u8)
            },
            FractionalSecond::Microseconds(us) => {
                check_in_range(us, MICROS_MIN, MICROS_MAX, err_val)?;
                (PRECISION_DTSO_MICROS_TAG, (us >> 15) as u8)
            },
            FractionalSecond::Nanoseconds(ns) => {
                check_in_range(ns, NANOS_MIN, NANOS_MAX, err_val)?;
                (PRECISION_DTSO_NANOS_TAG, (ns >> 25) as u8)
            },
            FractionalSecond::None => (PRECISION_DTSO_NONE_TAG, offset_num >> 2),
        };

        Self::serialize_raw(year_num(year, err_val)?, month_num(month, err_val)?,
                            day_num(day, err_val)?, hour_num(hour, err_val)?,
                            minute_num(minute, err_val)?, second_num(second, err_val)?,
                            frac_second, offset_num, precision_tag,
                            first_var_length_byte_fragment, writer)
            .map_err(|_| ComponentSerializationError::IoError)
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        let f = frac_second::decode_fixed_width(self.frac_second_fw);
        let (precision_tag, first_var_length_byte_fragment) = match f {
            FractionalSecond::Milliseconds(ms) => {
                (PRECISION_DTSO_MILLIS_TAG, (ms >> 5) as u8)
            },
            FractionalSecond::Microseconds(us) => {
                (PRECISION_DTSO_MICROS_TAG, (us >> 15) as u8)
            },
            FractionalSecond::Nanoseconds(ns) => {
                (PRECISION_DTSO_NANOS_TAG, (ns >> 25) as u8)
            },
            FractionalSecond::None => (PRECISION_DTSO_NONE_TAG, self.offset >> 2),
        };

        Self::serialize_raw(self.year, self.month, self.day, self.hour, self.minute,
                            self.second, f, self.offset, precision_tag,
                            first_var_length_byte_fragment, writer)
            .map_err(|_| SerializationError::IoError)
    }

    fn serialize_raw<W: Write>(year: u16, month: u8, day: u8, hour: u8, minute: u8,
                               second: u8, frac_second: FractionalSecond,
                               offset: u8, precision_tag: u8,
                               first_var_length_byte_fragment: u8, writer: &mut W)
                               -> Result<usize, Error> {
        let b0 = DATE_TIME_SUBSECOND_OFFSET_TAG | precision_tag | (year >> 9) as u8;
        let b1 = (year >> 1) as u8;
        let b2 = (year << 7) as u8 | (month << 3) | (day >> 2);
        let b3 = (day << 6) | (hour << 1) | (minute >> 5);
        let b4 = (minute << 3) | (second >> 3);
        let b5 = (second << 5) | first_var_length_byte_fragment;

        let mut buf = [b0, b1, b2, b3, b4, b5, 0, 0, 0, 0];

        // write variable length fractional second
        let slice_end_index = match frac_second {
            FractionalSecond::None => {
                // tail end of offset
                buf[6] = offset << 6;
                7
            },
            FractionalSecond::Milliseconds(ms) => {
                buf[6] = ((ms << 3) as u8) | (offset >> 4);
                buf[7] = offset << 4;
                8
            },
            FractionalSecond::Microseconds(us) => {
                buf[6] = (us >> 7) as u8;
                buf[7] = ((us << 1) as u8) | offset >> 6;
                buf[8] = offset << 2;
                9
            },
            FractionalSecond::Nanoseconds(ns) => {
                buf[6] = (ns >> 17) as u8;
                buf[7] = (ns >> 9) as u8;
                buf[8] = (ns >> 1) as u8;
                buf[9] = (ns << 7) as u8 | offset;
                10
            }
        };

        write_array_map_err(&buf[0..slice_end_index], writer)
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
}

const MIN_SERIALIZED_SIZE: usize = 7;
const MAX_SERIALIZED_SIZE: usize = 10;
