#[allow(non_camel_case_types)]
use crate::enums::r#type::Type;

#[derive(Debug, Clone, Copy)]
pub struct Constant {
    pub(crate) r#type: Type,
    pub(crate) value: ConstantValue,
}

#[derive(Clone, Copy)]
#[repr(C)]
#[allow(non_snake_case)]
pub union ConstantValue {
    pub(crate) valueBoolean: bool,
    pub(crate) valueNumber: f64,
    pub(crate) valueInteger64: i64,
    pub(crate) valueVector: [f32; 4],
    pub(crate) valueString: u32,
    pub(crate) valueImport: u32,
    pub(crate) valueTable: u32,
    pub(crate) valueClosure: u32,
    pub(crate) valueClassShape: u32,
}

// A union has no active-variant tag of its own, so `Debug` cannot read a field
// safely; print it opaquely. The active variant is known only via the owning
// `Constant`'s `type` discriminant.
impl core::fmt::Debug for ConstantValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("ConstantValue(..)")
    }
}
