use crate::mechanical_port::source::{component::Component, core::binary_reader::BinaryReader};

pub trait NestedInputBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn input_id_changed(&mut self) {}
}

pub struct NestedInputBase {
    pub base: Component,
    input_id: u32,
}

impl Default for NestedInputBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            input_id: u32::MAX,
        }
    }
}

impl NestedInputBase {
    pub const TYPE_KEY: u16 = 121;
    pub const INPUT_ID_PROPERTY_KEY: u16 = 237;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn input_id(&self) -> u32 {
        self.input_id
    }
    pub fn set_input_id(&mut self, value: u32, callbacks: &mut impl NestedInputBaseCallbacks) {
        if !self.set_input_id_value(value) {
            return;
        }
        callbacks.input_id_changed();
        NestedInputBaseCallbacks::notify_property_changed(callbacks, Self::INPUT_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_input_id_value(&mut self, value: u32) -> bool {
        if self.input_id == value {
            return false;
        }
        self.input_id = value;
        true
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NestedInputBaseCallbacks) {
        self.input_id = object.input_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NestedInputBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::INPUT_ID_PROPERTY_KEY => {
                self.input_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for NestedInputBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NestedInputBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
