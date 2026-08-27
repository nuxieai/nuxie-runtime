use crate::mechanical_port::source::core::{
    binary_reader::BinaryReader, field_types::core_string_type::CoreStringType,
};

pub trait AssetBaseCallbacks {
    fn name_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

#[derive(Default)]
pub struct AssetBase {
    name: String,
}

impl AssetBase {
    pub const TYPE_KEY: u16 = 99;
    pub const NAME_PROPERTY_KEY: u16 = 203;

    pub fn is_type_of(type_key: u16) -> bool {
        type_key == Self::TYPE_KEY
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name<C: AssetBaseCallbacks>(&mut self, value: String, callbacks: &mut C) {
        if self.name == value {
            return;
        }
        self.name = value;
        callbacks.name_changed();
        callbacks.notify_property_changed(Self::NAME_PROPERTY_KEY);
    }

    pub fn copy(&mut self, object: &Self) {
        self.name.clone_from(&object.name);
    }

    pub fn deserialize(&mut self, property_key: u16, reader: &mut BinaryReader<'_>) -> bool {
        match property_key {
            Self::NAME_PROPERTY_KEY => {
                self.name = CoreStringType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}
