use crate::mechanical_port::source::generated::data_bind::bindable_property_color_base::{
    BindablePropertyColorBase, BindablePropertyColorBaseCallbacks,
};

#[derive(Default)]
pub struct BindablePropertyColor {
    pub base: BindablePropertyColorBase,
}
impl BindablePropertyColorBaseCallbacks for BindablePropertyColor {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}
impl BindablePropertyColor {
    pub const DEFAULT_VALUE: i32 = 0;
}
