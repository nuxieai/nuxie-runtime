#[inline(always)]
pub const fn testbits(x: i32, m: i32) -> i32 {
    x & m
}
