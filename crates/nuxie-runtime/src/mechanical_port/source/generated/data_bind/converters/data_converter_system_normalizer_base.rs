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
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for DataConverterSystemNormalizerBase {
    type Target = DataConverterOperationValue;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DataConverterSystemNormalizerBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
