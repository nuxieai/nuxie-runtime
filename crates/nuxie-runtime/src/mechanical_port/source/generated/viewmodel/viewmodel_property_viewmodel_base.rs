use crate::mechanical_port::source::viewmodel::viewmodel_property_viewmodel::ViewModelPropertyViewModel;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, viewmodel::viewmodel_property::ViewModelProperty,
};

pub trait ViewModelPropertyViewModelBaseCallbacks: crate::mechanical_port::source::generated::viewmodel::viewmodel_property_base::ViewModelPropertyBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn view_model_reference_id_changed(&mut self) {}
}

pub struct ViewModelPropertyViewModelBase {
    pub base: ViewModelProperty,
    view_model_reference_id: u32,
}

impl Default for ViewModelPropertyViewModelBase {
    fn default() -> Self {
        Self {
            base: ViewModelProperty::default(),
            view_model_reference_id: 0,
        }
    }
}

impl ViewModelPropertyViewModelBase {
    pub const TYPE_KEY: u16 = 436;
    pub const VIEW_MODEL_REFERENCE_ID_PROPERTY_KEY: u16 = 565;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 430 | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn view_model_reference_id(&self) -> u32 {
        self.view_model_reference_id
    }
    pub fn set_view_model_reference_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ViewModelPropertyViewModelBaseCallbacks,
    ) {
        if !self.set_view_model_reference_id_value(value) {
            return;
        }
        callbacks.view_model_reference_id_changed();
        ViewModelPropertyViewModelBaseCallbacks::notify_property_changed(
            callbacks,
            Self::VIEW_MODEL_REFERENCE_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_view_model_reference_id_value(&mut self, value: u32) -> bool {
        if self.view_model_reference_id == value {
            return false;
        }
        self.view_model_reference_id = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ViewModelPropertyViewModelBaseCallbacks,
    ) -> ViewModelPropertyViewModel {
        let mut cloned = ViewModelPropertyViewModel::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ViewModelPropertyViewModelBaseCallbacks,
    ) {
        self.view_model_reference_id = object.view_model_reference_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelPropertyViewModelBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VIEW_MODEL_REFERENCE_ID_PROPERTY_KEY => {
                self.view_model_reference_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ViewModelPropertyViewModelBase {
    type Target = ViewModelProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelPropertyViewModelBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
