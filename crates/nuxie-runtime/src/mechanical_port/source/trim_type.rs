#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TrimType {
    None = 0,
    Start = 1,
    End = 2,
    All = 3,
}
