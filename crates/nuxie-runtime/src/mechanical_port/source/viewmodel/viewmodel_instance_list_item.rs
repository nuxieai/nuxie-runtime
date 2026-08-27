use std::ptr::NonNull;

use crate::mechanical_port::source::{
    artboard::Artboard,
    generated::viewmodel::viewmodel_instance_list_item_base::ViewModelInstanceListItemBase,
    importers::{
        import_stack::ImportStack, viewmodel_instance_list_importer::ViewModelInstanceListImporter,
    },
    refcnt::RiveRc,
    status_code::StatusCode,
};

use super::{
    viewmodel_instance::ViewModelInstance, viewmodel_instance_list::ViewModelInstanceList,
};

#[derive(Default)]
pub struct ViewModelInstanceListItem {
    pub base: ViewModelInstanceListItemBase,
    view_model_instance: Option<RiveRc<ViewModelInstance>>,
    // The upstream field has no initializer; Option preserves its null/assigned states safely.
    artboard: Option<NonNull<Artboard>>,
}

impl ViewModelInstanceListItem {
    pub fn set_view_model_instance(&mut self, value: Option<RiveRc<ViewModelInstance>>) {
        self.view_model_instance = value;
    }

    pub fn view_model_instance(&self) -> Option<RiveRc<ViewModelInstance>> {
        self.view_model_instance.clone()
    }

    pub fn set_artboard(&mut self, value: Option<NonNull<Artboard>>) {
        self.artboard = value;
    }

    pub fn artboard(&self) -> Option<NonNull<Artboard>> {
        self.artboard
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest::<ViewModelInstanceListImporter>(
            crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_list_base::ViewModelInstanceListBase::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        self.base.ref_();
        importer.add_item(unsafe { RiveRc::from_raw(self) });
        self.base.import(import_stack)
    }
}
