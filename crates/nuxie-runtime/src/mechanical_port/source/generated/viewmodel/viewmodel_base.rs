use crate::mechanical_port::source::viewmodel::viewmodel::ViewModel;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_component::ViewModelComponent,
};

pub trait ViewModelBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn view_model_type_changed(&mut self) {}
}

pub struct ViewModelBase {
    pub base: ViewModelComponent,
    view_model_type: u32,
}

impl Default for ViewModelBase {
    fn default() -> Self {
        Self {
            base: ViewModelComponent::default(),
            view_model_type: 0,
        }
    }
}

impl ViewModelBase {
    pub const TYPE_KEY: u16 = 435;
    pub const VIEW_MODEL_TYPE_PROPERTY_KEY: u16 = 981;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn view_model_type(&self) -> u32 {
        self.view_model_type
    }
    pub fn set_view_model_type(&mut self, value: u32, callbacks: &mut impl ViewModelBaseCallbacks) {
        if self.view_model_type == value {
            return;
        }
        self.view_model_type = value;
        callbacks.view_model_type_changed();
        callbacks.notify_property_changed(Self::VIEW_MODEL_TYPE_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl ViewModelBaseCallbacks) -> ViewModel {
        let mut cloned = ViewModel::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ViewModelBaseCallbacks) {
        self.view_model_type = object.view_model_type;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VIEW_MODEL_TYPE_PROPERTY_KEY => {
                self.view_model_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
