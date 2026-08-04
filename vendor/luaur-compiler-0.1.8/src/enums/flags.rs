#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Flags {
    Flag_NoneSafe = 1 << 0,
}

impl Flags {
    pub const Flag_NoneSafe: Flags = Flags::Flag_NoneSafe;
}

impl Default for Flags {
    fn default() -> Self {
        Self::Flag_NoneSafe
    }
}
