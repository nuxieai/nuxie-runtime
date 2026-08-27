use crate::mechanical_port::source::generated::data_bind::bindable_property_boolean_base::BindablePropertyBooleanBase;

#[derive(Default)]
pub struct BindablePropertyBoolean {
    pub base: BindablePropertyBooleanBase,
}
impl BindablePropertyBoolean {
    pub const DEFAULT_VALUE: bool = false;
}
