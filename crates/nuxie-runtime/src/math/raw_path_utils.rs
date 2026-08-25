// Direct source-correspondence owner for pinned `src/math/raw_path_utils.cpp`.
fn cubic_extract(points: [(f32, f32); 4], start_t: f32, end_t: f32) -> [(f32, f32); 4] {
    if start_t == 0.0 && end_t == 1.0 {
        points
    } else if start_t == 0.0 {
        let chopped = cubic_subdivide(points, end_t);
        [chopped[0], chopped[1], chopped[2], chopped[3]]
    } else if end_t == 1.0 {
        let chopped = cubic_subdivide(points, start_t);
        [chopped[3], chopped[4], chopped[5], chopped[6]]
    } else {
        let chopped = cubic_subdivide(points, end_t);
        let chopped_again = cubic_subdivide(
            [chopped[0], chopped[1], chopped[2], chopped[3]],
            start_t / end_t,
        );
        [
            chopped_again[3],
            chopped_again[4],
            chopped_again[5],
            chopped_again[6],
        ]
    }
}

fn cubic_subdivide(points: [(f32, f32); 4], t: f32) -> [(f32, f32); 7] {
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
