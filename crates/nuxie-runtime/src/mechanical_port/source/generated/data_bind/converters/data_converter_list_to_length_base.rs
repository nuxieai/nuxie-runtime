use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::data_converter::DataConverter,
    data_bind::converters::data_converter_list_to_length::DataConverterListToLength,
};

pub struct DataConverterListToLengthBase {
    pub base: DataConverter,
}

impl Default for DataConverterListToLengthBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
        }
    }
}

impl DataConverterListToLengthBase {
    pub const TYPE_KEY: u16 = 591;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> DataConverterListToLength {
        let mut cloned = DataConverterListToLength::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for DataConverterListToLengthBase {
    type Target = DataConverter;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DataConverterListToLengthBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
