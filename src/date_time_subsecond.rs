use std::io::{Read, Write};

use super::{Serializable, Date, Time, SubSecond, DeserializationError, SerializationError, read_exact, check_option_outside_range, check_outside_range, write_array_map_err, TemporalField, FractionalSecond, PrecisionTag, YEAR_MAX, YEAR_MIN, MONTH_MAX, MONTH_MIN, DAY_MAX, DAY_MIN, HOUR_MAX, HOUR_MIN, MINUTE_MAX, MINUTE_MIN, SECOND_MAX, SECOND_MIN, MILLIS_MAX, MILLIS_MIN, MICROS_MAX, MICROS_MIN, NANOS_MAX, NANOS_MIN, DATE_TIME_SUBSECOND_TAG, YEAR_RAW_NONE, MONTH_RAW_NONE, DAY_RAW_NONE, HOUR_RAW_NONE, MINUTE_RAW_NONE, SECOND_RAW_NONE, PRECISION_DTS_MASK, PRECISION_DTS_MILLIS_TAG, PRECISION_DTS_MICROS_TAG, PRECISION_DTS_NANOS_TAG, PRECISION_DTS_NONE_TAG};

pub struct DateTimeSubSecond {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    frac_second: FractionalSecond
}

impl DateTimeSubSecond {
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

        let frac_second = match precision {
            PrecisionTag::None => FractionalSecond::None,
            PrecisionTag::Milli => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..(MIN_SERIALIZED_SIZE + 1)])?;
                let mut ms = ((byte5 & 0x3F) as u16) << 4;
                ms |= (buf[6] >> 4) as u16;

                FractionalSecond::Milliseconds(ms)
            }
            PrecisionTag::Micro => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..(MIN_SERIALIZED_SIZE + 2)])?;
                let mut us = ((byte5 & 0x3F) as u32) << 14;
                us |= (buf[6] as u32) << 6;
                us |= (buf[7] >> 2) as u32;

                FractionalSecond::Microseconds(us)
            }
            PrecisionTag::Nano => {
                read_exact(reader, &mut buf[MIN_SERIALIZED_SIZE..MAX_SERIALIZED_SIZE])?;
                let mut ns = ((byte5 & 0x3F) as u32) << 24;
                ns |= (buf[6] as u32) << 16;
                ns |= (buf[7] as u32) << 8;
                ns |= buf[8] as u32;

                FractionalSecond::Nanoseconds(ns)
            }
        };

        Ok(DateTimeSubSecond {
            year: raw_year,
            month: raw_month,
            day: raw_day,
            hour: raw_hour,
            minute: raw_minute,
            second: raw_second,
            frac_second: frac_second
        })
    }

    pub fn serialize_components<W: Write>(year: Option<u16>, month: Option<u8>, day: Option<u8>,
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


        let b0 = DATE_TIME_SUBSECOND_TAG | precision_tag | (year_num >> 8) as u8;
        let b1 = year_num as u8;
        let b2 = (month_num << 4) | (day_num >> 1);
        let b3 = (day_num << 7) | (hour_num << 2) | (minute_num >> 4);
        let b4 = (minute_num << 4) | (second_num >> 2);
        let b5 = (second_num << 6) | first_subsecond_byte_fragment;

        let mut buf = [b0, b1, b2, b3, b4, b5, 0, 0, 0];

        // write variable length fractional second
        let slice_end_index = match fractional_second {
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

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        Self::serialize_components(self.year(), self.month(), self.day(), self.hour(),
                                   self.minute(), self.second(), self.frac_second, writer)
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
        self.frac_second
    }
}

impl Serializable for DateTimeSubSecond {
    fn max_serialized_size() -> usize {
        MAX_SERIALIZED_SIZE
    }

    fn serialized_size(&self) -> usize {
        match self.frac_second {
            FractionalSecond::Milliseconds(_) => 7,
            FractionalSecond::Microseconds(_) => 8,
            FractionalSecond::Nanoseconds(_) => MAX_SERIALIZED_SIZE,
            FractionalSecond::None => MIN_SERIALIZED_SIZE,
        }
    }
}


const MIN_SERIALIZED_SIZE: usize = 6;
const MAX_SERIALIZED_SIZE: usize = 9;
