use super::util::{
    f32_shift_right_jam,
    f32_norm_round_and_pack,
    f32_round_and_pack,
    f32_pack_raw, f32_pack,
    f32_propagate_nan,
    f32_sign, f32_exp, f32_frac,
};

use crate::soft_f32::f32_sub;

pub fn f32_add(a: u32, b: u32) -> u32 {
    // Sign
    let mut a_sign = f32_sign(a);
    let mut b_sign = f32_sign(b);
    let mut r_sign;

    if a_sign != b_sign {
        // Consider as substraction
        return f32_sub(a, b);
    }

    // Exp
    let mut a_exp = f32_exp(a);
    let mut b_exp = f32_exp(b);
    let mut r_exp;

    // Frac
    let mut a_frac = f32_frac(a);
    let mut b_frac = f32_frac(b);
    let mut r_frac;

    let diff_exp = a_exp - b_exp;

    if diff_exp == 0 {
        if a_exp == 0 {
            r_sign = a_sign;
            r_exp = a_exp;
            r_frac = a_frac + b_frac;
            return f32_pack_raw(r_sign, r_exp, r_frac);
        }

        if a_exp == 0xFF {
            if (a_frac | b_frac) != 0 {
                // Propagate NaN
                return f32_propagate_nan(a, b);
            } else {
                r_sign = a_sign;
                r_exp = a_exp;
                r_frac = a_frac + b_frac;
                return f32_pack_raw(r_sign, r_exp, r_frac);
            }
        }

        r_sign = a_sign;
        r_exp = a_exp;
        r_frac = 0x01000000 + a_frac + b_frac;

        if (r_frac & 0x01) == 0 && r_exp < 0xFE {
            return f32_pack_raw(r_sign, r_exp, r_frac >> 1);
        }

        r_frac <<= 6;
    } else {
        r_sign = a_sign;

        // Prepare fractions to do calculation
        a_frac <<= 6;
        b_frac <<= 6;

        if diff_exp < 0 {
            // a_exp < b_exp
            if b_exp == 0xFF {
                if b_sign != 0 {
                    // Propagate NaN
                    return f32_propagate_nan(a, b);
                } else {
                    // Return a NaN
                    return f32_pack_raw(r_sign, 0xFF, 0);
                }
            }

            r_exp = b_exp;

            // Prepare the Fraction of A to do an addition with B
            if a_exp != 0 {
                a_frac += 0x20000000;
            } else {
                a_frac += a_frac;
            }
            a_frac = f32_shift_right_jam(a_frac, -diff_exp);
        } else {
            // a_exp > b_exp, a_exp == b_exp is considered in the other case
            if a_exp == 0xFF {
                if a_sign != 0 {
                    // Propagate NaN
                    return f32_propagate_nan(a, b);
                } else {
                    // Return a NaN
                    return f32_pack_raw(a_sign, a_exp, a_frac);
                }
            }

            r_exp = a_exp;

            // Prepare the Fraction of B to do an addition with A
            if b_exp != 0 {
                b_frac += 0x20000000;
            } else {
                b_frac += b_frac;
            }
            b_frac = f32_shift_right_jam(b_frac, diff_exp);
        }

        r_frac = 0x20000000 + a_frac + b_frac;

        if r_frac < 0x40000000 {
            r_exp -= 1;
            r_frac <<= 1;
        }
    }

    f32_round_and_pack(r_sign, r_exp, r_frac)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_f32_add() {
        // 0.1 + 0.2 = 0.3
        assert_eq!(crate::soft_f32::f32_add(0x3DCCCCCD, 0x3E4CCCCD), 0x3E99999A);
        // -0.1 + -0.2 = -0.3
        assert_eq!(crate::soft_f32::f32_add(0xBDCCCCCD, 0xBE4CCCCD), 0xBE99999A);

        // 12345 + 67890 = 80235
        assert_eq!(crate::soft_f32::f32_add(0x4640E400, 0x47849900), 0x479CB580);
        // -12345 + -67890 = -80235
        assert_eq!(crate::soft_f32::f32_add(0xC640E400, 0xC7849900), 0xC79CB580);

        // 0.002 + 0.002 = 0.004
        assert_eq!(crate::soft_f32::f32_add(0x3B03126F, 0x3B03126F), 0x3B83126F);
        // -0.002 + -0.002 = -0.004
        assert_eq!(crate::soft_f32::f32_add(0xBB03126F, 0xBB03126F), 0xBB83126F);

        // -0.1 + 0.2 = 0.1
        assert_eq!(crate::soft_f32::f32_add(0xBDCCCCCD, 0x3E4CCCCD), 0x3DCCCCCD);
        // 0.1 + -0.2 = -0.1
        assert_eq!(crate::soft_f32::f32_add(0x3DCCCCCD, 0xBE4CCCCD), 0xBDCCCCCD);
        
        // 3.4028235E38 + 1.701412E38 = Inf
        assert_eq!(crate::soft_f32::f32_add(0x7F7FFFFF, 0x7F000001), 0x7F800000);       // PANIC!
    }

    #[test]
    fn test_f32_add_inf_nan() {
        // Inf + 1 = Inf
        assert_eq!(crate::soft_f32::f32_add(0x7F800000, 0x3F800000), 0x7F800000);

        // -Inf + 1 = -Inf
        assert_eq!(crate::soft_f32::f32_add(0xFF800000, 0x3F800000), 0xFF800000);

        // -Inf + Inf = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_add(0xFF800000, 0x7F800000)), true);

        // Inf + -1 = Inf
        assert_eq!(crate::soft_f32::f32_add(0x7F800000, 0x3F800000), 0x7F800000);

        // -Inf + -1 = -Inf
        assert_eq!(crate::soft_f32::f32_add(0xFF800000, 0x3F800000), 0xFF800000);

        // common tests about NaN
        crate::soft_f32::test_util::test_nan(super::f32_add);

        // NaN + 1 = NaN
        assert_eq!(crate::soft_f32::f32_add(0xFFFFFFFF, 0x3F800000), 0xFFFFFFFF);

        // NaN + -1 = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_add(0xFFFFFFFF, 0x3F800000)), true);        // what is it? 1 == -1?

        // NaN + Inf = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_add(0xFFFFFFFF, 0x7F800000)), true);        // why not use assert!()

        // NaN + -Inf = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_add(0xFFFFFFFF, 0xFF800000)), true);
    }
}

