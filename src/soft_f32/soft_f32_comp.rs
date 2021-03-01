use super::util::{
    f32_sign, f32_exp, f32_frac,
    f32_is_nan,
};

pub fn f32_ne(a: u32, b: u32) -> bool {
    if f32_is_nan(a) || f32_is_nan(b) {
        // Unable to compare
        return false;
    }

    !f32_eq(a, b)
}

pub fn f32_eq(a: u32, b: u32) -> bool {
    if f32_is_nan(a) || f32_is_nan(b) {
        // Unable to compare
        return false;
    }

    a == b || ((a | b) << 1) == 0
}

pub fn f32_ge(a: u32, b: u32) -> bool {
    if f32_is_nan(a) || f32_is_nan(b) {
        // Unable to compare
        return false;
    }

    !f32_lt(a, b)
}

pub fn f32_lt(a: u32, b: u32) -> bool {
    if f32_is_nan(a) || f32_is_nan(b) {
        // Unable to compare
        return false;
    }

    // Sign
    let mut a_sign = f32_sign(a);
    let mut b_sign = f32_sign(b);

    if a_sign != b_sign {
        // Different sign
        // FIXME: Add a test for this case
        if (a | b) << 1 != 0 {
            return a_sign == 1;
        }
    } else {
        // Same sign
        if a != b {
            if a_sign != 0 {
                return a > b;
            } else {
                return a < b;
            }
        }
    }
    false
}

pub fn f32_gt(a: u32, b: u32) -> bool {
    if f32_is_nan(a) || f32_is_nan(b) {
        // Unable to compare
        return false;
    }

    !f32_le(a, b)
}

pub fn f32_le(a: u32, b: u32) -> bool {
    if f32_is_nan(a) || f32_is_nan(b) {
        // Unable to compare
        return false;
    }

    // Sign
    let mut a_sign = f32_sign(a);
    let mut b_sign = f32_sign(b);

    if a_sign != b_sign {
        // Different sign
        // FIXME: Add a test for this case
        if (a | b) << 1 == 0 || a_sign == 1 {
            return true;
        }
    } else {
        // Same sign
        if a == b {
            return true;
        }

        if a_sign != 0 {
            return a > b;
        } else {
            return a < b;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_f32_eq() {
        // 0.3 == 0.2 - false
        assert_eq!(crate::soft_f32::f32_eq(0x3E99999A, 0x3E4CCCCD), false);

        // 0.2 == 0.3 - false
        assert_eq!(crate::soft_f32::f32_eq(0x3E4CCCCD, 0x3E99999A), false);

        // -0.1 == -0.2 - false
        assert_eq!(crate::soft_f32::f32_eq(0xBDCCCCCD, 0xBE4CCCCD), false);

        // -0.2 == -0.1 - false
        assert_eq!(crate::soft_f32::f32_eq(0xBE4CCCCD, 0xBDCCCCCD), false);

        // 0.2 == 0.2 - true
        assert_eq!(crate::soft_f32::f32_eq(0x3E4CCCCD, 0x3E4CCCCD), true);

        // -0.2 == 0.2 - false
        assert_eq!(crate::soft_f32::f32_eq(0xBE4CCCCD, 0x3E4CCCCD), false);

        // Inf == 0.004 - false
        assert_eq!(crate::soft_f32::f32_eq(0x7F800000, 0x3B83126F), false);

        // -Inf < 0.004 - true
        assert_eq!(crate::soft_f32::f32_eq(0xFF800000, 0x3B83126F), false);

        // -Inf == Inf - true
        assert_eq!(crate::soft_f32::f32_eq(0xFF800000, 0x7F800000), false);

        // -Inf == Nan - false
        assert_eq!(crate::soft_f32::f32_eq(0xFF800000, 0xFFFFFFFF), false);

        // Inf == Nan - false
        assert_eq!(crate::soft_f32::f32_eq(0x7F800000, 0xFFFFFFFF), false);

        // Nan == Nan - false
        assert_eq!(crate::soft_f32::f32_eq(0xFFFFFFFF, 0xFFFFFFFF), false);

        // -Inf == -Inf - true
        assert_eq!(crate::soft_f32::f32_eq(0xFF800000, 0xFF800000), true);

        // Inf == Inf - true
        assert_eq!(crate::soft_f32::f32_eq(0x7F800000, 0x7F800000), true);
    }

    #[test]
    fn test_f32_lt() {
        // 0.3 < 0.2 - false
        assert_eq!(crate::soft_f32::f32_lt(0x3E99999A, 0x3E4CCCCD), false);

        // 0.2 < 0.3 - true
        assert_eq!(crate::soft_f32::f32_lt(0x3E4CCCCD, 0x3E99999A), true);

        // -0.1 < -0.2 - false
        assert_eq!(crate::soft_f32::f32_lt(0xBDCCCCCD, 0xBE4CCCCD), false);

        // -0.2 < -0.1 - true
        assert_eq!(crate::soft_f32::f32_lt(0xBE4CCCCD, 0xBDCCCCCD), true);

        // 0.2 < 0.2 - false
        assert_eq!(crate::soft_f32::f32_lt(0x3E4CCCCD, 0x3E4CCCCD), false);

        // -0.2 < 0.2 - true
        assert_eq!(crate::soft_f32::f32_lt(0xBE4CCCCD, 0x3E4CCCCD), true);

        // Inf < 0.004 - false
        assert_eq!(crate::soft_f32::f32_lt(0x7F800000, 0x3B83126F), false);

        // -Inf < 0.004 - true
        assert_eq!(crate::soft_f32::f32_lt(0xFF800000, 0x3B83126F), true);

        // -Inf < Inf - true
        assert_eq!(crate::soft_f32::f32_lt(0xFF800000, 0x7F800000), true);

        // -Inf < Nan - false
        assert_eq!(crate::soft_f32::f32_lt(0xFF800000, 0xFFFFFFFF), false);

        // Inf < Nan - false
        assert_eq!(crate::soft_f32::f32_lt(0x7F800000, 0xFFFFFFFF), false);

        // Nan < Nan - false
        assert_eq!(crate::soft_f32::f32_lt(0xFFFFFFFF, 0xFFFFFFFF), false);
    }

    #[test]
    fn test_f32_le() {
        // 0.3 <= 0.2 - false
        assert_eq!(crate::soft_f32::f32_le(0x3E99999A, 0x3E4CCCCD), false);

        // 0.2 <= 0.3 - true
        assert_eq!(crate::soft_f32::f32_le(0x3E4CCCCD, 0x3E99999A), true);

        // -0.1 <= -0.2 - false
        assert_eq!(crate::soft_f32::f32_le(0xBDCCCCCD, 0xBE4CCCCD), false);

        // -0.2 <= -0.1 - true
        assert_eq!(crate::soft_f32::f32_le(0xBE4CCCCD, 0xBDCCCCCD), true);

        // 0.2 <= 0.2 - true
        assert_eq!(crate::soft_f32::f32_le(0x3E4CCCCD, 0x3E4CCCCD), true);

        // -0.2 <= 0.2 - true
        assert_eq!(crate::soft_f32::f32_le(0xBE4CCCCD, 0x3E4CCCCD), true);

        // Inf <= 0.004 - false
        assert_eq!(crate::soft_f32::f32_le(0x7F800000, 0x3B83126F), false);

        // -Inf <= 0.004 - true
        assert_eq!(crate::soft_f32::f32_le(0xFF800000, 0x3B83126F), true);

        // -Inf <= Inf - true
        assert_eq!(crate::soft_f32::f32_le(0xFF800000, 0x7F800000), true);

        // -Inf <= Nan - false
        assert_eq!(crate::soft_f32::f32_le(0xFF800000, 0xFFFFFFFF), false);

        // Inf < Nan - false
        assert_eq!(crate::soft_f32::f32_le(0x7F800000, 0xFFFFFFFF), false);

        // Nan <= Nan - false
        assert_eq!(crate::soft_f32::f32_le(0xFFFFFFFF, 0xFFFFFFFF), false);
    }
}
