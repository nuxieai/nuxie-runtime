#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum Type {
    Checked,
    Native,
    Deprecated,
    DebugNoinline,
    Unknown,
}

impl Default for Type {
    fn default() -> Self {
        Self::Unknown
    }
}
