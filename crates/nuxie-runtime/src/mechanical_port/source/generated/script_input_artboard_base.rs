use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, custom_property::CustomProperty,
    script_input_artboard::ScriptInputArtboard,
};

pub trait ScriptInputArtboardBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn artboard_id_changed(&mut self) {}
}

pub struct ScriptInputArtboardBase {
    pub base: CustomProperty,
    artboard_id: u32,
}

impl Default for ScriptInputArtboardBase {
    fn default() -> Self {
        Self {
            base: CustomProperty::default(),
            artboard_id: u32::MAX,
        }
    }
}

impl ScriptInputArtboardBase {
    pub const TYPE_KEY: u16 = 621;
    pub const ARTBOARD_ID_PROPERTY_KEY: u16 = 876;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 167 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn artboard_id(&self) -> u32 {
        self.artboard_id
    }
    pub fn set_artboard_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ScriptInputArtboardBaseCallbacks,
    ) {
        if !self.set_artboard_id_value(value) {
            return;
        }
        callbacks.artboard_id_changed();
        ScriptInputArtboardBaseCallbacks::notify_property_changed(
            callbacks,
            Self::ARTBOARD_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_artboard_id_value(&mut self, value: u32) -> bool {
        if self.artboard_id == value {
            return false;
        }
        self.artboard_id = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ScriptInputArtboardBaseCallbacks,
    ) -> ScriptInputArtboard {
        let mut cloned = ScriptInputArtboard::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ScriptInputArtboardBaseCallbacks) {
        self.artboard_id = object.artboard_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ScriptInputArtboardBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ARTBOARD_ID_PROPERTY_KEY => {
                self.artboard_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ScriptInputArtboardBase {
    type Target = CustomProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScriptInputArtboardBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
