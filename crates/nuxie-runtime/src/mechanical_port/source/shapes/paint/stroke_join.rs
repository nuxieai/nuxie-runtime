#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StrokeJoin {
    Miter = 0,
    Round = 1,
    Bevel = 2,
}
