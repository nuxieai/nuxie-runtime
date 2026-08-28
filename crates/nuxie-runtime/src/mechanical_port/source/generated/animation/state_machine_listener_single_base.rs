use crate::mechanical_port::source::{
    animation::state_machine_listener::StateMachineListener,
    animation::state_machine_listener_single::StateMachineListenerSingle,
    core::binary_reader::BinaryReader,
};

pub trait StateMachineListenerSingleBaseCallbacks: crate::mechanical_port::source::generated::animation::state_machine_listener_base::StateMachineListenerBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn listener_type_value_changed(&mut self) {}
    fn event_id_changed(&mut self) {}
    fn view_model_path_ids_changed(&mut self) {}
    fn decode_view_model_path_ids(&mut self, value: &[u8]);
    fn copy_view_model_path_ids(&mut self, object: &StateMachineListenerSingleBase);
}

pub struct StateMachineListenerSingleBase {
    pub base: StateMachineListener,
    listener_type_value: u32,
    event_id: u32,
}

impl Default for StateMachineListenerSingleBase {
    fn default() -> Self {
        Self {
            base: StateMachineListener::default(),
            listener_type_value: 0,
            event_id: u32::MAX,
        }
    }
}

impl StateMachineListenerSingleBase {
    pub const TYPE_KEY: u16 = 114;
    pub const LISTENER_TYPE_VALUE_PROPERTY_KEY: u16 = 225;
    pub const EVENT_ID_PROPERTY_KEY: u16 = 399;
    pub const VIEW_MODEL_PATH_IDS_PROPERTY_KEY: u16 = 868;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 654 | 54)
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
        callbacks: &mut impl StateMachineListenerSingleBaseCallbacks,
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
    pub fn event_id(&self) -> u32 {
        self.event_id
    }
    pub fn set_event_id(
        &mut self,
        value: u32,
        callbacks: &mut impl StateMachineListenerSingleBaseCallbacks,
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
        callbacks: &mut impl StateMachineListenerSingleBaseCallbacks,
    ) -> StateMachineListenerSingle {
        let mut cloned = StateMachineListenerSingle::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl StateMachineListenerSingleBaseCallbacks,
    ) {
        self.listener_type_value = object.listener_type_value;
        self.event_id = object.event_id;
        callbacks.copy_view_model_path_ids(object);
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl StateMachineListenerSingleBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::LISTENER_TYPE_VALUE_PROPERTY_KEY => {
                self.listener_type_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::EVENT_ID_PROPERTY_KEY => {
                self.event_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::VIEW_MODEL_PATH_IDS_PROPERTY_KEY => {
                let value = crate::mechanical_port::source::core::field_types::core_bytes_type::CoreBytesType::deserialize(reader);
                callbacks.decode_view_model_path_ids(value.as_slice());
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for StateMachineListenerSingleBase {
    type Target = StateMachineListener;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StateMachineListenerSingleBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
