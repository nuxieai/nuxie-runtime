use crate::mechanical_port::source::{
    animation::listener_action::ListenerAction,
    animation::scripted_listener_action::ScriptedListenerAction, core::binary_reader::BinaryReader,
};

pub trait ScriptedListenerActionBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn script_asset_id_changed(&mut self) {}
}

pub struct ScriptedListenerActionBase {
    pub base: ListenerAction,
    script_asset_id: u32,
}

impl Default for ScriptedListenerActionBase {
    fn default() -> Self {
        Self {
            base: ListenerAction::default(),
            script_asset_id: u32::MAX,
        }
    }
}

impl ScriptedListenerActionBase {
    pub const TYPE_KEY: u16 = 646;
    pub const SCRIPT_ASSET_ID_PROPERTY_KEY: u16 = 930;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 125)
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
        callbacks: &mut impl ScriptedListenerActionBaseCallbacks,
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
        callbacks: &mut impl ScriptedListenerActionBaseCallbacks,
    ) -> ScriptedListenerAction {
        let mut cloned = ScriptedListenerAction::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ScriptedListenerActionBaseCallbacks,
    ) {
        self.script_asset_id = object.script_asset_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ScriptedListenerActionBaseCallbacks,
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
