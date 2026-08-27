use crate::mechanical_port::source::{
    animation::listener_bool_change::ListenerBoolChange,
    animation::listener_input_change::ListenerInputChange, core::binary_reader::BinaryReader,
};

pub trait ListenerBoolChangeBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct ListenerBoolChangeBase {
    pub base: ListenerInputChange,
    value: u32,
}

impl Default for ListenerBoolChangeBase {
    fn default() -> Self {
        Self {
            base: ListenerInputChange::default(),
            value: 1,
        }
    }
}

impl ListenerBoolChangeBase {
    pub const TYPE_KEY: u16 = 117;
    pub const VALUE_PROPERTY_KEY: u16 = 228;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 116 | 125)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> u32 {
        self.value
    }
    pub fn set_value(&mut self, value: u32, callbacks: &mut impl ListenerBoolChangeBaseCallbacks) {
        if self.value == value {
            return;
        }
        self.value = value;
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ListenerBoolChangeBaseCallbacks,
    ) -> ListenerBoolChange {
        let mut cloned = ListenerBoolChange::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ListenerBoolChangeBaseCallbacks) {
        self.value = object.value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ListenerBoolChangeBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
