use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, core::Core, viewmodel::data_enum::DataEnum,
};

pub struct DataEnumBase {
    pub base: Core,
}

impl Default for DataEnumBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
        }
    }
}

impl DataEnumBase {
    pub const TYPE_KEY: u16 = 510;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> DataEnum {
        let mut cloned = DataEnum::default();
        cloned.base.copy(self);
        cloned
    }
    pub fn copy(&mut self, object: &Self) {}
    pub fn deserialize(&mut self, property_key: u16, reader: &mut BinaryReader<'_>) -> bool {
        false
    }
}

impl std::ops::Deref for DataEnumBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DataEnumBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
