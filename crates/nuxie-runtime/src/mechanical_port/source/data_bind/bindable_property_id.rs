use crate::mechanical_port::source::generated::data_bind::bindable_property_id_base::{
    BindablePropertyIdBase, BindablePropertyIdBaseCallbacks,
};

#[derive(Default)]
pub struct BindablePropertyId {
    pub base: BindablePropertyIdBase,
}

impl std::ops::Deref for BindablePropertyId {
    type Target = BindablePropertyIdBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for BindablePropertyId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl BindablePropertyIdBaseCallbacks for BindablePropertyId {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}
