use crate::mechanical_port::source::generated::data_bind::bindable_property_base::BindablePropertyBase;

#[derive(Default)]
pub struct BindableProperty {
    pub base: BindablePropertyBase,
}

impl std::ops::Deref for BindableProperty {
    type Target = BindablePropertyBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for BindableProperty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
