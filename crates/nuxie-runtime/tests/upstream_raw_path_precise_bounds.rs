//! Complete "compute precise bounds of a raw path" case from bounds_test.cpp
//! at upstream af45192d7335c988bb7fe34ef2e9f8413e9b7a85.

use nuxie_runtime::source::math::{mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D};

fn approx(actual: f32, expected: f32) -> bool {
    // Catch Approx widens operands to double and defaults to 100 * float epsilon.
    (f64::from(actual) - f64::from(expected)).abs()
        <= 100.0 * f64::from(f32::EPSILON) * f64::from(expected).abs()
}

#[test]
fn compute_precise_bounds_of_a_raw_path() {
    let mut path = RawPath::default();
    path.move_to(0.0, 0.0);
    path.cubic_to(236.0, 10.0, 569.0, -58.0, 366.0, 180.0);
    path.cubic_to(163.0, 420.0, 508.0, 365.0, 408.0, 456.0);
    path.cubic_to(308.0, 547.0, -236.0, -10.0, 0.0, 0.0);
    path.close();

    let bounds = path.bounds();
    assert!(approx(bounds.left(), -236.0));
    assert!(approx(bounds.top(), -58.0));
    assert!(approx(bounds.right(), 569.0));
    assert!(approx(bounds.bottom(), 547.0));

    let precise_bounds = path.precise_bounds();
    assert!(approx(precise_bounds.left(), -58.79769));
    assert!(approx(precise_bounds.top(), -1.78456));
    assert!(approx(precise_bounds.right(), 428.90216));
    assert!(approx(precise_bounds.bottom(), 466.05313));

    // Rotation moves the cubic extrema. Mapping the control points before
    // solving must agree with precise bounds of a separately transformed copy.
    let xform = Mat2D::from_rotation(0.7).scale(Vec2D::new(1.5, 0.5));
    let transformed = path.transform(xform).precise_bounds();
    let in_place = path.precise_bounds_with_transform(xform);
    assert!(approx(in_place.left(), transformed.left()));
    assert!(approx(in_place.top(), transformed.top()));
    assert!(approx(in_place.right(), transformed.right()));
    assert!(approx(in_place.bottom(), transformed.bottom()));
    assert!(!approx(in_place.left(), precise_bounds.left()));
}
