use std::io::{Read, Write, Error};

use super::*;

#[derive(Debug)]
pub struct TimeOnly {
    hour: u8,
    minute: u8,
    second: u8,
}

impl TimeOnly {
    pub fn new(hour: Option<u8>, minute: Option<u8>, second: Option<u8>) -> Result<TimeOnly, CreationError> {
        check_hour_option(hour, CreationError::InvalidFieldValue)?;
        check_minute_option(minute, CreationError::InvalidFieldValue)?;
        check_second_option(second, CreationError::InvalidFieldValue)?;

        Ok(TimeOnly {
            hour: hour_num(hour),
            minute: minute_num(minute),
            second: second_num(second),
        })
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<TimeOnly, DeserializationError> {
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

    pub fn serialize_components<W: Write>(hour: Option<u8>, minute: Option<u8>, second: Option<u8>, writer: &mut W)
                                          -> Result<usize, ComponentSerializationError> {
        check_hour_option(hour, ComponentSerializationError::InvalidFieldValue)?;
        check_minute_option(minute, ComponentSerializationError::InvalidFieldValue)?;
        check_second_option(second, ComponentSerializationError::InvalidFieldValue)?;

        Self::serialize_raw(hour_num(hour), minute_num(minute), second_num(second), writer)
            .map_err(|_| ComponentSerializationError::IoError)
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<usize, SerializationError> {
        Self::serialize_raw(self.hour, self.minute, self.second, writer)
            .map_err(|_| SerializationError::IoError)
    }

    fn serialize_raw<W: Write>(hour: u8, minute: u8, second: u8, writer: &mut W) -> Result<usize, Error> {
        let b0 = TIME_TAG | hour >> 4;
        let b1 = (hour << 4) | (minute >> 2);
        let b2 = (minute << 6) | (second);

        write_array_map_err(&[b0, b1, b2], writer)
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
