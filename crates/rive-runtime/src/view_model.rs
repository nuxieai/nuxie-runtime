use std::collections::BTreeMap;

use crate::{
    RuntimeFile, RuntimeViewModelPointer,
    runtime_imported_view_model_artboard_property_path_for_name,
    runtime_imported_view_model_asset_property_path_for_name,
    runtime_imported_view_model_boolean_property_path_for_name,
    runtime_imported_view_model_boolean_property_path_for_name_path,
    runtime_imported_view_model_color_property_path_for_name,
    runtime_imported_view_model_color_property_path_for_name_path,
    runtime_imported_view_model_enum_property_path_for_name,
    runtime_imported_view_model_enum_property_path_for_name_path,
    runtime_imported_view_model_list_property_path_for_name,
    runtime_imported_view_model_number_property_path_for_name,
    runtime_imported_view_model_number_property_path_for_name_path,
    runtime_imported_view_model_string_property_path_for_name,
    runtime_imported_view_model_string_property_path_for_name_path,
    runtime_imported_view_model_symbol_list_index_property_path_for_name,
    runtime_imported_view_model_symbol_list_index_property_path_for_name_path,
    runtime_imported_view_model_trigger_property_path_for_name,
    runtime_imported_view_model_view_model_property_path_for_name,
    runtime_imported_view_model_view_model_property_path_for_name_path,
    runtime_view_model_reference_index_for_property_path,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelNumberSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelNumberSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelBooleanSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelBooleanSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelStringSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelStringSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelColorSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelColorSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelEnumSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelEnumSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelSymbolListIndexSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelSymbolListIndexSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelAssetSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelAssetSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelArtboardSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelArtboardSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelTriggerSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelTriggerSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelListSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelListSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelViewModelSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelViewModelSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelNumberSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelNumberSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelBooleanSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelBooleanSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelStringSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelStringSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelColorSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelColorSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelEnumSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelEnumSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelSymbolListIndexSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelSymbolListIndexSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelAssetSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelAssetSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelArtboardSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelArtboardSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelTriggerSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelTriggerSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelListSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelListSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelViewModelSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelViewModelSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeImportedViewModelInstanceContext {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) number_overrides: BTreeMap<Vec<u32>, f32>,
    pub(crate) boolean_overrides: BTreeMap<Vec<u32>, bool>,
    pub(crate) string_overrides: BTreeMap<Vec<u32>, Vec<u8>>,
    pub(crate) color_overrides: BTreeMap<Vec<u32>, u32>,
    pub(crate) enum_overrides: BTreeMap<Vec<u32>, u64>,
    pub(crate) symbol_list_index_overrides: BTreeMap<Vec<u32>, u64>,
    pub(crate) asset_overrides: BTreeMap<Vec<u32>, u64>,
    pub(crate) artboard_overrides: BTreeMap<Vec<u32>, u64>,
    pub(crate) trigger_overrides: BTreeMap<Vec<u32>, u64>,
    pub(crate) list_overrides: BTreeMap<Vec<u32>, usize>,
    pub(crate) view_model_overrides: BTreeMap<Vec<u32>, RuntimeViewModelPointer>,
}

impl RuntimeImportedViewModelInstanceContext {
    pub fn new(file: &RuntimeFile, view_model_index: usize, instance_index: usize) -> Option<Self> {
        let view_model = file.view_model(view_model_index)?;
        view_model.instances.into_iter().nth(instance_index)?;
        Some(Self {
            view_model_index,
            instance_index,
            number_overrides: BTreeMap::new(),
            boolean_overrides: BTreeMap::new(),
            string_overrides: BTreeMap::new(),
            color_overrides: BTreeMap::new(),
            enum_overrides: BTreeMap::new(),
            symbol_list_index_overrides: BTreeMap::new(),
            asset_overrides: BTreeMap::new(),
            artboard_overrides: BTreeMap::new(),
            trigger_overrides: BTreeMap::new(),
            list_overrides: BTreeMap::new(),
            view_model_overrides: BTreeMap::new(),
        })
    }

    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn number_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeImportedViewModelNumberSourceHandle> {
        let path = runtime_imported_view_model_number_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        )?;
        Some(RuntimeImportedViewModelNumberSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn number_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeImportedViewModelNumberSourceHandle> {
        let path = runtime_imported_view_model_number_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        )?;
        Some(RuntimeImportedViewModelNumberSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn set_number_by_source_handle(
        &mut self,
        file: &RuntimeFile,
        handle: &RuntimeImportedViewModelNumberSourceHandle,
        value: f32,
    ) -> bool {
        if handle.view_model_index != self.view_model_index
            || handle.instance_index != self.instance_index
        {
            return false;
        }
        self.set_number_by_resolved_property_path(file, handle.path.clone(), value)
    }

    pub fn set_number_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: f32,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_number_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        ) else {
            return false;
        };
        self.set_number_by_resolved_property_path(file, path, value)
    }

    pub fn set_number_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        value: f32,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_number_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        ) else {
            return false;
        };
        self.set_number_by_resolved_property_path(file, path, value)
    }

    fn set_number_by_resolved_property_path(
        &mut self,
        file: &RuntimeFile,
        path: Vec<u32>,
        value: f32,
    ) -> bool {
        let Some(view_model) = file.view_model(self.view_model_index) else {
            return false;
        };
        let Some(instance) = view_model.instances.into_iter().nth(self.instance_index) else {
            return false;
        };
        let current = self.number_overrides.get(&path).copied().or_else(|| {
            let source =
                file.data_context_view_model_property_for_instance(instance.object, &path)?;
            file.view_model_instance_number_value_for_object(source)
        });
        if current == Some(value) {
            return false;
        }

        self.number_overrides.insert(path, value);
        true
    }

    pub fn boolean_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeImportedViewModelBooleanSourceHandle> {
        let path = runtime_imported_view_model_boolean_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        )?;
        Some(RuntimeImportedViewModelBooleanSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn boolean_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeImportedViewModelBooleanSourceHandle> {
        let path = runtime_imported_view_model_boolean_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        )?;
        Some(RuntimeImportedViewModelBooleanSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn set_boolean_by_source_handle(
        &mut self,
        file: &RuntimeFile,
        handle: &RuntimeImportedViewModelBooleanSourceHandle,
        value: bool,
    ) -> bool {
        if handle.view_model_index != self.view_model_index
            || handle.instance_index != self.instance_index
        {
            return false;
        }
        self.set_boolean_by_resolved_property_path(file, handle.path.clone(), value)
    }

    pub fn set_boolean_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: bool,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_boolean_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        ) else {
            return false;
        };
        self.set_boolean_by_resolved_property_path(file, path, value)
    }

    pub fn set_boolean_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        value: bool,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_boolean_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        ) else {
            return false;
        };
        self.set_boolean_by_resolved_property_path(file, path, value)
    }

    fn set_boolean_by_resolved_property_path(
        &mut self,
        file: &RuntimeFile,
        path: Vec<u32>,
        value: bool,
    ) -> bool {
        let Some(view_model) = file.view_model(self.view_model_index) else {
            return false;
        };
        let Some(instance) = view_model.instances.into_iter().nth(self.instance_index) else {
            return false;
        };
        let current = self.boolean_overrides.get(&path).copied().or_else(|| {
            let source =
                file.data_context_view_model_property_for_instance(instance.object, &path)?;
            file.view_model_instance_boolean_value_for_object(source)
        });
        if current == Some(value) {
            return false;
        }

        self.boolean_overrides.insert(path, value);
        true
    }

    pub fn string_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeImportedViewModelStringSourceHandle> {
        let path = runtime_imported_view_model_string_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        )?;
        Some(RuntimeImportedViewModelStringSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn string_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeImportedViewModelStringSourceHandle> {
        let path = runtime_imported_view_model_string_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        )?;
        Some(RuntimeImportedViewModelStringSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn set_string_by_source_handle(
        &mut self,
        file: &RuntimeFile,
        handle: &RuntimeImportedViewModelStringSourceHandle,
        value: &[u8],
    ) -> bool {
        if handle.view_model_index != self.view_model_index
            || handle.instance_index != self.instance_index
        {
            return false;
        }
        self.set_string_by_resolved_property_path(file, handle.path.clone(), value)
    }

    pub fn set_string_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: &[u8],
    ) -> bool {
        let Some(path) = runtime_imported_view_model_string_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        ) else {
            return false;
        };
        self.set_string_by_resolved_property_path(file, path, value)
    }

    pub fn set_string_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        value: &[u8],
    ) -> bool {
        let Some(path) = runtime_imported_view_model_string_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        ) else {
            return false;
        };
        self.set_string_by_resolved_property_path(file, path, value)
    }

    fn set_string_by_resolved_property_path(
        &mut self,
        file: &RuntimeFile,
        path: Vec<u32>,
        value: &[u8],
    ) -> bool {
        let Some(view_model) = file.view_model(self.view_model_index) else {
            return false;
        };
        let Some(instance) = view_model.instances.into_iter().nth(self.instance_index) else {
            return false;
        };
        let current_matches = if let Some(current) = self.string_overrides.get(&path) {
            current.as_slice() == value
        } else {
            let Some(source) =
                file.data_context_view_model_property_for_instance(instance.object, &path)
            else {
                return false;
            };
            let Some(current) = file.view_model_instance_string_value_bytes_for_object(source)
            else {
                return false;
            };
            current == value
        };
        if current_matches {
            return false;
        }

        self.string_overrides.insert(path, value.to_vec());
        true
    }

    pub fn color_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeImportedViewModelColorSourceHandle> {
        let path = runtime_imported_view_model_color_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        )?;
        Some(RuntimeImportedViewModelColorSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn color_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeImportedViewModelColorSourceHandle> {
        let path = runtime_imported_view_model_color_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        )?;
        Some(RuntimeImportedViewModelColorSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn set_color_by_source_handle(
        &mut self,
        file: &RuntimeFile,
        handle: &RuntimeImportedViewModelColorSourceHandle,
        value: u32,
    ) -> bool {
        if handle.view_model_index != self.view_model_index
            || handle.instance_index != self.instance_index
        {
            return false;
        }
        self.set_color_by_resolved_property_path(file, handle.path.clone(), value)
    }

    pub fn set_color_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u32,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_color_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        ) else {
            return false;
        };
        self.set_color_by_resolved_property_path(file, path, value)
    }

    pub fn set_color_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        value: u32,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_color_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        ) else {
            return false;
        };
        self.set_color_by_resolved_property_path(file, path, value)
    }

    fn set_color_by_resolved_property_path(
        &mut self,
        file: &RuntimeFile,
        path: Vec<u32>,
        value: u32,
    ) -> bool {
        let Some(view_model) = file.view_model(self.view_model_index) else {
            return false;
        };
        let Some(instance) = view_model.instances.into_iter().nth(self.instance_index) else {
            return false;
        };
        let current = self.color_overrides.get(&path).copied().or_else(|| {
            let source =
                file.data_context_view_model_property_for_instance(instance.object, &path)?;
            file.view_model_instance_color_value_for_object(source)
        });
        if current == Some(value) {
            return false;
        }

        self.color_overrides.insert(path, value);
        true
    }

    pub fn enum_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeImportedViewModelEnumSourceHandle> {
        let path = runtime_imported_view_model_enum_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        )?;
        Some(RuntimeImportedViewModelEnumSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn enum_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeImportedViewModelEnumSourceHandle> {
        let path = runtime_imported_view_model_enum_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        )?;
        Some(RuntimeImportedViewModelEnumSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn set_enum_by_source_handle(
        &mut self,
        file: &RuntimeFile,
        handle: &RuntimeImportedViewModelEnumSourceHandle,
        value: u64,
    ) -> bool {
        if handle.view_model_index != self.view_model_index
            || handle.instance_index != self.instance_index
        {
            return false;
        }
        self.set_enum_by_resolved_property_path(file, handle.path.clone(), value)
    }

    pub fn set_enum_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_enum_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        ) else {
            return false;
        };
        self.set_enum_by_resolved_property_path(file, path, value)
    }

    pub fn set_enum_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        value: u64,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_enum_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        ) else {
            return false;
        };
        self.set_enum_by_resolved_property_path(file, path, value)
    }

    fn set_enum_by_resolved_property_path(
        &mut self,
        file: &RuntimeFile,
        path: Vec<u32>,
        value: u64,
    ) -> bool {
        let Some(view_model) = file.view_model(self.view_model_index) else {
            return false;
        };
        let Some(instance) = view_model.instances.into_iter().nth(self.instance_index) else {
            return false;
        };
        let current = self.enum_overrides.get(&path).copied().or_else(|| {
            let source =
                file.data_context_view_model_property_for_instance(instance.object, &path)?;
            (source.type_name == "ViewModelInstanceEnum")
                .then(|| source.uint_property("propertyValue"))
                .flatten()
        });
        if current == Some(value) {
            return false;
        }

        self.enum_overrides.insert(path, value);
        true
    }

    pub fn symbol_list_index_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeImportedViewModelSymbolListIndexSourceHandle> {
        let path = runtime_imported_view_model_symbol_list_index_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        )?;
        Some(RuntimeImportedViewModelSymbolListIndexSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn symbol_list_index_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeImportedViewModelSymbolListIndexSourceHandle> {
        let path = runtime_imported_view_model_symbol_list_index_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        )?;
        Some(RuntimeImportedViewModelSymbolListIndexSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn set_symbol_list_index_by_source_handle(
        &mut self,
        file: &RuntimeFile,
        handle: &RuntimeImportedViewModelSymbolListIndexSourceHandle,
        value: u64,
    ) -> bool {
        if handle.view_model_index != self.view_model_index
            || handle.instance_index != self.instance_index
        {
            return false;
        }
        self.set_symbol_list_index_by_resolved_property_path(file, handle.path.clone(), value)
    }

    pub fn set_symbol_list_index_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_symbol_list_index_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        ) else {
            return false;
        };
        self.set_symbol_list_index_by_resolved_property_path(file, path, value)
    }

    pub fn set_symbol_list_index_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        value: u64,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_symbol_list_index_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        ) else {
            return false;
        };
        self.set_symbol_list_index_by_resolved_property_path(file, path, value)
    }

    fn set_symbol_list_index_by_resolved_property_path(
        &mut self,
        file: &RuntimeFile,
        path: Vec<u32>,
        value: u64,
    ) -> bool {
        let Some(view_model) = file.view_model(self.view_model_index) else {
            return false;
        };
        let Some(instance) = view_model.instances.into_iter().nth(self.instance_index) else {
            return false;
        };
        let current = self
            .symbol_list_index_overrides
            .get(&path)
            .copied()
            .or_else(|| {
                let source =
                    file.data_context_view_model_property_for_instance(instance.object, &path)?;
                file.view_model_instance_symbol_list_index_value_for_object(source)
            });
        if current == Some(value) {
            return false;
        }

        self.symbol_list_index_overrides.insert(path, value);
        true
    }

    pub fn asset_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeImportedViewModelAssetSourceHandle> {
        let path = runtime_imported_view_model_asset_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        )?;
        Some(RuntimeImportedViewModelAssetSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn set_asset_by_source_handle(
        &mut self,
        file: &RuntimeFile,
        handle: &RuntimeImportedViewModelAssetSourceHandle,
        value: u64,
    ) -> bool {
        if handle.view_model_index != self.view_model_index
            || handle.instance_index != self.instance_index
        {
            return false;
        }
        self.set_asset_by_resolved_property_path(file, handle.path.clone(), value)
    }

    pub fn set_asset_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_asset_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        ) else {
            return false;
        };
        self.set_asset_by_resolved_property_path(file, path, value)
    }

    pub fn set_asset_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        value: u64,
    ) -> bool {
        if property_path.contains('/') {
            return false;
        }
        self.set_asset_by_property_name(file, property_path, value)
    }

    fn set_asset_by_resolved_property_path(
        &mut self,
        file: &RuntimeFile,
        path: Vec<u32>,
        value: u64,
    ) -> bool {
        let Some(view_model) = file.view_model(self.view_model_index) else {
            return false;
        };
        let Some(instance) = view_model.instances.into_iter().nth(self.instance_index) else {
            return false;
        };
        let current = self.asset_overrides.get(&path).copied().or_else(|| {
            let source =
                file.data_context_view_model_property_for_instance(instance.object, &path)?;
            file.view_model_instance_asset_index_for_object(source)
        });
        if current == Some(value) {
            return false;
        }

        self.asset_overrides.insert(path, value);
        true
    }

    pub fn artboard_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeImportedViewModelArtboardSourceHandle> {
        let path = runtime_imported_view_model_artboard_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        )?;
        Some(RuntimeImportedViewModelArtboardSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn set_artboard_by_source_handle(
        &mut self,
        file: &RuntimeFile,
        handle: &RuntimeImportedViewModelArtboardSourceHandle,
        value: u64,
    ) -> bool {
        if handle.view_model_index != self.view_model_index
            || handle.instance_index != self.instance_index
        {
            return false;
        }
        self.set_artboard_by_resolved_property_path(file, handle.path.clone(), value)
    }

    pub fn set_artboard_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_artboard_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        ) else {
            return false;
        };
        self.set_artboard_by_resolved_property_path(file, path, value)
    }

    pub fn set_artboard_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        value: u64,
    ) -> bool {
        if property_path.contains('/') {
            return false;
        }
        self.set_artboard_by_property_name(file, property_path, value)
    }

    fn set_artboard_by_resolved_property_path(
        &mut self,
        file: &RuntimeFile,
        path: Vec<u32>,
        value: u64,
    ) -> bool {
        let Some(view_model) = file.view_model(self.view_model_index) else {
            return false;
        };
        let Some(instance) = view_model.instances.into_iter().nth(self.instance_index) else {
            return false;
        };
        let current = self.artboard_overrides.get(&path).copied().or_else(|| {
            let source =
                file.data_context_view_model_property_for_instance(instance.object, &path)?;
            file.view_model_instance_artboard_index_for_object(source)
        });
        if current == Some(value) {
            return false;
        }

        self.artboard_overrides.insert(path, value);
        true
    }

    pub fn trigger_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeImportedViewModelTriggerSourceHandle> {
        let path = runtime_imported_view_model_trigger_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        )?;
        Some(RuntimeImportedViewModelTriggerSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn set_trigger_by_source_handle(
        &mut self,
        file: &RuntimeFile,
        handle: &RuntimeImportedViewModelTriggerSourceHandle,
        value: u64,
    ) -> bool {
        if handle.view_model_index != self.view_model_index
            || handle.instance_index != self.instance_index
        {
            return false;
        }
        self.set_trigger_by_resolved_property_path(file, handle.path.clone(), value)
    }

    pub fn set_trigger_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_trigger_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        ) else {
            return false;
        };
        self.set_trigger_by_resolved_property_path(file, path, value)
    }

    pub fn set_trigger_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        value: u64,
    ) -> bool {
        if property_path.contains('/') {
            return false;
        }
        self.set_trigger_by_property_name(file, property_path, value)
    }

    fn set_trigger_by_resolved_property_path(
        &mut self,
        file: &RuntimeFile,
        path: Vec<u32>,
        value: u64,
    ) -> bool {
        let Some(view_model) = file.view_model(self.view_model_index) else {
            return false;
        };
        let Some(instance) = view_model.instances.into_iter().nth(self.instance_index) else {
            return false;
        };
        let current = self.trigger_overrides.get(&path).copied().or_else(|| {
            let source =
                file.data_context_view_model_property_for_instance(instance.object, &path)?;
            file.view_model_instance_trigger_count_for_object(source)
        });
        if current == Some(value) {
            return false;
        }

        self.trigger_overrides.insert(path, value);
        true
    }

    pub fn list_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeImportedViewModelListSourceHandle> {
        let path = runtime_imported_view_model_list_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        )?;
        Some(RuntimeImportedViewModelListSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn set_list_item_count_by_source_handle(
        &mut self,
        file: &RuntimeFile,
        handle: &RuntimeImportedViewModelListSourceHandle,
        item_count: usize,
    ) -> bool {
        if handle.view_model_index != self.view_model_index
            || handle.instance_index != self.instance_index
        {
            return false;
        }
        self.set_list_item_count_by_resolved_property_path(file, handle.path.clone(), item_count)
    }

    pub fn set_list_item_count_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        item_count: usize,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_list_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        ) else {
            return false;
        };
        self.set_list_item_count_by_resolved_property_path(file, path, item_count)
    }

    pub fn set_list_item_count_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        item_count: usize,
    ) -> bool {
        if property_path.contains('/') {
            return false;
        }
        self.set_list_item_count_by_property_name(file, property_path, item_count)
    }

    fn set_list_item_count_by_resolved_property_path(
        &mut self,
        file: &RuntimeFile,
        path: Vec<u32>,
        item_count: usize,
    ) -> bool {
        let Some(view_model) = file.view_model(self.view_model_index) else {
            return false;
        };
        let Some(instance) = view_model.instances.into_iter().nth(self.instance_index) else {
            return false;
        };
        let current = self.list_overrides.get(&path).copied().or_else(|| {
            let source =
                file.data_context_view_model_property_for_instance(instance.object, &path)?;
            file.view_model_instance_list_size_for_object(source)
        });
        if current == Some(item_count) {
            return false;
        }

        self.list_overrides.insert(path, item_count);
        true
    }

    pub fn view_model_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeImportedViewModelViewModelSourceHandle> {
        let path = runtime_imported_view_model_view_model_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        )?;
        Some(RuntimeImportedViewModelViewModelSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn view_model_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeImportedViewModelViewModelSourceHandle> {
        let path = runtime_imported_view_model_view_model_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        )?;
        Some(RuntimeImportedViewModelViewModelSourceHandle {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            path,
        })
    }

    pub fn set_view_model_by_source_handle(
        &mut self,
        file: &RuntimeFile,
        handle: &RuntimeImportedViewModelViewModelSourceHandle,
        instance_index: usize,
    ) -> bool {
        if handle.view_model_index != self.view_model_index
            || handle.instance_index != self.instance_index
        {
            return false;
        }
        self.set_view_model_by_resolved_property_path(file, handle.path.clone(), instance_index)
    }

    pub fn set_view_model_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        instance_index: usize,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_view_model_property_path_for_name(
            file,
            self.view_model_index,
            property_name,
        ) else {
            return false;
        };
        self.set_view_model_by_resolved_property_path(file, path, instance_index)
    }

    pub fn set_view_model_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        instance_index: usize,
    ) -> bool {
        let Some(path) = runtime_imported_view_model_view_model_property_path_for_name_path(
            file,
            self.view_model_index,
            property_path,
        ) else {
            return false;
        };
        self.set_view_model_by_resolved_property_path(file, path, instance_index)
    }

    fn set_view_model_by_resolved_property_path(
        &mut self,
        file: &RuntimeFile,
        path: Vec<u32>,
        instance_index: usize,
    ) -> bool {
        let Some(view_model) = file.view_model(self.view_model_index) else {
            return false;
        };
        let Some(instance) = view_model.instances.into_iter().nth(self.instance_index) else {
            return false;
        };
        let Some(referenced_view_model_index) =
            runtime_view_model_reference_index_for_property_path(file, &path)
        else {
            return false;
        };
        let Some(object_id) = file
            .view_model(referenced_view_model_index)
            .and_then(|view_model| view_model.instances.into_iter().nth(instance_index))
            .map(|instance| instance.object.id)
        else {
            return false;
        };
        let value = RuntimeViewModelPointer::Imported { object_id };
        let current = self.view_model_overrides.get(&path).copied().or_else(|| {
            file.data_context_view_model_instance_for_instance(instance.object, &path)
                .map(|reference| RuntimeViewModelPointer::Imported {
                    object_id: reference.object.id,
                })
        });
        if current == Some(value) {
            return false;
        }

        self.view_model_overrides.insert(path, value);
        true
    }
}
