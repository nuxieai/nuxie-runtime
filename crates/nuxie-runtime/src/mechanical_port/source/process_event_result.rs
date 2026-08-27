#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProcessEventResult {
    None,
    Pointer,
    Scroll,
}
