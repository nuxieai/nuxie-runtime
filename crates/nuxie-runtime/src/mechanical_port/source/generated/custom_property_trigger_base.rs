use crate::mechanical_port::source::{
    core::{
        binary_reader::BinaryReader,
        field_types::{core_callback_type::CallbackData, core_uint_type::CoreUintType},
    },
    custom_property::CustomProperty,
    custom_property_trigger::CustomPropertyTrigger,
};

pub trait CustomPropertyTriggerBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn fire(&mut self, value: &mut CallbackData<'_>);
    fn property_value_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}
pub struct CustomPropertyTriggerBase {
    pub base: CustomProperty,
    property_value: u32,
}
impl Default for CustomPropertyTriggerBase {
    fn default() -> Self {
        Self {
            base: CustomProperty::default(),
            property_value: 0,
        }
    }
}
impl CustomPropertyTriggerBase {
    pub const TYPE_KEY: u16 = 613;
    pub const FIRE_PROPERTY_KEY: u16 = 869;
    pub const PROPERTY_VALUE_PROPERTY_KEY: u16 = 870;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 167 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_value(&self) -> u32 {
        self.property_value
    }
    pub fn set_property_value<C: CustomPropertyTriggerBaseCallbacks>(
        &mut self,
        value: u32,
        c: &mut C,
    ) {
        if !self.set_property_value_value(value) {
            return;
        }
        c.property_value_changed();
        c.notify_property_changed(Self::PROPERTY_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_property_value_value(&mut self, value: u32) -> bool {
        if self.property_value == value {
            return false;
        }
        self.property_value = value;
        true
    }
    pub fn copy<C: CustomPropertyTriggerBaseCallbacks>(&mut self, object: &Self, c: &mut C) {
        self.property_value = object.property_value;
        self.base.base.copy(&object.base.base, c);
    }
    pub fn deserialize<C: CustomPropertyTriggerBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        c: &mut C,
    ) -> bool {
        match key {
            Self::PROPERTY_VALUE_PROPERTY_KEY => {
                self.property_value = CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(key, reader, c),
        }
    }
    pub fn clone_into<C: CustomPropertyTriggerBaseCallbacks>(
        &self,
        c: &mut C,
    ) -> CustomPropertyTrigger {
        let mut cloned = CustomPropertyTrigger::default();
        cloned.base.copy(self, c);
        cloned
    }
}

impl std::ops::Deref for CustomPropertyTriggerBase {
    type Target = CustomProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomPropertyTriggerBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
