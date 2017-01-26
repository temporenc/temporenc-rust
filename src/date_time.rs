use std::io::{Read, Write};

use super::{Serializable, Date, Time, DeserializationError, SerializationError, next_byte, check_option_outside_range, write_array_map_err, TypeTag, TemporalField, YEAR_MAX, YEAR_MIN, MONTH_MAX, MONTH_MIN, DAY_MAX, DAY_MIN, HOUR_MAX, HOUR_MIN, MINUTE_MAX, MINUTE_MIN, SECOND_MAX, SECOND_MIN, DATE_TIME_TAG, YEAR_RAW_NONE, MONTH_RAW_NONE, DAY_RAW_NONE, HOUR_RAW_NONE, MINUTE_RAW_NONE, SECOND_RAW_NONE};


#[derive(Debug)]
pub struct DateTime {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
}

impl DateTime {
    pub fn deserialize<R: Read>(reader: R) -> Result<DateTime, DeserializationError> {
        let mut bytes = reader.bytes();
        let byte0 = next_byte(&mut bytes)?;

        if !TypeTag::DateTime.matches(byte0) {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        // 2-bit tag, 12-bit year, 4-bit month, 5-bit day, 5-bit hour, 6-bit minute, 6-bit second
        // TTYY YYYY | YYYY YYMM | MMDD DDDH | HHHH MMMM | MMSS SSSS

        // bits 3-14
        let byte1 = next_byte(&mut bytes)?;
        let mut raw_year = ((byte0 & 0x3F) as u16) << 6;
        raw_year |= (byte1 >> 2) as u16;
        let year = if raw_year == YEAR_RAW_NONE {
            None
        } else {
            Some(raw_year)
        };

        // bits 15-18
        let byte2 = next_byte(&mut bytes)?;
        let raw_month = ((byte1 & 0x03) << 2) | (byte2 >> 6);
        let month = if raw_month == MONTH_RAW_NONE {
            None
        } else {
            Some(raw_month + 1)
        };

        // bits 19-23
        let raw_day = (byte2 & 0x3E) >> 1;
        let day = if raw_day == DAY_RAW_NONE {
            None
        } else {
            Some(raw_day + 1)
        };

        // bits 24-28
        let byte3 = next_byte(&mut bytes)?;
        let raw_hour = ((byte2 & 0x01) << 4) | (byte3 >> 4);
        let hour = if raw_hour == HOUR_RAW_NONE {
            None
        } else {
            Some(raw_hour)
        };

        // bits 29-34
        let byte4 = next_byte(&mut bytes)?;
        let raw_minute = ((byte3 & 0x0F) << 2) | (byte4 >> 6);
        let minute = if raw_minute == MINUTE_RAW_NONE {
            None
        } else {
            Some(raw_minute)
        };

        // bits 35-40
        let raw_second = byte4 & 0x3F;
        let second = if raw_second == SECOND_RAW_NONE {
            None
        } else {
            Some(raw_second)
        };

        Ok(DateTime {
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
        })
    }

    pub fn serialize_components<W: Write>(year: Option<u16>, month: Option<u8>, day: Option<u8>,
                                          hour: Option<u8>, minute: Option<u8>, second: Option<u8>,
                                          writer: &mut W) -> Result<usize, SerializationError> {
        check_option_outside_range(year, YEAR_MIN, YEAR_MAX, TemporalField::Year)?;
        check_option_outside_range(month, MONTH_MIN, MONTH_MAX, TemporalField::Month)?;
        check_option_outside_range(day, DAY_MIN, DAY_MAX, TemporalField::Day)?;
        check_option_outside_range(hour, HOUR_MIN, HOUR_MAX, TemporalField::Hour)?;
        check_option_outside_range(minute, MINUTE_MIN, MINUTE_MAX, TemporalField::Minute)?;
        check_option_outside_range(second, SECOND_MIN, SECOND_MAX, TemporalField::Second)?;

        let year_num = year.unwrap_or(YEAR_RAW_NONE);
        let month_num = month.map(|m| m - 1).unwrap_or(MONTH_RAW_NONE);
        let day_num = day.map(|d| d - 1).unwrap_or(DAY_RAW_NONE);
        let hour_num = hour.unwrap_or(HOUR_RAW_NONE);
        let minute_num = minute.unwrap_or(MINUTE_RAW_NONE);
        let second_num = second.unwrap_or(SECOND_RAW_NONE);

        let b0 = DATE_TIME_TAG | (year_num >> 6) as u8;
        let b1 = ((year_num << 2) as u8) | (month_num >> 2);
        let b2 = (month_num << 6) | (day_num << 1) | (hour_num >> 4);
        let b3 = (hour_num << 4) | (minute_num >> 2);
        let b4 = (minute_num << 6) | second_num;

        write_array_map_err(&[b0, b1, b2, b3, b4], writer)
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        Self::serialize_components(self.year, self.month, self.day, self.hour, self.minute, self.second,
                              writer)
    }
}

impl Date for DateTime {
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

impl Time for DateTime {
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

impl Serializable for DateTime {
    fn max_serialized_size() -> usize {
        5
    }

    fn serialized_size(&self) -> usize {
        Self::max_serialized_size()
    }
}
