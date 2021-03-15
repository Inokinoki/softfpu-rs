use crate::soft_f32::f32_is_nan;

pub(crate) const INF: u32 = 0x7F800000;         // Consider making they public
pub(crate) const NEG_INF: u32 = 0xFF800000;
pub(crate) const ONE: u32 = 0x3F800000;
pub(crate) const NEG_ONE: u32 = 0xBF800000;

pub fn test_nan<F>(f: F)
    where
        F: Fn(u32, u32) -> u32 {

    // NaN `f` ± 1 = NaN
    assert!(f32_is_nan(f(0xFFFFFFFF, ONE)));
    assert!(f32_is_nan(f(0xFFFFFFFF, NEG_ONE)));

    // NaN `f` ± INF = NaN
    assert!(f32_is_nan(f(0xFFFFFFFF, INF)));
    assert!(f32_is_nan(f(0xFFFFFFFF, NEG_INF)));

    // NaN `f` NaN = NaN
    assert!(f32_is_nan(f(0xFFFFF114, 0xFFFFF514)));
}