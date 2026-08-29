use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader,
    data_bind::converters::data_converter_operation::DataConverterOperation,
    data_bind::converters::data_converter_operation_value::DataConverterOperationValue,
};

pub trait DataConverterOperationValueBaseCallbacks: crate::mechanical_port::source::generated::data_bind::converters::data_converter_operation_base::DataConverterOperationBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn operation_value_changed(&mut self) {}
}

pub struct DataConverterOperationValueBase {
    pub base: DataConverterOperation,
    operation_value: f32,
}

impl Default for DataConverterOperationValueBase {
    fn default() -> Self {
        Self {
            base: DataConverterOperation::default(),
            operation_value: 1.0,
        }
    }
}

impl DataConverterOperationValueBase {
    pub const TYPE_KEY: u16 = 500;
    pub const OPERATION_VALUE_PROPERTY_KEY: u16 = 681;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 516 | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn operation_value(&self) -> f32 {
        self.operation_value
    }
    pub fn set_operation_value(
        &mut self,
        value: f32,
        callbacks: &mut impl DataConverterOperationValueBaseCallbacks,
    ) {
        if !self.set_operation_value_value(value) {
            return;
        }
        callbacks.operation_value_changed();
        DataConverterOperationValueBaseCallbacks::notify_property_changed(
            callbacks,
            Self::OPERATION_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_operation_value_value(&mut self, value: f32) -> bool {
        if self.operation_value == value {
            return false;
        }
        self.operation_value = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl DataConverterOperationValueBaseCallbacks,
    ) -> DataConverterOperationValue {
        let mut cloned = DataConverterOperationValue::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl DataConverterOperationValueBaseCallbacks,
    ) {
        self.operation_value = object.operation_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataConverterOperationValueBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::OPERATION_VALUE_PROPERTY_KEY => {
                self.operation_value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for DataConverterOperationValueBase {
    type Target = DataConverterOperation;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DataConverterOperationValueBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
