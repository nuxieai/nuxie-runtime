use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::data_converter::DataConverter,
    data_bind::converters::data_converter_trigger::DataConverterTrigger,
};

pub struct DataConverterTriggerBase {
    pub base: DataConverter,
}

impl Default for DataConverterTriggerBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
        }
    }
}

impl DataConverterTriggerBase {
    pub const TYPE_KEY: u16 = 504;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> DataConverterTrigger {
        let mut cloned = DataConverterTrigger::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for DataConverterTriggerBase {
    type Target = DataConverter;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DataConverterTriggerBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
