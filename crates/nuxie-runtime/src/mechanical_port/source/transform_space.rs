#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct TransformSpace(u32);

#[allow(non_upper_case_globals)]
impl TransformSpace {
    pub const World: Self = Self(0);
    pub const Local: Self = Self(1);
}

impl From<u32> for TransformSpace {
    fn from(value: u32) -> Self {
        // C++'s fixed-underlying enum cast preserves unnamed wire values too.
        Self(value)
    }
}
