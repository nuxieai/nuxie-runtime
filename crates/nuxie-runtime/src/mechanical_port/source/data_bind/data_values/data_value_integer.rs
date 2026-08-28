use super::{data_type::DataType, data_value::DataValue};
use core::any::Any;

/// The scalar lane shared by every DataValueInteger-derived source owner.
pub fn integer_value(value: &dyn DataValue) -> Option<u32> {
    if !value.is_type_of(DataType::Integer) {
        return None;
    }
    use super::{
        data_value_artboard::DataValueArtboard, data_value_asset_blob::DataValueAssetBlob,
        data_value_asset_font::DataValueAssetFont, data_value_asset_image::DataValueAssetImage,
        data_value_enum::DataValueEnum, data_value_symbol_list_index::DataValueSymbolListIndex,
        data_value_trigger::DataValueTrigger,
    };
    macro_rules! lane { ($($ty:ty),* $(,)?) => { $(if let Some(value) = value.as_any().downcast_ref::<$ty>() { return Some(value.value()); })* }; }
    lane!(
        DataValueInteger,
        DataValueEnum,
        DataValueTrigger,
        DataValueSymbolListIndex,
        DataValueAssetImage,
        DataValueAssetFont,
        DataValueAssetBlob,
        DataValueArtboard
    );
    None
}
#[derive(Clone, Debug, Default)]
pub struct DataValueInteger {
    value: u32,
}
impl DataValueInteger {
    pub const TYPE_KEY: DataType = DataType::Integer;
    pub const DEFAULT_VALUE: u32 = 0;
    pub fn new(value: u32) -> Self {
        Self { value }
    }
    pub fn value(&self) -> u32 {
        self.value
    }
    pub fn set_value(&mut self, value: u32) {
        self.value = value
    }
}
impl DataValue for DataValueInteger {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, data_type: DataType) -> bool {
        data_type == DataType::Integer
    }
}
