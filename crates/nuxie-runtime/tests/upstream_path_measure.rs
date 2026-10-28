//! Complete path_measure_test.cpp at upstream 209da799.
use nuxie_runtime::source::math::{path_measure::PathMeasure, raw_path::RawPath};

fn approx(actual: f32, expected: f32) -> bool {
    (f64::from(actual) - f64::from(expected)).abs()
        <= 100.0 * f64::from(f32::EPSILON) * f64::from(expected).abs()
}

#[test]
fn a_distance_past_the_end_lands_on_the_end() {
    let mut path = RawPath::default();
    path.move_to(0.0, 0.0);
    path.line_to(10.0, 0.0);
    path.line_to(10.0, 10.0);
    let measure = PathMeasure::from_path_default(&path);
    let past = measure.at_distance(measure.length() * 1.001);
    assert!(approx(past.pos.x, 10.0));
    assert!(approx(past.pos.y, 10.0));
    let full = measure.at_percentage(1.0);
    assert!(approx(full.pos.x, 10.0));
    assert!(approx(full.pos.y, 10.0));
}

#[test]
fn an_empty_path_measure_positions_at_the_origin() {
    let result = PathMeasure::default().at_percentage(0.5);
    assert_eq!(result.pos.x, 0.0);
    assert_eq!(result.pos.y, 0.0);
    assert_eq!(result.tan.x, 0.0);
    assert_eq!(result.tan.y, 0.0);
}
