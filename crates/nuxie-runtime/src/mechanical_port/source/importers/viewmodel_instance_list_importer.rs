use std::any::Any;

use crate::mechanical_port::source::{
    core::CoreHandle, status_code::StatusCode,
    viewmodel::viewmodel_instance_list::ViewModelInstanceList,
};

use super::import_stack::ImportStackObject;

pub struct ViewModelInstanceListImporter {
    list: CoreHandle,
}

impl ViewModelInstanceListImporter {
    pub fn new(list: CoreHandle) -> Self {
        Self { list }
    }
    pub fn add_item(&mut self, item: CoreHandle) {
        self.list
            .with_downcast_mut::<ViewModelInstanceList, _>(|list| list.internal_add_item(item))
            .expect("ViewModelInstanceListImporter retains a ViewModelInstanceList");
    }
}

impl ImportStackObject for ViewModelInstanceListImporter {
    fn resolve(&mut self) -> StatusCode {
        StatusCode::Ok
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
