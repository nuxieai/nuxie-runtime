// Direct source-correspondence owner for pinned `src/math/bezier_utils.cpp`.
fn cubic_measure_segment_count(points: [(f32, f32); 4], inv_tolerance: f32) -> u32 {
    wangs_cubic(points, inv_tolerance)
        .ceil()
        .ceil()
        .min(TRIM_CONTOUR_MAX_SEGMENTS as f32) as u32
}

fn wangs_cubic(points: [(f32, f32); 4], precision: f32) -> f32 {
    let first = vector_length_squared((
        points[0].0 - 2.0 * points[1].0 + points[2].0,
        points[0].1 - 2.0 * points[1].1 + points[2].1,
    ));
    let second = vector_length_squared((
        points[1].0 - 2.0 * points[2].0 + points[3].0,
        points[1].1 - 2.0 * points[2].1 + points[3].1,
    ));
    let length_term_pow2 = 9.0 * 4.0 / 64.0 * precision * precision;
    (first.max(second) * length_term_pow2).sqrt().sqrt()
}

fn eval_cubic(points: [(f32, f32); 4], t: f32) -> (f32, f32) {
    let a = (
        points[3].0 + 3.0 * (points[1].0 - points[2].0) - points[0].0,
        points[3].1 + 3.0 * (points[1].1 - points[2].1) - points[0].1,
    );
    let b = (
        3.0 * (points[2].0 - 2.0 * points[1].0 + points[0].0),
        3.0 * (points[2].1 - 2.0 * points[1].1 + points[0].1),
    );
    let c = (
        3.0 * (points[1].0 - points[0].0),
        3.0 * (points[1].1 - points[0].1),
    );
    (
        ((a.0 * t + b.0) * t + c.0) * t + points[0].0,
        ((a.1 * t + b.1) * t + c.1) * t + points[0].1,
    )
}

fn cubic_position_tangent(points: [(f32, f32); 4], t: f32) -> ((f32, f32), (f32, f32)) {
    if t == 0.0 {
        let tangent_to = if points[0] != points[1] {
            points[1]
        } else if points[1] != points[2] {
            points[2]
        } else {
            points[3]
        };
        return (
            points[0],
            (tangent_to.0 - points[0].0, tangent_to.1 - points[0].1),
        );
    }
    if t == 1.0 {
        let tangent_from = if points[3] != points[2] {
            points[2]
        } else if points[2] != points[1] {
            points[1]
        } else {
            points[0]
        };
        return (
            points[3],
            (points[3].0 - tangent_from.0, points[3].1 - tangent_from.1),
        );
    }

    let a = (
        points[3].0 + 3.0 * (points[1].0 - points[2].0) - points[0].0,
        points[3].1 + 3.0 * (points[1].1 - points[2].1) - points[0].1,
    );
    let b = (
        3.0 * (points[2].0 - 2.0 * points[1].0 + points[0].0),
        3.0 * (points[2].1 - 2.0 * points[1].1 + points[0].1),
    );
    let c = (
        3.0 * (points[1].0 - points[0].0),
        3.0 * (points[1].1 - points[0].1),
    );
    let tan = normalized_vector((
        (3.0 * a.0 * t + 2.0 * b.0) * t + c.0,
        (3.0 * a.1 * t + 2.0 * b.1) * t + c.1,
    ));
    (eval_cubic(points, t), tan)
}
