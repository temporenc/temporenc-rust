use std::io::{Read, Write};

use super::{Serializable, Date, Time, SubSecond, Offset, DeserializationError, SerializationError, next_byte, check_option_outside_range, check_outside_range, write_array_map_err, TypeTag, TemporalField, OffsetValue, FractionalSecond, PrecisionTag, YEAR_MAX, YEAR_MIN, MONTH_MAX, MONTH_MIN, DAY_MAX, DAY_MIN, HOUR_MAX, HOUR_MIN, MINUTE_MAX, MINUTE_MIN, SECOND_MAX, SECOND_MIN, MILLIS_MAX, MILLIS_MIN, MICROS_MAX, MICROS_MIN, NANOS_MAX, NANOS_MIN, DATE_TIME_SUBSECOND_OFFSET_TAG, YEAR_RAW_NONE, MONTH_RAW_NONE, DAY_RAW_NONE, HOUR_RAW_NONE, MINUTE_RAW_NONE, SECOND_RAW_NONE, PRECISION_DTSO_MASK, PRECISION_DTSO_MILLIS_TAG, PRECISION_DTSO_MICROS_TAG, PRECISION_DTSO_NANOS_TAG, PRECISION_DTSO_NONE_TAG};

use super::date_time_offset::encode_offset_num;

#[derive(Debug)]
pub struct DateTimeSubSecondOffset {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
    frac_second: FractionalSecond,
    offset: OffsetValue
}

impl DateTimeSubSecondOffset {
    pub fn deserialize<R: Read>(reader: R) -> Result<DateTimeSubSecondOffset, DeserializationError> {
        let mut bytes = reader.bytes();
        let byte0 = next_byte(&mut bytes)?;

        if !TypeTag::DateTimeSubSecondOffset.matches(byte0) {
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

        let byte1 = next_byte(&mut bytes)?;
        let byte2 = next_byte(&mut bytes)?;
        let mut raw_year = ((byte0 & 0x07) as u16) << 9;
        raw_year |= (byte1 as u16) << 1;
        raw_year |= ((byte2 as u16) & 0x80) >> 7;
        let year = if raw_year == YEAR_RAW_NONE {
            None
        } else {
            Some(raw_year)
        };

        let raw_month = (byte2 & 0x78) >> 3;
        let month = if raw_month == MONTH_RAW_NONE {
            None
        } else {
            Some(raw_month + 1)
        };

        let byte3 = next_byte(&mut bytes)?;
        let raw_day = ((byte2 & 0x07) << 2) | (byte3 >> 6);
        let day = if raw_day == DAY_RAW_NONE {
            None
        } else {
            Some(raw_day + 1)
        };

        let raw_hour = (byte3 & 0x3E) >> 1;
        let hour = if raw_hour == HOUR_RAW_NONE {
            None
        } else {
            Some(raw_hour)
        };

        let byte4 = next_byte(&mut bytes)?;
        let raw_minute = ((byte3 & 0x01) << 5) | (byte4 >> 3);
        let minute = if raw_minute == MINUTE_RAW_NONE {
            None
        } else {
            Some(raw_minute)
        };

        let byte5 = next_byte(&mut bytes)?;
        let raw_second = ((byte4 & 0x07) << 3) | (byte5 >> 5);
        let second = if raw_second == SECOND_RAW_NONE {
            None
        } else {
            Some(raw_second)
        };

        let (frac_second, last_variable_byte) = match precision {
            PrecisionTag::Milli => {
                let mut ms = ((byte5 & 0x1F) as u16) << 5;
                let byte6 = next_byte(&mut bytes)?;
                ms |= (byte6 >> 3) as u16;

                (FractionalSecond::Milliseconds(ms), byte6)
            }
            PrecisionTag::Micro => {
                let mut us = ((byte5 & 0x1F) as u32) << 15;
                us |= (next_byte(&mut bytes)? as u32) << 7;
                let byte7 = next_byte(&mut bytes)?;
                us |= (byte7 >> 1) as u32;

                (FractionalSecond::Microseconds(us), byte7)
            }
            PrecisionTag::Nano => {
                let mut ns = ((byte5 & 0x3F) as u32) << 25;
                ns |= (next_byte(&mut bytes)? as u32) << 17;
                ns |= (next_byte(&mut bytes)? as u32) << 9;
                ns |= (next_byte(&mut bytes)? as u32) << 1;
                let byte9 = next_byte(&mut bytes)?;
                ns |= (byte9 >> 7) as u32;

                (FractionalSecond::Nanoseconds(ns), byte9)
            },
            PrecisionTag::None => (FractionalSecond::None, byte5),
        };

        let raw_offset = match precision {
            PrecisionTag::Milli => ((last_variable_byte & 0x07) << 4) | (next_byte(&mut bytes)? >> 4),
            PrecisionTag::Micro => ((last_variable_byte & 0x01) << 6) | (next_byte(&mut bytes)? >> 2),
            PrecisionTag::Nano => last_variable_byte & 0x7F,
            PrecisionTag::None => ((last_variable_byte & 0x1F) << 2) | (next_byte(&mut bytes)? >> 6),
        };
        let offset = match raw_offset {
            127 => OffsetValue::None,
            126 => OffsetValue::SpecifiedElsewhere,
            x => OffsetValue::UtcOffset(((x as i16) - 64) * 15)
        };

        Ok(DateTimeSubSecondOffset {
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
            frac_second: frac_second,
            offset: offset
        })
    }

    pub fn serialize_components<W: Write>(year: Option<u16>, month: Option<u8>, day: Option<u8>,
                                     hour: Option<u8>, minute: Option<u8>, second: Option<u8>,
                                     fractional_second: FractionalSecond, offset: OffsetValue,
                                     writer: &mut W)
                                     -> Result<usize, SerializationError> {
        check_option_outside_range(year, YEAR_MIN, YEAR_MAX, TemporalField::Year)?;
        check_option_outside_range(month, MONTH_MIN, MONTH_MAX, TemporalField::Month)?;
        check_option_outside_range(day, DAY_MIN, DAY_MAX, TemporalField::Day)?;
        check_option_outside_range(hour, HOUR_MIN, HOUR_MAX, TemporalField::Hour)?;
        check_option_outside_range(minute, MINUTE_MIN, MINUTE_MAX, TemporalField::Minute)?;
        check_option_outside_range(second, SECOND_MIN, SECOND_MAX, TemporalField::Second)?;

        let offset_num = encode_offset_num(offset)?;

        let (precision_tag, first_var_length_byte_fragment) = match fractional_second {
            FractionalSecond::Milliseconds(ms) => {
                check_outside_range(ms, MILLIS_MIN, MILLIS_MAX, TemporalField::FractionalSecond)?;
                (PRECISION_DTSO_MILLIS_TAG, (ms >> 5) as u8)
            },
            FractionalSecond::Microseconds(us) => {
                check_outside_range(us, MICROS_MIN, MICROS_MAX, TemporalField::FractionalSecond)?;
                (PRECISION_DTSO_MICROS_TAG, (us >> 15) as u8)
            },
            FractionalSecond::Nanoseconds(ns) => {
                check_outside_range(ns, NANOS_MIN, NANOS_MAX, TemporalField::FractionalSecond)?;
                (PRECISION_DTSO_NANOS_TAG, (ns >> 25) as u8)
            },
            FractionalSecond::None => (PRECISION_DTSO_NONE_TAG, offset_num >> 2),
        };

        let year_num = year.unwrap_or(YEAR_RAW_NONE);
        let month_num = month.map(|m| m - 1).unwrap_or(MONTH_RAW_NONE);
        let day_num = day.map(|d| d - 1).unwrap_or(DAY_RAW_NONE);
        let hour_num = hour.unwrap_or(HOUR_RAW_NONE);
        let minute_num = minute.unwrap_or(MINUTE_RAW_NONE);
        let second_num = second.unwrap_or(SECOND_RAW_NONE);

        let b0 = DATE_TIME_SUBSECOND_OFFSET_TAG | precision_tag | (year_num >> 9) as u8;
        let b1 = (year_num >> 1) as u8;
        let b2 = (year_num << 7) as u8 | (month_num << 3) | (day_num >> 2);
        let b3 = (day_num << 6) | (hour_num << 1) | (minute_num >> 5);
        let b4 = (minute_num << 3) | (second_num >> 3);
        let b5 = (second_num << 5) | first_var_length_byte_fragment;

        let mut buf = [b0, b1, b2, b3, b4, b5, 0, 0, 0, 0];

        // write variable length fractional second
        let slice_end_index = match fractional_second {
            FractionalSecond::None => {
                // tail end of offset
                buf[6] = offset_num << 6;
                7
            },
            FractionalSecond::Milliseconds(ms) => {
                buf[6] = ((ms << 3) as u8) | (offset_num >> 4);
                buf[7] = offset_num << 4;
                8
            },
            FractionalSecond::Microseconds(us) => {
                buf[6] = (us >> 7) as u8;
                buf[7] = ((us << 1) as u8) | offset_num >> 6;
                buf[8] = offset_num << 2;
                9
            },
            FractionalSecond::Nanoseconds(ns) => {
                buf[6] = (ns >> 17) as u8;
                buf[7] = (ns >> 9) as u8;
                buf[8] = (ns >> 1) as u8;
                buf[9] = (ns << 7) as u8 | offset_num;
                10
            }
        };

        write_array_map_err(&buf[0..slice_end_index], writer)
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        Self::serialize_components(self.year, self.month, self.day, self.hour, self.minute, self.second,
                              self.frac_second, self.offset, writer)
    }
}

impl Date for DateTimeSubSecondOffset {
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

impl Time for DateTimeSubSecondOffset {
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

impl SubSecond for DateTimeSubSecondOffset {
    fn fractional_second(&self) -> FractionalSecond {
        self.frac_second
    }
}

impl Offset for DateTimeSubSecondOffset {
    fn offset(&self) -> OffsetValue {
        self.offset
    }
}

impl Serializable for DateTimeSubSecondOffset {
    fn max_serialized_size() -> usize {
        10
    }

    fn serialized_size(&self) -> usize {
        match self.frac_second {
            FractionalSecond::Milliseconds(_) => 8,
            FractionalSecond::Microseconds(_) => 9,
            FractionalSecond::Nanoseconds(_) => 10,
            FractionalSecond::None => 7,
        }
    }
}
