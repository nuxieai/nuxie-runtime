#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LuauCaptureType {
    LCT_VAL = 0,
    LCT_REF = 1,
    LCT_UPVAL = 2,
}

impl LuauCaptureType {
    pub const LCT_VAL: Self = Self::LCT_VAL;
    pub const LCT_REF: Self = Self::LCT_REF;
    pub const LCT_UPVAL: Self = Self::LCT_UPVAL;
}
