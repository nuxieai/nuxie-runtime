#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DrawTargetPlacement {
    Before = 0,
    After = 1,
}
