#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum GestureClickPhase {
    Out = 0,
    Down = 1,
    Clicked = 2,
    Disabled = 3,
}
