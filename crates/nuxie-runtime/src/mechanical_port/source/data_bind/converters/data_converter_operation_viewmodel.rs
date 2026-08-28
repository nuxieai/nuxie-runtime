use super::data_converter_operation::ArithmeticOperation;
use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::{data_context::RuntimeDataContextHandle, data_values::data_value::DataValue},
    generated::data_bind::converters::{
        data_converter_operation_base::DataConverterOperationBaseCallbacks,
        data_converter_operation_viewmodel_base::{
            DataConverterOperationViewModelBase, DataConverterOperationViewModelBaseCallbacks,
        },
    },
    viewmodel::viewmodel_instance_number::ViewModelInstanceNumber,
    viewmodel::viewmodel_instance_value::ValueDependentHandle,
};
pub struct DataConverterOperationViewModel {
    pub base: DataConverterOperationViewModelBase,
    source: Option<CoreHandle>,
    source_path_ids: Vec<u32>,
}
impl Default for DataConverterOperationViewModel {
    fn default() -> Self {
        Self {
            base: DataConverterOperationViewModelBase::default(),
            source: None,
            source_path_ids: Vec::new(),
        }
    }
}
impl DataConverterOperationViewModel {
    pub fn new(operation: ArithmeticOperation) -> Self {
        let mut converter = Self::default();
        converter.base.base.base.set_operation_type(
            operation as u32,
            &mut DataConverterOperationViewModelInitializationCallbacks,
        );
        converter
    }
    fn resolve_value(&self) -> f32 {
        self.source
            .as_ref()
            .and_then(|source| {
                source.with_downcast::<ViewModelInstanceNumber, _>(|source| source.property_value())
            })
            .unwrap_or(0.0)
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        self.base.base.convert_value(input, self.resolve_value())
    }
    pub fn reverse_convert(&self, input: &dyn DataValue) -> Box<dyn DataValue> {
        Box::new(
            self.base
                .base
                .reverse_convert_value(input, self.resolve_value()),
        )
    }
    pub fn decode_source_path_ids(&mut self, bytes: &[u8]) {
        let mut index = 0;
        while index < bytes.len() {
            let mut value = 0u32;
            let mut shift = 0;
            loop {
                let byte = bytes[index];
                index += 1;
                value |= ((byte & 0x7f) as u32) << shift;
                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }
            self.source_path_ids.push(value);
        }
    }
    pub fn copy_source_path_ids(&mut self, other: &Self) {
        self.source_path_ids = other.source_path_ids.clone()
    }
    pub fn source_path_ids(&self) -> &[u32] {
        &self.source_path_ids
    }
    pub fn bind_from_context(&mut self, context: RuntimeDataContextHandle, data_bind: CoreHandle) {
        self.base
            .base
            .base
            .bind_from_context(context.clone(), data_bind.clone());
        self.source = context
            .with_context(|context| context.get_view_model_property(&self.source_path_ids))
            .filter(|source| {
                source
                    .with_downcast::<ViewModelInstanceNumber, _>(|_| true)
                    .unwrap_or(false)
            });
        if let Some(source) = self.source.as_ref() {
            source.with_mut(|source| {
                if let Some(source) = source.as_view_model_instance_value_mut() {
                    source.add_dependent(ValueDependentHandle::core(data_bind));
                }
            });
        }
    }
}

impl DataConverterOperationViewModelBaseCallbacks for DataConverterOperationViewModel {
    fn decode_source_path_ids(&mut self, value: &[u8]) {
        Self::decode_source_path_ids(self, value);
    }

    fn copy_source_path_ids(&mut self, _object: &DataConverterOperationViewModelBase) {}
}

impl crate::mechanical_port::source::generated::core_registry::DataConverterCapability
    for DataConverterOperationViewModel
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

    fn bind_from_context(&mut self, context: RuntimeDataContextHandle, data_bind: CoreHandle) {
        Self::bind_from_context(self, context, data_bind);
    }

    fn unbind(&mut self) {
        self.base.base.base.base.unbind();
    }

    fn update(&mut self) {
        self.base.base.base.base.update();
    }

    fn reset(&mut self) {
        self.base.base.base.base.reset();
    }

    fn advance(&mut self, elapsed: f32) -> bool {
        self.base.base.base.base.advance(elapsed)
    }
}

struct DataConverterOperationViewModelInitializationCallbacks;

impl DataConverterOperationBaseCallbacks
    for DataConverterOperationViewModelInitializationCallbacks
{
    fn notify_property_changed(&mut self, _property_key: u16) {}
}
