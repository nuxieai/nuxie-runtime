//! Upstream raw_path_test.cpp: nearby-origin coarse area, 1f7efd6c.
use nuxie_runtime::source::math::raw_path::RawPath;

fn assert_approx(actual: f32, expected: f32) {
    // Catch's default float Approx relative epsilon, with zero margin/scale.
    assert!(
        (actual - expected).abs() <= 100.0 * f32::EPSILON * expected.abs(),
        "{actual} != Approx({expected})"
    );
}

#[test]
fn coarse_area_measured_from_a_nearby_origin_ignores_position() {
    let mut curved_at_origin = 0.0;
    for offset in [0.0, 1e3, 1e6] {
        let mut path = RawPath::default();
        path.move_to(offset, offset);
        path.line_to(offset + 100.0, offset);
        path.line_to(offset + 100.0, offset + 100.0);
        path.line_to(offset, offset + 100.0);
        path.close();
        assert_approx(
            path.compute_coarse_area_with_origin(path.bounds().center()),
            10000.0,
        );

        let mut curved = RawPath::default();
        curved.move_to(offset, offset);
        curved.cubic_to(
            offset + 100.0,
            offset,
            offset + 100.0,
            offset + 100.0,
            offset,
            offset + 100.0,
        );
        curved.close();
        let area = curved.compute_coarse_area_with_origin(curved.bounds().center());
        if offset == 0.0 {
            curved_at_origin = area;
            assert!(curved_at_origin > 5000.0);
            assert_approx(curved.compute_coarse_area(), curved_at_origin);
        }
        assert_approx(area, curved_at_origin);
    }
}
