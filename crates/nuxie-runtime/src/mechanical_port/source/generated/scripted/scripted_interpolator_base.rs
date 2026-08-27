use crate::mechanical_port::source::{
    animation::keyframe_interpolator::KeyFrameInterpolator,
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
    scripted::scripted_interpolator::ScriptedInterpolator,
};

pub trait ScriptedInterpolatorBaseCallbacks {
    fn script_asset_id_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct ScriptedInterpolatorBase {
    pub base: KeyFrameInterpolator,
    script_asset_id: u32,
}

impl Default for ScriptedInterpolatorBase {
    fn default() -> Self {
        Self {
            base: KeyFrameInterpolator::default(),
            script_asset_id: u32::MAX,
        }
    }
}

impl ScriptedInterpolatorBase {
    pub const TYPE_KEY: u16 = 972;
    pub const SCRIPT_ASSET_ID_PROPERTY_KEY: u16 = 1015;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 175)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn script_asset_id(&self) -> u32 {
        self.script_asset_id
    }
    pub fn set_script_asset_id<C: ScriptedInterpolatorBaseCallbacks>(
        &mut self,
        value: u32,
        c: &mut C,
    ) {
        if self.script_asset_id == value {
            return;
        }
        self.script_asset_id = value;
        c.script_asset_id_changed();
        c.notify_property_changed(Self::SCRIPT_ASSET_ID_PROPERTY_KEY);
    }
    pub fn clone_into<C: ScriptedInterpolatorBaseCallbacks>(
        &self,
        c: &mut C,
    ) -> ScriptedInterpolator {
        let mut cloned = ScriptedInterpolator::default();
        cloned.base.copy(self, c);
        cloned
    }
    pub fn copy<C: ScriptedInterpolatorBaseCallbacks>(&mut self, object: &Self, c: &mut C) {
        self.script_asset_id = object.script_asset_id;
        self.base.base.copy(&object.base.base, c);
    }
    pub fn deserialize<C: ScriptedInterpolatorBaseCallbacks>(
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
            _ => self.base.base.deserialize(key, reader, c),
        }
    }
}
