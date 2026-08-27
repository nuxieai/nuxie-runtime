use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::data_converter::DataConverter,
    data_bind::converters::data_converter_operation::DataConverterOperation,
};

pub trait DataConverterOperationBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn operation_type_changed(&mut self) {}
}

pub struct DataConverterOperationBase {
    pub base: DataConverter,
    operation_type: u32,
}

impl Default for DataConverterOperationBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
            operation_type: 0,
        }
    }
}

impl DataConverterOperationBase {
    pub const TYPE_KEY: u16 = 516;
    pub const OPERATION_TYPE_PROPERTY_KEY: u16 = 682;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn operation_type(&self) -> u32 {
        self.operation_type
    }
    pub fn set_operation_type(
        &mut self,
        value: u32,
        callbacks: &mut impl DataConverterOperationBaseCallbacks,
    ) {
        if self.operation_type == value {
            return;
        }
        self.operation_type = value;
        callbacks.operation_type_changed();
        callbacks.notify_property_changed(Self::OPERATION_TYPE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl DataConverterOperationBaseCallbacks,
    ) -> DataConverterOperation {
        let mut cloned = DataConverterOperation::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl DataConverterOperationBaseCallbacks,
    ) {
        self.operation_type = object.operation_type;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataConverterOperationBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::OPERATION_TYPE_PROPERTY_KEY => {
                self.operation_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
