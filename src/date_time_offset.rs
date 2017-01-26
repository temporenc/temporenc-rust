use std::io::{Read, Write};

use super::{Serializable, Date, Time, Offset, DeserializationError, SerializationError, next_byte, check_option_outside_range, check_outside_range, write_map_err, TypeTag, TemporalField, OffsetValue, YEAR_MAX, YEAR_MIN, MONTH_MAX, MONTH_MIN, DAY_MAX, DAY_MIN, HOUR_MAX, HOUR_MIN, MINUTE_MAX, MINUTE_MIN, SECOND_MAX, SECOND_MIN, OFFSET_MAX, OFFSET_MIN, DATE_TIME_OFFSET_TAG, YEAR_RAW_NONE, MONTH_RAW_NONE, DAY_RAW_NONE, HOUR_RAW_NONE, MINUTE_RAW_NONE, SECOND_RAW_NONE, OFFSET_RAW_NONE, OFFSET_RAW_ELSEWHERE};


#[derive(Debug)]
pub struct DateTimeOffset {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
    offset: OffsetValue
}

impl DateTimeOffset {
    pub fn deserialize<R: Read>(reader: R) -> Result<DateTimeOffset, DeserializationError> {
        let mut bytes = reader.bytes();
        let byte0 = next_byte(&mut bytes)?;

        if !TypeTag::DateTimeOffset.matches(byte0) {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        // 3-bit tag, 12-bit year, 4-bit month, 5-bit day, 5-bit hour, 6-bit minute, 6-bit second,
        // 7-bit offset
        // TTTY YYYY | YYYY YYYM | MMMD DDDD | HHHH HMMM | MMMS SSSS | SOOO OOOO

        // bits 4-15
        let mut raw_year = ((byte0 & 0x1F) as u16) << 7;
        let byte1 = next_byte(&mut bytes)?;
        raw_year |= (byte1 as u16) >> 1;
        let year = if raw_year == YEAR_RAW_NONE {
            None
        } else {
            Some(raw_year)
        };

        // bits 16-19
        let byte2 = next_byte(&mut bytes)?;
        let raw_month = ((byte1 & 0x01) << 3) | ((byte2 & 0xE0) >> 5);
        let month = if raw_month == MONTH_RAW_NONE {
            None
        } else {
            Some(raw_month + 1)
        };

        // bits 20-24
        let raw_day = byte2 & 0x1F;
        let day = if raw_day == DAY_RAW_NONE {
            None
        } else {
            Some(raw_day + 1)
        };

        // bits 25-29
        let byte3 = next_byte(&mut bytes)?;
        let raw_hour = byte3 >> 3;
        let hour = if raw_hour == HOUR_RAW_NONE {
            None
        } else {
            Some(raw_hour)
        };

        // bits 30-35
        let byte4 = next_byte(&mut bytes)?;
        let raw_minute = ((byte3 & 0x07) << 3) | (byte4 >> 5);
        let minute = if raw_minute == MINUTE_RAW_NONE {
            None
        } else {
            Some(raw_minute)
        };

        // bits 36-41
        let byte5 = next_byte(&mut bytes)?;
        let raw_second = (byte4 & 0x1F) << 1 | (byte5 >> 7);
        let second = if raw_second == SECOND_RAW_NONE {
            None
        } else {
            Some(raw_second)
        };

        let raw_offset = byte5 & 0x7F;
        let offset = match raw_offset {
            127 => OffsetValue::None,
            126 => OffsetValue::SpecifiedElsewhere,
            x => OffsetValue::UtcOffset(((x as i16) - 64) * 15)
        };

        Ok(DateTimeOffset {
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
            offset: offset
        })
    }

    pub fn serialize_components<W: Write>(year: Option<u16>, month: Option<u8>, day: Option<u8>,
                                          hour: Option<u8>, minute: Option<u8>, second: Option<u8>,
                                          offset: OffsetValue, writer: &mut W)
                                            -> Result<usize, SerializationError> {
        check_option_outside_range(year, YEAR_MIN, YEAR_MAX, TemporalField::Year)?;
        check_option_outside_range(month, MONTH_MIN, MONTH_MAX, TemporalField::Month)?;
        check_option_outside_range(day, DAY_MIN, DAY_MAX, TemporalField::Day)?;
        check_option_outside_range(hour, HOUR_MIN, HOUR_MAX, TemporalField::Hour)?;
        check_option_outside_range(minute, MINUTE_MIN, MINUTE_MAX, TemporalField::Minute)?;
        check_option_outside_range(second, SECOND_MIN, SECOND_MAX, TemporalField::Second)?;

        let offset_num = encode_offset_num(offset)?;

        let year_num = year.unwrap_or(YEAR_RAW_NONE);
        let month_num = month.map(|m| m - 1).unwrap_or(MONTH_RAW_NONE);
        let day_num = day.map(|d| d - 1).unwrap_or(DAY_RAW_NONE);
        let hour_num = hour.unwrap_or(HOUR_RAW_NONE);
        let minute_num = minute.unwrap_or(MINUTE_RAW_NONE);
        let second_num = second.unwrap_or(SECOND_RAW_NONE);

        let mut bytes_written = write_map_err(DATE_TIME_OFFSET_TAG | (year_num >> 7) as u8, writer)?;
        bytes_written += write_map_err(((year_num << 1) as u8) | (month_num >> 3), writer)?;
        bytes_written += write_map_err((month_num << 5) | day_num, writer)?;
        bytes_written += write_map_err((hour_num << 3) | (minute_num >> 3), writer)?;
        bytes_written += write_map_err((minute_num << 5) | (second_num >> 1), writer)?;
        bytes_written += write_map_err((second_num << 7) | offset_num, writer)?;

        Ok(bytes_written)
    }


    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        Self::serialize_components(self.year, self.month, self.day, self.hour, self.minute,
                                   self.second, self.offset, writer)
    }
}

impl Date for DateTimeOffset {
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

impl Time for DateTimeOffset {
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

impl Offset for DateTimeOffset {
    fn offset(&self) -> OffsetValue {
        self.offset
    }
}

pub fn encode_offset_num(offset: OffsetValue) -> Result<u8, SerializationError> {
    match offset {
        OffsetValue::None => Ok(OFFSET_RAW_NONE),
        OffsetValue::SpecifiedElsewhere => Ok(OFFSET_RAW_ELSEWHERE),
        OffsetValue::UtcOffset(o) => {
            check_outside_range(o, OFFSET_MIN, OFFSET_MAX, TemporalField::Offset)?;

            if o % 15 != 0 {
                return Err(SerializationError::InvalidFieldValue(TemporalField::Offset));
            };

            Ok(((o / 15) + 64) as u8)
        }
    }
}

impl Serializable for DateTimeOffset {
    fn max_serialized_size() -> usize {
        6
    }

    fn serialized_size(&self) -> usize {
        Self::max_serialized_size()
    }
}