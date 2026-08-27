use crate::mechanical_port::source::{
    animation::state_machine_bool::StateMachineBool,
    animation::state_machine_input::StateMachineInput, core::binary_reader::BinaryReader,
};

pub trait StateMachineBoolBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct StateMachineBoolBase {
    pub base: StateMachineInput,
    value: bool,
}

impl Default for StateMachineBoolBase {
    fn default() -> Self {
        Self {
            base: StateMachineInput::default(),
            value: false,
        }
    }
}

impl StateMachineBoolBase {
    pub const TYPE_KEY: u16 = 59;
    pub const VALUE_PROPERTY_KEY: u16 = 141;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 55 | 54)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> bool {
        self.value
    }
    pub fn set_value(&mut self, value: bool, callbacks: &mut impl StateMachineBoolBaseCallbacks) {
        if self.value == value {
            return;
        }
        self.value = value;
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl StateMachineBoolBaseCallbacks,
    ) -> StateMachineBool {
        let mut cloned = StateMachineBool::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl StateMachineBoolBaseCallbacks) {
        self.value = object.value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl StateMachineBoolBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
