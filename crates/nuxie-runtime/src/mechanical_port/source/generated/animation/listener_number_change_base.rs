use crate::mechanical_port::source::{
    animation::listener_input_change::ListenerInputChange,
    animation::listener_number_change::ListenerNumberChange, core::binary_reader::BinaryReader,
};

pub trait ListenerNumberChangeBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct ListenerNumberChangeBase {
    pub base: ListenerInputChange,
    value: f32,
}

impl Default for ListenerNumberChangeBase {
    fn default() -> Self {
        Self {
            base: ListenerInputChange::default(),
            value: 0.0,
        }
    }
}

impl ListenerNumberChangeBase {
    pub const TYPE_KEY: u16 = 118;
    pub const VALUE_PROPERTY_KEY: u16 = 229;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 116 | 125)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    pub fn set_value(
        &mut self,
        value: f32,
        callbacks: &mut impl ListenerNumberChangeBaseCallbacks,
    ) {
        if self.value == value {
            return;
        }
        self.value = value;
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ListenerNumberChangeBaseCallbacks,
    ) -> ListenerNumberChange {
        let mut cloned = ListenerNumberChange::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ListenerNumberChangeBaseCallbacks) {
        self.value = object.value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ListenerNumberChangeBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
