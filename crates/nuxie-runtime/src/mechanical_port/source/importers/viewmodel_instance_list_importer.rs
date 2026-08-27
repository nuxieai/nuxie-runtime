use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    refcnt::RiveRc,
    status_code::StatusCode,
    viewmodel::{
        viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
    },
};

use super::import_stack::ImportStackObject;

pub struct ViewModelInstanceListImporter {
    list: NonNull<ViewModelInstanceList>,
}

impl ViewModelInstanceListImporter {
    pub fn new(list: NonNull<ViewModelInstanceList>) -> Self {
        Self { list }
    }
    pub fn add_item(&mut self, item: RiveRc<ViewModelInstanceListItem>) {
        unsafe { self.list.as_mut().internal_add_item(item) };
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
