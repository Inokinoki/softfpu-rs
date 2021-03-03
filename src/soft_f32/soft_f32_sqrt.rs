use super::util::{
    f32_shift_right_jam,
    f32_norm_round_and_pack,
    f32_round_and_pack,
    f32_pack_raw, f32_pack,
    f32_propagate_nan,
    f32_sign, f32_exp, f32_frac,
    f32_norm_subnormal_frac,
    f32_short_shift_right_jam64,
    f32_approx_recip_sqrt,
};


pub fn f32_sqrt(a: u32) -> u32 {
    // Sign
    let mut a_sign = f32_sign(a);

    // Exp
    let mut a_exp = f32_exp(a);
    let mut r_exp;

    // Frac
    let mut a_frac = f32_frac(a);
    let mut r_frac;

    if a_exp == 0xFF {
        if a_frac != 0 {
            f32_propagate_nan(a, 0);
        }
        if a_sign == 0 {
            return a;
        }

        // Invalid
        return f32_pack_raw(0, 0xFF, 0);
    }

    if a_sign != 0 {
        if a_exp | a_frac == 0 {
            // 0 or -0
            return a;
        }

        // Invalid
        return f32_pack_raw(0, 0xFF, 0);
    }

    if a_exp == 0 {
        if a_frac == 0 {
            return a;
        }
        let (exp, frac) = f32_norm_subnormal_frac(a_frac);
        a_exp = exp;
        a_frac = frac;
    }

    r_exp = ((a_exp - 0x7F) >> 1) + 0x7E;
    a_exp &= 1;

    a_frac = (a_frac | 0x00800000) << 8;

    let result = f32_approx_recip_sqrt(a_exp as u32, a_frac as u32);
    let r_frac_u64: u64 = ((a_frac as u32) as u64) * (result as u64);
    let r_frac_u32 = (r_frac_u64>> 32) as u32;
    r_frac = r_frac_u32 as i32;

    if a_exp != 0 {
        r_frac >>= 1;
    }
    r_frac += 2;

    if (r_frac & 0x3F) < 2 {
        let r_shifted_frac = (r_frac >> 2) as u32;
        let neg_rem = r_shifted_frac * r_shifted_frac;
        r_frac = (r_shifted_frac << 2) as i32;

        if neg_rem & 0x80000000 != 0 {
            r_frac |= 0x01;
        } else {
            if neg_rem != 0 {
                r_frac -= 1;
            }
        }
    }

    f32_round_and_pack(0, r_exp, r_frac)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_f32_sqrt() {
        // FIXME: sqrt(0.01) = 0.1
        // 0x3DA3D70A
        // 0b111101101000111101011100001010
        // assert_eq!(crate::soft_f32::f32_sqrt(0x3C23d70A), 0x3DCCCCCD);
        // FIXME: sqrt(4) = 2
        // 0x3FC5461B
        // 0b111111110001010100011000011011
        // assert_eq!(crate::soft_f32::f32_sqrt(0x40800000), 0x40000000);

        // sqrt(0) = 0
        assert_eq!(crate::soft_f32::f32_sqrt(0x00), 0x00);
        // sqrt(-0) = -0
        assert_eq!(crate::soft_f32::f32_sqrt(0x80000000), 0x80000000);
    }

    #[test]
    fn test_f32_sqrt_inf_nan() {
        // TODO: add some tests
    }
}

