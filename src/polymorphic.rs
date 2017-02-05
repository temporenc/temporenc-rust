use std::io::Read;

use super::*;

struct Polymorphic {
    date: Option<Box<Date>>,
    time: Option<Box<Time>>,
    frac_second: Option<FractionalSecond>,
    offset: Option<OffsetValue>,
    temporal_type: TemporalType
}

impl Polymorphic {
    pub fn deserialize<R: Read>(reader: &mut R) -> Result<Polymorphic, DeserializationError> {
        // read 1 byte
        let mut buf = [0; 1];
        read_exact(reader, &mut buf[0..])?;
        let byte0 = buf[0];

        // let deser code read the byte we just read from reader
        let mut chain = buf.chain(reader);

        return if byte0 & 0b1110_0000 == DATE_TAG {
            Ok(Polymorphic {
                date: Some(Box::new(DateOnly::deserialize(&mut chain)?)),
                time: None,
                frac_second: None,
                offset: None,
                temporal_type: TemporalType::Date
            })
        } else if byte0 & 0b1111_1110 == TIME_TAG {
            Ok(Polymorphic {
                date: None,
                time: Some(Box::new(TimeOnly::deserialize(&mut chain)?)),
                frac_second: None,
                offset: None,
                temporal_type: TemporalType::Time
            })
        } else if byte0 & 0b1100_0000 == DATE_TIME_TAG {
            let d = DateTime::deserialize(&mut chain)?;
            Ok(Polymorphic {
                date: Some(Box::new(d)),
                time: Some(Box::new(d)),
                frac_second: None,
                offset: None,
                temporal_type: TemporalType::DateTime
            })
        } else if byte0 & 0b1110_0000 == DATE_TIME_OFFSET_TAG {
            let d = DateTimeOffset::deserialize(&mut chain)?;

            Ok(Polymorphic {
                date: Some(Box::new(d)),
                time: Some(Box::new(d)),
                frac_second: None,
                offset: Some(d.offset()),
                temporal_type: TemporalType::DateTimeOffset
            })
        } else if byte0 & 0b1100_0000 == DATE_TIME_SUBSECOND_TAG {
            let d = DateTimeSubSecond::deserialize(&mut chain)?;

            Ok(Polymorphic {
                date: Some(Box::new(d)),
                time: Some(Box::new(d)),
                frac_second: Some(d.fractional_second()),
                offset: None,
                temporal_type: TemporalType::DateTimeSubSecond
            })
        } else if byte0 & 0b1110_0000 == DATE_TIME_SUBSECOND_OFFSET_TAG {
            let d = DateTimeSubSecondOffset::deserialize(&mut chain)?;

            Ok(Polymorphic {
                date: Some(Box::new(d)),
                time: Some(Box::new(d)),
                frac_second: Some(d.fractional_second()),
                offset: Some(d.offset()),
                temporal_type: TemporalType::DateTimeSubSecondOffset
            })
        } else {
            return Err(DeserializationError::IncorrectTypeTag)
        };
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

    pub fn offset(&self) -> Option<&Offset> {
        None
    }

    pub fn temporal_type(&self) -> TemporalType {
        self.temporal_type
    }
}
