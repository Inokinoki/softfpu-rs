
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

pub(crate) fn f32_norm_subnormal_frac(frac: i32) -> (i32, i32) {
    let shift_count = f32_count_leading_zero(frac) - 8;

    (1 - shift_count, frac << shift_count)
}

use std::convert::TryFrom;

pub(crate) fn f32_short_shift_right_jam64(a: u64, count: i32) -> i32 {
    let b: i32 = i32::try_from((a >> count) & 0xFFFFFFFF).unwrap();
    if (a & ((0x01 << count) - 1)) != 0 {
        return b | 1;
    }
    b
}

pub(crate) fn f32_approx_recip(a: u32) -> u32 {
    let k0s: &[u64] = &[
        0xFFC4, 0xF0BE, 0xE363, 0xD76F, 0xCCAD, 0xC2F0, 0xBA16, 0xB201,
        0xAA97, 0xA3C6, 0x9D7A, 0x97A6, 0x923C, 0x8D32, 0x887E, 0x8417,
    ];

    let k1s: &[u64] = &[
        0xF0F1, 0xD62C, 0xBFA1, 0xAC77, 0x9C0A, 0x8DDB, 0x8185, 0x76BA,
        0x6D3B, 0x64D4, 0x5D5C, 0x56B1, 0x50B6, 0x4B55, 0x4679, 0x4211,
    ];

    let a_u64 = a as u64;

    let index = ((a >> 27) & 0x0F) as usize;

    let eps = (a >> 11) as u64;

    let r0: u64 = k0s[index] - ((k1s[index] * eps) >> 20);
    
    let delta0: u32 = ((r0 * a_u64) >> 7) as u32;

    let r: u64  = (r0 << 16) + ((r0 * delta0 as u64) >> 24);

    let sqr_delta0 = (delta0 as u64 * delta0 as u64) >> 32;

    let result = (r + (r * sqr_delta0) >> 48) as u32;

    result
}

pub(crate) fn f32_approx_recip_sqrt(odd_exp: u32, a: u32) -> u32 {
    let k0s: &[u16] = &[
        0xB4C9, 0xFFAB, 0xAA7D, 0xF11C, 0xA1C5, 0xE4C7, 0x9A43, 0xDA29,
        0x93B5, 0xD0E5, 0x8DED, 0xC8B7, 0x88C6, 0xC16D, 0x8424, 0xBAE1,
    ];

    let k1s: &[u16] = &[
        0xA5A5, 0xEA42, 0x8C21, 0xC62D, 0x788F, 0xAA7F, 0x6928, 0x94B6,
        0x5CC7, 0x8335, 0x52A6, 0x74E2, 0x4A3E, 0x68FE, 0x432B, 0x5EFD,
    ];

    let index = (((a >> 27) & 0x0E) + odd_exp) as usize;
    let eps = a >> 12;
    let r0: u16 = (k0s[index] - (((k1s[index] as u64 * eps as u64) >> 20) & 0xFFFF) as u16);
    let mut e_sqr_r0: u32 = r0 as u32 * r0 as u32;

    if odd_exp == 0 {
        e_sqr_r0 <<= 1;
    }

    let delta0 = !(((e_sqr_r0 as u64 * a as u64)>>23) as u32);

    let mut r: u32 = ((r0 as u32) << 16) + (((r0 as u64) * delta0 as u64) >> 25) as u32;

    let sqr_delta0 = ((delta0 as u64) * (delta0 as u64)) >> 32;

    r += (((r0 as u32) >> 1) + ((r0 as u32) >> 3) - ((((r0 as u64) << 14) * (sqr_delta0 as u64)) >> 48) as u32 );

    if r & 0x80000000 == 0 {
        r = 0x80000000;
    }

    r
}
