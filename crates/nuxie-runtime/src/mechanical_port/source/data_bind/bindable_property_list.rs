use crate::mechanical_port::source::generated::data_bind::bindable_property_list_base::{
    BindablePropertyListBase, BindablePropertyListBaseCallbacks,
};

#[derive(Default)]
pub struct BindablePropertyList {
    pub base: BindablePropertyListBase,
}
impl BindablePropertyListBaseCallbacks for BindablePropertyList {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}
