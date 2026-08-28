use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::viewmodel::viewmodel_instance_list_item_base::ViewModelInstanceListItemBase,
    importers::{
        import_stack::ImportStack, viewmodel_instance_list_importer::ViewModelInstanceListImporter,
    },
    status_code::StatusCode,
};

#[derive(Default)]
pub struct ViewModelInstanceListItem {
    pub base: ViewModelInstanceListItemBase,
    view_model_instance: Option<CoreHandle>,
    artboard: Option<CoreHandle>,
}

impl ViewModelInstanceListItem {
    pub fn set_view_model_instance(&mut self, value: Option<CoreHandle>) {
        self.view_model_instance = value;
    }

    pub fn view_model_instance(&self) -> Option<CoreHandle> {
        self.view_model_instance.clone()
    }

    pub fn set_artboard(&mut self, value: Option<CoreHandle>) {
        self.artboard = value;
    }

    pub fn artboard(&self) -> Option<CoreHandle> {
        self.artboard.clone()
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest::<ViewModelInstanceListImporter>(
            crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_list_base::ViewModelInstanceListBase::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        let Some(item) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_item(item);
        self.base.import(import_stack)
    }
}
