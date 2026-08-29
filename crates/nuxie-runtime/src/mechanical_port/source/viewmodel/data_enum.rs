use crate::mechanical_port::source::{
    core::CoreHandle, generated::viewmodel::data_enum_base::DataEnumBase,
};

use super::data_enum_value::DataEnumValue;

pub struct DataEnum {
    pub base: DataEnumBase,
    values: Vec<CoreHandle>,
    name: String,
}

impl std::ops::Deref for DataEnum {
    type Target = DataEnumBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for DataEnum {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Default for DataEnum {
    fn default() -> Self {
        Self {
            base: DataEnumBase::default(),
            values: Vec::new(),
            name: String::new(),
        }
    }
}

impl DataEnum {
    pub fn add_value(&mut self, value: CoreHandle) {
        self.values.push(value);
    }

    pub fn values(&self) -> &[CoreHandle] {
        &self.values
    }

    pub fn value_by_name(&self, key: &str) -> String {
        for value in &self.values {
            let result = value.with_downcast::<DataEnumValue, _>(|value| {
                (value.base.key() == key).then(|| display_value(value))
            });
            if let Some(Some(result)) = result {
                return result;
            }
        }
        String::new()
    }

    pub fn value_by_index(&self, index: u32) -> String {
        self.values
            .get(index as usize)
            .and_then(|value| value.with_downcast::<DataEnumValue, _>(display_value))
            .unwrap_or_default()
    }

    pub fn key_at(&self, index: u32) -> String {
        self.values
            .get(index as usize)
            .and_then(|value| {
                value.with_downcast::<DataEnumValue, _>(|value| value.base.key().to_owned())
            })
            .unwrap_or_default()
    }

    pub fn set_value_by_name(&mut self, key: &str, value: String) -> bool {
        for enum_value in &self.values {
            let changed = enum_value
                .with_downcast_mut::<DataEnumValue, _>(|enum_value| {
                    if enum_value.base.key() == key {
                        enum_value.set_value(value.clone());
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false);
            if changed {
                return true;
            }
        }
        false
    }

    pub fn set_value_by_index(&mut self, index: u32, value: String) -> bool {
        self.values
            .get(index as usize)
            .and_then(|enum_value| {
                enum_value.with_downcast_mut::<DataEnumValue, _>(|enum_value| {
                    enum_value.set_value(value);
                })
            })
            .is_some()
    }

    pub fn value_index_by_name(&self, key: &str) -> i32 {
        self.values
            .iter()
            .position(|value| {
                value
                    .with_downcast::<DataEnumValue, _>(|value| value.base.key() == key)
                    .unwrap_or(false)
            })
            .map_or(-1, |index| index as i32)
    }

    pub fn value_index(&self, index: u32) -> i32 {
        if (index as usize) < self.values.len() {
            index as i32
        } else {
            -1
        }
    }

    pub fn enum_name(&self) -> &str {
        &self.name
    }
}

fn display_value(value: &DataEnumValue) -> String {
    if value.base.value().is_empty() {
        value.base.key().to_owned()
    } else {
        value.base.value().to_owned()
    }
}

impl crate::mechanical_port::source::data_bind::data_values::data_value_enum::DataEnum
    for DataEnum
{
    fn value(&self, index: u32) -> String {
        self.value_by_index(index)
    }
}
