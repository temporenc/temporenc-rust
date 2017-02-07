use std::io::{Read, Write};

use super::*;

/// Just a Time.
#[derive(Debug, PartialEq)]
pub struct TimeOnly {
    hour: u8,
    minute: u8,
    second: u8,
}

impl TimeOnly {
    #[inline]
    pub fn new(hour: Option<u8>, minute: Option<u8>, second: Option<u8>) -> Result<TimeOnly, CreationError> {
        Ok(TimeOnly {
            hour: hour_num(hour)?,
            minute: minute_num(minute)?,
            second: second_num(second)?,
        })
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

    fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        let b0 = TIME_TAG | self.hour >> 4;
        let b1 = (self.hour << 4) | (self.minute >> 2);
        let b2 = (self.minute << 6) | (self.second);

        write_array_map_err(&[b0, b1, b2], writer)
            .map_err(|_| SerializationError::IoError)
    }

}

impl Deserializable for TimeOnly {
    fn deserialize<R: Read>(reader: &mut R) -> Result<TimeOnly, DeserializationError> {
        let mut buf = [0; SERIALIZED_SIZE];
        read_exact(reader, &mut buf)?;

        let byte0 = buf[0];
        if byte0 & 0b1111_1110 != TIME_TAG {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        // 7-bit tag, 5-bit hour, 6-bit minute, 6-bit second
        // TTTT TTTH HHHH MMMM MMSS SSSS

        let mut raw_hour = byte0 << 4;
        let byte1 = buf[1];
        raw_hour |= (byte1 & 0xF0) >> 4;

        let mut raw_minute = (byte1 & 0x0F) << 2;
        let byte2 = buf[2];
        raw_minute |= (byte2 & 0xC0) >> 6;

        let raw_second = byte2 & 0x3F;

        check_deser_in_range_or_none(raw_hour, HOUR_MIN, HOUR_MAX, HOUR_RAW_NONE)?;
        check_deser_in_range_or_none(raw_minute, MINUTE_MIN, MINUTE_MAX, MINUTE_RAW_NONE)?;
        check_deser_in_range_or_none(raw_second, SECOND_MIN, SECOND_MAX, SECOND_RAW_NONE)?;

        Ok(TimeOnly {
            hour: raw_hour,
            minute: raw_minute,
            second: raw_second
        })
    }
}

const SERIALIZED_SIZE: usize = 3;
