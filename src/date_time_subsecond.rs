use std::io::{Read, Write};

use super::*;
use super::frac_second;

#[derive(Debug)]
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
    #[inline]
    pub fn new(year: Option<u16>, month: Option<u8>, day: Option<u8>, hour: Option<u8>,
               minute: Option<u8>, second: Option<u8>, frac_second: FractionalSecond) -> Result<DateTimeSubSecond, CreationError> {
        let err_val = CreationError::InvalidFieldValue;

        check_frac_second(frac_second, err_val)?;

        Ok(DateTimeSubSecond {
            year: year_num(year, err_val)?,
            month: month_num(month, err_val)?,
            day: day_num(day, err_val)?,
            hour: hour_num(hour, err_val)?,
            minute: minute_num(minute, err_val)?,
            second: second_num(second, err_val)?,
            frac_second_fw: frac_second::encode_fixed_width(&frac_second)
        })
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<DateTimeSubSecond, DeserializationError> {
        let mut buf = [0; MAX_SERIALIZED_SIZE];
        read_exact(reader, &mut buf[0..MIN_SERIALIZED_SIZE])?;

        let byte0 = buf[0];

        if byte0 & 0b1100_0000 != DATE_TIME_SUBSECOND_TAG {
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

        let frac_second_fw = match precision {
            PrecisionTag::None => frac_second::encode_none(),
            PrecisionTag::Milli => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..(MIN_SERIALIZED_SIZE + 1)])?;
                let mut ms = ((byte5 & 0x3F) as u16) << 4;
                ms |= (buf[6] >> 4) as u16;

                check_in_range(ms, MILLIS_MIN, MILLIS_MAX,
                               DeserializationError::InvalidFieldValue)?;
                frac_second::encode_millis(ms)
            }
            PrecisionTag::Micro => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..(MIN_SERIALIZED_SIZE + 2)])?;
                let mut us = ((byte5 & 0x3F) as u32) << 14;
                us |= (buf[6] as u32) << 6;
                us |= (buf[7] >> 2) as u32;

                check_in_range(us, MICROS_MIN, MICROS_MAX,
                               DeserializationError::InvalidFieldValue)?;
                frac_second::encode_micros(us)
            }
            PrecisionTag::Nano => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..MAX_SERIALIZED_SIZE])?;
                let mut ns = ((byte5 & 0x3F) as u32) << 24;
                ns |= (buf[6] as u32) << 16;
                ns |= (buf[7] as u32) << 8;
                ns |= buf[8] as u32;

                check_in_range(ns, NANOS_MIN, NANOS_MAX,
                               DeserializationError::InvalidFieldValue)?;
                frac_second::encode_nanos(ns)
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

    pub fn serialize_components<W: Write>(year: Option<u16>, month: Option<u8>,
                                          day: Option<u8>, hour: Option<u8>,
                                          minute: Option<u8>, second: Option<u8>,
                                          frac_second: FractionalSecond, writer: &mut W)
                                          -> Result<usize, ComponentSerializationError> {
        let err_val = ComponentSerializationError::InvalidFieldValue;

        let (precision_tag, first_subsecond_byte_fragment) = match frac_second {
            FractionalSecond::None => (PRECISION_DTS_NONE_TAG, 0x0),
            FractionalSecond::Milliseconds(ms) => {
                check_in_range(ms, MILLIS_MIN, MILLIS_MAX, err_val)?;
                (PRECISION_DTS_MILLIS_TAG, (ms >> 4) as u8)
            },
            FractionalSecond::Microseconds(us) => {
                check_in_range(us, MICROS_MIN, MICROS_MAX, err_val)?;
                (PRECISION_DTS_MICROS_TAG, (us >> 14) as u8)
            },
            FractionalSecond::Nanoseconds(ns) => {
                check_in_range(ns, NANOS_MIN, NANOS_MAX, err_val)?;
                (PRECISION_DTS_NANOS_TAG, (ns >> 24) as u8)
            }
        };

        Self::serialize_raw(year_num(year, err_val)?, month_num(month, err_val)?,
                            day_num(day, err_val)?, hour_num(hour, err_val)?,
                            minute_num(minute, err_val)?, second_num(second, err_val)?,
                            frac_second, precision_tag, first_subsecond_byte_fragment,
                            writer)
            .map_err(|_| ComponentSerializationError::IoError)
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        let f = frac_second::decode_fixed_width(self.frac_second_fw);
        let (precision_tag, first_subsecond_byte_fragment) = match f {
            FractionalSecond::None => (PRECISION_DTS_NONE_TAG, 0x0),
            FractionalSecond::Milliseconds(ms) => {
                (PRECISION_DTS_MILLIS_TAG, (ms >> 4) as u8)
            },
            FractionalSecond::Microseconds(us) => {
                (PRECISION_DTS_MICROS_TAG, (us >> 14) as u8)
            },
            FractionalSecond::Nanoseconds(ns) => {
                (PRECISION_DTS_NANOS_TAG, (ns >> 24) as u8)
            }
        };

        Self::serialize_raw(self.year, self.month, self.day, self.hour, self.minute,
                            self.second, f, precision_tag,
                            first_subsecond_byte_fragment, writer)
            .map_err(|_| SerializationError::IoError)
    }

    fn serialize_raw<W: Write>(year: u16, month: u8, day: u8, hour: u8, minute: u8,
                               second: u8, frac_second: FractionalSecond,
                               precision_tag: u8, first_subsecond_byte_fragment: u8,
                               writer: &mut W) -> Result<usize, Error> {
        let b0 = DATE_TIME_SUBSECOND_TAG | precision_tag | (year >> 8) as u8;
        let b1 = year as u8;
        let b2 = (month << 4) | (day >> 1);
        let b3 = (day << 7) | (hour << 2) | (minute >> 4);
        let b4 = (minute << 4) | (second >> 2);
        let b5 = (second << 6) | first_subsecond_byte_fragment;

        let mut buf = [b0, b1, b2, b3, b4, b5, 0, 0, 0];

        // write variable length fractional second
        let slice_end_index = match frac_second {
            FractionalSecond::None => 6,
            FractionalSecond::Milliseconds(ms) => {
                buf[6] = (ms << 4) as u8;
                7
            },
            FractionalSecond::Microseconds(us) => {
                buf[6] = (us >> 6) as u8;
                buf[7] = (us << 2) as u8;
                8
            },
            FractionalSecond::Nanoseconds(ns) => {
                buf[6] = (ns >> 16) as u8;
                buf[7] = (ns >> 8) as u8;
                buf[8] = ns as u8;
                9
            }
        };

        write_array_map_err(&buf[0..slice_end_index], writer)
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
}


const MIN_SERIALIZED_SIZE: usize = 6;
const MAX_SERIALIZED_SIZE: usize = 9;
