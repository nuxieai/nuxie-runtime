use crate::mechanical_port::source::generated::data_bind::bindable_property_enum_base::BindablePropertyEnumBase;

#[derive(Default)]
pub struct BindablePropertyEnum {
    pub base: BindablePropertyEnumBase,
}
impl BindablePropertyEnum {
    pub const DEFAULT_VALUE: u16 = 0;
}
