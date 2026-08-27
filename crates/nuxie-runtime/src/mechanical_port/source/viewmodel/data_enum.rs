use crate::mechanical_port::source::generated::viewmodel::data_enum_base::DataEnumBase;

use std::ptr::NonNull;

use super::data_enum_value::DataEnumValue;

pub struct DataEnum {
    pub base: DataEnumBase,
    values: Vec<NonNull<DataEnumValue>>,
    name: String,
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
    pub fn add_value(&mut self, value: NonNull<DataEnumValue>) {
        self.values.push(value);
    }
    pub fn values(&self) -> &[NonNull<DataEnumValue>] {
        &self.values
    }
    pub fn value_by_name(&self, key: &str) -> String {
        for value in &self.values {
            let value = unsafe { value.as_ref() };
            if value.base.key() == key {
                return if value.base.value().is_empty() {
                    value.base.key().to_owned()
                } else {
                    value.base.value().to_owned()
                };
            }
        }
        String::new()
    }
    pub fn value_by_index(&self, index: u32) -> String {
        let Some(value) = self.values.get(index as usize) else {
            return String::new();
        };
        let value = unsafe { value.as_ref() };
        if value.base.value().is_empty() {
            value.base.key().to_owned()
        } else {
            value.base.value().to_owned()
        }
    }
    pub fn set_value_by_name(&mut self, key: &str, value: String) -> bool {
        for enum_value in &mut self.values {
            let enum_value = unsafe { enum_value.as_mut() };
            if enum_value.base.key() == key {
                enum_value.base.set_value(value);
                return true;
            }
        }
        false
    }
    pub fn set_value_by_index(&mut self, index: u32, value: String) -> bool {
        let Some(enum_value) = self.values.get_mut(index as usize) else {
            return false;
        };
        unsafe { enum_value.as_mut() }.base.set_value(value);
        true
    }
    pub fn value_index_by_name(&self, key: &str) -> i32 {
        self.values
            .iter()
            .position(|value| unsafe { value.as_ref() }.base.key() == key)
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

impl Drop for DataEnum {
    fn drop(&mut self) {
        for value in self.values.drain(..) {
            unsafe { drop(Box::from_raw(value.as_ptr())) };
        }
    }
}
