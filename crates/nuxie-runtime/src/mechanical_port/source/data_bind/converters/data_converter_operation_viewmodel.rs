use super::data_converter_operation::ArithmeticOperation;
use crate::mechanical_port::source::{
    data_bind::data_values::data_value::DataValue,
    generated::data_bind::converters::{
        data_converter_operation_base::DataConverterOperationBaseCallbacks,
        data_converter_operation_viewmodel_base::{
            DataConverterOperationViewModelBase, DataConverterOperationViewModelBaseCallbacks,
        },
    },
};
pub trait NumberSource {
    fn property_value(&self) -> f32;
    fn add_dependent(&self, data_bind: *mut ());
}
pub trait OperationDataContext {
    fn number_property(&self, path: &[u32]) -> Option<*mut dyn NumberSource>;
}
pub struct DataConverterOperationViewModel {
    pub base: DataConverterOperationViewModelBase,
    source: Option<*mut dyn NumberSource>,
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
            .map_or(0.0, |source| unsafe { (&*source).property_value() })
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
    pub fn bind_from_context(&mut self, context: &dyn OperationDataContext, data_bind: *mut ()) {
        self.source = context.number_property(&self.source_path_ids);
        if let Some(source) = self.source {
            unsafe {
                (&*source).add_dependent(data_bind);
            }
        }
    }
}

impl DataConverterOperationViewModelBaseCallbacks for DataConverterOperationViewModel {
    fn decode_source_path_ids(&mut self, value: &[u8]) {
        Self::decode_source_path_ids(self, value);
    }

    fn copy_source_path_ids(&mut self, _object: &DataConverterOperationViewModelBase) {}
}

struct DataConverterOperationViewModelInitializationCallbacks;

impl DataConverterOperationBaseCallbacks
    for DataConverterOperationViewModelInitializationCallbacks
{
    fn notify_property_changed(&mut self, _property_key: u16) {}
}
