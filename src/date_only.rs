use std::io::{Read, Write};

use super::*;

#[derive(Debug, PartialEq)]
pub struct DateOnly {
    year: u16,
    month: u8,
    day: u8
}

impl DateOnly {
    #[inline]
    pub fn new(year: Option<u16>, month: Option<u8>, day: Option<u8>)
               -> Result<DateOnly, CreationError> {
        Ok(DateOnly {
            year: year_num(year)?,
            month: month_num(month)?,
            day: day_num(day)?,
        })
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<DateOnly, DeserializationError> {
        let mut buf = [0; SERIALIZED_SIZE];
        read_exact(reader, &mut buf)?;

        let byte0 = buf[0];

        if byte0 & 0b1110_0000 != DATE_TAG {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        // 3-bit tag, 12-bit year, 4-bit month, 5-bit day
        // TTTY YYYY YYYY YYYM MMMD DDDD

        let mut raw_year = ((byte0 & 0x1F) as u16) << 7;
        let byte1 = buf[1];
        raw_year |= (byte1 as u16) >> 1;

        let mut raw_month = (byte1 & 0x01) << 3;
        let byte2 = buf[2];
        raw_month |= (byte2 & 0xE0) >> 5;

        let raw_day = byte2 & 0x1F;

        // no need to check year as every possible number is a valid year
        check_deser_in_range_or_none(raw_month, MONTH_RAW_MIN, MONTH_RAW_MAX, MONTH_RAW_NONE)?;
        // no need to check day as every possible number is a valid day

        Ok(DateOnly {
            year: raw_year,
            month: raw_month,
            day: raw_day
        })
    }

}

impl Date for DateOnly {
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

impl Serializable for DateOnly {
    fn max_serialized_size() -> usize {
        SERIALIZED_SIZE
    }

    fn serialized_size(&self) -> usize {
        SERIALIZED_SIZE
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        let b0 = DATE_TAG | ((self.year >> 7) as u8);
        let b1 = ((self.year << 1) as u8) | (self.month >> 3);
        let b2 = (self.month << 5) | self.day;

        write_array_map_err(&[b0, b1, b2], writer)
            .map_err(|_| SerializationError::IoError)
    }
}

const SERIALIZED_SIZE: usize = 3;
