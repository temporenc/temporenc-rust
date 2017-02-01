use std::io::{Read, Write};

use super::{Serializable, Time, DeserializationError, SerializationError, read_exact, check_option_outside_range, write_array_map_err, TemporalField, HOUR_MAX, HOUR_MIN, MINUTE_MAX, MINUTE_MIN, SECOND_MAX, SECOND_MIN, TIME_TAG, HOUR_RAW_NONE, MINUTE_RAW_NONE, SECOND_RAW_NONE};


#[derive(Debug)]
pub struct TimeOnly {
    hour: u8,
    minute: u8,
    second: u8,
}

impl TimeOnly {
    pub fn deserialize<R: Read>(reader: &mut R) -> Result<TimeOnly, DeserializationError> {
        let mut buf = [0; SERIALIZED_SIZE];
        read_exact(reader, &mut buf)?;

        let byte0 = buf[0];
        if byte0 & 0b1111_1110 != TIME_TAG {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        // 7-bit tag, 5-bit hour, 6-bit minute, 6-bit second
        // TTTT TTTH HHHH MMMM MMSS SSSS

        // bits 8-12
        let mut raw_hour = byte0 << 4;
        let byte1 = buf[1];
        raw_hour |= (byte1 & 0xF0) >> 4;

        // bits 13-18
        let mut raw_minute = (byte1 & 0x0F) << 2;
        let byte2 = buf[2];
        raw_minute |= (byte2 & 0xC0) >> 6;

        // bits 19-24
        let raw_second = byte2 & 0x3F;

        Ok(TimeOnly {
            hour: raw_hour,
            minute: raw_minute,
            second: raw_second
        })
    }

    pub fn serialize_components<W: Write>(hour: Option<u8>, minute: Option<u8>, second: Option<u8>, writer: &mut W)
                                      -> Result<usize, SerializationError> {
        check_option_outside_range(hour, HOUR_MIN, HOUR_MAX, TemporalField::Hour)?;
        check_option_outside_range(minute, MINUTE_MIN, MINUTE_MAX, TemporalField::Minute)?;
        check_option_outside_range(second, SECOND_MIN, SECOND_MAX, TemporalField::Second)?;

        let hour_num = hour.unwrap_or(HOUR_RAW_NONE);
        let minute_num = minute.unwrap_or(MINUTE_RAW_NONE);
        let second_num = second.unwrap_or(SECOND_RAW_NONE);

        let b0 = TIME_TAG | hour_num >> 4;
        let b1 = (hour_num << 4) | (minute_num >> 2);
        let b2 = (minute_num << 6) | (second_num);

        write_array_map_err(&[b0, b1, b2], writer)
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        Self::serialize_components(self.hour(), self.minute(), self.second(), writer)
    }
}

impl Time for TimeOnly {
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

impl Serializable for TimeOnly {
    fn max_serialized_size() -> usize {
        SERIALIZED_SIZE
    }

    fn serialized_size(&self) -> usize {
        SERIALIZED_SIZE
    }
}

const SERIALIZED_SIZE: usize = 3;
