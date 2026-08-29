#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DataConverterRangeMapperFlags(pub u16);

impl DataConverterRangeMapperFlags {
    pub const NONE: Self = Self(0);
    pub const CLAMP_LOWER: Self = Self(1 << 0);
    pub const CLAMP_UPPER: Self = Self(1 << 1);
    pub const MODULO: Self = Self(1 << 2);
    pub const REVERSE: Self = Self(1 << 3);
}
