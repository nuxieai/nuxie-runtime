use crate::mechanical_port::source::data_bind::converters::data_converter_operation_viewmodel::DataConverterOperationViewModel;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader,
    data_bind::converters::data_converter_operation::DataConverterOperation,
};

pub trait DataConverterOperationViewModelBaseCallbacks {
    fn source_path_ids_changed(&mut self) {}
    fn decode_source_path_ids(&mut self, value: &[u8]);
    fn copy_source_path_ids(&mut self, object: &DataConverterOperationViewModelBase);
}

pub struct DataConverterOperationViewModelBase {
    pub base: DataConverterOperation,
}

impl Default for DataConverterOperationViewModelBase {
    fn default() -> Self {
        Self {
            base: DataConverterOperation::default(),
        }
    }
}

impl DataConverterOperationViewModelBase {
    pub const TYPE_KEY: u16 = 517;
    pub const SOURCE_PATH_IDS_PROPERTY_KEY: u16 = 711;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 516 | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl DataConverterOperationViewModelBaseCallbacks,
    ) -> DataConverterOperationViewModel {
        let mut cloned = DataConverterOperationViewModel::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl DataConverterOperationViewModelBaseCallbacks,
    ) {
        callbacks.copy_source_path_ids(object);
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataConverterOperationViewModelBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::SOURCE_PATH_IDS_PROPERTY_KEY => {
                let value = crate::mechanical_port::source::core::field_types::core_bytes_type::CoreBytesType::deserialize(reader);
                callbacks.decode_source_path_ids(value.as_slice());
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
