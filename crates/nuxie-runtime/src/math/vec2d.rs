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
    (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t)
}

fn weighted_lerp_point(from: (f32, f32), to: (f32, f32), t: f32) -> (f32, f32) {
    let inverse_t = 1.0 - t;
    (from.0 * inverse_t + to.0 * t, from.1 * inverse_t + to.1 * t)
}
