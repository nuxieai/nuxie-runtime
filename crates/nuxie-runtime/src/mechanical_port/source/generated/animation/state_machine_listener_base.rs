use crate::mechanical_port::source::{
    animation::state_machine_component::StateMachineComponent,
    animation::state_machine_listener::StateMachineListener, core::binary_reader::BinaryReader,
};

pub trait StateMachineListenerBaseCallbacks: crate::mechanical_port::source::generated::animation::state_machine_component_base::StateMachineComponentBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn target_id_changed(&mut self) {}
}

pub struct StateMachineListenerBase {
    pub base: StateMachineComponent,
    target_id: u32,
}

impl Default for StateMachineListenerBase {
    fn default() -> Self {
        Self {
            base: StateMachineComponent::default(),
            target_id: u32::MAX,
        }
    }
}

impl StateMachineListenerBase {
    pub const TYPE_KEY: u16 = 654;
    pub const TARGET_ID_PROPERTY_KEY: u16 = 224;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 54)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn target_id(&self) -> u32 {
        self.target_id
    }
    pub fn set_target_id(
        &mut self,
        value: u32,
        callbacks: &mut impl StateMachineListenerBaseCallbacks,
    ) {
        if !self.set_target_id_value(value) {
            return;
        }
        callbacks.target_id_changed();
        StateMachineListenerBaseCallbacks::notify_property_changed(
            callbacks,
            Self::TARGET_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_target_id_value(&mut self, value: u32) -> bool {
        if self.target_id == value {
            return false;
        }
        self.target_id = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl StateMachineListenerBaseCallbacks,
    ) -> StateMachineListener {
        let mut cloned = StateMachineListener::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl StateMachineListenerBaseCallbacks) {
        self.target_id = object.target_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl StateMachineListenerBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::TARGET_ID_PROPERTY_KEY => {
                self.target_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for StateMachineListenerBase {
    type Target = StateMachineComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StateMachineListenerBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
