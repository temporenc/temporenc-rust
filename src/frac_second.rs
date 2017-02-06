#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FractionalSecond {
    Milliseconds(u16),
    Microseconds(u32),
    Nanoseconds(u32),
    None
}

pub fn encode_fixed_width(f: &FractionalSecond) -> u32 {
    match f {
        &FractionalSecond::Milliseconds(x) => encode_millis(x),
        &FractionalSecond::Microseconds(x) => encode_micros(x),
        &FractionalSecond::Nanoseconds(x) => encode_nanos(x),
        &FractionalSecond::None => FRAC_SECOND_FIXED_WIDTH_NONE
    }
}

pub fn encode_millis(millis: u16) -> u32 {
    FRAC_SECOND_FIXED_WIDTH_MILLI | (millis as u32)
}

pub fn encode_micros(micros: u32) -> u32 {
    FRAC_SECOND_FIXED_WIDTH_MICRO | micros
}

pub fn encode_nanos(nanos: u32) -> u32 {
    FRAC_SECOND_FIXED_WIDTH_NANO | nanos
}

pub fn encode_none() -> u32 {
    FRAC_SECOND_FIXED_WIDTH_NONE
}

#[inline]
pub fn decode_fixed_width(encoded: u32) -> FractionalSecond {
    let prefix = FRAC_SECOND_FIXED_WIDTH_PREFIX_MASK & encoded;
    let value = FRAC_SECOND_FIXED_WIDTH_VALUE_MASK & encoded;

    match prefix {
        FRAC_SECOND_FIXED_WIDTH_MILLI => FractionalSecond::Milliseconds(value as u16),
        FRAC_SECOND_FIXED_WIDTH_MICRO => FractionalSecond::Microseconds(value),
        FRAC_SECOND_FIXED_WIDTH_NANO => FractionalSecond::Nanoseconds(value),
        FRAC_SECOND_FIXED_WIDTH_NONE => FractionalSecond::None,
        _ => panic!("Corrupt fixed width encoded fractional second")
    }
}

// 2-bit indicators for fixed-width encoding of fractional seconds
pub const FRAC_SECOND_FIXED_WIDTH_NONE: u32 = 0x00_00_00_00;
pub const FRAC_SECOND_FIXED_WIDTH_MILLI: u32 = 0x40_00_00_00;
pub const FRAC_SECOND_FIXED_WIDTH_MICRO: u32 = 0x80_00_00_00;
pub const FRAC_SECOND_FIXED_WIDTH_NANO: u32 = 0xC0_00_00_00;
pub const FRAC_SECOND_FIXED_WIDTH_PREFIX_MASK: u32 = 0xC0_00_00_00;
pub const FRAC_SECOND_FIXED_WIDTH_VALUE_MASK: u32 = 0x3F_FF_FF_FF;

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;


    #[test]
    fn fixed_width_roundtrip_millis() {
        roundtrip(FractionalSecond::Milliseconds(MILLIS_MIN));
        roundtrip(FractionalSecond::Milliseconds(MILLIS_MIN));
    }

    #[test]
    fn fixed_width_roundtrip_micros() {
        roundtrip(FractionalSecond::Microseconds(MICROS_MIN));
        roundtrip(FractionalSecond::Microseconds(MICROS_MAX));
    }

    #[test]
    fn fixed_width_roundtrip_nanos() {
        roundtrip(FractionalSecond::Nanoseconds(NANOS_MIN));
        roundtrip(FractionalSecond::Nanoseconds(NANOS_MAX));
    }

    #[test]
    fn fixed_width_roundtrip_none() {
        roundtrip(FractionalSecond::None);
    }

    fn roundtrip(f: FractionalSecond) {
        assert_eq!(f, decode_fixed_width(encode_fixed_width(&f)));
    }
}
