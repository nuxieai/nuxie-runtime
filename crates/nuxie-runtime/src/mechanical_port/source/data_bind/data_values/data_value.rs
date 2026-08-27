use super::data_type::DataType;
use core::any::Any;
pub trait DataValue: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn is_type_of(&self, _data_type: DataType) -> bool {
        false
    }
    fn compare(&self, _comparand: Option<&dyn DataValue>) -> bool {
        false
    }
    fn interpolate(
        &self,
        _to: Option<&dyn DataValue>,
        _destination: Option<&mut dyn DataValue>,
        _mix: f32,
    ) {
    }
    fn copy_value(&self, _destination: Option<&mut dyn DataValue>) {}
}
