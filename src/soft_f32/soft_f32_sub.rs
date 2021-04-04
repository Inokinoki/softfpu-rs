use super::util::{
    f32_shift_right_jam,
    f32_norm_round_and_pack,
    f32_round_and_pack,
    f32_pack_raw, f32_pack,
    f32_propagate_nan,
    f32_sign, f32_exp, f32_frac,
};

pub fn f32_sub(a: u32, b: u32) -> u32 {
    // Sign
    let mut a_sign = f32_sign(a);
    let mut b_sign = f32_sign(b);
    let mut r_sign = a_sign;

    // Exp
    let mut a_exp = f32_exp(a);
    let mut b_exp = f32_exp(b);
    let mut r_exp = 0;

    // Frac
    let mut a_frac = f32_frac(a);
    let mut b_frac = f32_frac(b);
    let mut r_frac = 0;

    let diff_exp = a_exp - b_exp;

    a_frac <<= 7;
    b_frac <<= 7;

    if diff_exp == 0 {
        if a_exp == 0xFF {
            if (a_sign | b_sign) != 0 {
                // Propagate NaN
                return f32_propagate_nan(a, b);
            } else {
                // Return a NaN
                // FIXME: 0x7FC00000 is used in IBM IEEE, while 0xFFC00000 is used otherwise
                return f32_pack_raw(0, 0xFF, 0);
            }
        }

        if a_exp == 0 {
            a_exp = 1;
            b_exp = 1;
        }

        if a_frac > b_frac {
            // Fraction of A is greater
            r_exp = a_exp;
            r_frac = a_frac - b_frac;
        } else if b_frac > a_frac {
            // Fraction of B is greater
            r_sign ^= 1;
            r_exp = b_exp;
            r_frac = b_frac - a_frac;
        } else {
            // Same, will cause a 0
            return f32_pack(0, 0, 0);
        }
        return f32_norm_round_and_pack(r_sign, r_exp - 1, r_frac);
    } else if diff_exp > 0 {
        // Exp of A is greater
        if a_exp == 0xFF {
            if a_frac != 0 {
                // Propagate NaN
                return f32_propagate_nan(a, b);
            } else {
                r_sign = a_sign;
                r_exp = a_exp;
                r_frac = a_frac;
            }
            return f32_pack_raw(r_sign, r_exp, r_frac);
        }

        if b_exp != 0 {
            b_frac += 0x40000000;
        } else {
            b_frac += b_frac;
        }

        b_frac = f32_shift_right_jam(b_frac, diff_exp);
        a_frac |= 0x40000000;   // Add the implicit 1

        r_exp = a_exp;
        r_frac = a_frac - b_frac;
    } else {
        // Exp of B is greater
        if b_exp == 0xFF {
            if b_frac != 0 {
                // Propagate NaN
                return f32_propagate_nan(a, b);
            } else {
                // Return a NaN
                return f32_pack_raw(r_sign ^ 1, 0xFF, 0);
            }
        }

        if a_exp != 0 {
            a_frac += 0x40000000;
        } else {
            a_frac += a_frac;
        }

        a_frac = f32_shift_right_jam(a_frac, -diff_exp);
        b_frac |= 0x40000000;

        r_sign ^= 1;
        r_exp = b_exp;
        r_frac = b_frac - a_frac;
    }
    return f32_norm_round_and_pack(r_sign, r_exp - 1, r_frac);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_f32_sub() {
        // 0.3 - 0.2 = 0.1
        assert_eq!(crate::soft_f32::f32_sub(0x3E99999A, 0x3E4CCCCD), 0x3DCCCCCE);

        // 0.2 - 0.3 = -0.1
        assert_eq!(crate::soft_f32::f32_sub(0x3E4CCCCD, 0x3E99999A), 0xBDCCCCCE);

        // 80235 - 67890 = 12345
        assert_eq!(crate::soft_f32::f32_sub(0x479CB580, 0x47849900), 0x4640E400);

        // 0.004 - 0.004 = 0
        assert_eq!(crate::soft_f32::f32_sub(0x3B83126F, 0x3B83126F), 0x00000000);

        // FIXME
        // assert_eq!(crate::soft_f32::f32_sub(0x0, 0xBDCCCCCE), 0x3DCCCCCE);
        assert_eq!(crate::soft_f32::f32_sub(0x0, 0x3DCCCCCE), 0xBDCCCCCE);
        assert_eq!(crate::soft_f32::f32_sub(0x0, 0x0), 0x0);
        assert_eq!(crate::soft_f32::f32_sub(0x0, 0x80000000), 0x0);
    }
}
