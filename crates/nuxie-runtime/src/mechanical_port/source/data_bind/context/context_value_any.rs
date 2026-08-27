use super::{
    context_target_value::FieldType,
    context_value::{ContextApplyBinding, DataBindContextValue},
};
use crate::mechanical_port::source::data_bind::data_values::{
    data_value_boolean::DataValueBoolean, data_value_color::DataValueColor,
    data_value_number::DataValueNumber, data_value_string::DataValueString,
};
pub struct DataBindContextValueAny {
    base: DataBindContextValue,
}
impl DataBindContextValueAny {
    pub fn new(binding: &mut dyn ContextApplyBinding) -> Self {
        Self {
            base: DataBindContextValue::new(binding),
        }
    }
    pub fn apply(
        &mut self,
        property_key: u32,
        is_main_direction: bool,
        binding: &mut dyn ContextApplyBinding,
    ) {
        self.base.sync_source_value(binding);
        let value = binding.convert(self.base.data_value().unwrap(), is_main_direction);
        match binding.field_type() {
            FieldType::Double => {
                if let Some(value) = value.as_any().downcast_ref::<DataValueNumber>() {
                    binding.set_double(property_key, value.value())
                }
            }
            FieldType::Uint => {
                if let Some(value) = value.as_any().downcast_ref::<DataValueNumber>() {
                    if binding.target_is_solo() {
                        binding.solo_update_by_index(value.value().round() as usize)
                    } else {
                        binding.set_uint(
                            property_key,
                            if value.value() < 0.0 {
                                0
                            } else {
                                value.value().round() as u32
                            },
                        )
                    }
                } else if let Some(value) = value.as_any().downcast_ref::<DataValueString>() {
                    if binding.target_is_solo() {
                        binding.solo_update_by_name(value.value().to_owned())
                    }
                }
            }
            FieldType::String => {
                if let Some(value) = value.as_any().downcast_ref::<DataValueString>() {
                    binding.set_string(property_key, value.value().to_owned())
                }
            }
            FieldType::Bool => {
                if let Some(value) = value.as_any().downcast_ref::<DataValueBoolean>() {
                    binding.set_bool(property_key, value.value())
                }
            }
            FieldType::Color => {
                if let Some(value) = value.as_any().downcast_ref::<DataValueColor>() {
                    binding.set_color(property_key, value.value())
                }
            }
            FieldType::Other => {}
        }
    }
}
