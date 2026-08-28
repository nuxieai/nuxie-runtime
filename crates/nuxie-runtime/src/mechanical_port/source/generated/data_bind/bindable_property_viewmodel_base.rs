use crate::mechanical_port::source::data_bind::bindable_property_viewmodel::BindablePropertyViewModel;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::bindable_property_id::BindablePropertyId,
};

pub struct BindablePropertyViewModelBase {
    pub base: BindablePropertyId,
}

impl Default for BindablePropertyViewModelBase {
    fn default() -> Self {
        Self {
            base: BindablePropertyId::default(),
        }
    }
}

impl BindablePropertyViewModelBase {
    pub const TYPE_KEY: u16 = 662;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 596 | 9)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> BindablePropertyViewModel {
        let mut cloned = BindablePropertyViewModel::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for BindablePropertyViewModelBase {
    type Target = BindablePropertyId;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BindablePropertyViewModelBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
