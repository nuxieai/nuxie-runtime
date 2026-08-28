use crate::mechanical_port::source::{
    animation::listener_types::listener_input_type::ListenerInputType,
    animation::listener_types::listener_input_type_event::ListenerInputTypeEvent,
    core::binary_reader::BinaryReader,
};

pub trait ListenerInputTypeEventBaseCallbacks: crate::mechanical_port::source::generated::animation::listener_types::listener_input_type_base::ListenerInputTypeBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn event_id_changed(&mut self) {}
}

pub struct ListenerInputTypeEventBase {
    pub base: ListenerInputType,
    event_id: u32,
}

impl Default for ListenerInputTypeEventBase {
    fn default() -> Self {
        Self {
            base: ListenerInputType::default(),
            event_id: u32::MAX,
        }
    }
}

impl ListenerInputTypeEventBase {
    pub const TYPE_KEY: u16 = 659;
    pub const EVENT_ID_PROPERTY_KEY: u16 = 962;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 658)
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
        callbacks: &mut impl ListenerInputTypeEventBaseCallbacks,
    ) {
        if !self.set_event_id_value(value) {
            return;
        }
        callbacks.event_id_changed();
        callbacks.notify_property_changed(Self::EVENT_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_event_id_value(&mut self, value: u32) -> bool {
        if self.event_id == value {
            return false;
        }
        self.event_id = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ListenerInputTypeEventBaseCallbacks,
    ) -> ListenerInputTypeEvent {
        let mut cloned = ListenerInputTypeEvent::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ListenerInputTypeEventBaseCallbacks,
    ) {
        self.event_id = object.event_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ListenerInputTypeEventBaseCallbacks,
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

impl std::ops::Deref for ListenerInputTypeEventBase {
    type Target = ListenerInputType;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ListenerInputTypeEventBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
