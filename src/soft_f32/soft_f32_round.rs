use super::util::{
    f32_shift_right_jam,
    f32_norm_round_and_pack,
    f32_round_and_pack,
    f32_pack_raw, f32_pack,
    f32_propagate_nan,
    f32_sign, f32_exp, f32_frac,
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_f32_round() {
        // round(0.01) = 0
        assert_eq!(crate::soft_f32::f32_round(0x3C23D70A), 0x00000000);
        // round(4) = 4
        assert_eq!(crate::soft_f32::f32_round(0x40800000), 0x40800000);
    }
}
