//! Direct ports from `tests/unit_tests/runtime/raw_path_test.cpp`.

use nuxie_render_api::{Aabb, Mat2D, PathVerb, RawPath};

fn cpp_bounds(path: &RawPath) -> Aabb {
    path.bounds()
        .unwrap_or_else(|| Aabb::new(0.0, 0.0, 0.0, 0.0))
}

#[test]
fn rawpath_basics() {
    let mut path = RawPath::new();

    assert!(path.verbs().is_empty());
    assert_eq!(cpp_bounds(&path), Aabb::new(0.0, 0.0, 0.0, 0.0));

    path.move_to(1.0, 2.0);
    assert!(!path.verbs().is_empty());
    assert_eq!(cpp_bounds(&path), Aabb::new(1.0, 2.0, 1.0, 2.0));

    path = RawPath::new();
    assert!(path.verbs().is_empty());
    assert_eq!(cpp_bounds(&path), Aabb::new(0.0, 0.0, 0.0, 0.0));

    path.move_to(1.0, -2.0);
    path.line_to(3.0, 4.0);
    path.line_to(-1.0, 5.0);
    assert!(!path.verbs().is_empty());
    assert_eq!(cpp_bounds(&path), Aabb::new(-1.0, -2.0, 3.0, 5.0));
}

fn validate_path_verb_point_counts(path: &RawPath) -> bool {
    let point_count = path
        .verbs()
        .iter()
        .map(|verb| match verb {
            PathVerb::Move | PathVerb::Line => 1,
            PathVerb::Quad => 2,
            PathVerb::Cubic => 3,
            PathVerb::Close => 0,
        })
        .sum::<usize>();
    point_count == path.points().len()
}

fn check_path_reversal(path: &RawPath) {
    let mut forwards = RawPath::new();
    forwards.add_path(path, Mat2D::IDENTITY);
    assert!(validate_path_verb_point_counts(&forwards));
    assert_eq!(&forwards, path);

    let mut backwards = RawPath::new();
    backwards.add_path_backwards(path, Mat2D::IDENTITY);
    assert_eq!(backwards.verbs().len(), path.verbs().len());
    assert_eq!(backwards.points().len(), path.points().len());
    assert!(validate_path_verb_point_counts(&backwards));

    let mut backwards_backwards = RawPath::new();
    backwards_backwards.add_path_backwards(&backwards, Mat2D::IDENTITY);
    assert!(validate_path_verb_point_counts(&backwards_backwards));
    assert_eq!(&backwards_backwards, path);
    assert_eq!(backwards_backwards, forwards);
}

#[test]
fn add_path_backwards() {
    {
        let mut path = RawPath::new();
        check_path_reversal(&path);

        path.move_to(0.0, 0.0);
        check_path_reversal(&path);

        path.move_to(0.0, 0.0);
        check_path_reversal(&path);

        path.move_to(10.0, 10.0);
        check_path_reversal(&path);

        path.close();
        check_path_reversal(&path);
    }

    {
        let mut path = RawPath::new();
        path.line_to(1.0, 2.0);
        check_path_reversal(&path);

        path.close();
        path.line_to(3.0, 4.0);
        check_path_reversal(&path);
    }

    {
        let mut path = RawPath::new();
        path.line_to(1.0, 2.0);
        path.close();
        path.close();
        path.close();
        path.close();
        path.close();
        path.line_to(3.0, 4.0);
        path.close();
        path.close();
        path.close();
        path.close();
        check_path_reversal(&path);
    }

    {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(32.0, 84.0);
        path.line_to(36.0, 76.0);
        path.close();
        path.move_to(0.0, 0.0);
        path.cubic_to(1.0, 57.0, 32.0, 10.0, 33.0, 86.0);
        path.line_to(20.0, 99.0);
        path.close();
        path.move_to(22.0, 59.0);
        path.move_to(62.0, 76.0);
        path.cubic_to(74.0, 39.0, 50.0, 35.0, 60.0, 26.0);
        path.move_to(26.0, 46.0);
        path.line_to(58.0, 76.0);
        path.line_to(93.0, 76.0);
        path.close();
        path.move_to(36.0, 74.0);
        path.line_to(3.0, 22.0);
        path.move_to(48.0, 47.0);
        path.move_to(47.0, 48.0);
        path.line_to(18.0, 94.0);
        path.cubic_to(35.0, 87.0, 73.0, 1.0, 74.0, 7.0);
        path.move_to(6.0, 50.0);
        path.close();
        path.close();
        path.line_to(13.0, 97.0);
        path.cubic_to(31.0, 26.0, 72.0, 94.0, 69.0, 32.0);
        path.move_to(80.0, 77.0);
        path.cubic_to(40.0, 23.0, 34.0, 34.0, 77.0, 41.0);
        path.cubic_to(58.0, 64.0, 26.0, 42.0, 28.0, 4.0);
        path.close();
        path.move_to(80.0, 77.0);
        path.line_to(20.0, 22.0);
        path.line_to(75.0, 26.0);
        path.line_to(88.0, 92.0);
        path.cubic_to(27.0, 7.0, 23.0, 84.0, 7.0, 89.0);
        path.close();
        check_path_reversal(&path);
    }
}
