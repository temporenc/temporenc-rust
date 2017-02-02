use std::io::{Read, Write};

use super::{Serializable, Date, Time, DeserializationError, SerializationError, read_exact, check_option_in_range, write_array_map_err, check_deser_in_range_or_none, YEAR_MAX, YEAR_MIN, MONTH_MAX, MONTH_MIN, DAY_MAX, DAY_MIN, HOUR_MAX, HOUR_MIN, MINUTE_MAX, MINUTE_MIN, SECOND_MAX, SECOND_MIN, DATE_TIME_TAG, YEAR_RAW_NONE, MONTH_RAW_NONE, DAY_RAW_NONE, HOUR_RAW_NONE, MINUTE_RAW_NONE, SECOND_RAW_NONE, MONTH_RAW_MIN, MONTH_RAW_MAX};


#[derive(Debug)]
pub struct DateTime {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl DateTime {
    pub fn deserialize<R: Read>(reader: &mut R) -> Result<DateTime, DeserializationError> {
        let mut buf = [0; SERIALIZED_SIZE];
        read_exact(reader, &mut buf)?;

        let byte0 = buf[0];

        if byte0 & 0b1100_0000 != DATE_TIME_TAG {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        // 2-bit tag, 12-bit year, 4-bit month, 5-bit day, 5-bit hour, 6-bit minute, 6-bit second
        // TTYY YYYY | YYYY YYMM | MMDD DDDH | HHHH MMMM | MMSS SSSS

        let byte1 = buf[1];
        let mut raw_year = ((byte0 & 0x3F) as u16) << 6;
        raw_year |= (byte1 >> 2) as u16;

        let byte2 = buf[2];
        let raw_month = ((byte1 & 0x03) << 2) | (byte2 >> 6);

        let raw_day = (byte2 & 0x3E) >> 1;

        let byte3 = buf[3];
        let raw_hour = ((byte2 & 0x01) << 4) | (byte3 >> 4);

        let byte4 = buf[4];
        let raw_minute = ((byte3 & 0x0F) << 2) | (byte4 >> 6);

        let raw_second = byte4 & 0x3F;

        // no need to check year as every possible number is a valid year
        check_deser_in_range_or_none(raw_month, MONTH_RAW_MIN, MONTH_RAW_MAX, MONTH_RAW_NONE)?;
        // no need to check day as every possible number is a valid day
        check_deser_in_range_or_none(raw_hour, HOUR_MIN, HOUR_MAX, HOUR_RAW_NONE)?;
        check_deser_in_range_or_none(raw_minute, MINUTE_MIN, MINUTE_MAX, MINUTE_RAW_NONE)?;
        check_deser_in_range_or_none(raw_second, SECOND_MIN, SECOND_MAX, SECOND_RAW_NONE)?;

        Ok(DateTime {
            year: raw_year,
            month: raw_month,
            day: raw_day,
            hour: raw_hour,
            minute: raw_minute,
            second: raw_second,
        })
    }

    pub fn serialize_components<W: Write>(year: Option<u16>, month: Option<u8>, day: Option<u8>,
                                          hour: Option<u8>, minute: Option<u8>, second: Option<u8>,
                                          writer: &mut W) -> Result<usize, SerializationError> {
        check_option_in_range(year, YEAR_MIN, YEAR_MAX)?;
        check_option_in_range(month, MONTH_MIN, MONTH_MAX)?;
        check_option_in_range(day, DAY_MIN, DAY_MAX)?;
        check_option_in_range(hour, HOUR_MIN, HOUR_MAX)?;
        check_option_in_range(minute, MINUTE_MIN, MINUTE_MAX)?;
        check_option_in_range(second, SECOND_MIN, SECOND_MAX)?;

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
        Self::serialize_components(self.year(), self.month(), self.day(), self.hour(),
                                   self.minute(), self.second(), writer)
    }
}

impl Date for DateTime {
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

impl Time for DateTime {
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

impl Serializable for DateTime {
    fn max_serialized_size() -> usize {
        SERIALIZED_SIZE
    }

    fn serialized_size(&self) -> usize {
        SERIALIZED_SIZE
    }
}

const SERIALIZED_SIZE: usize = 5;
