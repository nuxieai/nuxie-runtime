#[allow(non_camel_case_types)]
use crate::enums::r#type::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstantKey {
    pub(crate) r#type: Type,
    pub(crate) value: u64,
    pub(crate) extra: u64,
}
