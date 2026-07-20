#[inline]
pub fn luai_lerpf(a: f32, b: f32, t: f32) -> f32 {
    if t == 1.0 {
        b
    } else {
        a + (b - a) * t
    }
}
