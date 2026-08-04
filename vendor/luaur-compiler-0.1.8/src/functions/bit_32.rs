#[allow(non_snake_case)]
pub fn bit32(v: f64) -> u32 {
    // convert through signed 64-bit integer to match runtime behavior and gracefully truncate negative integers
    v as i64 as u32
}
