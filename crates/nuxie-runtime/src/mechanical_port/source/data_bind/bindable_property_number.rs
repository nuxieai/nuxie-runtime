use crate::mechanical_port::source::generated::data_bind::bindable_property_number_base::BindablePropertyNumberBase;

#[derive(Default)]
pub struct BindablePropertyNumber {
    pub base: BindablePropertyNumberBase,
}
impl BindablePropertyNumber {
    pub const DEFAULT_VALUE: f32 = 0.0;
}
