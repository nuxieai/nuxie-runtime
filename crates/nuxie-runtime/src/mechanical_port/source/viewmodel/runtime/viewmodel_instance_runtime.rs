use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::mechanical_port::source::core::CoreHandle;

use super::{
    viewmodel_instance_artboard_runtime::ViewModelInstanceArtboardRuntime,
    viewmodel_instance_asset_blob_runtime::ViewModelInstanceAssetBlobRuntime,
    viewmodel_instance_asset_font_runtime::ViewModelInstanceAssetFontRuntime,
    viewmodel_instance_asset_image_runtime::ViewModelInstanceAssetImageRuntime,
    viewmodel_instance_boolean_runtime::ViewModelInstanceBooleanRuntime,
    viewmodel_instance_color_runtime::ViewModelInstanceColorRuntime,
    viewmodel_instance_enum_runtime::ViewModelInstanceEnumRuntime,
    viewmodel_instance_list_index_runtime::ViewModelInstanceListIndexRuntime,
    viewmodel_instance_list_runtime::ViewModelInstanceListRuntime,
    viewmodel_instance_number_runtime::ViewModelInstanceNumberRuntime,
    viewmodel_instance_string_runtime::ViewModelInstanceStringRuntime,
    viewmodel_instance_trigger_runtime::ViewModelInstanceTriggerRuntime,
    viewmodel_instance_value_runtime::{DataType, ViewModelInstanceValueRuntime},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyData {
    pub data_type: DataType,
    pub name: String,
    pub enum_name: String,
}

pub type RuntimeViewModelInstanceHandle = Rc<ViewModelInstanceRuntime>;

pub struct ViewModelInstanceRuntime {
    instance: CoreHandle,
    properties: RefCell<HashMap<String, ViewModelInstanceValueRuntime>>,
    view_model_instances: RefCell<HashMap<String, RuntimeViewModelInstanceHandle>>,
}

impl ViewModelInstanceRuntime {
    pub fn new(instance: CoreHandle) -> Self {
        debug_assert!(
            instance
                .with(|instance| instance.as_view_model_instance().is_some())
                .unwrap_or(false)
        );
        Self {
            instance,
            properties: RefCell::new(HashMap::new()),
            view_model_instances: RefCell::new(HashMap::new()),
        }
    }

    pub fn into_handle(self) -> RuntimeViewModelInstanceHandle {
        Rc::new(self)
    }

    pub fn name(&self) -> String {
        self.instance
            .with(|instance| {
                instance
                    .as_view_model_instance()
                    .map(|instance| instance.base.name().to_owned())
            })
            .flatten()
            .unwrap_or_default()
    }

    pub fn view_model_name(&self) -> String {
        self.instance
            .with(|instance| {
                let model = instance.as_view_model_instance()?.get_view_model()?;
                model.with(|model| {
                    model
                        .as_view_model()
                        .map(|model| model.base.name().to_owned())
                })
            })
            .flatten()
            .unwrap_or_default()
    }

    pub fn property_count(&self) -> usize {
        self.instance
            .with(|instance| {
                instance
                    .as_view_model_instance()
                    .map(|instance| instance.property_values().len())
            })
            .flatten()
            .unwrap_or_default()
    }

    pub fn instance(&self) -> CoreHandle {
        self.instance.clone()
    }

    fn data_type(value: &CoreHandle) -> DataType {
        value
            .with(|value| {
                if value.as_view_model_instance_string().is_some() {
                    DataType::String
                } else if value.as_view_model_instance_number().is_some() {
                    DataType::Number
                } else if value.as_view_model_instance_boolean().is_some() {
                    DataType::Boolean
                } else if value.as_view_model_instance_color().is_some() {
                    DataType::Color
                } else if value.as_view_model_instance_list().is_some() {
                    DataType::List
                } else if value.as_view_model_instance_enum().is_some() {
                    DataType::Enum
                } else if value.as_view_model_instance_trigger().is_some() {
                    DataType::Trigger
                } else if value.as_view_model_instance_view_model().is_some() {
                    DataType::ViewModel
                } else if value.as_view_model_instance_symbol_list_index().is_some() {
                    DataType::SymbolListIndex
                } else if value.as_view_model_instance_asset_image().is_some() {
                    DataType::AssetImage
                } else if value.as_view_model_instance_artboard().is_some() {
                    DataType::Artboard
                } else if value.as_view_model_instance_asset_font().is_some() {
                    DataType::AssetFont
                } else if value.as_view_model_instance_asset_blob().is_some() {
                    DataType::AssetBlob
                } else {
                    DataType::None
                }
            })
            .unwrap_or(DataType::None)
    }

    fn get_property_name_from_path<'a>(&self, path: &'a str) -> &'a str {
        path.rsplit_once('/').map_or(path, |(_, name)| name)
    }

    fn view_model_instance_at_path(&self, path: &str) -> Option<RuntimeViewModelInstanceHandle> {
        let (first, rest) = path
            .split_once('/')
            .map_or((path, ""), |(first, rest)| (first, rest));
        if first.is_empty() {
            return None;
        }
        let instance = self.instance_runtime(first)?;
        if rest.is_empty() {
            Some(instance)
        } else {
            instance.view_model_instance_at_path(rest)
        }
    }

    fn get_property_instance(
        &self,
        name: &str,
        expected: DataType,
    ) -> Option<ViewModelInstanceValueRuntime> {
        if let Some(runtime) = self.properties.borrow().get(name) {
            return (runtime.data_type() == expected).then(|| runtime.clone());
        }
        let value = self
            .instance
            .with(|instance| {
                instance
                    .as_view_model_instance()?
                    .property_value_named(name)
            })
            .flatten()?;
        (Self::data_type(&value) == expected).then_some(())?;
        let runtime = ViewModelInstanceValueRuntime::new(value, expected)?;
        self.properties
            .borrow_mut()
            .insert(name.to_owned(), runtime.clone());
        Some(runtime)
    }

    fn property_of_type(
        &self,
        path: &str,
        expected: DataType,
    ) -> Option<ViewModelInstanceValueRuntime> {
        let name = self.get_property_name_from_path(path);
        if let Some((parents, _)) = path.rsplit_once('/') {
            self.view_model_instance_at_path(parents)?
                .get_property_instance(name, expected)
        } else {
            self.get_property_instance(name, expected)
        }
    }

    pub fn property_number(&self, path: &str) -> Option<ViewModelInstanceNumberRuntime> {
        ViewModelInstanceNumberRuntime::new(self.property_of_type(path, DataType::Number)?)
    }
    pub fn property_string(&self, path: &str) -> Option<ViewModelInstanceStringRuntime> {
        ViewModelInstanceStringRuntime::new(self.property_of_type(path, DataType::String)?)
    }
    pub fn property_boolean(&self, path: &str) -> Option<ViewModelInstanceBooleanRuntime> {
        ViewModelInstanceBooleanRuntime::new(self.property_of_type(path, DataType::Boolean)?)
    }
    pub fn property_color(&self, path: &str) -> Option<ViewModelInstanceColorRuntime> {
        ViewModelInstanceColorRuntime::new(self.property_of_type(path, DataType::Color)?)
    }
    pub fn property_enum(&self, path: &str) -> Option<ViewModelInstanceEnumRuntime> {
        ViewModelInstanceEnumRuntime::new(self.property_of_type(path, DataType::Enum)?)
    }
    pub fn property_trigger(&self, path: &str) -> Option<ViewModelInstanceTriggerRuntime> {
        ViewModelInstanceTriggerRuntime::new(self.property_of_type(path, DataType::Trigger)?)
    }
    pub fn property_list(&self, path: &str) -> Option<ViewModelInstanceListRuntime> {
        ViewModelInstanceListRuntime::new(self.property_of_type(path, DataType::List)?)
    }
    pub fn property_list_index(&self, path: &str) -> Option<ViewModelInstanceListIndexRuntime> {
        ViewModelInstanceListIndexRuntime::new(
            self.property_of_type(path, DataType::SymbolListIndex)?,
        )
    }
    pub fn property_image(&self, path: &str) -> Option<ViewModelInstanceAssetImageRuntime> {
        ViewModelInstanceAssetImageRuntime::new(self.property_of_type(path, DataType::AssetImage)?)
    }
    pub fn property_font(&self, path: &str) -> Option<ViewModelInstanceAssetFontRuntime> {
        ViewModelInstanceAssetFontRuntime::new(self.property_of_type(path, DataType::AssetFont)?)
    }
    pub fn property_blob(&self, path: &str) -> Option<ViewModelInstanceAssetBlobRuntime> {
        ViewModelInstanceAssetBlobRuntime::new(self.property_of_type(path, DataType::AssetBlob)?)
    }
    pub fn property_artboard(&self, path: &str) -> Option<ViewModelInstanceArtboardRuntime> {
        ViewModelInstanceArtboardRuntime::new(self.property_of_type(path, DataType::Artboard)?)
    }

    fn instance_runtime(&self, name: &str) -> Option<RuntimeViewModelInstanceHandle> {
        if let Some(runtime) = self.view_model_instances.borrow().get(name) {
            return Some(runtime.clone());
        }
        let property = self
            .instance
            .with(|instance| {
                instance
                    .as_view_model_instance()?
                    .property_value_named(name)
            })
            .flatten()?;
        let instance = property
            .with(|property| {
                property
                    .as_view_model_instance_view_model()?
                    .reference_view_model_instance()
            })
            .flatten()?;
        let runtime = Rc::new(Self::new(instance));
        self.view_model_instances
            .borrow_mut()
            .insert(name.to_owned(), runtime.clone());
        Some(runtime)
    }

    pub fn property_view_model(&self, path: &str) -> Option<RuntimeViewModelInstanceHandle> {
        let name = self.get_property_name_from_path(path);
        if let Some((parents, _)) = path.rsplit_once('/') {
            self.view_model_instance_at_path(parents)?
                .instance_runtime(name)
        } else {
            self.instance_runtime(name)
        }
    }

    pub fn property(&self, path: &str) -> Option<ViewModelInstanceValueRuntime> {
        let name = self.get_property_name_from_path(path);
        if let Some((parents, _)) = path.rsplit_once('/') {
            let owner = self.view_model_instance_at_path(parents)?;
            let value = owner
                .instance
                .with(|instance| {
                    instance
                        .as_view_model_instance()?
                        .property_value_named(name)
                })
                .flatten()?;
            let data_type = Self::data_type(&value);
            owner.get_property_instance(name, data_type)
        } else {
            let value = self
                .instance
                .with(|instance| {
                    instance
                        .as_view_model_instance()?
                        .property_value_named(name)
                })
                .flatten()?;
            let data_type = Self::data_type(&value);
            self.get_property_instance(name, data_type)
        }
    }

    pub fn replace_view_model(&self, path: &str, value: RuntimeViewModelInstanceHandle) -> bool {
        let name = self.get_property_name_from_path(path);
        let owner = if let Some((parents, _)) = path.rsplit_once('/') {
            self.view_model_instance_at_path(parents)
        } else {
            None
        };
        let target = owner.as_deref().unwrap_or(self);
        let replaced = target
            .instance
            .with_mut(|instance| {
                instance
                    .as_view_model_instance_mut()
                    .is_some_and(|instance| {
                        instance.replace_view_model_by_name(name, value.instance())
                    })
            })
            .unwrap_or(false);
        if replaced {
            target
                .view_model_instances
                .borrow_mut()
                .insert(name.to_owned(), value);
        }
        replaced
    }

    pub fn properties(&self) -> Vec<PropertyData> {
        let values = self
            .instance
            .with(|instance| {
                instance
                    .as_view_model_instance()
                    .map(|instance| instance.property_values().to_vec())
            })
            .flatten()
            .unwrap_or_default();
        values
            .into_iter()
            .map(|value| {
                let data_type = Self::data_type(&value);
                let name = value
                    .with(|value| {
                        value
                            .as_view_model_instance_value()
                            .map(|value| value.name())
                    })
                    .flatten()
                    .unwrap_or_default();
                let enum_name = if data_type == DataType::Enum {
                    self.get_property_instance(&name, data_type)
                        .and_then(ViewModelInstanceEnumRuntime::new)
                        .map(|value| value.enum_type())
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                PropertyData {
                    data_type,
                    name,
                    enum_name,
                }
            })
            .collect()
    }
}
