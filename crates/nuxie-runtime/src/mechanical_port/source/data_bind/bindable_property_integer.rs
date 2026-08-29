use crate::mechanical_port::source::generated::data_bind::bindable_property_integer_base::{
    BindablePropertyIntegerBase, BindablePropertyIntegerBaseCallbacks,
};

#[derive(Default)]
pub struct BindablePropertyInteger {
    pub base: BindablePropertyIntegerBase,
}

impl std::ops::Deref for BindablePropertyInteger {
    type Target = BindablePropertyIntegerBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for BindablePropertyInteger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl BindablePropertyIntegerBaseCallbacks for BindablePropertyInteger {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}
impl BindablePropertyInteger {
    pub const DEFAULT_VALUE: u32 = 0;
}
