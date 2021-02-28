
pub(crate) fn f32_shift_right_jam(a: i32, dist: i32) -> i32 {
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

use crate::soft_float::{ RoundingMode, DetectTininess };

pub(crate) fn f32_sign(a: u32) -> i32 {
    ((a >> 31) & 0x01) as i32
}

pub(crate) fn f32_exp(a: u32) -> i32 {
    ((a >> 23) & 0x0FF) as i32
}

pub(crate) fn f32_frac(a: u32) -> i32 {
    (a & 0x7FFFFF) as i32
}

pub(crate) fn f32_frac_old(a: u32) -> i32 {
    ((a & 0x7FFFFF) | (0x01 << 23)) as i32
}

pub(crate) fn f32_pack_raw(in_sign: i32, in_exp: i32, in_frac: i32) -> u32 {
    // FIXME: why + is not equivalent to |
    ((in_sign << 31) | (in_exp << 23) + in_frac) as u32
}

pub fn f32_is_nan(a: u32) -> bool {
    let is_exp_nan = ((a & 0x7F800000) == 0x7F800000);
    let is_frac_nan = ((a & 0x007FFFFF) != 0);

    is_exp_nan && is_frac_nan
}

pub(crate) fn f32_is_frac_nan(a: u32) -> bool {
    let is_exp_nan = ((a & 0x7FC00000) == 0x7F800000);
    let is_frac_nan = (a & 0x003FFFFF) != 0;

    is_exp_nan && is_frac_nan
}

pub(crate) fn f32_propagate_nan(in_a: u32, in_b: u32) -> u32 {
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

pub(crate) fn f32_round_and_pack(in_sign: i32, in_exp: i32, in_frac: i32) -> u32 {
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

pub(crate) fn f32_count_leading_zero(in_frac: i32) -> i32 {
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

pub(crate) fn f32_norm_round_and_pack(in_sign: i32, in_exp: i32, in_frac: i32) -> u32 {
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
