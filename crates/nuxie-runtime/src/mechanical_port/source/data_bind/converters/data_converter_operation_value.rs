use super::data_converter_operation::ArithmeticOperation;
use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::data_values::data_value::DataValue,
    generated::data_bind::converters::{
        data_converter_operation_base::DataConverterOperationBaseCallbacks,
        data_converter_operation_value_base::{
            DataConverterOperationValueBase, DataConverterOperationValueBaseCallbacks,
        },
    },
};
pub struct DataConverterOperationValue {
    pub base: DataConverterOperationValueBase,
}

impl std::ops::Deref for DataConverterOperationValue {
    type Target = DataConverterOperationValueBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for DataConverterOperationValue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl crate::mechanical_port::source::generated::core_registry::DataConverterCapability
    for DataConverterOperationValue
{
    fn convert(
        &mut self,
        input: &dyn DataValue,
        _data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(Self::convert(self, input));
    }

    fn reverse_convert(
        &mut self,
        input: &dyn DataValue,
        _data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        let value = Self::reverse_convert(self, input);
        output(value.as_ref());
    }

    fn output_type(
        &self,
    ) -> crate::mechanical_port::source::data_bind::data_values::data_type::DataType {
        self.base.base.output_type()
    }

    crate::data_converter_capability_lifecycle!(base.base.base.base);
}

impl Default for DataConverterOperationValue {
    fn default() -> Self {
        Self {
            base: DataConverterOperationValueBase::default(),
        }
    }
}
impl DataConverterOperationValue {
    pub fn new(operation: ArithmeticOperation, operation_value: f32) -> Self {
        let mut converter = Self::default();
        converter.base.base.base.set_operation_type(
            operation as u32,
            &mut DataConverterOperationValueInitializationCallbacks,
        );
        converter.base.set_operation_value(
            operation_value,
            &mut DataConverterOperationValueInitializationCallbacks,
        );
        converter
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        self.base
            .base
            .convert_value(input, self.base.operation_value())
    }
    pub fn reverse_convert(&self, input: &dyn DataValue) -> Box<dyn DataValue> {
        Box::new(
            self.base
                .base
                .reverse_convert_value(input, self.base.operation_value()),
        )
    }
    pub fn operation_value_changed(&mut self) {
        self.base.base.mark_converter_dirty()
    }
}

impl DataConverterOperationValueBaseCallbacks for DataConverterOperationValue {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn operation_value_changed(&mut self) {
        Self::operation_value_changed(self);
    }
}

struct DataConverterOperationValueInitializationCallbacks;

impl DataConverterOperationValueBaseCallbacks
    for DataConverterOperationValueInitializationCallbacks
{
    fn notify_property_changed(&mut self, _property_key: u16) {}
}

impl DataConverterOperationBaseCallbacks for DataConverterOperationValueInitializationCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}
