#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShapePathFlags {
    None = 0,
    Hidden = 1 << 0,
    IsCounterClockwise = 1 << 1,
}
