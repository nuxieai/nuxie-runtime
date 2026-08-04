#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Global {
    Default = 0,
    Mutable,
    Written,
}

impl Global {
    pub const Default: Self = Self::Default;
    pub const Mutable: Self = Self::Mutable;
    pub const Written: Self = Self::Written;
}

impl Default for Global {
    fn default() -> Self {
        Self::Default
    }
}
