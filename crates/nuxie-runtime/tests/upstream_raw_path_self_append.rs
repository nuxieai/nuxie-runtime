//! raw_path_test.cpp self-append regression at db31ea87 and verb counts at 183df53f.
use nuxie_runtime::source::math::{
    aabb::Aabb,
    mat2d::Mat2D,
    path_types::{PathDirection, PathVerb},
    raw_path::RawPath,
    vec2d::Vec2D,
};

#[test]
fn raw_path_appends_itself_with_and_without_a_transform() {
    let mut path = RawPath::default();
    path.move_to(1.0, 2.0);
    path.line_to(3.0, 4.0);
    path.close();
    path.add_self_path(None);
    assert_eq!(path.verbs().len(), 6);
    assert_eq!(path.points().len(), 4);
    assert_eq!(path.points()[2], Vec2D::new(1.0, 2.0));
    assert_eq!(path.points()[3], Vec2D::new(3.0, 4.0));
    assert_eq!(path.verbs()[3], PathVerb::Move);
    let shift = Mat2D::new(1.0, 0.0, 0.0, 1.0, 10.0, 20.0);
    path.add_self_path(Some(&shift));
    assert_eq!(path.verbs().len(), 12);
    assert_eq!(path.points().len(), 8);
    assert_eq!(path.points()[4], Vec2D::new(11.0, 22.0));
    assert_eq!(path.points()[7], Vec2D::new(13.0, 24.0));
    assert_eq!(path.verbs()[9], PathVerb::Move);
}

fn check_verb_counts(path: &RawPath) {
    for verb in [
        PathVerb::Move,
        PathVerb::Line,
        PathVerb::Quad,
        PathVerb::Cubic,
        PathVerb::Close,
    ] {
        let expected = path.verbs().iter().filter(|&&v| v == verb).count();
        assert_eq!(path.verb_count(verb), expected);
    }
}

#[test]
fn verb_counts_track_every_mutation() {
    let mut path = RawPath::default();
    check_verb_counts(&path);

    path.line_to_point(Vec2D::new(1.0, 1.0)); // Injects an implicit move.
    path.quad_to_points(Vec2D::new(2.0, 2.0), Vec2D::new(3.0, 0.0));
    path.cubic_to_points(
        Vec2D::new(4.0, 4.0),
        Vec2D::new(5.0, 4.0),
        Vec2D::new(6.0, 0.0),
    );
    path.close();
    path.close(); // No-op, the contour is already closed.
    path.add_rect(Aabb::new(0.0, 0.0, 10.0, 10.0), PathDirection::Clockwise);
    path.add_oval(Aabb::new(0.0, 0.0, 10.0, 10.0), PathDirection::Clockwise);
    let poly = [
        Vec2D::new(0.0, 0.0),
        Vec2D::new(5.0, 0.0),
        Vec2D::new(5.0, 5.0),
    ];
    path.add_poly(&poly, true);
    path.add_poly(&poly, false);
    path.quad_to_cubic(7.0, 7.0, 8.0, 0.0);
    check_verb_counts(&path);
    assert_eq!(path.verb_count(PathVerb::Move), 5);

    let mut copy = path.clone();
    check_verb_counts(&copy);
    check_verb_counts(&path.transform(Mat2D::new(2.0, 0.0, 0.0, 2.0, 1.0, 1.0)));
    check_verb_counts(&RawPath::from_slices(path.verbs(), path.points()));

    copy.add_path(&path, None);
    check_verb_counts(&copy);
    copy.add_self_path(None);
    check_verb_counts(&copy);
    copy.add_path_backwards(&path, None);
    check_verb_counts(&copy);

    let mut degenerate = RawPath::default();
    degenerate.move_to_point(Vec2D::new(1.0, 1.0));
    degenerate.line_to_point(Vec2D::new(1.0, 1.0));
    degenerate.cubic_to_points(
        Vec2D::new(1.0, 1.0),
        Vec2D::new(1.0, 1.0),
        Vec2D::new(1.0, 1.0),
    );
    degenerate.line_to_point(Vec2D::new(2.0, 2.0));
    degenerate.close();
    // A matrix makes add_path_backwards prune empty segments.
    let identity = Mat2D::identity();
    copy.add_path_backwards(&degenerate, Some(&identity));
    check_verb_counts(&copy);
    degenerate.prune_empty_segments();
    check_verb_counts(&degenerate);
    assert_eq!(degenerate.verb_count(PathVerb::Cubic), 0);

    copy.swap(&mut degenerate);
    check_verb_counts(&copy);
    check_verb_counts(&degenerate);

    // mem::take retains an empty, usable moved-from owner in safe Rust.
    let mut moved = std::mem::take(&mut degenerate);
    check_verb_counts(&moved);
    check_verb_counts(&degenerate);
    assert_eq!(degenerate.verb_count(PathVerb::Move), 0);
    degenerate = std::mem::take(&mut moved);
    check_verb_counts(&degenerate);
    check_verb_counts(&moved);

    path.rewind();
    check_verb_counts(&path);
    copy.reset();
    check_verb_counts(&copy);
}
