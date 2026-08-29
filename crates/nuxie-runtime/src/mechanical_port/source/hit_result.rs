#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HitResult {
    None,
    Hit,
    HitOpaque,
}
