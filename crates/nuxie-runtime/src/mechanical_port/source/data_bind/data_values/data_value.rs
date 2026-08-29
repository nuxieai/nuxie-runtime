use super::data_type::DataType;
use core::any::Any;

/// The instantiable, untyped C++ DataValue base (not a typed None value).
#[derive(Default)]
pub struct EmptyDataValue;

/// Retain a value across a Rust owner borrow boundary without changing its
/// concrete type or the identity of referenced assets/instances.
pub fn clone_data_value(value: &dyn DataValue) -> Box<dyn DataValue> {
    macro_rules! concrete {
        ($module:ident, $ty:ident) => {
            if let Some(value) = value.as_any().downcast_ref::<super::$module::$ty>() {
                return Box::new(value.clone());
            }
        };
    }
    concrete!(data_value_boolean, DataValueBoolean);
    concrete!(data_value_color, DataValueColor);
    concrete!(data_value_number, DataValueNumber);
    concrete!(data_value_string, DataValueString);
    concrete!(data_value_integer, DataValueInteger);
    concrete!(data_value_enum, DataValueEnum);
    concrete!(data_value_list, DataValueList);
    concrete!(data_value_symbol_list_index, DataValueSymbolListIndex);
    concrete!(data_value_trigger, DataValueTrigger);
    concrete!(data_value_artboard, DataValueArtboard);
    concrete!(data_value_viewmodel, DataValueViewModel);
    concrete!(data_value_asset_image, DataValueAssetImage);
    concrete!(data_value_asset_font, DataValueAssetFont);
    concrete!(data_value_asset_blob, DataValueAssetBlob);
    assert!(
        value.as_any().is::<EmptyDataValue>(),
        "all translated DataValue concrete owners must preserve their type"
    );
    Box::new(EmptyDataValue)
}

impl DataValue for EmptyDataValue {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

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
