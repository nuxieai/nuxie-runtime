#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutMeasureMode {
    Undefined = 0,
    Exactly = 1,
    AtMost = 2,
}
