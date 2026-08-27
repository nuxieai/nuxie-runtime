use crate::mechanical_port::source::{
    animation::state_machine_fire_action::StateMachineFireAction,
    animation::state_machine_fire_event::StateMachineFireEvent, core::binary_reader::BinaryReader,
};

pub trait StateMachineFireEventBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn event_id_changed(&mut self) {}
}

pub struct StateMachineFireEventBase {
    pub base: StateMachineFireAction,
    event_id: u32,
}

impl Default for StateMachineFireEventBase {
    fn default() -> Self {
        Self {
            base: StateMachineFireAction::default(),
            event_id: u32::MAX,
        }
    }
}

impl StateMachineFireEventBase {
    pub const TYPE_KEY: u16 = 169;
    pub const EVENT_ID_PROPERTY_KEY: u16 = 392;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 615)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn event_id(&self) -> u32 {
        self.event_id
    }
    pub fn set_event_id(
        &mut self,
        value: u32,
        callbacks: &mut impl StateMachineFireEventBaseCallbacks,
    ) {
        if self.event_id == value {
            return;
        }
        self.event_id = value;
        callbacks.event_id_changed();
        callbacks.notify_property_changed(Self::EVENT_ID_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl StateMachineFireEventBaseCallbacks,
    ) -> StateMachineFireEvent {
        let mut cloned = StateMachineFireEvent::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl StateMachineFireEventBaseCallbacks) {
        self.event_id = object.event_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl StateMachineFireEventBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::EVENT_ID_PROPERTY_KEY => {
                self.event_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
