// Direct source-correspondence owner for pinned `src/math/raw_path_utils.cpp`
// and `include/rive/math/raw_path_utils.hpp`.

fn two(vector: (f32, f32)) -> (f32, f32) {
    (vector.0 + vector.0, vector.1 + vector.1)
}

#[derive(Debug, Clone, Copy)]
struct EvalQuad {
    // at^2 + bt + c
    a: (f32, f32),
    b: (f32, f32),
    c: (f32, f32),
}

impl EvalQuad {
    fn new(points: [(f32, f32); 3]) -> Self {
        Self {
            a: point_add(point_sub(points[0], two(points[1])), points[2]),
            b: two(point_sub(points[1], points[0])),
            c: points[0],
        }
    }

    fn evaluate(&self, t: f32) -> (f32, f32) {
        point_add(
            point_scale(point_add(point_scale(self.a, t), self.b), t),
            self.c,
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct EvalCubic {
    // at^3 + bt^2 + ct + d
    a: (f32, f32),
    b: (f32, f32),
    c: (f32, f32),
    d: (f32, f32),
}

impl EvalCubic {
    fn new(points: [(f32, f32); 4]) -> Self {
        Self {
            a: point_sub(
                point_add(points[3], point_scale(point_sub(points[1], points[2]), 3.0)),
                points[0],
            ),
            b: point_scale(
                point_add(point_sub(points[2], two(points[1])), points[0]),
                3.0,
            ),
            c: point_scale(point_sub(points[1], points[0]), 3.0),
            d: points[0],
        }
    }

    fn evaluate(&self, t: f32) -> (f32, f32) {
        point_add(
            point_scale(
                point_add(
                    point_scale(point_add(point_scale(self.a, t), self.b), t),
                    self.c,
                ),
                t,
            ),
            self.d,
        )
    }
}

fn quad_subdivide(points: [(f32, f32); 3], t: f32) -> [(f32, f32); 5] {
    debug_assert!(t >= 0.0 && t <= 1.0);
    let ab = weighted_lerp_point(points[0], points[1], t);
    let bc = weighted_lerp_point(points[1], points[2], t);
    [points[0], ab, weighted_lerp_point(ab, bc, t), bc, points[2]]
}

fn cubic_subdivide(points: [(f32, f32); 4], t: f32) -> [(f32, f32); 7] {
    debug_assert!(t >= 0.0 && t <= 1.0);
    let ab = weighted_lerp_point(points[0], points[1], t);
    let bc = weighted_lerp_point(points[1], points[2], t);
    let cd = weighted_lerp_point(points[2], points[3], t);
    let abc = weighted_lerp_point(ab, bc, t);
    let bcd = weighted_lerp_point(bc, cd, t);
    [
        points[0],
        ab,
        abc,
        weighted_lerp_point(abc, bcd, t),
        bcd,
        cd,
        points[3],
    ]
}

fn line_extract(points: [(f32, f32); 2], start_t: f32, end_t: f32) -> [(f32, f32); 2] {
    debug_assert!(start_t <= end_t);
    debug_assert!(start_t >= 0.0 && end_t <= 1.0);

    [
        weighted_lerp_point(points[0], points[1], start_t),
        weighted_lerp_point(points[0], points[1], end_t),
    ]
}

fn quad_extract(points: [(f32, f32); 3], start_t: f32, end_t: f32) -> [(f32, f32); 3] {
    debug_assert!(start_t <= end_t);
    debug_assert!(start_t >= 0.0 && end_t <= 1.0);

    if start_t == 0.0 && end_t == 1.0 {
        points
    } else if start_t == 0.0 {
        let subdivided = quad_subdivide(points, end_t);
        [subdivided[0], subdivided[1], subdivided[2]]
    } else if end_t == 1.0 {
        let subdivided = quad_subdivide(points, start_t);
        [subdivided[2], subdivided[3], subdivided[4]]
    } else {
        debug_assert!(end_t > 0.0);
        let subdivided = quad_subdivide(points, end_t);
        let subdivided_again = quad_subdivide(
            [subdivided[0], subdivided[1], subdivided[2]],
            start_t / end_t,
        );
        [
            subdivided_again[2],
            subdivided_again[3],
            subdivided_again[4],
        ]
    }
}

fn cubic_extract(points: [(f32, f32); 4], start_t: f32, end_t: f32) -> [(f32, f32); 4] {
    debug_assert!(start_t <= end_t);
    debug_assert!(start_t >= 0.0 && end_t <= 1.0);

    if start_t == 0.0 && end_t == 1.0 {
        points
    } else if start_t == 0.0 {
        let subdivided = cubic_subdivide(points, end_t);
        [subdivided[0], subdivided[1], subdivided[2], subdivided[3]]
    } else if end_t == 1.0 {
        let subdivided = cubic_subdivide(points, start_t);
        [subdivided[3], subdivided[4], subdivided[5], subdivided[6]]
    } else {
        debug_assert!(end_t > 0.0);
        let subdivided = cubic_subdivide(points, end_t);
        let subdivided_again = cubic_subdivide(
            [subdivided[0], subdivided[1], subdivided[2], subdivided[3]],
            start_t / end_t,
        );
        [
            subdivided_again[3],
            subdivided_again[4],
            subdivided_again[5],
            subdivided_again[6],
        ]
    }
}

fn point_add(left: (f32, f32), right: (f32, f32)) -> (f32, f32) {
    (left.0 + right.0, left.1 + right.1)
}

fn point_sub(left: (f32, f32), right: (f32, f32)) -> (f32, f32) {
    (left.0 - right.0, left.1 - right.1)
}

fn point_scale(point: (f32, f32), scale: f32) -> (f32, f32) {
    (point.0 * scale, point.1 * scale)
}
