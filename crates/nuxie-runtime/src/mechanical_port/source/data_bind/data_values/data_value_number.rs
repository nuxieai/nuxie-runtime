use super::{data_type::DataType, data_value::DataValue};
use core::any::Any;
#[derive(Clone, Debug, Default)]
pub struct DataValueNumber {
    value: f32,
}
impl DataValueNumber {
    pub const TYPE_KEY: DataType = DataType::Number;
    pub const DEFAULT_VALUE: f32 = 0.0;
    pub fn new(value: f32) -> Self {
        Self { value }
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    pub fn set_value(&mut self, value: f32) {
        self.value = value
    }
}
impl DataValue for DataValueNumber {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, data_type: DataType) -> bool {
        data_type == DataType::Number
    }
    fn compare(&self, comparand: Option<&dyn DataValue>) -> bool {
        comparand
            .and_then(|v| v.as_any().downcast_ref::<Self>())
            .is_some_and(|v| v.value == self.value)
    }
    fn interpolate(
        &self,
        to: Option<&dyn DataValue>,
        destination: Option<&mut dyn DataValue>,
        mix: f32,
    ) {
        if let (Some(to), Some(destination)) = (
            to.and_then(|v| v.as_any().downcast_ref::<Self>()),
            destination.and_then(|v| v.as_any_mut().downcast_mut::<Self>()),
        ) {
            let inverse = 1.0 - mix;
            destination.value = to.value * mix + self.value * inverse;
        }
    }
    fn copy_value(&self, destination: Option<&mut dyn DataValue>) {
        if let Some(destination) = destination.and_then(|v| v.as_any_mut().downcast_mut::<Self>()) {
            destination.value = self.value;
        }
    }
}
