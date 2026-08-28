use crate::mechanical_port::source::{
    animation::state_machine_input::StateMachineInput,
    animation::state_machine_number::StateMachineNumber, core::binary_reader::BinaryReader,
};

pub trait StateMachineNumberBaseCallbacks: crate::mechanical_port::source::generated::animation::state_machine_component_base::StateMachineComponentBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct StateMachineNumberBase {
    pub base: StateMachineInput,
    value: f32,
}

impl Default for StateMachineNumberBase {
    fn default() -> Self {
        Self {
            base: StateMachineInput::default(),
            value: 0.0,
        }
    }
}

impl StateMachineNumberBase {
    pub const TYPE_KEY: u16 = 56;
    pub const VALUE_PROPERTY_KEY: u16 = 140;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 55 | 54)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    pub fn set_value(&mut self, value: f32, callbacks: &mut impl StateMachineNumberBaseCallbacks) {
        if !self.set_value_value(value) {
            return;
        }
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_value_value(&mut self, value: f32) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl StateMachineNumberBaseCallbacks,
    ) -> StateMachineNumber {
        let mut cloned = StateMachineNumber::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl StateMachineNumberBaseCallbacks) {
        self.value = object.value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl StateMachineNumberBaseCallbacks,
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

impl std::ops::Deref for StateMachineNumberBase {
    type Target = StateMachineInput;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StateMachineNumberBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
