use crate::mechanical_port::source::{
    animation::state_machine_fire_action::StateMachineFireAction, core::Core,
    core::binary_reader::BinaryReader,
};

pub trait StateMachineFireActionBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn occurs_value_changed(&mut self) {}
}

pub struct StateMachineFireActionBase {
    pub base: Core,
    occurs_value: u32,
}

impl Default for StateMachineFireActionBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            occurs_value: 0,
        }
    }
}

impl StateMachineFireActionBase {
    pub const TYPE_KEY: u16 = 615;
    pub const OCCURS_VALUE_PROPERTY_KEY: u16 = 393;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn occurs_value(&self) -> u32 {
        self.occurs_value
    }
    pub fn set_occurs_value(
        &mut self,
        value: u32,
        callbacks: &mut impl StateMachineFireActionBaseCallbacks,
    ) {
        if !self.set_occurs_value_value(value) {
            return;
        }
        callbacks.occurs_value_changed();
        callbacks.notify_property_changed(Self::OCCURS_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_occurs_value_value(&mut self, value: u32) -> bool {
        if self.occurs_value == value {
            return false;
        }
        self.occurs_value = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl StateMachineFireActionBaseCallbacks,
    ) -> StateMachineFireAction {
        let mut cloned = StateMachineFireAction::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl StateMachineFireActionBaseCallbacks,
    ) {
        self.occurs_value = object.occurs_value;
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl StateMachineFireActionBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::OCCURS_VALUE_PROPERTY_KEY => {
                self.occurs_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}

impl std::ops::Deref for StateMachineFireActionBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StateMachineFireActionBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
