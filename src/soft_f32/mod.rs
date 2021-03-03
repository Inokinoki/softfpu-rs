
mod soft_f32_add;
mod soft_f32_sub;
mod soft_f32_mul;
mod soft_f32_div;
mod soft_f32_comp;
mod util;

// Operations
use soft_f32_add::f32_add;
use soft_f32_sub::f32_sub;
use soft_f32_mul::f32_mul;
use soft_f32_div::f32_div;

// Comparisons
use soft_f32_comp::f32_eq;
use soft_f32_comp::f32_ne;
use soft_f32_comp::f32_lt;
use soft_f32_comp::f32_gt;
use soft_f32_comp::f32_le;
use soft_f32_comp::f32_ge;

// Utilities
pub use util::{
    f32_is_nan
};

// F32 struct
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

use std::ops;

impl ops::Add<F32> for F32 {
    type Output = F32;

    fn add(self, other: F32) -> F32 {
        F32 {
            value: crate::soft_f32::soft_f32_add::f32_add(self.value, other.value)
        }
    }
}

impl ops::Sub<F32> for F32 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        F32 {
            value: crate::soft_f32::soft_f32_sub::f32_sub(self.value, other.value)
        }
    }
}

impl ops::Mul<F32> for F32 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        F32 {
            value: crate::soft_f32::soft_f32_mul::f32_mul(self.value, other.value)
        }
    }
}

impl ops::Div<F32> for F32 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        F32 {
            value: crate::soft_f32::soft_f32_div::f32_div(self.value, other.value)
        }
    }
}

use std::cmp;

impl cmp::PartialEq for F32 {
    // Implement equal with only symmetric and transitive for F32.
    //
    // Ref: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
    // For example, in floating point numbers NaN != NaN,
    // so floating point types implement PartialEq but not Eq.
    fn eq(&self, other: &Self) -> bool {
        f32_eq(self.value, other.value)
    }

    fn ne(&self, other: &Self) -> bool {
        f32_ne(self.value, other.value)
    }
}

impl cmp::PartialOrd for F32 {
    // Implement compare with only symmetric and transitive for F32.
    //
    // Ref: https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        if f32_eq(self.value, other.value) { return Some(cmp::Ordering::Equal); }
        if f32_lt(self.value, other.value) { return Some(cmp::Ordering::Less); }
        if f32_gt(self.value, other.value) { return Some(cmp::Ordering::Greater); }

        None
    }

    fn gt(&self, other: &Self) -> bool {
        f32_gt(self.value, other.value)
    }

    fn lt(&self, other: &Self) -> bool {
        f32_lt(self.value, other.value)
    }

    fn ge(&self, other: &Self) -> bool {
        f32_ge(self.value, other.value)
    }

    fn le(&self, other: &Self) -> bool {
        f32_le(self.value, other.value)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_f32_add_with_struct() {
        let v0_1 = crate::soft_f32::F32::from_u32(0x3DCCCCCD);
        let v0_2 = crate::soft_f32::F32::from_u32(0x3E4CCCCD);

        let v0_3 = v0_1 + v0_2;

        assert_eq!(v0_3.value(), 0x3E99999A);
    }

    #[test]
    fn test_f32_sub_with_struct() {
        let v0_1 = crate::soft_f32::F32::from_u32(0x3DCCCCCD);
        let v0_2 = crate::soft_f32::F32::from_u32(0x3E4CCCCD);

        let v0_1_result = v0_2 - v0_1;

        assert_eq!(v0_1_result.value(), 0x3DCCCCCD);
    }

    #[test]
    fn test_f32_mul_with_struct() {
        let v0_1 = crate::soft_f32::F32::from_u32(0x3DCCCCCD);
        let v0_2 = crate::soft_f32::F32::from_u32(0x3E4CCCCD);

        let v0_02 = v0_2 * v0_1;

        assert_eq!(v0_02.value(), 0x3CA3D70B);
    }

    #[test]
    fn test_f32_div_with_struct() {
        let v0_1 = crate::soft_f32::F32::from_u32(0x3DCCCCCD);
        let v0_2 = crate::soft_f32::F32::from_u32(0x3E4CCCCD);

        let v0_5 = v0_1 / v0_2;

        assert_eq!(v0_5.value(), 0x3F000000);
    }

    #[test]
    fn test_f32_compare_with_struct() {
        let v0_1 = crate::soft_f32::F32::from_u32(0x3DCCCCCD);
        let v0_2 = crate::soft_f32::F32::from_u32(0x3E4CCCCD);

        assert_eq!(v0_1 == v0_2, false);
        assert_eq!(v0_1 != v0_2, true);

        assert_eq!(v0_1 < v0_2, true);
        assert_eq!(v0_1 <= v0_2, true);
        assert_eq!(v0_1 < v0_1, false);
        assert_eq!(v0_1 <= v0_1, true);

        assert_eq!(v0_1 > v0_2, false);
        assert_eq!(v0_1 >= v0_2, false);
        assert_eq!(v0_1 > v0_1, false);
        assert_eq!(v0_1 >= v0_1, true);
    }
}
