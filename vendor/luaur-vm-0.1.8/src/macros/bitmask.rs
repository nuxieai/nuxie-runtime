#[inline(always)]
pub const fn bitmask(b: i32) -> i32 {
    1 << b
}

#[inline(always)]
pub const fn bit2mask(b1: i32, b2: i32) -> i32 {
    bitmask(b1) | bitmask(b2)
}
