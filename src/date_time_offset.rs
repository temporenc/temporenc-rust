use std::io::{Read, Write};

use super::*;

#[derive(Debug, PartialEq)]
pub struct DateTimeOffset {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    offset: u8
}

impl DateTimeOffset {
    #[inline]
    pub fn new(year: Option<u16>, month: Option<u8>, day: Option<u8>, hour: Option<u8>,
               minute: Option<u8>, second: Option<u8>, offset: OffsetValue) -> Result<DateTimeOffset, CreationError> {
        Ok(DateTimeOffset {
            year: year_num(year)?,
            month: month_num(month)?,
            day: day_num(day)?,
            hour: hour_num(hour)?,
            minute: minute_num(minute)?,
            second: second_num(second)?,
            offset: offset_num(offset)?
        })
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<DateTimeOffset, DeserializationError> {
        let mut buf = [0; SERIALIZED_SIZE];
        read_exact(reader, &mut buf)?;

        let byte0 = buf[0];

        if byte0 & 0b1110_0000 != DATE_TIME_OFFSET_TAG {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        // 3-bit tag, 12-bit year, 4-bit month, 5-bit day, 5-bit hour, 6-bit minute, 6-bit second,
        // 7-bit offset
        // TTTY YYYY | YYYY YYYM | MMMD DDDD | HHHH HMMM | MMMS SSSS | SOOO OOOO

        let mut raw_year = ((byte0 & 0x1F) as u16) << 7;
        let byte1 = buf[1];
        raw_year |= (byte1 as u16) >> 1;

        let byte2 = buf[2];
        let raw_month = ((byte1 & 0x01) << 3) | ((byte2 & 0xE0) >> 5);

        let raw_day = byte2 & 0x1F;

        let byte3 = buf[3];
        let raw_hour = byte3 >> 3;

        let byte4 = buf[4];
        let raw_minute = ((byte3 & 0x07) << 3) | (byte4 >> 5);

        let byte5 = buf[5];
        let raw_second = (byte4 & 0x1F) << 1 | (byte5 >> 7);

        let raw_offset = byte5 & 0x7F;

        // no need to check year as every possible number is a valid year
        check_deser_in_range_or_none(raw_month, MONTH_RAW_MIN, MONTH_RAW_MAX, MONTH_RAW_NONE)?;
        // no need to check day as every possible number is a valid day
        check_deser_in_range_or_none(raw_hour, HOUR_MIN, HOUR_MAX, HOUR_RAW_NONE)?;
        check_deser_in_range_or_none(raw_minute, MINUTE_MIN, MINUTE_MAX, MINUTE_RAW_NONE)?;
        check_deser_in_range_or_none(raw_second, SECOND_MIN, SECOND_MAX, SECOND_RAW_NONE)?;
        // no need to check offset as every possible number is a valid offset

        Ok(DateTimeOffset {
            year: raw_year,
            month: raw_month,
            day: raw_day,
            hour: raw_hour,
            minute: raw_minute,
            second: raw_second,
            offset: raw_offset
        })
    }

}

impl Date for DateTimeOffset {
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

impl Time for DateTimeOffset {
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

impl Offset for DateTimeOffset {
    fn offset(&self) -> OffsetValue {
        match self.offset {
            127 => OffsetValue::None,
            126 => OffsetValue::SpecifiedElsewhere,
            x => OffsetValue::UtcOffset(((x as i16) - 64) * 15)
        }
    }
}

impl Serializable for DateTimeOffset {
    fn max_serialized_size() -> usize {
        SERIALIZED_SIZE
    }

    fn serialized_size(&self) -> usize {
        SERIALIZED_SIZE
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        let b0 = DATE_TIME_OFFSET_TAG | (self.year >> 7) as u8;
        let b1 = ((self.year << 1) as u8) | (self.month >> 3);
        let b2 = (self.month << 5) | self.day;
        let b3 = (self.hour << 3) | (self.minute >> 3);
        let b4 = (self.minute << 5) | (self.second >> 1);
        let b5 = (self.second << 7) | self.offset;

        write_array_map_err(&[b0, b1, b2, b3, b4, b5], writer)
            .map_err(|_| SerializationError::IoError)
    }
}

const SERIALIZED_SIZE: usize = 6;
