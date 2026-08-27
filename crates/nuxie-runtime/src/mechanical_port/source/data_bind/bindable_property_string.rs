use crate::mechanical_port::source::generated::data_bind::bindable_property_string_base::BindablePropertyStringBase;

#[derive(Default)]
pub struct BindablePropertyString {
    pub base: BindablePropertyStringBase,
}
impl BindablePropertyString {
    pub const DEFAULT_VALUE: &'static str = "";
}
