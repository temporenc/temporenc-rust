use std::io::{Read, Write};

use super::{Date, Serializable, DeserializationError, SerializationError, read_exact, check_option_outside_range, write_array_map_err, TypeTag, TemporalField, YEAR_MAX, YEAR_MIN, MONTH_MAX, MONTH_MIN, DAY_MAX, DAY_MIN, DATE_TAG, YEAR_RAW_NONE, MONTH_RAW_NONE, DAY_RAW_NONE};


#[derive(Debug)]
pub struct DateOnly {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>
}

impl DateOnly {
    pub fn deserialize<R: Read>(reader: &mut R) -> Result<DateOnly, DeserializationError> {
        let mut buf = [0; SERIALIZED_SIZE];
        read_exact(reader, &mut buf)?;

        let byte0 = buf[0];

        if !TypeTag::DateOnly.matches(byte0) {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        // 3-bit tag, 12-bit year, 4-bit month, 5-bit day
        // TTTY YYYY YYYY YYYM MMMD DDDD

        // bits 4-15
        let mut raw_year = ((byte0 & 0x1F) as u16) << 7;
        let byte1 = buf[1];
        raw_year |= (byte1 as u16) >> 1;

        let year = if raw_year == YEAR_RAW_NONE {
            None
        } else {
            Some(raw_year)
        };

        // bits 16-19
        let mut raw_month = (byte1 & 0x01) << 3;
        let byte2 = buf[2];
        raw_month |= (byte2 & 0xE0) >> 5;

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

        // TODO check types that don't saturate full range (e.g. month)
        Ok(DateOnly {
            year: year,
            month: month,
            day: day
        })
    }

    pub fn serialize_components<W: Write>(year: Option<u16>, month: Option<u8>, day: Option<u8>,
                                          writer: &mut W) -> Result<usize, SerializationError> {
        check_option_outside_range(year, YEAR_MIN, YEAR_MAX, TemporalField::Year)?;
        check_option_outside_range(month, MONTH_MIN, MONTH_MAX, TemporalField::Month)?;
        check_option_outside_range(day, DAY_MIN, DAY_MAX, TemporalField::Day)?;

        let year_num = year.unwrap_or(YEAR_RAW_NONE);
        let month_num = month.map(|m| m - 1).unwrap_or(MONTH_RAW_NONE);
        let day_num = day.map(|d| d - 1).unwrap_or(DAY_RAW_NONE);

        let b0 = DATE_TAG | ((year_num >> 7) as u8);
        let b1 = ((year_num << 1) as u8) | (month_num >> 3);
        let b2 = (month_num << 5) | day_num;

        write_array_map_err(&[b0, b1, b2], writer)
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        Self::serialize_components(self.year, self.month, self.day, writer)
    }
}

impl Date for DateOnly {
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

impl Serializable for DateOnly {
    fn max_serialized_size() -> usize {
        SERIALIZED_SIZE
    }

    fn serialized_size(&self) -> usize {
        SERIALIZED_SIZE
    }
}

const SERIALIZED_SIZE: usize = 3;
