use super::vec2d::Vec2D;

pub struct CubicUtilities;
impl CubicUtilities {
    pub fn compute_hull(
        from: Vec2D,
        from_out: Vec2D,
        to_in: Vec2D,
        to: Vec2D,
        t: f32,
        hull: &mut [Vec2D; 6],
    ) {
        hull[0] = Vec2D::lerp(from, from_out, t);
        hull[1] = Vec2D::lerp(from_out, to_in, t);
        hull[2] = Vec2D::lerp(to_in, to, t);
        hull[3] = Vec2D::lerp(hull[0], hull[1], t);
        hull[4] = Vec2D::lerp(hull[1], hull[2], t);
        hull[5] = Vec2D::lerp(hull[3], hull[4], t);
    }
    pub fn too_far(a: Vec2D, b: Vec2D, threshold: f32) -> bool {
        (a.x - b.x).abs().max((a.y - b.y).abs()) > threshold
    }
    pub fn should_split_cubic(
        from: Vec2D,
        from_out: Vec2D,
        to_in: Vec2D,
        to: Vec2D,
        threshold: f32,
    ) -> bool {
        let one_third = Vec2D::lerp(from, to, 1.0 / 3.0);
        let two_thirds = Vec2D::lerp(from, to, 2.0 / 3.0);
        Self::too_far(from_out, one_third, threshold) || Self::too_far(to_in, two_thirds, threshold)
    }
    pub fn cubic_at(t: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
        let inverse = 1.0 - t;
        inverse.powi(3) * a
            + 3.0 * inverse * inverse * t * b
            + 3.0 * inverse * t * t * c
            + t.powi(3) * d
    }
}
