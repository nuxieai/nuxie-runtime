//! RadialGradient shares LinearGradient callback/stop ownership and differs
//! only when materializing the shader radius from start/end distance.

pub(crate) fn radius(start_x: f32, start_y: f32, end_x: f32, end_y: f32) -> f32 {
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    (dx * dx + dy * dy).sqrt()
}
