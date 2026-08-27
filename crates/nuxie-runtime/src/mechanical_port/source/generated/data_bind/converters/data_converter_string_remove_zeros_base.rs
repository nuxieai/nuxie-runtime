use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::data_converter::DataConverter,
    data_bind::converters::data_converter_string_remove_zeros::DataConverterStringRemoveZeros,
};

pub struct DataConverterStringRemoveZerosBase {
    pub base: DataConverter,
}

impl Default for DataConverterStringRemoveZerosBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
        }
    }
}

impl DataConverterStringRemoveZerosBase {
    pub const TYPE_KEY: u16 = 531;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> DataConverterStringRemoveZeros {
        let mut cloned = DataConverterStringRemoveZeros::default();
        cloned.base.copy(self);
        cloned
    }
}
