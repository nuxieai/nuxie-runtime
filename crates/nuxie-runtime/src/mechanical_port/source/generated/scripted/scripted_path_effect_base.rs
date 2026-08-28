use crate::mechanical_port::source::{
    container_component::ContainerComponent,
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
    scripted::scripted_path_effect::ScriptedPathEffect,
};

pub trait ScriptedPathEffectBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn script_asset_id_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct ScriptedPathEffectBase {
    pub base: ContainerComponent,
    script_asset_id: u32,
}

impl Default for ScriptedPathEffectBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
            script_asset_id: u32::MAX,
        }
    }
}

impl ScriptedPathEffectBase {
    pub const TYPE_KEY: u16 = 640;
    pub const SCRIPT_ASSET_ID_PROPERTY_KEY: u16 = 912;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn script_asset_id(&self) -> u32 {
        self.script_asset_id
    }
    pub fn set_script_asset_id<C: ScriptedPathEffectBaseCallbacks>(
        &mut self,
        value: u32,
        c: &mut C,
    ) {
        if !self.set_script_asset_id_value(value) {
            return;
        }
        c.script_asset_id_changed();
        c.notify_property_changed(Self::SCRIPT_ASSET_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_script_asset_id_value(&mut self, value: u32) -> bool {
        if self.script_asset_id == value {
            return false;
        }
        self.script_asset_id = value;
        true
    }
    pub fn clone_into<C: ScriptedPathEffectBaseCallbacks>(&self, c: &mut C) -> ScriptedPathEffect {
        let mut cloned = ScriptedPathEffect::default();
        cloned.base.copy(self, c);
        cloned
    }
    pub fn copy<C: ScriptedPathEffectBaseCallbacks>(&mut self, object: &Self, c: &mut C) {
        self.script_asset_id = object.script_asset_id;
        self.base.copy(&object.base, c);
    }
    pub fn deserialize<C: ScriptedPathEffectBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        c: &mut C,
    ) -> bool {
        match key {
            Self::SCRIPT_ASSET_ID_PROPERTY_KEY => {
                self.script_asset_id = CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(key, reader, c),
        }
    }
}

impl std::ops::Deref for ScriptedPathEffectBase {
    type Target = ContainerComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScriptedPathEffectBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
