use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
    drawable::Drawable,
    scripted::scripted_drawable::ScriptedDrawable,
};

pub trait ScriptedDrawableBaseCallbacks {
    fn script_asset_id_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct ScriptedDrawableBase {
    pub base: Drawable,
    script_asset_id: u32,
}

impl Default for ScriptedDrawableBase {
    fn default() -> Self {
        Self {
            base: Drawable::default(),
            script_asset_id: u32::MAX,
        }
    }
}

impl ScriptedDrawableBase {
    pub const TYPE_KEY: u16 = 603;
    pub const SCRIPT_ASSET_ID_PROPERTY_KEY: u16 = 848;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn script_asset_id(&self) -> u32 {
        self.script_asset_id
    }
    pub fn set_script_asset_id<C: ScriptedDrawableBaseCallbacks>(&mut self, value: u32, c: &mut C) {
        if self.script_asset_id == value {
            return;
        }
        self.script_asset_id = value;
        c.script_asset_id_changed();
        c.notify_property_changed(Self::SCRIPT_ASSET_ID_PROPERTY_KEY);
    }
    pub fn clone_into<C: ScriptedDrawableBaseCallbacks>(&self, c: &mut C) -> ScriptedDrawable {
        let mut cloned = ScriptedDrawable::default();
        cloned.base.copy(self, c);
        cloned
    }
    pub fn copy<C: ScriptedDrawableBaseCallbacks>(&mut self, object: &Self, c: &mut C) {
        self.script_asset_id = object.script_asset_id;
        self.base.base.copy(&object.base.base, c);
    }
    pub fn deserialize<C: ScriptedDrawableBaseCallbacks>(
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
