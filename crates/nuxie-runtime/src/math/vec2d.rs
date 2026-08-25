// Direct source-correspondence owner for pinned `src/math/vec2d.cpp`.
fn distance(from: (f32, f32), to: (f32, f32)) -> f32 {
    let x = to.0 - from.0;
    let y = to.1 - from.1;
    x.mul_add(x, y * y).sqrt()
}

fn distance_squared(from: (f32, f32), to: (f32, f32)) -> f32 {
    vector_length_squared((to.0 - from.0, to.1 - from.1))
}

fn vector_length_squared(vector: (f32, f32)) -> f32 {
    vector.0.mul_add(vector.0, vector.1 * vector.1)
}

fn lerp_point(from: (f32, f32), to: (f32, f32), t: f32) -> (f32, f32) {
    (
        (to.0 - from.0).mul_add(t, from.0),
        (to.1 - from.1).mul_add(t, from.1),
    )
}

fn weighted_lerp_point(from: (f32, f32), to: (f32, f32), t: f32) -> (f32, f32) {
    let inverse_t = 1.0 - t;
    (
        from.0.mul_add(inverse_t, to.0 * t),
        from.1.mul_add(inverse_t, to.1 * t),
    )
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

    #[test]
    fn raw_path_weighted_lerp_preserves_pinned_product_then_add_boundaries() {
        let from = (f32::from_bits(0xfb9f_06ba), 0.0);
        let to = (f32::from_bits(0xf189_75cd), 0.0);
        let t = f32::from_bits(0x4177_5082);
        assert_eq!(weighted_lerp_point(from, to, t).0.to_bits(), 0x7d8f_b10c);
    }
}
