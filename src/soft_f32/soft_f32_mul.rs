use super::util::{
    f32_shift_right_jam,
    f32_norm_round_and_pack,
    f32_round_and_pack,
    f32_pack_raw, f32_pack,
    f32_propagate_nan,
    f32_sign, f32_exp, f32_frac,
    f32_norm_subnormal_frac,
    f32_short_shift_right_jam64,
};

use crate::soft_f32::f32_sub;

pub fn f32_mul(a: u32, b: u32) -> u32 {
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
        // NaN
        if a_frac != 0 || (b_exp == 0xFF && b_frac != 0) {
            return f32_propagate_nan(a, b);
        }

        // Inf
        if b_exp | b_frac == 0 {
            // Default NaN
            return f32_pack_raw(r_sign, 0xFF, 0);
        }

        return f32_pack_raw(r_sign, 0xFF, 0);
    }
    if b_exp == 0xFF {
        // Nan
        if b_frac != 0 {
            return f32_propagate_nan(a, b);
        }

        // Inf
        if a_exp | a_frac == 0 {
            // Default NaN
            return f32_pack_raw(r_sign, 0xFF, 0);
        }

        return f32_pack_raw(r_sign, 0xFF, 0);
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
    if b_exp == 0 {
        if b_sign == 0 {
            // Zero
            return f32_pack_raw(r_sign, 0, 0);
        }

        let (exp, frac) = f32_norm_subnormal_frac(b_frac);
        b_exp = exp;
        b_frac = frac;
    }

    r_exp = a_exp + b_exp - 0x7F;

    a_frac = (a_frac | 0x00800000) << 7;
    b_frac = (b_frac | 0x00800000) << 8;

    let a_frac_u32: u32 = a_frac as u32;
    let b_frac_u32: u32 = b_frac as u32;
    let a_frac_u64: u64 = a_frac_u32 as u64;
    let b_frac_u64: u64 = b_frac_u32 as u64;
    let frac_prod: u64 = a_frac_u64 * b_frac_u64;
    r_frac = f32_short_shift_right_jam64(frac_prod, 32);

    if r_frac < 0x40000000 {
        r_exp -= 1;
        r_frac <<= 1;
    }

    f32_round_and_pack(r_sign, r_exp, r_frac)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_f32_mul() {
        // 0.1 x 0.2 = 0.02
        assert_eq!(crate::soft_f32::f32_mul(0x3DCCCCCD, 0x3E4CCCCD), 0x3CA3D70B);
        // -0.1 x -0.2 = 0.02
        assert_eq!(crate::soft_f32::f32_mul(0xBDCCCCCD, 0xBE4CCCCD), 0x3CA3D70B);

        // 12345 x 67890 = 8.381021E8
        assert_eq!(crate::soft_f32::f32_mul(0x4640E400, 0x47849900), 0x4E47D1B1);
        // -12345 + -67890 = 8.381021E8
        assert_eq!(crate::soft_f32::f32_mul(0xC640E400, 0xC7849900), 0x4E47D1B1);

        // -0.1 x 0.2 = -0.02
        assert_eq!(crate::soft_f32::f32_mul(0xBDCCCCCD, 0x3E4CCCCD), 0xBCA3D70B);
        // 0.1 x -0.2 = -0.02
        assert_eq!(crate::soft_f32::f32_mul(0x3DCCCCCD, 0xBE4CCCCD), 0xBCA3D70B);
    }

    #[test]
    fn test_f32_mul_inf_nan() {
        // Inf x 1 = Inf
        assert_eq!(crate::soft_f32::f32_mul(0x7F800000, 0x3F800000), 0x7F800000);

        // -Inf x 1 = -Inf
        assert_eq!(crate::soft_f32::f32_mul(0xFF800000, 0x3F800000), 0xFF800000);

        // -Inf x Inf = -Inf
        assert_eq!(crate::soft_f32::f32_mul(0xFF800000, 0x7F800000), 0xFF800000);

        // Inf x -1 = -Inf
        assert_eq!(crate::soft_f32::f32_mul(0x7F800000, 0xBF800000), 0xFF800000);

        // -Inf x -1 = Inf
        assert_eq!(crate::soft_f32::f32_mul(0xFF800000, 0xBF800000), 0x7F800000);

        // NaN x 1 = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_mul(0xFFFFFFFF, 0x3F800000)), true);

        // NaN x -1 = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_mul(0xFFFFFFFF, 0x3F800000)), true);

        // NaN x Inf = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_mul(0xFFFFFFFF, 0x7F800000)), true);

        // NaN x -Inf = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_mul(0xFFFFFFFF, 0xFF800000)), true);
    }
}
