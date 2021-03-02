use super::util::{
    f32_shift_right_jam,
    f32_norm_round_and_pack,
    f32_round_and_pack,
    f32_pack_raw, f32_pack,
    f32_propagate_nan,
    f32_sign, f32_exp, f32_frac,
    f32_norm_subnormal_frac,
    f32_short_shift_right_jam64,
    f32_approx_recip,
};


pub fn f32_div(a: u32, b: u32) -> u32 {
    // Sign
    let mut a_sign = f32_sign(a);
    let mut b_sign = f32_sign(b);
    let mut r_sign = a_sign ^ b_sign;

    // Exp
    let mut a_exp = f32_exp(a);
    let mut b_exp = f32_exp(b);
    let mut r_exp;

    // Frac
    let mut a_frac = f32_frac(a);
    let mut b_frac = f32_frac(b);
    let mut r_frac;

    if a_exp == 0xFF {
        if a_sign != 0 {
            return f32_propagate_nan(a, b);
        }
        if b_exp == 0xFF {
            if b_sign != 0 {
                return f32_propagate_nan(a, b);
            } else {
                // Invalid
                return f32_pack_raw(r_sign, 0xFF, 0);
            }
        }
    }
    if b_exp == 0xFF {
        if b_frac != 0 {
            return f32_propagate_nan(a, b);
        }
        return f32_pack_raw(r_sign, 0, 0);
    }

    if b_exp == 0 {
        if b_sign == 0 {
            if (a_exp | a_frac) == 0 {
                // Invalid, return default NaN
                return f32_pack_raw(r_sign, 0xFF, 0);
            }
            // Zero
            return f32_pack_raw(r_sign, 0, 0);
        }

        let (exp, frac) = f32_norm_subnormal_frac(b_frac);
        a_exp = exp;
        a_frac = frac;
    }
    if a_exp == 0 {
        if a_sign == 0 {
            // Zero
            return f32_pack_raw(r_sign, 0, 0);
        }

        let (exp, frac) = f32_norm_subnormal_frac(a_frac);
        a_exp = exp;
        a_frac = frac;
    }

    r_exp = a_exp - b_exp + 0x7E;
    a_frac |= 0x00800000;
    b_frac |= 0x00800000;

    if a_frac < b_frac {
        r_exp -= 1;
        a_frac <<= 8;
    } else {
        a_frac <<= 7;
    }
    b_frac <<= 8;

    let a_frac_u64 = a_frac as u64;
    let b_frac_u32 = b_frac as u32;
    let r_frac_u64 = (a_frac_u64 * f32_approx_recip(b_frac_u32) as u64) >> 32;
    r_frac = r_frac_u64 as i32;

    r_frac += 2;

    if (r_frac & 0x3F) < 2 {
        r_frac &= (!3);

        let r_frac_u64 = r_frac as u64;
        let rem = (a_frac_u64 << 31) - (r_frac_u64 << 1) * b_frac_u32 as u64;

        if rem & 0x8000000000000000 != 0 {
            r_frac -= 4;
        } else {
            if rem != 0 {
                r_frac |= 1;
            }
        }
    }

    f32_round_and_pack(r_sign, r_exp, r_frac)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_f32_div() {
        // FIXME: 0.1 / 0.2 = 0.5
        // 0x3E800002
        // assert_eq!(crate::soft_f32::f32_div(0x3DCCCCCD, 0x3E4CCCCD), 0x3F000000);
        // FIXME: -0.1 / -0.2 = 0.5
        // 0x3E800002
        // assert_eq!(crate::soft_f32::f32_div(0xBDCCCCCD, 0xBE4CCCCD), 0x3F000000);

        // FIXME: 12345 / 67890 = 8.381021E8
        // 0x3D800001
        // assert_eq!(crate::soft_f32::f32_div(0x4640E400, 0x47849900), 0x3E3A33D0);
        // FIXME: -12345 / -67890 = 8.381021E8
        // 0x3D800001
        // assert_eq!(crate::soft_f32::f32_div(0xC640E400, 0xC7849900), 0x3E3A33D0);

        // FIXME: -0.1 / 0.2 = -0.5
        // 0xBE800002
        // assert_eq!(crate::soft_f32::f32_div(0xBDCCCCCD, 0x3E4CCCCD), 0xBF000000);
        // FIXME: 0.1 / -0.2 = -0.5
        // 0xBE800002
        // assert_eq!(crate::soft_f32::f32_div(0x3DCCCCCD, 0xBE4CCCCD), 0xBF000000);
    }

    #[test]
    fn test_f32_div_inf_nan() {
        // FIXME: 1 / 0 = Inf
        // 0
        // assert_eq!(crate::soft_f32::f32_div(0x3F800000, 0x00000000), 0x7F800000);

        // FIXME: 1 / -0 = -Inf
        // 0xB3800005
        // assert_eq!(crate::soft_f32::f32_div(0x3F800000, 0x80000000), 0xFF800000);

        // Inf / 1 = Inf
        //assert_eq!(crate::soft_f32::f32_div(0x7F800000, 0x3F800000), 0x7F800000);

        // -Inf / 1 = -Inf
        // assert_eq!(crate::soft_f32::f32_div(0xFF800000, 0x3F800000), 0xFF800000);

        // -Inf / Inf = -Inf
        // assert_eq!(crate::soft_f32::f32_div(0xFF800000, 0x7F800000), 0xFF800000);

        // Inf x -1 = -Inf
        //assert_eq!(crate::soft_f32::f32_div(0x7F800000, 0xBF800000), 0xFF800000);

        // -Inf / -1 = -Inf
        // assert_eq!(crate::soft_f32::f32_div(0xFF800000, 0xBF800000), 0x7F800000);

        // NaN / 1 = NaN
        // assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_div(0xFFFFFFFF, 0x3F800000)), true);

        // NaN / -1 = NaN
        // assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_div(0xFFFFFFFF, 0x3F800000)), true);

        // NaN / Inf = NaN
        // assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_div(0xFFFFFFFF, 0x7F800000)), true);

        // NaN / -Inf = NaN
        // assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_div(0xFFFFFFFF, 0xFF800000)), true);
    }
}

