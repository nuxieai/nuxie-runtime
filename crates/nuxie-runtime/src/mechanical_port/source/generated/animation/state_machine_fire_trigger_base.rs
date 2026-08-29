use crate::mechanical_port::source::{
    animation::state_machine_fire_action::StateMachineFireAction,
    animation::state_machine_fire_trigger::StateMachineFireTrigger,
    core::binary_reader::BinaryReader,
};

pub trait StateMachineFireTriggerBaseCallbacks: crate::mechanical_port::source::generated::animation::state_machine_fire_action_base::StateMachineFireActionBaseCallbacks {
    fn view_model_path_ids_changed(&mut self) {}
    fn decode_view_model_path_ids(&mut self, value: &[u8]);
    fn copy_view_model_path_ids(&mut self, object: &StateMachineFireTrigger);
}

pub struct StateMachineFireTriggerBase {
    pub base: StateMachineFireAction,
}

impl Default for StateMachineFireTriggerBase {
    fn default() -> Self {
        Self {
            base: StateMachineFireAction::default(),
        }
    }
}

impl StateMachineFireTriggerBase {
    pub const TYPE_KEY: u16 = 614;
    pub const VIEW_MODEL_PATH_IDS_PROPERTY_KEY: u16 = 871;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 615)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(source: &StateMachineFireTrigger) -> StateMachineFireTrigger {
        let mut cloned = StateMachineFireTrigger::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(source, &mut cloned);
        cloned.base = base;
        cloned
    }
    pub fn copy(
        &mut self,
        object: &StateMachineFireTrigger,
        callbacks: &mut impl StateMachineFireTriggerBaseCallbacks,
    ) {
        callbacks.copy_view_model_path_ids(object);
        self.base.copy(&object.base.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl StateMachineFireTriggerBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VIEW_MODEL_PATH_IDS_PROPERTY_KEY => {
                let value = crate::mechanical_port::source::core::field_types::core_bytes_type::CoreBytesType::deserialize(reader);
                callbacks.decode_view_model_path_ids(value.as_slice());
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for StateMachineFireTriggerBase {
    type Target = StateMachineFireAction;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StateMachineFireTriggerBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
