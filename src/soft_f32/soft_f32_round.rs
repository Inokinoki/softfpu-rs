use super::util::{
    f32_shift_right_jam,
    f32_norm_round_and_pack,
    f32_round_and_pack,
    f32_pack_raw, f32_pack,
    f32_propagate_nan,
    f32_sign, f32_exp, f32_frac,
    f32_is_nan,
    f32_count_leading_zero,
};

pub fn f32_round(a: u32) -> u32 {
    // Sign
    let mut a_sign = f32_sign(a);

    // Exp
    let mut a_exp = f32_exp(a);

    // Frac
    let mut a_frac = f32_frac(a);

    if (a_exp < 0x7E) {
        if ((a << 1) == 0) {
            return a;
        }
        let mut z = a & f32_pack_raw(1, 0, 0);
        if (a_frac != 0) {
            if (a_exp == 0x7E) {
                z |= f32_pack_raw(0, 0x7F, 0);
            }
        }
        return z;
    }
    if (0x96 <= a_exp) {
        if (a_exp == 0xFF && a_frac != 0) {
            return f32_propagate_nan(a, 0);
        }
        return a;
    }
    let last_bit_mask = (1 << (0x96 - a_exp));
    let round_bits_mask = last_bit_mask - 1;
    // By default use near even round mode
    let mut r = a;
    r += (last_bit_mask >> 1);
    if ((r & round_bits_mask) == 0) {
        r &= (!last_bit_mask);
    }
    r &= (!round_bits_mask);
    return r;
}

pub fn from_int32(a: i32) -> u32 {
    /*
    1. Convert the int representation into a sign and a positive binary number
    2. Convert the positive binary number to a fixed point representation
        where the integral part = 1.xxxxx
        (This step uses shift operations - you shift the decimal point to the left
        until you find the most significant 1 bit in the binary number)
        Let M be the mantissa with the leading 1 bit omitted
        Let E be the exponent of the fixed point representation
    3. Express the exponent E in  excess 127  code
    4. Assemble:
          Sign Exponent Mantissa     into  a IEEE 754 respresentation 
    */
    if (a == 0) {
        return 0;
    }

    let frac: i32;
    if (a < 0) {
        frac = (!a) + 1;
    } else {
        frac = a;
    }
    let leading_zero = f32_count_leading_zero(frac);
    let shift = (32 - leading_zero - 1);
    let exp = shift + 0x7F;
    let sign = if (a < 0) { 1 } else { 0 };
    f32_pack(sign, exp, (frac << (24 - shift - 1)) & 0x7fffff)
}

// TODO: Add more convertors
pub fn to_int32(a: u32) -> i32 {
    let p = f32_round(a);
    let mut r: i32 = 0;

    if (f32_is_nan(p)) {
        return std::i32::MAX;
    } else if (p == 0x7F800000 || p == 0xFF800000) {
        // Infinity
        if (f32_sign(p) != 0) {
            return std::i32::MIN;
        }
        return std::i32::MAX;
    } else {
        if (p == 0 || p == 0x80000000) {
            // +- 0
            return 0;
        }

        let sign = f32_sign(a);
        let exp = f32_exp(a);
        let mut frac = f32_frac(a);

        frac |= 0x800000;

        if (exp < 0x7F) {
            // It must be a zero, because it is too tiny
            return 0;
        }

        let shift = exp - 0x7F;
        if (sign == 0) {
            return frac >> (23 - shift);
        } else {
            return -!((frac >> (23 - shift)) - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_f32_round() {
        // round(0.01) = 0
        assert_eq!(crate::soft_f32::f32_round(0x3C23D70A), 0x00000000);
        // round(4) = 4
        assert_eq!(crate::soft_f32::f32_round(0x40800000), 0x40800000);
    }

    #[test]
    fn test_f32_to_i32() {
        // round(0.01) = 0
        assert_eq!(crate::soft_f32::soft_f32_round::to_int32(0x3C23D70A), 0);
        // round(4) = 4
        assert_eq!(crate::soft_f32::soft_f32_round::to_int32(0x40800000), 4);
        // round(244.5) = 244
        assert_eq!(crate::soft_f32::soft_f32_round::to_int32(0x43748000), 244);
        // round(128.2) = 128
        assert_eq!(crate::soft_f32::soft_f32_round::to_int32(0x43003333), 128);

        // FIXME: round(0x4f000000) = 2147483647)
        // assert_eq!(crate::soft_f32::soft_f32_round::to_int32(0x4F000000), std::i32::MAX);
        // FIXME: round(0xcf000000) = -2147483648)
        // assert_eq!(crate::soft_f32::soft_f32_round::to_int32(0xCF000000), std::i32::MIN);
    }

    #[test]
    fn test_i32_to_f32() {
        // from_int32(0) = 0.0        
        assert_eq!(crate::soft_f32::soft_f32_round::from_int32(0), 0x0);

        // from_int32(4) = 4.0
        assert_eq!(crate::soft_f32::soft_f32_round::from_int32(4), 0x40800000);

        // from_int32(-4) = -4.0
        assert_eq!(crate::soft_f32::soft_f32_round::from_int32(-4), 0xC0800000);

        // from_int32(80235) = 0x479CB580
        assert_eq!(crate::soft_f32::soft_f32_round::from_int32(80235), 0x479CB580);

        // from_int32(80235) = 0x479CB580
        assert_eq!(crate::soft_f32::soft_f32_round::from_int32(-80235), 0xC79CB580);

        // FIXME: from_int32(2147483647) =  0x4f000000
        // assert_eq!(crate::soft_f32::soft_f32_round::from_int32(std::i32::MAX), 0x4F000000);

        // FIXME: from_int32(-2147483648) = 0xcf000000
        // assert_eq!(crate::soft_f32::soft_f32_round::from_int32(std::i32::MIN), 0xCF000000);
    }
}
