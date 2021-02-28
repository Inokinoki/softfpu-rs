
mod soft_f32_add;
mod soft_f32_sub;
mod util;

// Operations
use soft_f32_add::f32_add;
use soft_f32_sub::f32_sub;

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
}
