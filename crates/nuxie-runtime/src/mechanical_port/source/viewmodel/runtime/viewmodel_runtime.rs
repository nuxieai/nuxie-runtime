use std::rc::Rc;

use crate::mechanical_port::source::{core::CoreHandle, file::RuntimeFileWeakHandle};

use super::viewmodel_instance_runtime::{
    PropertyData, RuntimeViewModelInstanceHandle, ViewModelInstanceRuntime,
};
use super::viewmodel_instance_value_runtime::DataType;

#[derive(Clone)]
pub struct RuntimeViewModelHandle(Rc<RuntimeViewModel>);

struct RuntimeViewModel {
    view_model: CoreHandle,
    file: RuntimeFileWeakHandle,
}

impl RuntimeViewModelHandle {
    pub fn new(view_model: CoreHandle, file: RuntimeFileWeakHandle) -> Option<Self> {
        view_model
            .with(|view_model| view_model.as_view_model().is_some())
            .filter(|is_model| *is_model)?;
        Some(Self(Rc::new(RuntimeViewModel { view_model, file })))
    }

    pub fn view_model_handle(&self) -> CoreHandle {
        self.0.view_model.clone()
    }

    pub fn file(&self) -> RuntimeFileWeakHandle {
        self.0.file.clone()
    }

    pub fn instance_count(&self) -> usize {
        self.0
            .view_model
            .with(|model| model.as_view_model().map(|model| model.instance_count()))
            .flatten()
            .unwrap_or_default()
    }

    pub fn property_count(&self) -> usize {
        self.0
            .view_model
            .with(|model| model.as_view_model().map(|model| model.properties().len()))
            .flatten()
            .unwrap_or_default()
    }

    pub fn name(&self) -> String {
        self.0
            .view_model
            .with(|model| {
                model
                    .as_view_model()
                    .map(|model| model.base.name().to_owned())
            })
            .flatten()
            .unwrap_or_default()
    }

    pub(super) fn property_data(property: CoreHandle) -> PropertyData {
        use crate::mechanical_port::source::generated::viewmodel::{
            viewmodel_property_artboard_base::ViewModelPropertyArtboardBase,
            viewmodel_property_asset_blob_base::ViewModelPropertyAssetBlobBase,
            viewmodel_property_asset_font_base::ViewModelPropertyAssetFontBase,
            viewmodel_property_asset_image_base::ViewModelPropertyAssetImageBase,
            viewmodel_property_boolean_base::ViewModelPropertyBooleanBase,
            viewmodel_property_color_base::ViewModelPropertyColorBase,
            viewmodel_property_enum_base::ViewModelPropertyEnumBase,
            viewmodel_property_list_base::ViewModelPropertyListBase,
            viewmodel_property_number_base::ViewModelPropertyNumberBase,
            viewmodel_property_string_base::ViewModelPropertyStringBase,
            viewmodel_property_symbol_list_index_base::ViewModelPropertySymbolListIndexBase,
            viewmodel_property_trigger_base::ViewModelPropertyTriggerBase,
            viewmodel_property_viewmodel_base::ViewModelPropertyViewModelBase,
        };
        let data_type = if property.is_type_of(ViewModelPropertyStringBase::TYPE_KEY) {
            DataType::String
        } else if property.is_type_of(ViewModelPropertyNumberBase::TYPE_KEY) {
            DataType::Number
        } else if property.is_type_of(ViewModelPropertyBooleanBase::TYPE_KEY) {
            DataType::Boolean
        } else if property.is_type_of(ViewModelPropertyColorBase::TYPE_KEY) {
            DataType::Color
        } else if property.is_type_of(ViewModelPropertyListBase::TYPE_KEY) {
            DataType::List
        } else if property.is_type_of(ViewModelPropertyEnumBase::TYPE_KEY) {
            DataType::Enum
        } else if property.is_type_of(ViewModelPropertyTriggerBase::TYPE_KEY) {
            DataType::Trigger
        } else if property.is_type_of(ViewModelPropertyViewModelBase::TYPE_KEY) {
            DataType::ViewModel
        } else if property.is_type_of(ViewModelPropertySymbolListIndexBase::TYPE_KEY) {
            DataType::SymbolListIndex
        } else if property.is_type_of(ViewModelPropertyAssetImageBase::TYPE_KEY) {
            DataType::AssetImage
        } else if property.is_type_of(ViewModelPropertyArtboardBase::TYPE_KEY) {
            DataType::Artboard
        } else if property.is_type_of(ViewModelPropertyAssetFontBase::TYPE_KEY) {
            DataType::AssetFont
        } else if property.is_type_of(ViewModelPropertyAssetBlobBase::TYPE_KEY) {
            DataType::AssetBlob
        } else {
            DataType::None
        };
        let (name, enum_name) = property
            .with(|property| {
                let base = property.as_view_model_property()?;
                let enum_name = property
                    .as_view_model_property_enum()
                    .and_then(|property| property.data_enum())
                    .and_then(|data| {
                        data.with(|data| {
                            data.data_enum_name()
                                .or_else(|| data.as_data_enum().map(|data| data.enum_name()))
                                .map(str::to_owned)
                        })
                        .flatten()
                    })
                    .unwrap_or_default();
                Some((base.base.name().to_owned(), enum_name))
            })
            .flatten()
            .unwrap_or_default();
        PropertyData {
            data_type,
            name,
            enum_name,
        }
    }

    pub fn properties(&self) -> Vec<PropertyData> {
        self.0
            .view_model
            .with(|model| {
                model.as_view_model().map(|model| {
                    model
                        .properties()
                        .into_iter()
                        .map(Self::property_data)
                        .collect()
                })
            })
            .flatten()
            .unwrap_or_default()
    }

    pub fn instance_names(&self) -> Vec<String> {
        self.0
            .view_model
            .with(|model| {
                model.as_view_model().map(|model| {
                    model
                        .instances()
                        .into_iter()
                        .filter_map(|instance| {
                            instance
                                .with(|instance| {
                                    instance
                                        .as_view_model_instance()
                                        .map(|instance| instance.base.name().to_owned())
                                })
                                .flatten()
                        })
                        .collect()
                })
            })
            .flatten()
            .unwrap_or_default()
    }

    fn runtime(&self, instance: CoreHandle) -> RuntimeViewModelInstanceHandle {
        ViewModelInstanceRuntime::new(instance).into_handle()
    }

    pub fn create_instance_from_index(
        &self,
        index: usize,
    ) -> Option<RuntimeViewModelInstanceHandle> {
        let source = self
            .0
            .view_model
            .with(|model| model.as_view_model()?.instance_at(index))??;
        let instance = crate::mechanical_port::source::viewmodel::viewmodel_instance::ViewModelInstance::clone_instance(&source)?;
        self.0
            .file
            .with_file(|file| file.complete_view_model_instance(&instance))?;
        Some(self.runtime(instance))
    }

    pub fn create_instance_from_name(&self, name: &str) -> Option<RuntimeViewModelInstanceHandle> {
        let instance = crate::mechanical_port::source::viewmodel::viewmodel::ViewModel::create_from_instance_handle(&self.0.view_model, name)?;
        Some(self.runtime(instance))
    }

    pub fn create_default_instance(&self) -> RuntimeViewModelInstanceHandle {
        let instance = self
            .0
            .file
            .with_file(|file| file.create_default_view_model_instance(self.0.view_model.clone()))
            .flatten();
        instance
            .map(|instance| self.runtime(instance))
            .unwrap_or_else(|| self.create_instance())
    }

    pub fn create_instance(&self) -> RuntimeViewModelInstanceHandle {
        let instance = crate::mechanical_port::source::viewmodel::viewmodel::ViewModel::create_instance_handle(&self.0.view_model)
            .expect("a live RuntimeViewModelHandle retains its File and can create an instance");
        self.runtime(instance)
    }
}
