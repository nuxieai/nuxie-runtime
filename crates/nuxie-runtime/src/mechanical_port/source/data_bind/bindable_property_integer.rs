use crate::mechanical_port::source::generated::data_bind::bindable_property_integer_base::BindablePropertyIntegerBase;

#[derive(Default)]
pub struct BindablePropertyInteger {
    pub base: BindablePropertyIntegerBase,
}
impl BindablePropertyInteger {
    pub const DEFAULT_VALUE: u32 = 0;
}
