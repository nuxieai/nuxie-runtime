#[inline]
pub fn perlin_lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}
