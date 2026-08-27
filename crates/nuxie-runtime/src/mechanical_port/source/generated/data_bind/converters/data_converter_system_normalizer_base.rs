use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader,
    data_bind::converters::data_converter_operation_value::DataConverterOperationValue,
    data_bind::converters::data_converter_system_normalizer::DataConverterSystemNormalizer,
};

pub struct DataConverterSystemNormalizerBase {
    pub base: DataConverterOperationValue,
}

impl Default for DataConverterSystemNormalizerBase {
    fn default() -> Self {
        Self {
            base: DataConverterOperationValue::default(),
        }
    }
}

impl DataConverterSystemNormalizerBase {
    pub const TYPE_KEY: u16 = 515;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 500 | 516 | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> DataConverterSystemNormalizer {
        let mut cloned = DataConverterSystemNormalizer::default();
        cloned.base.copy(self);
        cloned
    }
}
