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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PathVerb {
    Move = 0,
    Line = 1,
    Quad = 2,
    Cubic = 4,
    Close = 5,
}

pub fn path_verb_to_point_count(verb: PathVerb) -> usize {
    match verb {
        PathVerb::Move | PathVerb::Line => 1,
        PathVerb::Quad => 2,
        PathVerb::Cubic => 3,
        PathVerb::Close => 0,
    }
}
