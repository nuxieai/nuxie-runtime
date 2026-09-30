//! raw_path_test.cpp self-append regression at db31ea87.
use nuxie_runtime::source::math::{
    mat2d::Mat2D, path_types::PathVerb, raw_path::RawPath, vec2d::Vec2D,
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
