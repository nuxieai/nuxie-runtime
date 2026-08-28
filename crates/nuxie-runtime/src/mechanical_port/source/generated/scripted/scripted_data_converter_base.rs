use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
    data_bind::converters::data_converter::DataConverter,
    scripted::scripted_data_converter::ScriptedDataConverter,
};

pub trait ScriptedDataConverterBaseCallbacks: crate::mechanical_port::source::generated::data_bind::converters::data_converter_base::DataConverterBaseCallbacks {
    fn script_asset_id_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct ScriptedDataConverterBase {
    pub base: DataConverter,
    script_asset_id: u32,
}

impl Default for ScriptedDataConverterBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
            script_asset_id: u32::MAX,
        }
    }
}

impl ScriptedDataConverterBase {
    pub const TYPE_KEY: u16 = 629;
    pub const SCRIPT_ASSET_ID_PROPERTY_KEY: u16 = 892;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn script_asset_id(&self) -> u32 {
        self.script_asset_id
    }

    pub fn set_script_asset_id<C: ScriptedDataConverterBaseCallbacks>(
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

    pub fn clone_into<C: ScriptedDataConverterBaseCallbacks>(
        &self,
        c: &mut C,
    ) -> ScriptedDataConverter {
        let mut cloned = ScriptedDataConverter::default();
        cloned.base.copy(self, c);
        cloned
    }
    pub fn copy<C: ScriptedDataConverterBaseCallbacks>(&mut self, object: &Self, c: &mut C) {
        self.script_asset_id = object.script_asset_id;
        self.base.base.copy(&object.base.base, c);
    }
    pub fn deserialize<C: ScriptedDataConverterBaseCallbacks>(
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

impl std::ops::Deref for ScriptedDataConverterBase {
    type Target = DataConverter;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScriptedDataConverterBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
