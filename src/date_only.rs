use std::io::{Read, Write, Error};

use super::*;

#[derive(Debug)]
pub struct DateOnly {
    year: u16,
    month: u8,
    day: u8
}

impl DateOnly {
    #[inline]
    pub fn new(year: Option<u16>, month: Option<u8>, day: Option<u8>)
               -> Result<DateOnly, CreationError> {
        let err_val = CreationError::InvalidFieldValue;

        Ok(DateOnly {
            year: year_num(year, err_val)?,
            month: month_num(month, err_val)?,
            day: day_num(day, err_val)?,
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

    pub fn serialize_components<W: Write>(year: Option<u16>, month: Option<u8>, day: Option<u8>,
                                          writer: &mut W) -> Result<usize, ComponentSerializationError> {
        let err_val = ComponentSerializationError::InvalidFieldValue;

        Self::serialize_raw(year_num(year, err_val)?, month_num(month, err_val)?,
                            day_num(day, err_val)?, writer)
            .map_err(|_| ComponentSerializationError::IoError)
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        Self::serialize_raw(self.year, self.month, self.day, writer)
            .map_err(|_| SerializationError::IoError)
    }

    fn serialize_raw<W: Write>(year: u16, month: u8, day: u8, writer: &mut W) -> Result<usize, Error> {
        let b0 = DATE_TAG | ((year >> 7) as u8);
        let b1 = ((year << 1) as u8) | (month >> 3);
        let b2 = (month << 5) | day;

        write_array_map_err(&[b0, b1, b2], writer)
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
}

const SERIALIZED_SIZE: usize = 3;
