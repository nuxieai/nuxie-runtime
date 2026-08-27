#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DataConverterToStringFlags(pub u16);

impl DataConverterToStringFlags {
    pub const NONE: Self = Self(0);
    pub const ROUND: Self = Self(1 << 0);
    pub const TRAILING_ZEROS: Self = Self(1 << 1);
    pub const FORMAT_WITH_COMMAS: Self = Self(1 << 2);
}
