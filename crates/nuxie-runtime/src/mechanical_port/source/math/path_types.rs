#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
    Clockwise,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathDirection {
    Clockwise,
    Counterclockwise,
}
pub use nuxie_render_api::PathVerb;

pub fn path_verb_to_point_count(verb: PathVerb) -> usize {
    match verb {
        PathVerb::Move | PathVerb::Line => 1,
        PathVerb::Quad => 2,
        PathVerb::Cubic => 3,
        PathVerb::Close => 0,
    }
}
