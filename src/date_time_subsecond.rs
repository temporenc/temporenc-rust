use std::io::{Read, Write};

use super::{Date, Time, SubSecond, DeserializationError, SerializationError, next_byte, check_option_outside_range, check_outside_range, write_map_err, TypeTag, TemporalField, FractionalSecond, PrecisionTag, YEAR_MAX, YEAR_MIN, MONTH_MAX, MONTH_MIN, DAY_MAX, DAY_MIN, HOUR_MAX, HOUR_MIN, MINUTE_MAX, MINUTE_MIN, SECOND_MAX, SECOND_MIN, MILLIS_MAX, MILLIS_MIN, MICROS_MAX, MICROS_MIN, NANOS_MAX, NANOS_MIN, DATE_TIME_SUBSECOND_TAG, YEAR_RAW_NONE, MONTH_RAW_NONE, DAY_RAW_NONE, HOUR_RAW_NONE, MINUTE_RAW_NONE, SECOND_RAW_NONE, PRECISION_DTS_MASK, PRECISION_DTS_MILLIS_TAG, PRECISION_DTS_MICROS_TAG, PRECISION_DTS_NANOS_TAG, PRECISION_DTS_NONE_TAG};

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
