use crate::mechanical_port::source::{
    core::CoreHandle, generated::viewmodel::viewmodel_property_enum_base::ViewModelPropertyEnumBase,
};

use super::data_enum::DataEnum;

#[derive(Default)]
pub struct ViewModelPropertyEnum {
    pub base: ViewModelPropertyEnumBase,
    data_enum: Option<CoreHandle>,
}

impl std::ops::Deref for ViewModelPropertyEnum {
    type Target = ViewModelPropertyEnumBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ViewModelPropertyEnum {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ViewModelPropertyEnum {
    pub fn set_data_enum(&mut self, value: CoreHandle) {
        self.data_enum = Some(value);
    }

    pub fn data_enum(&self) -> Option<CoreHandle> {
        self.data_enum.clone()
    }

    pub fn value_named(&self, name: &str) -> String {
        self.data_enum()
            .and_then(|data| data.with_downcast::<DataEnum, _>(|data| data.value_by_name(name)))
            .unwrap_or_default()
    }

    pub fn value_at(&self, index: u32) -> String {
        self.data_enum()
            .and_then(|data| data.with_downcast::<DataEnum, _>(|data| data.value_by_index(index)))
            .unwrap_or_default()
    }

    pub fn set_value_named(&mut self, name: &str, value: String) -> bool {
        self.data_enum
            .as_ref()
            .and_then(|data| {
                data.with_downcast_mut::<DataEnum, _>(|data| data.set_value_by_name(name, value))
            })
            .unwrap_or(false)
    }

    pub fn set_value_at(&mut self, index: u32, value: String) -> bool {
        self.data_enum
            .as_ref()
            .and_then(|data| {
                data.with_downcast_mut::<DataEnum, _>(|data| data.set_value_by_index(index, value))
            })
            .unwrap_or(false)
    }

    pub fn value_index_named(&self, name: &str) -> i32 {
        self.data_enum()
            .and_then(|data| {
                data.with_downcast::<DataEnum, _>(|data| data.value_index_by_name(name))
            })
            .unwrap_or(-1)
    }

    pub fn value_index_at(&self, index: u32) -> i32 {
        self.data_enum()
            .and_then(|data| data.with_downcast::<DataEnum, _>(|data| data.value_index(index)))
            .unwrap_or(-1)
    }
}
