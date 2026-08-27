use crate::mechanical_port::source::generated::data_bind::bindable_property_color_base::BindablePropertyColorBase;

#[derive(Default)]
pub struct BindablePropertyColor {
    pub base: BindablePropertyColorBase,
}
impl BindablePropertyColor {
    pub const DEFAULT_VALUE: i32 = 0;
}
