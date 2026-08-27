#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StrokeCap {
    Butt = 0,
    Round = 1,
    Square = 2,
}
