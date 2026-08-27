// Direct source-correspondence owner for pinned `src/math/vec2d.cpp`.
fn distance(from: (f32, f32), to: (f32, f32)) -> f32 {
    nuxie_render_api::Vec2D::distance(
        nuxie_render_api::Vec2D::new(from.0, from.1),
        nuxie_render_api::Vec2D::new(to.0, to.1),
    )
}

fn distance_squared(from: (f32, f32), to: (f32, f32)) -> f32 {
    nuxie_render_api::Vec2D::distance_squared(
        nuxie_render_api::Vec2D::new(from.0, from.1),
        nuxie_render_api::Vec2D::new(to.0, to.1),
    )
}

fn vector_length_squared(vector: (f32, f32)) -> f32 {
    nuxie_render_api::Vec2D::new(vector.0, vector.1).length_squared()
}

fn lerp_point(from: (f32, f32), to: (f32, f32), t: f32) -> (f32, f32) {
    let point = nuxie_render_api::Vec2D::lerp(
        nuxie_render_api::Vec2D::new(from.0, from.1),
        nuxie_render_api::Vec2D::new(to.0, to.1),
        t,
    );
    (point.x, point.y)
}

#[cfg(test)]
mod exact_vec2d_tests {
    use super::*;

    #[test]
    fn contour_lerp_preserves_pinned_vec2d_contraction() {
        let from = (f32::from_bits(0x3c3a_6d8a), 0.0);
        let to = (f32::from_bits(0x657c_889f), 0.0);
        let t = f32::from_bits(0x1b04_ab9a);
        assert_eq!(lerp_point(from, to, t).0.to_bits(), 0x4103_0e55);
    }
}
