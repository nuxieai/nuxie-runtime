use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Arc;

use nuxie_binary::{
    RuntimeDataValue, RuntimeFile, RuntimeObject, RuntimeViewModel, RuntimeViewModelInstance,
    RuntimeViewModelInstanceReference,
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

#[derive(Debug, Clone)]
pub struct RuntimeOwnedViewModelInstance {
    pub(crate) view_model_index: usize,
    instance_identity: u64,
    mutation_generation: u64,
    property_names: Vec<(String, usize)>,
    numbers: Vec<RuntimeOwnedViewModelNumber>,
    booleans: Vec<RuntimeOwnedViewModelBoolean>,
    strings: Vec<RuntimeOwnedViewModelString>,
    colors: Vec<RuntimeOwnedViewModelColor>,
    enums: Vec<RuntimeOwnedViewModelEnum>,
    symbol_list_indices: Vec<RuntimeOwnedViewModelSymbolListIndex>,
    lists: Vec<RuntimeOwnedViewModelList>,
    assets: Vec<RuntimeOwnedViewModelAsset>,
    font_assets: Vec<RuntimeOwnedViewModelFontAsset>,
    artboards: Vec<RuntimeOwnedViewModelArtboard>,
    triggers: Vec<RuntimeOwnedViewModelTrigger>,
    view_models: Vec<RuntimeOwnedViewModelViewModel>,
}

/// The ordered set of owned view-model instances visible to an artboard or
/// state machine.
///
/// Rive resolves the main instance first, followed by global slots in file
/// view-model order. A global slot is addressed by the declared global view
/// model, independently of the view model that produced the occupying
/// instance. That distinction is what permits a different view model to be
/// installed as an override for a global slot.
#[derive(Debug, Clone, Default)]
pub struct RuntimeOwnedViewModelContext {
    main: Option<RuntimeOwnedViewModelInstance>,
    global_slots: BTreeMap<usize, RuntimeOwnedViewModelInstance>,
}

impl RuntimeOwnedViewModelContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_main(main: RuntimeOwnedViewModelInstance) -> Self {
        Self {
            main: Some(main),
            global_slots: BTreeMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.main.is_none() && self.global_slots.is_empty()
    }

    pub fn main(&self) -> Option<&RuntimeOwnedViewModelInstance> {
        self.main.as_ref()
    }

    pub fn main_mut(&mut self) -> Option<&mut RuntimeOwnedViewModelInstance> {
        self.main.as_mut()
    }

    pub fn set_main(&mut self, main: RuntimeOwnedViewModelInstance) {
        self.main = Some(main);
    }

    pub fn take_main(&mut self) -> Option<RuntimeOwnedViewModelInstance> {
        self.main.take()
    }

    /// Returns instances in C++ `DataContext::viewModelInstances()` order:
    /// main first, then globals by their file view-model slot.
    pub fn instances(&self) -> impl Iterator<Item = &RuntimeOwnedViewModelInstance> {
        self.main.iter().chain(self.global_slots.values())
    }

    pub fn global_slot(&self, view_model_index: usize) -> Option<&RuntimeOwnedViewModelInstance> {
        self.global_slots.get(&view_model_index)
    }

    pub fn global_slot_mut(
        &mut self,
        view_model_index: usize,
    ) -> Option<&mut RuntimeOwnedViewModelInstance> {
        self.global_slots.get_mut(&view_model_index)
    }

    pub fn global_named(
        &self,
        file: &RuntimeFile,
        name: &str,
    ) -> Option<&RuntimeOwnedViewModelInstance> {
        let slot = runtime_global_view_model_index_named(file, name)?;
        self.global_slots.get(&slot)
    }

    pub fn global_named_mut(
        &mut self,
        file: &RuntimeFile,
        name: &str,
    ) -> Option<&mut RuntimeOwnedViewModelInstance> {
        let slot = runtime_global_view_model_index_named(file, name)?;
        self.global_slots.get_mut(&slot)
    }

    /// Installs `instance` into the named global slot. The instance's own view
    /// model intentionally need not match the slot's declared view model.
    pub fn set_global_named(
        &mut self,
        file: &RuntimeFile,
        name: &str,
        instance: RuntimeOwnedViewModelInstance,
    ) -> bool {
        let Some(slot) = runtime_global_view_model_index_named(file, name) else {
            return false;
        };
        self.global_slots.insert(slot, instance);
        true
    }

    pub fn set_global_slot(
        &mut self,
        file: &RuntimeFile,
        view_model_index: usize,
        instance: RuntimeOwnedViewModelInstance,
    ) -> bool {
        if !runtime_view_model_is_global(file, view_model_index) {
            return false;
        }
        self.global_slots.insert(view_model_index, instance);
        true
    }

    /// Completes any missing instances the same way C++ state-machine `bind()`
    /// does: the artboard's main default first, then every global default.
    /// Existing slots, including cross-view-model overrides, are preserved.
    pub fn complete_for_artboard(&mut self, file: &RuntimeFile, artboard_index: usize) -> bool {
        let main_view_model_index = file
            .resolved_view_model_for_artboard(artboard_index)
            .map(|view_model| view_model.view_model_index);
        self.complete(file, main_view_model_index)
    }

    pub fn complete(&mut self, file: &RuntimeFile, main_view_model_index: Option<usize>) -> bool {
        let mut changed = false;
        if self.main.is_none() {
            if let Some(view_model_index) = main_view_model_index {
                if let Some(instance) =
                    runtime_default_owned_view_model_instance(file, view_model_index)
                {
                    self.main = Some(instance);
                    changed = true;
                }
            }
        }
        for view_model_index in runtime_global_view_model_indices(file) {
            if self.global_slots.contains_key(&view_model_index) {
                continue;
            }
            let Some(instance) = runtime_default_owned_view_model_instance(file, view_model_index)
            else {
                continue;
            };
            self.global_slots.insert(view_model_index, instance);
            changed = true;
        }
        changed
    }
}

fn runtime_default_owned_view_model_instance(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Option<RuntimeOwnedViewModelInstance> {
    RuntimeOwnedViewModelInstance::from_instance(file, view_model_index, 0)
        .or_else(|| RuntimeOwnedViewModelInstance::new(file, view_model_index))
}

fn runtime_view_model_is_global(file: &RuntimeFile, view_model_index: usize) -> bool {
    file.view_model(view_model_index)
        .and_then(|view_model| view_model.object.uint_property("viewModelType"))
        == Some(2)
}

pub fn runtime_global_view_model_indices(file: &RuntimeFile) -> Vec<usize> {
    file.view_models()
        .iter()
        .enumerate()
        .filter_map(|(index, view_model)| {
            (view_model.object.uint_property("viewModelType") == Some(2)).then_some(index)
        })
        .collect()
}

pub fn runtime_global_view_model_names(file: &RuntimeFile) -> Vec<String> {
    runtime_global_view_model_indices(file)
        .into_iter()
        .filter_map(|index| {
            file.view_model(index)
                .and_then(|view_model| view_model.object.string_property("name"))
                .map(str::to_owned)
        })
        .collect()
}

fn runtime_global_view_model_index_named(file: &RuntimeFile, name: &str) -> Option<usize> {
    file.view_models()
        .iter()
        .enumerate()
        .find_map(|(index, view_model)| {
            (view_model.object.uint_property("viewModelType") == Some(2)
                && view_model.object.string_property("name") == Some(name))
            .then_some(index)
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelNumberSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelNumberSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelBooleanSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelBooleanSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelStringSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelStringSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelColorSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelColorSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelEnumSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelEnumSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelSymbolListIndexSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelSymbolListIndexSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelAssetSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelAssetSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelFontAssetSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelFontAssetSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelArtboardSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelArtboardSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelTriggerSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelTriggerSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelListSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelListSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeOwnedViewModelListHandle {
    value: Rc<RefCell<RuntimeOwnedViewModelListValue>>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeOwnedViewModelListItemEntry {
    pub(crate) occurrence_identity: u64,
    pub(crate) instance: RuntimeOwnedViewModelInstance,
}

impl RuntimeOwnedViewModelListHandle {
    pub(crate) fn items(&self) -> Vec<RuntimeOwnedViewModelInstance> {
        self.value
            .borrow()
            .items
            .iter()
            .map(|item| item.instance.borrow().clone())
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn item_entries(&self) -> Vec<RuntimeOwnedViewModelListItemEntry> {
        self.value
            .borrow()
            .items
            .iter()
            .map(|item| RuntimeOwnedViewModelListItemEntry {
                occurrence_identity: item.occurrence_identity,
                instance: item.instance.borrow().clone(),
            })
            .collect()
    }

    /// Mirrors `ArtboardComponentList::updateList`: immediately before rows
    /// are mounted, C++ writes each wrapper's current logical position into
    /// the synthetic `itemIndex` symbol on its view-model instance. Mutate the
    /// shared list instances before taking the row snapshots so bindings and
    /// public handles observe the same value.
    pub(crate) fn item_entries_with_logical_indices(
        &self,
        file: &RuntimeFile,
    ) -> Vec<RuntimeOwnedViewModelListItemEntry> {
        self.value
            .borrow()
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let mut instance = item.instance.borrow_mut();
                set_component_list_item_index(file, &mut instance, index);
                RuntimeOwnedViewModelListItemEntry {
                    occurrence_identity: item.occurrence_identity,
                    instance: instance.clone(),
                }
            })
            .collect()
    }

    pub(crate) fn text_runs(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.value
            .borrow()
            .items
            .iter()
            .filter_map(|item| {
                let item = item.instance.borrow();
                Some((
                    item.string_value_by_property_name("textContent")?.to_vec(),
                    item.string_value_by_property_name("textStyle")
                        .unwrap_or_default()
                        .to_vec(),
                ))
            })
            .collect()
    }
}

pub(crate) fn set_component_list_item_index(
    _file: &RuntimeFile,
    instance: &mut RuntimeOwnedViewModelInstance,
    index: usize,
) -> bool {
    // `ViewModelInstanceValue::registerSymbol` overwrites the itemIndex
    // symbol as values are registered. Generated instances register in
    // property order; imported instances register in instance-value order.
    // The constructors preserve that winner as the last entry here.
    let Some(property_index) = instance
        .symbol_list_indices
        .last()
        .map(|symbol_list_index| symbol_list_index.property_index)
    else {
        return false;
    };
    instance.set_symbol_list_index_by_property_index(property_index, index as u64)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelViewModelSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelViewModelSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelNumber {
    property_index: usize,
    value: f32,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelBoolean {
    property_index: usize,
    value: bool,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelString {
    property_index: usize,
    value: Vec<u8>,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelColor {
    property_index: usize,
    value: u32,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelEnum {
    property_index: usize,
    value: u64,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelSymbolListIndex {
    property_index: usize,
    value: u64,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelList {
    property_index: usize,
    value: Rc<RefCell<RuntimeOwnedViewModelListValue>>,
}

#[derive(Debug, Clone, Default)]
struct RuntimeOwnedViewModelListValue {
    item_count: usize,
    items: Vec<RuntimeOwnedViewModelListItem>,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelListItem {
    occurrence_identity: u64,
    instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>,
}

fn reset_runtime_owned_triggers(triggers: &mut [RuntimeOwnedViewModelTrigger]) -> bool {
    let mut changed = false;
    for trigger in triggers {
        if trigger.value != 0 {
            trigger.value = 0;
            changed = true;
        }
    }
    changed
}

fn collect_runtime_owned_list_children(
    lists: &[RuntimeOwnedViewModelList],
    children: &mut Vec<Rc<RefCell<RuntimeOwnedViewModelInstance>>>,
) {
    for list in lists {
        children.extend(
            list.value
                .borrow()
                .items
                .iter()
                .map(|item| Rc::clone(&item.instance)),
        );
    }
}

impl RuntimeOwnedViewModelListItem {
    fn new(instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_OCCURRENCE_IDENTITY: AtomicU64 = AtomicU64::new(0);
        Self {
            occurrence_identity: NEXT_OCCURRENCE_IDENTITY.fetch_add(1, Ordering::Relaxed),
            instance,
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelAsset {
    property_index: usize,
    value: u64,
}

/// The two-part value stored by C++ `ViewModelInstanceAssetFont`.
///
/// `file_asset_index` is the serialized `propertyValue` and names a dense
/// file-asset entry. `live_font_bytes` is the private runtime-only FontAsset
/// used when that index does not resolve to a file FontAsset. Assigning a live
/// font sets the index to the C++ missing-id sentinel; assigning a file index
/// deliberately preserves the private value, matching the upstream object.
#[derive(Debug, Clone)]
pub struct RuntimeFontAssetValue {
    file_asset_index: u64,
    live_font_bytes: Option<Arc<[u8]>>,
}

impl RuntimeFontAssetValue {
    pub const MISSING_FILE_ASSET_INDEX: u64 = u32::MAX as u64;

    pub fn from_file_asset_index(file_asset_index: u64) -> Self {
        Self {
            file_asset_index,
            live_font_bytes: None,
        }
    }

    pub fn file_asset_index(&self) -> u64 {
        self.file_asset_index
    }

    pub fn live_font_bytes(&self) -> Option<&[u8]> {
        self.live_font_bytes.as_deref()
    }

    pub fn live_font_bytes_arc(&self) -> Option<&Arc<[u8]>> {
        self.live_font_bytes.as_ref()
    }

    pub(crate) fn same_runtime_value(&self, value: &Self) -> bool {
        if self.file_asset_index != value.file_asset_index {
            return false;
        }
        match (&self.live_font_bytes, &value.live_font_bytes) {
            (Some(current), Some(next)) => Arc::ptr_eq(current, next),
            (None, None) => true,
            _ => false,
        }
    }

    pub(crate) fn set_file_asset_index(&mut self, file_asset_index: u64) -> bool {
        if self.file_asset_index == file_asset_index {
            return false;
        }
        self.file_asset_index = file_asset_index;
        true
    }

    pub(crate) fn set_live_font_bytes(&mut self, font_bytes: Option<Arc<[u8]>>) -> bool {
        let same_live_font = match (&self.live_font_bytes, &font_bytes) {
            (Some(current), Some(next)) => Arc::ptr_eq(current, next),
            (None, None) => true,
            _ => false,
        };
        let was_missing = self.file_asset_index == Self::MISSING_FILE_ASSET_INDEX;
        self.file_asset_index = Self::MISSING_FILE_ASSET_INDEX;
        if same_live_font {
            return !was_missing;
        }
        self.live_font_bytes = font_bytes;
        true
    }

    /// Apply the complete value carried by a font data bind.
    ///
    /// This deliberately differs from [`Self::set_file_asset_index`]. A
    /// public `propertyValue` write preserves the private live font in C++,
    /// while `ViewModelInstanceAssetFont::applyValue(DataValueInteger*)`
    /// first applies (and therefore clears or replaces) the retained Font
    /// payload before falling back to the serialized file-asset index.
    pub(crate) fn apply_data_bind_value(&mut self, value: &Self) -> bool {
        if self.same_runtime_value(value) {
            return false;
        }
        self.file_asset_index = value.file_asset_index;
        self.live_font_bytes = value.live_font_bytes.clone();
        true
    }
}

impl Default for RuntimeFontAssetValue {
    fn default() -> Self {
        Self::from_file_asset_index(Self::MISSING_FILE_ASSET_INDEX)
    }
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelFontAsset {
    property_index: usize,
    value: RuntimeFontAssetValue,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelArtboard {
    property_index: usize,
    value: u64,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelTrigger {
    property_index: usize,
    value: u64,
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelViewModel {
    property_index: usize,
    property_name: String,
    value: RuntimeViewModelPointer,
    referenced_view_model_index: Option<usize>,
    property_names: Vec<(String, usize)>,
    numbers: Vec<RuntimeOwnedViewModelNumber>,
    imported_numbers: BTreeMap<u32, Vec<RuntimeOwnedViewModelNumber>>,
    booleans: Vec<RuntimeOwnedViewModelBoolean>,
    imported_booleans: BTreeMap<u32, Vec<RuntimeOwnedViewModelBoolean>>,
    strings: Vec<RuntimeOwnedViewModelString>,
    imported_strings: BTreeMap<u32, Vec<RuntimeOwnedViewModelString>>,
    colors: Vec<RuntimeOwnedViewModelColor>,
    imported_colors: BTreeMap<u32, Vec<RuntimeOwnedViewModelColor>>,
    enums: Vec<RuntimeOwnedViewModelEnum>,
    imported_enums: BTreeMap<u32, Vec<RuntimeOwnedViewModelEnum>>,
    symbol_list_indices: Vec<RuntimeOwnedViewModelSymbolListIndex>,
    imported_symbol_list_indices: BTreeMap<u32, Vec<RuntimeOwnedViewModelSymbolListIndex>>,
    lists: Vec<RuntimeOwnedViewModelList>,
    imported_lists: BTreeMap<u32, Vec<RuntimeOwnedViewModelList>>,
    assets: Vec<RuntimeOwnedViewModelAsset>,
    imported_assets: BTreeMap<u32, Vec<RuntimeOwnedViewModelAsset>>,
    font_assets: Vec<RuntimeOwnedViewModelFontAsset>,
    imported_font_assets: BTreeMap<u32, Vec<RuntimeOwnedViewModelFontAsset>>,
    artboards: Vec<RuntimeOwnedViewModelArtboard>,
    imported_artboards: BTreeMap<u32, Vec<RuntimeOwnedViewModelArtboard>>,
    triggers: Vec<RuntimeOwnedViewModelTrigger>,
    imported_triggers: BTreeMap<u32, Vec<RuntimeOwnedViewModelTrigger>>,
    view_model_instance_ids: Vec<u32>,
    children: Vec<RuntimeOwnedViewModelViewModel>,
    imported_children: BTreeMap<u32, Vec<RuntimeOwnedViewModelViewModel>>,
}

impl RuntimeOwnedViewModelViewModel {
    fn active_children(&self) -> Option<&[RuntimeOwnedViewModelViewModel]> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => Some(self.children.as_slice()),
            RuntimeViewModelPointer::Imported { object_id } => {
                self.imported_children.get(&object_id).map(Vec::as_slice)
            }
            _ => None,
        }
    }

    fn generated_children_mut(&mut self) -> Option<&mut Vec<RuntimeOwnedViewModelViewModel>> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => Some(&mut self.children),
            _ => None,
        }
    }

    fn active_children_mut(&mut self) -> Option<&mut Vec<RuntimeOwnedViewModelViewModel>> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => Some(&mut self.children),
            RuntimeViewModelPointer::Imported { object_id } => {
                self.imported_children.get_mut(&object_id)
            }
            _ => None,
        }
    }

    fn property_index_by_name(&self, property_name: &str) -> Option<usize> {
        runtime_owned_view_model_property_index_by_name(&self.property_names, property_name)
    }

    fn number_value_by_property_index(&self, property_index: usize) -> Option<f32> {
        self.numbers
            .iter()
            .find(|number| number.property_index == property_index)
            .map(|number| number.value)
    }

    fn active_number_value_by_property_index(&self, property_index: usize) -> Option<f32> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.number_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_numbers
                .get(&object_id)
                .and_then(|numbers| {
                    numbers
                        .iter()
                        .find(|number| number.property_index == property_index)
                })
                .map(|number| number.value),
            _ => None,
        }
    }

    fn boolean_value_by_property_index(&self, property_index: usize) -> Option<bool> {
        self.booleans
            .iter()
            .find(|boolean| boolean.property_index == property_index)
            .map(|boolean| boolean.value)
    }

    fn active_boolean_value_by_property_index(&self, property_index: usize) -> Option<bool> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.boolean_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_booleans
                .get(&object_id)
                .and_then(|booleans| {
                    booleans
                        .iter()
                        .find(|boolean| boolean.property_index == property_index)
                })
                .map(|boolean| boolean.value),
            _ => None,
        }
    }

    fn string_value_by_property_index(&self, property_index: usize) -> Option<&[u8]> {
        self.strings
            .iter()
            .find(|string| string.property_index == property_index)
            .map(|string| string.value.as_slice())
    }

    fn active_string_value_by_property_index(&self, property_index: usize) -> Option<&[u8]> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.string_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_strings
                .get(&object_id)
                .and_then(|strings| {
                    strings
                        .iter()
                        .find(|string| string.property_index == property_index)
                })
                .map(|string| string.value.as_slice()),
            _ => None,
        }
    }

    fn color_value_by_property_index(&self, property_index: usize) -> Option<u32> {
        self.colors
            .iter()
            .find(|color| color.property_index == property_index)
            .map(|color| color.value)
    }

    fn active_color_value_by_property_index(&self, property_index: usize) -> Option<u32> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.color_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_colors
                .get(&object_id)
                .and_then(|colors| {
                    colors
                        .iter()
                        .find(|color| color.property_index == property_index)
                })
                .map(|color| color.value),
            _ => None,
        }
    }

    fn enum_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.enums
            .iter()
            .find(|enum_value| enum_value.property_index == property_index)
            .map(|enum_value| enum_value.value)
    }

    fn active_enum_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.enum_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_enums
                .get(&object_id)
                .and_then(|enums| {
                    enums
                        .iter()
                        .find(|enum_value| enum_value.property_index == property_index)
                })
                .map(|enum_value| enum_value.value),
            _ => None,
        }
    }

    fn symbol_list_index_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.symbol_list_indices
            .iter()
            .find(|symbol_list_index| symbol_list_index.property_index == property_index)
            .map(|symbol_list_index| symbol_list_index.value)
    }

    fn active_symbol_list_index_value_by_property_index(
        &self,
        property_index: usize,
    ) -> Option<u64> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.symbol_list_index_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_symbol_list_indices
                .get(&object_id)
                .and_then(|symbol_list_indices| {
                    symbol_list_indices.iter().find(|symbol_list_index| {
                        symbol_list_index.property_index == property_index
                    })
                })
                .map(|symbol_list_index| symbol_list_index.value),
            _ => None,
        }
    }

    fn list_item_count_by_property_index(&self, property_index: usize) -> Option<usize> {
        self.lists
            .iter()
            .find(|list| list.property_index == property_index)
            .map(|list| list.value.borrow().item_count)
    }

    fn active_list_item_count_by_property_index(&self, property_index: usize) -> Option<usize> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.list_item_count_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_lists
                .get(&object_id)
                .and_then(|lists| {
                    lists
                        .iter()
                        .find(|list| list.property_index == property_index)
                })
                .map(|list| list.value.borrow().item_count),
            _ => None,
        }
    }

    fn active_list_by_property_index(
        &self,
        property_index: usize,
    ) -> Option<&RuntimeOwnedViewModelList> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => self
                .lists
                .iter()
                .find(|list| list.property_index == property_index),
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_lists
                .get(&object_id)?
                .iter()
                .find(|list| list.property_index == property_index),
            _ => None,
        }
    }

    fn materialize_active_instance(&self) -> Option<RuntimeOwnedViewModelInstance> {
        let view_model_index = self.referenced_view_model_index?;
        macro_rules! active_values {
            ($owned:expr, $imported:expr) => {{
                match self.value {
                    RuntimeViewModelPointer::OwnedGenerated { .. } => $owned.clone(),
                    RuntimeViewModelPointer::Imported { object_id } => {
                        $imported.get(&object_id).cloned().unwrap_or_default()
                    }
                    _ => Vec::new(),
                }
            }};
        }
        Some(RuntimeOwnedViewModelInstance {
            view_model_index,
            instance_identity: RuntimeOwnedViewModelInstance::next_instance_identity(),
            mutation_generation: 0,
            property_names: self.property_names.clone(),
            numbers: active_values!(&self.numbers, self.imported_numbers),
            booleans: active_values!(&self.booleans, self.imported_booleans),
            strings: active_values!(&self.strings, self.imported_strings),
            colors: active_values!(&self.colors, self.imported_colors),
            enums: active_values!(&self.enums, self.imported_enums),
            symbol_list_indices: active_values!(
                &self.symbol_list_indices,
                self.imported_symbol_list_indices
            ),
            lists: active_values!(&self.lists, self.imported_lists),
            assets: active_values!(&self.assets, self.imported_assets),
            font_assets: active_values!(&self.font_assets, self.imported_font_assets),
            artboards: active_values!(&self.artboards, self.imported_artboards),
            triggers: active_values!(&self.triggers, self.imported_triggers),
            view_models: match self.value {
                RuntimeViewModelPointer::OwnedGenerated { .. } => self.children.clone(),
                RuntimeViewModelPointer::Imported { object_id } => self
                    .imported_children
                    .get(&object_id)
                    .cloned()
                    .unwrap_or_default(),
                _ => Vec::new(),
            },
        })
    }

    fn asset_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.assets
            .iter()
            .find(|asset| asset.property_index == property_index)
            .map(|asset| asset.value)
    }

    fn active_asset_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.asset_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_assets
                .get(&object_id)
                .and_then(|assets| {
                    assets
                        .iter()
                        .find(|asset| asset.property_index == property_index)
                })
                .map(|asset| asset.value),
            _ => None,
        }
    }

    fn font_asset_value_by_property_index(
        &self,
        property_index: usize,
    ) -> Option<&RuntimeFontAssetValue> {
        self.font_assets
            .iter()
            .find(|asset| asset.property_index == property_index)
            .map(|asset| &asset.value)
    }

    fn active_font_asset_value_by_property_index(
        &self,
        property_index: usize,
    ) -> Option<&RuntimeFontAssetValue> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.font_asset_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_font_assets
                .get(&object_id)
                .and_then(|assets| {
                    assets
                        .iter()
                        .find(|asset| asset.property_index == property_index)
                })
                .map(|asset| &asset.value),
            _ => None,
        }
    }

    fn artboard_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.artboards
            .iter()
            .find(|artboard| artboard.property_index == property_index)
            .map(|artboard| artboard.value)
    }

    fn active_artboard_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.artboard_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_artboards
                .get(&object_id)
                .and_then(|artboards| {
                    artboards
                        .iter()
                        .find(|artboard| artboard.property_index == property_index)
                })
                .map(|artboard| artboard.value),
            _ => None,
        }
    }

    pub(crate) fn trigger_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.triggers
            .iter()
            .find(|trigger| trigger.property_index == property_index)
            .map(|trigger| trigger.value)
    }

    fn active_trigger_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.trigger_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_triggers
                .get(&object_id)
                .and_then(|triggers| {
                    triggers
                        .iter()
                        .find(|trigger| trigger.property_index == property_index)
                })
                .map(|trigger| trigger.value),
            _ => None,
        }
    }

    /// Mirrors the recursive portion of C++
    /// `ViewModelInstanceViewModel::advanced()` for the currently selected
    /// nested instance. Shared list children are returned to the caller so it
    /// can recurse after releasing this instance's mutable borrow.
    fn advance_script_frame(
        &mut self,
        shared_children: &mut Vec<Rc<RefCell<RuntimeOwnedViewModelInstance>>>,
    ) -> bool {
        match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                let mut changed = reset_runtime_owned_triggers(&mut self.triggers);
                collect_runtime_owned_list_children(&self.lists, shared_children);
                for child in &mut self.children {
                    changed |= child.advance_script_frame(shared_children);
                }
                changed
            }
            RuntimeViewModelPointer::Imported { object_id } => {
                let mut changed = self
                    .imported_triggers
                    .get_mut(&object_id)
                    .is_some_and(|triggers| reset_runtime_owned_triggers(triggers));
                if let Some(lists) = self.imported_lists.get(&object_id) {
                    collect_runtime_owned_list_children(lists, shared_children);
                }
                if let Some(children) = self.imported_children.get_mut(&object_id) {
                    for child in children {
                        changed |= child.advance_script_frame(shared_children);
                    }
                }
                changed
            }
            RuntimeViewModelPointer::Null | RuntimeViewModelPointer::DataContextRoot => false,
        }
    }

    fn set_number_by_property_name(&mut self, property_name: &str, value: f32) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_number_by_property_index(property_index, value)
    }

    fn set_number_by_property_index(&mut self, property_index: usize, value: f32) -> bool {
        let Some(number) = self
            .numbers
            .iter_mut()
            .find(|number| number.property_index == property_index)
        else {
            return false;
        };
        if number.value == value {
            return false;
        }
        number.value = value;
        true
    }

    fn set_boolean_by_property_name(&mut self, property_name: &str, value: bool) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_boolean_by_property_index(property_index, value)
    }

    fn set_boolean_by_property_index(&mut self, property_index: usize, value: bool) -> bool {
        let Some(boolean) = self
            .booleans
            .iter_mut()
            .find(|boolean| boolean.property_index == property_index)
        else {
            return false;
        };
        if boolean.value == value {
            return false;
        }
        boolean.value = value;
        true
    }

    fn set_string_by_property_name(&mut self, property_name: &str, value: &[u8]) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_string_by_property_index(property_index, value)
    }

    fn set_string_by_property_index(&mut self, property_index: usize, value: &[u8]) -> bool {
        let Some(string) = self
            .strings
            .iter_mut()
            .find(|string| string.property_index == property_index)
        else {
            return false;
        };
        if string.value == value {
            return false;
        }
        string.value = value.to_vec();
        true
    }

    fn set_color_by_property_name(&mut self, property_name: &str, value: u32) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_color_by_property_index(property_index, value)
    }

    fn set_color_by_property_index(&mut self, property_index: usize, value: u32) -> bool {
        let Some(color) = self
            .colors
            .iter_mut()
            .find(|color| color.property_index == property_index)
        else {
            return false;
        };
        if color.value == value {
            return false;
        }
        color.value = value;
        true
    }

    fn set_enum_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_enum_by_property_index(property_index, value)
    }

    fn set_enum_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(enum_value) = self
            .enums
            .iter_mut()
            .find(|enum_value| enum_value.property_index == property_index)
        else {
            return false;
        };
        if enum_value.value == value {
            return false;
        }
        enum_value.value = value;
        true
    }

    fn set_symbol_list_index_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_symbol_list_index_by_property_index(property_index, value)
    }

    fn set_symbol_list_index_by_property_index(
        &mut self,
        property_index: usize,
        value: u64,
    ) -> bool {
        let Some(symbol_list_index) = self
            .symbol_list_indices
            .iter_mut()
            .find(|symbol_list_index| symbol_list_index.property_index == property_index)
        else {
            return false;
        };
        if symbol_list_index.value == value {
            return false;
        }
        symbol_list_index.value = value;
        true
    }

    fn set_list_item_count_by_property_name(
        &mut self,
        property_name: &str,
        item_count: usize,
    ) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_list_item_count_by_property_index(property_index, item_count)
    }

    fn set_list_item_count_by_property_index(
        &mut self,
        property_index: usize,
        item_count: usize,
    ) -> bool {
        let Some(list) = self
            .lists
            .iter_mut()
            .find(|list| list.property_index == property_index)
        else {
            return false;
        };
        let mut value = list.value.borrow_mut();
        if value.item_count == item_count {
            return false;
        }
        value.item_count = item_count;
        value.items.truncate(item_count);
        true
    }

    fn set_asset_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_asset_by_property_index(property_index, value)
    }

    fn set_asset_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(asset) = self
            .assets
            .iter_mut()
            .find(|asset| asset.property_index == property_index)
        else {
            return false;
        };
        if asset.value == value {
            return false;
        }
        asset.value = value;
        true
    }

    fn set_font_asset_index_by_property_name(
        &mut self,
        property_name: &str,
        file_asset_index: u64,
    ) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_font_asset_index_by_property_index(property_index, file_asset_index)
    }

    fn set_font_asset_index_by_property_index(
        &mut self,
        property_index: usize,
        file_asset_index: u64,
    ) -> bool {
        let Some(asset) = self
            .font_assets
            .iter_mut()
            .find(|asset| asset.property_index == property_index)
        else {
            return false;
        };
        asset.value.set_file_asset_index(file_asset_index)
    }

    fn set_live_font_bytes_by_property_name(
        &mut self,
        property_name: &str,
        font_bytes: Option<Arc<[u8]>>,
    ) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_live_font_bytes_by_property_index(property_index, font_bytes)
    }

    fn set_live_font_bytes_by_property_index(
        &mut self,
        property_index: usize,
        font_bytes: Option<Arc<[u8]>>,
    ) -> bool {
        let Some(asset) = self
            .font_assets
            .iter_mut()
            .find(|asset| asset.property_index == property_index)
        else {
            return false;
        };
        asset.value.set_live_font_bytes(font_bytes)
    }

    fn set_artboard_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_artboard_by_property_index(property_index, value)
    }

    fn set_artboard_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(artboard) = self
            .artboards
            .iter_mut()
            .find(|artboard| artboard.property_index == property_index)
        else {
            return false;
        };
        if artboard.value == value {
            return false;
        }
        artboard.value = value;
        true
    }

    fn set_trigger_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_trigger_by_property_index(property_index, value)
    }

    fn set_trigger_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(trigger) = self
            .triggers
            .iter_mut()
            .find(|trigger| trigger.property_index == property_index)
        else {
            return false;
        };
        if trigger.value == value {
            return false;
        }
        trigger.value = value;
        true
    }

    fn sync_number_by_property_index(&mut self, property_index: usize, value: f32) -> bool {
        let values = match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.numbers,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_numbers.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if current.value == value {
            return false;
        }
        current.value = value;
        true
    }

    fn sync_boolean_by_property_index(&mut self, property_index: usize, value: bool) -> bool {
        let values = match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.booleans,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_booleans.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if current.value == value {
            return false;
        }
        current.value = value;
        true
    }

    fn sync_string_by_property_index(&mut self, property_index: usize, value: &[u8]) -> bool {
        let values = match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.strings,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_strings.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if current.value == value {
            return false;
        }
        current.value = value.to_vec();
        true
    }

    fn sync_color_by_property_index(&mut self, property_index: usize, value: u32) -> bool {
        let values = match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.colors,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_colors.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if current.value == value {
            return false;
        }
        current.value = value;
        true
    }

    fn sync_enum_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let values = match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.enums,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_enums.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if current.value == value {
            return false;
        }
        current.value = value;
        true
    }

    fn sync_symbol_list_index_by_property_index(
        &mut self,
        property_index: usize,
        value: u64,
    ) -> bool {
        let values = match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.symbol_list_indices,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_symbol_list_indices.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if current.value == value {
            return false;
        }
        current.value = value;
        true
    }

    fn sync_asset_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let values = match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.assets,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_assets.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if current.value == value {
            return false;
        }
        current.value = value;
        true
    }

    fn sync_font_asset_index_by_property_index(
        &mut self,
        property_index: usize,
        file_asset_index: u64,
    ) -> bool {
        let values = match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.font_assets,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_font_assets.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        current.value.set_file_asset_index(file_asset_index)
    }

    fn sync_live_font_bytes_by_property_index(
        &mut self,
        property_index: usize,
        font_bytes: Option<Arc<[u8]>>,
    ) -> bool {
        let values = match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.font_assets,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_font_assets.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        current.value.set_live_font_bytes(font_bytes)
    }

    fn apply_font_asset_data_bind_value_by_property_index(
        &mut self,
        property_index: usize,
        value: &RuntimeFontAssetValue,
    ) -> bool {
        let values = match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.font_assets,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_font_assets.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        current.value.apply_data_bind_value(value)
    }

    fn sync_artboard_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let values = match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.artboards,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_artboards.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if current.value == value {
            return false;
        }
        current.value = value;
        true
    }

    fn sync_trigger_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let values = match self.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.triggers,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_triggers.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if current.value == value {
            return false;
        }
        current.value = value;
        true
    }
}

fn runtime_owned_view_model_path_key(path: &[usize]) -> u64 {
    let mut key = 0xcbf29ce484222325u64;
    for segment in path {
        key ^= *segment as u64;
        key = key.wrapping_mul(0x100000001b3);
    }
    key
}

fn runtime_owned_view_model_property_index_by_name(
    property_names: &[(String, usize)],
    property_name: &str,
) -> Option<usize> {
    property_names
        .iter()
        .find_map(|(name, index)| (name == property_name).then_some(*index))
}

fn runtime_owned_view_model_property_names(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<(String, usize)> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .map(|(property_index, property)| {
                    (
                        property
                            .string_property("name")
                            .unwrap_or_default()
                            .to_owned(),
                        property_index,
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_imported_view_model_number_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyNumber",
    )
}

fn runtime_imported_view_model_number_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyNumber"],
    )
}

pub(crate) fn runtime_default_view_model_number_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_number_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_number_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_number_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_boolean_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyBoolean",
    )
}

fn runtime_imported_view_model_boolean_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyBoolean"],
    )
}

pub(crate) fn runtime_default_view_model_boolean_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_boolean_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_boolean_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_boolean_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_string_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyString",
    )
}

fn runtime_imported_view_model_string_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyString"],
    )
}

pub(crate) fn runtime_default_view_model_string_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_string_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_string_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_string_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_color_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyColor",
    )
}

fn runtime_imported_view_model_color_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyColor"],
    )
}

pub(crate) fn runtime_default_view_model_color_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_color_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_color_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_color_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_enum_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_type_names(
        file,
        view_model_index,
        property_name,
        &[
            "ViewModelPropertyEnum",
            "ViewModelPropertyEnumCustom",
            "ViewModelPropertyEnumSystem",
        ],
    )
}

fn runtime_imported_view_model_enum_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &[
            "ViewModelPropertyEnum",
            "ViewModelPropertyEnumCustom",
            "ViewModelPropertyEnumSystem",
        ],
    )
}

pub(crate) fn runtime_default_view_model_enum_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_enum_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_enum_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_enum_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_symbol_list_index_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertySymbolListIndex",
    )
}

fn runtime_imported_view_model_symbol_list_index_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertySymbolListIndex"],
    )
}

pub(crate) fn runtime_default_view_model_symbol_list_index_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_symbol_list_index_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_symbol_list_index_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_symbol_list_index_property_path_for_name_path(
        file,
        0,
        property_path,
    )
}

fn runtime_imported_view_model_asset_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_type_names(
        file,
        view_model_index,
        property_name,
        &["ViewModelPropertyAsset", "ViewModelPropertyAssetImage"],
    )
}

fn runtime_imported_view_model_asset_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyAsset", "ViewModelPropertyAssetImage"],
    )
}

pub(crate) fn runtime_default_view_model_asset_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_asset_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_asset_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_asset_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_artboard_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyArtboard",
    )
}

fn runtime_imported_view_model_artboard_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyArtboard"],
    )
}

pub(crate) fn runtime_default_view_model_artboard_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_artboard_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_artboard_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_artboard_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_trigger_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyTrigger",
    )
}

fn runtime_imported_view_model_trigger_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyTrigger"],
    )
}

pub(crate) fn runtime_default_view_model_trigger_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_trigger_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_trigger_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_trigger_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_list_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyList",
    )
}

fn runtime_imported_view_model_list_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyList"],
    )
}

pub(crate) fn runtime_default_view_model_list_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_list_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_list_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_list_property_path_for_name_path(file, 0, property_path)
}

fn runtime_imported_view_model_view_model_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyViewModel",
    )
}

fn runtime_imported_view_model_view_model_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyViewModel"],
    )
}

fn runtime_imported_view_model_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
    property_type_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_type_names(
        file,
        view_model_index,
        property_name,
        &[property_type_name],
    )
}

fn runtime_imported_view_model_property_path_for_type_names(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
    property_type_names: &[&str],
) -> Option<Vec<u32>> {
    if property_name.is_empty() {
        return None;
    }
    let view_model = file.view_model(view_model_index)?;
    view_model
        .properties
        .into_iter()
        .enumerate()
        .find_map(|(property_index, property)| {
            if !property_type_names.contains(&property.type_name) {
                return None;
            }
            if property.string_property("name")? != property_name {
                return None;
            }
            Some(vec![
                u32::try_from(view_model_index).ok()?,
                u32::try_from(property_index).ok()?,
            ])
        })
}

fn runtime_imported_view_model_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
    property_type_names: &[&str],
) -> Option<Vec<u32>> {
    let property_names = property_path.split('/').collect::<Vec<_>>();
    if property_names.is_empty() || property_names.iter().any(|segment| segment.is_empty()) {
        return None;
    }

    let mut current_view_model_index = view_model_index;
    let mut path = vec![u32::try_from(view_model_index).ok()?];
    for (property_name_index, property_name) in property_names.iter().enumerate() {
        let view_model = file.view_model(current_view_model_index)?;
        let (property_index, property) = view_model
            .properties
            .into_iter()
            .enumerate()
            .find(|(_, property)| property.string_property("name") == Some(*property_name))?;
        path.push(u32::try_from(property_index).ok()?);
        if property_name_index + 1 == property_names.len() {
            return property_type_names
                .contains(&property.type_name)
                .then_some(path);
        }
        if property.type_name != "ViewModelPropertyViewModel" {
            return None;
        }
        current_view_model_index =
            usize::try_from(property.uint_property("viewModelReferenceId")?).ok()?;
    }

    None
}

pub(crate) fn runtime_view_model_reference_index_for_property_path(
    file: &RuntimeFile,
    property_path: &[u32],
) -> Option<usize> {
    let mut current_view_model_index = usize::try_from(*property_path.first()?).ok()?;
    let property_indices = property_path.get(1..)?;
    if property_indices.is_empty() {
        return None;
    }

    for (segment_index, property_index) in property_indices.iter().enumerate() {
        let view_model = file.view_model(current_view_model_index)?;
        let property = view_model
            .properties
            .into_iter()
            .nth(usize::try_from(*property_index).ok()?)?;
        if property.type_name != "ViewModelPropertyViewModel" {
            return None;
        }
        let referenced_view_model_index =
            usize::try_from(property.uint_property("viewModelReferenceId")?).ok()?;
        if segment_index + 1 == property_indices.len() {
            return Some(referenced_view_model_index);
        }
        current_view_model_index = referenced_view_model_index;
    }

    None
}

fn runtime_owned_view_model_numbers(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelNumber> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyNumber").then_some(
                        RuntimeOwnedViewModelNumber {
                            property_index,
                            value: 0.0,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_numbers_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelNumber> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyNumber" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_number_value_for_object(source)?;
                    Some(RuntimeOwnedViewModelNumber {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_numbers(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelNumber>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_numbers_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_booleans(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelBoolean> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyBoolean").then_some(
                        RuntimeOwnedViewModelBoolean {
                            property_index,
                            value: false,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_booleans_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelBoolean> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyBoolean" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_boolean_value_for_object(source)?;
                    Some(RuntimeOwnedViewModelBoolean {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_booleans(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelBoolean>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_booleans_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_strings(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelString> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyString").then_some(
                        RuntimeOwnedViewModelString {
                            property_index,
                            value: Vec::new(),
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_strings_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelString> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyString" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_string_value_for_object(source)?;
                    Some(RuntimeOwnedViewModelString {
                        property_index,
                        value: value.as_bytes().to_vec(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_strings(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelString>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_strings_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_colors(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelColor> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyColor").then_some(
                        RuntimeOwnedViewModelColor {
                            property_index,
                            value: 0xFF000000,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_colors_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelColor> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyColor" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_color_value_for_object(source)?;
                    Some(RuntimeOwnedViewModelColor {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_colors(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelColor>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_colors_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_enums(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelEnum> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    matches!(
                        property.type_name,
                        "ViewModelPropertyEnum"
                            | "ViewModelPropertyEnumCustom"
                            | "ViewModelPropertyEnumSystem"
                    )
                    .then_some(RuntimeOwnedViewModelEnum {
                        property_index,
                        value: 0,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_enums_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelEnum> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyEnumCustom" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_enum_value_index_for_object(source)?;
                    Some(RuntimeOwnedViewModelEnum {
                        property_index,
                        value: u64::try_from(value).ok()?,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_enums(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelEnum>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_enums_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_symbol_list_indices(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelSymbolListIndex> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertySymbolListIndex").then_some(
                        RuntimeOwnedViewModelSymbolListIndex {
                            property_index,
                            value: 0,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_symbol_list_indices_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelSymbolListIndex> {
    let mut values: Vec<RuntimeOwnedViewModelSymbolListIndex> = file
        .view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertySymbolListIndex" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value =
                        file.view_model_instance_symbol_list_index_value_for_object(source)?;
                    Some(RuntimeOwnedViewModelSymbolListIndex {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // C++'s symbol table is last-registration-wins, and imported value order
    // is independent of view-model property order. Move the exact symbol-map
    // winner to the end so component-list index writes target the same value.
    if let Some(property_index) = file
        .view_model_instance_value_for_symbol_object(view_model_instance, 15)
        .and_then(|value| value.uint_property("viewModelPropertyId"))
        .and_then(|property_index| usize::try_from(property_index).ok())
        && let Some(position) = values
            .iter()
            .position(|value| value.property_index == property_index)
    {
        let value = values.remove(position);
        values.push(value);
    }
    values
}

fn runtime_owned_view_model_imported_symbol_list_indices(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelSymbolListIndex>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_symbol_list_indices_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_lists(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelList> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyList").then_some(
                        RuntimeOwnedViewModelList {
                            property_index,
                            value: Rc::new(RefCell::new(RuntimeOwnedViewModelListValue::default())),
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_lists_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelList> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyList" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let (item_count, items) =
                        match file.view_model_instance_source_data_value_for_object(source)? {
                            RuntimeDataValue::List(items) => {
                                let item_count = items.len();
                                let hydrated = items
                                    .into_iter()
                                    .filter_map(|item| {
                                        let reference = file
                                            .referenced_view_model_instance_for_list_item_object(
                                                item,
                                            )?;
                                        runtime_owned_view_model_list_item_instance(file, reference)
                                            .map(|instance| {
                                                RuntimeOwnedViewModelListItem::new(Rc::new(
                                                    RefCell::new(instance),
                                                ))
                                            })
                                    })
                                    .collect::<Vec<_>>();
                                (item_count, hydrated)
                            }
                            _ => (0, Vec::new()),
                        };
                    Some(RuntimeOwnedViewModelList {
                        property_index,
                        value: Rc::new(RefCell::new(RuntimeOwnedViewModelListValue {
                            item_count,
                            items,
                        })),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_list_item_instance(
    file: &RuntimeFile,
    reference: RuntimeViewModelInstanceReference<'_>,
) -> Option<RuntimeOwnedViewModelInstance> {
    thread_local! {
        static HYDRATING: RefCell<BTreeSet<(usize, usize)>> = RefCell::new(BTreeSet::new());
    }
    let key = (reference.view_model_index, reference.instance_index);
    if !HYDRATING.with(|hydrating| hydrating.borrow_mut().insert(key)) {
        return None;
    }
    let instance = RuntimeOwnedViewModelInstance::from_instance(
        file,
        reference.view_model_index,
        reference.instance_index,
    );
    HYDRATING.with(|hydrating| {
        hydrating.borrow_mut().remove(&key);
    });
    instance
}

fn runtime_owned_view_model_imported_lists(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelList>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_lists_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_assets(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelAsset> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    matches!(
                        property.type_name,
                        "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage"
                    )
                    .then_some(RuntimeOwnedViewModelAsset {
                        property_index,
                        value: u64::from(u32::MAX),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_assets_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelAsset> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if !matches!(
                        property.type_name,
                        "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage"
                    ) {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_asset_index_for_object(source)?;
                    Some(RuntimeOwnedViewModelAsset {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_assets(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelAsset>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_assets_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_font_assets(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelFontAsset> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyAssetFont").then_some(
                        RuntimeOwnedViewModelFontAsset {
                            property_index,
                            value: RuntimeFontAssetValue::default(),
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_font_assets_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelFontAsset> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyAssetFont" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let file_asset_index =
                        file.view_model_instance_font_asset_index_for_object(source)?;
                    Some(RuntimeOwnedViewModelFontAsset {
                        property_index,
                        value: RuntimeFontAssetValue::from_file_asset_index(file_asset_index),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_font_assets(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelFontAsset>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_font_assets_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_artboards(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelArtboard> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyArtboard").then_some(
                        RuntimeOwnedViewModelArtboard {
                            property_index,
                            // C++ `ViewModelInstanceArtboardBase` initializes
                            // an unassigned property to its `-1` sentinel.
                            value: u64::from(u32::MAX),
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_artboards_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelArtboard> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyArtboard" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_artboard_index_for_object(source)?;
                    Some(RuntimeOwnedViewModelArtboard {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_artboards(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelArtboard>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_artboards_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_triggers(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelTrigger> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyTrigger").then_some(
                        RuntimeOwnedViewModelTrigger {
                            property_index,
                            value: 0,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_triggers_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelTrigger> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    if property.type_name != "ViewModelPropertyTrigger" {
                        return None;
                    }
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    let source = file.data_context_view_model_property_for_instance(
                        view_model_instance,
                        &path,
                    )?;
                    let value = file.view_model_instance_trigger_count_for_object(source)?;
                    Some(RuntimeOwnedViewModelTrigger {
                        property_index,
                        value,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_imported_triggers(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelTrigger>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_triggers_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_view_model_view_model_property_path_for_names(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &[&str],
) -> Option<Vec<u32>> {
    if property_path.is_empty() {
        return None;
    }

    let mut current_view_model_index = view_model_index;
    let mut path = vec![u32::try_from(view_model_index).ok()?];
    for (segment_index, property_name) in property_path.iter().enumerate() {
        if property_name.is_empty() {
            return None;
        }
        let is_last = segment_index + 1 == property_path.len();
        let view_model = file.view_model(current_view_model_index)?;
        let (property_index, property) =
            view_model
                .properties
                .into_iter()
                .enumerate()
                .find(|(_, property)| {
                    property.type_name == "ViewModelPropertyViewModel"
                        && property.string_property("name") == Some(*property_name)
                })?;
        path.push(u32::try_from(property_index).ok()?);

        if !is_last {
            current_view_model_index = property.uint_property("viewModelReferenceId").and_then(
                |view_model_reference_id| usize::try_from(view_model_reference_id).ok(),
            )?;
        }
    }
    Some(path)
}

pub(crate) fn runtime_view_model_view_model_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    let property_path = property_path.split('/').collect::<Vec<_>>();
    runtime_view_model_view_model_property_path_for_names(file, view_model_index, &property_path)
}

pub(crate) fn runtime_default_view_model_view_model_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_view_model_view_model_property_path_for_names(file, 0, &[property_name])
}

pub(crate) fn runtime_default_view_model_view_model_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_view_model_view_model_property_path_for_name_path(file, 0, property_path)
}

fn runtime_owned_view_model_view_model_children(
    file: &RuntimeFile,
    view_model_index: usize,
    parent_path: &[usize],
    ancestor_view_model_indices: &[usize],
) -> Vec<RuntimeOwnedViewModelViewModel> {
    if ancestor_view_model_indices.contains(&view_model_index) {
        return Vec::new();
    }
    if file.view_model(view_model_index).is_none() {
        return Vec::new();
    }
    let mut child_ancestors = ancestor_view_model_indices.to_vec();
    child_ancestors.push(view_model_index);

    runtime_owned_view_model_property_children(
        file,
        view_model_index,
        None,
        parent_path,
        &child_ancestors,
        true,
    )
}

fn runtime_owned_view_model_view_model_children_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
    parent_path: &[usize],
    ancestor_view_model_indices: &[usize],
) -> Vec<RuntimeOwnedViewModelViewModel> {
    if ancestor_view_model_indices.contains(&view_model_index) {
        return Vec::new();
    }
    let mut child_ancestors = ancestor_view_model_indices.to_vec();
    child_ancestors.push(view_model_index);

    runtime_owned_view_model_property_children(
        file,
        view_model_index,
        Some(view_model_instance),
        parent_path,
        &child_ancestors,
        false,
    )
}

fn runtime_owned_view_model_property_children(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: Option<&RuntimeObject>,
    parent_path: &[usize],
    child_ancestors: &[usize],
    use_generated_defaults: bool,
) -> Vec<RuntimeOwnedViewModelViewModel> {
    let Some(view_model) = file.view_model(view_model_index) else {
        return Vec::new();
    };

    view_model
        .properties
        .into_iter()
        .enumerate()
        .filter_map(|(property_index, property)| {
            if property.type_name != "ViewModelPropertyViewModel" {
                return None;
            }
            let referenced_view_model_index = property
                .uint_property("viewModelReferenceId")
                .and_then(|view_model_reference_id| usize::try_from(view_model_reference_id).ok());
            let referenced_view_model = referenced_view_model_index
                .and_then(|view_model_index| file.view_model(view_model_index));
            let mut path = parent_path.to_vec();
            path.push(property_index);
            let imported_value = view_model_instance
                .and_then(|view_model_instance| {
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    file.data_context_view_model_instance_for_instance(view_model_instance, &path)
                })
                .map(|reference| RuntimeViewModelPointer::Imported {
                    object_id: reference.object.id,
                })
                .or_else(|| {
                    // C++ ViewModelInstanceViewModel::referenceViewModelInstance
                    // reads this serialized index even when the earlier
                    // ArtboardImporter-scoped relationship was unavailable.
                    let view_model_instance = view_model_instance?;
                    let value = file.view_model_instance_value_for_property_id_object(
                        view_model_instance,
                        u32::try_from(property_index).ok()?,
                    )?;
                    if value.type_name != "ViewModelInstanceViewModel" {
                        return None;
                    }
                    let instance_index =
                        usize::try_from(value.uint_property("propertyValue")?).ok()?;
                    let referenced_instance = referenced_view_model
                        .as_ref()?
                        .instances
                        .get(instance_index)?;
                    Some(RuntimeViewModelPointer::Imported {
                        object_id: referenced_instance.object.id,
                    })
                });
            let value = if let Some(value) = imported_value {
                value
            } else if use_generated_defaults && referenced_view_model.is_some() {
                RuntimeViewModelPointer::OwnedGenerated {
                    view_model_index,
                    property_index,
                    path_key: runtime_owned_view_model_path_key(&path),
                }
            } else {
                RuntimeViewModelPointer::Null
            };
            let has_referenced_view_model = referenced_view_model.is_some();
            let view_model_instance_ids = referenced_view_model
                .map(|view_model| {
                    view_model
                        .instances
                        .into_iter()
                        .map(|instance| instance.object.id)
                        .collect()
                })
                .unwrap_or_default();
            let children = if has_referenced_view_model {
                referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_view_model_children(
                            file,
                            view_model_index,
                            &path,
                            &child_ancestors,
                        )
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            let imported_children = referenced_view_model_index
                .and_then(|referenced_view_model_index| {
                    file.view_model(referenced_view_model_index)
                        .map(|view_model| (referenced_view_model_index, view_model))
                })
                .map(|(referenced_view_model_index, view_model)| {
                    view_model
                        .instances
                        .into_iter()
                        .map(|instance| {
                            (
                                instance.object.id,
                                runtime_owned_view_model_view_model_children_for_instance(
                                    file,
                                    referenced_view_model_index,
                                    instance.object,
                                    &path,
                                    child_ancestors,
                                ),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some(RuntimeOwnedViewModelViewModel {
                property_index,
                property_name: property
                    .string_property("name")
                    .unwrap_or_default()
                    .to_owned(),
                value,
                referenced_view_model_index,
                property_names: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_property_names(file, view_model_index)
                    })
                    .unwrap_or_default(),
                numbers: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_numbers(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_numbers: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_numbers(file, view_model_index)
                    })
                    .unwrap_or_default(),
                booleans: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_booleans(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_booleans: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_booleans(file, view_model_index)
                    })
                    .unwrap_or_default(),
                strings: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_strings(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_strings: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_strings(file, view_model_index)
                    })
                    .unwrap_or_default(),
                colors: referenced_view_model_index
                    .map(|view_model_index| runtime_owned_view_model_colors(file, view_model_index))
                    .unwrap_or_default(),
                imported_colors: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_colors(file, view_model_index)
                    })
                    .unwrap_or_default(),
                enums: referenced_view_model_index
                    .map(|view_model_index| runtime_owned_view_model_enums(file, view_model_index))
                    .unwrap_or_default(),
                imported_enums: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_enums(file, view_model_index)
                    })
                    .unwrap_or_default(),
                symbol_list_indices: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_symbol_list_indices(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_symbol_list_indices: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_symbol_list_indices(
                            file,
                            view_model_index,
                        )
                    })
                    .unwrap_or_default(),
                lists: referenced_view_model_index
                    .map(|view_model_index| runtime_owned_view_model_lists(file, view_model_index))
                    .unwrap_or_default(),
                imported_lists: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_lists(file, view_model_index)
                    })
                    .unwrap_or_default(),
                assets: referenced_view_model_index
                    .map(|view_model_index| runtime_owned_view_model_assets(file, view_model_index))
                    .unwrap_or_default(),
                imported_assets: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_assets(file, view_model_index)
                    })
                    .unwrap_or_default(),
                font_assets: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_font_assets(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_font_assets: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_font_assets(file, view_model_index)
                    })
                    .unwrap_or_default(),
                artboards: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_artboards(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_artboards: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_artboards(file, view_model_index)
                    })
                    .unwrap_or_default(),
                triggers: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_triggers(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_triggers: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_triggers(file, view_model_index)
                    })
                    .unwrap_or_default(),
                view_model_instance_ids,
                children,
                imported_children,
            })
        })
        .collect()
}

impl RuntimeOwnedViewModelInstance {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn number_value_by_property_name(&self, property_name: &str) -> Option<f32> {
        let property_index = self.property_index_by_name(property_name)?;
        self.number_value_by_property_index(property_index)
    }

    pub fn color_value_by_property_name(&self, property_name: &str) -> Option<u32> {
        let property_index = self.property_index_by_name(property_name)?;
        self.color_value_by_property_index(property_index)
    }

    /// Resolve a schema property ordinal to its dense numeric-value slot.
    ///
    /// Callers that retain the returned slot may subsequently read and write
    /// the number in O(1) without repeating a name or property scan. The slot
    /// is meaningful only for this view-model schema.
    pub fn number_slot_by_property_index(&self, property_index: usize) -> Option<usize> {
        self.numbers
            .iter()
            .position(|number| number.property_index == property_index)
    }

    /// Read a number through a previously resolved dense numeric-value slot.
    pub fn number_value_by_slot(&self, number_slot: usize) -> Option<f32> {
        self.numbers.get(number_slot).map(|number| number.value)
    }

    pub fn string_value_by_property_name(&self, property_name: &str) -> Option<&[u8]> {
        let property_index = self.property_index_by_name(property_name)?;
        self.string_value_by_property_index(property_index)
    }

    pub fn boolean_value_by_property_name(&self, property_name: &str) -> Option<bool> {
        let property_index = self.property_index_by_name(property_name)?;
        self.boolean_value_by_property_index(property_index)
    }

    pub fn trigger_value_by_property_name(&self, property_name: &str) -> Option<u64> {
        let property_index = self.property_index_by_name(property_name)?;
        self.trigger_value_by_property_index(property_index)
    }

    pub fn nested_view_model_selection_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<(usize, Option<usize>)> {
        let property_index = self.property_index_by_name(property_name)?;
        let view_model = self
            .view_models
            .iter()
            .find(|view_model| view_model.property_index == property_index)?;
        let view_model_index = view_model.referenced_view_model_index?;
        let instance_index = match view_model.value {
            RuntimeViewModelPointer::OwnedGenerated { .. } => None,
            RuntimeViewModelPointer::Imported { object_id } => Some(
                view_model
                    .view_model_instance_ids
                    .iter()
                    .position(|id| *id == object_id)?,
            ),
            RuntimeViewModelPointer::Null | RuntimeViewModelPointer::DataContextRoot => {
                return None;
            }
        };
        Some((view_model_index, instance_index))
    }

    pub(crate) fn instance_identity(&self) -> u64 {
        self.instance_identity
    }

    // Distinguishes separately-created owned instances whose
    // mutation_generation counters would otherwise collide (e.g. binding
    // from_instance(vm, 1) after from_instance(vm, 0), both at generation 0).
    // C++ Artboard::bindViewModelInstance rebinds unconditionally, so the
    // invented change-detection key must never treat two different instances
    // as equal.
    fn next_instance_identity() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        NEXT.fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) fn mutation_generation(&self) -> u64 {
        self.mutation_generation
    }

    fn mark_mutated(&mut self) {
        self.mutation_generation = self.mutation_generation.wrapping_add(1);
    }

    fn track_mutation(&mut self, changed: bool) -> bool {
        if changed {
            self.mark_mutated();
        }
        changed
    }

    /// Advance the values stored directly in this instance and in embedded
    /// view-model properties. C++ `ViewModelInstance::advanced()` also walks
    /// list items; those are shared `Rc` instances in Rust, so they are
    /// returned for recursion after this `RefCell` borrow is released.
    pub(crate) fn advance_script_frame_local(
        &mut self,
    ) -> (bool, Vec<Rc<RefCell<RuntimeOwnedViewModelInstance>>>) {
        let mut shared_children = Vec::new();
        let mut changed = reset_runtime_owned_triggers(&mut self.triggers);
        collect_runtime_owned_list_children(&self.lists, &mut shared_children);
        for view_model in &mut self.view_models {
            changed |= view_model.advance_script_frame(&mut shared_children);
        }
        if changed {
            self.mark_mutated();
        }
        (changed, shared_children)
    }

    pub fn new(file: &RuntimeFile, view_model_index: usize) -> Option<Self> {
        Self::from_view_model(file, view_model_index, None)
    }

    pub fn from_instance(
        file: &RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
    ) -> Option<Self> {
        let view_model = file.view_model(view_model_index)?;
        let instance = view_model.instances.into_iter().nth(instance_index)?;
        Self::from_view_model(file, view_model_index, Some(instance.object))
    }

    pub(crate) fn from_instance_object(
        file: &RuntimeFile,
        view_model_index: usize,
        instance: &RuntimeObject,
    ) -> Option<Self> {
        Self::from_view_model(file, view_model_index, Some(instance))
    }

    fn from_view_model(
        file: &RuntimeFile,
        view_model_index: usize,
        instance: Option<&RuntimeObject>,
    ) -> Option<Self> {
        file.view_model(view_model_index)?;
        let use_generated_defaults = instance.is_none();
        let parent_path = [view_model_index];
        let numbers = instance
            .map(|instance| {
                runtime_owned_view_model_numbers_for_instance(file, view_model_index, instance)
            })
            .unwrap_or_else(|| runtime_owned_view_model_numbers(file, view_model_index));
        let booleans = instance
            .map(|instance| {
                runtime_owned_view_model_booleans_for_instance(file, view_model_index, instance)
            })
            .unwrap_or_else(|| runtime_owned_view_model_booleans(file, view_model_index));
        let strings = instance
            .map(|instance| {
                runtime_owned_view_model_strings_for_instance(file, view_model_index, instance)
            })
            .unwrap_or_else(|| runtime_owned_view_model_strings(file, view_model_index));
        let colors = instance
            .map(|instance| {
                runtime_owned_view_model_colors_for_instance(file, view_model_index, instance)
            })
            .unwrap_or_else(|| runtime_owned_view_model_colors(file, view_model_index));
        let enums = instance
            .map(|instance| {
                runtime_owned_view_model_enums_for_instance(file, view_model_index, instance)
            })
            .unwrap_or_else(|| runtime_owned_view_model_enums(file, view_model_index));
        let symbol_list_indices = instance
            .map(|instance| {
                runtime_owned_view_model_symbol_list_indices_for_instance(
                    file,
                    view_model_index,
                    instance,
                )
            })
            .unwrap_or_else(|| {
                runtime_owned_view_model_symbol_list_indices(file, view_model_index)
            });
        let lists = instance
            .map(|instance| {
                runtime_owned_view_model_lists_for_instance(file, view_model_index, instance)
            })
            .unwrap_or_else(|| runtime_owned_view_model_lists(file, view_model_index));
        let assets = instance
            .map(|instance| {
                runtime_owned_view_model_assets_for_instance(file, view_model_index, instance)
            })
            .unwrap_or_else(|| runtime_owned_view_model_assets(file, view_model_index));
        let font_assets = instance
            .map(|instance| {
                runtime_owned_view_model_font_assets_for_instance(file, view_model_index, instance)
            })
            .unwrap_or_else(|| runtime_owned_view_model_font_assets(file, view_model_index));
        let artboards = instance
            .map(|instance| {
                runtime_owned_view_model_artboards_for_instance(file, view_model_index, instance)
            })
            .unwrap_or_else(|| runtime_owned_view_model_artboards(file, view_model_index));
        let triggers = instance
            .map(|instance| {
                runtime_owned_view_model_triggers_for_instance(file, view_model_index, instance)
            })
            .unwrap_or_else(|| runtime_owned_view_model_triggers(file, view_model_index));
        let view_models = runtime_owned_view_model_property_children(
            file,
            view_model_index,
            instance,
            &parent_path,
            &[view_model_index],
            use_generated_defaults,
        );
        Some(Self {
            view_model_index,
            instance_identity: Self::next_instance_identity(),
            mutation_generation: 0,
            property_names: runtime_owned_view_model_property_names(file, view_model_index),
            numbers,
            booleans,
            strings,
            colors,
            enums,
            symbol_list_indices,
            lists,
            assets,
            font_assets,
            artboards,
            triggers,
            view_models,
        })
    }

    fn property_index_by_name(&self, property_name: &str) -> Option<usize> {
        runtime_owned_view_model_property_index_by_name(&self.property_names, property_name)
    }

    pub fn set_number_by_property_index(&mut self, property_index: usize, value: f32) -> bool {
        let Some(number) = self
            .numbers
            .iter_mut()
            .find(|number| number.property_index == property_index)
        else {
            return false;
        };
        if number.value == value {
            return false;
        }
        number.value = value;
        self.mark_mutated();
        true
    }

    /// Write a number through a previously resolved dense numeric-value slot.
    ///
    /// This is the direct hot-path counterpart to
    /// [`Self::number_slot_by_property_index`]. It performs no allocation or
    /// schema/name lookup.
    pub fn set_number_by_slot(&mut self, number_slot: usize, value: f32) -> bool {
        let Some(number) = self.numbers.get_mut(number_slot) else {
            return false;
        };
        if number.value == value {
            return false;
        }
        number.value = value;
        self.mark_mutated();
        true
    }

    pub fn set_number_by_property_name(&mut self, property_name: &str, value: f32) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_number_by_property_index(property_index, value)
    }

    pub fn number_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelNumberSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.numbers
            .iter()
            .any(|number| number.property_index == property_index)
            .then_some(RuntimeOwnedViewModelNumberSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn number_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelNumberSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.number_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelNumberSourceHandle { property_path })
    }

    pub fn set_number_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelNumberSourceHandle,
        value: f32,
    ) -> bool {
        self.set_number_by_property_path(&handle.property_path, value)
    }

    pub fn set_number_by_property_name_path(&mut self, property_path: &str, value: f32) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_number_by_property_names(&property_path, value)
    }

    pub fn set_number_by_property_names(&mut self, property_path: &[&str], value: f32) -> bool {
        if property_path.len() == 1 {
            return self.set_number_by_property_name(property_path[0], value);
        }
        let Some((number_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_number_by_property_name(number_name, value);
        self.track_mutation(changed)
    }

    pub(crate) fn set_number_by_property_path(
        &mut self,
        property_path: &[usize],
        value: f32,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_number_by_property_index(property_path[0], value);
        }
        let Some((number_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_number_by_property_index(*number_index, value);
        self.track_mutation(changed)
    }

    fn number_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .numbers
                .iter()
                .any(|number| number.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (number_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(number_name)?;
        if !view_model
            .numbers
            .iter()
            .any(|number| number.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_boolean_by_property_index(&mut self, property_index: usize, value: bool) -> bool {
        let Some(boolean) = self
            .booleans
            .iter_mut()
            .find(|boolean| boolean.property_index == property_index)
        else {
            return false;
        };
        if boolean.value == value {
            return false;
        }
        boolean.value = value;
        self.mark_mutated();
        true
    }

    pub fn set_boolean_by_property_name(&mut self, property_name: &str, value: bool) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_boolean_by_property_index(property_index, value)
    }

    pub fn boolean_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelBooleanSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.booleans
            .iter()
            .any(|boolean| boolean.property_index == property_index)
            .then_some(RuntimeOwnedViewModelBooleanSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn boolean_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelBooleanSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.boolean_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelBooleanSourceHandle { property_path })
    }

    pub fn set_boolean_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelBooleanSourceHandle,
        value: bool,
    ) -> bool {
        self.set_boolean_by_property_path(&handle.property_path, value)
    }

    pub fn set_boolean_by_property_name_path(&mut self, property_path: &str, value: bool) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_boolean_by_property_names(&property_path, value)
    }

    pub fn set_boolean_by_property_names(&mut self, property_path: &[&str], value: bool) -> bool {
        if property_path.len() == 1 {
            return self.set_boolean_by_property_name(property_path[0], value);
        }
        let Some((boolean_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_boolean_by_property_name(boolean_name, value);
        self.track_mutation(changed)
    }

    pub(crate) fn set_boolean_by_property_path(
        &mut self,
        property_path: &[usize],
        value: bool,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_boolean_by_property_index(property_path[0], value);
        }
        let Some((boolean_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_boolean_by_property_index(*boolean_index, value);
        self.track_mutation(changed)
    }

    fn boolean_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .booleans
                .iter()
                .any(|boolean| boolean.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (boolean_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(boolean_name)?;
        if !view_model
            .booleans
            .iter()
            .any(|boolean| boolean.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_string_by_property_index(&mut self, property_index: usize, value: &[u8]) -> bool {
        let Some(string) = self
            .strings
            .iter_mut()
            .find(|string| string.property_index == property_index)
        else {
            return false;
        };
        if string.value == value {
            return false;
        }
        string.value = value.to_vec();
        self.mark_mutated();
        true
    }

    pub fn set_string_by_property_name(&mut self, property_name: &str, value: &[u8]) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_string_by_property_index(property_index, value)
    }

    pub fn string_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelStringSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.strings
            .iter()
            .any(|string| string.property_index == property_index)
            .then_some(RuntimeOwnedViewModelStringSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn string_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelStringSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.string_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelStringSourceHandle { property_path })
    }

    pub fn set_string_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelStringSourceHandle,
        value: &[u8],
    ) -> bool {
        self.set_string_by_property_path(&handle.property_path, value)
    }

    pub fn set_string_by_property_name_path(&mut self, property_path: &str, value: &[u8]) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_string_by_property_names(&property_path, value)
    }

    pub fn set_string_by_property_names(&mut self, property_path: &[&str], value: &[u8]) -> bool {
        if property_path.len() == 1 {
            return self.set_string_by_property_name(property_path[0], value);
        }
        let Some((string_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_string_by_property_name(string_name, value);
        self.track_mutation(changed)
    }

    pub(crate) fn set_string_by_property_path(
        &mut self,
        property_path: &[usize],
        value: &[u8],
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_string_by_property_index(property_path[0], value);
        }
        let Some((string_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_string_by_property_index(*string_index, value);
        self.track_mutation(changed)
    }

    fn string_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .strings
                .iter()
                .any(|string| string.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (string_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(string_name)?;
        if !view_model
            .strings
            .iter()
            .any(|string| string.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_color_by_property_index(&mut self, property_index: usize, value: u32) -> bool {
        let Some(color) = self
            .colors
            .iter_mut()
            .find(|color| color.property_index == property_index)
        else {
            return false;
        };
        if color.value == value {
            return false;
        }
        color.value = value;
        self.mark_mutated();
        true
    }

    pub fn set_color_by_property_name(&mut self, property_name: &str, value: u32) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_color_by_property_index(property_index, value)
    }

    pub fn color_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelColorSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.colors
            .iter()
            .any(|color| color.property_index == property_index)
            .then_some(RuntimeOwnedViewModelColorSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn color_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelColorSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.color_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelColorSourceHandle { property_path })
    }

    pub fn set_color_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelColorSourceHandle,
        value: u32,
    ) -> bool {
        match self.color_value_by_property_path(&handle.property_path) {
            Some(current) if current == value => return false,
            Some(_) => {}
            None => return false,
        }
        self.set_color_by_property_path(&handle.property_path, value)
    }

    pub fn set_color_by_property_name_path(&mut self, property_path: &str, value: u32) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_color_by_property_names(&property_path, value)
    }

    pub fn set_color_by_property_names(&mut self, property_path: &[&str], value: u32) -> bool {
        if property_path.len() == 1 {
            return self.set_color_by_property_name(property_path[0], value);
        }
        let Some((color_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_color_by_property_name(color_name, value);
        self.track_mutation(changed)
    }

    pub(crate) fn set_color_by_property_path(
        &mut self,
        property_path: &[usize],
        value: u32,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_color_by_property_index(property_path[0], value);
        }
        let Some((color_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_color_by_property_index(*color_index, value);
        self.track_mutation(changed)
    }

    fn color_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .colors
                .iter()
                .any(|color| color.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (color_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(color_name)?;
        if !view_model
            .colors
            .iter()
            .any(|color| color.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_enum_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(enum_value) = self
            .enums
            .iter_mut()
            .find(|enum_value| enum_value.property_index == property_index)
        else {
            return false;
        };
        if enum_value.value == value {
            return false;
        }
        enum_value.value = value;
        self.mark_mutated();
        true
    }

    pub fn set_enum_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_enum_by_property_index(property_index, value)
    }

    pub fn enum_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelEnumSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.enums
            .iter()
            .any(|enum_value| enum_value.property_index == property_index)
            .then_some(RuntimeOwnedViewModelEnumSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn enum_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelEnumSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.enum_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelEnumSourceHandle { property_path })
    }

    pub fn set_enum_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelEnumSourceHandle,
        value: u64,
    ) -> bool {
        self.set_enum_by_property_path(&handle.property_path, value)
    }

    pub fn set_enum_by_property_name_path(&mut self, property_path: &str, value: u64) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_enum_by_property_names(&property_path, value)
    }

    pub fn set_enum_by_property_names(&mut self, property_path: &[&str], value: u64) -> bool {
        if property_path.len() == 1 {
            return self.set_enum_by_property_name(property_path[0], value);
        }
        let Some((enum_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_enum_by_property_name(enum_name, value);
        self.track_mutation(changed)
    }

    pub(crate) fn set_enum_by_property_path(
        &mut self,
        property_path: &[usize],
        value: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_enum_by_property_index(property_path[0], value);
        }
        let Some((enum_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_enum_by_property_index(*enum_index, value);
        self.track_mutation(changed)
    }

    fn enum_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .enums
                .iter()
                .any(|enum_value| enum_value.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (enum_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(enum_name)?;
        if !view_model
            .enums
            .iter()
            .any(|enum_value| enum_value.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_symbol_list_index_by_property_index(
        &mut self,
        property_index: usize,
        value: u64,
    ) -> bool {
        let Some(symbol_list_index) = self
            .symbol_list_indices
            .iter_mut()
            .find(|symbol_list_index| symbol_list_index.property_index == property_index)
        else {
            return false;
        };
        if symbol_list_index.value == value {
            return false;
        }
        symbol_list_index.value = value;
        self.mark_mutated();
        true
    }

    pub fn set_symbol_list_index_by_property_name(
        &mut self,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_symbol_list_index_by_property_index(property_index, value)
    }

    pub fn symbol_list_index_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelSymbolListIndexSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.symbol_list_indices
            .iter()
            .any(|symbol_list_index| symbol_list_index.property_index == property_index)
            .then_some(RuntimeOwnedViewModelSymbolListIndexSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn symbol_list_index_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelSymbolListIndexSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.symbol_list_index_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelSymbolListIndexSourceHandle { property_path })
    }

    pub fn set_symbol_list_index_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelSymbolListIndexSourceHandle,
        value: u64,
    ) -> bool {
        self.set_symbol_list_index_by_property_path(&handle.property_path, value)
    }

    pub fn set_symbol_list_index_by_property_name_path(
        &mut self,
        property_path: &str,
        value: u64,
    ) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_symbol_list_index_by_property_names(&property_path, value)
    }

    pub fn set_symbol_list_index_by_property_names(
        &mut self,
        property_path: &[&str],
        value: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_symbol_list_index_by_property_name(property_path[0], value);
        }
        let Some((symbol_list_index_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed =
            view_model.set_symbol_list_index_by_property_name(symbol_list_index_name, value);
        self.track_mutation(changed)
    }

    pub(crate) fn set_symbol_list_index_by_property_path(
        &mut self,
        property_path: &[usize],
        value: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_symbol_list_index_by_property_index(property_path[0], value);
        }
        let Some((symbol_list_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_symbol_list_index_by_property_index(*symbol_list_index, value);
        self.track_mutation(changed)
    }

    fn symbol_list_index_property_path_by_names(
        &self,
        property_path: &[&str],
    ) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .symbol_list_indices
                .iter()
                .any(|symbol_list_index| symbol_list_index.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (symbol_list_index_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(symbol_list_index_name)?;
        if !view_model
            .symbol_list_indices
            .iter()
            .any(|symbol_list_index| symbol_list_index.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_list_item_count_by_property_index(
        &mut self,
        property_index: usize,
        item_count: usize,
    ) -> bool {
        let Some(list) = self
            .lists
            .iter_mut()
            .find(|list| list.property_index == property_index)
        else {
            return false;
        };
        let mut value = list.value.borrow_mut();
        if value.item_count == item_count {
            return false;
        }
        value.item_count = item_count;
        value.items.truncate(item_count);
        drop(value);
        self.mark_mutated();
        true
    }

    pub fn set_list_item_count_by_property_name(
        &mut self,
        property_name: &str,
        item_count: usize,
    ) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_list_item_count_by_property_index(property_index, item_count)
    }

    pub fn list_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelListSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.lists
            .iter()
            .any(|list| list.property_index == property_index)
            .then_some(RuntimeOwnedViewModelListSourceHandle {
                property_path: vec![property_index],
            })
    }

    fn list_value_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<Rc<RefCell<RuntimeOwnedViewModelListValue>>> {
        let property_index = self.property_index_by_name(property_name)?;
        self.lists
            .iter()
            .find(|list| list.property_index == property_index)
            .map(|list| Rc::clone(&list.value))
    }

    pub(crate) fn list_items_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<Vec<Rc<RefCell<RuntimeOwnedViewModelInstance>>>> {
        Some(
            self.list_value_by_property_name(property_name)?
                .borrow()
                .items
                .iter()
                .map(|item| Rc::clone(&item.instance))
                .collect(),
        )
    }

    pub(crate) fn script_list_children(&self) -> Vec<Rc<RefCell<RuntimeOwnedViewModelInstance>>> {
        let mut children = Vec::new();
        collect_runtime_owned_list_children(&self.lists, &mut children);
        children
    }

    pub(crate) fn push_list_item_by_property_name(
        &mut self,
        property_name: &str,
        item: Rc<RefCell<RuntimeOwnedViewModelInstance>>,
    ) -> bool {
        let Some(list) = self.list_value_by_property_name(property_name) else {
            return false;
        };
        let mut list = list.borrow_mut();
        list.items.push(RuntimeOwnedViewModelListItem::new(item));
        list.item_count = list.items.len();
        drop(list);
        self.mark_mutated();
        true
    }

    pub(crate) fn insert_list_item_by_property_name(
        &mut self,
        property_name: &str,
        index: usize,
        item: Rc<RefCell<RuntimeOwnedViewModelInstance>>,
    ) -> bool {
        let Some(list) = self.list_value_by_property_name(property_name) else {
            return false;
        };
        let mut list = list.borrow_mut();
        let index = index.min(list.items.len());
        list.items
            .insert(index, RuntimeOwnedViewModelListItem::new(item));
        list.item_count = list.items.len();
        drop(list);
        self.mark_mutated();
        true
    }

    pub(crate) fn pop_list_item_by_property_name(
        &mut self,
        property_name: &str,
    ) -> Option<Rc<RefCell<RuntimeOwnedViewModelInstance>>> {
        let list = self.list_value_by_property_name(property_name)?;
        let mut list = list.borrow_mut();
        let item = list.items.pop()?;
        list.item_count = list.items.len();
        drop(list);
        self.mark_mutated();
        Some(item.instance)
    }

    pub(crate) fn shift_list_item_by_property_name(
        &mut self,
        property_name: &str,
    ) -> Option<Rc<RefCell<RuntimeOwnedViewModelInstance>>> {
        let list = self.list_value_by_property_name(property_name)?;
        let mut list = list.borrow_mut();
        if list.items.is_empty() {
            return None;
        }
        let item = list.items.remove(0);
        list.item_count = list.items.len();
        drop(list);
        self.mark_mutated();
        Some(item.instance)
    }

    pub(crate) fn swap_list_items_by_property_name(
        &mut self,
        property_name: &str,
        first: usize,
        second: usize,
    ) -> bool {
        let Some(list) = self.list_value_by_property_name(property_name) else {
            return false;
        };
        let mut list = list.borrow_mut();
        if first >= list.items.len() || second >= list.items.len() || first == second {
            return false;
        }
        list.items.swap(first, second);
        drop(list);
        self.mark_mutated();
        true
    }

    pub(crate) fn clear_list_items_by_property_name(&mut self, property_name: &str) -> bool {
        let Some(list) = self.list_value_by_property_name(property_name) else {
            return false;
        };
        let mut list = list.borrow_mut();
        if list.items.is_empty() && list.item_count == 0 {
            return false;
        }
        list.items.clear();
        list.item_count = 0;
        drop(list);
        self.mark_mutated();
        true
    }

    pub(crate) fn remove_list_item_at_by_property_name(
        &mut self,
        property_name: &str,
        index: usize,
    ) -> bool {
        let Some(list) = self.list_value_by_property_name(property_name) else {
            return false;
        };
        let mut list = list.borrow_mut();
        if index >= list.items.len() {
            return false;
        }
        list.items.remove(index);
        list.item_count = list.items.len();
        drop(list);
        self.mark_mutated();
        true
    }

    pub(crate) fn remove_list_items_by_identity(
        &mut self,
        property_name: &str,
        item: &Rc<RefCell<RuntimeOwnedViewModelInstance>>,
        remove_all: bool,
    ) -> bool {
        let Some(list) = self.list_value_by_property_name(property_name) else {
            return false;
        };
        let mut list = list.borrow_mut();
        let old_len = list.items.len();
        if remove_all {
            list.items
                .retain(|candidate| !Rc::ptr_eq(&candidate.instance, item));
        } else if let Some(index) = list
            .items
            .iter()
            .position(|candidate| Rc::ptr_eq(&candidate.instance, item))
        {
            list.items.remove(index);
        }
        if list.items.len() == old_len {
            return false;
        }
        list.item_count = list.items.len();
        drop(list);
        self.mark_mutated();
        true
    }

    pub fn list_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelListSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.list_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelListSourceHandle { property_path })
    }

    pub fn set_list_item_count_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelListSourceHandle,
        item_count: usize,
    ) -> bool {
        self.set_list_item_count_by_property_path(&handle.property_path, item_count)
    }

    pub fn set_list_item_count_by_property_name_path(
        &mut self,
        property_path: &str,
        item_count: usize,
    ) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_list_item_count_by_property_names(&property_path, item_count)
    }

    pub fn set_list_item_count_by_property_names(
        &mut self,
        property_path: &[&str],
        item_count: usize,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_list_item_count_by_property_name(property_path[0], item_count);
        }
        let Some((list_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_list_item_count_by_property_name(list_name, item_count);
        self.track_mutation(changed)
    }

    pub(crate) fn set_list_item_count_by_property_path(
        &mut self,
        property_path: &[usize],
        item_count: usize,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_list_item_count_by_property_index(property_path[0], item_count);
        }
        let Some((list_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_list_item_count_by_property_index(*list_index, item_count);
        self.track_mutation(changed)
    }

    fn list_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .lists
                .iter()
                .any(|list| list.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (list_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(list_name)?;
        if !view_model
            .lists
            .iter()
            .any(|list| list.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_asset_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(asset) = self
            .assets
            .iter_mut()
            .find(|asset| asset.property_index == property_index)
        else {
            return false;
        };
        if asset.value == value {
            return false;
        }
        asset.value = value;
        self.mark_mutated();
        true
    }

    pub fn set_asset_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_asset_by_property_index(property_index, value)
    }

    pub fn asset_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelAssetSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.assets
            .iter()
            .any(|asset| asset.property_index == property_index)
            .then_some(RuntimeOwnedViewModelAssetSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn asset_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelAssetSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.asset_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelAssetSourceHandle { property_path })
    }

    pub fn set_asset_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelAssetSourceHandle,
        value: u64,
    ) -> bool {
        self.set_asset_by_property_path(&handle.property_path, value)
    }

    pub fn set_asset_by_property_name_path(&mut self, property_path: &str, value: u64) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_asset_by_property_names(&property_path, value)
    }

    pub fn set_asset_by_property_names(&mut self, property_path: &[&str], value: u64) -> bool {
        if property_path.len() == 1 {
            return self.set_asset_by_property_name(property_path[0], value);
        }
        let Some((asset_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_asset_by_property_name(asset_name, value);
        self.track_mutation(changed)
    }

    pub(crate) fn set_asset_by_property_path(
        &mut self,
        property_path: &[usize],
        value: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_asset_by_property_index(property_path[0], value);
        }
        let Some((asset_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_asset_by_property_index(*asset_index, value);
        self.track_mutation(changed)
    }

    fn asset_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .assets
                .iter()
                .any(|asset| asset.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (asset_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(asset_name)?;
        if !view_model
            .assets
            .iter()
            .any(|asset| asset.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn font_asset_value_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<&RuntimeFontAssetValue> {
        let property_index = self.property_index_by_name(property_name)?;
        self.font_asset_value_by_property_index(property_index)
    }

    pub fn set_font_asset_index_by_property_index(
        &mut self,
        property_index: usize,
        file_asset_index: u64,
    ) -> bool {
        let Some(asset) = self
            .font_assets
            .iter_mut()
            .find(|asset| asset.property_index == property_index)
        else {
            return false;
        };
        let changed = asset.value.set_file_asset_index(file_asset_index);
        self.track_mutation(changed)
    }

    pub fn set_font_asset_index_by_property_name(
        &mut self,
        property_name: &str,
        file_asset_index: u64,
    ) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_font_asset_index_by_property_index(property_index, file_asset_index)
    }

    pub fn set_live_font_bytes_by_property_index(
        &mut self,
        property_index: usize,
        font_bytes: Option<Arc<[u8]>>,
    ) -> bool {
        let Some(asset) = self
            .font_assets
            .iter_mut()
            .find(|asset| asset.property_index == property_index)
        else {
            return false;
        };
        let changed = asset.value.set_live_font_bytes(font_bytes);
        self.track_mutation(changed)
    }

    pub fn set_live_font_bytes_by_property_name(
        &mut self,
        property_name: &str,
        font_bytes: Option<Arc<[u8]>>,
    ) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_live_font_bytes_by_property_index(property_index, font_bytes)
    }

    pub fn font_asset_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelFontAssetSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.font_assets
            .iter()
            .any(|asset| asset.property_index == property_index)
            .then_some(RuntimeOwnedViewModelFontAssetSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn font_asset_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelFontAssetSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.font_asset_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelFontAssetSourceHandle { property_path })
    }

    pub fn set_font_asset_index_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelFontAssetSourceHandle,
        file_asset_index: u64,
    ) -> bool {
        self.set_font_asset_index_by_property_path(&handle.property_path, file_asset_index)
    }

    pub fn set_live_font_bytes_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelFontAssetSourceHandle,
        font_bytes: Option<Arc<[u8]>>,
    ) -> bool {
        self.set_live_font_bytes_by_property_path(&handle.property_path, font_bytes)
    }

    pub fn set_font_asset_index_by_property_name_path(
        &mut self,
        property_path: &str,
        file_asset_index: u64,
    ) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_font_asset_index_by_property_names(&property_path, file_asset_index)
    }

    pub fn set_live_font_bytes_by_property_name_path(
        &mut self,
        property_path: &str,
        font_bytes: Option<Arc<[u8]>>,
    ) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_live_font_bytes_by_property_names(&property_path, font_bytes)
    }

    fn set_font_asset_index_by_property_names(
        &mut self,
        property_path: &[&str],
        file_asset_index: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_font_asset_index_by_property_name(property_path[0], file_asset_index);
        }
        let Some((asset_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed =
            view_model.set_font_asset_index_by_property_name(asset_name, file_asset_index);
        self.track_mutation(changed)
    }

    fn set_live_font_bytes_by_property_names(
        &mut self,
        property_path: &[&str],
        font_bytes: Option<Arc<[u8]>>,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_live_font_bytes_by_property_name(property_path[0], font_bytes);
        }
        let Some((asset_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_live_font_bytes_by_property_name(asset_name, font_bytes);
        self.track_mutation(changed)
    }

    pub(crate) fn set_font_asset_index_by_property_path(
        &mut self,
        property_path: &[usize],
        file_asset_index: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_font_asset_index_by_property_index(property_path[0], file_asset_index);
        }
        let Some((asset_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed =
            view_model.set_font_asset_index_by_property_index(*asset_index, file_asset_index);
        self.track_mutation(changed)
    }

    pub(crate) fn set_live_font_bytes_by_property_path(
        &mut self,
        property_path: &[usize],
        font_bytes: Option<Arc<[u8]>>,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_live_font_bytes_by_property_index(property_path[0], font_bytes);
        }
        let Some((asset_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_live_font_bytes_by_property_index(*asset_index, font_bytes);
        self.track_mutation(changed)
    }

    fn font_asset_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .font_assets
                .iter()
                .any(|asset| asset.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (asset_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(asset_name)?;
        if !view_model
            .font_assets
            .iter()
            .any(|asset| asset.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_artboard_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(artboard) = self
            .artboards
            .iter_mut()
            .find(|artboard| artboard.property_index == property_index)
        else {
            return false;
        };
        if artboard.value == value {
            return false;
        }
        artboard.value = value;
        self.mark_mutated();
        true
    }

    pub fn set_artboard_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_artboard_by_property_index(property_index, value)
    }

    pub fn artboard_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelArtboardSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.artboards
            .iter()
            .any(|artboard| artboard.property_index == property_index)
            .then_some(RuntimeOwnedViewModelArtboardSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn artboard_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelArtboardSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.artboard_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelArtboardSourceHandle { property_path })
    }

    pub fn set_artboard_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelArtboardSourceHandle,
        value: u64,
    ) -> bool {
        self.set_artboard_by_property_path(&handle.property_path, value)
    }

    pub fn set_artboard_by_property_name_path(&mut self, property_path: &str, value: u64) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_artboard_by_property_names(&property_path, value)
    }

    pub fn set_artboard_by_property_names(&mut self, property_path: &[&str], value: u64) -> bool {
        if property_path.len() == 1 {
            return self.set_artboard_by_property_name(property_path[0], value);
        }
        let Some((artboard_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_artboard_by_property_name(artboard_name, value);
        self.track_mutation(changed)
    }

    pub(crate) fn set_artboard_by_property_path(
        &mut self,
        property_path: &[usize],
        value: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_artboard_by_property_index(property_path[0], value);
        }
        let Some((artboard_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_artboard_by_property_index(*artboard_index, value);
        self.track_mutation(changed)
    }

    fn artboard_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .artboards
                .iter()
                .any(|artboard| artboard.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (artboard_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(artboard_name)?;
        if !view_model
            .artboards
            .iter()
            .any(|artboard| artboard.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_trigger_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(trigger) = self
            .triggers
            .iter_mut()
            .find(|trigger| trigger.property_index == property_index)
        else {
            return false;
        };
        if trigger.value == value {
            return false;
        }
        trigger.value = value;
        self.mark_mutated();
        true
    }

    pub fn set_trigger_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_trigger_by_property_index(property_index, value)
    }

    pub fn trigger_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelTriggerSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.triggers
            .iter()
            .any(|trigger| trigger.property_index == property_index)
            .then_some(RuntimeOwnedViewModelTriggerSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn trigger_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelTriggerSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let property_path = self.trigger_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelTriggerSourceHandle { property_path })
    }

    pub fn set_trigger_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelTriggerSourceHandle,
        value: u64,
    ) -> bool {
        self.set_trigger_by_property_path(&handle.property_path, value)
    }

    pub fn set_trigger_by_property_name_path(&mut self, property_path: &str, value: u64) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_trigger_by_property_names(&property_path, value)
    }

    pub fn set_trigger_by_property_names(&mut self, property_path: &[&str], value: u64) -> bool {
        if property_path.len() == 1 {
            return self.set_trigger_by_property_name(property_path[0], value);
        }
        let Some((trigger_name, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_names_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_trigger_by_property_name(trigger_name, value);
        self.track_mutation(changed)
    }

    pub(crate) fn set_trigger_by_property_path(
        &mut self,
        property_path: &[usize],
        value: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_trigger_by_property_index(property_path[0], value);
        }
        let Some((trigger_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return false;
        }
        let changed = view_model.set_trigger_by_property_index(*trigger_index, value);
        self.track_mutation(changed)
    }

    fn trigger_property_path_by_names(&self, property_path: &[&str]) -> Option<Vec<usize>> {
        if property_path.len() == 1 {
            let property_index = self.property_index_by_name(property_path[0])?;
            return self
                .triggers
                .iter()
                .any(|trigger| trigger.property_index == property_index)
                .then_some(vec![property_index]);
        }

        let (trigger_name, view_model_names) = property_path.split_last()?;
        let (view_model_path, view_model) =
            self.view_model_property_path_by_names(view_model_names)?;
        if !matches!(
            view_model.value,
            RuntimeViewModelPointer::OwnedGenerated { .. }
        ) {
            return None;
        }
        let property_index = view_model.property_index_by_name(trigger_name)?;
        if !view_model
            .triggers
            .iter()
            .any(|trigger| trigger.property_index == property_index)
        {
            return None;
        }
        let mut property_path = view_model_path;
        property_path.push(property_index);
        Some(property_path)
    }

    pub fn set_view_model_by_property_index(
        &mut self,
        property_index: usize,
        instance_index: usize,
    ) -> bool {
        self.set_view_model_by_property_path(&[property_index], instance_index)
    }

    pub fn view_model_source_handle_by_property_name(
        &self,
        property_name: &str,
    ) -> Option<RuntimeOwnedViewModelViewModelSourceHandle> {
        let property_index = self.property_index_by_name(property_name)?;
        self.view_models
            .iter()
            .any(|view_model| view_model.property_index == property_index)
            .then_some(RuntimeOwnedViewModelViewModelSourceHandle {
                property_path: vec![property_index],
            })
    }

    pub fn view_model_source_handle_by_property_name_path(
        &self,
        property_path: &str,
    ) -> Option<RuntimeOwnedViewModelViewModelSourceHandle> {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return None;
        }
        let (property_path, _) = self.view_model_property_path_by_names(&property_path)?;
        Some(RuntimeOwnedViewModelViewModelSourceHandle { property_path })
    }

    pub fn set_view_model_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelViewModelSourceHandle,
        instance_index: usize,
    ) -> bool {
        self.set_view_model_by_property_path(&handle.property_path, instance_index)
    }

    pub fn set_view_model_by_property_path(
        &mut self,
        property_path: &[usize],
        instance_index: usize,
    ) -> bool {
        let Some(view_model) = self.relinkable_view_model_by_property_path_mut(property_path)
        else {
            return false;
        };
        let Some(object_id) = view_model
            .view_model_instance_ids
            .get(instance_index)
            .copied()
        else {
            return false;
        };
        let value = RuntimeViewModelPointer::Imported { object_id };
        if view_model.value == value {
            return false;
        }
        view_model.value = value;
        self.mark_mutated();
        true
    }

    pub fn set_view_model_by_property_name_path(
        &mut self,
        property_path: &str,
        instance_index: usize,
    ) -> bool {
        let property_path = property_path.split('/').collect::<Vec<_>>();
        if property_path.is_empty() || property_path.iter().any(|segment| segment.is_empty()) {
            return false;
        }
        self.set_view_model_by_property_names(&property_path, instance_index)
    }

    pub fn set_view_model_by_property_names(
        &mut self,
        property_path: &[&str],
        instance_index: usize,
    ) -> bool {
        let Some(view_model) = self.view_model_by_property_names_mut(property_path) else {
            return false;
        };
        let Some(object_id) = view_model
            .view_model_instance_ids
            .get(instance_index)
            .copied()
        else {
            return false;
        };
        let value = RuntimeViewModelPointer::Imported { object_id };
        if view_model.value == value {
            return false;
        }
        view_model.value = value;
        self.mark_mutated();
        true
    }

    fn view_model_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<&RuntimeOwnedViewModelViewModel> {
        let (property_index, rest) = property_path.split_first()?;
        let mut view_model = self
            .view_models
            .iter()
            .find(|view_model| view_model.property_index == *property_index)?;
        for property_index in rest {
            view_model = view_model
                .active_children()?
                .iter()
                .find(|view_model| view_model.property_index == *property_index)?;
        }
        Some(view_model)
    }

    fn combined_context_source_property_index(
        context_path: &[usize],
        source_tail: &[u32],
        index: usize,
    ) -> Option<usize> {
        if index < context_path.len() {
            Some(context_path[index])
        } else {
            usize::try_from(*source_tail.get(index - context_path.len())?).ok()
        }
    }

    fn view_model_by_context_source_property_prefix(
        &self,
        context_path: &[usize],
        source_tail: &[u32],
        prefix_len: usize,
    ) -> Option<&RuntimeOwnedViewModelViewModel> {
        if prefix_len == 0 {
            return None;
        }
        let first_property_index =
            Self::combined_context_source_property_index(context_path, source_tail, 0)?;
        let mut view_model = self
            .view_models
            .iter()
            .find(|view_model| view_model.property_index == first_property_index)?;
        for index in 1..prefix_len {
            let property_index =
                Self::combined_context_source_property_index(context_path, source_tail, index)?;
            view_model = view_model
                .active_children()?
                .iter()
                .find(|view_model| view_model.property_index == property_index)?;
        }
        Some(view_model)
    }

    fn context_source_value_target(
        &self,
        context_path: &[usize],
        source_path: &[u32],
    ) -> Option<(Option<&RuntimeOwnedViewModelViewModel>, usize)> {
        if source_path.is_empty() {
            return None;
        }
        let view_model_index = self.view_model_index_by_property_path(context_path)?;
        if usize::try_from(source_path[0]).ok()? != view_model_index {
            return None;
        }
        let source_tail = &source_path[1..];
        let path_len = context_path.len() + source_tail.len();
        if path_len == 0 {
            return None;
        }
        let property_index =
            Self::combined_context_source_property_index(context_path, source_tail, path_len - 1)?;
        if path_len == 1 {
            return Some((None, property_index));
        }
        let parent = self.view_model_by_context_source_property_prefix(
            context_path,
            source_tail,
            path_len - 1,
        )?;
        Some((Some(parent), property_index))
    }

    fn context_source_view_model_target(
        &self,
        context_path: &[usize],
        source_path: &[u32],
    ) -> Option<&RuntimeOwnedViewModelViewModel> {
        if source_path.is_empty() {
            return None;
        }
        let view_model_index = self.view_model_index_by_property_path(context_path)?;
        if usize::try_from(source_path[0]).ok()? != view_model_index {
            return None;
        }
        let source_tail = &source_path[1..];
        let path_len = context_path.len() + source_tail.len();
        self.view_model_by_context_source_property_prefix(context_path, source_tail, path_len)
    }

    fn view_model_by_property_path_mut(
        &mut self,
        property_path: &[usize],
    ) -> Option<&mut RuntimeOwnedViewModelViewModel> {
        let (property_index, rest) = property_path.split_first()?;
        let mut view_model = self
            .view_models
            .iter_mut()
            .find(|view_model| view_model.property_index == *property_index)?;
        for property_index in rest {
            view_model = view_model
                .active_children_mut()?
                .iter_mut()
                .find(|view_model| view_model.property_index == *property_index)?;
        }
        Some(view_model)
    }

    fn relinkable_view_model_by_property_path_mut(
        &mut self,
        property_path: &[usize],
    ) -> Option<&mut RuntimeOwnedViewModelViewModel> {
        let (property_index, rest) = property_path.split_first()?;
        let mut view_model = self
            .view_models
            .iter_mut()
            .find(|view_model| view_model.property_index == *property_index)?;
        for property_index in rest {
            view_model = view_model
                .generated_children_mut()?
                .iter_mut()
                .find(|view_model| view_model.property_index == *property_index)?;
        }
        Some(view_model)
    }

    pub(crate) fn sync_number_by_property_path(
        &mut self,
        property_path: &[usize],
        value: f32,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_number_by_property_index(property_path[0], value);
        }
        let Some((property_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        let changed = view_model.sync_number_by_property_index(*property_index, value);
        self.track_mutation(changed)
    }

    pub(crate) fn sync_boolean_by_property_path(
        &mut self,
        property_path: &[usize],
        value: bool,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_boolean_by_property_index(property_path[0], value);
        }
        let Some((property_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        let changed = view_model.sync_boolean_by_property_index(*property_index, value);
        self.track_mutation(changed)
    }

    pub(crate) fn sync_string_by_property_path(
        &mut self,
        property_path: &[usize],
        value: &[u8],
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_string_by_property_index(property_path[0], value);
        }
        let Some((property_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        let changed = view_model.sync_string_by_property_index(*property_index, value);
        self.track_mutation(changed)
    }

    pub(crate) fn sync_color_by_property_path(
        &mut self,
        property_path: &[usize],
        value: u32,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_color_by_property_index(property_path[0], value);
        }
        let Some((property_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        let changed = view_model.sync_color_by_property_index(*property_index, value);
        self.track_mutation(changed)
    }

    pub(crate) fn sync_enum_by_property_path(
        &mut self,
        property_path: &[usize],
        value: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_enum_by_property_index(property_path[0], value);
        }
        let Some((property_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        let changed = view_model.sync_enum_by_property_index(*property_index, value);
        self.track_mutation(changed)
    }

    pub(crate) fn sync_symbol_list_index_by_property_path(
        &mut self,
        property_path: &[usize],
        value: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_symbol_list_index_by_property_index(property_path[0], value);
        }
        let Some((property_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        let changed = view_model.sync_symbol_list_index_by_property_index(*property_index, value);
        self.track_mutation(changed)
    }

    pub(crate) fn sync_asset_by_property_path(
        &mut self,
        property_path: &[usize],
        value: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_asset_by_property_index(property_path[0], value);
        }
        let Some((property_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        let changed = view_model.sync_asset_by_property_index(*property_index, value);
        self.track_mutation(changed)
    }

    pub(crate) fn sync_font_asset_index_by_property_path(
        &mut self,
        property_path: &[usize],
        file_asset_index: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_font_asset_index_by_property_index(property_path[0], file_asset_index);
        }
        let Some((property_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        let changed =
            view_model.sync_font_asset_index_by_property_index(*property_index, file_asset_index);
        self.track_mutation(changed)
    }

    pub(crate) fn sync_live_font_bytes_by_property_path(
        &mut self,
        property_path: &[usize],
        font_bytes: Option<Arc<[u8]>>,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_live_font_bytes_by_property_index(property_path[0], font_bytes);
        }
        let Some((property_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        let changed =
            view_model.sync_live_font_bytes_by_property_index(*property_index, font_bytes);
        self.track_mutation(changed)
    }

    pub(crate) fn apply_font_asset_data_bind_value_by_property_path(
        &mut self,
        property_path: &[usize],
        value: &RuntimeFontAssetValue,
    ) -> bool {
        if property_path.len() == 1 {
            let Some(asset) = self
                .font_assets
                .iter_mut()
                .find(|asset| asset.property_index == property_path[0])
            else {
                return false;
            };
            let changed = asset.value.apply_data_bind_value(value);
            return self.track_mutation(changed);
        }
        let Some((property_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        let changed =
            view_model.apply_font_asset_data_bind_value_by_property_index(*property_index, value);
        self.track_mutation(changed)
    }

    pub(crate) fn sync_artboard_by_property_path(
        &mut self,
        property_path: &[usize],
        value: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_artboard_by_property_index(property_path[0], value);
        }
        let Some((property_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        let changed = view_model.sync_artboard_by_property_index(*property_index, value);
        self.track_mutation(changed)
    }

    pub(crate) fn sync_trigger_by_property_path(
        &mut self,
        property_path: &[usize],
        value: u64,
    ) -> bool {
        if property_path.len() == 1 {
            return self.set_trigger_by_property_index(property_path[0], value);
        }
        let Some((property_index, view_model_path)) = property_path.split_last() else {
            return false;
        };
        let Some(view_model) = self.view_model_by_property_path_mut(view_model_path) else {
            return false;
        };
        let changed = view_model.sync_trigger_by_property_index(*property_index, value);
        self.track_mutation(changed)
    }

    fn view_model_by_property_names_mut(
        &mut self,
        property_path: &[&str],
    ) -> Option<&mut RuntimeOwnedViewModelViewModel> {
        let (property_name, rest) = property_path.split_first()?;
        let mut view_model = self
            .view_models
            .iter_mut()
            .find(|view_model| view_model.property_name == *property_name)?;
        for property_name in rest {
            view_model = view_model
                .generated_children_mut()?
                .iter_mut()
                .find(|view_model| view_model.property_name == *property_name)?;
        }
        Some(view_model)
    }

    fn view_model_property_path_by_names(
        &self,
        property_path: &[&str],
    ) -> Option<(Vec<usize>, &RuntimeOwnedViewModelViewModel)> {
        let (property_name, rest) = property_path.split_first()?;
        let mut path = Vec::new();
        let mut view_model = self
            .view_models
            .iter()
            .find(|view_model| view_model.property_name == *property_name)?;
        path.push(view_model.property_index);
        for property_name in rest {
            if !matches!(
                view_model.value,
                RuntimeViewModelPointer::OwnedGenerated { .. }
            ) {
                return None;
            }
            view_model = view_model
                .children
                .iter()
                .find(|view_model| view_model.property_name == *property_name)?;
            path.push(view_model.property_index);
        }
        Some((path, view_model))
    }

    fn number_value_by_property_index(&self, property_index: usize) -> Option<f32> {
        self.numbers
            .iter()
            .find(|number| number.property_index == property_index)
            .map(|number| number.value)
    }

    pub(crate) fn number_value_by_property_path(&self, property_path: &[usize]) -> Option<f32> {
        if property_path.len() == 1 {
            return self.number_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_number_value_by_property_index(*property_index)
    }

    pub(crate) fn number_value_by_context_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
    ) -> Option<f32> {
        if name_based {
            let property_path =
                self.property_path_for_context_source_path(file, context_path, source_path, true)?;
            return self.number_value_by_property_path(&property_path);
        }
        let (parent, property_index) =
            self.context_source_value_target(context_path, source_path)?;
        match parent {
            Some(view_model) => view_model.active_number_value_by_property_index(property_index),
            None => self.number_value_by_property_index(property_index),
        }
    }

    fn boolean_value_by_property_index(&self, property_index: usize) -> Option<bool> {
        self.booleans
            .iter()
            .find(|boolean| boolean.property_index == property_index)
            .map(|boolean| boolean.value)
    }

    pub(crate) fn boolean_value_by_property_path(&self, property_path: &[usize]) -> Option<bool> {
        if property_path.len() == 1 {
            return self.boolean_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_boolean_value_by_property_index(*property_index)
    }

    pub(crate) fn boolean_value_by_context_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
    ) -> Option<bool> {
        if name_based {
            let property_path =
                self.property_path_for_context_source_path(file, context_path, source_path, true)?;
            return self.boolean_value_by_property_path(&property_path);
        }
        let (parent, property_index) =
            self.context_source_value_target(context_path, source_path)?;
        match parent {
            Some(view_model) => view_model.active_boolean_value_by_property_index(property_index),
            None => self.boolean_value_by_property_index(property_index),
        }
    }

    fn string_value_by_property_index(&self, property_index: usize) -> Option<&[u8]> {
        self.strings
            .iter()
            .find(|string| string.property_index == property_index)
            .map(|string| string.value.as_slice())
    }

    pub(crate) fn string_value_by_property_path(&self, property_path: &[usize]) -> Option<&[u8]> {
        if property_path.len() == 1 {
            return self.string_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_string_value_by_property_index(*property_index)
    }

    pub(crate) fn string_value_by_context_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
    ) -> Option<&[u8]> {
        if name_based {
            let property_path =
                self.property_path_for_context_source_path(file, context_path, source_path, true)?;
            return self.string_value_by_property_path(&property_path);
        }
        let (parent, property_index) =
            self.context_source_value_target(context_path, source_path)?;
        match parent {
            Some(view_model) => view_model.active_string_value_by_property_index(property_index),
            None => self.string_value_by_property_index(property_index),
        }
    }

    fn color_value_by_property_index(&self, property_index: usize) -> Option<u32> {
        self.colors
            .iter()
            .find(|color| color.property_index == property_index)
            .map(|color| color.value)
    }

    pub(crate) fn color_value_by_property_path(&self, property_path: &[usize]) -> Option<u32> {
        if property_path.len() == 1 {
            return self.color_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_color_value_by_property_index(*property_index)
    }

    pub(crate) fn color_value_by_context_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
    ) -> Option<u32> {
        if name_based {
            let property_path =
                self.property_path_for_context_source_path(file, context_path, source_path, true)?;
            return self.color_value_by_property_path(&property_path);
        }
        let (parent, property_index) =
            self.context_source_value_target(context_path, source_path)?;
        match parent {
            Some(view_model) => view_model.active_color_value_by_property_index(property_index),
            None => self.color_value_by_property_index(property_index),
        }
    }

    fn enum_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.enums
            .iter()
            .find(|enum_value| enum_value.property_index == property_index)
            .map(|enum_value| enum_value.value)
    }

    pub(crate) fn enum_value_by_property_path(&self, property_path: &[usize]) -> Option<u64> {
        if property_path.len() == 1 {
            return self.enum_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_enum_value_by_property_index(*property_index)
    }

    pub(crate) fn enum_value_by_context_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
    ) -> Option<u64> {
        if name_based {
            let property_path =
                self.property_path_for_context_source_path(file, context_path, source_path, true)?;
            return self.enum_value_by_property_path(&property_path);
        }
        let (parent, property_index) =
            self.context_source_value_target(context_path, source_path)?;
        match parent {
            Some(view_model) => view_model.active_enum_value_by_property_index(property_index),
            None => self.enum_value_by_property_index(property_index),
        }
    }

    fn symbol_list_index_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.symbol_list_indices
            .iter()
            .find(|symbol_list_index| symbol_list_index.property_index == property_index)
            .map(|symbol_list_index| symbol_list_index.value)
    }

    pub(crate) fn symbol_list_index_value_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<u64> {
        if property_path.len() == 1 {
            return self.symbol_list_index_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_symbol_list_index_value_by_property_index(*property_index)
    }

    pub(crate) fn symbol_list_index_value_by_context_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
    ) -> Option<u64> {
        if name_based {
            let property_path =
                self.property_path_for_context_source_path(file, context_path, source_path, true)?;
            return self.symbol_list_index_value_by_property_path(&property_path);
        }
        let (parent, property_index) =
            self.context_source_value_target(context_path, source_path)?;
        match parent {
            Some(view_model) => {
                view_model.active_symbol_list_index_value_by_property_index(property_index)
            }
            None => self.symbol_list_index_value_by_property_index(property_index),
        }
    }

    fn list_item_count_by_property_index(&self, property_index: usize) -> Option<usize> {
        self.lists
            .iter()
            .find(|list| list.property_index == property_index)
            .map(|list| list.value.borrow().item_count)
    }

    pub(crate) fn list_item_count_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<usize> {
        if property_path.len() == 1 {
            return self.list_item_count_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_list_item_count_by_property_index(*property_index)
    }

    pub(crate) fn list_handle_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<RuntimeOwnedViewModelListHandle> {
        let (property_index, view_model_path) = property_path.split_last()?;
        let list = if view_model_path.is_empty() {
            self.lists
                .iter()
                .find(|list| list.property_index == *property_index)?
        } else {
            self.view_model_by_property_path(view_model_path)?
                .active_list_by_property_index(*property_index)?
        };
        Some(RuntimeOwnedViewModelListHandle {
            value: Rc::clone(&list.value),
        })
    }

    pub(crate) fn list_item_count_by_context_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
    ) -> Option<usize> {
        if name_based {
            let property_path =
                self.property_path_for_context_source_path(file, context_path, source_path, true)?;
            return self.list_item_count_by_property_path(&property_path);
        }
        let (parent, property_index) =
            self.context_source_value_target(context_path, source_path)?;
        match parent {
            Some(view_model) => view_model.active_list_item_count_by_property_index(property_index),
            None => self.list_item_count_by_property_index(property_index),
        }
    }

    fn asset_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.assets
            .iter()
            .find(|asset| asset.property_index == property_index)
            .map(|asset| asset.value)
    }

    pub(crate) fn asset_value_by_property_path(&self, property_path: &[usize]) -> Option<u64> {
        if property_path.len() == 1 {
            return self.asset_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_asset_value_by_property_index(*property_index)
    }

    pub(crate) fn asset_value_by_context_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
    ) -> Option<u64> {
        if name_based {
            let property_path =
                self.property_path_for_context_source_path(file, context_path, source_path, true)?;
            return self.asset_value_by_property_path(&property_path);
        }
        let (parent, property_index) =
            self.context_source_value_target(context_path, source_path)?;
        match parent {
            Some(view_model) => view_model.active_asset_value_by_property_index(property_index),
            None => self.asset_value_by_property_index(property_index),
        }
    }

    fn font_asset_value_by_property_index(
        &self,
        property_index: usize,
    ) -> Option<&RuntimeFontAssetValue> {
        self.font_assets
            .iter()
            .find(|asset| asset.property_index == property_index)
            .map(|asset| &asset.value)
    }

    pub(crate) fn font_asset_value_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<&RuntimeFontAssetValue> {
        if property_path.len() == 1 {
            return self.font_asset_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_font_asset_value_by_property_index(*property_index)
    }

    pub(crate) fn font_asset_value_by_context_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
    ) -> Option<&RuntimeFontAssetValue> {
        if name_based {
            let property_path =
                self.property_path_for_context_source_path(file, context_path, source_path, true)?;
            return self.font_asset_value_by_property_path(&property_path);
        }
        let (parent, property_index) =
            self.context_source_value_target(context_path, source_path)?;
        match parent {
            Some(view_model) => {
                view_model.active_font_asset_value_by_property_index(property_index)
            }
            None => self.font_asset_value_by_property_index(property_index),
        }
    }

    fn artboard_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.artboards
            .iter()
            .find(|artboard| artboard.property_index == property_index)
            .map(|artboard| artboard.value)
    }

    pub(crate) fn artboard_value_by_property_path(&self, property_path: &[usize]) -> Option<u64> {
        if property_path.len() == 1 {
            return self.artboard_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_artboard_value_by_property_index(*property_index)
    }

    pub(crate) fn artboard_value_by_context_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
    ) -> Option<u64> {
        if name_based {
            let property_path =
                self.property_path_for_context_source_path(file, context_path, source_path, true)?;
            return self.artboard_value_by_property_path(&property_path);
        }
        let (parent, property_index) =
            self.context_source_value_target(context_path, source_path)?;
        match parent {
            Some(view_model) => view_model.active_artboard_value_by_property_index(property_index),
            None => self.artboard_value_by_property_index(property_index),
        }
    }

    pub(crate) fn trigger_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.triggers
            .iter()
            .find(|trigger| trigger.property_index == property_index)
            .map(|trigger| trigger.value)
    }

    pub(crate) fn trigger_value_by_property_path(&self, property_path: &[usize]) -> Option<u64> {
        if property_path.len() == 1 {
            return self.trigger_value_by_property_index(property_path[0]);
        }
        let (property_index, view_model_path) = property_path.split_last()?;
        let view_model = self.view_model_by_property_path(view_model_path)?;
        view_model.active_trigger_value_by_property_index(*property_index)
    }

    pub(crate) fn trigger_value_by_context_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
    ) -> Option<u64> {
        if name_based {
            let property_path =
                self.property_path_for_context_source_path(file, context_path, source_path, true)?;
            return self.trigger_value_by_property_path(&property_path);
        }
        let (parent, property_index) =
            self.context_source_value_target(context_path, source_path)?;
        match parent {
            Some(view_model) => view_model.active_trigger_value_by_property_index(property_index),
            None => self.trigger_value_by_property_index(property_index),
        }
    }

    pub(crate) fn view_model_value_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<RuntimeViewModelPointer> {
        self.view_model_by_property_path(property_path)
            .map(|view_model| view_model.value)
    }

    pub(crate) fn view_model_value_by_context_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
    ) -> Option<RuntimeViewModelPointer> {
        if name_based {
            let property_path =
                self.property_path_for_context_source_path(file, context_path, source_path, true)?;
            return self.view_model_value_by_property_path(&property_path);
        }
        self.context_source_view_model_target(context_path, source_path)
            .map(|view_model| view_model.value)
    }

    pub(crate) fn view_model_index_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<usize> {
        if property_path.is_empty() {
            return Some(self.view_model_index);
        }
        let view_model = self.view_model_by_property_path(property_path)?;
        match view_model.value {
            RuntimeViewModelPointer::OwnedGenerated { .. }
            | RuntimeViewModelPointer::Imported { .. } => view_model.referenced_view_model_index,
            _ => None,
        }
    }

    pub(crate) fn nested_instance_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<RuntimeOwnedViewModelInstance> {
        self.view_model_by_property_path(property_path)?
            .materialize_active_instance()
    }

    pub(crate) fn property_path_for_context_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
    ) -> Option<Vec<usize>> {
        self.property_path_for_context_source_path_with_manifest_mode(
            file,
            context_path,
            source_path,
            name_based,
            false,
        )
    }

    pub(crate) fn property_path_for_context_source_path_with_manifest_mode(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        name_based: bool,
        scripting_manifest: bool,
    ) -> Option<Vec<usize>> {
        if name_based {
            return self.property_path_for_context_name_source_path(
                file,
                context_path,
                source_path,
                scripting_manifest,
            );
        }

        if source_path.is_empty() {
            return None;
        }
        let view_model_index = self.view_model_index_by_property_path(context_path)?;
        if usize::try_from(source_path[0]).ok()? != view_model_index {
            return None;
        }
        let mut property_path =
            Vec::with_capacity(context_path.len() + source_path.len().saturating_sub(1));
        property_path.extend_from_slice(context_path);
        for property_index in &source_path[1..] {
            property_path.push(usize::try_from(*property_index).ok()?);
        }
        Some(property_path)
    }

    fn property_path_for_context_name_source_path(
        &self,
        file: &RuntimeFile,
        context_path: &[usize],
        source_path: &[u32],
        scripting_manifest: bool,
    ) -> Option<Vec<usize>> {
        if source_path.is_empty() {
            return None;
        }
        let manifest = if scripting_manifest {
            file.scripting_manifest()?
        } else {
            file.manifest()?
        };
        let source_path = source_path
            .first()
            .and_then(|path_id| manifest.resolve_path(*path_id))
            .filter(|resolved_path| !resolved_path.is_empty())
            .unwrap_or(source_path);
        let mut property_path = Vec::with_capacity(context_path.len() + source_path.len());
        property_path.extend_from_slice(context_path);
        for name_id in source_path {
            let property_name = manifest.resolve_name(*name_id)?;
            let property_index = if property_path.is_empty() {
                self.property_index_by_name(property_name)?
            } else {
                self.view_model_by_property_path(&property_path)?
                    .property_index_by_name(property_name)?
            };
            property_path.push(property_index);
        }
        Some(property_path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeViewModelPointer {
    Null,
    DataContextRoot,
    OwnedGenerated {
        view_model_index: usize,
        property_index: usize,
        path_key: u64,
    },
    Imported {
        object_id: u32,
    },
}

#[derive(Debug, Clone)]
pub struct RuntimeDataContext<'a> {
    file: &'a RuntimeFile,
    current_instance: &'a RuntimeObject,
    parent_instances: Vec<&'a RuntimeObject>,
}

impl<'a> RuntimeDataContext<'a> {
    pub fn new(
        file: &'a RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
    ) -> Option<Self> {
        let view_model = file.view_model(view_model_index)?;
        let instance = view_model.instances.get(instance_index)?;
        Self::from_instance_object(file, instance.object)
    }

    pub fn from_instance_reference(
        file: &'a RuntimeFile,
        instance: RuntimeViewModelInstanceReference<'a>,
    ) -> Option<Self> {
        Self::from_instance_object(file, instance.object)
    }

    pub fn from_instance_object(
        file: &'a RuntimeFile,
        instance: &'a RuntimeObject,
    ) -> Option<Self> {
        (instance.type_name == "ViewModelInstance").then_some(Self {
            file,
            current_instance: instance,
            parent_instances: Vec::new(),
        })
    }

    pub fn with_parent(mut self, parent: &RuntimeDataContext<'a>) -> Self {
        self.parent_instances.push(parent.current_instance);
        self.parent_instances
            .extend(parent.parent_instances.iter().copied());
        self
    }

    pub fn current_instance(&self) -> &'a RuntimeObject {
        self.current_instance
    }

    pub fn parent_instances(&self) -> &[&'a RuntimeObject] {
        &self.parent_instances
    }

    pub fn absolute_property(&self, path: &[u32]) -> Option<&'a RuntimeObject> {
        let chain = self.instance_chain();
        self.file
            .data_context_view_model_property_for_instance_chain(&chain, path)
    }

    pub fn absolute_property_ref(&self, path: &[u32]) -> Option<RuntimeDataContextValueRef> {
        let view_models = self.file.view_models();
        self.absolute_property(path)
            .and_then(|value| runtime_data_context_value_ref(self.file, &view_models, value))
    }

    pub fn absolute_instance(&self, path: &[u32]) -> Option<RuntimeViewModelInstanceReference<'a>> {
        let chain = self.instance_chain();
        self.file
            .data_context_view_model_instance_for_instance_chain(&chain, path)
    }

    pub fn absolute_instance_ref(&self, path: &[u32]) -> Option<RuntimeDataContextInstanceRef> {
        let view_models = self.file.view_models();
        self.absolute_instance(path)
            .and_then(|instance| runtime_data_context_instance_ref(&view_models, instance))
    }

    pub fn property_from_path(&self, path: &[u32]) -> Option<&'a RuntimeObject> {
        self.file
            .view_model_instance_property_from_path_for_object(self.current_instance, path)
    }

    pub fn property_from_path_ref(&self, path: &[u32]) -> Option<RuntimeDataContextValueRef> {
        let view_models = self.file.view_models();
        self.property_from_path(path)
            .and_then(|value| runtime_data_context_value_ref(self.file, &view_models, value))
    }

    pub fn relative_property(&self, path: &[u32]) -> Option<&'a RuntimeObject> {
        let chain = self.instance_chain();
        self.file
            .data_context_relative_view_model_property_for_instance_chain(&chain, path)
    }

    pub fn relative_property_ref(&self, path: &[u32]) -> Option<RuntimeDataContextValueRef> {
        let view_models = self.file.view_models();
        self.relative_property(path)
            .and_then(|value| runtime_data_context_value_ref(self.file, &view_models, value))
    }

    pub fn relative_instance(&self, path: &[u32]) -> Option<RuntimeViewModelInstanceReference<'a>> {
        let chain = self.instance_chain();
        self.file
            .data_context_relative_view_model_instance_for_instance_chain(&chain, path)
    }

    pub fn relative_instance_ref(&self, path: &[u32]) -> Option<RuntimeDataContextInstanceRef> {
        let view_models = self.file.view_models();
        self.relative_instance(path)
            .and_then(|instance| runtime_data_context_instance_ref(&view_models, instance))
    }

    fn instance_chain(&self) -> Vec<&'a RuntimeObject> {
        let mut chain = Vec::with_capacity(self.parent_instances.len() + 1);
        chain.push(self.current_instance);
        chain.extend(self.parent_instances.iter().copied());
        chain
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDataContextLookupReport {
    pub kind: RuntimeDataContextLookupKind,
    pub current_view_model_index: usize,
    pub current_instance_index: usize,
    pub parent_view_model_index: Option<usize>,
    pub parent_instance_index: Option<usize>,
    pub path: Vec<u32>,
    pub value: Option<RuntimeDataContextValueRef>,
    pub instance: Option<RuntimeDataContextInstanceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDataContextLookupKind {
    AbsoluteInstance,
    AbsoluteProperty,
    PropertyFromPath,
    RelativeProperty,
    RelativeInstance,
    AbsolutePropertyParentFallback,
    RelativePropertyParentFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDataContextValueRef {
    pub view_model_index: usize,
    pub instance_index: usize,
    pub value_index: usize,
    pub core_type: u32,
    pub view_model_property_id: u32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDataContextInstanceRef {
    pub view_model_index: usize,
    pub instance_index: usize,
    pub core_type: u32,
    pub name: String,
    pub view_model_id: u32,
}

pub fn runtime_data_context_lookup_reports(
    file: &RuntimeFile,
) -> Vec<RuntimeDataContextLookupReport> {
    let view_models = file.view_models();
    let manifest_name_ids = runtime_data_context_manifest_name_ids(file);
    let mut reports = Vec::new();

    for (view_model_index, view_model) in view_models.iter().enumerate() {
        for (instance_index, instance) in view_model.instances.iter().enumerate() {
            let Some(context) = RuntimeDataContext::from_instance_object(file, instance.object)
            else {
                continue;
            };
            let absolute_path = vec![runtime_object_u32_property(
                context.current_instance(),
                "viewModelId",
            )];
            collect_runtime_data_context_absolute_lookups(
                file,
                &view_models,
                &mut reports,
                &context,
                view_model_index,
                instance_index,
                instance,
                absolute_path,
                0,
            );
            collect_runtime_data_context_property_from_path_lookups(
                file,
                &view_models,
                &mut reports,
                &context,
                view_model_index,
                instance_index,
                instance,
                Vec::new(),
                0,
            );
            collect_runtime_data_context_relative_lookups(
                file,
                &view_models,
                &manifest_name_ids,
                &mut reports,
                &context,
                view_model_index,
                instance_index,
                instance,
                Vec::new(),
                0,
            );
        }
    }
    collect_runtime_data_context_parent_fallback_lookups(
        file,
        &view_models,
        &manifest_name_ids,
        &mut reports,
    );

    reports
}

fn collect_runtime_data_context_absolute_lookups<'a>(
    file: &'a RuntimeFile,
    view_models: &[RuntimeViewModel<'a>],
    reports: &mut Vec<RuntimeDataContextLookupReport>,
    context: &RuntimeDataContext<'a>,
    root_view_model_index: usize,
    root_instance_index: usize,
    instance: &RuntimeViewModelInstance<'a>,
    path: Vec<u32>,
    depth: usize,
) {
    if depth > 8 {
        return;
    }

    reports.push(RuntimeDataContextLookupReport {
        kind: RuntimeDataContextLookupKind::AbsoluteInstance,
        current_view_model_index: root_view_model_index,
        current_instance_index: root_instance_index,
        parent_view_model_index: None,
        parent_instance_index: None,
        path: path.clone(),
        value: None,
        instance: context.absolute_instance_ref(&path),
    });

    for value in &instance.values {
        let mut value_path = path.clone();
        value_path.push(runtime_object_u32_property(
            value.object,
            "viewModelPropertyId",
        ));
        reports.push(RuntimeDataContextLookupReport {
            kind: RuntimeDataContextLookupKind::AbsoluteProperty,
            current_view_model_index: root_view_model_index,
            current_instance_index: root_instance_index,
            parent_view_model_index: None,
            parent_instance_index: None,
            path: value_path.clone(),
            value: context.absolute_property_ref(&value_path),
            instance: None,
        });

        if value.object.type_name != "ViewModelInstanceViewModel" {
            continue;
        }
        let Some(reference) = file.referenced_view_model_instance_for_value_object(value.object)
        else {
            continue;
        };
        reports.push(RuntimeDataContextLookupReport {
            kind: RuntimeDataContextLookupKind::AbsoluteInstance,
            current_view_model_index: root_view_model_index,
            current_instance_index: root_instance_index,
            parent_view_model_index: None,
            parent_instance_index: None,
            path: value_path.clone(),
            value: None,
            instance: context.absolute_instance_ref(&value_path),
        });

        if let Some(referenced_instance) = runtime_view_model_instance_from_reference(
            view_models,
            reference.view_model_index,
            reference.instance_index,
        ) {
            collect_runtime_data_context_absolute_lookups(
                file,
                view_models,
                reports,
                context,
                root_view_model_index,
                root_instance_index,
                referenced_instance,
                value_path,
                depth + 1,
            );
        }
    }
}

fn collect_runtime_data_context_property_from_path_lookups<'a>(
    file: &'a RuntimeFile,
    view_models: &[RuntimeViewModel<'a>],
    reports: &mut Vec<RuntimeDataContextLookupReport>,
    context: &RuntimeDataContext<'a>,
    root_view_model_index: usize,
    root_instance_index: usize,
    instance: &RuntimeViewModelInstance<'a>,
    path: Vec<u32>,
    depth: usize,
) {
    if depth > 8 {
        return;
    }

    for value in &instance.values {
        let mut value_path = path.clone();
        value_path.push(runtime_object_u32_property(
            value.object,
            "viewModelPropertyId",
        ));
        reports.push(RuntimeDataContextLookupReport {
            kind: RuntimeDataContextLookupKind::PropertyFromPath,
            current_view_model_index: root_view_model_index,
            current_instance_index: root_instance_index,
            parent_view_model_index: None,
            parent_instance_index: None,
            path: value_path.clone(),
            value: context.property_from_path_ref(&value_path),
            instance: None,
        });

        if value.object.type_name != "ViewModelInstanceViewModel" {
            continue;
        }
        let Some(reference) = file.referenced_view_model_instance_for_value_object(value.object)
        else {
            continue;
        };
        if let Some(referenced_instance) = runtime_view_model_instance_from_reference(
            view_models,
            reference.view_model_index,
            reference.instance_index,
        ) {
            collect_runtime_data_context_property_from_path_lookups(
                file,
                view_models,
                reports,
                context,
                root_view_model_index,
                root_instance_index,
                referenced_instance,
                value_path,
                depth + 1,
            );
        }
    }
}

fn collect_runtime_data_context_relative_lookups<'a>(
    file: &'a RuntimeFile,
    view_models: &[RuntimeViewModel<'a>],
    manifest_name_ids: &[(Vec<u8>, u32)],
    reports: &mut Vec<RuntimeDataContextLookupReport>,
    context: &RuntimeDataContext<'a>,
    root_view_model_index: usize,
    root_instance_index: usize,
    instance: &RuntimeViewModelInstance<'a>,
    path: Vec<u32>,
    depth: usize,
) {
    if depth > 8 || manifest_name_ids.is_empty() {
        return;
    }

    for value in &instance.values {
        let Some(name) = file.view_model_instance_value_name_for_object(value.object) else {
            continue;
        };
        let Some(name_id) = runtime_data_context_name_id(manifest_name_ids, name.as_bytes()) else {
            continue;
        };

        let mut value_path = path.clone();
        value_path.push(name_id);
        reports.push(RuntimeDataContextLookupReport {
            kind: RuntimeDataContextLookupKind::RelativeProperty,
            current_view_model_index: root_view_model_index,
            current_instance_index: root_instance_index,
            parent_view_model_index: None,
            parent_instance_index: None,
            path: value_path.clone(),
            value: context.relative_property_ref(&value_path),
            instance: None,
        });

        if value.object.type_name != "ViewModelInstanceViewModel" {
            continue;
        }
        let Some(reference) = file.referenced_view_model_instance_for_value_object(value.object)
        else {
            continue;
        };
        reports.push(RuntimeDataContextLookupReport {
            kind: RuntimeDataContextLookupKind::RelativeInstance,
            current_view_model_index: root_view_model_index,
            current_instance_index: root_instance_index,
            parent_view_model_index: None,
            parent_instance_index: None,
            path: value_path.clone(),
            value: None,
            instance: context.relative_instance_ref(&value_path),
        });

        if let Some(referenced_instance) = runtime_view_model_instance_from_reference(
            view_models,
            reference.view_model_index,
            reference.instance_index,
        ) {
            collect_runtime_data_context_relative_lookups(
                file,
                view_models,
                manifest_name_ids,
                reports,
                context,
                root_view_model_index,
                root_instance_index,
                referenced_instance,
                value_path,
                depth + 1,
            );
        }
    }
}

fn collect_runtime_data_context_parent_fallback_lookups<'a>(
    file: &'a RuntimeFile,
    view_models: &[RuntimeViewModel<'a>],
    manifest_name_ids: &[(Vec<u8>, u32)],
    reports: &mut Vec<RuntimeDataContextLookupReport>,
) {
    if view_models.len() < 2 {
        return;
    }

    for (current_view_model_index, current_view_model) in view_models.iter().enumerate() {
        let Some(current_instance) = current_view_model.instances.first() else {
            continue;
        };
        for (parent_view_model_index, parent_view_model) in view_models.iter().enumerate() {
            if parent_view_model_index == current_view_model_index {
                continue;
            }
            let Some(parent_instance) = parent_view_model.instances.first() else {
                continue;
            };
            let Some(parent_value) = parent_instance.values.first() else {
                continue;
            };
            let Some(context) =
                RuntimeDataContext::from_instance_object(file, current_instance.object)
            else {
                continue;
            };
            let Some(parent_context) =
                RuntimeDataContext::from_instance_object(file, parent_instance.object)
            else {
                continue;
            };
            let context = context.with_parent(&parent_context);

            let absolute_path = vec![
                runtime_object_u32_property(parent_instance.object, "viewModelId"),
                runtime_object_u32_property(parent_value.object, "viewModelPropertyId"),
            ];
            reports.push(RuntimeDataContextLookupReport {
                kind: RuntimeDataContextLookupKind::AbsolutePropertyParentFallback,
                current_view_model_index,
                current_instance_index: 0,
                parent_view_model_index: Some(parent_view_model_index),
                parent_instance_index: Some(0),
                path: absolute_path.clone(),
                value: context.absolute_property_ref(&absolute_path),
                instance: None,
            });

            if let Some(name_id) = file
                .view_model_instance_value_name_for_object(parent_value.object)
                .and_then(|name| runtime_data_context_name_id(manifest_name_ids, name.as_bytes()))
            {
                let relative_path = vec![name_id];
                reports.push(RuntimeDataContextLookupReport {
                    kind: RuntimeDataContextLookupKind::RelativePropertyParentFallback,
                    current_view_model_index,
                    current_instance_index: 0,
                    parent_view_model_index: Some(parent_view_model_index),
                    parent_instance_index: Some(0),
                    path: relative_path.clone(),
                    value: context.relative_property_ref(&relative_path),
                    instance: None,
                });
            }
            return;
        }
    }
}

fn runtime_data_context_manifest_name_ids(file: &RuntimeFile) -> Vec<(Vec<u8>, u32)> {
    file.manifest()
        .map(|manifest| {
            manifest
                .names
                .iter()
                .filter_map(|(id, name)| {
                    u32::try_from(*id)
                        .ok()
                        .map(|id| (name.as_bytes().to_vec(), id))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_data_context_name_id(names: &[(Vec<u8>, u32)], name: &[u8]) -> Option<u32> {
    names
        .iter()
        .find_map(|(candidate, id)| (candidate.as_slice() == name).then_some(*id))
}

fn runtime_view_model_instance_from_reference<'models, 'file>(
    view_models: &'models [RuntimeViewModel<'file>],
    view_model_index: usize,
    instance_index: usize,
) -> Option<&'models RuntimeViewModelInstance<'file>> {
    view_models
        .get(view_model_index)?
        .instances
        .get(instance_index)
}

fn runtime_data_context_value_ref(
    file: &RuntimeFile,
    view_models: &[RuntimeViewModel<'_>],
    value: &RuntimeObject,
) -> Option<RuntimeDataContextValueRef> {
    for (view_model_index, view_model) in view_models.iter().enumerate() {
        for (instance_index, instance) in view_model.instances.iter().enumerate() {
            for (value_index, candidate) in instance.values.iter().enumerate() {
                if candidate.object.id != value.id {
                    continue;
                }
                return Some(RuntimeDataContextValueRef {
                    view_model_index,
                    instance_index,
                    value_index,
                    core_type: u32::from(value.type_key),
                    view_model_property_id: runtime_object_u32_property(
                        value,
                        "viewModelPropertyId",
                    ),
                    name: file
                        .view_model_instance_value_name_for_object(value)
                        .unwrap_or_default()
                        .to_owned(),
                });
            }
        }
    }

    None
}

fn runtime_data_context_instance_ref(
    view_models: &[RuntimeViewModel<'_>],
    reference: RuntimeViewModelInstanceReference<'_>,
) -> Option<RuntimeDataContextInstanceRef> {
    let instance = view_models
        .get(reference.view_model_index)?
        .instances
        .get(reference.instance_index)?;
    Some(RuntimeDataContextInstanceRef {
        view_model_index: reference.view_model_index,
        instance_index: reference.instance_index,
        core_type: u32::from(instance.object.type_key),
        name: instance
            .object
            .string_property("name")
            .unwrap_or_default()
            .to_owned(),
        view_model_id: runtime_object_u32_property(instance.object, "viewModelId"),
    })
}

fn runtime_object_u32_property(object: &RuntimeObject, property: &str) -> u32 {
    object
        .uint_property(property)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod owned_context_tests {
    use super::*;
    use crate::properties::property_key_for_name;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue};
    use nuxie_schema::definition_by_name;

    fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, name: &str, value: AuthoringValue) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
            value,
        }
    }

    fn view_model_records(
        name: &str,
        view_model_type: u64,
        view_model_index: u64,
        value: f32,
    ) -> Vec<AuthoringRecord> {
        vec![
            record(
                "ViewModel",
                vec![
                    property("ViewModel", "name", AuthoringValue::String(name.to_owned())),
                    property(
                        "ViewModel",
                        "viewModelType",
                        AuthoringValue::Uint(view_model_type),
                    ),
                ],
            ),
            record(
                "ViewModelPropertyNumber",
                vec![property(
                    "ViewModelPropertyNumber",
                    "name",
                    AuthoringValue::String("value".to_owned()),
                )],
            ),
            record(
                "ViewModelInstance",
                vec![
                    property(
                        "ViewModelInstance",
                        "viewModelId",
                        AuthoringValue::Uint(view_model_index),
                    ),
                    property(
                        "ViewModelInstance",
                        "name",
                        AuthoringValue::String("Default".to_owned()),
                    ),
                ],
            ),
            record(
                "ViewModelInstanceNumber",
                vec![
                    property(
                        "ViewModelInstanceNumber",
                        "viewModelPropertyId",
                        AuthoringValue::Uint(0),
                    ),
                    property(
                        "ViewModelInstanceNumber",
                        "propertyValue",
                        AuthoringValue::Double(value),
                    ),
                ],
            ),
        ]
    }

    fn global_context_fixture() -> RuntimeFile {
        let mut records = vec![record("Backboard", Vec::new())];
        records.extend(view_model_records("Global Z", 2, 0, 10.0));
        records.extend(view_model_records("Main", 0, 1, 20.0));
        records.extend(view_model_records("Global A", 2, 2, 30.0));
        records.push(record(
            "Artboard",
            vec![property("Artboard", "viewModelId", AuthoringValue::Uint(1))],
        ));
        RuntimeFile::from_authoring_records(records).expect("global context fixture imports")
    }

    fn symbol_list_index_order_fixture() -> RuntimeFile {
        RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Rows".to_owned()),
                )],
            ),
            record(
                "ViewModelPropertySymbolListIndex",
                vec![property(
                    "ViewModelPropertySymbolListIndex",
                    "name",
                    AuthoringValue::String("first".to_owned()),
                )],
            ),
            record(
                "ViewModelPropertySymbolListIndex",
                vec![property(
                    "ViewModelPropertySymbolListIndex",
                    "name",
                    AuthoringValue::String("second".to_owned()),
                )],
            ),
            record(
                "ViewModelInstance",
                vec![
                    property("ViewModelInstance", "viewModelId", AuthoringValue::Uint(0)),
                    property(
                        "ViewModelInstance",
                        "name",
                        AuthoringValue::String("Default".to_owned()),
                    ),
                ],
            ),
            // Imported value order deliberately opposes property order. C++
            // registers `second` and then overwrites itemIndex with `first`.
            record(
                "ViewModelInstanceSymbolListIndex",
                vec![
                    property(
                        "ViewModelInstanceSymbolListIndex",
                        "viewModelPropertyId",
                        AuthoringValue::Uint(1),
                    ),
                    property(
                        "ViewModelInstanceSymbolListIndex",
                        "propertyValue",
                        AuthoringValue::Uint(22),
                    ),
                ],
            ),
            record(
                "ViewModelInstanceSymbolListIndex",
                vec![
                    property(
                        "ViewModelInstanceSymbolListIndex",
                        "viewModelPropertyId",
                        AuthoringValue::Uint(0),
                    ),
                    property(
                        "ViewModelInstanceSymbolListIndex",
                        "propertyValue",
                        AuthoringValue::Uint(11),
                    ),
                ],
            ),
        ])
        .expect("symbol-list-index order fixture imports")
    }

    fn nested_trigger_fixture() -> RuntimeFile {
        RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Root".to_owned()),
                )],
            ),
            record(
                "ViewModelPropertyViewModel",
                vec![
                    property(
                        "ViewModelPropertyViewModel",
                        "name",
                        AuthoringValue::String("child".to_owned()),
                    ),
                    property(
                        "ViewModelPropertyViewModel",
                        "viewModelReferenceId",
                        AuthoringValue::Uint(1),
                    ),
                ],
            ),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Child".to_owned()),
                )],
            ),
            record(
                "ViewModelPropertyTrigger",
                vec![property(
                    "ViewModelPropertyTrigger",
                    "name",
                    AuthoringValue::String("fire".to_owned()),
                )],
            ),
        ])
        .expect("nested trigger fixture imports")
    }

    #[test]
    fn generated_artboard_property_starts_unassigned() {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Main".to_owned()),
                )],
            ),
            record(
                "ViewModelPropertyArtboard",
                vec![property(
                    "ViewModelPropertyArtboard",
                    "name",
                    AuthoringValue::String("artboard".to_owned()),
                )],
            ),
        ])
        .expect("artboard property fixture imports");
        let context =
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("generated view-model instance");

        assert_eq!(
            context.artboard_value_by_property_path(&[0]),
            Some(u64::from(u32::MAX))
        );
    }

    #[test]
    fn font_assets_preserve_file_identity_and_private_live_value_without_becoming_images() {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record(
                "FontAsset",
                vec![property("FontAsset", "assetId", AuthoringValue::Uint(7))],
            ),
            record(
                "ImageAsset",
                vec![property("ImageAsset", "assetId", AuthoringValue::Uint(8))],
            ),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Main".to_owned()),
                )],
            ),
            record(
                "ViewModelPropertyAssetFont",
                vec![property(
                    "ViewModelPropertyAssetFont",
                    "name",
                    AuthoringValue::String("font".to_owned()),
                )],
            ),
            record(
                "ViewModelPropertyAssetImage",
                vec![property(
                    "ViewModelPropertyAssetImage",
                    "name",
                    AuthoringValue::String("image".to_owned()),
                )],
            ),
            record(
                "ViewModelInstance",
                vec![
                    property("ViewModelInstance", "viewModelId", AuthoringValue::Uint(0)),
                    property(
                        "ViewModelInstance",
                        "name",
                        AuthoringValue::String("Default".to_owned()),
                    ),
                ],
            ),
            record(
                "ViewModelInstanceAssetFont",
                vec![
                    property(
                        "ViewModelInstanceAssetFont",
                        "viewModelPropertyId",
                        AuthoringValue::Uint(0),
                    ),
                    property(
                        "ViewModelInstanceAssetFont",
                        "propertyValue",
                        AuthoringValue::Uint(0),
                    ),
                ],
            ),
            record(
                "ViewModelInstanceAssetImage",
                vec![
                    property(
                        "ViewModelInstanceAssetImage",
                        "viewModelPropertyId",
                        AuthoringValue::Uint(1),
                    ),
                    property(
                        "ViewModelInstanceAssetImage",
                        "propertyValue",
                        AuthoringValue::Uint(1),
                    ),
                ],
            ),
        ])
        .expect("font asset view-model fixture imports");
        let mut context = RuntimeOwnedViewModelInstance::from_instance(&file, 0, 0)
            .expect("imported view-model instance");

        assert_eq!(
            context
                .font_asset_value_by_property_name("font")
                .map(RuntimeFontAssetValue::file_asset_index),
            Some(0)
        );
        assert!(
            context
                .asset_source_handle_by_property_name("font")
                .is_none()
        );
        assert!(
            context
                .font_asset_source_handle_by_property_name("image")
                .is_none()
        );
        assert_eq!(context.asset_value_by_property_path(&[1]), Some(1));

        let live: Arc<[u8]> = vec![1, 2, 3, 4].into();
        assert!(context.set_live_font_bytes_by_property_name("font", Some(Arc::clone(&live))));
        let live_value = context
            .font_asset_value_by_property_name("font")
            .expect("font value");
        assert_eq!(
            live_value.file_asset_index(),
            RuntimeFontAssetValue::MISSING_FILE_ASSET_INDEX
        );
        assert!(
            live_value
                .live_font_bytes_arc()
                .is_some_and(|value| Arc::ptr_eq(value, &live))
        );
        assert!(
            !context.set_live_font_bytes_by_property_name("font", Some(Arc::clone(&live))),
            "reassigning the same live font pointer is a no-op once the sentinel is set"
        );

        assert!(context.set_font_asset_index_by_property_name("font", 0));
        let file_value = context
            .font_asset_value_by_property_name("font")
            .expect("font value");
        assert_eq!(file_value.file_asset_index(), 0);
        assert!(
            file_value
                .live_font_bytes_arc()
                .is_some_and(|value| Arc::ptr_eq(value, &live)),
            "setting a file identity preserves the private live fallback like C++"
        );

        let listener_live: Arc<[u8]> = vec![5, 6, 7, 8].into();
        let mut listener_value = RuntimeFontAssetValue::default();
        assert!(listener_value.set_live_font_bytes(Some(Arc::clone(&listener_live))));
        assert!(context.apply_font_asset_data_bind_value_by_property_path(&[0], &listener_value,));
        let applied_live = context
            .font_asset_value_by_property_name("font")
            .expect("font value");
        assert_eq!(
            applied_live.file_asset_index(),
            RuntimeFontAssetValue::MISSING_FILE_ASSET_INDEX
        );
        assert!(
            applied_live
                .live_font_bytes_arc()
                .is_some_and(|value| Arc::ptr_eq(value, &listener_live)),
            "a listener/data-bind round-trip retains the live font payload"
        );

        let listener_file_value = RuntimeFontAssetValue::from_file_asset_index(0);
        assert!(
            context.apply_font_asset_data_bind_value_by_property_path(&[0], &listener_file_value,)
        );
        let applied_file = context
            .font_asset_value_by_property_name("font")
            .expect("font value");
        assert_eq!(applied_file.file_asset_index(), 0);
        assert_eq!(
            applied_file.live_font_bytes(),
            None,
            "a file-font listener value clears the previous private live font"
        );

        assert!(context.set_live_font_bytes_by_property_name("font", None));
        let cleared = context
            .font_asset_value_by_property_name("font")
            .expect("font value");
        assert_eq!(
            cleared.file_asset_index(),
            RuntimeFontAssetValue::MISSING_FILE_ASSET_INDEX
        );
        assert_eq!(cleared.live_font_bytes(), None);
    }

    #[test]
    fn global_view_models_keep_file_order_and_complete_defaults() {
        let file = global_context_fixture();
        assert_eq!(runtime_global_view_model_indices(&file), vec![0, 2]);
        assert_eq!(
            runtime_global_view_model_names(&file),
            vec!["Global Z".to_owned(), "Global A".to_owned()]
        );

        let mut context = RuntimeOwnedViewModelContext::new();
        assert!(context.complete_for_artboard(&file, 0));
        assert!(!context.complete_for_artboard(&file, 0));
        assert_eq!(
            context
                .instances()
                .map(RuntimeOwnedViewModelInstance::view_model_index)
                .collect::<Vec<_>>(),
            vec![1, 0, 2]
        );
        assert_eq!(
            context
                .global_named(&file, "Global Z")
                .and_then(|instance| instance.number_value_by_property_name("value")),
            Some(10.0)
        );
    }

    #[test]
    fn global_slots_allow_cross_view_model_overrides_and_reject_standard_names() {
        let file = global_context_fixture();
        let override_instance = RuntimeOwnedViewModelInstance::from_instance(&file, 1, 0)
            .expect("main default instance");
        let mut context = RuntimeOwnedViewModelContext::new();
        assert!(context.set_global_named(&file, "Global Z", override_instance));
        assert!(
            !context.set_global_named(
                &file,
                "Main",
                RuntimeOwnedViewModelInstance::from_instance(&file, 0, 0)
                    .expect("global default instance")
            )
        );
        assert!(context.complete_for_artboard(&file, 0));
        assert_eq!(
            context
                .instances()
                .map(RuntimeOwnedViewModelInstance::view_model_index)
                .collect::<Vec<_>>(),
            vec![1, 1, 2]
        );
        assert_eq!(
            context
                .global_named(&file, "Global Z")
                .and_then(|instance| instance.number_value_by_property_name("value")),
            Some(20.0)
        );
    }

    #[test]
    fn list_occurrences_keep_wrapper_identity_separate_from_instance_identity() {
        let file = global_context_fixture();
        let instance = Rc::new(RefCell::new(
            RuntimeOwnedViewModelInstance::from_instance(&file, 1, 0)
                .expect("main default instance"),
        ));
        let handle = RuntimeOwnedViewModelListHandle {
            value: Rc::new(RefCell::new(RuntimeOwnedViewModelListValue {
                item_count: 2,
                items: vec![
                    RuntimeOwnedViewModelListItem::new(Rc::clone(&instance)),
                    RuntimeOwnedViewModelListItem::new(instance),
                ],
            })),
        };

        let entries = handle.item_entries();
        assert_eq!(entries.len(), 2);
        assert_ne!(
            entries[0].occurrence_identity,
            entries[1].occurrence_identity
        );
        assert_eq!(
            entries[0].instance.instance_identity(),
            entries[1].instance.instance_identity()
        );
    }

    #[test]
    fn component_list_item_index_uses_cpp_symbol_registration_order() {
        let file = symbol_list_index_order_fixture();
        let mut imported =
            RuntimeOwnedViewModelInstance::from_instance(&file, 0, 0).expect("imported instance");

        assert!(set_component_list_item_index(&file, &mut imported, 7));
        assert_eq!(
            imported.symbol_list_index_value_by_property_path(&[0]),
            Some(7)
        );
        assert_eq!(
            imported.symbol_list_index_value_by_property_path(&[1]),
            Some(22)
        );

        let mut generated =
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("generated instance");
        assert!(set_component_list_item_index(&file, &mut generated, 9));
        assert_eq!(
            generated.symbol_list_index_value_by_property_path(&[0]),
            Some(0)
        );
        assert_eq!(
            generated.symbol_list_index_value_by_property_path(&[1]),
            Some(9)
        );
    }

    #[test]
    fn script_frame_advance_resets_embedded_view_model_triggers() {
        let file = nested_trigger_fixture();
        let mut instance =
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("generated root instance");
        assert!(instance.set_trigger_by_property_name_path("child/fire", 1));
        assert_eq!(instance.trigger_value_by_property_path(&[0, 0]), Some(1));

        let (changed, shared_children) = instance.advance_script_frame_local();

        assert!(changed);
        assert!(shared_children.is_empty());
        assert_eq!(instance.trigger_value_by_property_path(&[0, 0]), Some(0));
    }
}
