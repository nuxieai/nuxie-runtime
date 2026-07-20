#[allow(non_snake_case)]
#[inline(always)]
pub const fn rol(x: u32, s: u32) -> u32 {
    (x >> s) | (x << (32 - s))
}
