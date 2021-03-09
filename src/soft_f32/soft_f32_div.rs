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
        if a_frac != 0 {
            return f32_propagate_nan(a, b);
        }
        if b_exp == 0xFF {
            if b_frac != 0 {
                return f32_propagate_nan(a, b);
            } else {
                // Invalid
                return f32_pack_raw(r_sign, 0xFF, 0);
            }
        }
        // INFINITY
        return f32_pack_raw(r_sign, 0xFF, 0);
    }
    if b_exp == 0xFF {
        if b_frac != 0 {
            return f32_propagate_nan(a, b);
        }
        return f32_pack_raw(r_sign, 0, 0);
    }

    if b_exp == 0 {
        if b_frac == 0 {
            if (a_exp | a_frac) == 0 {
                // Invalid, return default NaN
                return f32_pack_raw(r_sign, 0xFF, 0);
            }
            // Infinity
            return f32_pack_raw(r_sign, 0xFF, 0);
        }

        let (exp, frac) = f32_norm_subnormal_frac(b_frac);
        b_exp = exp;
        b_frac = frac;
    }
    if a_exp == 0 {
        if a_frac == 0 {
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

    // Use u64 to divide u32
    let mut a_frac_u64 = a_frac as u64;
    if a_frac < b_frac {
        r_exp -= 1;
        a_frac_u64 <<= 31;
    } else {
        a_frac_u64 <<= 30;
    }
    let mut r_frac_u64 = a_frac_u64 / (b_frac as u64);

    if (r_frac_u64 & 0x3F) == 0 {
        if r_frac_u64 * (b_frac as u64) != a_frac_u64 {
            r_frac_u64 = r_frac_u64 | 0x01;
        }
    }

    r_frac = (r_frac_u64 & 0xFFFFFFFF) as i32;

    f32_round_and_pack(r_sign, r_exp, r_frac)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_f32_div() {
        // 0.1 / 0.2 = 0.5
        assert_eq!(crate::soft_f32::f32_div(0x3DCCCCCD, 0x3E4CCCCD), 0x3F000000);
        // -0.1 / -0.2 = 0.5
        assert_eq!(crate::soft_f32::f32_div(0xBDCCCCCD, 0xBE4CCCCD), 0x3F000000);

        // 12345 / 67890 = 8.381021E8
        assert_eq!(crate::soft_f32::f32_div(0x4640E400, 0x47849900), 0x3E3A33D0);
        // -12345 / -67890 = 8.381021E8
        assert_eq!(crate::soft_f32::f32_div(0xC640E400, 0xC7849900), 0x3E3A33D0);

        // -0.1 / 0.2 = -0.5
        assert_eq!(crate::soft_f32::f32_div(0xBDCCCCCD, 0x3E4CCCCD), 0xBF000000);
        // 0.1 / -0.2 = -0.5
        assert_eq!(crate::soft_f32::f32_div(0x3DCCCCCD, 0xBE4CCCCD), 0xBF000000);
    }

    #[test]
    fn test_f32_div_inf_nan() {
        // 1 / 0 = Inf
        assert_eq!(crate::soft_f32::f32_div(0x3F800000, 0x00000000), 0x7F800000);

        //  1 / -0 = -Inf
        assert_eq!(crate::soft_f32::f32_div(0x3F800000, 0x80000000), 0xFF800000);

        // Inf / 1 = Inf
        assert_eq!(crate::soft_f32::f32_div(0x7F800000, 0x3F800000), 0x7F800000);

        // -Inf / 1 = -Inf
        assert_eq!(crate::soft_f32::f32_div(0xFF800000, 0x3F800000), 0xFF800000);

        // -Inf / Inf = -Inf
        assert_eq!(crate::soft_f32::f32_div(0xFF800000, 0x7F800000), 0xFF800000);

        // Inf / -1 = -Inf
        assert_eq!(crate::soft_f32::f32_div(0x7F800000, 0xBF800000), 0xFF800000);

        // -Inf / -1 = -Inf
        assert_eq!(crate::soft_f32::f32_div(0xFF800000, 0xBF800000), 0x7F800000);

        // NaN / 1 = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_div(0xFFFFFFFF, 0x3F800000)), true);

        // NaN / -1 = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_div(0xFFFFFFFF, 0x3F800000)), true);

        // NaN / Inf = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_div(0xFFFFFFFF, 0x7F800000)), true);

        // NaN / -Inf = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_div(0xFFFFFFFF, 0xFF800000)), true);
    }
}

