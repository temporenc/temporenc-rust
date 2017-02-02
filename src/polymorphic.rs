use std::io::Read;

use super::{Serializable, Date, Time, SubSecond, Offset, DeserializationError, SerializationError, read_exact, check_option_outside_range, check_outside_range, write_array_map_err, encode_offset_num, TemporalField, OffsetValue, FractionalSecond, PrecisionTag, DateOnly, TimeOnly, DateTime, DateTimeOffset, DateTimeSubSecond, DateTimeSubSecondOffset, Type, YEAR_MAX, YEAR_MIN, MONTH_MAX, MONTH_MIN, DAY_MAX, DAY_MIN, HOUR_MAX, HOUR_MIN, MINUTE_MAX, MINUTE_MIN, SECOND_MAX, SECOND_MIN, MILLIS_MAX, MILLIS_MIN, MICROS_MAX, MICROS_MIN, NANOS_MAX, NANOS_MIN, DATE_TAG, TIME_TAG, DATE_TIME_TAG, DATE_TIME_OFFSET_TAG, DATE_TIME_SUBSECOND_TAG, DATE_TIME_SUBSECOND_OFFSET_TAG, YEAR_RAW_NONE, MONTH_RAW_NONE, DAY_RAW_NONE, HOUR_RAW_NONE, MINUTE_RAW_NONE, SECOND_RAW_NONE, PRECISION_DTSO_MASK, PRECISION_DTSO_MILLIS_TAG, PRECISION_DTSO_MICROS_TAG, PRECISION_DTSO_NANOS_TAG, PRECISION_DTSO_NONE_TAG};

struct Polymorphic {
    date: Option<DateOnly>,
    time: Option<TimeOnly>,
    frac_second: FractionalSecond,
    offset: OffsetValue
}

impl Polymorphic {
    pub fn deserialize<R: Read>(reader: &mut R) -> Result<Polymorphic, DeserializationError> {
        // largest types are 10 bytes
        let mut buf = [0; 10];
        // smallest types are 3 bytes
        read_exact(reader, &mut buf[0..3])?;

        let byte0 = buf[0];

        let t = if byte0 & 0b1110_0000 == DATE_TAG {
            Type::Date
        } else if byte0 & 0b1111_1110 == TIME_TAG {
            Type::Time
        } else if byte0 & 0b1100_0000 == DATE_TIME_TAG {
            Type::DateTime
        } else if byte0 & 0b1110_0000 == DATE_TIME_OFFSET_TAG {
            Type::DateTimeOffset
        } else if byte0 & 0b1100_0000 == DATE_TIME_SUBSECOND_TAG {
            Type::DateTimeSubSecond
        } else if byte0 & 0b1110_0000 == DATE_TIME_SUBSECOND_OFFSET_TAG {
            Type::DateTimeSubSecondOffset
        } else {
            return Err(DeserializationError::IncorrectTypeTag)
        };

        Ok(Polymorphic {
            date: None,
            time: None,
            frac_second: FractionalSecond::None,
            offset: OffsetValue::None
        })
    }

    pub fn date(&self) -> Option<&Date> {
        None
    }

    pub fn time(&self) -> Option<&Time> {
        None
    }

    pub fn sub_second(&self) -> Option<&SubSecond> {
        None
    }
}

impl Offset for Polymorphic {
    fn offset(&self) -> OffsetValue {
        OffsetValue::None
    }
}
