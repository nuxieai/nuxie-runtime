use crate::mechanical_port::source::{
    animation::listener_types::listener_input_type::ListenerInputType, core::Core,
    core::binary_reader::BinaryReader,
};

pub trait ListenerInputTypeBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn listener_type_value_changed(&mut self) {}
}

pub struct ListenerInputTypeBase {
    pub base: Core,
    listener_type_value: u32,
}

impl Default for ListenerInputTypeBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            listener_type_value: 0,
        }
    }
}

impl ListenerInputTypeBase {
    pub const TYPE_KEY: u16 = 658;
    pub const LISTENER_TYPE_VALUE_PROPERTY_KEY: u16 = 965;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn listener_type_value(&self) -> u32 {
        self.listener_type_value
    }
    pub fn set_listener_type_value(
        &mut self,
        value: u32,
        callbacks: &mut impl ListenerInputTypeBaseCallbacks,
    ) {
        if !self.set_listener_type_value_value(value) {
            return;
        }
        callbacks.listener_type_value_changed();
        callbacks.notify_property_changed(Self::LISTENER_TYPE_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_listener_type_value_value(&mut self, value: u32) -> bool {
        if self.listener_type_value == value {
            return false;
        }
        self.listener_type_value = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ListenerInputTypeBaseCallbacks,
    ) -> ListenerInputType {
        let mut cloned = ListenerInputType::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ListenerInputTypeBaseCallbacks) {
        self.listener_type_value = object.listener_type_value;
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ListenerInputTypeBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::LISTENER_TYPE_VALUE_PROPERTY_KEY => {
                self.listener_type_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}

impl std::ops::Deref for ListenerInputTypeBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ListenerInputTypeBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
