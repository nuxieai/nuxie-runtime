#[allow(non_camel_case_types)]
use crate::enums::r#type::Type;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstantKey {
    pub(crate) r#type: Type,
    pub(crate) value: u64,
    pub(crate) extra1: u64,
    pub(crate) extra2: u64,
    pub(crate) extra3: u64,
}
