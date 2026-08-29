#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Easing {
    EaseIn = 0,
    EaseOut = 1,
    EaseInOut = 2,
}
