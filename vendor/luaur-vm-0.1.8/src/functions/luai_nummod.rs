#[inline]
pub fn luai_nummod(a: f64, b: f64) -> f64 {
    a - (a / b).floor() * b
}
