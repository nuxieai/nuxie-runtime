use crate::mechanical_port::source::{
    generated::viewmodel::viewmodel_property_enum_base::ViewModelPropertyEnumBase, refcnt::RiveRc,
};

use super::data_enum::DataEnum;

pub struct ViewModelPropertyEnum {
    pub base: ViewModelPropertyEnumBase,
    data_enum: Option<RiveRc<DataEnum>>,
}

impl ViewModelPropertyEnum {
    pub fn set_data_enum(&mut self, value: *mut DataEnum) {
        unsafe { (*value).base.ref_() };
        self.data_enum = Some(unsafe { RiveRc::from_raw(value) });
    }

    pub fn data_enum(&self) -> Option<&DataEnum> {
        self.data_enum.as_deref()
    }

    pub fn value_named(&self, name: &str) -> String {
        self.data_enum()
            .map_or_else(String::new, |data| data.value_by_name(name))
    }

    pub fn value_at(&self, index: u32) -> String {
        self.data_enum()
            .map_or_else(String::new, |data| data.value_by_index(index))
    }

    pub fn set_value_named(&mut self, name: &str, value: String) -> bool {
        self.data_enum
            .as_deref_mut()
            .is_some_and(|data| data.set_value_by_name(name, value))
    }

    pub fn set_value_at(&mut self, index: u32, value: String) -> bool {
        self.data_enum
            .as_deref_mut()
            .is_some_and(|data| data.set_value_by_index(index, value))
    }

    pub fn value_index_named(&self, name: &str) -> i32 {
        self.data_enum()
            .map_or(-1, |data| data.value_index_by_name(name))
    }

    pub fn value_index_at(&self, index: u32) -> i32 {
        self.data_enum().map_or(-1, |data| data.value_index(index))
    }
}
