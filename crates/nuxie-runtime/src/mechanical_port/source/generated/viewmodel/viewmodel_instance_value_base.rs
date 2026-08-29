use crate::mechanical_port::source::viewmodel::viewmodel_instance_value::ViewModelInstanceValue;

use crate::mechanical_port::source::{component::Component, core::binary_reader::BinaryReader};

pub trait ViewModelInstanceValueBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn view_model_property_id_changed(&mut self) {}
}

pub struct ViewModelInstanceValueBase {
    pub base: Component,
    view_model_property_id: u32,
}

impl Default for ViewModelInstanceValueBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            view_model_property_id: 0,
        }
    }
}

impl ViewModelInstanceValueBase {
    pub const TYPE_KEY: u16 = 428;
    pub const VIEW_MODEL_PROPERTY_ID_PROPERTY_KEY: u16 = 554;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn view_model_property_id(&self) -> u32 {
        self.view_model_property_id
    }
    pub fn set_view_model_property_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ViewModelInstanceValueBaseCallbacks,
    ) {
        if !self.set_view_model_property_id_value(value) {
            return;
        }
        callbacks.view_model_property_id_changed();
        ViewModelInstanceValueBaseCallbacks::notify_property_changed(
            callbacks,
            Self::VIEW_MODEL_PROPERTY_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_view_model_property_id_value(&mut self, value: u32) -> bool {
        if self.view_model_property_id == value {
            return false;
        }
        self.view_model_property_id = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ViewModelInstanceValueBaseCallbacks,
    ) -> ViewModelInstanceValue {
        let mut cloned = ViewModelInstanceValue::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ViewModelInstanceValueBaseCallbacks,
    ) {
        self.view_model_property_id = object.view_model_property_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelInstanceValueBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VIEW_MODEL_PROPERTY_ID_PROPERTY_KEY => {
                self.view_model_property_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ViewModelInstanceValueBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelInstanceValueBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
