use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use nuxie_render_api::RenderImage;

use nuxie_binary::{
    RuntimeDataValue, RuntimeFile, RuntimeObject, RuntimeViewModel, RuntimeViewModelInstance,
    RuntimeViewModelInstanceReference,
};

pub use crate::view_model_cell::RuntimeFontAssetValue;
use crate::view_model_cell::{
    RuntimeCellDependent, RuntimeCellDirt, RuntimeCellDirtSink, RuntimeViewModelCell,
    RuntimeViewModelCellValue,
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

#[derive(Debug)]
pub struct RuntimeImportedViewModelInstanceContext {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    /// Contexts constructed by an artboard occurrence retain its canonical
    /// file-owned cells immediately. Detached compatibility contexts adopt
    /// those cells on bind; cloning preserves the public value-copy boundary.
    trigger_instance: RefCell<crate::view_model_cell::RuntimeViewModelInstanceCells>,
    adopted_trigger_owner_identity: Cell<Option<u64>>,
    adopt_full_trigger_snapshot: Cell<bool>,
    pub(crate) number_overrides: BTreeMap<Vec<u32>, f32>,
    pub(crate) boolean_overrides: BTreeMap<Vec<u32>, bool>,
    pub(crate) string_overrides: BTreeMap<Vec<u32>, Vec<u8>>,
    pub(crate) color_overrides: BTreeMap<Vec<u32>, u32>,
    pub(crate) enum_overrides: BTreeMap<Vec<u32>, u64>,
    pub(crate) symbol_list_index_overrides: BTreeMap<Vec<u32>, u64>,
    pub(crate) asset_overrides: BTreeMap<Vec<u32>, u64>,
    pub(crate) artboard_overrides: BTreeMap<Vec<u32>, u64>,
    pub(crate) list_overrides: BTreeMap<Vec<u32>, usize>,
    pub(crate) view_model_overrides: BTreeMap<Vec<u32>, RuntimeViewModelPointer>,
}

impl Clone for RuntimeImportedViewModelInstanceContext {
    fn clone(&self) -> Self {
        let adopt_full_trigger_snapshot = self.adopted_trigger_owner_identity.get().is_some()
            || self.adopt_full_trigger_snapshot.get();
        let trigger_instance = self.trigger_instance.borrow().detached_clone();
        Self {
            view_model_index: self.view_model_index,
            instance_index: self.instance_index,
            trigger_instance: RefCell::new(trigger_instance),
            adopted_trigger_owner_identity: Cell::new(None),
            adopt_full_trigger_snapshot: Cell::new(adopt_full_trigger_snapshot),
            number_overrides: self.number_overrides.clone(),
            boolean_overrides: self.boolean_overrides.clone(),
            string_overrides: self.string_overrides.clone(),
            color_overrides: self.color_overrides.clone(),
            enum_overrides: self.enum_overrides.clone(),
            symbol_list_index_overrides: self.symbol_list_index_overrides.clone(),
            asset_overrides: self.asset_overrides.clone(),
            artboard_overrides: self.artboard_overrides.clone(),
            list_overrides: self.list_overrides.clone(),
            view_model_overrides: self.view_model_overrides.clone(),
        }
    }
}

impl RuntimeImportedViewModelInstanceContext {
    pub fn new(file: &RuntimeFile, view_model_index: usize, instance_index: usize) -> Option<Self> {
        let view_model = file.view_model(view_model_index)?;
        view_model.instances.into_iter().nth(instance_index)?;
        let trigger_instance =
            crate::view_model_cell::RuntimeViewModelInstanceCells::from_serialized_instance(
                file,
                view_model_index,
                instance_index,
            )?;
        Some(Self {
            view_model_index,
            instance_index,
            trigger_instance: RefCell::new(trigger_instance),
            adopted_trigger_owner_identity: Cell::new(None),
            adopt_full_trigger_snapshot: Cell::new(false),
            number_overrides: BTreeMap::new(),
            boolean_overrides: BTreeMap::new(),
            string_overrides: BTreeMap::new(),
            color_overrides: BTreeMap::new(),
            enum_overrides: BTreeMap::new(),
            symbol_list_index_overrides: BTreeMap::new(),
            asset_overrides: BTreeMap::new(),
            artboard_overrides: BTreeMap::new(),
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

    pub(crate) fn trigger_owner_identity(&self) -> u64 {
        self.trigger_instance.borrow().allocation_identity()
    }

    pub(crate) fn from_file_trigger_instance(
        file: &RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
        trigger_instance: crate::view_model_cell::RuntimeViewModelInstanceCells,
    ) -> Option<Self> {
        let context = Self::new(file, view_model_index, instance_index)?;
        let owner_identity = trigger_instance.allocation_identity();
        *context.trigger_instance.borrow_mut() = trigger_instance;
        context
            .adopted_trigger_owner_identity
            .set(Some(owner_identity));
        Some(context)
    }

    pub(crate) fn trigger_cell_for_source_path(
        &self,
        path: &[u32],
    ) -> Option<RuntimeViewModelCell> {
        let cell = self.trigger_instance.borrow().cell_for_source_path(path)?;
        matches!(cell.value(), RuntimeViewModelCellValue::Trigger(_)).then_some(cell)
    }

    pub(crate) fn adopt_file_trigger_instance(
        &self,
        instance: crate::view_model_cell::RuntimeViewModelInstanceCells,
    ) -> bool {
        let owner_identity = instance.allocation_identity();
        if let Some(adopted_identity) = self.adopted_trigger_owner_identity.get() {
            return adopted_identity == owner_identity;
        }
        let copied = if self.adopt_full_trigger_snapshot.get() {
            self.trigger_instance
                .borrow()
                .copy_all_trigger_values_into(&instance)
        } else {
            true
        };
        if !copied {
            return false;
        }
        *self.trigger_instance.borrow_mut() = instance;
        self.adopted_trigger_owner_identity
            .set(Some(owner_identity));
        self.adopt_full_trigger_snapshot.set(false);
        true
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
        _file: &RuntimeFile,
        path: Vec<u32>,
        value: u64,
    ) -> bool {
        if self.adopted_trigger_owner_identity.get().is_none() {
            // A detached compatibility context has no file occurrence and
            // cannot reproduce C++ mutation order. Construct it through the
            // owning RuntimeArtboardInstance when pre-bind writes are needed.
            return false;
        }
        self.trigger_cell_for_source_path(&path)
            .is_some_and(|cell| cell.set_value(RuntimeViewModelCellValue::Trigger(value)))
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

#[path = "viewmodel/mod.rs"]
mod authored_viewmodel;

pub use authored_viewmodel::runtime::{
    RuntimeBindableArtboard, RuntimeViewModelImage, ViewModelInstanceArtboardRuntime,
    ViewModelInstanceAssetFontRuntime, ViewModelInstanceAssetImageRuntime,
    ViewModelInstanceBooleanRuntime, ViewModelInstanceColorRuntime, ViewModelInstanceEnumRuntime,
    ViewModelInstanceListIndexRuntime, ViewModelInstanceListRuntime,
    ViewModelInstanceNumberRuntime, ViewModelInstanceRuntime, ViewModelInstanceRuntimeProperty,
    ViewModelInstanceStringRuntime, ViewModelInstanceTriggerRuntime, ViewModelInstanceValueRuntime,
    ViewModelRuntime, ViewModelRuntimeDataType, ViewModelRuntimeProperty,
};
pub(crate) use authored_viewmodel::*;
pub use authored_viewmodel::{
    RuntimeOwnedViewModelArtboardSourceHandle, RuntimeOwnedViewModelAssetSourceHandle,
    RuntimeOwnedViewModelBooleanSourceHandle, RuntimeOwnedViewModelColorSourceHandle,
    RuntimeOwnedViewModelContext, RuntimeOwnedViewModelContextHandle,
    RuntimeOwnedViewModelEnumSourceHandle, RuntimeOwnedViewModelFontAssetSourceHandle,
    RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance,
    RuntimeOwnedViewModelListSourceHandle, RuntimeOwnedViewModelListStringMatchBooleanHandle,
    RuntimeOwnedViewModelNumberSourceHandle, RuntimeOwnedViewModelStringSourceHandle,
    RuntimeOwnedViewModelSymbolListIndexSourceHandle, RuntimeOwnedViewModelTriggerSourceHandle,
    RuntimeOwnedViewModelViewModelSourceHandle, RuntimeViewModelLinkError,
    runtime_global_view_model_indices, runtime_global_view_model_names,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeViewModelPointer {
    Null,
    DataContextRoot,
    /// Exact identity of a runtime-linked retained child. The data-bind
    /// source separately owns the endpoint containing the child `Rc`; this
    /// value is its pointer-shaped read model for bindable comparisons.
    Retained {
        allocation_identity: u64,
    },
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

    /// Resolve legacy ProjectDO relative paths whose ids are FNV-1a hashes of
    /// UTF-8 property names rather than Rive manifest-name ordinals.
    ///
    /// A hash that names more than one property is deliberately unresolved:
    /// hash-only input cannot safely recover the original property identity.
    pub(crate) fn project_relative_property_by_name_hash_path(
        &self,
        path: &[u32],
    ) -> Option<&'a RuntimeObject> {
        self.project_relative_property_by_name_segments(path, |name_hash, name| {
            runtime_project_name_hash(name) == *name_hash
        })
    }

    /// Resolve ProjectDO relative paths by their exact UTF-8 property names.
    pub(crate) fn project_relative_property_by_name_path(
        &self,
        path: &[String],
    ) -> Option<&'a RuntimeObject> {
        self.project_relative_property_by_name_segments(path, |property_name, name| {
            property_name == name
        })
    }

    fn project_relative_property_by_name_segments<T>(
        &self,
        path: &[T],
        matches_name: impl Fn(&T, &str) -> bool,
    ) -> Option<&'a RuntimeObject> {
        if path.is_empty() {
            return None;
        }
        for candidate in self.instance_chain() {
            let mut instance = candidate;
            let mut failed = false;
            for (index, segment) in path.iter().enumerate() {
                let view_model_index = instance
                    .uint_property("viewModelId")
                    .and_then(|value| usize::try_from(value).ok());
                let Some(view_model) =
                    view_model_index.and_then(|index| self.file.view_model(index))
                else {
                    failed = true;
                    break;
                };
                let mut matches = view_model.properties.iter().enumerate().filter_map(
                    |(property_index, property)| {
                        property
                            .string_property("name")
                            .is_some_and(|name| matches_name(segment, name))
                            .then_some(property_index)
                    },
                );
                let Some(property_index) = matches.next() else {
                    failed = true;
                    break;
                };
                if matches.next().is_some() {
                    return None;
                }
                let Some(property_index) = u32::try_from(property_index).ok() else {
                    failed = true;
                    break;
                };
                let Some(value) = self
                    .file
                    .view_model_instance_value_for_property_id_object(instance, property_index)
                else {
                    failed = true;
                    break;
                };
                if path.get(index.saturating_add(1)).is_none() {
                    return Some(value);
                }
                let Some(reference) = self
                    .file
                    .referenced_view_model_instance_for_value_object(value)
                else {
                    failed = true;
                    break;
                };
                instance = reference.object;
            }
            if !failed {
                return None;
            }
        }
        None
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
    use crate::view_model_cell::{RuntimeCellDirtSink, RuntimeCellNotificationQueue};
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

    fn mutable_list_default_fixture() -> RuntimeFile {
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
                "ViewModelPropertyList",
                vec![property(
                    "ViewModelPropertyList",
                    "name",
                    AuthoringValue::String("items".to_owned()),
                )],
            ),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Item".to_owned()),
                )],
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
                    property("ViewModelInstance", "viewModelId", AuthoringValue::Uint(0)),
                    property(
                        "ViewModelInstance",
                        "name",
                        AuthoringValue::String("Root Default".to_owned()),
                    ),
                ],
            ),
            record(
                "ViewModelInstanceList",
                vec![property(
                    "ViewModelInstanceList",
                    "viewModelPropertyId",
                    AuthoringValue::Uint(0),
                )],
            ),
            record(
                "ViewModelInstance",
                vec![
                    property("ViewModelInstance", "viewModelId", AuthoringValue::Uint(1)),
                    property(
                        "ViewModelInstance",
                        "name",
                        AuthoringValue::String("Item Default".to_owned()),
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
                        AuthoringValue::Double(10.0),
                    ),
                ],
            ),
            record(
                "ViewModelInstanceListItem",
                vec![
                    property(
                        "ViewModelInstanceListItem",
                        "viewModelId",
                        AuthoringValue::Uint(1),
                    ),
                    property(
                        "ViewModelInstanceListItem",
                        "viewModelInstanceId",
                        AuthoringValue::Uint(0),
                    ),
                ],
            ),
        ])
        .expect("mutable list default fixture imports")
    }

    fn mutable_list_trigger_fixture() -> RuntimeFile {
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
                "ViewModelPropertyList",
                vec![property(
                    "ViewModelPropertyList",
                    "name",
                    AuthoringValue::String("items".to_owned()),
                )],
            ),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Item".to_owned()),
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
        .expect("mutable list trigger fixture imports")
    }

    #[test]
    fn owned_advance_context_tracks_live_list_insertion_and_removal() {
        let file = mutable_list_trigger_fixture();
        let root = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("root"),
        );
        let row = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("row"),
        );
        let mut advance_context = RuntimeOwnedViewModelAdvanceContext::default();
        advance_context.extend(&root.borrow());

        assert!(root.insert_list_item_by_property_name_path("items", 0, &row));
        assert!(row.borrow_mut().set_trigger_by_property_name("fire", 1));
        advance_context.advanced();
        assert_eq!(row.borrow().trigger_value_by_property_name("fire"), Some(0));

        assert!(row.borrow_mut().set_trigger_by_property_name("fire", 1));
        assert!(root.remove_list_item_by_property_name_path("items", 0));
        advance_context.advanced();
        assert_eq!(
            row.borrow().trigger_value_by_property_name("fire"),
            Some(1),
            "a removed row is no longer part of C++ DataContext::advanced"
        );
    }

    fn list_row_relink_fixture() -> RuntimeFile {
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
                "ViewModelPropertyList",
                vec![property(
                    "ViewModelPropertyList",
                    "name",
                    AuthoringValue::String("items".to_owned()),
                )],
            ),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Row".to_owned()),
                )],
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
                        AuthoringValue::Uint(2),
                    ),
                ],
            ),
            record(
                "ViewModelPropertyList",
                vec![property(
                    "ViewModelPropertyList",
                    "name",
                    AuthoringValue::String("nested".to_owned()),
                )],
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
                "ViewModelPropertyNumber",
                vec![property(
                    "ViewModelPropertyNumber",
                    "name",
                    AuthoringValue::String("value".to_owned()),
                )],
            ),
            record(
                "ViewModelPropertyViewModel",
                vec![
                    property(
                        "ViewModelPropertyViewModel",
                        "name",
                        AuthoringValue::String("leaf".to_owned()),
                    ),
                    property(
                        "ViewModelPropertyViewModel",
                        "viewModelReferenceId",
                        AuthoringValue::Uint(3),
                    ),
                ],
            ),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Leaf".to_owned()),
                )],
            ),
        ])
        .expect("list-row relink fixture imports")
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
                .map(|value| value.file_asset_index()),
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
                .map(|instance| instance.view_model_index())
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
                .map(|instance| instance.view_model_index())
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
                parent_relay: Weak::new(),
                item_count: 2,
                items: vec![
                    RuntimeOwnedViewModelListItem::new(Rc::clone(&instance)),
                    RuntimeOwnedViewModelListItem::new(instance),
                ],
            })),
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::List),
        };

        let entries = handle.item_entries();
        assert_eq!(entries.len(), 2);
        assert_ne!(
            entries[0].occurrence_identity,
            entries[1].occurrence_identity
        );
        assert_eq!(
            entries[0].instance.borrow().instance_identity(),
            entries[1].instance.borrow().instance_identity()
        );
        assert!(entries[0].instance.ptr_eq(&entries[1].instance));
    }

    #[test]
    fn owned_context_clones_retain_main_handle_identity_and_mutations() {
        let file = global_context_fixture();
        let handle = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::from_instance(&file, 1, 0)
                .expect("main default instance"),
        );
        let context = RuntimeOwnedViewModelContext::from_main_handle(handle.clone());
        let alias = context.clone();

        assert!(
            context
                .main_handle()
                .is_some_and(|main| main.ptr_eq(&handle))
        );
        assert!(alias.main_handle().is_some_and(|main| main.ptr_eq(&handle)));
        assert!(
            alias
                .main_mut()
                .is_some_and(|mut main| main.set_number_by_property_name("value", 42.0))
        );
        assert_eq!(
            context
                .main()
                .and_then(|main| main.number_value_by_property_name("value")),
            Some(42.0)
        );
    }

    #[test]
    fn detached_instance_clone_owns_an_independent_value_cell() {
        let file = global_context_fixture();
        let mut source = RuntimeOwnedViewModelInstance::from_instance(&file, 1, 0)
            .expect("main default instance");
        assert!(source.set_number_by_property_name("value", 42.0));
        let mut detached = source.clone();
        assert_eq!(detached.instance_identity(), source.instance_identity());
        let source_cell = source
            .cell_by_property_path(&[0])
            .expect("source value cell");
        let detached_cell = detached
            .cell_by_property_path(&[0])
            .expect("detached value cell");
        assert!(!source_cell.ptr_eq(&detached_cell));

        assert!(detached.set_number_by_property_name("value", 43.0));
        assert_eq!(source.number_value_by_property_name("value"), Some(42.0));
    }

    fn linked_child_fixture() -> RuntimeFile {
        let mut records = vec![
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
                "ViewModelPropertyViewModel",
                vec![
                    property(
                        "ViewModelPropertyViewModel",
                        "name",
                        AuthoringValue::String("child2".to_owned()),
                    ),
                    property(
                        "ViewModelPropertyViewModel",
                        "viewModelReferenceId",
                        AuthoringValue::Uint(1),
                    ),
                ],
            ),
        ];
        let mut child_records = view_model_records("Child", 0, 1, 5.0);
        child_records.insert(
            2,
            record(
                "ViewModelPropertyAssetFont",
                vec![property(
                    "ViewModelPropertyAssetFont",
                    "name",
                    AuthoringValue::String("font".to_owned()),
                )],
            ),
        );
        child_records.push(record(
            "ViewModelInstanceAssetFont",
            vec![
                property(
                    "ViewModelInstanceAssetFont",
                    "viewModelPropertyId",
                    AuthoringValue::Uint(1),
                ),
                property(
                    "ViewModelInstanceAssetFont",
                    "propertyValue",
                    AuthoringValue::Uint(0),
                ),
            ],
        ));
        records.extend(child_records);
        RuntimeFile::from_authoring_records(records).expect("linked child fixture imports")
    }

    fn linked_structural_endpoint_fixture() -> RuntimeFile {
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
                "ViewModelPropertyViewModel",
                vec![
                    property(
                        "ViewModelPropertyViewModel",
                        "name",
                        AuthoringValue::String("leaf".to_owned()),
                    ),
                    property(
                        "ViewModelPropertyViewModel",
                        "viewModelReferenceId",
                        AuthoringValue::Uint(2),
                    ),
                ],
            ),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Leaf".to_owned()),
                )],
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
                    property("ViewModelInstance", "viewModelId", AuthoringValue::Uint(2)),
                    property(
                        "ViewModelInstance",
                        "name",
                        AuthoringValue::String("First".to_owned()),
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
                        AuthoringValue::Double(1.0),
                    ),
                ],
            ),
            record(
                "ViewModelInstance",
                vec![
                    property("ViewModelInstance", "viewModelId", AuthoringValue::Uint(2)),
                    property(
                        "ViewModelInstance",
                        "name",
                        AuthoringValue::String("Second".to_owned()),
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
                        AuthoringValue::Double(2.0),
                    ),
                ],
            ),
        ])
        .expect("linked structural endpoint fixture imports")
    }

    /// #RB-1 e2b: nested children are shared by identity within a live graph
    /// (C++ rcp children), while `Clone` stays a deep copy that ports C++
    /// `copyViewModelInstance`'s instancesMap — internal sharing topology
    /// survives inside the copy without sharing anything with the source.
    #[test]
    fn linked_children_share_identity_and_clones_preserve_topology() {
        let file = linked_child_fixture();
        let owner_a = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("root instance"),
        );
        let owner_b = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("root instance"),
        );
        let child = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::from_instance(&file, 1, 0).expect("child instance"),
        );

        assert_eq!(
            owner_a.link_view_model_by_property_name_path("child", &child),
            Ok(true)
        );
        assert_eq!(
            owner_a.link_view_model_by_property_name_path("child2", &child),
            Ok(true)
        );
        assert_eq!(
            owner_b.link_view_model_by_property_name_path("child", &child),
            Ok(true)
        );
        assert_eq!(
            owner_b.link_view_model_by_property_name_path("child", &child),
            Ok(true),
            "C++ re-runs detach/attach, dirt, and relink for the same retained child pointer"
        );

        let child_font_cell = child
            .borrow()
            .cell_by_property_path(&[1])
            .expect("child font has a retained cell");
        let owner_font_cell = owner_a
            .borrow()
            .cell_by_property_path(&[0, 1])
            .expect("owner path reaches the retained child font cell");
        assert!(
            child_font_cell.ptr_eq(&owner_font_cell),
            "linked AssetFont paths retain the exact child cell"
        );

        // One write through the retained child is visible through every
        // owner: two references to the same logical child hold the same
        // underlying cells.
        assert!(
            child
                .borrow_mut()
                .set_number_by_property_name("value", 77.0)
        );
        assert_eq!(
            owner_a
                .borrow()
                .number_value_by_property_name_path("child/value"),
            Some(77.0)
        );
        assert_eq!(
            owner_b
                .borrow()
                .number_value_by_property_name_path("child/value"),
            Some(77.0)
        );

        // Writes through one owner's path land in the shared child, live.
        assert!(
            owner_a
                .borrow_mut()
                .set_number_by_property_name_path("child/value", 78.0)
        );
        assert_eq!(
            child.borrow().number_value_by_property_name("value"),
            Some(78.0)
        );
        assert_eq!(
            owner_b
                .borrow()
                .number_value_by_property_name_path("child/value"),
            Some(78.0)
        );
        assert_eq!(
            owner_a
                .borrow()
                .number_value_by_property_name_path("child2/value"),
            Some(78.0),
            "the second slot on the writing owner shares the same child"
        );

        // AssetFont's complete two-part payload lives on the one retained
        // cell, matching C++'s retained child plus
        // `ViewModelInstanceAssetFont` (`viewmodel_instance_viewmodel.hpp:
        // 19-39`, `viewmodel_instance_asset_font.cpp:13-75`). A parent borrow
        // held across a direct child write must therefore re-read it live.
        let held_owner_a = owner_a.borrow();
        let child_live: Arc<[u8]> = vec![1, 2, 3, 4].into();
        assert!(
            child
                .borrow_mut()
                .set_live_font_bytes_by_property_name("font", Some(Arc::clone(&child_live)))
        );
        assert!(
            held_owner_a
                .font_asset_value_by_property_path(&[0, 1])
                .and_then(|value| value.live_font_bytes_arc().cloned())
                .is_some_and(|value| Arc::ptr_eq(&value, &child_live))
        );
        drop(held_owner_a);

        let owner_live: Arc<[u8]> = vec![5, 6, 7, 8].into();
        assert!(
            owner_a
                .borrow_mut()
                .set_live_font_bytes_by_property_path(&[0, 1], Some(Arc::clone(&owner_live)),)
        );
        assert!(
            child
                .borrow()
                .font_asset_value_by_property_name("font")
                .and_then(|value| value.live_font_bytes_arc().cloned())
                .is_some_and(|value| Arc::ptr_eq(&value, &owner_live))
        );
        assert!(
            owner_b
                .borrow_mut()
                .set_font_asset_index_by_property_name_path("child/font", 7)
        );
        assert_eq!(
            child
                .borrow()
                .font_asset_value_by_property_name("font")
                .map(|value| value.file_asset_index()),
            Some(7),
        );
        assert!(
            child
                .borrow()
                .font_asset_value_by_property_name("font")
                .and_then(|value| value.live_font_bytes_arc().cloned())
                .is_some_and(|value| Arc::ptr_eq(&value, &owner_live)),
            "a propertyValue write preserves C++'s private live-Font fallback"
        );
        assert_eq!(
            owner_a
                .borrow()
                .font_asset_value_by_property_path(&[0, 1])
                .map(|value| value.file_asset_index()),
            Some(7),
        );

        // A retained ViewModel endpoint also carries structural selection,
        // not just scalar cells. The C++ setter stores the retained child and
        // calls `propertyValueChanged` immediately
        // (`viewmodel_instance_viewmodel.hpp:23-35`); replacement then
        // synchronously relinks dependents (`viewmodel_instance.cpp:118-188`).
        // Holding the parent borrow across the child's nested replacement must
        // therefore expose the new active leaf without a retry or next frame.
        let structural_file = linked_structural_endpoint_fixture();
        let structural_owner = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&structural_file, 0).expect("structural root"),
        );
        let structural_child = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&structural_file, 1).expect("structural child"),
        );
        assert_eq!(
            structural_owner.link_view_model_by_property_name_path("child", &structural_child),
            Ok(true)
        );
        let structural_rebind = RuntimeCellDirtSink::new();
        structural_owner.add_rebind_dependent(&structural_rebind);
        let first_leaf_cell = structural_owner
            .borrow()
            .cell_by_property_path(&[0, 0, 0])
            .expect("first nested source cell");
        let second_leaf = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::from_instance(&structural_file, 2, 1)
                .expect("second leaf"),
        );
        let held_structural_owner = structural_owner.borrow();
        assert_eq!(
            structural_child.link_view_model_by_property_name_path("leaf", &second_leaf),
            Ok(true)
        );
        assert!(
            structural_rebind
                .take_dirt()
                .contains(RuntimeCellDirt::BINDINGS),
            "nested replacement pushes DataContext relink dirt instead of waiting for a generation poll"
        );
        let second_leaf_cell = held_structural_owner
            .cell_by_property_path(&[0, 0, 0])
            .expect("replacement nested source cell");
        assert!(!first_leaf_cell.ptr_eq(&second_leaf_cell));
        assert_eq!(
            held_structural_owner.number_value_by_property_path(&[0, 0, 0]),
            Some(2.0),
            "retained structural selection is live under an existing parent borrow"
        );
        drop(held_structural_owner);
        let structural_clone = RuntimeOwnedViewModelHandle::new(structural_owner.borrow().clone());
        assert!(
            second_leaf
                .borrow_mut()
                .set_number_by_property_name("value", 3.0)
        );
        assert_eq!(
            structural_owner
                .borrow()
                .number_value_by_property_path(&[0, 0, 0]),
            Some(3.0)
        );
        assert_eq!(
            structural_clone
                .borrow()
                .number_value_by_property_path(&[0, 0, 0]),
            Some(2.0),
            "deep copy detaches the structural endpoint from the source graph"
        );

        // A ViewModel-valued property has one active retained child in C++,
        // not an imported selection plus an independent linked override.
        // Selecting an authored instance after an explicit link therefore
        // detaches that link (`viewmodel_instance_viewmodel.hpp:15-35`).
        let selection_owner = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("selection root"),
        );
        assert!(
            selection_owner
                .borrow_mut()
                .set_view_model_by_property_name_path("child", 0)
        );
        assert_eq!(
            selection_owner
                .borrow()
                .number_value_by_property_name_path("child/value"),
            Some(5.0)
        );
        assert_eq!(
            selection_owner.link_view_model_by_property_name_path("child", &child),
            Ok(true)
        );
        assert_eq!(
            selection_owner
                .borrow()
                .number_value_by_property_name_path("child/value"),
            Some(78.0)
        );
        assert!(
            selection_owner
                .borrow_mut()
                .set_view_model_by_property_name_path("child", 0)
        );
        assert!(
            selection_owner
                .linked_view_model_by_property_name_path("child")
                .is_none(),
            "authored selection replaces rather than layers over the retained child"
        );
        assert_eq!(
            selection_owner
                .borrow()
                .number_value_by_property_name_path("child/value"),
            Some(5.0)
        );

        let bind_live: Arc<[u8]> = vec![9, 10, 11, 12].into();
        let mut bind_value = RuntimeFontAssetValue::default();
        assert!(bind_value.set_live_font_bytes(Some(Arc::clone(&bind_live))));
        assert!(
            owner_a
                .borrow_mut()
                .apply_font_asset_data_bind_value_by_property_path(&[0, 1], &bind_value)
        );
        assert!(
            child
                .borrow()
                .font_asset_value_by_property_name("font")
                .and_then(|value| value.live_font_bytes_arc().cloned())
                .is_some_and(|value| Arc::ptr_eq(&value, &bind_live)),
            "target-to-source application mutates the retained child payload"
        );

        // Clone is a DEEP copy: nothing is shared with the source graph...
        let cloned = RuntimeOwnedViewModelHandle::new(owner_a.borrow().clone());
        assert!(
            cloned
                .borrow_mut()
                .set_number_by_property_name_path("child/value", 99.0)
        );
        assert_eq!(
            child.borrow().number_value_by_property_name("value"),
            Some(78.0),
            "a deep copy must not write through to the source graph"
        );

        // ...but the copy preserves internal sharing topology: both child
        // slots reference ONE copied child (one instancesMap per clone).
        let cloned_child = cloned
            .linked_view_model_by_property_name_path("child")
            .expect("the copy retains its linked child");
        let cloned_child2 = cloned
            .linked_view_model_by_property_name_path("child2")
            .expect("the copy retains its second linked child");
        assert!(
            cloned_child.ptr_eq(&cloned_child2),
            "a child referenced twice is copied once and referenced twice"
        );
        assert_eq!(
            cloned_child.borrow().number_value_by_property_name("value"),
            Some(99.0)
        );
        assert_eq!(
            cloned
                .borrow()
                .number_value_by_property_name_path("child2/value"),
            Some(99.0),
            "the second slot observes the shared copied child"
        );
        assert!(
            cloned_child
                .borrow()
                .font_asset_value_by_property_name("font")
                .is_some_and(|value| value.live_font_bytes_arc().is_none()),
            "pinned C++ Font clone constructs a fresh empty private FontAsset"
        );
    }

    #[test]
    fn string_endpoint_shares_identity_while_clone_copies_bytes() {
        let source = RuntimeOwnedViewModelString::new(0, b"initial".to_vec());
        let mut shared = source.share();
        assert!(source.cell.ptr_eq(&shared.cell));
        assert!(shared.set_value(b"linked"));
        assert_eq!(source.value().as_ref(), b"linked");

        let mut cloned = source.clone();
        assert!(!source.cell.ptr_eq(&cloned.cell));
        assert_eq!(cloned.value().as_ref(), b"linked");
        assert!(cloned.set_value(b"copy"));
        assert_eq!(source.value().as_ref(), b"linked");
    }

    #[test]
    fn font_endpoint_preserves_pinned_setter_dirt_multiplicity() {
        fn observed_font(
            index: u64,
        ) -> (
            RuntimeOwnedViewModelFontAsset,
            RuntimeCellNotificationQueue,
            RuntimeCellDirtSink,
        ) {
            let font = RuntimeOwnedViewModelFontAsset::new(
                0,
                RuntimeFontAssetValue::from_file_asset_index(index),
            );
            let queue = RuntimeCellNotificationQueue::default();
            let sink = RuntimeCellDirtSink::reporting_listener(&queue, 0);
            font.cell.add_dependent(&sink);
            (font, queue, sink)
        }

        fn drain(queue: &RuntimeCellNotificationQueue) -> usize {
            let mut reports = Vec::new();
            queue.swap_into(&mut reports);
            reports.len()
        }

        let first: Arc<[u8]> = vec![1].into();
        let second: Arc<[u8]> = vec![2].into();

        // `value(Font*)` early-return path: the same pointer only forces the
        // sentinel, so it reports once from a non-sentinel index and zero
        // times when already sentinel (asset_font.cpp:29-34).
        let (mut same, same_queue, _same_sink) = observed_font(7);
        assert!(same.set_live_font_bytes(Some(Arc::clone(&first))));
        drain(&same_queue);
        assert!(same.set_file_asset_index(7));
        drain(&same_queue);
        assert!(same.set_live_font_bytes(Some(Arc::clone(&first))));
        assert_eq!(drain(&same_queue), 1);
        assert!(!same.set_live_font_bytes(Some(Arc::clone(&first))));
        assert_eq!(drain(&same_queue), 0);

        // A different pointer from a non-sentinel index reports once for
        // propertyValue(-1) and once for the unconditional live-Font dirt;
        // from the sentinel it reports only the latter (lines 35-61).
        assert!(same.set_file_asset_index(7));
        drain(&same_queue);
        assert!(same.set_live_font_bytes(Some(Arc::clone(&second))));
        assert_eq!(drain(&same_queue), 2);
        assert!(same.set_live_font_bytes(Some(Arc::clone(&first))));
        assert_eq!(drain(&same_queue), 1);

        // Null DataValueAssetFont applies value(nullptr) and then the index,
        // preserving both transient sentinel reports even when the final
        // payload equals the start (lines 64-75).
        let (mut null_apply, null_queue, _null_sink) = observed_font(7);
        let null_value = RuntimeFontAssetValue::from_file_asset_index(7);
        assert!(null_apply.apply_data_bind_value(&null_value));
        assert_eq!(drain(&null_queue), 2);
        assert_eq!(null_apply.value().file_asset_index(), 7);

        let (mut live_apply, live_queue, _live_sink) = observed_font(7);
        let mut live_value = RuntimeFontAssetValue::default();
        assert!(live_value.set_live_font_bytes(Some(Arc::clone(&first))));
        assert!(live_apply.apply_data_bind_value(&live_value));
        assert_eq!(drain(&live_queue), 2);
        assert_eq!(
            live_apply.value().file_asset_index(),
            RuntimeFontAssetValue::MISSING_FILE_ASSET_INDEX
        );
    }

    #[test]
    fn dynamic_list_row_relink_reaches_every_parent_while_scalar_dirt_stays_local() {
        let file = list_row_relink_fixture();
        let root_a = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("first root"),
        );
        let root_b = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("second root"),
        );
        let row = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("shared row"),
        );
        let first_child = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2).expect("first child"),
        );
        let second_child = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2).expect("second child"),
        );
        assert_eq!(
            row.link_view_model_by_property_name_path("child", &first_child),
            Ok(true)
        );
        assert!(root_a.insert_list_item_by_property_name_path("items", 0, &row));
        assert!(root_b.insert_list_item_by_property_name_path("items", 0, &row));
        let root_a_rebind = RuntimeCellDirtSink::new();
        let root_b_rebind = RuntimeCellDirtSink::new();
        root_a.add_rebind_dependent(&root_a_rebind);
        root_b.add_rebind_dependent(&root_b_rebind);

        assert!(row.borrow_mut().set_number_by_property_name("value", 1.0));
        assert!(root_a_rebind.take_dirt().is_empty());
        assert!(root_b_rebind.take_dirt().is_empty());

        // C++ `replaceViewModelByName` calls `rebindDependents`, which walks
        // every pointer-unique parent (`viewmodel_instance.cpp:118-154,406-415`).
        assert_eq!(
            row.link_view_model_by_property_name_path("child", &second_child),
            Ok(true)
        );
        assert!(
            root_a_rebind
                .take_dirt()
                .contains(RuntimeCellDirt::BINDINGS),
            "C++ rebindDependents pushes relink dirt to the containing DataBindContainer"
        );
        assert!(
            root_b_rebind
                .take_dirt()
                .contains(RuntimeCellDirt::BINDINGS)
        );
    }

    #[test]
    fn removing_dynamic_list_row_detaches_only_that_parent() {
        let file = list_row_relink_fixture();
        let root_a = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("first root"),
        );
        let root_b = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("second root"),
        );
        let row = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("shared row"),
        );
        assert!(root_a.insert_list_item_by_property_name_path("items", 0, &row));
        assert!(root_b.insert_list_item_by_property_name_path("items", 0, &row));
        assert!(root_a.remove_list_item_by_property_name_path("items", 0));
        let root_a_rebind = RuntimeCellDirtSink::new();
        let root_b_rebind = RuntimeCellDirtSink::new();
        root_a.add_rebind_dependent(&root_a_rebind);
        root_b.add_rebind_dependent(&root_b_rebind);
        let child = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2).expect("replacement child"),
        );

        assert_eq!(
            row.link_view_model_by_property_name_path("child", &child),
            Ok(true)
        );
        assert!(root_a_rebind.take_dirt().is_empty());
        assert!(
            root_b_rebind
                .take_dirt()
                .contains(RuntimeCellDirt::BINDINGS)
        );
    }

    #[test]
    fn detached_graph_rebuilds_dynamic_list_parent_relays_without_source_leakage() {
        let file = list_row_relink_fixture();
        let source = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("source root"),
        );
        let source_row = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("source row"),
        );
        assert!(source.insert_list_item_by_property_name_path("items", 0, &source_row));
        let detached = RuntimeOwnedViewModelHandle::detached_graph(std::slice::from_ref(&source))
            .pop()
            .expect("detached root");
        let detached_row = detached
            .list_items_by_property_name_path("items")
            .and_then(|mut items| items.pop())
            .expect("detached row");
        assert!(!detached_row.ptr_eq(&source_row));
        let source_rebind = RuntimeCellDirtSink::new();
        let detached_rebind = RuntimeCellDirtSink::new();
        source.add_rebind_dependent(&source_rebind);
        detached.add_rebind_dependent(&detached_rebind);
        let child = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2).expect("detached child"),
        );

        assert_eq!(
            detached_row.link_view_model_by_property_name_path("child", &child),
            Ok(true)
        );
        assert!(source_rebind.take_dirt().is_empty());
        assert!(
            detached_rebind
                .take_dirt()
                .contains(RuntimeCellDirt::BINDINGS)
        );
    }

    #[test]
    fn nested_dynamic_list_row_relink_reaches_the_root_relay() {
        let file = list_row_relink_fixture();
        let root = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("root"),
        );
        let row = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("row"),
        );
        let nested = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2).expect("nested row"),
        );
        assert!(root.insert_list_item_by_property_name_path("items", 0, &row));
        assert!(row.insert_list_item_by_property_name_path("nested", 0, &nested));
        let root_rebind = RuntimeCellDirtSink::new();
        root.add_rebind_dependent(&root_rebind);
        let leaf = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 3).expect("leaf"),
        );

        assert_eq!(
            nested.link_view_model_by_property_name_path("leaf", &leaf),
            Ok(true)
        );
        assert!(root_rebind.take_dirt().contains(RuntimeCellDirt::BINDINGS));
    }

    #[test]
    fn duplicate_list_occurrence_removal_matches_cpp_pointer_unique_parent_semantics() {
        let file = list_row_relink_fixture();
        let root = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("root"),
        );
        let row = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("shared row"),
        );
        assert!(root.insert_list_item_by_property_name_path("items", 0, &row));
        assert!(root.insert_list_item_by_property_name_path("items", 1, &row));
        assert!(root.remove_list_item_by_property_name_path("items", 0));
        assert_eq!(root.list_item_count_by_property_name_path("items"), Some(1));
        let root_rebind = RuntimeCellDirtSink::new();
        root.add_rebind_dependent(&root_rebind);
        let child = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2).expect("child"),
        );

        // C++ addParent dedupes by parent pointer, while removeParent erases
        // it outright (`viewmodel_instance.cpp:346-363`).
        assert_eq!(
            row.link_view_model_by_property_name_path("child", &child),
            Ok(true)
        );
        assert!(root_rebind.take_dirt().is_empty());
    }

    #[test]
    fn list_pop_preserves_but_shift_removes_the_pinned_parent_registration() {
        let file = list_row_relink_fixture();
        let popped_root = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("pop root"),
        );
        let popped_row = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("popped row"),
        );
        assert!(popped_root.insert_list_item_by_property_name_path("items", 0, &popped_row));
        assert!(
            popped_root
                .borrow_mut()
                .pop_list_item_by_property_path(&[0])
                .is_some()
        );
        let popped_root_rebind = RuntimeCellDirtSink::new();
        popped_root.add_rebind_dependent(&popped_root_rebind);
        let first_child = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2).expect("first child"),
        );
        assert_eq!(
            popped_row.link_view_model_by_property_name_path("child", &first_child),
            Ok(true)
        );
        assert!(
            popped_root_rebind
                .take_dirt()
                .contains(RuntimeCellDirt::BINDINGS)
        );

        let shifted_root = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("shift root"),
        );
        let shifted_row = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("shifted row"),
        );
        assert!(shifted_root.insert_list_item_by_property_name_path("items", 0, &shifted_row));
        assert!(
            shifted_root
                .borrow_mut()
                .shift_list_item_by_property_path(&[0])
                .is_some()
        );
        let shifted_root_rebind = RuntimeCellDirtSink::new();
        shifted_root.add_rebind_dependent(&shifted_root_rebind);
        let second_child = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2).expect("second child"),
        );
        assert_eq!(
            shifted_row.link_view_model_by_property_name_path("child", &second_child),
            Ok(true)
        );
        assert!(
            shifted_root_rebind.take_dirt().is_empty(),
            "C++ shift delegates to removeItem and detaches; pop does not (`viewmodel_instance_list.cpp:76-114`)"
        );
    }

    #[test]
    fn same_index_swap_and_empty_update_dirty_the_exact_list_property() {
        let file = list_row_relink_fixture();
        let root = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("root"),
        );
        let row = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("row"),
        );
        assert!(root.insert_list_item_by_property_name_path("items", 0, &row));
        let cell = root
            .borrow()
            .cell_by_property_path(&[0])
            .expect("retained list property");
        let sink = RuntimeCellDirtSink::new();
        cell.add_dependent(&sink);

        // C++ dirties every in-range swap, including i == j
        // (`viewmodel_instance_list.cpp:183-190`).
        assert!(root.swap_list_items_by_property_name_path("items", 0, 0));
        assert!(sink.take_dirt().contains(RuntimeCellDirt::BINDINGS));

        let empty = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("empty root"),
        );
        let (source, empty_cell) = {
            let instance = empty.borrow();
            let source = instance
                .list_source_handle_by_property_name("items")
                .expect("list source");
            let cell = instance
                .cell_by_property_path(source.path())
                .expect("empty list cell");
            (source, cell)
        };
        let empty_sink = RuntimeCellDirtSink::new();
        empty_cell.add_dependent(&empty_sink);
        assert_eq!(
            empty
                .borrow_mut()
                .replace_list_items_by_source_handle(&source, Vec::new()),
            Some(true)
        );
        assert!(empty_sink.take_dirt().contains(RuntimeCellDirt::BINDINGS));
    }

    #[test]
    fn retained_view_model_source_reads_the_current_linked_child_identity() {
        let file = list_row_relink_fixture();
        let row = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("row"),
        );
        let source = row
            .borrow()
            .structural_source_by_property_path(&[1])
            .expect("child endpoint source");
        let child = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2).expect("child"),
        );
        let child_identity = child.borrow().instance_identity();
        let child_allocation = child.borrow().allocation_identity();

        assert_eq!(
            row.link_view_model_by_property_name_path("child", &child),
            Ok(true)
        );
        assert_eq!(
            row.borrow().view_model_index_by_property_path(&[1]),
            Some(2),
            "C++ DataContext::tryGetViewModelInstance follows the live ViewModelInstanceViewModel reference when the authored path ends at that property (`data_context.cpp:335-363`)"
        );
        assert_eq!(
            source.view_model_pointer(),
            Some(RuntimeViewModelPointer::Retained {
                allocation_identity: child_allocation,
            })
        );

        let detached = child.borrow().clone();
        assert_eq!(detached.instance_identity(), child_identity);
        assert_ne!(detached.allocation_identity(), child_allocation);
    }

    #[test]
    fn authored_list_rows_start_without_a_parent_registration() {
        let file = mutable_list_default_fixture();
        let root =
            RuntimeOwnedViewModelInstance::from_instance(&file, 0, 0).expect("authored root");
        let row = root
            .list_items_by_property_name("items")
            .and_then(|mut items| items.pop())
            .expect("authored row");

        assert!(row.borrow().parent_relay.parents.borrow().is_empty());
    }

    #[test]
    fn mutable_imported_list_items_keep_row_local_cell_dirt() {
        let file = mutable_list_default_fixture();
        let target = RuntimeOwnedViewModelInstance::from_instance(&file, 0, 0)
            .expect("mutable imported root instance");
        let list_source = target
            .list_source_handle_by_property_name("items")
            .expect("root list source");
        let list = target
            .list_handle_by_property_path(list_source.path())
            .expect("mutable imported list");
        let row = list
            .item_entries()
            .into_iter()
            .next()
            .expect("imported list row");
        let occurrence_identity = row.occurrence_identity;
        let root_rebind = RuntimeCellDirtSink::new();
        target.parent_relay.add_dependent(&root_rebind);
        let row_cell = row
            .instance
            .borrow()
            .cell_by_property_path(&[0])
            .expect("row value cell");
        let row_dirt = RuntimeCellDirtSink::new();
        row_cell.add_dependent(&row_dirt);

        assert!(
            row.instance
                .borrow_mut()
                .set_number_by_property_name("value", 42.0)
        );

        assert!(
            root_rebind.take_dirt().is_empty(),
            "C++ list rows retain their own property dirt; scalar writes do not call the parent `rebindDependents` path (`viewmodel_instance.cpp:406-415`)"
        );
        assert!(row_dirt.take_dirt().contains(RuntimeCellDirt::BINDINGS));
        let refreshed = list
            .item_entries()
            .into_iter()
            .next()
            .expect("refreshed list row");
        assert_eq!(refreshed.occurrence_identity, occurrence_identity);
        assert_eq!(
            refreshed
                .instance
                .borrow()
                .number_value_by_property_name("value"),
            Some(42.0)
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

        let (mut changed, mut shared_children) = instance.advance_script_frame_local();
        assert!(
            !changed && shared_children.len() == 1,
            "C++ File::createViewModelInstance retains the generated child as a concrete referenceViewModelInstance (`file.cpp:1141-1200`)"
        );
        while let Some(child) = shared_children.pop() {
            let (child_changed, mut grandchildren) =
                child.borrow_mut().advance_script_frame_local();
            changed |= child_changed;
            shared_children.append(&mut grandchildren);
        }

        assert!(changed);
        assert_eq!(instance.trigger_value_by_property_path(&[0, 0]), Some(0));
    }
}
