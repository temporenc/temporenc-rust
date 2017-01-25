use std::io::{Read, Write};

use super::{Time, DeserializationError, SerializationError, next_byte, check_option_outside_range, write_map_err, TypeTag, TemporalField, HOUR_MAX, HOUR_MIN, MINUTE_MAX, MINUTE_MIN, SECOND_MAX, SECOND_MIN, TIME_TAG, HOUR_RAW_NONE, MINUTE_RAW_NONE, SECOND_RAW_NONE};


#[derive(Debug)]
pub struct TimeOnly {
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
}

impl TimeOnly {
    pub fn deserialize<R: Read>(reader: R) -> Result<TimeOnly, DeserializationError> {
        let mut bytes = reader.bytes();

        let byte0 = next_byte(&mut bytes)?;
        if !TypeTag::TimeOnly.matches(byte0) {
            return Err(DeserializationError::IncorrectTypeTag);
        }

        // 7-bit tag, 5-bit hour, 6-bit minute, 6-bit second
        // TTTT TTTH HHHH MMMM MMSS SSSS

        // bits 8-12
        let mut raw_hour = byte0 << 4;
        let byte1 = next_byte(&mut bytes)?;
        raw_hour |= (byte1 & 0xF0) >> 4;
        let hour = if raw_hour == HOUR_RAW_NONE {
            None
        } else {
            Some(raw_hour)
        };

        // bits 13-18
        let mut raw_minute = (byte1 & 0x0F) << 2;
        let byte2 = next_byte(&mut bytes)?;
        raw_minute |= (byte2 & 0xC0) >> 6;
        let minute = if raw_minute == MINUTE_RAW_NONE {
            None
        } else {
            Some(raw_minute)
        };

        // bits 19-24
        let raw_second = byte2 & 0x3F;
        let second = if raw_second == SECOND_RAW_NONE {
            None
        } else {
            Some(raw_second)
        };

        Ok(TimeOnly {
            hour: hour,
            minute: minute,
            second: second
        })
    }

    pub fn serialize<W: Write>(hour: Option<u8>, minute: Option<u8>, second: Option<u8>, writer: &mut W)
                               -> Result<usize, SerializationError> {
        check_option_outside_range(hour, HOUR_MIN, HOUR_MAX, TemporalField::Hour)?;
        check_option_outside_range(minute, MINUTE_MIN, MINUTE_MAX, TemporalField::Minute)?;
        check_option_outside_range(second, SECOND_MIN, SECOND_MAX, TemporalField::Second)?;

        let hour_num = hour.unwrap_or(HOUR_RAW_NONE);
        let minute_num = minute.unwrap_or(MINUTE_RAW_NONE);
        let second_num = second.unwrap_or(SECOND_RAW_NONE);

        let b1 = TIME_TAG | hour_num >> 4;
        let mut bytes_written = write_map_err(b1, writer)?;
        let b2 = (hour_num << 4) | (minute_num >> 2);
        bytes_written += write_map_err(b2, writer)?;
        let b3 = (minute_num << 6) | (second_num);
        bytes_written += write_map_err(b3, writer)?;

        Ok(bytes_written)
    }
}

impl Time for TimeOnly {
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
