use std::ops;

use crate::soft_float::{ RoundingMode, DetectTininess };


pub struct F32 {
    value: u32,
}

impl F32 {
    pub fn from_u32(value: u32) -> F32 {
        F32 {
            value: value
        }
    }

    pub fn value(self) -> u32 {
        self.value
    }
}

impl ops::Add<F32> for F32 {
    type Output = F32;

    fn add(self, other: F32) -> F32 {
        F32 {
            value: f32_add(self.value, other.value)
        }
    }
}


fn f32_shift_right_jam(a: i32, dist: i32) -> i32 {
    if dist < 31 {
        if (a << (-dist & 31)) != 0 {
            (a >> dist) | 1
        } else {
            (a >> dist) | 0
        }
    } else {
        if a != 0 {
            1
        } else {
            0
        }
    }
}

fn f32_sign(a: u32) -> i32 {
    ((a >> 31) & 0x01) as i32
}

fn f32_exp(a: u32) -> i32 {
    ((a >> 23) & 0x0FF) as i32
}

fn f32_frac(a: u32) -> i32 {
    (a & 0x7FFFFF) as i32
}

fn f32_frac_old(a: u32) -> i32 {
    ((a & 0x7FFFFF) | (0x01 << 23)) as i32
}

fn f32_pack_raw(in_sign: i32, in_exp: i32, in_frac: i32) -> u32 {
    // FIXME: why + is not equivalent to |
    ((in_sign << 31) | (in_exp << 23) + in_frac) as u32
}

fn f32_is_nan(a: u32) -> bool {
    let is_exp_nan = ((a & 0x7F800000) == 0x7F800000);
    let is_frac_nan = ((a & 0x007FFFFF) != 0);

    is_exp_nan && is_frac_nan
}

fn f32_is_frac_nan(a: u32) -> bool {
    let is_exp_nan = ((a & 0x7FC00000) == 0x7F800000);
    let is_frac_nan = (a & 0x003FFFFF) != 0;

    is_exp_nan && is_frac_nan
}

fn f32_propagate_nan(in_a: u32, in_b: u32) -> u32 {
    let mut a = in_a | 0x00400000;
    let mut b = in_b | 0x00400000;

    let is_a_frac_nan = f32_is_frac_nan(in_a);
    let is_b_frac_nan = f32_is_frac_nan(in_b);

    if is_a_frac_nan | is_b_frac_nan {
        if is_a_frac_nan {
            if !is_b_frac_nan {
                return match f32_is_nan(b) {
                    true => b,
                    false => a,
                };
            }
        } else {
            return match f32_is_nan(a) {
                true => a,
                false => b,
            };
        }
    }

    let a_frac = f32_frac(a);
    let b_frac = f32_frac(b);

    if a_frac < b_frac {
        return b;
    } else if b_frac < a_frac {
        return a;
    } else {
        if a < b {
            return a;
        }
        return b;
    }
}


pub fn f32_pack(in_sign: i32, in_exp: i32, in_frac: i32) -> u32 {
    ((in_sign << 31) | ((in_exp & 0x0FF) << 23) | (in_frac & 0x007fffff)) as u32
}

fn f32_round_and_pack(in_sign: i32, in_exp: i32, in_frac: i32) -> u32 {
    let rounding_mode = RoundingMode::NearEven;
    let detect_tininess = DetectTininess::After;

    let mut round_increment = 0x40;

    let mut sign = in_sign;
    let mut exp = in_exp;
    let mut frac = in_frac;

    match &rounding_mode {
        NearEven => { /* Do nothing */ }
        NearMaxMag => { /* Do nothing */ }
        _ => {
            if sign == 1 {
                match &rounding_mode {
                    Min => round_increment = 0x7F,
                    _ => round_increment = 0,
                }
            } else {
                match &rounding_mode {
                    Max => round_increment = 0x7F,
                    _ => round_increment = 0,
                }
            }
        }
    }
    let mut round_bits = frac & 0x7F;

    if exp >= 0xFD {    // FIXME: exponential value is wrongly detected here
        if exp < 0 {
            let is_tiny = (
                match detect_tininess {
                    Before => true,
                    _ => false,
                } || exp < -1 || (frac as u32) + (round_increment as u32) < 0x80000000
            );
            frac = f32_shift_right_jam(frac, -exp);
            exp = 0;
            round_bits = frac & 0x7F;
            if is_tiny && round_bits != 0 {
                // Underflow
            }
        } else if exp > 0xFD || (frac as u32) + (round_increment as u32) >= 0x80000000 {
            // Overflow and inexact
            return (((sign << 31) | ((exp & 0x0FF) << 23) | (frac & 0x7fffff)) - ! round_increment) as u32
        }
    }

    frac = (frac + round_increment) >> 7;

    // if ( roundBits ) { softfloat_exceptionFlags |= softfloat_flag_inexact; }

    match &rounding_mode {
        NearEven => {
            if round_bits ^ 0x40 == 0 {
                frac &= 0x7FFFFFFE;
            } else {
                frac &= 0x7FFFFFFF;
            }
        }
    }
    if frac == 0 { exp = 0; }

    f32_pack_raw(sign, exp, frac)
}

fn f32_count_leading_zero(in_frac: i32) -> i32 {
    let f32_count_leading_zeros_8: &[i32] = &[
        8, 7, 6, 6, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ];

    let mut count = 0;
    let mut frac = in_frac;
    if frac < 0x10000 {
        count = 16;
        frac <<= 16;
    }
    if frac < 0x1000000 {
        count += 8;
        frac <<= 8;
    }
    count + f32_count_leading_zeros_8[((frac >> 24) & 0xFF) as usize]
}

fn f32_norm_round_and_pack(in_sign: i32, in_exp: i32, in_frac: i32) -> u32 {
    let shift_count = f32_count_leading_zero(in_frac) - 1;
    let mut sign = in_sign;
    let mut exp = in_exp - shift_count;
    let mut frac = in_frac;

    if exp < 0xFD && shift_count >= 7 {
        if frac == 0 {
            exp = 0;
        }
        return f32_pack_raw(sign, exp, frac << shift_count);
    } else {
        return f32_round_and_pack(sign, exp, frac << shift_count);
    }
}

pub fn f32_add2(a: u32, b: u32) -> u32 {
    // Sign
    let mut a_sign = f32_sign(a);
    let mut b_sign = f32_sign(b);
    let mut r_sign = 0;

    // Exp
    let mut a_exp = f32_exp(a);
    let mut b_exp = f32_exp(b);
    let mut r_exp = 0;

    // Frac
    let mut a_frac = f32_frac(a);
    let mut b_frac = f32_frac(b);
    let mut r_frac = 0;

    let diff_exp = a_exp - b_exp;
    let absdiff_exp = diff_exp.abs();

    // Check any types of zero
    if a_exp == 0 && (a_frac & 0x7FFFFF) == 0 {
        return b;
    }
    if b_exp == 0 && (b_frac & 0x7FFFFF) == 0 {
        return a;
    }

    // Normalize one of the two numbers, if they are with different exponential
    if diff_exp > 0 {
        b_exp += absdiff_exp;
        b_frac >>= absdiff_exp;
    } else if diff_exp < 0 {
        a_exp += absdiff_exp;
        a_frac >>= absdiff_exp;
    }

    if a_sign == b_sign {
        // Case of same sign
        r_sign = a_sign;
        r_exp = a_exp;
        r_frac = a_frac + b_frac;
    } else {
        // Case of different sign
        if a_sign == 1 {
            // a is negative
            r_frac = b_frac - a_frac;
        } else {
            // b is negative
            r_frac = a_frac - b_frac;
        }

        r_exp = a_exp;

        r_sign = if r_frac >= 0 { 0 } else { 1 };

        if r_frac < 0 {
            r_frac = -r_frac;
        }

        while r_frac < (1 << 23) && r_frac != 0 {
            r_frac <<= 1;
            r_exp -= 1;
        }
    }

    // Handle overflow
    if r_frac >= (1 << 24) {
        r_frac >>= 1;
        r_exp += 1;
    }

    // Handle zero
    if r_frac == 0 {
        r_exp = 0;
    }

    f32_pack(r_sign, r_exp, r_frac)    // With no round
}

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
        assert_eq!(r_sign, 0);
        assert_eq!(r_exp, 0x8F);
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
    }

    #[test]
    fn test_f32_sub() {
        // FIXME: 0.3 - 0.2 = 0.1
        // assert_eq!(crate::soft_f32::f32_sub(0x3E99999A, 0x3E4CCCCD), 0x3DCCCCCE);

        // 80235 - 67890 = 12345
        assert_eq!(crate::soft_f32::f32_sub(0x479CB580, 0x47849900), 0x4640E400);

        // 0.004 - 0.004 = 0
        assert_eq!(crate::soft_f32::f32_sub(0x3B83126F, 0x3B83126F), 0x00000000);
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

        // NaN + 1 = NaN
        assert_eq!(crate::soft_f32::f32_add(0xFFFFFFFF, 0x3F800000), 0xFFFFFFFF);

        // NaN + -1 = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_add(0xFFFFFFFF, 0x3F800000)), true);

        // NaN + Inf = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_add(0xFFFFFFFF, 0x7F800000)), true);

        // NaN + -Inf = NaN
        assert_eq!(crate::soft_f32::f32_is_nan(crate::soft_f32::f32_add(0xFFFFFFFF, 0xFF800000)), true);
    }

    #[test]
    fn test_f32_add_with_struct() {
        let v0_1 = crate::soft_f32::F32::from_u32(0x3DCCCCCD);
        let v0_2 = crate::soft_f32::F32::from_u32(0x3E4CCCCD);

        let v0_3 = v0_1 + v0_2;

        assert_eq!(v0_3.value(), 0x3E99999A);
    }
}
