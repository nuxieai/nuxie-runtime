use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::generated::viewmodel::viewmodel_property_enum_system_base::ViewModelPropertyEnumSystemBase;

use super::data_enum::DataEnum;

#[derive(Default)]
pub struct ViewModelPropertyEnumSystem {
    pub base: ViewModelPropertyEnumSystemBase,
}

thread_local! {
    static SYSTEM_DATA_ENUM: Rc<RefCell<DataEnum>> = Rc::new(RefCell::new(DataEnum::default()));
}

impl ViewModelPropertyEnumSystem {
    pub fn data_enum(&self) -> Rc<RefCell<DataEnum>> {
        SYSTEM_DATA_ENUM.with(Rc::clone)
    }
}
