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
    pub fn assign_list_index(&self, index: u32) {
        use crate::mechanical_port::source::{
            generated::{
                core_registry::CoreRegistry,
                viewmodel::viewmodel_instance_symbol_list_index_base::ViewModelInstanceSymbolListIndexBase,
            },
            viewmodel::{symbol_type::SymbolType, viewmodel_instance::ViewModelInstance},
        };
        if let Some(symbol) = self.view_model_instance.as_ref().and_then(|instance| {
            instance
                .with_downcast::<ViewModelInstance, _>(|instance| {
                    instance.property_value_for_symbol(SymbolType::ItemIndex)
                })
                .flatten()
        }) {
            CoreRegistry::set_uint_handle(
                &symbol,
                ViewModelInstanceSymbolListIndexBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
                index,
            );
        }
    }
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
        let Some(_) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_item(self);
        self.base.import(import_stack)
    }
}
