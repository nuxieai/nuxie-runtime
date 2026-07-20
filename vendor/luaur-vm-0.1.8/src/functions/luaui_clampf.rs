#[inline]
pub fn luaui_clampf(v: f32, min: f32, max: f32) -> f32 {
    let r = if v < min { min } else { v };
    if r > max {
        max
    } else {
        r
    }
}
