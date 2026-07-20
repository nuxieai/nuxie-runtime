#[allow(non_snake_case)]
#[inline(always)]
pub fn luai_num2unsigned(i: &mut u32, n: f64) {
    // The C++ implementation uses x86 FPU assembly (fistp) for rounding to a 64-bit integer.
    // In Rust, the idiomatic and portable equivalent for converting a float to an unsigned integer
    // with truncation (matching the behavior of the Luau VM's fallback/standard paths) is `as`.
    // Note: `as` in Rust performs saturating conversion for floats to ints since 1.45.0,
    // but for Luau's bitwise/unsigned needs, we cast to i64 then to u32 to match the `l` variable logic.
    *i = (n as i64) as u32;
}
