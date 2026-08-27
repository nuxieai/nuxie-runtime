use crate::mechanical_port::source::{
    animation::scripted_transition_condition::ScriptedTransitionCondition,
    animation::transition_condition::TransitionCondition, core::binary_reader::BinaryReader,
};

pub trait ScriptedTransitionConditionBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn script_asset_id_changed(&mut self) {}
}

pub struct ScriptedTransitionConditionBase {
    pub base: TransitionCondition,
    script_asset_id: u32,
}

impl Default for ScriptedTransitionConditionBase {
    fn default() -> Self {
        Self {
            base: TransitionCondition::default(),
            script_asset_id: u32::MAX,
        }
    }
}

impl ScriptedTransitionConditionBase {
    pub const TYPE_KEY: u16 = 647;
    pub const SCRIPT_ASSET_ID_PROPERTY_KEY: u16 = 931;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 476)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn script_asset_id(&self) -> u32 {
        self.script_asset_id
    }
    pub fn set_script_asset_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ScriptedTransitionConditionBaseCallbacks,
    ) {
        if self.script_asset_id == value {
            return;
        }
        self.script_asset_id = value;
        callbacks.script_asset_id_changed();
        callbacks.notify_property_changed(Self::SCRIPT_ASSET_ID_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ScriptedTransitionConditionBaseCallbacks,
    ) -> ScriptedTransitionCondition {
        let mut cloned = ScriptedTransitionCondition::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ScriptedTransitionConditionBaseCallbacks,
    ) {
        self.script_asset_id = object.script_asset_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ScriptedTransitionConditionBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::SCRIPT_ASSET_ID_PROPERTY_KEY => {
                self.script_asset_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
