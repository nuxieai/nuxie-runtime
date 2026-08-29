#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ClipResult {
    NoClip = 0,
    Clip = 1,
    EmptyClip = 2,
}
