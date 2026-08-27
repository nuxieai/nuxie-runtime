use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::data_converter::DataConverter,
    data_bind::converters::data_converter_boolean_negate::DataConverterBooleanNegate,
};

pub struct DataConverterBooleanNegateBase {
    pub base: DataConverter,
}

impl Default for DataConverterBooleanNegateBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
        }
    }
}

impl DataConverterBooleanNegateBase {
    pub const TYPE_KEY: u16 = 535;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> DataConverterBooleanNegate {
        let mut cloned = DataConverterBooleanNegate::default();
        cloned.base.copy(self);
        cloned
    }
}
