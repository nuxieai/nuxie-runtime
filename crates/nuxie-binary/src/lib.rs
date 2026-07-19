use anyhow::{Context, Result, bail};
use nuxie_schema::{
    BitmaskPassthrough, CoreRegistryFieldKind, Definition, FieldKind, Property,
    StoredFieldInitializer, UintStorage, core_registry_field_kind_by_property_key,
    definition_by_name, definition_by_type_key, object_supports_property,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    ops::{BitOr, BitOrAssign},
};

pub const SUPPORTED_MAJOR_VERSION: u64 = 7;
pub const SUPPORTED_MINOR_VERSION: u64 = 2;
pub const VIEW_MODEL_SYMBOL_ITEM_INDEX: u8 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[repr(u32)]
pub enum RuntimeDataType {
    None = 0,
    String = 1,
    Number = 2,
    Boolean = 3,
    Color = 4,
    List = 5,
    EnumType = 6,
    Trigger = 7,
    ViewModel = 8,
    Integer = 9,
    SymbolListIndex = 10,
    AssetImage = 11,
    Artboard = 12,
    AssetFont = 13,
    Input = 99,
    Any = 100,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeDataBindUpdateQueue {
    Persisting,
    DirtyToSource,
    DirtyToTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct RuntimeComponentDirt(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindCollapseEffect {
    pub is_collapsed: bool,
    pub changed: bool,
    pub requests_dirty_update: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindAddDirtEffect {
    pub dirt: RuntimeComponentDirt,
    pub target_origin: bool,
    pub changed: bool,
    pub invalidates_context_value: bool,
    pub requests_dirty_update: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindAddEffect {
    pub queues_pending_addition: bool,
    pub appends_to_data_binds: bool,
    pub appends_to_persisting_list: bool,
    pub sets_persisting_list_flag: bool,
    pub sets_container: bool,
    pub binds_from_data_context: bool,
    pub runs_initial_update: bool,
    pub initial_update_applies_target_to_source: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindSourceEffect {
    pub adds_source_dependent: bool,
    pub sets_source: bool,
    pub updates_artboard_component_list_reset: bool,
    pub artboard_component_list_should_reset_instances: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindClearSourceEffect {
    pub removes_source_dependent: bool,
    pub clears_source: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindBindEffect {
    pub clears_existing_context_value: bool,
    pub context_value_type: Option<RuntimeDataType>,
    pub resets_converter: bool,
    pub removes_existing_target_observer: bool,
    pub adds_target_observer: bool,
    pub observing_after: bool,
    pub add_dirt_effect: RuntimeDataBindAddDirtEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindTargetEffect {
    pub unchanged: bool,
    pub removes_existing_target_observer: bool,
    pub sets_target: bool,
    pub adds_target_observer: bool,
    pub observing_after: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindUnbindEffect {
    pub clear_source_effect: RuntimeDataBindClearSourceEffect,
    pub removes_target_observer: bool,
    pub observing_after: bool,
    pub unbinds_converter: bool,
    pub clears_context_value: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindInitializeEffect {
    pub target_is_component: bool,
    pub adds_component_collapsable: bool,
    pub runs_collapse: bool,
    pub collapse_effect: Option<RuntimeDataBindCollapseEffect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindRelinkEffect {
    pub calls_container_rebuild: bool,
    pub rebuild_binds_from_context: bool,
    pub rebuild_has_data_context: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeDataBindContextBindBranch {
    NoDataContext,
    BindSource,
    UnbindMissingSource,
    AddDirtExistingSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindContextBindEffect {
    pub branch: RuntimeDataBindContextBindBranch,
    pub resolves_path: bool,
    pub marks_path_resolved: bool,
    pub updates_source_path_ids: bool,
    pub uses_relative_view_model_property_lookup: bool,
    pub uses_view_model_property_lookup: bool,
    pub clear_source_effect: Option<RuntimeDataBindClearSourceEffect>,
    pub source_effect: Option<RuntimeDataBindSourceEffect>,
    pub bind_effect: Option<RuntimeDataBindBindEffect>,
    pub unbind_effect: Option<RuntimeDataBindUnbindEffect>,
    pub add_dirt_effect: Option<RuntimeDataBindAddDirtEffect>,
    pub binds_converter_from_context: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindUpdateEffect {
    pub calls_update_dependents: bool,
    pub updates_converter_dependents: bool,
    pub applies_target_to_source_before_update: bool,
    pub clears_dirt: bool,
    pub applies_source_to_target: bool,
    pub source_to_target_is_main_direction: bool,
    pub suppresses_dirt_while_applying_source_to_target: bool,
    pub applies_target_to_source_after_update: bool,
    pub target_to_source_is_main_direction: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindRemoveEffect {
    pub queues_pending_removal: bool,
    pub removes_from_data_binds: bool,
    pub scans_persisting_list: bool,
    pub clears_persisting_list_flag: bool,
    pub scans_dirty_lists: bool,
    pub clears_dirty_list_flag: bool,
    pub clears_container: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindContainerBindContextEffect {
    pub bind_from_context_data_bind_ids: Vec<usize>,
    pub stores_data_context: bool,
    pub clears_data_context: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindContainerUnbindEffect {
    pub unbinds_data_bind_ids: Vec<usize>,
    pub clears_data_context: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindContainerAdvanceEffect {
    pub advances_data_bind_ids: Vec<usize>,
    pub did_update: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeDataBindContainerUpdateReturnReason {
    AlreadyProcessing,
    NoActiveQueues,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindContainerUpdateStep {
    pub data_bind_id: usize,
    pub queue: RuntimeDataBindUpdateQueue,
    pub apply_target_to_source: bool,
    pub clears_dirty_list_flag: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindContainerUpdateEffect {
    pub return_reason: Option<RuntimeDataBindContainerUpdateReturnReason>,
    pub enters_processing: bool,
    pub update_steps: Vec<RuntimeDataBindContainerUpdateStep>,
    pub skipped_persisting_data_bind_ids: Vec<usize>,
    pub clears_dirty_to_source_queue: bool,
    pub clears_dirty_to_target_queue: bool,
    pub clears_dirty_list_flag_data_bind_ids: Vec<usize>,
    pub next_dirty_to_source_data_bind_ids: Vec<usize>,
    pub next_dirty_to_target_data_bind_ids: Vec<usize>,
    pub flushes_pending_addition_ids: Vec<usize>,
    pub flushes_pending_removal_ids: Vec<usize>,
    pub exits_processing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataBindContainerAddDirtyEffect {
    pub skips_persisting_to_source: bool,
    pub skips_already_dirty: bool,
    pub queue: Option<RuntimeDataBindUpdateQueue>,
    pub queues_pending: bool,
    pub sets_dirty_list_flag: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataConverterBindContextEffect {
    pub stores_parent_data_bind: bool,
    pub owned_data_bind_context_effect: RuntimeDataBindContainerBindContextEffect,
    pub group_child_bind_from_context_converter_ids: Vec<usize>,
    pub operation_view_model_uses_source_path_lookup: bool,
    pub operation_view_model_sets_number_source: bool,
    pub operation_view_model_adds_data_bind_dependent: bool,
    pub formula_checks_parent_data_bind_source: bool,
    pub formula_sets_source_from_parent: bool,
    pub formula_adds_source_dependent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataConverterUnbindEffect {
    pub owned_data_bind_unbind_effect: Option<RuntimeDataBindContainerUnbindEffect>,
    pub group_child_unbind_converter_ids: Vec<usize>,
    pub formula_removes_source_dependent: bool,
    pub formula_clears_source: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataConverterUpdateEffect {
    pub owned_data_bind_update_effect: Option<RuntimeDataBindContainerUpdateEffect>,
    pub group_child_update_converter_ids: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataConverterResetEffect {
    pub group_child_reset_converter_ids: Vec<usize>,
    pub resets_interpolator_advance_count: bool,
    pub disposes_interpolator_advancer_values: bool,
    pub clears_interpolator_smoothing_animation: bool,
    pub clears_interpolator_initialized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataConverterMarkDirtyEffect {
    pub parent_add_dirt_effect: Option<RuntimeDataBindAddDirtEffect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataConverterPropertyChangeEffect {
    pub clears_number_to_list_items: bool,
    pub mark_converter_dirty_effect: RuntimeDataConverterMarkDirtyEffect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataConverterAddDirtyDataBindEffect {
    pub mark_converter_dirty_effect: RuntimeDataConverterMarkDirtyEffect,
    pub container_add_dirty_effect: RuntimeDataBindContainerAddDirtyEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDataConverterFormulaAddDirtEffect {
    pub clears_randoms: bool,
}

impl RuntimeComponentDirt {
    pub const NONE: Self = Self(0);
    pub const COLLAPSED: Self = Self(1 << 0);
    pub const DEPENDENTS: Self = Self(1 << 1);
    pub const COMPONENTS: Self = Self(1 << 2);
    pub const DRAW_ORDER: Self = Self(1 << 3);
    pub const PATH: Self = Self(1 << 4);
    pub const TEXT_SHAPE: Self = Self(1 << 4);
    pub const SKIN: Self = Self(1 << 4);
    pub const VERTICES: Self = Self(1 << 5);
    pub const TEXT_COVERAGE: Self = Self(1 << 5);
    pub const TRANSFORM: Self = Self(1 << 6);
    pub const WORLD_TRANSFORM: Self = Self(1 << 7);
    pub const RENDER_OPACITY: Self = Self(1 << 8);
    pub const PAINT: Self = Self(1 << 9);
    pub const STOPS: Self = Self(1 << 10);
    pub const LAYOUT_STYLE: Self = Self(1 << 11);
    pub const BINDINGS: Self = Self(1 << 12);
    pub const NSLICER: Self = Self(1 << 13);
    pub const BINDINGS_TARGET: Self = Self(1 << 13);
    pub const SCRIPT_UPDATE: Self = Self(1 << 14);
    pub const CLIPPING: Self = Self(1 << 15);
    pub const FILTHY: Self = Self(0xFFFE);

    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u16 {
        self.0
    }

    pub const fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for RuntimeComponentDirt {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for RuntimeComponentDirt {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl RuntimeDataType {
    pub fn as_cpp_u32(self) -> u32 {
        self as u32
    }
}

#[derive(Debug, Clone, Copy)]
enum RuntimeStateMachineLayerComponentOwner {
    State {
        state_machine_index: usize,
        layer_index: usize,
        state_index: usize,
    },
    Transition {
        state_machine_index: usize,
        layer_index: usize,
        state_index: usize,
        transition_index: usize,
    },
}

#[derive(Debug, Clone, Copy)]
struct RuntimeStateMachineScriptedObjectOwner {
    state_machine_index: usize,
    scripted_object_index: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeFile {
    pub header: RuntimeHeader,
    pub objects: Vec<Option<RuntimeObject>>,
    pub import_statuses: Vec<RuntimeImportStatus>,
}

/// One dense, file-global FileAsset entry and the in-band contents imported
/// for it by a scripting-enabled FileAsset importer.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeFileAssetContents<'a> {
    pub ordinal: usize,
    pub asset: &'a RuntimeObject,
    pub contents: Option<&'a [u8]>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AuthoringRecord {
    pub type_key: u16,
    pub properties: Vec<AuthoringProperty>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AuthoringProperty {
    pub key: u16,
    pub value: AuthoringValue,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum AuthoringValue {
    Bool(bool),
    Bytes(Vec<u8>),
    Color(u32),
    Double(f32),
    String(String),
    Uint(u64),
}

impl RuntimeFile {
    pub fn from_authoring_records(records: Vec<AuthoringRecord>) -> Result<Self> {
        let mut property_field_ids = BTreeMap::new();
        let objects = records
            .into_iter()
            .enumerate()
            .map(|(id, record)| {
                authoring_record_to_runtime_object(id, record, &mut property_field_ids).map(Some)
            })
            .collect::<Result<Vec<_>>>()?;

        // Authoring can deliberately construct the scripting-enabled record
        // vocabulary even when the external C++ conformance reader was built
        // without WITH_RIVE_SCRIPTING. Keep byte-file import on the conformance
        // profile while giving authored ScriptAsset/ShaderAsset contents their
        // real FileAsset importer context.
        let file = finalize_runtime_file_with_script_assets(
            RuntimeHeader {
                major_version: SUPPORTED_MAJOR_VERSION,
                minor_version: SUPPORTED_MINOR_VERSION,
                file_id: 0,
                property_field_ids,
            },
            objects,
            true,
        )?;
        validate_authoring_import_statuses(&file)?;
        validate_authoring_artboard_local_objects(&file)?;
        Ok(file)
    }

    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    pub fn known_object_count(&self) -> usize {
        self.objects
            .iter()
            .filter(|object| object.is_some())
            .count()
    }

    pub fn object(&self, id: usize) -> Option<&RuntimeObject> {
        self.objects.get(id).and_then(|object| object.as_ref())
    }

    pub fn import_status(&self, id: usize) -> Option<RuntimeImportStatus> {
        self.import_statuses.get(id).copied()
    }

    pub fn imported_object_count(&self) -> usize {
        self.import_statuses
            .iter()
            .filter(|status| matches!(status, RuntimeImportStatus::Imported))
            .count()
    }

    pub fn artboards(&self) -> Vec<&RuntimeObject> {
        self.cpp_artboards().collect()
    }

    pub fn artboard(&self, index: usize) -> Option<&RuntimeObject> {
        self.cpp_artboards().nth(index)
    }

    pub fn artboard_local_object(
        &self,
        artboard_index: usize,
        local_index: usize,
    ) -> Option<&RuntimeObject> {
        self.artboard_local_object_slots(artboard_index)?
            .get(local_index)
            .copied()
            .flatten()
    }

    /// Returns the validated C++ artboard-local object table in one pass.
    ///
    /// The vector index is the C++ artboard-local id. `None` entries are
    /// significant: C++ keeps null slots for abstract, unknown, dropped, or
    /// invalid local objects, so callers must not compact the result.
    pub fn artboard_local_object_slots(
        &self,
        artboard_index: usize,
    ) -> Option<Vec<Option<&RuntimeObject>>> {
        let range = self.cpp_artboard_range(artboard_index)?;
        let mut slots = runtime_artboard_local_slots(&self.objects, &self.import_statuses, range);
        validate_cpp_artboard_local_slots(&mut slots, &self.objects);
        Some(
            slots
                .into_iter()
                .map(|file_index| file_index.and_then(|file_index| self.object(file_index)))
                .collect(),
        )
    }

    pub fn default_artboard(&self) -> Option<&RuntimeObject> {
        self.artboard(0)
    }

    pub fn artboard_named(&self, name: &str) -> Option<&RuntimeObject> {
        self.artboard_named_bytes(name.as_bytes())
    }

    pub fn artboard_named_bytes(&self, name: &[u8]) -> Option<&RuntimeObject> {
        self.cpp_artboards()
            .find(|artboard| artboard.string_property_bytes("name").unwrap_or_default() == name)
    }

    pub fn resolved_view_model_for_artboard(
        &self,
        artboard_index: usize,
    ) -> Option<RuntimeViewModelReference<'_>> {
        let artboard = self.artboard(artboard_index)?;
        self.resolved_view_model_for_artboard_object(artboard)
    }

    pub fn resolved_view_model_for_artboard_object(
        &self,
        artboard: &RuntimeObject,
    ) -> Option<RuntimeViewModelReference<'_>> {
        if artboard.type_name != "Artboard" {
            return None;
        }

        let artboard_id = usize::try_from(artboard.id).ok()?;
        if self.import_status(artboard_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let view_model_index = usize::try_from(artboard.uint_property("viewModelId")?).ok()?;
        let view_model = self.view_model(view_model_index)?;
        Some(RuntimeViewModelReference {
            view_model_index,
            object: view_model.object,
        })
    }

    pub fn resolved_artboard_for_referencer(&self, object_id: usize) -> Option<&RuntimeObject> {
        let referencer = self.object(object_id)?;
        self.resolved_artboard_for_referencer_object(referencer)
    }

    pub fn resolved_artboard_for_referencer_object(
        &self,
        referencer: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let object_id = usize::try_from(referencer.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let artboard_index = usize::try_from(cpp_artboard_referencer_index(referencer)?).ok()?;
        self.artboard(artboard_index)
    }

    pub fn resolved_handle_source_for_joystick(
        &self,
        joystick_id: usize,
    ) -> Option<&RuntimeObject> {
        let joystick = self.object(joystick_id)?;
        self.resolved_handle_source_for_joystick_object(joystick)
    }

    pub fn resolved_handle_source_for_joystick_object(
        &self,
        joystick: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        if joystick.type_name != "Joystick" {
            return None;
        }

        let joystick_id = usize::try_from(joystick.id).ok()?;
        if self.import_status(joystick_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let handle_source_id = joystick.uint_property("handleSourceId")?;
        if handle_source_id == u64::from(u32::MAX) {
            return None;
        }

        let (_, _, slots, _) = self.cpp_artboard_local_context_for_object(joystick)?;
        let handle_source = local_object_reference(&slots, &self.objects, Some(handle_source_id))?;
        runtime_object_is_cpp_transform_component(handle_source).then_some(handle_source)
    }

    pub fn resolved_x_animation_for_joystick(&self, joystick_id: usize) -> Option<&RuntimeObject> {
        let joystick = self.object(joystick_id)?;
        self.resolved_x_animation_for_joystick_object(joystick)
    }

    pub fn resolved_x_animation_for_joystick_object(
        &self,
        joystick: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        self.resolved_axis_animation_for_joystick_object(joystick, "xId")
    }

    pub fn resolved_y_animation_for_joystick(&self, joystick_id: usize) -> Option<&RuntimeObject> {
        let joystick = self.object(joystick_id)?;
        self.resolved_y_animation_for_joystick_object(joystick)
    }

    pub fn resolved_y_animation_for_joystick_object(
        &self,
        joystick: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        self.resolved_axis_animation_for_joystick_object(joystick, "yId")
    }

    pub fn artboard_component_list_map_rules(
        &self,
        list_id: usize,
    ) -> Vec<RuntimeArtboardListMapRule<'_>> {
        let Some(list) = self.object(list_id) else {
            return Vec::new();
        };

        self.artboard_component_list_map_rules_for_object(list)
    }

    pub fn artboard_component_list_map_rules_for_object(
        &self,
        list: &RuntimeObject,
    ) -> Vec<RuntimeArtboardListMapRule<'_>> {
        if list.type_name != "ArtboardComponentList" {
            return Vec::new();
        }

        let Some((_, range, slots, list_local_index)) =
            self.cpp_artboard_local_context_for_object(list)
        else {
            return Vec::new();
        };

        self.objects[range.0..range.1]
            .iter()
            .enumerate()
            .filter_map(|(offset, object)| {
                let file_index = range.0 + offset;
                if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                if object.type_name != "ArtboardListMapRule" {
                    return None;
                }
                if !slots.iter().any(|slot| *slot == Some(file_index)) {
                    return None;
                }
                if object.uint_property("parentId") != Some(list_local_index as u64) {
                    return None;
                }

                Some(RuntimeArtboardListMapRule {
                    object,
                    view_model_id: object.uint_property("viewModelId")?,
                    artboard_id: object.uint_property("artboardId")?,
                })
            })
            .collect()
    }

    pub fn resolved_artboard_for_artboard_component_list_item(
        &self,
        list_id: usize,
        list_item_id: usize,
    ) -> Option<RuntimeArtboardListItemArtboard<'_>> {
        let list = self.object(list_id)?;
        let list_item = self.object(list_item_id)?;
        self.resolved_artboard_for_artboard_component_list_item_objects(list, list_item)
    }

    pub fn resolved_artboard_for_artboard_component_list_item_objects(
        &self,
        list: &RuntimeObject,
        list_item: &RuntimeObject,
    ) -> Option<RuntimeArtboardListItemArtboard<'_>> {
        if list.type_name != "ArtboardComponentList" {
            return None;
        }

        let list_id = usize::try_from(list.id).ok()?;
        if self.import_status(list_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let referenced_instance =
            self.referenced_view_model_instance_for_list_item_object(list_item)?;
        let view_model_id = referenced_instance.object.uint_property("viewModelId")?;

        if let Some(rule) = self
            .artboard_component_list_map_rules_for_object(list)
            .into_iter()
            .rev()
            .find(|rule| rule.view_model_id == view_model_id)
            && let Ok(artboard_index) = usize::try_from(rule.artboard_id)
            && let Some(object) = self.artboard(artboard_index)
        {
            return Some(RuntimeArtboardListItemArtboard {
                view_model_index: referenced_instance.view_model_index,
                instance_index: referenced_instance.instance_index,
                artboard_index,
                object,
            });
        }

        self.cpp_artboards()
            .enumerate()
            .find(|(_, artboard)| artboard.uint_property("viewModelId") == Some(view_model_id))
            .map(|(artboard_index, object)| RuntimeArtboardListItemArtboard {
                view_model_index: referenced_instance.view_model_index,
                instance_index: referenced_instance.instance_index,
                artboard_index,
                object,
            })
    }

    pub fn artboard_animations(&self, artboard_index: usize) -> Vec<&RuntimeObject> {
        self.cpp_artboard_objects_named(artboard_index, "LinearAnimation")
    }

    pub fn artboard_animation(
        &self,
        artboard_index: usize,
        animation_index: usize,
    ) -> Option<&RuntimeObject> {
        self.cpp_artboard_objects_named(artboard_index, "LinearAnimation")
            .into_iter()
            .nth(animation_index)
    }

    pub fn artboard_animation_named(
        &self,
        artboard_index: usize,
        name: &str,
    ) -> Option<&RuntimeObject> {
        self.artboard_animation_named_bytes(artboard_index, name.as_bytes())
    }

    pub fn artboard_animation_named_bytes(
        &self,
        artboard_index: usize,
        name: &[u8],
    ) -> Option<&RuntimeObject> {
        self.cpp_artboard_objects_named(artboard_index, "LinearAnimation")
            .into_iter()
            .find(|animation| animation.string_property_bytes("name").unwrap_or_default() == name)
    }

    pub fn artboard_linear_animations(
        &self,
        artboard_index: usize,
    ) -> Vec<RuntimeLinearAnimation<'_>> {
        self.cpp_artboard_linear_animations(artboard_index)
    }

    pub fn artboard_linear_animation(
        &self,
        artboard_index: usize,
        animation_index: usize,
    ) -> Option<RuntimeLinearAnimation<'_>> {
        self.cpp_artboard_linear_animations(artboard_index)
            .into_iter()
            .nth(animation_index)
    }

    pub fn artboard_state_machines(&self, artboard_index: usize) -> Vec<&RuntimeObject> {
        self.cpp_artboard_objects_named(artboard_index, "StateMachine")
    }

    pub fn artboard_state_machine(
        &self,
        artboard_index: usize,
        state_machine_index: usize,
    ) -> Option<&RuntimeObject> {
        self.cpp_artboard_objects_named(artboard_index, "StateMachine")
            .into_iter()
            .nth(state_machine_index)
    }

    pub fn artboard_state_machine_named(
        &self,
        artboard_index: usize,
        name: &str,
    ) -> Option<&RuntimeObject> {
        self.artboard_state_machine_named_bytes(artboard_index, name.as_bytes())
    }

    pub fn artboard_state_machine_named_bytes(
        &self,
        artboard_index: usize,
        name: &[u8],
    ) -> Option<&RuntimeObject> {
        self.cpp_artboard_objects_named(artboard_index, "StateMachine")
            .into_iter()
            .find(|state_machine| {
                state_machine
                    .string_property_bytes("name")
                    .unwrap_or_default()
                    == name
            })
    }

    pub fn artboard_state_machine_graphs(
        &self,
        artboard_index: usize,
    ) -> Vec<RuntimeStateMachine<'_>> {
        self.cpp_artboard_state_machine_graphs(artboard_index)
    }

    pub fn artboard_state_machine_graph(
        &self,
        artboard_index: usize,
        state_machine_index: usize,
    ) -> Option<RuntimeStateMachine<'_>> {
        self.cpp_artboard_state_machine_graphs(artboard_index)
            .into_iter()
            .nth(state_machine_index)
    }

    pub fn artboard_data_binds(&self, artboard_index: usize) -> Vec<RuntimeDataBind<'_>> {
        self.cpp_artboard_data_binds(artboard_index)
    }

    pub fn artboard_data_bind(
        &self,
        artboard_index: usize,
        data_bind_index: usize,
    ) -> Option<RuntimeDataBind<'_>> {
        self.cpp_artboard_data_binds(artboard_index)
            .into_iter()
            .nth(data_bind_index)
    }

    pub fn data_bind_target_for_object<'a>(
        &'a self,
        data_bind: &RuntimeObject,
    ) -> Option<&'a RuntimeObject> {
        self.validate_data_bind(data_bind)?;
        self.cpp_data_bind_target_for_object(data_bind)
    }

    pub fn latest_bindable_property_for_object<'a>(
        &'a self,
        object: &RuntimeObject,
    ) -> Option<&'a RuntimeObject> {
        self.cpp_latest_bindable_property_for_object(object)
    }

    pub fn transition_view_model_condition_comparators<'a>(
        &'a self,
        condition: &RuntimeObject,
    ) -> Option<RuntimeTransitionViewModelConditionComparators<'a>> {
        self.validate_transition_view_model_condition(condition)?;
        Some(self.cpp_transition_view_model_condition_comparators(condition))
    }

    pub fn artboard_skins(&self, artboard_index: usize) -> Vec<RuntimeSkin<'_>> {
        self.cpp_artboard_skins(artboard_index)
    }

    pub fn artboard_skin(
        &self,
        artboard_index: usize,
        skin_index: usize,
    ) -> Option<RuntimeSkin<'_>> {
        self.cpp_artboard_skins(artboard_index)
            .into_iter()
            .nth(skin_index)
    }

    pub fn artboard_meshes(&self, artboard_index: usize) -> Vec<RuntimeMesh<'_>> {
        self.cpp_artboard_meshes(artboard_index)
    }

    pub fn artboard_geometry(&self, artboard_index: usize) -> Option<RuntimeArtboardGeometry<'_>> {
        let index = self.cpp_artboard_index(artboard_index)?;
        Some(RuntimeArtboardGeometry {
            meshes: index.meshes(),
            paths: index.paths(),
            shapes: index.shapes(),
            shape_paint_containers: index.shape_paint_containers(),
            n_slicer_details: index.n_slicer_details(),
        })
    }

    pub fn artboard_mesh(
        &self,
        artboard_index: usize,
        mesh_index: usize,
    ) -> Option<RuntimeMesh<'_>> {
        self.cpp_artboard_meshes(artboard_index)
            .into_iter()
            .nth(mesh_index)
    }

    pub fn artboard_paths(&self, artboard_index: usize) -> Vec<RuntimePath<'_>> {
        self.cpp_artboard_paths(artboard_index)
    }

    pub fn artboard_path(
        &self,
        artboard_index: usize,
        path_index: usize,
    ) -> Option<RuntimePath<'_>> {
        self.cpp_artboard_paths(artboard_index)
            .into_iter()
            .nth(path_index)
    }

    pub fn artboard_shapes(&self, artboard_index: usize) -> Vec<RuntimeShape<'_>> {
        self.cpp_artboard_shapes(artboard_index)
    }

    pub fn artboard_shape(
        &self,
        artboard_index: usize,
        shape_index: usize,
    ) -> Option<RuntimeShape<'_>> {
        self.cpp_artboard_shapes(artboard_index)
            .into_iter()
            .nth(shape_index)
    }

    pub fn artboard_shape_paint_containers(
        &self,
        artboard_index: usize,
    ) -> Vec<RuntimeShapePaintContainer<'_>> {
        self.cpp_artboard_shape_paint_containers(artboard_index)
    }

    pub fn artboard_shape_paint_container(
        &self,
        artboard_index: usize,
        container_index: usize,
    ) -> Option<RuntimeShapePaintContainer<'_>> {
        self.cpp_artboard_shape_paint_containers(artboard_index)
            .into_iter()
            .nth(container_index)
    }

    pub fn artboard_n_slicer_details(
        &self,
        artboard_index: usize,
    ) -> Vec<RuntimeNSlicerDetails<'_>> {
        self.cpp_artboard_n_slicer_details(artboard_index)
    }

    pub fn artboard_n_slicer_detail(
        &self,
        artboard_index: usize,
        details_index: usize,
    ) -> Option<RuntimeNSlicerDetails<'_>> {
        self.cpp_artboard_n_slicer_details(artboard_index)
            .into_iter()
            .nth(details_index)
    }

    pub fn data_bind_path_for_referencer(
        &self,
        object_id: usize,
    ) -> Option<RuntimeDataBindPath<'_>> {
        let referencer = self.object(object_id)?;
        self.data_bind_path_for_referencer_object(referencer)
    }

    pub fn data_bind_path_for_referencer_object(
        &self,
        referencer: &RuntimeObject,
    ) -> Option<RuntimeDataBindPath<'_>> {
        let referencer_id = usize::try_from(referencer.id).ok()?;
        if self.import_status(referencer_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        if !cpp_is_data_bind_path_referencer(referencer) {
            return None;
        }

        if let Some(path_object) = self.cpp_claimed_data_bind_path_for(referencer_id) {
            let path_ids = path_object.data_bind_path_ids().unwrap_or_default();
            let resolved_path_ids = self.cpp_resolved_data_bind_path_ids(path_object, &path_ids);
            return Some(RuntimeDataBindPath {
                object: Some(path_object),
                property_name: "path",
                path_ids,
                resolved_path_ids,
            });
        }

        let property_name = cpp_inline_data_bind_path_property(referencer)?;
        let path_ids = referencer.data_bind_path_ids_property(property_name)?;
        Some(RuntimeDataBindPath {
            object: None,
            property_name,
            resolved_path_ids: path_ids.clone(),
            path_ids,
        })
    }

    pub fn resolved_data_bind_path_ids_for_referencer(&self, object_id: usize) -> Option<Vec<u32>> {
        let referencer = self.object(object_id)?;
        self.resolved_data_bind_path_ids_for_referencer_object(referencer)
    }

    pub fn resolved_data_bind_path_ids_for_referencer_object(
        &self,
        referencer: &RuntimeObject,
    ) -> Option<Vec<u32>> {
        self.data_bind_path_for_referencer_object(referencer)
            .map(|path| path.resolved_path_ids)
    }

    pub fn listener_input_type_view_model_path_ids_buffer(
        &self,
        input_type_id: usize,
    ) -> Option<Vec<u32>> {
        let input_type = self.object(input_type_id)?;
        self.listener_input_type_view_model_path_ids_buffer_for_object(input_type)
    }

    pub fn listener_input_type_view_model_path_ids_buffer_for_object(
        &self,
        input_type: &RuntimeObject,
    ) -> Option<Vec<u32>> {
        if input_type.type_name != "ListenerInputTypeViewModel" {
            return None;
        }

        let input_type_id = usize::try_from(input_type.id).ok()?;
        if self.import_status(input_type_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        input_type.id_list_property("viewModelPathIds")
    }

    pub fn data_bind_context_source_path_ids(&self, data_bind_id: usize) -> Option<Vec<u32>> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_context_source_path_ids_for_object(data_bind)
    }

    pub fn data_bind_context_source_path_ids_for_object(
        &self,
        data_bind: &RuntimeObject,
    ) -> Option<Vec<u32>> {
        if data_bind.type_name != "DataBindContext" {
            return None;
        }

        let data_bind_id = usize::try_from(data_bind.id).ok()?;
        if self.import_status(data_bind_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        Some(
            data_bind
                .id_list_property("sourcePathIds")
                .unwrap_or_default(),
        )
    }

    pub fn data_bind_context_resolved_source_path_ids(
        &self,
        data_bind_id: usize,
    ) -> Option<Vec<u32>> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_context_resolved_source_path_ids_for_object(data_bind)
    }

    pub fn data_bind_context_resolved_source_path_ids_for_object(
        &self,
        data_bind: &RuntimeObject,
    ) -> Option<Vec<u32>> {
        let source_path_ids = self.data_bind_context_source_path_ids_for_object(data_bind)?;
        if !cpp_data_bind_is_name_based(data_bind) {
            return Some(source_path_ids);
        }

        let Some(path_id) = source_path_ids.first().copied() else {
            return Some(source_path_ids);
        };

        if let Some(resolved_path) = self.manifest().and_then(|manifest| {
            manifest
                .resolve_path(path_id)
                .filter(|resolved_path| !resolved_path.is_empty())
                .map(<[u32]>::to_vec)
        }) {
            return Some(resolved_path);
        }

        Some(source_path_ids)
    }

    pub fn data_enums(&self) -> Vec<RuntimeDataEnum<'_>> {
        self.cpp_data_enums()
    }

    pub fn data_enum(&self, index: usize) -> Option<RuntimeDataEnum<'_>> {
        self.cpp_data_enums().into_iter().nth(index)
    }

    pub fn data_enum_value_for_key_bytes(
        &self,
        data_enum_index: usize,
        key: &[u8],
    ) -> Option<&[u8]> {
        let data_enum = self.data_enum(data_enum_index)?;
        let value = data_enum
            .values
            .into_iter()
            .find(|value| value.string_property_bytes("key").unwrap_or_default() == key)?;
        Some(cpp_data_enum_resolved_value_bytes(value))
    }

    pub fn data_enum_value_for_key(&self, data_enum_index: usize, key: &str) -> Option<&str> {
        std::str::from_utf8(self.data_enum_value_for_key_bytes(data_enum_index, key.as_bytes())?)
            .ok()
    }

    pub fn data_enum_value_for_index(
        &self,
        data_enum_index: usize,
        value_index: usize,
    ) -> Option<&[u8]> {
        let data_enum = self.data_enum(data_enum_index)?;
        let value = data_enum.values.get(value_index)?;
        Some(cpp_data_enum_resolved_value_bytes(value))
    }

    pub fn data_enum_value_index_for_index(
        &self,
        data_enum_index: usize,
        value_index: usize,
    ) -> Option<usize> {
        let data_enum = self.data_enum(data_enum_index)?;
        (value_index < data_enum.values.len()).then_some(value_index)
    }

    pub fn data_enum_value_index_for_key_bytes(
        &self,
        data_enum_index: usize,
        key: &[u8],
    ) -> Option<usize> {
        let data_enum = self.data_enum(data_enum_index)?;
        data_enum
            .values
            .into_iter()
            .position(|value| value.string_property_bytes("key").unwrap_or_default() == key)
    }

    pub fn data_enum_value_index_for_key(
        &self,
        data_enum_index: usize,
        key: &str,
    ) -> Option<usize> {
        self.data_enum_value_index_for_key_bytes(data_enum_index, key.as_bytes())
    }

    pub fn data_converters(&self) -> Vec<&RuntimeObject> {
        self.cpp_data_converters().collect()
    }

    pub fn data_converter(&self, index: usize) -> Option<&RuntimeObject> {
        self.cpp_data_converters().nth(index)
    }

    pub fn data_converter_output_type(
        &self,
        data_converter_index: usize,
    ) -> Option<RuntimeDataType> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_output_type_for_object(data_converter)
    }

    pub fn data_converter_output_type_for_object(
        &self,
        data_converter: &RuntimeObject,
    ) -> Option<RuntimeDataType> {
        self.validate_data_converter(data_converter)?;

        self.cpp_data_converter_output_type(data_converter, &mut BTreeSet::new())
    }

    pub fn data_converter_bind_context_effect(
        &self,
        data_converter_index: usize,
        owned_data_bind_ids: &[usize],
        has_data_context: bool,
        has_parent_data_bind: bool,
        parent_has_source: bool,
        operation_view_model_lookup_type: Option<RuntimeDataType>,
    ) -> Option<RuntimeDataConverterBindContextEffect> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_bind_context_effect_for_object(
            data_converter,
            owned_data_bind_ids,
            has_data_context,
            has_parent_data_bind,
            parent_has_source,
            operation_view_model_lookup_type,
        )
    }

    pub fn data_converter_bind_context_effect_for_object(
        &self,
        data_converter: &RuntimeObject,
        owned_data_bind_ids: &[usize],
        has_data_context: bool,
        has_parent_data_bind: bool,
        parent_has_source: bool,
        operation_view_model_lookup_type: Option<RuntimeDataType>,
    ) -> Option<RuntimeDataConverterBindContextEffect> {
        self.validate_data_converter(data_converter)?;

        let mut owned_data_bind_context_ids = Vec::new();
        for data_bind_id in owned_data_bind_ids {
            let data_bind = self.object(*data_bind_id)?;
            self.validate_data_bind(data_bind)?;
            let definition = definition_by_type_key(data_bind.type_key)?;
            if definition.is_a("DataBindContext") {
                owned_data_bind_context_ids.push(*data_bind_id);
            }
        }

        Some(cpp_data_converter_bind_context_effect(
            data_converter,
            &owned_data_bind_context_ids,
            has_data_context,
            has_parent_data_bind,
            parent_has_source,
            operation_view_model_lookup_type,
            self.cpp_data_converter_group_child_converter_ids(data_converter),
        ))
    }

    pub fn data_converter_unbind_effect(
        &self,
        data_converter_index: usize,
        owned_data_bind_ids: &[usize],
        has_formula_source: bool,
    ) -> Option<RuntimeDataConverterUnbindEffect> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_unbind_effect_for_object(
            data_converter,
            owned_data_bind_ids,
            has_formula_source,
        )
    }

    pub fn data_converter_unbind_effect_for_object(
        &self,
        data_converter: &RuntimeObject,
        owned_data_bind_ids: &[usize],
        has_formula_source: bool,
    ) -> Option<RuntimeDataConverterUnbindEffect> {
        self.validate_data_converter(data_converter)?;
        let owned_data_bind_unbind_effect =
            if cpp_data_converter_unbinds_owned_data_binds(data_converter) {
                Some(self.data_bind_container_unbind_effect(owned_data_bind_ids)?)
            } else {
                None
            };

        Some(cpp_data_converter_unbind_effect(
            data_converter,
            owned_data_bind_unbind_effect,
            self.cpp_data_converter_group_child_converter_ids(data_converter),
            has_formula_source,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_update_effect(
        &self,
        data_converter_index: usize,
        persisting_data_bind_ids: &[usize],
        persisting_can_skip: &[bool],
        dirty_to_source_data_bind_ids: &[usize],
        dirty_to_target_data_bind_ids: &[usize],
        pending_dirty_to_source_data_bind_ids: &[usize],
        pending_dirty_to_target_data_bind_ids: &[usize],
        pending_addition_ids: &[usize],
        pending_removal_ids: &[usize],
        is_processing: bool,
    ) -> Option<RuntimeDataConverterUpdateEffect> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_update_effect_for_object(
            data_converter,
            persisting_data_bind_ids,
            persisting_can_skip,
            dirty_to_source_data_bind_ids,
            dirty_to_target_data_bind_ids,
            pending_dirty_to_source_data_bind_ids,
            pending_dirty_to_target_data_bind_ids,
            pending_addition_ids,
            pending_removal_ids,
            is_processing,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_update_effect_for_object(
        &self,
        data_converter: &RuntimeObject,
        persisting_data_bind_ids: &[usize],
        persisting_can_skip: &[bool],
        dirty_to_source_data_bind_ids: &[usize],
        dirty_to_target_data_bind_ids: &[usize],
        pending_dirty_to_source_data_bind_ids: &[usize],
        pending_dirty_to_target_data_bind_ids: &[usize],
        pending_addition_ids: &[usize],
        pending_removal_ids: &[usize],
        is_processing: bool,
    ) -> Option<RuntimeDataConverterUpdateEffect> {
        self.validate_data_converter(data_converter)?;
        let owned_data_bind_update_effect =
            if cpp_data_converter_updates_owned_data_binds(data_converter) {
                Some(self.data_bind_container_update_effect(
                    persisting_data_bind_ids,
                    persisting_can_skip,
                    dirty_to_source_data_bind_ids,
                    dirty_to_target_data_bind_ids,
                    pending_dirty_to_source_data_bind_ids,
                    pending_dirty_to_target_data_bind_ids,
                    pending_addition_ids,
                    pending_removal_ids,
                    is_processing,
                    false,
                )?)
            } else {
                None
            };

        Some(cpp_data_converter_update_effect(
            data_converter,
            owned_data_bind_update_effect,
            self.cpp_data_converter_group_child_converter_ids(data_converter),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_mark_dirty_effect(
        &self,
        data_converter_index: usize,
        parent_data_bind_id: Option<usize>,
        parent_current_dirt: RuntimeComponentDirt,
        parent_suppress_dirt: bool,
        parent_is_collapsed: bool,
        parent_has_context_value: bool,
        parent_has_container: bool,
    ) -> Option<RuntimeDataConverterMarkDirtyEffect> {
        self.data_converter_mark_dirty_effect_with_origin(
            data_converter_index,
            parent_data_bind_id,
            parent_current_dirt,
            false,
            parent_suppress_dirt,
            parent_is_collapsed,
            parent_has_context_value,
            parent_has_container,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_mark_dirty_effect_with_origin(
        &self,
        data_converter_index: usize,
        parent_data_bind_id: Option<usize>,
        parent_current_dirt: RuntimeComponentDirt,
        parent_target_origin: bool,
        parent_suppress_dirt: bool,
        parent_is_collapsed: bool,
        parent_has_context_value: bool,
        parent_has_container: bool,
    ) -> Option<RuntimeDataConverterMarkDirtyEffect> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_mark_dirty_effect_for_object_with_origin(
            data_converter,
            parent_data_bind_id,
            parent_current_dirt,
            parent_target_origin,
            parent_suppress_dirt,
            parent_is_collapsed,
            parent_has_context_value,
            parent_has_container,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_mark_dirty_effect_for_object(
        &self,
        data_converter: &RuntimeObject,
        parent_data_bind_id: Option<usize>,
        parent_current_dirt: RuntimeComponentDirt,
        parent_suppress_dirt: bool,
        parent_is_collapsed: bool,
        parent_has_context_value: bool,
        parent_has_container: bool,
    ) -> Option<RuntimeDataConverterMarkDirtyEffect> {
        self.data_converter_mark_dirty_effect_for_object_with_origin(
            data_converter,
            parent_data_bind_id,
            parent_current_dirt,
            false,
            parent_suppress_dirt,
            parent_is_collapsed,
            parent_has_context_value,
            parent_has_container,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_mark_dirty_effect_for_object_with_origin(
        &self,
        data_converter: &RuntimeObject,
        parent_data_bind_id: Option<usize>,
        parent_current_dirt: RuntimeComponentDirt,
        parent_target_origin: bool,
        parent_suppress_dirt: bool,
        parent_is_collapsed: bool,
        parent_has_context_value: bool,
        parent_has_container: bool,
    ) -> Option<RuntimeDataConverterMarkDirtyEffect> {
        self.validate_data_converter(data_converter)?;
        let parent_add_dirt_effect = match parent_data_bind_id {
            Some(parent_data_bind_id) => Some(self.data_bind_add_dirt_effect_with_origin(
                parent_data_bind_id,
                parent_current_dirt,
                parent_target_origin,
                RuntimeComponentDirt::DEPENDENTS
                    | if parent_target_origin {
                        RuntimeComponentDirt::BINDINGS_TARGET
                    } else {
                        RuntimeComponentDirt::BINDINGS
                    },
                parent_suppress_dirt,
                parent_is_collapsed,
                parent_has_context_value,
                parent_has_container,
            )?),
            None => None,
        };

        Some(cpp_data_converter_mark_dirty_effect(parent_add_dirt_effect))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_property_change_effect(
        &self,
        data_converter_index: usize,
        property_name: &str,
        parent_data_bind_id: Option<usize>,
        parent_current_dirt: RuntimeComponentDirt,
        parent_suppress_dirt: bool,
        parent_is_collapsed: bool,
        parent_has_context_value: bool,
        parent_has_container: bool,
    ) -> Option<RuntimeDataConverterPropertyChangeEffect> {
        self.data_converter_property_change_effect_with_origin(
            data_converter_index,
            property_name,
            parent_data_bind_id,
            parent_current_dirt,
            false,
            parent_suppress_dirt,
            parent_is_collapsed,
            parent_has_context_value,
            parent_has_container,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_property_change_effect_with_origin(
        &self,
        data_converter_index: usize,
        property_name: &str,
        parent_data_bind_id: Option<usize>,
        parent_current_dirt: RuntimeComponentDirt,
        parent_target_origin: bool,
        parent_suppress_dirt: bool,
        parent_is_collapsed: bool,
        parent_has_context_value: bool,
        parent_has_container: bool,
    ) -> Option<RuntimeDataConverterPropertyChangeEffect> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_property_change_effect_for_object_with_origin(
            data_converter,
            property_name,
            parent_data_bind_id,
            parent_current_dirt,
            parent_target_origin,
            parent_suppress_dirt,
            parent_is_collapsed,
            parent_has_context_value,
            parent_has_container,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_property_change_effect_for_object(
        &self,
        data_converter: &RuntimeObject,
        property_name: &str,
        parent_data_bind_id: Option<usize>,
        parent_current_dirt: RuntimeComponentDirt,
        parent_suppress_dirt: bool,
        parent_is_collapsed: bool,
        parent_has_context_value: bool,
        parent_has_container: bool,
    ) -> Option<RuntimeDataConverterPropertyChangeEffect> {
        self.data_converter_property_change_effect_for_object_with_origin(
            data_converter,
            property_name,
            parent_data_bind_id,
            parent_current_dirt,
            false,
            parent_suppress_dirt,
            parent_is_collapsed,
            parent_has_context_value,
            parent_has_container,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_property_change_effect_for_object_with_origin(
        &self,
        data_converter: &RuntimeObject,
        property_name: &str,
        parent_data_bind_id: Option<usize>,
        parent_current_dirt: RuntimeComponentDirt,
        parent_target_origin: bool,
        parent_suppress_dirt: bool,
        parent_is_collapsed: bool,
        parent_has_context_value: bool,
        parent_has_container: bool,
    ) -> Option<RuntimeDataConverterPropertyChangeEffect> {
        self.validate_data_converter(data_converter)?;
        if !cpp_data_converter_property_change_marks_dirty(data_converter, property_name) {
            return None;
        }

        Some(cpp_data_converter_property_change_effect(
            data_converter,
            property_name,
            self.data_converter_mark_dirty_effect_for_object_with_origin(
                data_converter,
                parent_data_bind_id,
                parent_current_dirt,
                parent_target_origin,
                parent_suppress_dirt,
                parent_is_collapsed,
                parent_has_context_value,
                parent_has_container,
            )?,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_add_dirty_data_bind_effect(
        &self,
        data_converter_index: usize,
        data_bind_id: usize,
        parent_data_bind_id: Option<usize>,
        parent_current_dirt: RuntimeComponentDirt,
        parent_suppress_dirt: bool,
        parent_is_collapsed: bool,
        parent_has_context_value: bool,
        parent_has_container: bool,
        data_bind_in_persisting_list: bool,
        data_bind_in_dirty_list: bool,
        is_processing: bool,
    ) -> Option<RuntimeDataConverterAddDirtyDataBindEffect> {
        self.data_converter_add_dirty_data_bind_effect_with_origin(
            data_converter_index,
            data_bind_id,
            parent_data_bind_id,
            parent_current_dirt,
            false,
            parent_suppress_dirt,
            parent_is_collapsed,
            parent_has_context_value,
            parent_has_container,
            data_bind_in_persisting_list,
            data_bind_in_dirty_list,
            is_processing,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_add_dirty_data_bind_effect_with_origin(
        &self,
        data_converter_index: usize,
        data_bind_id: usize,
        parent_data_bind_id: Option<usize>,
        parent_current_dirt: RuntimeComponentDirt,
        parent_target_origin: bool,
        parent_suppress_dirt: bool,
        parent_is_collapsed: bool,
        parent_has_context_value: bool,
        parent_has_container: bool,
        data_bind_in_persisting_list: bool,
        data_bind_in_dirty_list: bool,
        is_processing: bool,
    ) -> Option<RuntimeDataConverterAddDirtyDataBindEffect> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_add_dirty_data_bind_effect_for_object_with_origin(
            data_converter,
            data_bind_id,
            parent_data_bind_id,
            parent_current_dirt,
            parent_target_origin,
            parent_suppress_dirt,
            parent_is_collapsed,
            parent_has_context_value,
            parent_has_container,
            data_bind_in_persisting_list,
            data_bind_in_dirty_list,
            is_processing,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_add_dirty_data_bind_effect_for_object(
        &self,
        data_converter: &RuntimeObject,
        data_bind_id: usize,
        parent_data_bind_id: Option<usize>,
        parent_current_dirt: RuntimeComponentDirt,
        parent_suppress_dirt: bool,
        parent_is_collapsed: bool,
        parent_has_context_value: bool,
        parent_has_container: bool,
        data_bind_in_persisting_list: bool,
        data_bind_in_dirty_list: bool,
        is_processing: bool,
    ) -> Option<RuntimeDataConverterAddDirtyDataBindEffect> {
        self.data_converter_add_dirty_data_bind_effect_for_object_with_origin(
            data_converter,
            data_bind_id,
            parent_data_bind_id,
            parent_current_dirt,
            false,
            parent_suppress_dirt,
            parent_is_collapsed,
            parent_has_context_value,
            parent_has_container,
            data_bind_in_persisting_list,
            data_bind_in_dirty_list,
            is_processing,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_converter_add_dirty_data_bind_effect_for_object_with_origin(
        &self,
        data_converter: &RuntimeObject,
        data_bind_id: usize,
        parent_data_bind_id: Option<usize>,
        parent_current_dirt: RuntimeComponentDirt,
        parent_target_origin: bool,
        parent_suppress_dirt: bool,
        parent_is_collapsed: bool,
        parent_has_context_value: bool,
        parent_has_container: bool,
        data_bind_in_persisting_list: bool,
        data_bind_in_dirty_list: bool,
        is_processing: bool,
    ) -> Option<RuntimeDataConverterAddDirtyDataBindEffect> {
        let mark_converter_dirty_effect = self
            .data_converter_mark_dirty_effect_for_object_with_origin(
                data_converter,
                parent_data_bind_id,
                parent_current_dirt,
                parent_target_origin,
                parent_suppress_dirt,
                parent_is_collapsed,
                parent_has_context_value,
                parent_has_container,
            )?;
        let container_add_dirty_effect = self.data_bind_container_add_dirty_effect(
            data_bind_id,
            data_bind_in_persisting_list,
            data_bind_in_dirty_list,
            is_processing,
        )?;

        Some(cpp_data_converter_add_dirty_data_bind_effect(
            mark_converter_dirty_effect,
            container_add_dirty_effect,
        ))
    }

    pub fn data_converter_formula_add_dirt_effect(
        &self,
        data_converter_index: usize,
    ) -> Option<RuntimeDataConverterFormulaAddDirtEffect> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_formula_add_dirt_effect_for_object(data_converter)
    }

    pub fn data_converter_formula_add_dirt_effect_for_object(
        &self,
        data_converter: &RuntimeObject,
    ) -> Option<RuntimeDataConverterFormulaAddDirtEffect> {
        self.validate_data_converter(data_converter)?;
        (data_converter.type_name == "DataConverterFormula").then_some(
            cpp_data_converter_formula_add_dirt_effect(
                data_converter.uint_property("randomModeValue").unwrap_or(0),
            ),
        )
    }

    pub fn data_converter_reset_effect(
        &self,
        data_converter_index: usize,
    ) -> Option<RuntimeDataConverterResetEffect> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_reset_effect_for_object(data_converter)
    }

    pub fn data_converter_reset_effect_for_object(
        &self,
        data_converter: &RuntimeObject,
    ) -> Option<RuntimeDataConverterResetEffect> {
        self.validate_data_converter(data_converter)?;
        Some(cpp_data_converter_reset_effect(
            data_converter,
            self.cpp_data_converter_group_child_converter_ids(data_converter),
        ))
    }

    pub fn data_converter_convert<'a>(
        &'a self,
        data_converter_index: usize,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_convert_for_object(data_converter, input)
    }

    pub fn data_converter_convert_for_object<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let object_id = usize::try_from(data_converter.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_converter.type_key)?;
        if !definition.is_a("DataConverter") {
            return None;
        }

        self.cpp_data_converter_convert(
            data_converter,
            &RuntimeConvertedDataValue::from(input),
            None,
            &[],
            None,
            &mut BTreeSet::new(),
        )
    }

    pub fn data_converter_convert_with_formula_randoms<'a>(
        &'a self,
        data_converter_index: usize,
        input: &RuntimeDataValue<'a>,
        randoms: &[f32],
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let data_converter = self.data_converter(data_converter_index)?;
        let mut random_source = RuntimeFormulaRandomSource::new(randoms);
        self.data_converter_convert_with_formula_randoms_for_object(
            data_converter,
            input,
            &mut random_source,
        )
    }

    pub fn data_converter_convert_with_formula_randoms_for_object<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
        input: &RuntimeDataValue<'a>,
        random_source: &mut RuntimeFormulaRandomSource<'_>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let object_id = usize::try_from(data_converter.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_converter.type_key)?;
        if !definition.is_a("DataConverter") {
            return None;
        }

        self.cpp_data_converter_convert(
            data_converter,
            &RuntimeConvertedDataValue::from(input),
            None,
            &[],
            Some(random_source),
            &mut BTreeSet::new(),
        )
    }

    pub fn data_converter_convert_with_context<'a>(
        &'a self,
        data_converter_index: usize,
        input: &RuntimeDataValue<'a>,
        view_model_instance_ids: &[usize],
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let data_converter = self.data_converter(data_converter_index)?;
        let view_model_instances = view_model_instance_ids
            .iter()
            .map(|id| self.object(*id))
            .collect::<Option<Vec<_>>>()?;
        self.data_converter_convert_with_context_for_object(
            data_converter,
            input,
            &view_model_instances,
        )
    }

    pub fn data_converter_convert_with_context_for_object<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
        input: &RuntimeDataValue<'a>,
        view_model_instances: &[&'a RuntimeObject],
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let object_id = usize::try_from(data_converter.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_converter.type_key)?;
        if !definition.is_a("DataConverter") {
            return None;
        }

        self.cpp_data_converter_convert(
            data_converter,
            &RuntimeConvertedDataValue::from(input),
            None,
            view_model_instances,
            None,
            &mut BTreeSet::new(),
        )
    }

    pub fn data_bind_convert<'a>(
        &'a self,
        data_bind_id: usize,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_convert_for_object(data_bind, input)
    }

    pub fn data_bind_convert_for_object<'a>(
        &'a self,
        data_bind: &'a RuntimeObject,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let object_id = usize::try_from(data_bind.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_bind.type_key)?;
        if !definition.is_a("DataBind") {
            return None;
        }

        let converter = self.resolved_data_converter_for_data_bind_object(data_bind)?;
        self.cpp_data_converter_convert(
            converter,
            &RuntimeConvertedDataValue::from(input),
            Some(data_bind.uint_property("flags").unwrap_or(0)),
            &[],
            None,
            &mut BTreeSet::new(),
        )
    }

    pub fn data_bind_convert_with_context<'a>(
        &'a self,
        data_bind_id: usize,
        input: &RuntimeDataValue<'a>,
        view_model_instance_ids: &[usize],
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let data_bind = self.object(data_bind_id)?;
        let view_model_instances = view_model_instance_ids
            .iter()
            .map(|id| self.object(*id))
            .collect::<Option<Vec<_>>>()?;
        self.data_bind_convert_with_context_for_object(data_bind, input, &view_model_instances)
    }

    pub fn data_bind_convert_with_context_for_object<'a>(
        &'a self,
        data_bind: &'a RuntimeObject,
        input: &RuntimeDataValue<'a>,
        view_model_instances: &[&'a RuntimeObject],
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let object_id = usize::try_from(data_bind.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_bind.type_key)?;
        if !definition.is_a("DataBind") {
            return None;
        }

        let converter = self.resolved_data_converter_for_data_bind_object(data_bind)?;
        self.cpp_data_converter_convert(
            converter,
            &RuntimeConvertedDataValue::from(input),
            Some(data_bind.uint_property("flags").unwrap_or(0)),
            view_model_instances,
            None,
            &mut BTreeSet::new(),
        )
    }

    pub fn data_converter_reverse_convert<'a>(
        &'a self,
        data_converter_index: usize,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_reverse_convert_for_object(data_converter, input)
    }

    pub fn data_converter_reverse_convert_for_object<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let object_id = usize::try_from(data_converter.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_converter.type_key)?;
        if !definition.is_a("DataConverter") {
            return None;
        }

        self.cpp_data_converter_reverse_convert(
            data_converter,
            &RuntimeConvertedDataValue::from(input),
            None,
            &[],
            None,
            &mut BTreeSet::new(),
        )
    }

    pub fn data_converter_reverse_convert_with_formula_randoms<'a>(
        &'a self,
        data_converter_index: usize,
        input: &RuntimeDataValue<'a>,
        randoms: &[f32],
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let data_converter = self.data_converter(data_converter_index)?;
        let mut random_source = RuntimeFormulaRandomSource::new(randoms);
        self.data_converter_reverse_convert_with_formula_randoms_for_object(
            data_converter,
            input,
            &mut random_source,
        )
    }

    pub fn data_converter_reverse_convert_with_formula_randoms_for_object<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
        input: &RuntimeDataValue<'a>,
        random_source: &mut RuntimeFormulaRandomSource<'_>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let object_id = usize::try_from(data_converter.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_converter.type_key)?;
        if !definition.is_a("DataConverter") {
            return None;
        }

        self.cpp_data_converter_reverse_convert(
            data_converter,
            &RuntimeConvertedDataValue::from(input),
            None,
            &[],
            Some(random_source),
            &mut BTreeSet::new(),
        )
    }

    pub fn data_converter_reverse_convert_with_context<'a>(
        &'a self,
        data_converter_index: usize,
        input: &RuntimeDataValue<'a>,
        view_model_instance_ids: &[usize],
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let data_converter = self.data_converter(data_converter_index)?;
        let view_model_instances = view_model_instance_ids
            .iter()
            .map(|id| self.object(*id))
            .collect::<Option<Vec<_>>>()?;
        self.data_converter_reverse_convert_with_context_for_object(
            data_converter,
            input,
            &view_model_instances,
        )
    }

    pub fn data_converter_reverse_convert_with_context_for_object<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
        input: &RuntimeDataValue<'a>,
        view_model_instances: &[&'a RuntimeObject],
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let object_id = usize::try_from(data_converter.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_converter.type_key)?;
        if !definition.is_a("DataConverter") {
            return None;
        }

        self.cpp_data_converter_reverse_convert(
            data_converter,
            &RuntimeConvertedDataValue::from(input),
            None,
            view_model_instances,
            None,
            &mut BTreeSet::new(),
        )
    }

    pub fn data_bind_reverse_convert<'a>(
        &'a self,
        data_bind_id: usize,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_reverse_convert_for_object(data_bind, input)
    }

    pub fn data_bind_reverse_convert_for_object<'a>(
        &'a self,
        data_bind: &'a RuntimeObject,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let object_id = usize::try_from(data_bind.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_bind.type_key)?;
        if !definition.is_a("DataBind") {
            return None;
        }

        let converter = self.resolved_data_converter_for_data_bind_object(data_bind)?;
        self.cpp_data_converter_reverse_convert(
            converter,
            &RuntimeConvertedDataValue::from(input),
            Some(data_bind.uint_property("flags").unwrap_or(0)),
            &[],
            None,
            &mut BTreeSet::new(),
        )
    }

    pub fn data_bind_reverse_convert_with_context<'a>(
        &'a self,
        data_bind_id: usize,
        input: &RuntimeDataValue<'a>,
        view_model_instance_ids: &[usize],
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let data_bind = self.object(data_bind_id)?;
        let view_model_instances = view_model_instance_ids
            .iter()
            .map(|id| self.object(*id))
            .collect::<Option<Vec<_>>>()?;
        self.data_bind_reverse_convert_with_context_for_object(
            data_bind,
            input,
            &view_model_instances,
        )
    }

    pub fn data_bind_reverse_convert_with_context_for_object<'a>(
        &'a self,
        data_bind: &'a RuntimeObject,
        input: &RuntimeDataValue<'a>,
        view_model_instances: &[&'a RuntimeObject],
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let object_id = usize::try_from(data_bind.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_bind.type_key)?;
        if !definition.is_a("DataBind") {
            return None;
        }

        let converter = self.resolved_data_converter_for_data_bind_object(data_bind)?;
        self.cpp_data_converter_reverse_convert(
            converter,
            &RuntimeConvertedDataValue::from(input),
            Some(data_bind.uint_property("flags").unwrap_or(0)),
            view_model_instances,
            None,
            &mut BTreeSet::new(),
        )
    }

    pub fn data_converter_stateful_convert<'a>(
        &'a self,
        data_converter_index: usize,
        state: &mut RuntimeDataConverterState,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_stateful_convert_for_object(data_converter, state, input)
    }

    pub fn data_converter_stateful_convert_for_object<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
        state: &mut RuntimeDataConverterState,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        self.validate_data_converter(data_converter)?;
        self.cpp_data_converter_stateful_convert(
            data_converter,
            &RuntimeConvertedDataValue::from(input),
            state,
            false,
            &mut BTreeSet::new(),
        )
    }

    pub fn data_converter_stateful_reverse_convert<'a>(
        &'a self,
        data_converter_index: usize,
        state: &mut RuntimeDataConverterState,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_stateful_reverse_convert_for_object(data_converter, state, input)
    }

    pub fn data_converter_stateful_reverse_convert_for_object<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
        state: &mut RuntimeDataConverterState,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        self.validate_data_converter(data_converter)?;
        self.cpp_data_converter_stateful_convert(
            data_converter,
            &RuntimeConvertedDataValue::from(input),
            state,
            true,
            &mut BTreeSet::new(),
        )
    }

    pub fn data_converter_stateful_advance(
        &self,
        data_converter_index: usize,
        state: &mut RuntimeDataConverterState,
        elapsed_seconds: f32,
    ) -> Option<bool> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_stateful_advance_for_object(data_converter, state, elapsed_seconds)
    }

    pub fn data_converter_stateful_advance_for_object(
        &self,
        data_converter: &RuntimeObject,
        state: &mut RuntimeDataConverterState,
        elapsed_seconds: f32,
    ) -> Option<bool> {
        self.validate_data_converter(data_converter)?;
        self.cpp_data_converter_stateful_advance(
            data_converter,
            state,
            elapsed_seconds,
            &mut BTreeSet::new(),
        )
    }

    pub fn data_bind_stateful_advance(
        &self,
        data_bind_id: usize,
        state: &mut RuntimeDataConverterState,
        elapsed_seconds: f32,
        source_is_bound: bool,
        data_bind_is_collapsed: bool,
    ) -> Option<bool> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_stateful_advance_for_object(
            data_bind,
            state,
            elapsed_seconds,
            source_is_bound,
            data_bind_is_collapsed,
        )
    }

    pub fn data_bind_stateful_advance_for_object(
        &self,
        data_bind: &RuntimeObject,
        state: &mut RuntimeDataConverterState,
        elapsed_seconds: f32,
        source_is_bound: bool,
        data_bind_is_collapsed: bool,
    ) -> Option<bool> {
        self.validate_data_bind(data_bind)?;
        if !source_is_bound || data_bind_is_collapsed {
            return Some(false);
        }
        let Some(converter) = self.resolved_data_converter_for_data_bind_object(data_bind) else {
            return Some(false);
        };
        self.cpp_data_converter_stateful_advance(
            converter,
            state,
            elapsed_seconds,
            &mut BTreeSet::new(),
        )
    }

    pub fn data_converter_interpolator_convert<'a>(
        &'a self,
        data_converter_index: usize,
        state: &mut RuntimeDataConverterInterpolatorState,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_interpolator_convert_for_object(data_converter, state, input)
    }

    pub fn data_converter_interpolator_convert_for_object<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
        state: &mut RuntimeDataConverterInterpolatorState,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        self.validate_data_converter_interpolator(data_converter)?;
        state.convert(data_converter, input)
    }

    pub fn data_converter_interpolator_reverse_convert<'a>(
        &'a self,
        data_converter_index: usize,
        state: &mut RuntimeDataConverterInterpolatorState,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        self.data_converter_interpolator_convert(data_converter_index, state, input)
    }

    pub fn data_converter_interpolator_reverse_convert_for_object<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
        state: &mut RuntimeDataConverterInterpolatorState,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        self.data_converter_interpolator_convert_for_object(data_converter, state, input)
    }

    pub fn data_converter_interpolator_advance(
        &self,
        data_converter_index: usize,
        state: &mut RuntimeDataConverterInterpolatorState,
        elapsed_seconds: f32,
    ) -> Option<bool> {
        let data_converter = self.data_converter(data_converter_index)?;
        self.data_converter_interpolator_advance_for_object(data_converter, state, elapsed_seconds)
    }

    pub fn data_converter_interpolator_advance_for_object(
        &self,
        data_converter: &RuntimeObject,
        state: &mut RuntimeDataConverterInterpolatorState,
        elapsed_seconds: f32,
    ) -> Option<bool> {
        self.validate_data_converter_interpolator(data_converter)?;
        state.advance(self, data_converter, elapsed_seconds)
    }

    pub fn data_converter_interpolators(&self) -> Vec<&RuntimeObject> {
        self.cpp_data_converter_interpolators()
    }

    pub fn data_converter_interpolator(&self, index: usize) -> Option<&RuntimeObject> {
        self.cpp_data_converter_interpolators()
            .into_iter()
            .nth(index)
    }

    pub fn data_converter_formula_tokens(
        &self,
        data_converter_index: usize,
    ) -> Vec<&RuntimeObject> {
        self.cpp_data_converter_formula_tokens(data_converter_index)
    }

    pub fn data_converter_formula_output_tokens(
        &self,
        data_converter_index: usize,
    ) -> Vec<RuntimeFormulaOutputToken<'_>> {
        self.cpp_data_converter_formula_output_tokens(data_converter_index)
    }

    pub fn data_converter_formula_tokens_for_object(
        &self,
        data_converter: &RuntimeObject,
    ) -> Vec<&RuntimeObject> {
        let Some(index) = self
            .data_converters()
            .into_iter()
            .position(|candidate| candidate.id == data_converter.id)
        else {
            return Vec::new();
        };

        self.cpp_data_converter_formula_tokens(index)
    }

    pub fn data_converter_formula_output_tokens_for_object(
        &self,
        data_converter: &RuntimeObject,
    ) -> Vec<RuntimeFormulaOutputToken<'_>> {
        let Some(index) = self
            .data_converters()
            .into_iter()
            .position(|candidate| candidate.id == data_converter.id)
        else {
            return Vec::new();
        };

        self.cpp_data_converter_formula_output_tokens(index)
    }

    pub fn resolved_view_model_for_number_to_list_converter(
        &self,
        data_converter_id: usize,
    ) -> Option<RuntimeViewModel<'_>> {
        let data_converter = self.object(data_converter_id)?;
        self.resolved_view_model_for_number_to_list_converter_object(data_converter)
    }

    pub fn resolved_view_model_for_number_to_list_converter_object(
        &self,
        data_converter: &RuntimeObject,
    ) -> Option<RuntimeViewModel<'_>> {
        let object_id = usize::try_from(data_converter.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }
        if data_converter.type_name != "DataConverterNumberToList" {
            return None;
        }

        let view_model_index =
            usize::try_from(data_converter.uint_property("viewModelId")?).ok()?;
        self.view_model(view_model_index)
    }

    pub fn scroll_physics(&self) -> Vec<&RuntimeObject> {
        self.cpp_scroll_physics().collect()
    }

    pub fn scroll_physics_object(&self, index: usize) -> Option<&RuntimeObject> {
        self.cpp_scroll_physics().nth(index)
    }

    pub fn resolved_scroll_physics_for_constraint(
        &self,
        scroll_constraint_id: usize,
    ) -> Option<&RuntimeObject> {
        let scroll_constraint = self.object(scroll_constraint_id)?;
        self.resolved_scroll_physics_for_constraint_object(scroll_constraint)
    }

    pub fn resolved_scroll_physics_for_constraint_object(
        &self,
        scroll_constraint: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let object_id = usize::try_from(scroll_constraint.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }
        if scroll_constraint.type_name != "ScrollConstraint" {
            return None;
        }

        let physics_index = usize::try_from(scroll_constraint.uint_property("physicsId")?).ok()?;
        self.scroll_physics_object(physics_index)
    }

    pub fn resolved_interpolator_for_data_converter(
        &self,
        data_converter_id: usize,
    ) -> Option<&RuntimeObject> {
        let data_converter = self.object(data_converter_id)?;
        self.resolved_interpolator_for_data_converter_object(data_converter)
    }

    pub fn resolved_interpolator_for_data_converter_object(
        &self,
        data_converter: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let object_id = usize::try_from(data_converter.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }
        if !matches!(
            data_converter.type_name,
            "DataConverterRangeMapper" | "DataConverterInterpolator"
        ) {
            return None;
        }

        let interpolator_index =
            usize::try_from(data_converter.uint_property("interpolatorId")?).ok()?;
        self.data_converter_interpolator(interpolator_index)
    }

    pub fn resolved_data_converter_for_data_bind(
        &self,
        data_bind_id: usize,
    ) -> Option<&RuntimeObject> {
        let data_bind = self.object(data_bind_id)?;
        self.resolved_data_converter_for_data_bind_object(data_bind)
    }

    pub fn resolved_data_converter_for_data_bind_object(
        &self,
        data_bind: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let object_id = usize::try_from(data_bind.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_bind.type_key)?;
        if !definition.is_a("DataBind") {
            return None;
        }

        let converter_index = usize::try_from(data_bind.uint_property("converterId")?).ok()?;
        self.data_converter(converter_index)
    }

    pub fn data_bind_source_output_type(&self, data_bind_id: usize) -> Option<RuntimeDataType> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_source_output_type_for_object(data_bind)
    }

    pub fn data_bind_source_output_type_for_object(
        &self,
        data_bind: &RuntimeObject,
    ) -> Option<RuntimeDataType> {
        let object_id = usize::try_from(data_bind.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_bind.type_key)?;
        if !definition.is_a("DataBind") {
            return None;
        }

        Some(RuntimeDataType::None)
    }

    pub fn data_bind_output_type(&self, data_bind_id: usize) -> Option<RuntimeDataType> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_output_type_for_object(data_bind)
    }

    pub fn data_bind_output_type_for_object(
        &self,
        data_bind: &RuntimeObject,
    ) -> Option<RuntimeDataType> {
        let object_id = usize::try_from(data_bind.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_bind.type_key)?;
        if !definition.is_a("DataBind") {
            return None;
        }

        if let Some(converter) = self.resolved_data_converter_for_data_bind_object(data_bind) {
            let converter_output_type = self.data_converter_output_type_for_object(converter)?;
            if converter_output_type != RuntimeDataType::Input
                && converter_output_type != RuntimeDataType::None
            {
                return Some(converter_output_type);
            }
        }

        self.data_bind_source_output_type_for_object(data_bind)
    }

    pub fn data_bind_to_source(&self, data_bind_id: usize) -> Option<bool> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_to_source_for_object(data_bind)
    }

    pub fn data_bind_to_source_for_object(&self, data_bind: &RuntimeObject) -> Option<bool> {
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_to_source(data_bind))
    }

    pub fn data_bind_to_target(&self, data_bind_id: usize) -> Option<bool> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_to_target_for_object(data_bind)
    }

    pub fn data_bind_to_target_for_object(&self, data_bind: &RuntimeObject) -> Option<bool> {
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_to_target(data_bind))
    }

    pub fn data_bind_binds_once(&self, data_bind_id: usize) -> Option<bool> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_binds_once_for_object(data_bind)
    }

    pub fn data_bind_binds_once_for_object(&self, data_bind: &RuntimeObject) -> Option<bool> {
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_binds_once(data_bind))
    }

    pub fn data_bind_is_main_to_source(&self, data_bind_id: usize) -> Option<bool> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_is_main_to_source_for_object(data_bind)
    }

    pub fn data_bind_is_main_to_source_for_object(
        &self,
        data_bind: &RuntimeObject,
    ) -> Option<bool> {
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_is_main_to_source(data_bind))
    }

    pub fn data_bind_source_to_target_runs_first(&self, data_bind_id: usize) -> Option<bool> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_source_to_target_runs_first_for_object(data_bind)
    }

    pub fn data_bind_source_to_target_runs_first_for_object(
        &self,
        data_bind: &RuntimeObject,
    ) -> Option<bool> {
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_source_to_target_runs_first(data_bind))
    }

    pub fn data_bind_reconcile_dirt(&self, data_bind_id: usize) -> Option<RuntimeComponentDirt> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_reconcile_dirt_for_object(data_bind)
    }

    pub fn data_bind_reconcile_dirt_for_object(
        &self,
        data_bind: &RuntimeObject,
    ) -> Option<RuntimeComponentDirt> {
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_reconcile_dirt(data_bind))
    }

    pub fn data_bind_is_name_based(&self, data_bind_id: usize) -> Option<bool> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_is_name_based_for_object(data_bind)
    }

    pub fn data_bind_is_name_based_for_object(&self, data_bind: &RuntimeObject) -> Option<bool> {
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_is_name_based(data_bind))
    }

    pub fn data_bind_target_supports_push(&self, data_bind_id: usize) -> Option<bool> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_target_supports_push_for_object(data_bind)
    }

    pub fn data_bind_target_supports_push_for_object(
        &self,
        data_bind: &RuntimeObject,
    ) -> Option<bool> {
        self.validate_data_bind(data_bind)?;
        let target = self.cpp_data_bind_target_for_object(data_bind);
        Some(cpp_data_bind_target_supports_push(data_bind, target))
    }

    pub fn data_bind_uses_persisting_list(&self, data_bind_id: usize) -> Option<bool> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_uses_persisting_list_for_object(data_bind)
    }

    pub fn data_bind_uses_persisting_list_for_object(
        &self,
        data_bind: &RuntimeObject,
    ) -> Option<bool> {
        self.validate_data_bind(data_bind)?;
        Some(
            cpp_data_bind_to_source(data_bind)
                && !self.data_bind_target_supports_push_for_object(data_bind)?,
        )
    }

    pub fn data_bind_can_skip(
        &self,
        data_bind_id: usize,
        target_is_collapsed: bool,
    ) -> Option<bool> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_can_skip_for_object(data_bind, target_is_collapsed)
    }

    pub fn data_bind_can_skip_for_object(
        &self,
        data_bind: &RuntimeObject,
        target_is_collapsed: bool,
    ) -> Option<bool> {
        self.validate_data_bind(data_bind)?;
        let target = self.cpp_data_bind_target_for_object(data_bind);
        Some(cpp_data_bind_can_skip(
            data_bind,
            target,
            target_is_collapsed,
        ))
    }

    pub fn data_bind_collapse_effect(
        &self,
        data_bind_id: usize,
        is_collapsed: bool,
        requested_is_collapsed: bool,
        has_dirt: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindCollapseEffect> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_collapse_effect_for_object(
            data_bind,
            is_collapsed,
            requested_is_collapsed,
            has_dirt,
            has_container,
        )
    }

    pub fn data_bind_collapse_effect_for_object(
        &self,
        data_bind: &RuntimeObject,
        is_collapsed: bool,
        requested_is_collapsed: bool,
        has_dirt: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindCollapseEffect> {
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_collapse_effect(
            data_bind,
            self.data_bind_target_supports_push_for_object(data_bind)?,
            is_collapsed,
            requested_is_collapsed,
            has_dirt,
            has_container,
        ))
    }

    pub fn data_bind_add_dirt_effect(
        &self,
        data_bind_id: usize,
        current_dirt: RuntimeComponentDirt,
        added_dirt: RuntimeComponentDirt,
        suppress_dirt: bool,
        is_collapsed: bool,
        has_context_value: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindAddDirtEffect> {
        self.data_bind_add_dirt_effect_with_origin(
            data_bind_id,
            current_dirt,
            false,
            added_dirt,
            suppress_dirt,
            is_collapsed,
            has_context_value,
            has_container,
        )
    }

    pub fn data_bind_add_dirt_effect_with_origin(
        &self,
        data_bind_id: usize,
        current_dirt: RuntimeComponentDirt,
        current_target_origin: bool,
        added_dirt: RuntimeComponentDirt,
        suppress_dirt: bool,
        is_collapsed: bool,
        has_context_value: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindAddDirtEffect> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_add_dirt_effect_for_object_with_origin(
            data_bind,
            current_dirt,
            current_target_origin,
            added_dirt,
            suppress_dirt,
            is_collapsed,
            has_context_value,
            has_container,
        )
    }

    pub fn data_bind_add_dirt_effect_for_object(
        &self,
        data_bind: &RuntimeObject,
        current_dirt: RuntimeComponentDirt,
        added_dirt: RuntimeComponentDirt,
        suppress_dirt: bool,
        is_collapsed: bool,
        has_context_value: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindAddDirtEffect> {
        self.data_bind_add_dirt_effect_for_object_with_origin(
            data_bind,
            current_dirt,
            false,
            added_dirt,
            suppress_dirt,
            is_collapsed,
            has_context_value,
            has_container,
        )
    }

    pub fn data_bind_add_dirt_effect_for_object_with_origin(
        &self,
        data_bind: &RuntimeObject,
        current_dirt: RuntimeComponentDirt,
        current_target_origin: bool,
        added_dirt: RuntimeComponentDirt,
        suppress_dirt: bool,
        is_collapsed: bool,
        has_context_value: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindAddDirtEffect> {
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_add_dirt_effect(
            current_dirt,
            current_target_origin,
            added_dirt,
            cpp_data_bind_source_to_target_runs_first(data_bind),
            suppress_dirt,
            is_collapsed,
            has_context_value,
            has_container,
        ))
    }

    pub fn data_bind_add_effect(
        &self,
        data_bind_id: usize,
        is_processing: bool,
        has_data_context: bool,
    ) -> Option<RuntimeDataBindAddEffect> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_add_effect_for_object(data_bind, is_processing, has_data_context)
    }

    pub fn data_bind_add_effect_for_object(
        &self,
        data_bind: &RuntimeObject,
        is_processing: bool,
        has_data_context: bool,
    ) -> Option<RuntimeDataBindAddEffect> {
        self.validate_data_bind(data_bind)?;
        let data_bind_definition = definition_by_type_key(data_bind.type_key)?;
        Some(cpp_data_bind_add_effect(
            data_bind,
            self.data_bind_target_supports_push_for_object(data_bind)?,
            data_bind_definition.is_a("DataBindContext"),
            is_processing,
            has_data_context,
        ))
    }

    pub fn data_bind_source_effect(
        &self,
        data_bind_id: usize,
        source_data_type: RuntimeDataType,
    ) -> Option<RuntimeDataBindSourceEffect> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_source_effect_for_object(data_bind, source_data_type)
    }

    pub fn data_bind_source_effect_for_object(
        &self,
        data_bind: &RuntimeObject,
        source_data_type: RuntimeDataType,
    ) -> Option<RuntimeDataBindSourceEffect> {
        self.validate_data_bind(data_bind)?;
        let target = self.cpp_data_bind_target_for_object(data_bind);
        Some(cpp_data_bind_source_effect(
            data_bind,
            target,
            source_data_type,
        ))
    }

    pub fn data_bind_clear_source_effect(
        &self,
        data_bind_id: usize,
        has_source: bool,
    ) -> Option<RuntimeDataBindClearSourceEffect> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_clear_source_effect_for_object(data_bind, has_source)
    }

    pub fn data_bind_clear_source_effect_for_object(
        &self,
        data_bind: &RuntimeObject,
        has_source: bool,
    ) -> Option<RuntimeDataBindClearSourceEffect> {
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_clear_source_effect(data_bind, has_source))
    }

    pub fn data_bind_bind_effect(
        &self,
        data_bind_id: usize,
        source_data_type: RuntimeDataType,
        has_target: bool,
        is_observing: bool,
        has_context_value: bool,
        current_dirt: RuntimeComponentDirt,
        is_collapsed: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindBindEffect> {
        self.data_bind_bind_effect_with_origin(
            data_bind_id,
            source_data_type,
            has_target,
            is_observing,
            has_context_value,
            current_dirt,
            false,
            is_collapsed,
            has_container,
        )
    }

    pub fn data_bind_bind_effect_with_origin(
        &self,
        data_bind_id: usize,
        source_data_type: RuntimeDataType,
        has_target: bool,
        is_observing: bool,
        has_context_value: bool,
        current_dirt: RuntimeComponentDirt,
        current_target_origin: bool,
        is_collapsed: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindBindEffect> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_bind_effect_for_object_with_origin(
            data_bind,
            source_data_type,
            has_target,
            is_observing,
            has_context_value,
            current_dirt,
            current_target_origin,
            is_collapsed,
            has_container,
        )
    }

    pub fn data_bind_bind_effect_for_object(
        &self,
        data_bind: &RuntimeObject,
        source_data_type: RuntimeDataType,
        has_target: bool,
        is_observing: bool,
        has_context_value: bool,
        current_dirt: RuntimeComponentDirt,
        is_collapsed: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindBindEffect> {
        self.data_bind_bind_effect_for_object_with_origin(
            data_bind,
            source_data_type,
            has_target,
            is_observing,
            has_context_value,
            current_dirt,
            false,
            is_collapsed,
            has_container,
        )
    }

    pub fn data_bind_bind_effect_for_object_with_origin(
        &self,
        data_bind: &RuntimeObject,
        source_data_type: RuntimeDataType,
        has_target: bool,
        is_observing: bool,
        has_context_value: bool,
        current_dirt: RuntimeComponentDirt,
        current_target_origin: bool,
        is_collapsed: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindBindEffect> {
        self.validate_data_bind(data_bind)?;
        let converter = self.resolved_data_converter_for_data_bind_object(data_bind);
        let converter_output_type = match converter {
            Some(converter) => Some(self.data_converter_output_type_for_object(converter)?),
            None => None,
        };
        let target_supports_push =
            has_target && self.data_bind_target_supports_push_for_object(data_bind)?;
        Some(cpp_data_bind_bind_effect(
            data_bind,
            converter.is_some(),
            converter_output_type,
            source_data_type,
            has_target,
            is_observing,
            has_context_value,
            current_dirt,
            current_target_origin,
            is_collapsed,
            has_container,
            target_supports_push,
        ))
    }

    pub fn data_bind_target_effect(
        &self,
        data_bind_id: usize,
        new_target_id: Option<usize>,
        current_target_is_same: bool,
        has_current_target: bool,
        is_observing: bool,
    ) -> Option<RuntimeDataBindTargetEffect> {
        let data_bind = self.object(data_bind_id)?;
        let new_target = match new_target_id {
            Some(new_target_id) => {
                if self.import_status(new_target_id) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }
                Some(self.object(new_target_id)?)
            }
            None => None,
        };
        self.data_bind_target_effect_for_object(
            data_bind,
            new_target,
            current_target_is_same,
            has_current_target,
            is_observing,
        )
    }

    pub fn data_bind_target_effect_for_object(
        &self,
        data_bind: &RuntimeObject,
        new_target: Option<&RuntimeObject>,
        current_target_is_same: bool,
        has_current_target: bool,
        is_observing: bool,
    ) -> Option<RuntimeDataBindTargetEffect> {
        self.validate_data_bind(data_bind)?;
        if let Some(new_target) = new_target {
            let new_target_id = usize::try_from(new_target.id).ok()?;
            if self.import_status(new_target_id) != Some(RuntimeImportStatus::Imported) {
                return None;
            }
        }
        let new_target_supports_push = cpp_data_bind_target_supports_push(data_bind, new_target);
        Some(cpp_data_bind_target_effect(
            data_bind,
            current_target_is_same,
            has_current_target,
            new_target.is_some(),
            is_observing,
            new_target_supports_push,
        ))
    }

    pub fn data_bind_unbind_effect(
        &self,
        data_bind_id: usize,
        has_source: bool,
        has_target: bool,
        is_observing: bool,
        has_context_value: bool,
    ) -> Option<RuntimeDataBindUnbindEffect> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_unbind_effect_for_object(
            data_bind,
            has_source,
            has_target,
            is_observing,
            has_context_value,
        )
    }

    pub fn data_bind_unbind_effect_for_object(
        &self,
        data_bind: &RuntimeObject,
        has_source: bool,
        has_target: bool,
        is_observing: bool,
        has_context_value: bool,
    ) -> Option<RuntimeDataBindUnbindEffect> {
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_unbind_effect(
            data_bind,
            self.resolved_data_converter_for_data_bind_object(data_bind)
                .is_some(),
            has_source,
            has_target,
            is_observing,
            has_context_value,
        ))
    }

    pub fn data_bind_initialize_effect(
        &self,
        data_bind_id: usize,
        already_collapsable: bool,
        is_collapsed: bool,
        target_is_collapsed: bool,
        has_dirt: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindInitializeEffect> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_initialize_effect_for_object(
            data_bind,
            already_collapsable,
            is_collapsed,
            target_is_collapsed,
            has_dirt,
            has_container,
        )
    }

    pub fn data_bind_initialize_effect_for_object(
        &self,
        data_bind: &RuntimeObject,
        already_collapsable: bool,
        is_collapsed: bool,
        target_is_collapsed: bool,
        has_dirt: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindInitializeEffect> {
        self.validate_data_bind(data_bind)?;
        let target = self.cpp_data_bind_target_for_object(data_bind);
        Some(cpp_data_bind_initialize_effect(
            data_bind,
            target,
            self.data_bind_target_supports_push_for_object(data_bind)?,
            already_collapsable,
            is_collapsed,
            target_is_collapsed,
            has_dirt,
            has_container,
        ))
    }

    pub fn data_bind_relink_effect(
        &self,
        data_bind_id: usize,
        has_container: bool,
        has_data_context: bool,
    ) -> Option<RuntimeDataBindRelinkEffect> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_relink_effect_for_object(data_bind, has_container, has_data_context)
    }

    pub fn data_bind_relink_effect_for_object(
        &self,
        data_bind: &RuntimeObject,
        has_container: bool,
        has_data_context: bool,
    ) -> Option<RuntimeDataBindRelinkEffect> {
        self.validate_data_bind(data_bind)?;
        let definition = definition_by_type_key(data_bind.type_key)?;
        Some(cpp_data_bind_relink_effect(
            definition.is_a("DataBindContext"),
            has_container,
            has_data_context,
        ))
    }

    pub fn data_bind_context_bind_effect(
        &self,
        data_bind_id: usize,
        source_path_is_resolved: bool,
        has_data_context: bool,
        lookup_has_source: bool,
        source_matches_lookup: bool,
        has_source: bool,
        source_data_type: RuntimeDataType,
        has_target: bool,
        is_observing: bool,
        has_context_value: bool,
        current_dirt: RuntimeComponentDirt,
        is_collapsed: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindContextBindEffect> {
        self.data_bind_context_bind_effect_with_origin(
            data_bind_id,
            source_path_is_resolved,
            has_data_context,
            lookup_has_source,
            source_matches_lookup,
            has_source,
            source_data_type,
            has_target,
            is_observing,
            has_context_value,
            current_dirt,
            false,
            is_collapsed,
            has_container,
        )
    }

    pub fn data_bind_context_bind_effect_with_origin(
        &self,
        data_bind_id: usize,
        source_path_is_resolved: bool,
        has_data_context: bool,
        lookup_has_source: bool,
        source_matches_lookup: bool,
        has_source: bool,
        source_data_type: RuntimeDataType,
        has_target: bool,
        is_observing: bool,
        has_context_value: bool,
        current_dirt: RuntimeComponentDirt,
        current_target_origin: bool,
        is_collapsed: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindContextBindEffect> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_context_bind_effect_for_object_with_origin(
            data_bind,
            source_path_is_resolved,
            has_data_context,
            lookup_has_source,
            source_matches_lookup,
            has_source,
            source_data_type,
            has_target,
            is_observing,
            has_context_value,
            current_dirt,
            current_target_origin,
            is_collapsed,
            has_container,
        )
    }

    pub fn data_bind_context_bind_effect_for_object(
        &self,
        data_bind: &RuntimeObject,
        source_path_is_resolved: bool,
        has_data_context: bool,
        lookup_has_source: bool,
        source_matches_lookup: bool,
        has_source: bool,
        source_data_type: RuntimeDataType,
        has_target: bool,
        is_observing: bool,
        has_context_value: bool,
        current_dirt: RuntimeComponentDirt,
        is_collapsed: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindContextBindEffect> {
        self.data_bind_context_bind_effect_for_object_with_origin(
            data_bind,
            source_path_is_resolved,
            has_data_context,
            lookup_has_source,
            source_matches_lookup,
            has_source,
            source_data_type,
            has_target,
            is_observing,
            has_context_value,
            current_dirt,
            false,
            is_collapsed,
            has_container,
        )
    }

    pub fn data_bind_context_bind_effect_for_object_with_origin(
        &self,
        data_bind: &RuntimeObject,
        source_path_is_resolved: bool,
        has_data_context: bool,
        lookup_has_source: bool,
        source_matches_lookup: bool,
        has_source: bool,
        source_data_type: RuntimeDataType,
        has_target: bool,
        is_observing: bool,
        has_context_value: bool,
        current_dirt: RuntimeComponentDirt,
        current_target_origin: bool,
        is_collapsed: bool,
        has_container: bool,
    ) -> Option<RuntimeDataBindContextBindEffect> {
        self.validate_data_bind(data_bind)?;
        if data_bind.type_name != "DataBindContext" {
            return None;
        }

        let source_path_ids = self.data_bind_context_source_path_ids_for_object(data_bind)?;
        let resolved_source_path_ids =
            self.data_bind_context_resolved_source_path_ids_for_object(data_bind)?;
        let target = self.cpp_data_bind_target_for_object(data_bind);
        let converter = self.resolved_data_converter_for_data_bind_object(data_bind);
        let converter_output_type = match converter {
            Some(converter) => Some(self.data_converter_output_type_for_object(converter)?),
            None => None,
        };
        let target_supports_push =
            has_target && self.data_bind_target_supports_push_for_object(data_bind)?;

        Some(cpp_data_bind_context_bind_effect(
            data_bind,
            target,
            converter.is_some(),
            converter_output_type,
            &source_path_ids,
            &resolved_source_path_ids,
            source_path_is_resolved,
            has_data_context,
            lookup_has_source,
            source_matches_lookup,
            has_source,
            source_data_type,
            has_target,
            is_observing,
            has_context_value,
            current_dirt,
            current_target_origin,
            is_collapsed,
            has_container,
            target_supports_push,
        ))
    }

    pub fn data_bind_update_effect(
        &self,
        data_bind_id: usize,
        dirt: RuntimeComponentDirt,
        apply_target_to_source: bool,
        has_source: bool,
        has_context_value: bool,
        has_target: bool,
    ) -> Option<RuntimeDataBindUpdateEffect> {
        let data_bind = self.object(data_bind_id)?;
        let in_persisting_list = self.data_bind_uses_persisting_list_for_object(data_bind)?;
        self.data_bind_update_effect_for_object_with_persisting_state(
            data_bind,
            dirt,
            apply_target_to_source,
            in_persisting_list,
            has_source,
            has_context_value,
            has_target,
        )
    }

    pub fn data_bind_update_effect_with_persisting_state(
        &self,
        data_bind_id: usize,
        dirt: RuntimeComponentDirt,
        apply_target_to_source: bool,
        in_persisting_list: bool,
        has_source: bool,
        has_context_value: bool,
        has_target: bool,
    ) -> Option<RuntimeDataBindUpdateEffect> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_update_effect_for_object_with_persisting_state(
            data_bind,
            dirt,
            apply_target_to_source,
            in_persisting_list,
            has_source,
            has_context_value,
            has_target,
        )
    }

    pub fn data_bind_update_effect_for_object(
        &self,
        data_bind: &RuntimeObject,
        dirt: RuntimeComponentDirt,
        apply_target_to_source: bool,
        has_source: bool,
        has_context_value: bool,
        has_target: bool,
    ) -> Option<RuntimeDataBindUpdateEffect> {
        let in_persisting_list = self.data_bind_uses_persisting_list_for_object(data_bind)?;
        self.data_bind_update_effect_for_object_with_persisting_state(
            data_bind,
            dirt,
            apply_target_to_source,
            in_persisting_list,
            has_source,
            has_context_value,
            has_target,
        )
    }

    pub fn data_bind_update_effect_for_object_with_persisting_state(
        &self,
        data_bind: &RuntimeObject,
        dirt: RuntimeComponentDirt,
        apply_target_to_source: bool,
        in_persisting_list: bool,
        has_source: bool,
        has_context_value: bool,
        has_target: bool,
    ) -> Option<RuntimeDataBindUpdateEffect> {
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_update_effect(
            data_bind,
            self.resolved_data_converter_for_data_bind_object(data_bind)
                .is_some(),
            dirt,
            apply_target_to_source,
            in_persisting_list,
            has_source,
            has_context_value,
            has_target,
        ))
    }

    pub fn data_bind_remove_effect(
        &self,
        data_bind_id: usize,
        is_processing: bool,
        in_persisting_list: bool,
        in_dirty_list: bool,
    ) -> Option<RuntimeDataBindRemoveEffect> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_remove_effect_for_object(
            data_bind,
            is_processing,
            in_persisting_list,
            in_dirty_list,
        )
    }

    pub fn data_bind_remove_effect_for_object(
        &self,
        data_bind: &RuntimeObject,
        is_processing: bool,
        in_persisting_list: bool,
        in_dirty_list: bool,
    ) -> Option<RuntimeDataBindRemoveEffect> {
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_remove_effect(
            is_processing,
            in_persisting_list,
            in_dirty_list,
        ))
    }

    pub fn data_bind_container_bind_context_effect(
        &self,
        data_bind_ids: &[usize],
        has_data_context: bool,
    ) -> Option<RuntimeDataBindContainerBindContextEffect> {
        let mut data_bind_context_ids = Vec::new();
        for data_bind_id in data_bind_ids {
            let data_bind = self.object(*data_bind_id)?;
            self.validate_data_bind(data_bind)?;
            let definition = definition_by_type_key(data_bind.type_key)?;
            if definition.is_a("DataBindContext") {
                data_bind_context_ids.push(*data_bind_id);
            }
        }
        Some(cpp_data_bind_container_bind_context_effect(
            &data_bind_context_ids,
            has_data_context,
        ))
    }

    pub fn data_bind_container_unbind_effect(
        &self,
        data_bind_ids: &[usize],
    ) -> Option<RuntimeDataBindContainerUnbindEffect> {
        for data_bind_id in data_bind_ids {
            let data_bind = self.object(*data_bind_id)?;
            self.validate_data_bind(data_bind)?;
        }
        Some(cpp_data_bind_container_unbind_effect(data_bind_ids))
    }

    pub fn data_bind_container_advance_effect(
        &self,
        data_bind_ids: &[usize],
        advance_results: &[bool],
    ) -> Option<RuntimeDataBindContainerAdvanceEffect> {
        if data_bind_ids.len() != advance_results.len() {
            return None;
        }
        for data_bind_id in data_bind_ids {
            let data_bind = self.object(*data_bind_id)?;
            self.validate_data_bind(data_bind)?;
        }
        Some(cpp_data_bind_container_advance_effect(
            data_bind_ids,
            advance_results,
        ))
    }

    pub fn data_bind_container_add_dirty_effect(
        &self,
        data_bind_id: usize,
        in_persisting_list: bool,
        in_dirty_list: bool,
        is_processing: bool,
    ) -> Option<RuntimeDataBindContainerAddDirtyEffect> {
        let data_bind = self.object(data_bind_id)?;
        self.validate_data_bind(data_bind)?;
        Some(cpp_data_bind_container_add_dirty_effect(
            data_bind,
            in_persisting_list,
            in_dirty_list,
            is_processing,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn data_bind_container_update_effect(
        &self,
        persisting_data_bind_ids: &[usize],
        persisting_can_skip: &[bool],
        dirty_to_source_data_bind_ids: &[usize],
        dirty_to_target_data_bind_ids: &[usize],
        pending_dirty_to_source_data_bind_ids: &[usize],
        pending_dirty_to_target_data_bind_ids: &[usize],
        pending_addition_ids: &[usize],
        pending_removal_ids: &[usize],
        is_processing: bool,
        apply_target_to_source: bool,
    ) -> Option<RuntimeDataBindContainerUpdateEffect> {
        if persisting_data_bind_ids.len() != persisting_can_skip.len() {
            return None;
        }
        for data_bind_id in persisting_data_bind_ids
            .iter()
            .chain(dirty_to_source_data_bind_ids)
            .chain(dirty_to_target_data_bind_ids)
            .chain(pending_dirty_to_source_data_bind_ids)
            .chain(pending_dirty_to_target_data_bind_ids)
            .chain(pending_addition_ids)
            .chain(pending_removal_ids)
        {
            let data_bind = self.object(*data_bind_id)?;
            self.validate_data_bind(data_bind)?;
        }
        Some(cpp_data_bind_container_update_effect(
            persisting_data_bind_ids,
            persisting_can_skip,
            dirty_to_source_data_bind_ids,
            dirty_to_target_data_bind_ids,
            pending_dirty_to_source_data_bind_ids,
            pending_dirty_to_target_data_bind_ids,
            pending_addition_ids,
            pending_removal_ids,
            is_processing,
            apply_target_to_source,
        ))
    }

    pub fn data_bind_update_queue(
        &self,
        data_bind_id: usize,
    ) -> Option<RuntimeDataBindUpdateQueue> {
        let data_bind = self.object(data_bind_id)?;
        self.data_bind_update_queue_for_object(data_bind)
    }

    pub fn data_bind_update_queue_for_object(
        &self,
        data_bind: &RuntimeObject,
    ) -> Option<RuntimeDataBindUpdateQueue> {
        self.validate_data_bind(data_bind)?;
        if self.data_bind_uses_persisting_list_for_object(data_bind)? {
            return Some(RuntimeDataBindUpdateQueue::Persisting);
        }
        if cpp_data_bind_to_source(data_bind) {
            return Some(RuntimeDataBindUpdateQueue::DirtyToSource);
        }
        Some(RuntimeDataBindUpdateQueue::DirtyToTarget)
    }

    pub fn sorted_data_bind_ids(&self, data_bind_ids: &[usize]) -> Option<Vec<usize>> {
        for data_bind_id in data_bind_ids {
            let data_bind = self.object(*data_bind_id)?;
            self.validate_data_bind(data_bind)?;
        }
        Some(cpp_sort_data_bind_ids(data_bind_ids, |data_bind_id| {
            let data_bind = self
                .object(*data_bind_id)
                .expect("sorted DataBind ids were validated before sorting");
            cpp_data_bind_to_source(data_bind)
        }))
    }

    pub fn data_converter_group_items(
        &self,
        data_converter_index: usize,
    ) -> Vec<RuntimeDataConverterGroupItem<'_>> {
        self.cpp_data_converter_group_items(data_converter_index)
    }

    pub fn data_converter_group_items_for_object(
        &self,
        data_converter: &RuntimeObject,
    ) -> Vec<RuntimeDataConverterGroupItem<'_>> {
        let Some(index) = self
            .data_converters()
            .into_iter()
            .position(|candidate| candidate.id == data_converter.id)
        else {
            return Vec::new();
        };

        self.cpp_data_converter_group_items(index)
    }

    pub fn data_converter_group_item(
        &self,
        data_converter_index: usize,
        item_index: usize,
    ) -> Option<RuntimeDataConverterGroupItem<'_>> {
        self.cpp_data_converter_group_items(data_converter_index)
            .into_iter()
            .nth(item_index)
    }

    pub fn resolved_data_converter_for_group_item(
        &self,
        group_item_id: usize,
    ) -> Option<&RuntimeObject> {
        let group_item = self.object(group_item_id)?;
        self.resolved_data_converter_for_group_item_object(group_item)
    }

    pub fn resolved_data_converter_for_group_item_object(
        &self,
        group_item: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let object_id = usize::try_from(group_item.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }
        if group_item.type_name != "DataConverterGroupItem" {
            return None;
        }

        let converter_index = usize::try_from(group_item.uint_property("converterId")?).ok()?;
        self.data_converter(converter_index)
    }

    pub fn view_models(&self) -> Vec<RuntimeViewModel<'_>> {
        self.cpp_view_models()
    }

    pub fn view_model(&self, index: usize) -> Option<RuntimeViewModel<'_>> {
        self.cpp_view_models().into_iter().nth(index)
    }

    pub fn view_model_named(&self, name: &str) -> Option<RuntimeViewModel<'_>> {
        self.view_model_named_bytes(name.as_bytes())
    }

    pub fn view_model_named_bytes(&self, name: &[u8]) -> Option<RuntimeViewModel<'_>> {
        self.cpp_view_models().into_iter().find(|view_model| {
            view_model
                .object
                .string_property_bytes("name")
                .unwrap_or_default()
                == name
        })
    }

    pub fn view_model_property_named(
        &self,
        view_model_index: usize,
        name: &str,
    ) -> Option<&RuntimeObject> {
        self.view_model_property_named_bytes(view_model_index, name.as_bytes())
    }

    pub fn view_model_property_named_bytes(
        &self,
        view_model_index: usize,
        name: &[u8],
    ) -> Option<&RuntimeObject> {
        let view_model = self.view_model(view_model_index)?;
        view_model
            .properties
            .into_iter()
            .find(|property| property.string_property_bytes("name").unwrap_or_default() == name)
    }

    pub fn view_model_property_for_symbol(
        &self,
        view_model_index: usize,
        symbol_type: u8,
    ) -> Option<&RuntimeObject> {
        let view_model = self.view_model(view_model_index)?;
        view_model.properties.into_iter().find(|property| {
            property.uint_property("symbolTypeValue") == Some(u64::from(symbol_type))
        })
    }

    pub fn data_enum_for_view_model_property(
        &self,
        property_id: usize,
    ) -> Option<RuntimeDataEnum<'_>> {
        let property = self.object(property_id)?;
        self.data_enum_for_view_model_property_object(property)
    }

    pub fn data_enum_for_view_model_property_object(
        &self,
        property: &RuntimeObject,
    ) -> Option<RuntimeDataEnum<'_>> {
        if property.type_name != "ViewModelPropertyEnumCustom" {
            return None;
        }

        let property_id = usize::try_from(property.id).ok()?;
        if self.import_status(property_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let data_enum_index = usize::try_from(property.uint_property("enumId")?).ok()?;
        let data_enum = self.data_enum(data_enum_index)?;
        (data_enum.object.id < property.id).then_some(data_enum)
    }

    fn cpp_view_model_property_enum_data(
        &self,
        property: &RuntimeObject,
    ) -> Option<RuntimeViewModelPropertyEnumData<'_>> {
        let property_id = usize::try_from(property.id).ok()?;
        if self.import_status(property_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        match property.type_name {
            "ViewModelPropertyEnumCustom" => {
                let data_enum = self.data_enum_for_view_model_property_object(property)?;
                Some(RuntimeViewModelPropertyEnumData {
                    name: data_enum
                        .object
                        .string_property_bytes("name")
                        .unwrap_or_default(),
                    values: data_enum.values,
                })
            }
            "ViewModelPropertyEnumSystem" => Some(RuntimeViewModelPropertyEnumData {
                name: b"",
                values: Vec::new(),
            }),
            _ => None,
        }
    }

    pub fn view_model_property_enum_value_for_key_bytes(
        &self,
        property_id: usize,
        key: &[u8],
    ) -> Option<&[u8]> {
        let property = self.object(property_id)?;
        self.view_model_property_enum_value_for_key_bytes_object(property, key)
    }

    pub fn view_model_property_enum_value_for_key(
        &self,
        property_id: usize,
        key: &str,
    ) -> Option<&str> {
        std::str::from_utf8(
            self.view_model_property_enum_value_for_key_bytes(property_id, key.as_bytes())?,
        )
        .ok()
    }

    pub fn view_model_property_enum_value_for_key_bytes_object(
        &self,
        property: &RuntimeObject,
        key: &[u8],
    ) -> Option<&[u8]> {
        let enum_data = self.cpp_view_model_property_enum_data(property)?;
        let value = enum_data
            .values
            .into_iter()
            .find(|value| value.string_property_bytes("key").unwrap_or_default() == key)?;
        Some(cpp_data_enum_resolved_value_bytes(value))
    }

    pub fn view_model_property_enum_value_for_key_object(
        &self,
        property: &RuntimeObject,
        key: &str,
    ) -> Option<&str> {
        std::str::from_utf8(
            self.view_model_property_enum_value_for_key_bytes_object(property, key.as_bytes())?,
        )
        .ok()
    }

    pub fn view_model_property_enum_value_for_index(
        &self,
        property_id: usize,
        value_index: usize,
    ) -> Option<&[u8]> {
        let property = self.object(property_id)?;
        self.view_model_property_enum_value_for_index_object(property, value_index)
    }

    pub fn view_model_property_enum_value_for_index_object(
        &self,
        property: &RuntimeObject,
        value_index: usize,
    ) -> Option<&[u8]> {
        let enum_data = self.cpp_view_model_property_enum_data(property)?;
        let value = enum_data.values.get(value_index)?;
        Some(cpp_data_enum_resolved_value_bytes(value))
    }

    pub fn view_model_property_enum_value_index_for_key_bytes(
        &self,
        property_id: usize,
        key: &[u8],
    ) -> Option<usize> {
        let property = self.object(property_id)?;
        self.view_model_property_enum_value_index_for_key_bytes_object(property, key)
    }

    pub fn view_model_property_enum_value_index_for_key(
        &self,
        property_id: usize,
        key: &str,
    ) -> Option<usize> {
        self.view_model_property_enum_value_index_for_key_bytes(property_id, key.as_bytes())
    }

    pub fn view_model_property_enum_value_index_for_key_bytes_object(
        &self,
        property: &RuntimeObject,
        key: &[u8],
    ) -> Option<usize> {
        let enum_data = self.cpp_view_model_property_enum_data(property)?;
        enum_data
            .values
            .into_iter()
            .position(|value| value.string_property_bytes("key").unwrap_or_default() == key)
    }

    pub fn view_model_property_enum_value_index_for_index(
        &self,
        property_id: usize,
        value_index: usize,
    ) -> Option<usize> {
        let property = self.object(property_id)?;
        self.view_model_property_enum_value_index_for_index_object(property, value_index)
    }

    pub fn view_model_property_enum_value_index_for_index_object(
        &self,
        property: &RuntimeObject,
        value_index: usize,
    ) -> Option<usize> {
        let enum_data = self.cpp_view_model_property_enum_data(property)?;
        (value_index < enum_data.values.len()).then_some(value_index)
    }

    pub fn view_model_instance_named(
        &self,
        view_model_index: usize,
        name: &str,
    ) -> Option<RuntimeViewModelInstanceReference<'_>> {
        self.view_model_instance_named_bytes(view_model_index, name.as_bytes())
    }

    pub fn view_model_instance_named_bytes(
        &self,
        view_model_index: usize,
        name: &[u8],
    ) -> Option<RuntimeViewModelInstanceReference<'_>> {
        let view_model = self.view_model(view_model_index)?;
        view_model
            .instances
            .into_iter()
            .enumerate()
            .find_map(|(instance_index, instance)| {
                (instance
                    .object
                    .string_property_bytes("name")
                    .unwrap_or_default()
                    == name)
                    .then_some(RuntimeViewModelInstanceReference {
                        view_model_index,
                        instance_index,
                        object: instance.object,
                    })
            })
    }

    pub fn view_model_default_instance(
        &self,
        view_model_index: usize,
    ) -> Option<RuntimeViewModelInstanceReference<'_>> {
        let view_model = self.view_model(view_model_index)?;
        let instance = view_model.instances.into_iter().next()?;
        Some(RuntimeViewModelInstanceReference {
            view_model_index,
            instance_index: 0,
            object: instance.object,
        })
    }

    pub fn data_context_view_model_property(
        &self,
        view_model_instance_id: usize,
        path: &[u32],
    ) -> Option<&RuntimeObject> {
        let view_model_instance = self.object(view_model_instance_id)?;
        self.data_context_view_model_property_for_instance(view_model_instance, path)
    }

    pub fn data_context_view_model_property_for_instance(
        &self,
        view_model_instance: &RuntimeObject,
        path: &[u32],
    ) -> Option<&RuntimeObject> {
        self.data_context_view_model_property_for_instance_chain(&[view_model_instance], path)
    }

    pub fn data_context_view_model_property_for_instance_chain(
        &self,
        view_model_instances: &[&RuntimeObject],
        path: &[u32],
    ) -> Option<&RuntimeObject> {
        let view_models = self.cpp_view_models();
        cpp_data_context_view_model_property(self, &view_models, view_model_instances, path)
    }

    pub fn data_context_relative_view_model_property(
        &self,
        view_model_instance_id: usize,
        path: &[u32],
    ) -> Option<&RuntimeObject> {
        let view_model_instance = self.object(view_model_instance_id)?;
        self.data_context_relative_view_model_property_for_instance(view_model_instance, path)
    }

    pub fn data_context_relative_view_model_property_for_instance(
        &self,
        view_model_instance: &RuntimeObject,
        path: &[u32],
    ) -> Option<&RuntimeObject> {
        self.data_context_relative_view_model_property_for_instance_chain(
            &[view_model_instance],
            path,
        )
    }

    pub fn data_context_relative_view_model_property_for_instance_chain(
        &self,
        view_model_instances: &[&RuntimeObject],
        path: &[u32],
    ) -> Option<&RuntimeObject> {
        let manifest = self.manifest()?;
        let view_models = self.cpp_view_models();
        cpp_data_context_relative_view_model_property(
            self,
            &view_models,
            &manifest,
            view_model_instances,
            path,
        )
    }

    pub fn data_context_view_model_instance(
        &self,
        view_model_instance_id: usize,
        path: &[u32],
    ) -> Option<RuntimeViewModelInstanceReference<'_>> {
        let view_model_instance = self.object(view_model_instance_id)?;
        self.data_context_view_model_instance_for_instance(view_model_instance, path)
    }

    pub fn data_context_view_model_instance_for_instance(
        &self,
        view_model_instance: &RuntimeObject,
        path: &[u32],
    ) -> Option<RuntimeViewModelInstanceReference<'_>> {
        self.data_context_view_model_instance_for_instance_chain(&[view_model_instance], path)
    }

    pub fn data_context_view_model_instance_for_instance_chain(
        &self,
        view_model_instances: &[&RuntimeObject],
        path: &[u32],
    ) -> Option<RuntimeViewModelInstanceReference<'_>> {
        let view_models = self.cpp_view_models();
        cpp_data_context_view_model_instance(self, &view_models, view_model_instances, path)
    }

    pub fn data_context_relative_view_model_instance(
        &self,
        view_model_instance_id: usize,
        path: &[u32],
    ) -> Option<RuntimeViewModelInstanceReference<'_>> {
        let view_model_instance = self.object(view_model_instance_id)?;
        self.data_context_relative_view_model_instance_for_instance(view_model_instance, path)
    }

    pub fn data_context_relative_view_model_instance_for_instance(
        &self,
        view_model_instance: &RuntimeObject,
        path: &[u32],
    ) -> Option<RuntimeViewModelInstanceReference<'_>> {
        self.data_context_relative_view_model_instance_for_instance_chain(
            &[view_model_instance],
            path,
        )
    }

    pub fn data_context_relative_view_model_instance_for_instance_chain(
        &self,
        view_model_instances: &[&RuntimeObject],
        path: &[u32],
    ) -> Option<RuntimeViewModelInstanceReference<'_>> {
        let manifest = self.manifest()?;
        let view_models = self.cpp_view_models();
        cpp_data_context_relative_view_model_instance(
            self,
            &view_models,
            &manifest,
            view_model_instances,
            path,
        )
    }

    pub fn referenced_view_model_instance_for_value(
        &self,
        value_id: usize,
    ) -> Option<RuntimeViewModelInstanceReference<'_>> {
        let value = self.object(value_id)?;
        self.referenced_view_model_instance_for_value_object(value)
    }

    pub fn referenced_view_model_instance_for_value_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<RuntimeViewModelInstanceReference<'_>> {
        if value.type_name != "ViewModelInstanceViewModel" {
            return None;
        }

        let value_id = usize::try_from(value.id).ok()?;
        if self.import_status(value_id) != Some(RuntimeImportStatus::Imported)
            || !self.cpp_has_latest_artboard_importer_before(value_id)
        {
            return None;
        }

        let view_models = self.cpp_view_models();
        let (owner_view_model_index, _) =
            cpp_owner_view_model_instance_indices(&view_models, value)?;
        let owner_view_model = view_models.get(owner_view_model_index)?;
        let property_index = usize::try_from(value.uint_property("viewModelPropertyId")?).ok()?;
        let property = owner_view_model.properties.get(property_index)?;
        if property.type_name != "ViewModelPropertyViewModel" {
            return None;
        }

        let referenced_view_model_index =
            usize::try_from(property.uint_property("viewModelReferenceId")?).ok()?;
        let referenced_view_model = view_models.get(referenced_view_model_index)?;
        let referenced_instance_index =
            usize::try_from(value.uint_property("propertyValue")?).ok()?;
        let referenced_instance = referenced_view_model
            .instances
            .get(referenced_instance_index)?;

        Some(RuntimeViewModelInstanceReference {
            view_model_index: referenced_view_model_index,
            instance_index: referenced_instance_index,
            object: referenced_instance.object,
        })
    }

    pub fn referenced_view_model_instance_for_list_item(
        &self,
        list_item_id: usize,
    ) -> Option<RuntimeViewModelInstanceReference<'_>> {
        let list_item = self.object(list_item_id)?;
        self.referenced_view_model_instance_for_list_item_object(list_item)
    }

    pub fn referenced_view_model_instance_for_list_item_object(
        &self,
        list_item: &RuntimeObject,
    ) -> Option<RuntimeViewModelInstanceReference<'_>> {
        if list_item.type_name != "ViewModelInstanceListItem" {
            return None;
        }

        let list_item_id = usize::try_from(list_item.id).ok()?;
        if self.import_status(list_item_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let view_models = self.cpp_view_models();
        let referenced_view_model_index =
            usize::try_from(list_item.uint_property("viewModelId")?).ok()?;
        let referenced_view_model = view_models.get(referenced_view_model_index)?;
        let referenced_instance_index =
            usize::try_from(list_item.uint_property("viewModelInstanceId")?).ok()?;
        let referenced_instance = referenced_view_model
            .instances
            .get(referenced_instance_index)?;

        Some(RuntimeViewModelInstanceReference {
            view_model_index: referenced_view_model_index,
            instance_index: referenced_instance_index,
            object: referenced_instance.object,
        })
    }

    pub fn view_model_instance_property_from_path(
        &self,
        instance_id: usize,
        path: &[u32],
    ) -> Option<&RuntimeObject> {
        let instance = self.object(instance_id)?;
        self.view_model_instance_property_from_path_for_object(instance, path)
    }

    pub fn view_model_instance_property_from_path_for_object(
        &self,
        instance: &RuntimeObject,
        path: &[u32],
    ) -> Option<&RuntimeObject> {
        if path.is_empty() {
            return None;
        }

        let view_models = self.cpp_view_models();
        let mut instance = cpp_view_model_instance_by_object(&view_models, instance)?;
        for (index, property_id) in path.iter().enumerate() {
            let value = cpp_view_model_instance_value_by_property_id(instance, *property_id)?;
            if index == path.len() - 1 {
                return Some(value);
            }
            if value.type_name != "ViewModelInstanceViewModel" {
                return None;
            }

            let reference = self.referenced_view_model_instance_for_value_object(value)?;
            instance = cpp_view_model_instance_by_object(&view_models, reference.object)?;
        }

        None
    }

    pub fn view_model_instance_value_named(
        &self,
        instance_id: usize,
        name: &str,
    ) -> Option<&RuntimeObject> {
        let instance = self.object(instance_id)?;
        self.view_model_instance_value_named_for_object(instance, name)
    }

    pub fn view_model_instance_value_named_for_object(
        &self,
        instance: &RuntimeObject,
        name: &str,
    ) -> Option<&RuntimeObject> {
        self.view_model_instance_value_named_bytes_for_object(instance, name.as_bytes())
    }

    pub fn view_model_instance_value_named_bytes_for_object(
        &self,
        instance: &RuntimeObject,
        name: &[u8],
    ) -> Option<&RuntimeObject> {
        let view_models = self.cpp_view_models();
        let instance = cpp_view_model_instance_by_object(&view_models, instance)?;
        let owner_view_model_index =
            usize::try_from(instance.object.uint_property("viewModelId")?).ok()?;
        let owner_view_model = view_models.get(owner_view_model_index)?;

        instance.values.iter().find_map(|value| {
            let property_index =
                usize::try_from(value.object.uint_property("viewModelPropertyId")?).ok()?;
            let property = owner_view_model.properties.get(property_index)?;
            (property.string_property_bytes("name").unwrap_or_default() == name)
                .then_some(value.object)
        })
    }

    pub fn view_model_instance_value_for_property_id(
        &self,
        instance_id: usize,
        property_id: u32,
    ) -> Option<&RuntimeObject> {
        let instance = self.object(instance_id)?;
        self.view_model_instance_value_for_property_id_object(instance, property_id)
    }

    pub fn view_model_instance_value_for_property_id_object(
        &self,
        instance: &RuntimeObject,
        property_id: u32,
    ) -> Option<&RuntimeObject> {
        let view_models = self.cpp_view_models();
        let instance = cpp_view_model_instance_by_object(&view_models, instance)?;
        cpp_view_model_instance_value_by_property_id(instance, property_id)
    }

    pub fn view_model_instance_value_for_symbol(
        &self,
        instance_id: usize,
        symbol_type: u8,
    ) -> Option<&RuntimeObject> {
        let instance = self.object(instance_id)?;
        self.view_model_instance_value_for_symbol_object(instance, symbol_type)
    }

    pub fn view_model_instance_value_for_symbol_object(
        &self,
        instance: &RuntimeObject,
        symbol_type: u8,
    ) -> Option<&RuntimeObject> {
        let view_models = self.cpp_view_models();
        let instance = cpp_view_model_instance_by_object(&view_models, instance)?;
        let owner_view_model_index =
            usize::try_from(instance.object.uint_property("viewModelId")?).ok()?;
        let owner_view_model = view_models.get(owner_view_model_index)?;

        let mut result = None;
        for value in &instance.values {
            let Some(symbol) = cpp_view_model_instance_value_symbol(owner_view_model, value.object)
            else {
                continue;
            };
            if symbol == symbol_type {
                result = Some(value.object);
            }
        }

        result
    }

    pub fn view_model_property_for_instance_value(
        &self,
        value_id: usize,
    ) -> Option<&RuntimeObject> {
        let value = self.object(value_id)?;
        self.view_model_property_for_instance_value_object(value)
    }

    pub fn view_model_property_for_instance_value_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let definition = definition_by_type_key(value.type_key)?;
        if !definition.is_a("ViewModelInstanceValue") {
            return None;
        }

        let value_id = usize::try_from(value.id).ok()?;
        if self.import_status(value_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let view_models = self.cpp_view_models();
        let (owner_view_model_index, _) =
            cpp_owner_view_model_instance_indices(&view_models, value)?;
        let owner_view_model = view_models.get(owner_view_model_index)?;
        let property_index = usize::try_from(value.uint_property("viewModelPropertyId")?).ok()?;
        owner_view_model.properties.get(property_index).copied()
    }

    pub fn view_model_instance_value_name(&self, value_id: usize) -> Option<&str> {
        let value = self.object(value_id)?;
        self.view_model_instance_value_name_for_object(value)
    }

    pub fn view_model_instance_value_name_for_object(&self, value: &RuntimeObject) -> Option<&str> {
        self.view_model_property_for_instance_value_object(value)?
            .string_property("name")
    }

    pub fn view_model_instance_value_data_type(&self, value_id: usize) -> Option<RuntimeDataType> {
        let value = self.object(value_id)?;
        self.view_model_instance_value_data_type_for_object(value)
    }

    pub fn view_model_instance_value_data_type_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<RuntimeDataType> {
        let definition = definition_by_type_key(value.type_key)?;
        if !definition.is_a("ViewModelInstanceValue") {
            return None;
        }

        let value_id = usize::try_from(value.id).ok()?;
        if self.import_status(value_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        Some(cpp_view_model_instance_value_data_type(value.type_name))
    }

    pub fn view_model_instance_number_value(&self, value_id: usize) -> Option<f32> {
        let value = self.object(value_id)?;
        self.view_model_instance_number_value_for_object(value)
    }

    pub fn view_model_instance_number_value_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<f32> {
        if self.view_model_instance_value_data_type_for_object(value)
            != Some(RuntimeDataType::Number)
        {
            return None;
        }

        value.double_property("propertyValue")
    }

    pub fn view_model_instance_string_value(&self, value_id: usize) -> Option<&str> {
        let value = self.object(value_id)?;
        self.view_model_instance_string_value_for_object(value)
    }

    pub fn view_model_instance_string_value_for_object<'a>(
        &self,
        value: &'a RuntimeObject,
    ) -> Option<&'a str> {
        if self.view_model_instance_value_data_type_for_object(value)
            != Some(RuntimeDataType::String)
        {
            return None;
        }

        value.string_property("propertyValue")
    }

    pub fn view_model_instance_string_value_bytes(&self, value_id: usize) -> Option<&[u8]> {
        let value = self.object(value_id)?;
        self.view_model_instance_string_value_bytes_for_object(value)
    }

    pub fn view_model_instance_string_value_bytes_for_object<'a>(
        &self,
        value: &'a RuntimeObject,
    ) -> Option<&'a [u8]> {
        if self.view_model_instance_value_data_type_for_object(value)
            != Some(RuntimeDataType::String)
        {
            return None;
        }

        value.string_property_bytes("propertyValue")
    }

    pub fn view_model_instance_boolean_value(&self, value_id: usize) -> Option<bool> {
        let value = self.object(value_id)?;
        self.view_model_instance_boolean_value_for_object(value)
    }

    pub fn view_model_instance_boolean_value_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<bool> {
        if self.view_model_instance_value_data_type_for_object(value)
            != Some(RuntimeDataType::Boolean)
        {
            return None;
        }

        value.bool_property("propertyValue")
    }

    pub fn view_model_instance_color_value(&self, value_id: usize) -> Option<u32> {
        let value = self.object(value_id)?;
        self.view_model_instance_color_value_for_object(value)
    }

    pub fn view_model_instance_color_value_for_object(&self, value: &RuntimeObject) -> Option<u32> {
        if self.view_model_instance_value_data_type_for_object(value)
            != Some(RuntimeDataType::Color)
        {
            return None;
        }

        value.color_property("propertyValue")
    }

    pub fn view_model_instance_list_size(&self, value_id: usize) -> Option<usize> {
        let value = self.object(value_id)?;
        self.view_model_instance_list_size_for_object(value)
    }

    pub fn view_model_instance_list_size_for_object(&self, value: &RuntimeObject) -> Option<usize> {
        if self.view_model_instance_value_data_type_for_object(value) != Some(RuntimeDataType::List)
        {
            return None;
        }

        let view_models = self.cpp_view_models();
        for view_model in &view_models {
            for instance in &view_model.instances {
                for instance_value in &instance.values {
                    if instance_value.object.id == value.id {
                        return Some(instance_value.list_items.len());
                    }
                }
            }
        }

        None
    }

    pub fn view_model_instance_trigger_count(&self, value_id: usize) -> Option<u64> {
        let value = self.object(value_id)?;
        self.view_model_instance_trigger_count_for_object(value)
    }

    pub fn view_model_instance_trigger_count_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<u64> {
        if self.view_model_instance_value_data_type_for_object(value)
            != Some(RuntimeDataType::Trigger)
        {
            return None;
        }

        value.uint_property("propertyValue")
    }

    pub fn view_model_instance_view_model_index(&self, value_id: usize) -> Option<u64> {
        let value = self.object(value_id)?;
        self.view_model_instance_view_model_index_for_object(value)
    }

    pub fn view_model_instance_view_model_index_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<u64> {
        if self.view_model_instance_value_data_type_for_object(value)
            != Some(RuntimeDataType::ViewModel)
        {
            return None;
        }

        value.uint_property("propertyValue")
    }

    pub fn view_model_instance_symbol_list_index_value(&self, value_id: usize) -> Option<u64> {
        let value = self.object(value_id)?;
        self.view_model_instance_symbol_list_index_value_for_object(value)
    }

    pub fn view_model_instance_symbol_list_index_value_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<u64> {
        if self.view_model_instance_value_data_type_for_object(value)
            != Some(RuntimeDataType::SymbolListIndex)
        {
            return None;
        }

        value.uint_property("propertyValue")
    }

    pub fn view_model_instance_asset_index(&self, value_id: usize) -> Option<u64> {
        let value = self.object(value_id)?;
        self.view_model_instance_asset_index_for_object(value)
    }

    pub fn view_model_instance_asset_index_for_object(&self, value: &RuntimeObject) -> Option<u64> {
        if self.view_model_instance_value_data_type_for_object(value)
            != Some(RuntimeDataType::AssetImage)
        {
            return None;
        }

        value.uint_property("propertyValue")
    }

    pub fn view_model_instance_font_asset_index(&self, value_id: usize) -> Option<u64> {
        let value = self.object(value_id)?;
        self.view_model_instance_font_asset_index_for_object(value)
    }

    pub fn view_model_instance_font_asset_index_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<u64> {
        if self.view_model_instance_value_data_type_for_object(value)
            != Some(RuntimeDataType::AssetFont)
        {
            return None;
        }

        value.uint_property("propertyValue")
    }

    pub fn view_model_instance_artboard_index(&self, value_id: usize) -> Option<u64> {
        let value = self.object(value_id)?;
        self.view_model_instance_artboard_index_for_object(value)
    }

    pub fn view_model_instance_artboard_index_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<u64> {
        if self.view_model_instance_value_data_type_for_object(value)
            != Some(RuntimeDataType::Artboard)
        {
            return None;
        }

        value.uint_property("propertyValue")
    }

    pub fn view_model_instance_source_data_value(
        &self,
        value_id: usize,
    ) -> Option<RuntimeDataValue<'_>> {
        let value = self.object(value_id)?;
        self.view_model_instance_source_data_value_for_object(value)
    }

    pub fn view_model_instance_source_data_value_for_object<'a>(
        &'a self,
        value: &'a RuntimeObject,
    ) -> Option<RuntimeDataValue<'a>> {
        match self.view_model_instance_value_data_type_for_object(value)? {
            RuntimeDataType::Number => Some(RuntimeDataValue::Number(
                value.double_property("propertyValue")?,
            )),
            RuntimeDataType::String => Some(RuntimeDataValue::String(
                value.string_property_bytes("propertyValue")?,
            )),
            RuntimeDataType::Boolean => Some(RuntimeDataValue::Boolean(
                value.bool_property("propertyValue")?,
            )),
            RuntimeDataType::Color => Some(RuntimeDataValue::Color(
                value.color_property("propertyValue")?,
            )),
            RuntimeDataType::EnumType => Some(RuntimeDataValue::Enum {
                value: value.uint_property("propertyValue")?,
                data_enum: self.data_enum_for_view_model_instance_enum_value_object(value),
            }),
            RuntimeDataType::Trigger => Some(RuntimeDataValue::Trigger(
                value.uint_property("propertyValue")?,
            )),
            RuntimeDataType::List => {
                let mut list_items = Vec::new();
                let view_models = self.cpp_view_models();
                for view_model in &view_models {
                    for instance in &view_model.instances {
                        for instance_value in &instance.values {
                            if instance_value.object.id == value.id {
                                list_items = instance_value.list_items.clone();
                            }
                        }
                    }
                }
                Some(RuntimeDataValue::List(list_items))
            }
            RuntimeDataType::SymbolListIndex => Some(RuntimeDataValue::SymbolListIndex(
                value.uint_property("propertyValue")?,
            )),
            RuntimeDataType::AssetImage => Some(RuntimeDataValue::AssetImage(
                value.uint_property("propertyValue")?,
            )),
            RuntimeDataType::AssetFont => Some(RuntimeDataValue::AssetFont(
                value.uint_property("propertyValue")?,
            )),
            RuntimeDataType::Artboard => Some(RuntimeDataValue::Artboard(
                value.uint_property("propertyValue")?,
            )),
            RuntimeDataType::ViewModel => Some(RuntimeDataValue::ViewModel(
                self.referenced_view_model_instance_for_value_object(value),
            )),
            RuntimeDataType::None
            | RuntimeDataType::Integer
            | RuntimeDataType::Input
            | RuntimeDataType::Any => Some(RuntimeDataValue::None),
        }
    }

    pub fn data_enum_for_view_model_instance_enum_value(
        &self,
        value_id: usize,
    ) -> Option<RuntimeDataEnum<'_>> {
        let value = self.object(value_id)?;
        self.data_enum_for_view_model_instance_enum_value_object(value)
    }

    pub fn data_enum_for_view_model_instance_enum_value_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<RuntimeDataEnum<'_>> {
        if value.type_name != "ViewModelInstanceEnum" {
            return None;
        }

        let property = self.view_model_property_for_instance_value_object(value)?;
        self.data_enum_for_view_model_property_object(property)
    }

    fn cpp_view_model_instance_enum_data(
        &self,
        value: &RuntimeObject,
    ) -> Option<RuntimeViewModelPropertyEnumData<'_>> {
        if value.type_name != "ViewModelInstanceEnum" {
            return None;
        }

        let property = self.view_model_property_for_instance_value_object(value)?;
        self.cpp_view_model_property_enum_data(property)
    }

    pub fn view_model_instance_enum_value_key(&self, value_id: usize) -> Option<&[u8]> {
        let value = self.object(value_id)?;
        self.view_model_instance_enum_value_key_for_object(value)
    }

    pub fn view_model_instance_enum_value_key_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<&[u8]> {
        let enum_data = self.cpp_view_model_instance_enum_data(value)?;
        let value_index = usize::try_from(value.uint_property("propertyValue")?).ok()?;
        Some(
            enum_data
                .values
                .get(value_index)
                .map(|enum_value| enum_value.string_property_bytes("key").unwrap_or_default())
                .unwrap_or_default(),
        )
    }

    pub fn view_model_instance_enum_value_key_string(&self, value_id: usize) -> Option<&str> {
        let value = self.object(value_id)?;
        self.view_model_instance_enum_value_key_string_for_object(value)
    }

    pub fn view_model_instance_enum_value_key_string_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<&str> {
        std::str::from_utf8(self.view_model_instance_enum_value_key_for_object(value)?).ok()
    }

    pub fn view_model_instance_enum_value_index(&self, value_id: usize) -> Option<usize> {
        let value = self.object(value_id)?;
        self.view_model_instance_enum_value_index_for_object(value)
    }

    pub fn view_model_instance_enum_value_index_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<usize> {
        let enum_data = self.cpp_view_model_instance_enum_data(value)?;
        let value_index = usize::try_from(value.uint_property("propertyValue")?).ok()?;
        Some(
            (value_index < enum_data.values.len())
                .then_some(value_index)
                .unwrap_or(0),
        )
    }

    pub fn view_model_instance_enum_value_keys(&self, value_id: usize) -> Option<Vec<&[u8]>> {
        let value = self.object(value_id)?;
        self.view_model_instance_enum_value_keys_for_object(value)
    }

    pub fn view_model_instance_enum_value_keys_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<Vec<&[u8]>> {
        let enum_data = self.cpp_view_model_instance_enum_data(value)?;
        Some(
            enum_data
                .values
                .into_iter()
                .map(|enum_value| enum_value.string_property_bytes("key").unwrap_or_default())
                .collect(),
        )
    }

    pub fn view_model_instance_enum_type(&self, value_id: usize) -> Option<&[u8]> {
        let value = self.object(value_id)?;
        self.view_model_instance_enum_type_for_object(value)
    }

    pub fn view_model_instance_enum_type_for_object(&self, value: &RuntimeObject) -> Option<&[u8]> {
        let enum_data = self.cpp_view_model_instance_enum_data(value)?;
        Some(enum_data.name)
    }

    pub fn view_model_instance_asset_file_assets(&self, value_id: usize) -> Vec<&RuntimeObject> {
        let Some(value) = self.object(value_id) else {
            return Vec::new();
        };

        self.view_model_instance_asset_file_assets_for_object(value)
    }

    pub fn view_model_instance_asset_file_assets_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Vec<&RuntimeObject> {
        self.cpp_view_model_instance_asset_file_assets(value)
    }

    pub fn resolved_file_asset_for_view_model_instance_asset(
        &self,
        value_id: usize,
    ) -> Option<&RuntimeObject> {
        let value = self.object(value_id)?;
        self.resolved_file_asset_for_view_model_instance_asset_object(value)
    }

    pub fn resolved_file_asset_for_view_model_instance_asset_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let asset_index = usize::try_from(value.uint_property("propertyValue")?).ok()?;
        self.cpp_view_model_instance_asset_file_assets(value)
            .into_iter()
            .nth(asset_index)
    }

    pub fn file_assets(&self) -> Vec<&RuntimeObject> {
        self.cpp_file_assets().collect()
    }

    pub fn file_asset(&self, index: usize) -> Option<&RuntimeObject> {
        self.cpp_file_assets().nth(index)
    }

    /// Dense FileAsset catalog with `FileAssetContents` associated by the
    /// scripting-enabled importer stack rather than by record adjacency.
    ///
    /// This is the extraction profile used by script-executing hosts. It scans
    /// the object stream once, tracks the latest imported FileAsset that
    /// creates an importer in a WITH_RIVE_SCRIPTING build, and attaches only
    /// imported `FileAssetContents` records to that entry.
    pub fn scripting_file_assets_with_contents(&self) -> Vec<RuntimeFileAssetContents<'_>> {
        let assets = self.file_assets();
        let ordinals_by_global = assets
            .iter()
            .enumerate()
            .map(|(ordinal, asset)| (asset.id, ordinal))
            .collect::<BTreeMap<_, _>>();
        let mut contents = vec![None; assets.len()];
        let mut latest_ordinal = None;

        for (index, object) in self.objects.iter().enumerate() {
            if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }
            let Some(object) = object.as_ref() else {
                continue;
            };
            if file_asset_creates_importer(object.type_name, true) {
                // Importer-owning FileAsset kinds such as ManifestAsset are
                // intentionally absent from the public dense catalog. They
                // still delimit contents ownership, so reset the candidate
                // even when there is no ordinal to publish.
                latest_ordinal = ordinals_by_global.get(&object.id).copied();
                continue;
            }
            if object.type_name == "FileAssetContents"
                && let Some(ordinal) = latest_ordinal
                && let Some(slot) = contents.get_mut(ordinal)
            {
                *slot = object.bytes_property("bytes");
            }
        }

        assets
            .into_iter()
            .enumerate()
            .map(|(ordinal, asset)| RuntimeFileAssetContents {
                ordinal,
                asset,
                contents: contents.get(ordinal).copied().flatten(),
            })
            .collect()
    }

    pub fn resolved_file_asset_for_object(&self, object_id: usize) -> Option<&RuntimeObject> {
        let object = self.object(object_id)?;
        self.resolved_file_asset_for_referencer(object)
    }

    pub fn resolved_file_asset_for_referencer(
        &self,
        referencer: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let object_id = usize::try_from(referencer.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let asset_index = usize::try_from(cpp_file_asset_referencer_index(referencer)?).ok()?;
        let asset = self.file_asset(asset_index)?;
        if cpp_file_asset_matches_referencer(referencer, asset) {
            Some(asset)
        } else {
            None
        }
    }

    pub fn manifest(&self) -> Option<RuntimeManifest> {
        self.manifest_with_script_assets(false)
    }

    pub fn scripting_manifest(&self) -> Option<RuntimeManifest> {
        self.manifest_with_script_assets(true)
    }

    fn manifest_with_script_assets(
        &self,
        script_assets_create_importers: bool,
    ) -> Option<RuntimeManifest> {
        let mut latest_file_asset = None;
        let mut manifest = None;

        for (index, object) in self.objects.iter().enumerate() {
            if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }

            let Some(object) = object.as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if file_asset_creates_importer(definition.name, script_assets_create_importers) {
                latest_file_asset = Some(object);
                if definition.name == "ManifestAsset" {
                    manifest = Some(RuntimeManifest::default());
                }
            }

            if definition.name == "FileAssetContents"
                && latest_file_asset.is_some_and(|asset| asset.type_name == "ManifestAsset")
            {
                manifest = Some(parse_cpp_manifest_asset(
                    object.bytes_property("bytes").unwrap_or(&[]),
                ));
            }
        }

        manifest
    }

    fn cpp_file_assets(&self) -> impl Iterator<Item = &RuntimeObject> {
        self.objects
            .iter()
            .enumerate()
            .filter_map(|(index, object)| {
                if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                cpp_file_assets_contains(object).then_some(object)
            })
    }

    fn cpp_view_model_instance_asset_file_assets<'a>(
        &'a self,
        value: &RuntimeObject,
    ) -> Vec<&'a RuntimeObject> {
        let Some(definition) = definition_by_type_key(value.type_key) else {
            return Vec::new();
        };
        if !definition.is_a("ViewModelInstanceAsset") {
            return Vec::new();
        }

        let Ok(value_index) = usize::try_from(value.id) else {
            return Vec::new();
        };
        if self.import_status(value_index) != Some(RuntimeImportStatus::Imported) {
            return Vec::new();
        }

        self.objects
            .iter()
            .take(value_index)
            .enumerate()
            .filter_map(|(index, object)| {
                if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                cpp_file_assets_contains(object).then_some(object)
            })
            .collect()
    }

    fn cpp_artboards(&self) -> impl Iterator<Item = &RuntimeObject> {
        self.objects
            .iter()
            .enumerate()
            .filter_map(|(index, object)| {
                if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                (object.type_name == "Artboard").then_some(object)
            })
    }

    fn cpp_has_latest_artboard_importer_before(&self, object_index: usize) -> bool {
        self.objects
            .iter()
            .take(object_index)
            .enumerate()
            .any(|(index, object)| {
                self.import_status(index) == Some(RuntimeImportStatus::Imported)
                    && object
                        .as_ref()
                        .is_some_and(|object| object.type_name == "Artboard")
            })
    }

    fn cpp_artboard_objects_named(
        &self,
        artboard_index: usize,
        type_name: &'static str,
    ) -> Vec<&RuntimeObject> {
        let Some((start, end)) = self.cpp_artboard_range(artboard_index) else {
            return Vec::new();
        };

        self.objects[start..end]
            .iter()
            .enumerate()
            .filter_map(|(offset, object)| {
                let file_index = start + offset;
                if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                (object.type_name == type_name).then_some(object)
            })
            .collect()
    }

    fn cpp_artboard_range(&self, artboard_index: usize) -> Option<(usize, usize)> {
        let artboard = self.artboard(artboard_index)?;
        let start = usize::try_from(artboard.id).ok()?;
        let end = self
            .objects
            .iter()
            .enumerate()
            .skip(start + 1)
            .find_map(|(index, object)| {
                matches!(object, Some(object) if object.type_name == "Artboard").then_some(index)
            })
            .unwrap_or(self.objects.len());

        Some((start, end))
    }

    fn cpp_artboard_index(&self, artboard_index: usize) -> Option<RuntimeArtboardIndex<'_>> {
        let range = self.cpp_artboard_range(artboard_index)?;
        Some(RuntimeArtboardIndex::new(
            &self.objects,
            &self.import_statuses,
            range,
        ))
    }

    fn cpp_artboard_local_context_for_object(
        &self,
        object: &RuntimeObject,
    ) -> Option<(usize, (usize, usize), Vec<Option<usize>>, usize)> {
        let file_index = usize::try_from(object.id).ok()?;
        for (artboard_index, range) in runtime_artboard_ranges(&self.objects)
            .into_iter()
            .enumerate()
        {
            if file_index < range.0 || file_index >= range.1 {
                continue;
            }

            let mut slots =
                runtime_artboard_local_slots(&self.objects, &self.import_statuses, range);
            validate_cpp_artboard_local_slots(&mut slots, &self.objects);
            let local_index = slots.iter().position(|slot| *slot == Some(file_index))?;
            return Some((artboard_index, range, slots, local_index));
        }

        None
    }

    fn resolved_axis_animation_for_joystick_object(
        &self,
        joystick: &RuntimeObject,
        property_name: &str,
    ) -> Option<&RuntimeObject> {
        if joystick.type_name != "Joystick" {
            return None;
        }

        let joystick_id = usize::try_from(joystick.id).ok()?;
        if self.import_status(joystick_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let (artboard_index, _, _, _) = self.cpp_artboard_local_context_for_object(joystick)?;
        let animation_index = usize::try_from(joystick.uint_property(property_name)?).ok()?;
        self.artboard_animation(artboard_index, animation_index)
    }

    fn cpp_artboard_linear_animations(
        &self,
        artboard_index: usize,
    ) -> Vec<RuntimeLinearAnimation<'_>> {
        let Some(range) = self.cpp_artboard_range(artboard_index) else {
            return Vec::new();
        };
        let mut local_slots =
            runtime_artboard_local_slots(&self.objects, &self.import_statuses, range);
        validate_cpp_artboard_local_slots(&mut local_slots, &self.objects);

        let mut animations = Vec::<RuntimeLinearAnimation<'_>>::new();
        let mut current_animation = None;
        let mut current_keyed_object = None;
        let mut current_keyed_property = None;

        for (offset, object) in self.objects[range.0..range.1].iter().enumerate() {
            let file_index = range.0 + offset;
            let Some(object) = object.as_ref() else {
                continue;
            };
            if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if definition.name == "LinearAnimation" {
                animations.push(RuntimeLinearAnimation {
                    object,
                    keyed_objects: Vec::new(),
                });
                current_animation = Some(animations.len() - 1);
                current_keyed_object = None;
                current_keyed_property = None;
                continue;
            }

            let Some(animation_index) = current_animation else {
                continue;
            };

            if definition.name == "KeyedObject" {
                if cpp_keyed_object_target(object, &local_slots, &self.objects).is_none() {
                    current_keyed_object = None;
                    current_keyed_property = None;
                    continue;
                }

                animations[animation_index]
                    .keyed_objects
                    .push(RuntimeKeyedObject {
                        object,
                        keyed_properties: Vec::new(),
                    });
                current_keyed_object = Some(animations[animation_index].keyed_objects.len() - 1);
                current_keyed_property = None;
                continue;
            }

            if definition.name == "KeyedProperty" {
                let Some(keyed_object_index) = current_keyed_object else {
                    continue;
                };
                if !cpp_keyed_object_supports_property(
                    animations[animation_index].keyed_objects[keyed_object_index].object,
                    object,
                    &local_slots,
                    &self.objects,
                ) {
                    current_keyed_property = None;
                    continue;
                }

                animations[animation_index].keyed_objects[keyed_object_index]
                    .keyed_properties
                    .push(RuntimeKeyedProperty {
                        object,
                        first_key_frame: None,
                    });
                current_keyed_property = Some((
                    keyed_object_index,
                    animations[animation_index].keyed_objects[keyed_object_index]
                        .keyed_properties
                        .len()
                        - 1,
                ));
                continue;
            }

            if definition.is_a("KeyFrame") {
                let Some((keyed_object_index, keyed_property_index)) = current_keyed_property
                else {
                    continue;
                };
                let first_key_frame = &mut animations[animation_index].keyed_objects
                    [keyed_object_index]
                    .keyed_properties[keyed_property_index]
                    .first_key_frame;
                if first_key_frame.is_none() {
                    *first_key_frame = Some(object);
                }
            }
        }

        animations
    }

    fn cpp_artboard_state_machine_graphs(
        &self,
        artboard_index: usize,
    ) -> Vec<RuntimeStateMachine<'_>> {
        let Some(range) = self.cpp_artboard_range(artboard_index) else {
            return Vec::new();
        };
        let data_bind_targets = self.cpp_data_bind_targets();
        let artboard_animations =
            self.cpp_artboard_objects_named(artboard_index, "LinearAnimation");
        let mut artboard_local_slots =
            runtime_artboard_local_slots(&self.objects, &self.import_statuses, range);
        validate_cpp_artboard_local_slots(&mut artboard_local_slots, &self.objects);

        let mut state_machines = Vec::<RuntimeStateMachine<'_>>::new();
        let mut current_state_machine: Option<usize> = None;
        let mut current_layer: Option<usize> = None;
        let mut current_listener: Option<usize> = None;
        let mut current_layer_component: Option<RuntimeStateMachineLayerComponentOwner> = None;
        let mut current_state_machine_scripted_object: Option<
            RuntimeStateMachineScriptedObjectOwner,
        > = None;

        for (offset, object) in self.objects[range.0..range.1].iter().enumerate() {
            let file_index = range.0 + offset;
            let Some(object) = object.as_ref() else {
                if self.import_status(file_index) == Some(RuntimeImportStatus::NullObject)
                    && let Some(state_machine_index) = current_state_machine
                    && let Some(layer_index) = current_layer
                {
                    state_machines[state_machine_index].layers[layer_index]
                        .states
                        .push(RuntimeLayerState {
                            object: None,
                            animation: None,
                            blend_animations: Vec::new(),
                            fire_actions: Vec::new(),
                            listener_actions: Vec::new(),
                            transitions: Vec::new(),
                        });
                    state_machines[state_machine_index].layers[layer_index].state_count += 1;
                }
                continue;
            };
            if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if definition.name == "StateMachine" {
                state_machines.push(RuntimeStateMachine {
                    object,
                    layers: Vec::new(),
                    inputs: Vec::new(),
                    listeners: Vec::new(),
                    data_binds: Vec::new(),
                    scripted_objects: Vec::new(),
                });
                current_state_machine = Some(state_machines.len() - 1);
                current_layer = None;
                current_listener = None;
                current_state_machine_scripted_object = None;
                continue;
            }

            let Some(state_machine_index) = current_state_machine else {
                continue;
            };

            if definition_adds_cpp_state_machine_scripted_object(definition) {
                state_machines[state_machine_index]
                    .scripted_objects
                    .push(RuntimeScriptedObject {
                        object,
                        inputs: Vec::new(),
                    });
                current_state_machine_scripted_object =
                    Some(RuntimeStateMachineScriptedObjectOwner {
                        state_machine_index,
                        scripted_object_index: state_machines[state_machine_index]
                            .scripted_objects
                            .len()
                            - 1,
                    });
            } else if definition_is_cpp_scripted_object(definition) {
                current_state_machine_scripted_object = None;
            }

            if definition.name.starts_with("ScriptInput") {
                if let Some(owner) = current_state_machine_scripted_object {
                    state_machines[owner.state_machine_index].scripted_objects
                        [owner.scripted_object_index]
                        .inputs
                        .push(object);
                }
                continue;
            }

            if definition.name == "StateMachineLayer" {
                state_machines[state_machine_index]
                    .layers
                    .push(RuntimeStateMachineLayer {
                        object,
                        state_count: 0,
                        states: Vec::new(),
                    });
                current_layer = Some(state_machines[state_machine_index].layers.len() - 1);
                current_listener = None;
                continue;
            }

            if definition.is_a("LayerState") {
                if let Some(layer_index) = current_layer {
                    state_machines[state_machine_index].layers[layer_index]
                        .states
                        .push(RuntimeLayerState {
                            object: Some(object),
                            animation: cpp_resolved_animation_state_animation(
                                object,
                                &artboard_animations,
                            ),
                            blend_animations: Vec::new(),
                            fire_actions: Vec::new(),
                            listener_actions: Vec::new(),
                            transitions: Vec::new(),
                        });
                    current_layer_component = Some(RuntimeStateMachineLayerComponentOwner::State {
                        state_machine_index,
                        layer_index,
                        state_index: state_machines[state_machine_index].layers[layer_index]
                            .states
                            .len()
                            - 1,
                    });
                    state_machines[state_machine_index].layers[layer_index].state_count += 1;
                }
                current_listener = None;
                continue;
            }

            if definition.is_a("BlendAnimation") {
                if let Some(layer_index) = current_layer
                    && let Some(state_index) = state_machines[state_machine_index].layers
                        [layer_index]
                        .states
                        .iter()
                        .rposition(|state| {
                            state.object.is_some_and(|object| {
                                definition_by_type_key(object.type_key)
                                    .is_some_and(|definition| definition.is_a("BlendState"))
                            })
                        })
                {
                    let animation_index =
                        usize::try_from(object.uint_property("animationId").unwrap_or(u64::MAX))
                            .ok()
                            .filter(|index| *index < artboard_animations.len());
                    let animation = animation_index
                        .and_then(|index| artboard_animations.get(index))
                        .copied();
                    state_machines[state_machine_index].layers[layer_index].states[state_index]
                        .blend_animations
                        .push(RuntimeBlendAnimation {
                            object,
                            animation_index,
                            animation,
                        });
                }
                current_listener = None;
                continue;
            }

            if definition.is_a("StateTransition") {
                if let Some(layer_index) = current_layer
                    && let Some(state_index) = state_machines[state_machine_index].layers
                        [layer_index]
                        .states
                        .iter()
                        .rposition(|state| state.object.is_some())
                {
                    let interpolator = cpp_resolved_state_transition_interpolator(
                        object,
                        range,
                        &self.objects,
                        &self.import_statuses,
                    );

                    state_machines[state_machine_index].layers[layer_index].states[state_index]
                        .transitions
                        .push(RuntimeStateTransition {
                            object,
                            state_to_index: None,
                            state_to: None,
                            interpolator,
                            exit_blend_animation_index: None,
                            exit_blend_animation: None,
                            exit_animation_index: None,
                            exit_animation: None,
                            fire_actions: Vec::new(),
                            listener_actions: Vec::new(),
                            conditions: Vec::new(),
                        });
                    current_layer_component =
                        Some(RuntimeStateMachineLayerComponentOwner::Transition {
                            state_machine_index,
                            layer_index,
                            state_index,
                            transition_index: state_machines[state_machine_index].layers
                                [layer_index]
                                .states[state_index]
                                .transitions
                                .len()
                                - 1,
                        });
                }
                current_listener = None;
                continue;
            }

            if definition.is_a("StateMachineFireAction") {
                if let Some(owner) = current_layer_component {
                    match owner {
                        RuntimeStateMachineLayerComponentOwner::State {
                            state_machine_index,
                            layer_index,
                            state_index,
                        } => {
                            state_machines[state_machine_index].layers[layer_index].states
                                [state_index]
                                .fire_actions
                                .push(cpp_runtime_state_machine_fire_action(
                                    object,
                                    &artboard_local_slots,
                                    &self.objects,
                                ));
                        }
                        RuntimeStateMachineLayerComponentOwner::Transition {
                            state_machine_index,
                            layer_index,
                            state_index,
                            transition_index,
                        } => {
                            state_machines[state_machine_index].layers[layer_index].states
                                [state_index]
                                .transitions[transition_index]
                                .fire_actions
                                .push(cpp_runtime_state_machine_fire_action(
                                    object,
                                    &artboard_local_slots,
                                    &self.objects,
                                ));
                        }
                    }
                }
                continue;
            }

            if definition.is_a("TransitionCondition") {
                if let Some(layer_index) = current_layer
                    && let Some(state_index) = state_machines[state_machine_index].layers
                        [layer_index]
                        .states
                        .iter()
                        .rposition(|state| state.object.is_some())
                    && let Some(transition) =
                        state_machines[state_machine_index].layers[layer_index].states[state_index]
                            .transitions
                            .last_mut()
                {
                    transition.conditions.push(object);
                }
                continue;
            }

            if definition.is_a("StateMachineInput") {
                state_machines[state_machine_index].inputs.push(object);
                current_layer = None;
                current_listener = None;
                continue;
            }

            if definition.is_a("StateMachineListener") {
                state_machines[state_machine_index]
                    .listeners
                    .push(RuntimeStateMachineListener {
                        object,
                        actions: Vec::new(),
                        listener_input_types: Vec::new(),
                    });
                current_layer = None;
                current_listener = Some(state_machines[state_machine_index].listeners.len() - 1);
                continue;
            }

            if definition.is_a("ListenerAction") {
                if listener_action_parent_kind_is_listener(object) {
                    if let Some(listener_index) = current_listener {
                        state_machines[state_machine_index].listeners[listener_index]
                            .actions
                            .push(cpp_runtime_listener_action(
                                object,
                                &artboard_local_slots,
                                &self.objects,
                            ));
                    }
                } else if let Some(owner) = current_layer_component {
                    match owner {
                        RuntimeStateMachineLayerComponentOwner::State {
                            state_machine_index,
                            layer_index,
                            state_index,
                        } => {
                            state_machines[state_machine_index].layers[layer_index].states
                                [state_index]
                                .listener_actions
                                .push(cpp_runtime_listener_action(
                                    object,
                                    &artboard_local_slots,
                                    &self.objects,
                                ));
                        }
                        RuntimeStateMachineLayerComponentOwner::Transition {
                            state_machine_index,
                            layer_index,
                            state_index,
                            transition_index,
                        } => {
                            state_machines[state_machine_index].layers[layer_index].states
                                [state_index]
                                .transitions[transition_index]
                                .listener_actions
                                .push(cpp_runtime_listener_action(
                                    object,
                                    &artboard_local_slots,
                                    &self.objects,
                                ));
                        }
                    }
                }
                continue;
            }

            if definition.is_a("ListenerInputType") {
                if let Some(listener_index) = current_listener {
                    state_machines[state_machine_index].listeners[listener_index]
                        .listener_input_types
                        .push(object);
                }
                continue;
            }

            if definition.is_a("DataBind")
                && data_bind_target_is_cpp_state_machine_owned(
                    data_bind_targets[file_index].map(|target| target.object),
                )
            {
                state_machines[state_machine_index].data_binds.push(object);
            }
        }

        resolve_runtime_state_machine_transition_targets(&mut state_machines);
        state_machines
    }

    fn cpp_artboard_data_binds(&self, artboard_index: usize) -> Vec<RuntimeDataBind<'_>> {
        if self.artboard(artboard_index).is_none() {
            return Vec::new();
        }

        let data_bind_targets = self.cpp_data_bind_targets();
        let artboard_indices = self.cpp_artboard_indices_by_file_index();
        let artboard_local_owners = self.cpp_artboard_local_owners();
        let mut latest_artboard_index = None;
        let mut data_binds = Vec::new();

        for (file_index, object) in self.objects.iter().enumerate() {
            if let Some(index) = artboard_indices[file_index] {
                latest_artboard_index = Some(index);
            }

            if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }

            let Some(object) = object.as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };
            if !definition.is_a("DataBind") {
                continue;
            }

            let target = data_bind_targets[file_index];
            let owner =
                cpp_data_bind_artboard_owner(target, latest_artboard_index, &artboard_local_owners);
            if owner != Some(artboard_index) {
                continue;
            }

            let target_local_id = target
                .and_then(|target| artboard_local_owners[target.file_index])
                .and_then(|(owner, local_id)| (owner == artboard_index).then_some(local_id));

            data_binds.push(RuntimeDataBind {
                object,
                converter: self.resolved_data_converter_for_data_bind_object(object),
                target: target.map(|target| target.object),
                target_local_id,
            });
        }

        data_binds
    }

    fn cpp_artboard_skins(&self, artboard_index: usize) -> Vec<RuntimeSkin<'_>> {
        let Some(range) = self.cpp_artboard_range(artboard_index) else {
            return Vec::new();
        };
        let mut local_slots =
            runtime_artboard_local_slots(&self.objects, &self.import_statuses, range);
        validate_cpp_artboard_local_slots(&mut local_slots, &self.objects);

        local_slots
            .iter()
            .enumerate()
            .filter_map(|(local_id, slot)| {
                let object = slot.and_then(|file_index| self.objects[file_index].as_ref())?;
                if object.type_name != "Skin" {
                    return None;
                }

                let (skinnable_local_id, skinnable) = local_object_reference_with_local_index(
                    &local_slots,
                    &self.objects,
                    object.uint_property("parentId"),
                )
                .filter(|(_, parent)| runtime_object_is_cpp_skinnable(parent))
                .map(|(local, parent)| (Some(local), Some(parent)))
                .unwrap_or((None, None));

                Some(RuntimeSkin {
                    local_id,
                    object,
                    skinnable_local_id,
                    skinnable,
                    tendons: cpp_skin_tendons(local_id, &local_slots, &self.objects),
                })
            })
            .collect()
    }

    fn cpp_artboard_meshes(&self, artboard_index: usize) -> Vec<RuntimeMesh<'_>> {
        self.cpp_artboard_index(artboard_index)
            .map(|index| index.meshes())
            .unwrap_or_default()
    }

    fn cpp_artboard_paths(&self, artboard_index: usize) -> Vec<RuntimePath<'_>> {
        self.cpp_artboard_index(artboard_index)
            .map(|index| index.paths())
            .unwrap_or_default()
    }

    fn cpp_artboard_shapes(&self, artboard_index: usize) -> Vec<RuntimeShape<'_>> {
        self.cpp_artboard_index(artboard_index)
            .map(|index| index.shapes())
            .unwrap_or_default()
    }

    fn cpp_artboard_shape_paint_containers(
        &self,
        artboard_index: usize,
    ) -> Vec<RuntimeShapePaintContainer<'_>> {
        self.cpp_artboard_index(artboard_index)
            .map(|index| index.shape_paint_containers())
            .unwrap_or_default()
    }

    fn cpp_artboard_n_slicer_details(
        &self,
        artboard_index: usize,
    ) -> Vec<RuntimeNSlicerDetails<'_>> {
        self.cpp_artboard_index(artboard_index)
            .map(|index| index.n_slicer_details())
            .unwrap_or_default()
    }

    fn cpp_data_bind_targets(&self) -> Vec<Option<CppDataBindTarget<'_>>> {
        let mut targets = vec![None; self.objects.len()];
        let mut last_bindable_object = None;

        for (file_index, object) in self.objects.iter().enumerate() {
            let Some(object) = object.as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if definition.is_a("DataBind") {
                targets[file_index] = last_bindable_object;
                continue;
            }

            last_bindable_object = Some(CppDataBindTarget { file_index, object });
            if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                last_bindable_object = None;
            }
        }

        targets
    }

    fn cpp_claimed_data_bind_path_for(&self, referencer_id: usize) -> Option<&RuntimeObject> {
        let mut latest_unclaimed_path = None;

        for (file_index, object) in self.objects.iter().enumerate() {
            let Some(object) = object.as_ref() else {
                continue;
            };

            if object.type_name == "DataBindPath" {
                if self.import_status(file_index) == Some(RuntimeImportStatus::Imported) {
                    latest_unclaimed_path = Some(file_index);
                }
                continue;
            }

            if cpp_claims_latest_data_bind_path(object) {
                let claimed_path = latest_unclaimed_path.take();
                if file_index == referencer_id {
                    return claimed_path.and_then(|path_index| self.object(path_index));
                }
            }
        }

        None
    }

    fn cpp_resolved_data_bind_path_ids(
        &self,
        path_object: &RuntimeObject,
        path_ids: &[u32],
    ) -> Vec<u32> {
        if path_object.type_name != "DataBindPath" || path_ids.len() != 1 {
            return path_ids.to_vec();
        }

        let Some(manifest) = self.manifest() else {
            return path_ids.to_vec();
        };

        manifest
            .resolve_path(path_ids[0])
            .map_or_else(Vec::new, <[u32]>::to_vec)
    }

    fn cpp_artboard_indices_by_file_index(&self) -> Vec<Option<usize>> {
        let mut artboard_indices = vec![None; self.objects.len()];
        for (artboard_index, artboard) in self.cpp_artboards().enumerate() {
            if let Ok(file_index) = usize::try_from(artboard.id) {
                if let Some(slot) = artboard_indices.get_mut(file_index) {
                    *slot = Some(artboard_index);
                }
            }
        }
        artboard_indices
    }

    fn cpp_artboard_local_owners(&self) -> Vec<Option<(usize, usize)>> {
        let mut owners = vec![None; self.objects.len()];
        for artboard_index in 0..self.artboards().len() {
            let Some(range) = self.cpp_artboard_range(artboard_index) else {
                continue;
            };
            let mut local_slots =
                runtime_artboard_local_slots(&self.objects, &self.import_statuses, range);
            validate_cpp_artboard_local_slots(&mut local_slots, &self.objects);
            for (local_id, file_index) in local_slots.into_iter().enumerate() {
                let Some(file_index) = file_index else {
                    continue;
                };
                if let Some(owner) = owners.get_mut(file_index) {
                    *owner = Some((artboard_index, local_id));
                }
            }
        }
        owners
    }

    fn cpp_data_enums(&self) -> Vec<RuntimeDataEnum<'_>> {
        let mut data_enums = Vec::<RuntimeDataEnum<'_>>::new();
        let mut latest_custom_enum = None;

        for (index, object) in self.objects.iter().enumerate() {
            if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }

            let Some(object) = object.as_ref() else {
                continue;
            };

            match object.type_name {
                "DataEnum" | "DataEnumCustom" => {
                    data_enums.push(RuntimeDataEnum {
                        object,
                        values: Vec::new(),
                    });
                    if object.type_name == "DataEnumCustom" {
                        latest_custom_enum = Some(data_enums.len() - 1);
                    }
                }
                "DataEnumValue" => {
                    if let Some(enum_index) = latest_custom_enum {
                        data_enums[enum_index].values.push(object);
                    }
                }
                _ => {}
            }
        }

        data_enums
    }

    fn cpp_data_converters(&self) -> impl Iterator<Item = &RuntimeObject> {
        self.objects
            .iter()
            .enumerate()
            .filter_map(|(index, object)| {
                if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                definition_by_type_key(object.type_key)
                    .is_some_and(|definition| definition.is_a("DataConverter"))
                    .then_some(object)
            })
    }

    fn validate_data_converter_interpolator<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
    ) -> Option<&'a RuntimeObject> {
        self.validate_data_converter(data_converter)?;
        (data_converter.type_name == "DataConverterInterpolator").then_some(data_converter)
    }

    fn validate_data_converter<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
    ) -> Option<&'a RuntimeObject> {
        let object_id = usize::try_from(data_converter.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }
        let definition = definition_by_type_key(data_converter.type_key)?;
        definition.is_a("DataConverter").then_some(data_converter)
    }

    fn validate_data_bind<'a>(&'a self, data_bind: &'a RuntimeObject) -> Option<&'a RuntimeObject> {
        let object_id = usize::try_from(data_bind.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }
        let definition = definition_by_type_key(data_bind.type_key)?;
        definition.is_a("DataBind").then_some(data_bind)
    }

    fn validate_transition_view_model_condition<'a>(
        &'a self,
        condition: &'a RuntimeObject,
    ) -> Option<&'a RuntimeObject> {
        let object_id = usize::try_from(condition.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }
        let definition = definition_by_type_key(condition.type_key)?;
        definition
            .is_a("TransitionViewModelCondition")
            .then_some(condition)
    }

    fn cpp_data_bind_target_for_object<'a>(
        &'a self,
        data_bind: &RuntimeObject,
    ) -> Option<&'a RuntimeObject> {
        let object_id = usize::try_from(data_bind.id).ok()?;
        self.cpp_data_bind_targets()
            .get(object_id)
            .and_then(|target| target.map(|target| target.object))
    }

    fn cpp_latest_bindable_property_for_object<'a>(
        &'a self,
        object: &RuntimeObject,
    ) -> Option<&'a RuntimeObject> {
        let object_id = usize::try_from(object.id).ok()?;
        let mut latest_bindable_property = None;
        for candidate in self.objects.iter().take(object_id).flatten() {
            if self.import_status(usize::try_from(candidate.id).ok()?)
                != Some(RuntimeImportStatus::Imported)
            {
                continue;
            }
            let Some(definition) = definition_by_type_key(candidate.type_key) else {
                continue;
            };
            if definition.is_a("BindableProperty") {
                latest_bindable_property = Some(candidate);
            }
        }
        latest_bindable_property
    }

    fn cpp_transition_view_model_condition_comparators<'a>(
        &'a self,
        condition: &RuntimeObject,
    ) -> RuntimeTransitionViewModelConditionComparators<'a> {
        let Some(object_id) = usize::try_from(condition.id).ok() else {
            return RuntimeTransitionViewModelConditionComparators::default();
        };
        let mut comparators = RuntimeTransitionViewModelConditionComparators::default();
        for candidate in self.objects.iter().skip(object_id + 1).flatten() {
            let Some(candidate_id) = usize::try_from(candidate.id).ok() else {
                continue;
            };
            if self.import_status(candidate_id) != Some(RuntimeImportStatus::Imported) {
                continue;
            }
            let Some(definition) = definition_by_type_key(candidate.type_key) else {
                continue;
            };
            if definition.is_a("TransitionViewModelCondition") {
                break;
            }
            if !definition.is_a("TransitionComparator") {
                continue;
            }
            if comparators.left.is_none() {
                comparators.left = Some(candidate);
            } else {
                comparators.right = Some(candidate);
            }
        }
        comparators
    }

    fn cpp_data_converter_output_type(
        &self,
        data_converter: &RuntimeObject,
        visiting: &mut BTreeSet<u32>,
    ) -> Option<RuntimeDataType> {
        if !visiting.insert(data_converter.id) {
            return None;
        }

        let output_type = if data_converter.type_name == "DataConverterGroup" {
            let Some(data_converter_index) = self
                .data_converters()
                .into_iter()
                .position(|candidate| candidate.id == data_converter.id)
            else {
                visiting.remove(&data_converter.id);
                return None;
            };

            let mut output_type = RuntimeDataType::None;
            for item in self
                .cpp_data_converter_group_items(data_converter_index)
                .into_iter()
                .rev()
            {
                let Some(converter) = item.converter else {
                    visiting.remove(&data_converter.id);
                    return None;
                };
                let child_output_type = self.cpp_data_converter_output_type(converter, visiting)?;
                if child_output_type != RuntimeDataType::Input {
                    output_type = child_output_type;
                    break;
                }
            }
            output_type
        } else {
            cpp_data_converter_direct_output_type(data_converter)?
        };

        visiting.remove(&data_converter.id);
        Some(output_type)
    }

    fn cpp_data_converter_convert<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
        input: &RuntimeConvertedDataValue<'a>,
        data_bind_flags: Option<u64>,
        view_model_instances: &[&RuntimeObject],
        mut formula_randoms: Option<&mut RuntimeFormulaRandomSource<'_>>,
        visiting: &mut BTreeSet<u32>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        if !visiting.insert(data_converter.id) {
            return None;
        }

        let value = match data_converter.type_name {
            "DataConverterGroup" => {
                let Some(data_converter_index) = self
                    .data_converters()
                    .into_iter()
                    .position(|candidate| candidate.id == data_converter.id)
                else {
                    visiting.remove(&data_converter.id);
                    return None;
                };

                let mut value = input.clone();
                for item in self.cpp_data_converter_group_items(data_converter_index) {
                    if let Some(converter) = item.converter {
                        value = self.cpp_data_converter_convert(
                            converter,
                            &value,
                            data_bind_flags,
                            view_model_instances,
                            formula_randoms.as_deref_mut(),
                            visiting,
                        )?;
                    }
                }
                value
            }
            "DataConverterBooleanNegate" => {
                RuntimeConvertedDataValue::Boolean(!input.as_boolean().unwrap_or(false))
            }
            "DataConverterListToLength" => {
                RuntimeConvertedDataValue::Number(input.list_len().unwrap_or(0) as f32)
            }
            "DataConverterNumberToList" => match input {
                RuntimeConvertedDataValue::List(_)
                | RuntimeConvertedDataValue::GeneratedList(_) => input.clone(),
                RuntimeConvertedDataValue::Number(value) => {
                    let count = if value.is_finite() {
                        value.floor().max(0.0) as usize
                    } else {
                        0
                    };
                    RuntimeConvertedDataValue::GeneratedList(
                        self.cpp_number_to_list_generated_items(data_converter, count),
                    )
                }
                _ => {
                    visiting.remove(&data_converter.id);
                    return None;
                }
            },
            "DataConverterTrigger" => RuntimeConvertedDataValue::Trigger(
                input
                    .as_cpp_integer_super_value()
                    .map(|value| u64::from(value.wrapping_add(1)))
                    .unwrap_or(0),
            ),
            "DataConverterToNumber" => RuntimeConvertedDataValue::Number(match input {
                RuntimeConvertedDataValue::String(value) => cpp_atof_f32(value),
                RuntimeConvertedDataValue::Enum { value, .. } => *value as f32,
                RuntimeConvertedDataValue::Number(value) => *value,
                RuntimeConvertedDataValue::Color(value) => (*value as i32) as f32,
                RuntimeConvertedDataValue::Boolean(value) => {
                    if *value {
                        1.0
                    } else {
                        0.0
                    }
                }
                RuntimeConvertedDataValue::SymbolListIndex(value) => *value as f32,
                _ => 0.0,
            }),
            "DataConverterToString" => RuntimeConvertedDataValue::String(
                self.cpp_data_converter_to_string(data_converter, input)?,
            ),
            "DataConverterRounder" => RuntimeConvertedDataValue::Number(match input {
                RuntimeConvertedDataValue::Number(value) => {
                    let decimals = data_converter.uint_property("decimals").unwrap_or(0) as f32;
                    let rounder = 10.0_f32.powf(decimals);
                    (value * rounder).round() / rounder
                }
                _ => 0.0,
            }),
            "DataConverterRangeMapper" => RuntimeConvertedDataValue::Number(
                self.cpp_convert_range_mapper(data_converter, input)?,
            ),
            "DataConverterInterpolator" => input.clone(),
            "DataConverterStringRemoveZeros" => RuntimeConvertedDataValue::String(match input {
                RuntimeConvertedDataValue::String(value) => cpp_remove_trailing_zeros(value),
                _ => Vec::new(),
            }),
            "DataConverterStringTrim" => RuntimeConvertedDataValue::String(match input {
                RuntimeConvertedDataValue::String(value) => {
                    cpp_trim_string(value, data_converter.uint_property("trimType").unwrap_or(1))
                }
                _ => Vec::new(),
            }),
            "DataConverterStringPad" => RuntimeConvertedDataValue::String(match input {
                RuntimeConvertedDataValue::String(value) => cpp_pad_string(
                    value,
                    data_converter.uint_property("length").unwrap_or(1),
                    data_converter
                        .string_property_bytes("text")
                        .unwrap_or_default(),
                    data_converter.uint_property("padType").unwrap_or(0),
                ),
                _ => Vec::new(),
            }),
            "DataConverterOperationValue" => {
                RuntimeConvertedDataValue::Number(cpp_convert_operation_value(
                    input,
                    data_converter.uint_property("operationType").unwrap_or(0),
                    data_converter
                        .double_property("operationValue")
                        .unwrap_or(1.0),
                ))
            }
            "DataConverterOperationViewModel" => {
                RuntimeConvertedDataValue::Number(cpp_convert_operation_value(
                    input,
                    data_converter.uint_property("operationType").unwrap_or(0),
                    self.cpp_data_converter_operation_view_model_value(
                        data_converter,
                        view_model_instances,
                    ),
                ))
            }
            "DataConverterSystemDegsToRads" | "DataConverterSystemNormalizer" => {
                let flags = data_bind_flags?;
                let operation_type = data_converter.uint_property("operationType").unwrap_or(0);
                let operation_value = data_converter
                    .double_property("operationValue")
                    .unwrap_or(1.0);
                RuntimeConvertedDataValue::Number(if flags & 1 == 1 {
                    cpp_reverse_convert_operation_value(input, operation_type, operation_value)
                } else {
                    cpp_convert_operation_value(input, operation_type, operation_value)
                })
            }
            "DataConverterFormula" => RuntimeConvertedDataValue::Number(self.cpp_convert_formula(
                data_converter,
                input,
                formula_randoms.as_deref_mut(),
            )?),
            "DataConverter" | "DataConverterOperation" | "ScriptedDataConverter" => input.clone(),
            _ => {
                visiting.remove(&data_converter.id);
                return None;
            }
        };

        visiting.remove(&data_converter.id);
        Some(value)
    }

    fn cpp_data_converter_reverse_convert<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
        input: &RuntimeConvertedDataValue<'a>,
        data_bind_flags: Option<u64>,
        view_model_instances: &[&RuntimeObject],
        mut formula_randoms: Option<&mut RuntimeFormulaRandomSource<'_>>,
        visiting: &mut BTreeSet<u32>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        if !visiting.insert(data_converter.id) {
            return None;
        }

        let value = match data_converter.type_name {
            "DataConverterGroup" => {
                let Some(data_converter_index) = self
                    .data_converters()
                    .into_iter()
                    .position(|candidate| candidate.id == data_converter.id)
                else {
                    visiting.remove(&data_converter.id);
                    return None;
                };

                let mut value = input.clone();
                for item in self
                    .cpp_data_converter_group_items(data_converter_index)
                    .into_iter()
                    .rev()
                {
                    if let Some(converter) = item.converter {
                        value = self.cpp_data_converter_reverse_convert(
                            converter,
                            &value,
                            data_bind_flags,
                            view_model_instances,
                            formula_randoms.as_deref_mut(),
                            visiting,
                        )?;
                    }
                }
                value
            }
            "DataConverterBooleanNegate" => {
                RuntimeConvertedDataValue::Boolean(!input.as_boolean().unwrap_or(false))
            }
            "DataConverterOperationValue" => {
                RuntimeConvertedDataValue::Number(cpp_reverse_convert_operation_value(
                    input,
                    data_converter.uint_property("operationType").unwrap_or(0),
                    data_converter
                        .double_property("operationValue")
                        .unwrap_or(1.0),
                ))
            }
            "DataConverterOperationViewModel" => {
                RuntimeConvertedDataValue::Number(cpp_reverse_convert_operation_value(
                    input,
                    data_converter.uint_property("operationType").unwrap_or(0),
                    self.cpp_data_converter_operation_view_model_value(
                        data_converter,
                        view_model_instances,
                    ),
                ))
            }
            "DataConverterRangeMapper" => RuntimeConvertedDataValue::Number(
                self.cpp_reverse_convert_range_mapper(data_converter, input)?,
            ),
            "DataConverterSystemDegsToRads" | "DataConverterSystemNormalizer" => {
                let flags = data_bind_flags?;
                let operation_type = data_converter.uint_property("operationType").unwrap_or(0);
                let operation_value = data_converter
                    .double_property("operationValue")
                    .unwrap_or(1.0);
                RuntimeConvertedDataValue::Number(if flags & 1 == 0 {
                    cpp_convert_operation_value(input, operation_type, operation_value)
                } else {
                    cpp_reverse_convert_operation_value(input, operation_type, operation_value)
                })
            }
            "DataConverterFormula" => RuntimeConvertedDataValue::Number(self.cpp_convert_formula(
                data_converter,
                input,
                formula_randoms.as_deref_mut(),
            )?),
            "DataConverter"
            | "DataConverterInterpolator"
            | "DataConverterOperation"
            | "DataConverterListToLength"
            | "DataConverterNumberToList"
            | "DataConverterRounder"
            | "DataConverterStringPad"
            | "DataConverterStringRemoveZeros"
            | "DataConverterStringTrim"
            | "DataConverterToNumber"
            | "DataConverterToString"
            | "DataConverterTrigger"
            | "ScriptedDataConverter" => input.clone(),
            _ => {
                visiting.remove(&data_converter.id);
                return None;
            }
        };

        visiting.remove(&data_converter.id);
        Some(value)
    }

    fn cpp_data_converter_stateful_convert<'a>(
        &'a self,
        data_converter: &'a RuntimeObject,
        input: &RuntimeConvertedDataValue<'a>,
        state: &mut RuntimeDataConverterState,
        reverse: bool,
        visiting: &mut BTreeSet<u32>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        match data_converter.type_name {
            "DataConverterGroup" => {
                if !visiting.insert(data_converter.id) {
                    return None;
                }
                let Some(data_converter_index) = self
                    .data_converters()
                    .into_iter()
                    .position(|candidate| candidate.id == data_converter.id)
                else {
                    visiting.remove(&data_converter.id);
                    return None;
                };

                let mut value = input.clone();
                let mut items = self.cpp_data_converter_group_items(data_converter_index);
                if reverse {
                    items.reverse();
                }
                for item in items {
                    if let Some(converter) = item.converter {
                        value = self.cpp_data_converter_stateful_convert(
                            converter, &value, state, reverse, visiting,
                        )?;
                    }
                }
                visiting.remove(&data_converter.id);
                Some(value)
            }
            "DataConverterInterpolator" => state
                .interpolator_state(data_converter.id)
                .convert_converted(data_converter, input),
            _ if reverse => self.cpp_data_converter_reverse_convert(
                data_converter,
                input,
                None,
                &[],
                None,
                &mut BTreeSet::new(),
            ),
            _ => self.cpp_data_converter_convert(
                data_converter,
                input,
                None,
                &[],
                None,
                &mut BTreeSet::new(),
            ),
        }
    }

    fn cpp_data_converter_stateful_advance(
        &self,
        data_converter: &RuntimeObject,
        state: &mut RuntimeDataConverterState,
        elapsed_seconds: f32,
        visiting: &mut BTreeSet<u32>,
    ) -> Option<bool> {
        match data_converter.type_name {
            "DataConverterGroup" => {
                if !visiting.insert(data_converter.id) {
                    return None;
                }
                let Some(data_converter_index) = self
                    .data_converters()
                    .into_iter()
                    .position(|candidate| candidate.id == data_converter.id)
                else {
                    visiting.remove(&data_converter.id);
                    return None;
                };

                let mut did_update = false;
                for item in self.cpp_data_converter_group_items(data_converter_index) {
                    if let Some(converter) = item.converter
                        && self.cpp_data_converter_stateful_advance(
                            converter,
                            state,
                            elapsed_seconds,
                            visiting,
                        )?
                    {
                        did_update = true;
                    }
                }
                visiting.remove(&data_converter.id);
                Some(did_update)
            }
            "DataConverterInterpolator" => state.interpolator_state(data_converter.id).advance(
                self,
                data_converter,
                elapsed_seconds,
            ),
            "DataConverter"
            | "DataConverterBooleanNegate"
            | "DataConverterFormula"
            | "DataConverterListToLength"
            | "DataConverterNumberToList"
            | "DataConverterOperation"
            | "DataConverterOperationValue"
            | "DataConverterOperationViewModel"
            | "DataConverterRangeMapper"
            | "DataConverterRounder"
            | "DataConverterStringPad"
            | "DataConverterStringRemoveZeros"
            | "DataConverterStringTrim"
            | "DataConverterSystemDegsToRads"
            | "DataConverterSystemNormalizer"
            | "DataConverterToNumber"
            | "DataConverterToString"
            | "DataConverterTrigger"
            | "ScriptedDataConverter" => Some(false),
            _ => None,
        }
    }

    fn cpp_data_converter_operation_view_model_value(
        &self,
        data_converter: &RuntimeObject,
        view_model_instances: &[&RuntimeObject],
    ) -> f32 {
        let Some(path) = data_converter.id_list_property("sourcePathIds") else {
            return 0.0;
        };
        let Some(value) =
            self.data_context_view_model_property_for_instance_chain(view_model_instances, &path)
        else {
            return 0.0;
        };
        if self.view_model_instance_value_data_type_for_object(value)
            != Some(RuntimeDataType::Number)
        {
            return 0.0;
        }
        value.double_property("propertyValue").unwrap_or(0.0)
    }

    fn cpp_number_to_list_generated_items(
        &self,
        data_converter: &RuntimeObject,
        count: usize,
    ) -> Vec<RuntimeGeneratedListItem> {
        let Some(view_model) =
            self.resolved_view_model_for_number_to_list_converter_object(data_converter)
        else {
            return Vec::new();
        };
        let view_model_index = data_converter.uint_property("viewModelId").unwrap_or(0) as usize;

        let value_core_types = view_model
            .instances
            .first()
            .map(|instance| {
                instance
                    .values
                    .iter()
                    .map(|value| value.object.type_key)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| {
                view_model
                    .properties
                    .iter()
                    .filter_map(|property| {
                        cpp_view_model_property_instance_type_key(property.type_name)
                    })
                    .collect::<Vec<_>>()
            });

        vec![
            RuntimeGeneratedListItem {
                view_model_index,
                view_model_id: view_model_index as u64,
                value_core_types,
            };
            count
        ]
    }

    fn cpp_convert_formula(
        &self,
        data_converter: &RuntimeObject,
        input: &RuntimeConvertedDataValue<'_>,
        mut formula_randoms: Option<&mut RuntimeFormulaRandomSource<'_>>,
    ) -> Option<f32> {
        let input_value = match input {
            RuntimeConvertedDataValue::Number(value) => *value,
            RuntimeConvertedDataValue::SymbolListIndex(value) => *value as f32,
            _ => return Some(0.0),
        };
        let mut result_value = input_value;
        let Some(data_converter_index) = self
            .data_converters()
            .into_iter()
            .position(|candidate| candidate.id == data_converter.id)
        else {
            return None;
        };

        let mut stack = Vec::new();
        for token in self.cpp_data_converter_formula_output_tokens(data_converter_index) {
            match token.object.type_name {
                "FormulaTokenOperation" => {
                    if stack.len() > 1 {
                        let right = stack.pop().expect("stack length checked");
                        let left = stack.pop().expect("stack length checked");
                        stack.push(cpp_apply_formula_operation(
                            left,
                            right,
                            token.object.uint_property("operationType").unwrap_or(0),
                        ));
                    }
                }
                "FormulaTokenFunction" => {
                    let value = cpp_apply_formula_function(
                        &mut stack,
                        token.object.uint_property("functionType").unwrap_or(0),
                        token.arguments_count,
                        data_converter.uint_property("randomModeValue").unwrap_or(0),
                        formula_randoms.as_deref_mut(),
                    )?;
                    stack.push(value);
                }
                "FormulaTokenInput" => stack.push(input_value),
                "FormulaTokenValue" => stack.push(
                    token
                        .object
                        .double_property("operationValue")
                        .unwrap_or(1.0),
                ),
                _ => {}
            }
        }

        if stack.len() == 1 {
            result_value = stack.pop().expect("stack length checked");
        }
        Some(result_value)
    }

    fn cpp_convert_range_mapper(
        &self,
        data_converter: &RuntimeObject,
        input: &RuntimeConvertedDataValue<'_>,
    ) -> Option<f32> {
        let min_input = data_converter.double_property("minInput").unwrap_or(1.0);
        let max_input = data_converter.double_property("maxInput").unwrap_or(1.0);
        let min_output = data_converter.double_property("minOutput").unwrap_or(1.0);
        let max_output = data_converter.double_property("maxOutput").unwrap_or(1.0);
        self.cpp_convert_range_mapper_with_bounds(
            data_converter,
            input,
            min_input,
            max_input,
            min_output,
            max_output,
        )
    }

    fn cpp_reverse_convert_range_mapper(
        &self,
        data_converter: &RuntimeObject,
        input: &RuntimeConvertedDataValue<'_>,
    ) -> Option<f32> {
        let min_input = data_converter.double_property("minInput").unwrap_or(1.0);
        let max_input = data_converter.double_property("maxInput").unwrap_or(1.0);
        let min_output = data_converter.double_property("minOutput").unwrap_or(1.0);
        let max_output = data_converter.double_property("maxOutput").unwrap_or(1.0);
        self.cpp_convert_range_mapper_with_bounds(
            data_converter,
            input,
            min_output,
            max_output,
            min_input,
            max_input,
        )
    }

    fn cpp_convert_range_mapper_with_bounds(
        &self,
        data_converter: &RuntimeObject,
        input: &RuntimeConvertedDataValue<'_>,
        min_input: f32,
        max_input: f32,
        min_output: f32,
        max_output: f32,
    ) -> Option<f32> {
        let RuntimeConvertedDataValue::Number(input_value) = input else {
            return Some(0.0);
        };

        if min_output == max_output {
            return Some(min_output);
        }

        const CLAMP_LOWER: u64 = 1 << 0;
        const CLAMP_UPPER: u64 = 1 << 1;
        const MODULO: u64 = 1 << 2;
        const REVERSE: u64 = 1 << 3;

        let flags = data_converter.uint_property("flags").unwrap_or(0);
        let mut value = *input_value;
        if value < min_input && flags & CLAMP_LOWER != 0 {
            value = min_input;
        } else if value > max_input && flags & CLAMP_UPPER != 0 {
            value = max_input;
        }
        if (value < min_input || value > max_input) && flags & MODULO != 0 {
            value = (cpp_positive_mod(value, max_input - min_input) + min_input).abs();
        }

        let mut percent = (value - min_input) / (max_input - min_input);
        if flags & REVERSE != 0 {
            percent = 1.0 - percent;
        }
        if let Some(interpolator) =
            self.resolved_interpolator_for_data_converter_object(data_converter)
        {
            if percent > 0.0 && percent < 1.0 {
                percent = cpp_key_frame_interpolator_transform(interpolator, percent)?;
            } else if data_converter
                .uint_property("interpolationType")
                .unwrap_or(1)
                == 0
            {
                percent = if percent <= 0.0 { 0.0 } else { 1.0 };
            }
        } else if data_converter
            .uint_property("interpolationType")
            .unwrap_or(1)
            == 0
        {
            percent = if percent <= 0.0 { 0.0 } else { 1.0 };
        }

        Some(percent * max_output + (1.0 - percent) * min_output)
    }

    fn cpp_data_converter_to_string<'a>(
        &self,
        data_converter: &RuntimeObject,
        input: &RuntimeConvertedDataValue<'a>,
    ) -> Option<Vec<u8>> {
        match input {
            RuntimeConvertedDataValue::Number(value) => Some(cpp_format_number_to_string(
                *value,
                data_converter.uint_property("flags").unwrap_or(0),
                data_converter.uint_property("decimals").unwrap_or(0),
            )),
            RuntimeConvertedDataValue::Enum { value, data_enum } => {
                let Some(data_enum) = data_enum.as_ref() else {
                    return Some(Vec::new());
                };
                let value_index = usize::try_from(*value).ok()?;
                let Some(enum_value) = data_enum.values.get(value_index) else {
                    return Some(Vec::new());
                };
                let display_value = enum_value
                    .string_property_bytes("value")
                    .unwrap_or_default();
                if display_value.is_empty() {
                    Some(
                        enum_value
                            .string_property_bytes("key")
                            .unwrap_or_default()
                            .to_vec(),
                    )
                } else {
                    Some(display_value.to_vec())
                }
            }
            RuntimeConvertedDataValue::String(value) => Some(value.clone()),
            RuntimeConvertedDataValue::Color(value) => {
                let color_format = data_converter
                    .string_property_bytes("colorFormat")
                    .unwrap_or_default();
                if color_format.is_empty() {
                    Some(((*value as i32).to_string()).into_bytes())
                } else {
                    Some(cpp_format_color_to_string(*value, color_format))
                }
            }
            RuntimeConvertedDataValue::Boolean(value) => {
                Some(if *value { b"1".to_vec() } else { b"0".to_vec() })
            }
            RuntimeConvertedDataValue::Trigger(value) => Some(value.to_string().into_bytes()),
            RuntimeConvertedDataValue::SymbolListIndex(value) => {
                Some(value.to_string().into_bytes())
            }
            _ => Some(Vec::new()),
        }
    }

    fn cpp_data_converter_interpolators(&self) -> Vec<&RuntimeObject> {
        let mut latest_artboard_importer = false;
        let mut interpolators = Vec::new();

        for (index, object) in self.objects.iter().enumerate() {
            if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }

            let Some(object) = object.as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if definition.name == "Artboard" {
                latest_artboard_importer = true;
                continue;
            }

            if definition.is_a("KeyFrameInterpolator") && !latest_artboard_importer {
                interpolators.push(object);
            }
        }

        interpolators
    }

    fn cpp_data_converter_formula_tokens(
        &self,
        data_converter_index: usize,
    ) -> Vec<&RuntimeObject> {
        self.cpp_data_converter_formula_output_tokens(data_converter_index)
            .into_iter()
            .map(|token| token.object)
            .collect()
    }

    fn cpp_data_converter_formula_output_tokens(
        &self,
        data_converter_index: usize,
    ) -> Vec<RuntimeFormulaOutputToken<'_>> {
        let Some(formula) = self.data_converter(data_converter_index) else {
            return Vec::new();
        };
        if formula.type_name != "DataConverterFormula" {
            return Vec::new();
        }

        let mut latest_formula_index = None;
        let mut current_converter_index = 0usize;
        let mut tokens = Vec::new();

        for (file_index, object) in self.objects.iter().enumerate() {
            if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }

            let Some(object) = object.as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if definition.is_a("DataConverter") {
                if definition.name == "DataConverterFormula" {
                    latest_formula_index = Some(current_converter_index);
                }
                current_converter_index += 1;
                continue;
            }

            if definition.is_a("FormulaToken") && latest_formula_index == Some(data_converter_index)
            {
                tokens.push(object);
            }
        }

        Self::cpp_data_converter_formula_output_queue(tokens)
    }

    fn cpp_data_converter_formula_output_queue<'a>(
        tokens: Vec<&'a RuntimeObject>,
    ) -> Vec<RuntimeFormulaOutputToken<'a>> {
        let mut operations_stack: Vec<&'a RuntimeObject> = Vec::new();
        let mut output_queue: Vec<RuntimeFormulaOutputToken<'a>> = Vec::new();
        let mut arguments_count = BTreeMap::new();

        for (token_index, token) in tokens.iter().enumerate() {
            match token.type_name {
                "FormulaTokenValue" | "FormulaTokenInput" => {
                    output_queue.push(RuntimeFormulaOutputToken::new(token, &arguments_count))
                }
                "FormulaTokenOperation" => {
                    while operations_stack.last().is_some_and(|operation| {
                        operation.type_name != "FormulaTokenParenthesisOpen"
                            && Self::cpp_formula_token_precedence(operation)
                                >= Self::cpp_formula_token_precedence(token)
                    }) {
                        let operation = operations_stack.pop().expect("stack has last token");
                        output_queue
                            .push(RuntimeFormulaOutputToken::new(operation, &arguments_count));
                    }
                    operations_stack.push(*token);
                }
                "FormulaTokenParenthesisOpen" | "FormulaTokenFunction" => {
                    let argument_count = if tokens
                        .get(token_index + 1)
                        .is_some_and(|next| next.type_name == "FormulaTokenParenthesisClose")
                    {
                        0
                    } else {
                        1
                    };
                    arguments_count.insert(token.id, argument_count);
                    operations_stack.push(*token);
                }
                "FormulaTokenParenthesisClose" => {
                    while operations_stack.last().is_some_and(|operation| {
                        operation.type_name != "FormulaTokenParenthesisOpen"
                            && operation.type_name != "FormulaTokenFunction"
                    }) {
                        let operation = operations_stack.pop().expect("stack has last token");
                        output_queue
                            .push(RuntimeFormulaOutputToken::new(operation, &arguments_count));
                    }
                    if let Some(opening_token) = operations_stack.pop()
                        && opening_token.type_name == "FormulaTokenFunction"
                    {
                        output_queue.push(RuntimeFormulaOutputToken::new(
                            opening_token,
                            &arguments_count,
                        ));
                    }
                }
                "FormulaTokenArgumentSeparator" if !operations_stack.is_empty() => {
                    if let Some(argument_token) = operations_stack
                        .iter()
                        .rev()
                        .find(|operation| arguments_count.contains_key(&operation.id))
                    {
                        let count = arguments_count
                            .get(&argument_token.id)
                            .copied()
                            .unwrap_or(0);
                        arguments_count.insert(argument_token.id, count + 1);
                    }
                    while operations_stack.last().is_some_and(|operation| {
                        operation.type_name != "FormulaTokenParenthesisOpen"
                            && operation.type_name != "FormulaTokenFunction"
                    }) {
                        let operation = operations_stack.pop().expect("stack has last token");
                        output_queue
                            .push(RuntimeFormulaOutputToken::new(operation, &arguments_count));
                    }
                }
                _ => {}
            }
        }

        while let Some(operation) = operations_stack.pop() {
            if operation.type_name != "FormulaTokenParenthesisOpen" {
                output_queue.push(RuntimeFormulaOutputToken::new(operation, &arguments_count));
            }
        }

        output_queue
    }

    fn cpp_formula_token_precedence(token: &RuntimeObject) -> u8 {
        let Some(definition) = definition_by_type_key(token.type_key) else {
            return 0;
        };
        if definition.is_a("FormulaTokenParenthesis") {
            return 1;
        }
        if definition.name == "FormulaTokenOperation" {
            return match token.uint_property("operationType").unwrap_or(0) {
                0 | 1 => 2,
                2 | 3 => 3,
                _ => 0,
            };
        }
        0
    }

    fn cpp_scroll_physics(&self) -> impl Iterator<Item = &RuntimeObject> {
        self.objects
            .iter()
            .enumerate()
            .filter_map(|(index, object)| {
                if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                definition_by_type_key(object.type_key)
                    .is_some_and(|definition| definition.is_a("ScrollPhysics"))
                    .then_some(object)
            })
    }

    fn cpp_data_converter_group_items(
        &self,
        data_converter_index: usize,
    ) -> Vec<RuntimeDataConverterGroupItem<'_>> {
        let Some(group) = self.data_converter(data_converter_index) else {
            return Vec::new();
        };
        if group.type_name != "DataConverterGroup" {
            return Vec::new();
        }

        let mut current_group_index = None;
        let mut current_converter_index = 0usize;
        let mut items = Vec::new();

        for (file_index, object) in self.objects.iter().enumerate() {
            if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }

            let Some(object) = object.as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if definition.is_a("DataConverter") {
                if definition.name == "DataConverterGroup" {
                    current_group_index = Some(current_converter_index);
                }
                current_converter_index += 1;
                continue;
            }

            if definition.name == "DataConverterGroupItem"
                && current_group_index == Some(data_converter_index)
            {
                items.push(RuntimeDataConverterGroupItem {
                    object,
                    converter: self.resolved_data_converter_for_group_item_object(object),
                });
            }
        }

        items
    }

    fn cpp_data_converter_group_child_converter_ids(
        &self,
        data_converter: &RuntimeObject,
    ) -> Vec<usize> {
        if data_converter.type_name != "DataConverterGroup" {
            return Vec::new();
        }

        self.data_converter_group_items_for_object(data_converter)
            .into_iter()
            .filter_map(|item| {
                item.converter
                    .and_then(|converter| usize::try_from(converter.id).ok())
            })
            .collect()
    }

    fn cpp_view_models(&self) -> Vec<RuntimeViewModel<'_>> {
        let mut view_models = Vec::<RuntimeViewModel<'_>>::new();
        let mut latest_view_model = None;
        let mut latest_view_model_instance = None;
        let mut latest_view_model_instance_list = None;

        for (index, object) in self.objects.iter().enumerate() {
            if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }

            let Some(object) = object.as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if definition.name == "ViewModel" {
                view_models.push(RuntimeViewModel {
                    object,
                    properties: Vec::new(),
                    instances: Vec::new(),
                });
                latest_view_model = Some(view_models.len() - 1);
                continue;
            }

            if definition.is_a("ViewModelProperty") {
                if let Some(view_model_index) = latest_view_model {
                    view_models[view_model_index].properties.push(object);
                }
                continue;
            }

            if definition.name == "ViewModelInstance" {
                latest_view_model_instance = None;
                let Some(view_model_index) = object.uint_property("viewModelId") else {
                    continue;
                };
                let Ok(view_model_index) = usize::try_from(view_model_index) else {
                    continue;
                };
                if let Some(view_model) = view_models.get_mut(view_model_index) {
                    view_model.instances.push(RuntimeViewModelInstance {
                        object,
                        values: Vec::new(),
                    });
                    latest_view_model_instance =
                        Some((view_model_index, view_model.instances.len() - 1));
                }
                continue;
            }

            if definition.is_a("ViewModelInstanceValue") {
                let Some((view_model_index, instance_index)) = latest_view_model_instance else {
                    if definition.name == "ViewModelInstanceList" {
                        latest_view_model_instance_list = None;
                    }
                    continue;
                };
                view_models[view_model_index].instances[instance_index]
                    .values
                    .push(RuntimeViewModelInstanceValue {
                        object,
                        list_items: Vec::new(),
                    });
                if definition.name == "ViewModelInstanceList" {
                    latest_view_model_instance_list = Some((
                        view_model_index,
                        instance_index,
                        view_models[view_model_index].instances[instance_index]
                            .values
                            .len()
                            - 1,
                    ));
                }
                continue;
            }

            if definition.name == "ViewModelInstanceListItem" {
                let Some((view_model_index, instance_index, value_index)) =
                    latest_view_model_instance_list
                else {
                    continue;
                };
                view_models[view_model_index].instances[instance_index].values[value_index]
                    .list_items
                    .push(object);
            }
        }

        view_models
    }
}

fn resolve_runtime_state_machine_transition_targets(
    state_machines: &mut [RuntimeStateMachine<'_>],
) {
    for state_machine in state_machines {
        for layer in &mut state_machine.layers {
            let state_objects = layer
                .states
                .iter()
                .map(|state| state.object)
                .collect::<Vec<_>>();

            for state in &mut layer.states {
                let state_is_blend = state.object.is_some_and(|object| {
                    definition_by_type_key(object.type_key)
                        .is_some_and(|definition| definition.is_a("BlendState"))
                });
                let blend_animations = state
                    .blend_animations
                    .iter()
                    .map(|animation| {
                        (
                            animation.object,
                            animation.animation_index,
                            animation.animation,
                        )
                    })
                    .collect::<Vec<_>>();

                for transition in &mut state.transitions {
                    let state_to_index = usize::try_from(
                        transition
                            .object
                            .uint_property("stateToId")
                            .unwrap_or(u64::MAX),
                    )
                    .ok()
                    .filter(|index| *index < state_objects.len());
                    transition.state_to_index = state_to_index;
                    transition.state_to = state_to_index.and_then(|index| state_objects[index]);

                    let transition_is_blend = definition_by_type_key(transition.object.type_key)
                        .is_some_and(|definition| definition.is_a("BlendStateTransition"));
                    if !state_is_blend || !transition_is_blend {
                        continue;
                    }

                    let exit_blend_animation_index = usize::try_from(
                        transition
                            .object
                            .uint_property("exitBlendAnimationId")
                            .unwrap_or(u64::MAX),
                    )
                    .ok()
                    .filter(|index| *index < blend_animations.len());
                    if let Some(index) = exit_blend_animation_index {
                        let (blend_animation, animation_index, animation) = blend_animations[index];
                        transition.exit_blend_animation_index = Some(index);
                        transition.exit_blend_animation = Some(blend_animation);
                        transition.exit_animation_index = animation_index;
                        transition.exit_animation = animation;
                    }
                }
            }
        }
    }
}

fn cpp_runtime_state_machine_fire_action<'a>(
    object: &'a RuntimeObject,
    artboard_local_slots: &[Option<usize>],
    objects: &'a [Option<RuntimeObject>],
) -> RuntimeStateMachineFireAction<'a> {
    let (event_local_index, event) =
        cpp_resolved_action_event(object, artboard_local_slots, objects);
    RuntimeStateMachineFireAction {
        object,
        event_local_index,
        event,
    }
}

fn cpp_runtime_listener_action<'a>(
    object: &'a RuntimeObject,
    artboard_local_slots: &[Option<usize>],
    objects: &'a [Option<RuntimeObject>],
) -> RuntimeListenerAction<'a> {
    let (event_local_index, event) =
        cpp_resolved_action_event(object, artboard_local_slots, objects);
    RuntimeListenerAction {
        object,
        event_local_index,
        event,
    }
}

fn cpp_resolved_action_event<'a>(
    action: &RuntimeObject,
    artboard_local_slots: &[Option<usize>],
    objects: &'a [Option<RuntimeObject>],
) -> (Option<usize>, Option<&'a RuntimeObject>) {
    if !matches!(
        action.type_name,
        "StateMachineFireEvent" | "ListenerFireEvent"
    ) {
        return (None, None);
    }

    let Some((local_index, event)) = local_object_reference_with_local_index(
        artboard_local_slots,
        objects,
        action.uint_property("eventId"),
    ) else {
        return (None, None);
    };

    let is_event =
        definition_by_type_key(event.type_key).is_some_and(|definition| definition.is_a("Event"));
    if is_event {
        (Some(local_index), Some(event))
    } else {
        (None, None)
    }
}

fn cpp_data_context_view_model_property<'a>(
    runtime_file: &'a RuntimeFile,
    view_models: &[RuntimeViewModel<'a>],
    view_model_instances: &[&RuntimeObject],
    path: &[u32],
) -> Option<&'a RuntimeObject> {
    if path.is_empty() {
        return None;
    }

    for view_model_instance in view_model_instances {
        let Some(mut instance) =
            cpp_view_model_instance_by_object(view_models, view_model_instance)
        else {
            continue;
        };
        if instance.object.uint_property("viewModelId") != Some(u64::from(path[0])) {
            continue;
        }
        if path.len() == 1 {
            return None;
        }

        let mut should_try_parent = false;
        for property_id in &path[1..path.len() - 1] {
            let Some(value) = cpp_view_model_instance_value_by_property_id(instance, *property_id)
            else {
                should_try_parent = true;
                break;
            };
            let Some(reference) =
                runtime_file.referenced_view_model_instance_for_value_object(value)
            else {
                should_try_parent = true;
                break;
            };
            let Some(referenced_instance) =
                cpp_view_model_instance_by_object(view_models, reference.object)
            else {
                should_try_parent = true;
                break;
            };
            instance = referenced_instance;
        }
        if should_try_parent {
            continue;
        }

        return cpp_view_model_instance_value_by_property_id(instance, *path.last()?);
    }

    None
}

fn cpp_data_context_relative_view_model_property<'a>(
    runtime_file: &'a RuntimeFile,
    view_models: &[RuntimeViewModel<'a>],
    manifest: &RuntimeManifest,
    view_model_instances: &[&RuntimeObject],
    path: &[u32],
) -> Option<&'a RuntimeObject> {
    if path.is_empty() {
        return None;
    }

    for view_model_instance in view_model_instances {
        let Some(mut instance) =
            cpp_view_model_instance_by_object(view_models, view_model_instance)
        else {
            continue;
        };

        if path.len() == 1 {
            if let Some(value) = cpp_view_model_instance_value_by_manifest_name(
                view_models,
                manifest,
                instance,
                path[0],
            ) {
                return Some(value);
            }
            continue;
        }

        let mut should_try_parent = false;
        for name_id in &path[..path.len() - 1] {
            let Some(value) = cpp_view_model_instance_value_by_manifest_name(
                view_models,
                manifest,
                instance,
                *name_id,
            ) else {
                should_try_parent = true;
                break;
            };
            let Some(reference) =
                runtime_file.referenced_view_model_instance_for_value_object(value)
            else {
                should_try_parent = true;
                break;
            };
            let Some(referenced_instance) =
                cpp_view_model_instance_by_object(view_models, reference.object)
            else {
                should_try_parent = true;
                break;
            };
            instance = referenced_instance;
        }
        if should_try_parent {
            continue;
        }

        if let Some(value) = cpp_view_model_instance_value_by_manifest_name(
            view_models,
            manifest,
            instance,
            *path.last()?,
        ) {
            return Some(value);
        }
    }

    None
}

fn cpp_data_context_view_model_instance<'a>(
    runtime_file: &'a RuntimeFile,
    view_models: &[RuntimeViewModel<'a>],
    view_model_instances: &[&RuntimeObject],
    path: &[u32],
) -> Option<RuntimeViewModelInstanceReference<'a>> {
    if path.is_empty() {
        return None;
    }

    for view_model_instance in view_model_instances {
        let Some(mut instance) =
            cpp_view_model_instance_by_object(view_models, view_model_instance)
        else {
            continue;
        };
        if instance.object.uint_property("viewModelId") != Some(u64::from(path[0])) {
            continue;
        }

        let mut should_try_parent = false;
        for property_id in &path[1..] {
            let Some(value) = cpp_view_model_instance_value_by_property_id(instance, *property_id)
            else {
                should_try_parent = true;
                break;
            };
            let Some(reference) =
                runtime_file.referenced_view_model_instance_for_value_object(value)
            else {
                should_try_parent = true;
                break;
            };
            let Some(referenced_instance) =
                cpp_view_model_instance_by_object(view_models, reference.object)
            else {
                should_try_parent = true;
                break;
            };
            instance = referenced_instance;
        }
        if should_try_parent {
            continue;
        }

        return cpp_view_model_instance_reference_by_object(view_models, instance.object);
    }

    None
}

fn cpp_data_context_relative_view_model_instance<'a>(
    runtime_file: &'a RuntimeFile,
    view_models: &[RuntimeViewModel<'a>],
    manifest: &RuntimeManifest,
    view_model_instances: &[&RuntimeObject],
    path: &[u32],
) -> Option<RuntimeViewModelInstanceReference<'a>> {
    if path.is_empty() {
        return None;
    }

    for view_model_instance in view_model_instances {
        let Some(mut instance) =
            cpp_view_model_instance_by_object(view_models, view_model_instance)
        else {
            continue;
        };

        let mut should_try_parent = false;
        for name_id in path {
            let Some(value) = cpp_view_model_instance_value_by_manifest_name(
                view_models,
                manifest,
                instance,
                *name_id,
            ) else {
                should_try_parent = true;
                break;
            };
            let Some(reference) =
                runtime_file.referenced_view_model_instance_for_value_object(value)
            else {
                should_try_parent = true;
                break;
            };
            let Some(referenced_instance) =
                cpp_view_model_instance_by_object(view_models, reference.object)
            else {
                should_try_parent = true;
                break;
            };
            instance = referenced_instance;
        }
        if should_try_parent {
            continue;
        }

        return cpp_view_model_instance_reference_by_object(view_models, instance.object);
    }

    None
}

fn cpp_view_model_instance_reference_by_object<'a>(
    view_models: &[RuntimeViewModel<'a>],
    object: &RuntimeObject,
) -> Option<RuntimeViewModelInstanceReference<'a>> {
    view_models
        .iter()
        .enumerate()
        .flat_map(|(view_model_index, view_model)| {
            view_model
                .instances
                .iter()
                .enumerate()
                .map(move |(instance_index, instance)| (view_model_index, instance_index, instance))
        })
        .find_map(|(view_model_index, instance_index, instance)| {
            (instance.object.id == object.id).then_some(RuntimeViewModelInstanceReference {
                view_model_index,
                instance_index,
                object: instance.object,
            })
        })
}

fn cpp_data_enum_resolved_value_bytes(value: &RuntimeObject) -> &[u8] {
    let resolved_value = value.string_property_bytes("value").unwrap_or_default();
    if resolved_value.is_empty() {
        return value.string_property_bytes("key").unwrap_or_default();
    }
    resolved_value
}

fn cpp_view_model_instance_by_object<'models, 'file>(
    view_models: &'models [RuntimeViewModel<'file>],
    object: &RuntimeObject,
) -> Option<&'models RuntimeViewModelInstance<'file>> {
    view_models
        .iter()
        .flat_map(|view_model| &view_model.instances)
        .find(|instance| instance.object.id == object.id)
}

fn cpp_view_model_instance_value_by_property_id<'models, 'file>(
    instance: &'models RuntimeViewModelInstance<'file>,
    property_id: u32,
) -> Option<&'file RuntimeObject> {
    instance
        .values
        .iter()
        .find(|value| {
            value.object.uint_property("viewModelPropertyId") == Some(u64::from(property_id))
        })
        .map(|value| value.object)
}

fn cpp_view_model_instance_value_by_manifest_name<'models, 'file>(
    view_models: &[RuntimeViewModel<'file>],
    manifest: &RuntimeManifest,
    instance: &'models RuntimeViewModelInstance<'file>,
    name_id: u32,
) -> Option<&'file RuntimeObject> {
    let name = manifest.resolve_name_bytes(name_id).unwrap_or_default();
    cpp_view_model_instance_value_by_name(view_models, instance, name)
}

fn cpp_view_model_instance_value_by_name<'models, 'file>(
    view_models: &[RuntimeViewModel<'file>],
    instance: &'models RuntimeViewModelInstance<'file>,
    name: &[u8],
) -> Option<&'file RuntimeObject> {
    let view_model_index = usize::try_from(instance.object.uint_property("viewModelId")?).ok()?;
    let view_model = view_models.get(view_model_index)?;
    instance
        .values
        .iter()
        .find(|value| {
            let Some(property_index) = value
                .object
                .uint_property("viewModelPropertyId")
                .and_then(|index| usize::try_from(index).ok())
            else {
                return false;
            };
            view_model
                .properties
                .get(property_index)
                .and_then(|property| property.string_property_bytes("name"))
                .unwrap_or_default()
                == name
        })
        .map(|value| value.object)
}

fn cpp_view_model_instance_value_symbol(
    view_model: &RuntimeViewModel<'_>,
    value: &RuntimeObject,
) -> Option<u8> {
    let property_index = value
        .uint_property("viewModelPropertyId")
        .and_then(|index| usize::try_from(index).ok())?;
    let property = view_model.properties.get(property_index)?;
    if property.type_name == "ViewModelPropertySymbolListIndex" {
        return Some(VIEW_MODEL_SYMBOL_ITEM_INDEX);
    }

    // `symbolTypeValue` is a ViewModel data-type discriminant whose domain is
    // the small RuntimeDataType enum (0..=13, plus 99/100) -- always <= 255. A
    // value that does not fit in u8 can only come from a malformed file, so we
    // treat it as "no symbol" (0) via u8::try_from rather than silently
    // truncating with `as u8`. For every in-domain value try_from is identical
    // to the old cast, so valid files are unaffected.
    let symbol = property
        .uint_property("symbolTypeValue")
        .and_then(|value| u8::try_from(value).ok())
        .unwrap_or(0);
    (symbol != 0).then_some(symbol)
}

fn cpp_owner_view_model_instance_indices(
    view_models: &[RuntimeViewModel<'_>],
    value: &RuntimeObject,
) -> Option<(usize, usize)> {
    view_models
        .iter()
        .enumerate()
        .flat_map(|(view_model_index, view_model)| {
            view_model
                .instances
                .iter()
                .enumerate()
                .map(move |(instance_index, instance)| (view_model_index, instance_index, instance))
        })
        .find_map(|(view_model_index, instance_index, instance)| {
            instance
                .values
                .iter()
                .any(|item| item.object.id == value.id)
                .then_some((view_model_index, instance_index))
        })
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RuntimeManifest {
    pub names: BTreeMap<i32, StringValue>,
    pub paths: BTreeMap<i32, Vec<u32>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeDataEnum<'a> {
    pub object: &'a RuntimeObject,
    pub values: Vec<&'a RuntimeObject>,
}

#[derive(Debug, Clone)]
pub enum RuntimeDataValue<'a> {
    None,
    Number(f32),
    String(&'a [u8]),
    Boolean(bool),
    Color(u32),
    Enum {
        value: u64,
        data_enum: Option<RuntimeDataEnum<'a>>,
    },
    Trigger(u64),
    List(Vec<&'a RuntimeObject>),
    SymbolListIndex(u64),
    AssetImage(u64),
    AssetFont(u64),
    Artboard(u64),
    ViewModel(Option<RuntimeViewModelInstanceReference<'a>>),
}

impl RuntimeDataValue<'_> {
    pub fn data_type(&self) -> RuntimeDataType {
        match self {
            Self::None => RuntimeDataType::None,
            Self::Number(_) => RuntimeDataType::Number,
            Self::String(_) => RuntimeDataType::String,
            Self::Boolean(_) => RuntimeDataType::Boolean,
            Self::Color(_) => RuntimeDataType::Color,
            Self::Enum { .. } => RuntimeDataType::EnumType,
            Self::Trigger(_) => RuntimeDataType::Trigger,
            Self::List(_) => RuntimeDataType::List,
            Self::SymbolListIndex(_) => RuntimeDataType::SymbolListIndex,
            Self::AssetImage(_) => RuntimeDataType::AssetImage,
            Self::AssetFont(_) => RuntimeDataType::AssetFont,
            Self::Artboard(_) => RuntimeDataType::Artboard,
            Self::ViewModel(_) => RuntimeDataType::ViewModel,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeConvertedDataValue<'a> {
    None,
    Number(f32),
    String(Vec<u8>),
    Boolean(bool),
    Color(u32),
    Enum {
        value: u64,
        data_enum: Option<RuntimeDataEnum<'a>>,
    },
    Integer(u64),
    Trigger(u64),
    List(Vec<&'a RuntimeObject>),
    GeneratedList(Vec<RuntimeGeneratedListItem>),
    SymbolListIndex(u64),
    AssetImage(u64),
    AssetFont(u64),
    Artboard(u64),
    ViewModel(Option<RuntimeViewModelInstanceReference<'a>>),
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeDataConverterState {
    interpolators: BTreeMap<u32, RuntimeDataConverterInterpolatorState>,
}

impl RuntimeDataConverterState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.interpolators.clear();
    }

    fn interpolator_state(
        &mut self,
        data_converter_id: u32,
    ) -> &mut RuntimeDataConverterInterpolatorState {
        self.interpolators.entry(data_converter_id).or_default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeDataConverterInterpolatorState {
    advance_count: u8,
    advancer: Option<RuntimeInterpolatorAdvancer>,
}

impl RuntimeDataConverterInterpolatorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.advance_count = 0;
        self.advancer = None;
    }

    fn convert<'a>(
        &mut self,
        data_converter: &RuntimeObject,
        input: &RuntimeDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        self.convert_converted(data_converter, &RuntimeConvertedDataValue::from(input))
    }

    fn convert_converted<'a>(
        &mut self,
        data_converter: &RuntimeObject,
        input: &RuntimeConvertedDataValue<'a>,
    ) -> Option<RuntimeConvertedDataValue<'a>> {
        if data_converter.double_property("duration").unwrap_or(1.0) == 0.0
            && let Some(advancer) = &mut self.advancer
        {
            if let Some(input_value) = RuntimeInterpolatorValue::from_converted_data_value(input) {
                advancer.reset_to_start(&input_value);
            }
            return Some(input.clone());
        }

        if self.advancer.is_none() {
            let Some(input_value) = RuntimeInterpolatorValue::from_converted_data_value(input)
            else {
                return Some(input.clone());
            };
            self.advancer = Some(RuntimeInterpolatorAdvancer::new(&input_value));
        }

        let Some(input_value) = RuntimeInterpolatorValue::from_converted_data_value(input) else {
            return Some(input.clone());
        };
        let advancer = self.advancer.as_mut().expect("advancer initialized");
        if self.advance_count < 2 {
            advancer.reset_values(&input_value);
        } else {
            advancer.update_values(&input_value);
        }
        Some(advancer.current_value().to_converted_data_value())
    }

    fn advance(
        &mut self,
        file: &RuntimeFile,
        data_converter: &RuntimeObject,
        elapsed_seconds: f32,
    ) -> Option<bool> {
        if self.advance_count < 2 && elapsed_seconds > 0.0 {
            self.advance_count += 1;
        }
        let Some(advancer) = &mut self.advancer else {
            return Some(true);
        };
        advancer.advance(file, data_converter, elapsed_seconds)
    }
}

#[derive(Debug, Clone)]
struct RuntimeInterpolatorAdvancer {
    animation_data_a: RuntimeInterpolatorAnimationData,
    animation_data_b: RuntimeInterpolatorAnimationData,
    current_value: RuntimeInterpolatorValue,
    is_smoothing_animation: bool,
}

impl RuntimeInterpolatorAdvancer {
    fn new(input: &RuntimeInterpolatorValue) -> Self {
        let default_value = input.default_for_kind();
        Self {
            animation_data_a: RuntimeInterpolatorAnimationData::new(default_value.clone()),
            animation_data_b: RuntimeInterpolatorAnimationData::new(default_value.clone()),
            current_value: default_value,
            is_smoothing_animation: false,
        }
    }

    fn current_value(&self) -> &RuntimeInterpolatorValue {
        &self.current_value
    }

    fn reset_values(&mut self, input: &RuntimeInterpolatorValue) {
        if self.is_smoothing_animation {
            self.animation_data_b.reset_values(input);
        } else {
            self.animation_data_a.reset_values(input);
        }
        self.current_value.copy_from(input);
    }

    fn reset_to_start(&mut self, input: &RuntimeInterpolatorValue) {
        self.reset_values(input);
        self.is_smoothing_animation = false;
        self.animation_data_a.elapsed_seconds = 0.0;
        self.animation_data_b.elapsed_seconds = 0.0;
    }

    fn update_values(&mut self, input: &RuntimeInterpolatorValue) {
        let target_matches = self.current_animation_data().to.compare(input);
        if target_matches {
            return;
        }

        if self.current_animation_data().elapsed_seconds != 0.0 {
            if self.is_smoothing_animation {
                self.animation_data_a
                    .copy_from(&self.animation_data_b.clone());
            }
            self.is_smoothing_animation = true;
        } else {
            self.is_smoothing_animation = false;
        }

        let current_value = self.current_value.clone();
        let animation_data = self.current_animation_data_mut();
        animation_data.from.copy_from(&current_value);
        animation_data.to.copy_from(input);
        animation_data.elapsed_seconds = 0.0;
    }

    fn advance(
        &mut self,
        file: &RuntimeFile,
        data_converter: &RuntimeObject,
        elapsed_seconds: f32,
    ) -> Option<bool> {
        let animation_index = self.current_animation_index();
        if self.animation_data(animation_index).to == self.current_value || elapsed_seconds == 0.0 {
            return Some(false);
        }

        let previous_time = self.animation_data(animation_index).elapsed_seconds;
        self.advance_animation_data(file, data_converter, elapsed_seconds, animation_index)?;
        let duration = data_converter.double_property("duration").unwrap_or(1.0);
        let _mark_dirty = previous_time < duration;
        Some(self.animation_data(animation_index).elapsed_seconds < duration)
    }

    fn advance_animation_data(
        &mut self,
        file: &RuntimeFile,
        data_converter: &RuntimeObject,
        elapsed_seconds: f32,
        animation_index: usize,
    ) -> Option<()> {
        if self.is_smoothing_animation {
            let factor = cpp_interpolator_state_factor(
                file,
                data_converter,
                self.animation_data_a.elapsed_seconds,
            )?;
            let interpolated = self.animation_data_a.interpolate(factor);
            self.animation_data_b.from.copy_from(&interpolated);
            if factor == 1.0 {
                self.animation_data_a
                    .copy_from(&self.animation_data_b.clone());
                self.is_smoothing_animation = false;
            } else {
                self.animation_data_a.elapsed_seconds += elapsed_seconds;
            }
        }

        let duration = data_converter.double_property("duration").unwrap_or(1.0);
        if self.animation_data(animation_index).elapsed_seconds >= duration {
            self.current_value
                .copy_from(&self.animation_data(animation_index).to.clone());
            if self.is_smoothing_animation {
                self.is_smoothing_animation = false;
                self.animation_data_a
                    .copy_from(&self.animation_data_b.clone());
                self.animation_data_a.elapsed_seconds = 0.0;
                self.animation_data_b.elapsed_seconds = 0.0;
            } else {
                self.animation_data_a.elapsed_seconds = 0.0;
            }
            return Some(());
        }

        self.animation_data_mut(animation_index).elapsed_seconds += elapsed_seconds;
        let factor = cpp_interpolator_state_factor(
            file,
            data_converter,
            self.animation_data(animation_index).elapsed_seconds,
        )?;
        let interpolated = self.animation_data(animation_index).interpolate(factor);
        self.current_value.copy_from(&interpolated);
        Some(())
    }

    fn current_animation_index(&self) -> usize {
        usize::from(self.is_smoothing_animation)
    }

    fn current_animation_data(&self) -> &RuntimeInterpolatorAnimationData {
        self.animation_data(self.current_animation_index())
    }

    fn current_animation_data_mut(&mut self) -> &mut RuntimeInterpolatorAnimationData {
        self.animation_data_mut(self.current_animation_index())
    }

    fn animation_data(&self, index: usize) -> &RuntimeInterpolatorAnimationData {
        if index == 0 {
            &self.animation_data_a
        } else {
            &self.animation_data_b
        }
    }

    fn animation_data_mut(&mut self, index: usize) -> &mut RuntimeInterpolatorAnimationData {
        if index == 0 {
            &mut self.animation_data_a
        } else {
            &mut self.animation_data_b
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeInterpolatorAnimationData {
    elapsed_seconds: f32,
    from: RuntimeInterpolatorValue,
    to: RuntimeInterpolatorValue,
}

impl RuntimeInterpolatorAnimationData {
    fn new(default_value: RuntimeInterpolatorValue) -> Self {
        Self {
            elapsed_seconds: 0.0,
            from: default_value.clone(),
            to: default_value,
        }
    }

    fn reset_values(&mut self, input: &RuntimeInterpolatorValue) {
        self.from.copy_from(input);
        self.to.copy_from(input);
    }

    fn copy_from(&mut self, source: &Self) {
        self.from.copy_from(&source.from);
        self.to.copy_from(&source.to);
        self.elapsed_seconds = source.elapsed_seconds;
    }

    fn interpolate(&self, factor: f32) -> RuntimeInterpolatorValue {
        self.from.interpolate(&self.to, factor)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum RuntimeInterpolatorValue {
    Number(f32),
    Color(u32),
}

impl RuntimeInterpolatorValue {
    fn from_converted_data_value(value: &RuntimeConvertedDataValue<'_>) -> Option<Self> {
        match value {
            RuntimeConvertedDataValue::Number(value) => Some(Self::Number(*value)),
            RuntimeConvertedDataValue::Color(value) => Some(Self::Color(*value)),
            _ => None,
        }
    }

    fn default_for_kind(&self) -> Self {
        match self {
            Self::Number(_) => Self::Number(0.0),
            Self::Color(_) => Self::Color(0),
        }
    }

    fn copy_from(&mut self, source: &Self) {
        if std::mem::discriminant(self) == std::mem::discriminant(source) {
            *self = source.clone();
        }
    }

    fn compare(&self, comparand: &Self) -> bool {
        self == comparand
    }

    fn interpolate(&self, to: &Self, factor: f32) -> Self {
        match (self, to) {
            (Self::Number(from), Self::Number(to)) => {
                Self::Number(*to * factor + *from * (1.0 - factor))
            }
            (Self::Color(from), Self::Color(to)) => Self::Color(cpp_color_lerp(*from, *to, factor)),
            _ => self.clone(),
        }
    }

    fn to_converted_data_value<'a>(&self) -> RuntimeConvertedDataValue<'a> {
        match self {
            Self::Number(value) => RuntimeConvertedDataValue::Number(*value),
            Self::Color(value) => RuntimeConvertedDataValue::Color(*value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeFormulaRandomSource<'a> {
    randoms: &'a [f32],
    next_random: usize,
    cached_randoms: Vec<f32>,
}

impl<'a> RuntimeFormulaRandomSource<'a> {
    pub fn new(randoms: &'a [f32]) -> Self {
        Self {
            randoms,
            next_random: 0,
            cached_randoms: Vec::new(),
        }
    }

    fn next(&mut self, random_mode: u64, random_index: usize) -> Option<f32> {
        if random_mode == 1 {
            return self.take_next();
        }

        while self.cached_randoms.len() <= random_index {
            let value = self.take_next()?;
            self.cached_randoms.push(value);
        }
        self.cached_randoms.get(random_index).copied()
    }

    fn take_next(&mut self) -> Option<f32> {
        let value = self.randoms.get(self.next_random).copied()?;
        self.next_random += 1;
        Some(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeGeneratedListItem {
    pub view_model_index: usize,
    pub view_model_id: u64,
    pub value_core_types: Vec<u16>,
}

impl<'a> From<&RuntimeDataValue<'a>> for RuntimeConvertedDataValue<'a> {
    fn from(value: &RuntimeDataValue<'a>) -> Self {
        match value {
            RuntimeDataValue::None => Self::None,
            RuntimeDataValue::Number(value) => Self::Number(*value),
            RuntimeDataValue::String(value) => Self::String(value.to_vec()),
            RuntimeDataValue::Boolean(value) => Self::Boolean(*value),
            RuntimeDataValue::Color(value) => Self::Color(*value),
            RuntimeDataValue::Enum { value, data_enum } => Self::Enum {
                value: *value,
                data_enum: data_enum.clone(),
            },
            RuntimeDataValue::Trigger(value) => Self::Trigger(*value),
            RuntimeDataValue::List(value) => Self::List(value.clone()),
            RuntimeDataValue::SymbolListIndex(value) => Self::SymbolListIndex(*value),
            RuntimeDataValue::AssetImage(value) => Self::AssetImage(*value),
            RuntimeDataValue::AssetFont(value) => Self::AssetFont(*value),
            RuntimeDataValue::Artboard(value) => Self::Artboard(*value),
            RuntimeDataValue::ViewModel(value) => Self::ViewModel(value.clone()),
        }
    }
}

impl RuntimeConvertedDataValue<'_> {
    pub fn data_type(&self) -> RuntimeDataType {
        match self {
            Self::None => RuntimeDataType::None,
            Self::Number(_) => RuntimeDataType::Number,
            Self::String(_) => RuntimeDataType::String,
            Self::Boolean(_) => RuntimeDataType::Boolean,
            Self::Color(_) => RuntimeDataType::Color,
            Self::Enum { .. } => RuntimeDataType::EnumType,
            Self::Integer(_) => RuntimeDataType::Integer,
            Self::Trigger(_) => RuntimeDataType::Trigger,
            Self::List(_) | Self::GeneratedList(_) => RuntimeDataType::List,
            Self::SymbolListIndex(_) => RuntimeDataType::SymbolListIndex,
            Self::AssetImage(_) => RuntimeDataType::AssetImage,
            Self::AssetFont(_) => RuntimeDataType::AssetFont,
            Self::Artboard(_) => RuntimeDataType::Artboard,
            Self::ViewModel(_) => RuntimeDataType::ViewModel,
        }
    }

    fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            _ => None,
        }
    }

    fn as_cpp_integer_super_value(&self) -> Option<u32> {
        match self {
            Self::Enum { value, .. }
            | Self::Integer(value)
            | Self::Trigger(value)
            | Self::AssetImage(value)
            | Self::AssetFont(value)
            | Self::Artboard(value) => Some(*value as u32),
            _ => None,
        }
    }

    fn list_len(&self) -> Option<usize> {
        match self {
            Self::List(items) => Some(items.len()),
            Self::GeneratedList(items) => Some(items.len()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeViewModelPropertyEnumData<'a> {
    name: &'a [u8],
    values: Vec<&'a RuntimeObject>,
}

#[derive(Debug, Clone)]
pub struct RuntimeViewModel<'a> {
    pub object: &'a RuntimeObject,
    pub properties: Vec<&'a RuntimeObject>,
    pub instances: Vec<RuntimeViewModelInstance<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeViewModelReference<'a> {
    pub view_model_index: usize,
    pub object: &'a RuntimeObject,
}

#[derive(Debug, Clone)]
pub struct RuntimeViewModelInstance<'a> {
    pub object: &'a RuntimeObject,
    pub values: Vec<RuntimeViewModelInstanceValue<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeViewModelInstanceValue<'a> {
    pub object: &'a RuntimeObject,
    pub list_items: Vec<&'a RuntimeObject>,
}

#[derive(Debug, Clone)]
pub struct RuntimeViewModelInstanceReference<'a> {
    pub view_model_index: usize,
    pub instance_index: usize,
    pub object: &'a RuntimeObject,
}

#[derive(Debug, Clone)]
pub struct RuntimeArtboardListMapRule<'a> {
    pub object: &'a RuntimeObject,
    pub view_model_id: u64,
    pub artboard_id: u64,
}

#[derive(Debug, Clone)]
pub struct RuntimeArtboardListItemArtboard<'a> {
    pub view_model_index: usize,
    pub instance_index: usize,
    pub artboard_index: usize,
    pub object: &'a RuntimeObject,
}

#[derive(Debug, Clone)]
pub struct RuntimeLinearAnimation<'a> {
    pub object: &'a RuntimeObject,
    pub keyed_objects: Vec<RuntimeKeyedObject<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachine<'a> {
    pub object: &'a RuntimeObject,
    pub layers: Vec<RuntimeStateMachineLayer<'a>>,
    pub inputs: Vec<&'a RuntimeObject>,
    pub listeners: Vec<RuntimeStateMachineListener<'a>>,
    pub data_binds: Vec<&'a RuntimeObject>,
    pub scripted_objects: Vec<RuntimeScriptedObject<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeScriptedObject<'a> {
    pub object: &'a RuntimeObject,
    pub inputs: Vec<&'a RuntimeObject>,
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachineLayer<'a> {
    pub object: &'a RuntimeObject,
    pub state_count: usize,
    pub states: Vec<RuntimeLayerState<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeLayerState<'a> {
    pub object: Option<&'a RuntimeObject>,
    pub animation: Option<&'a RuntimeObject>,
    pub blend_animations: Vec<RuntimeBlendAnimation<'a>>,
    pub fire_actions: Vec<RuntimeStateMachineFireAction<'a>>,
    pub listener_actions: Vec<RuntimeListenerAction<'a>>,
    pub transitions: Vec<RuntimeStateTransition<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachineFireAction<'a> {
    pub object: &'a RuntimeObject,
    pub event_local_index: Option<usize>,
    pub event: Option<&'a RuntimeObject>,
}

#[derive(Debug, Clone)]
pub struct RuntimeListenerAction<'a> {
    pub object: &'a RuntimeObject,
    pub event_local_index: Option<usize>,
    pub event: Option<&'a RuntimeObject>,
}

#[derive(Debug, Clone)]
pub struct RuntimeBlendAnimation<'a> {
    pub object: &'a RuntimeObject,
    pub animation_index: Option<usize>,
    pub animation: Option<&'a RuntimeObject>,
}

#[derive(Debug, Clone)]
pub struct RuntimeStateTransition<'a> {
    pub object: &'a RuntimeObject,
    pub state_to_index: Option<usize>,
    pub state_to: Option<&'a RuntimeObject>,
    pub interpolator: Option<&'a RuntimeObject>,
    pub exit_blend_animation_index: Option<usize>,
    pub exit_blend_animation: Option<&'a RuntimeObject>,
    pub exit_animation_index: Option<usize>,
    pub exit_animation: Option<&'a RuntimeObject>,
    pub fire_actions: Vec<RuntimeStateMachineFireAction<'a>>,
    pub listener_actions: Vec<RuntimeListenerAction<'a>>,
    pub conditions: Vec<&'a RuntimeObject>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeTransitionViewModelConditionComparators<'a> {
    pub left: Option<&'a RuntimeObject>,
    pub right: Option<&'a RuntimeObject>,
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachineListener<'a> {
    pub object: &'a RuntimeObject,
    pub actions: Vec<RuntimeListenerAction<'a>>,
    pub listener_input_types: Vec<&'a RuntimeObject>,
}

#[derive(Debug, Clone)]
pub struct RuntimeDataBind<'a> {
    pub object: &'a RuntimeObject,
    pub converter: Option<&'a RuntimeObject>,
    pub target: Option<&'a RuntimeObject>,
    pub target_local_id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct RuntimeSkin<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
    pub skinnable_local_id: Option<usize>,
    pub skinnable: Option<&'a RuntimeObject>,
    pub tendons: Vec<RuntimeTendon<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeTendon<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
    pub bone_local_id: Option<usize>,
    pub bone: Option<&'a RuntimeObject>,
}

#[derive(Debug, Clone)]
pub struct RuntimeShape<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
    pub paths: Vec<RuntimePath<'a>>,
    pub paints: Vec<RuntimeShapePaint<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeArtboardGeometry<'a> {
    pub meshes: Vec<RuntimeMesh<'a>>,
    pub paths: Vec<RuntimePath<'a>>,
    pub shapes: Vec<RuntimeShape<'a>>,
    pub shape_paint_containers: Vec<RuntimeShapePaintContainer<'a>>,
    pub n_slicer_details: Vec<RuntimeNSlicerDetails<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeShapePaintContainer<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
    pub paints: Vec<RuntimeShapePaint<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeShapePaint<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
    pub mutator_local_id: Option<usize>,
    pub mutator: Option<&'a RuntimeObject>,
    pub gradient_stops: Vec<RuntimeGradientStop<'a>>,
    pub feather_local_id: Option<usize>,
    pub feather: Option<&'a RuntimeObject>,
    pub effects: Vec<RuntimeStrokeEffect<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeGradientStop<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
}

#[derive(Debug, Clone)]
pub struct RuntimeStrokeEffect<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
    pub target_group_effect_local_id: Option<usize>,
    pub target_group_effect: Option<&'a RuntimeObject>,
    pub group_effects: Vec<RuntimeStrokeEffect<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeNSlicerDetails<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
    pub x_axes: Vec<RuntimeNSlicerAxis<'a>>,
    pub y_axes: Vec<RuntimeNSlicerAxis<'a>>,
    pub tile_modes: Vec<RuntimeNSlicerTileMode<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeNSlicerAxis<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
}

#[derive(Debug, Clone)]
pub struct RuntimeNSlicerTileMode<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
    pub patch_index: u64,
    pub style: u64,
}

#[derive(Debug, Clone)]
pub struct RuntimeMesh<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
    pub vertices: Vec<RuntimeMeshVertex<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeMeshVertex<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
    pub weight_local_id: Option<usize>,
    pub weight: Option<&'a RuntimeObject>,
}

#[derive(Debug, Clone)]
pub struct RuntimePath<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
    pub vertices: Vec<RuntimePathVertex<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimePathVertex<'a> {
    pub local_id: usize,
    pub object: &'a RuntimeObject,
    pub weight_local_id: Option<usize>,
    pub weight: Option<&'a RuntimeObject>,
}

#[derive(Debug, Clone)]
pub struct RuntimeDataBindPath<'a> {
    pub object: Option<&'a RuntimeObject>,
    pub property_name: &'static str,
    pub path_ids: Vec<u32>,
    pub resolved_path_ids: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct RuntimeDataConverterGroupItem<'a> {
    pub object: &'a RuntimeObject,
    pub converter: Option<&'a RuntimeObject>,
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeFormulaOutputToken<'a> {
    pub object: &'a RuntimeObject,
    pub arguments_count: usize,
}

impl<'a> RuntimeFormulaOutputToken<'a> {
    fn new(object: &'a RuntimeObject, arguments_count: &BTreeMap<u32, usize>) -> Self {
        Self {
            object,
            arguments_count: arguments_count.get(&object.id).copied().unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyedObject<'a> {
    pub object: &'a RuntimeObject,
    pub keyed_properties: Vec<RuntimeKeyedProperty<'a>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyedProperty<'a> {
    pub object: &'a RuntimeObject,
    pub first_key_frame: Option<&'a RuntimeObject>,
}

impl RuntimeManifest {
    pub fn resolve_name(&self, id: u32) -> Option<&str> {
        self.names
            .get(&cpp_manifest_resolver_key(id))
            .and_then(StringValue::as_str)
    }

    pub fn resolve_name_bytes(&self, id: u32) -> Option<&[u8]> {
        self.names
            .get(&cpp_manifest_resolver_key(id))
            .map(StringValue::as_bytes)
    }

    pub fn resolve_path(&self, id: u32) -> Option<&[u32]> {
        self.paths
            .get(&cpp_manifest_resolver_key(id))
            .map(Vec::as_slice)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum RuntimeImportStatus {
    NullObject,
    Imported,
    Dropped { reason: RuntimeImportDropReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeImportDropReason {
    MissingObject,
    InvalidObject,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeHeader {
    pub major_version: u64,
    pub minor_version: u64,
    pub file_id: u64,
    pub property_field_ids: BTreeMap<u32, HeaderFieldKind>,
}

impl RuntimeHeader {
    fn read(reader: &mut BinaryReader<'_>) -> Result<Self> {
        let fingerprint = reader.read_bytes_exact(4)?;
        if fingerprint != b"RIVE" {
            bail!("bad Rive fingerprint");
        }

        let major_version = read_cpp_int_var_uint(reader, "major version")?;
        let minor_version = read_cpp_int_var_uint(reader, "minor version")?;
        let file_id = read_cpp_int_var_uint(reader, "file id")?;

        let mut property_keys = Vec::new();
        loop {
            let property_key = read_cpp_int_var_uint(reader, "property key")?;
            if property_key == 0 {
                break;
            }
            property_keys.push(property_key as u32);
        }

        let mut property_field_ids = BTreeMap::new();
        let mut current_int = 0;
        let mut current_bit = 8;

        for property_key in property_keys {
            if current_bit == 8 {
                current_int = reader.read_u32()?;
                current_bit = 0;
            }

            let field_id = ((current_int >> current_bit) & 3) as u8;
            property_field_ids.insert(property_key, HeaderFieldKind::from_header_id(field_id));
            current_bit += 2;
        }

        Ok(Self {
            major_version,
            minor_version,
            file_id,
            property_field_ids,
        })
    }

    fn field_for_property(&self, key: u16) -> Option<HeaderFieldKind> {
        self.property_field_ids.get(&u32::from(key)).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HeaderFieldKind {
    Uint,
    StringOrBytes,
    Double,
    Color,
}

impl HeaderFieldKind {
    fn from_header_id(id: u8) -> Self {
        match id {
            0 => Self::Uint,
            1 => Self::StringOrBytes,
            2 => Self::Double,
            3 => Self::Color,
            _ => unreachable!("header field ids are packed into two bits"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeObject {
    pub id: u32,
    pub type_key: u16,
    pub type_name: &'static str,
    pub rust_variant: &'static str,
    pub properties: Vec<RuntimeProperty>,
    pub skipped_properties: Vec<SkippedProperty>,
}

impl RuntimeObject {
    pub fn property(&self, name: &str) -> Option<&RuntimeProperty> {
        self.properties
            .iter()
            .rev()
            .find(|property| property.name == name)
    }

    pub fn skipped_property(&self, name: &str) -> Option<&SkippedProperty> {
        self.skipped_properties
            .iter()
            .find(|property| property.name == Some(name))
    }

    pub fn string_property(&self, name: &str) -> Option<&str> {
        if let Some(property) = self.property(name) {
            return property.value.as_string();
        }

        match self.stored_field_initializer(name)? {
            StoredFieldInitializer::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn string_property_bytes(&self, name: &str) -> Option<&[u8]> {
        if let Some(property) = self.property(name) {
            return property.value.as_string_bytes();
        }

        match self.stored_field_initializer(name)? {
            StoredFieldInitializer::String(value) => Some(value.as_bytes()),
            _ => None,
        }
    }

    pub fn bytes_property(&self, name: &str) -> Option<&[u8]> {
        self.property(name)
            .and_then(|property| property.value.as_bytes())
    }

    pub fn id_list_property(&self, name: &str) -> Option<Vec<u32>> {
        let definition = definition_by_type_key(self.type_key)?;
        let property = property_by_name_in_hierarchy(definition, name)?;
        if property.declared_type != "List<Id>"
            || property.runtime_type != FieldKind::Bytes
            || !property.encoded
        {
            return None;
        }

        self.bytes_property(name).map(decode_cpp_u32_id_list)
    }

    pub fn data_bind_path_ids(&self) -> Option<Vec<u32>> {
        if self.type_name == "DataBindPath" {
            return self.data_bind_path_ids_property("path");
        }

        None
    }

    pub fn data_bind_path_ids_property(&self, name: &str) -> Option<Vec<u32>> {
        self.id_list_property(name)
    }

    pub fn mesh_triangle_indices(&self) -> Option<Vec<u16>> {
        if self.type_name != "Mesh" {
            return None;
        }

        self.bytes_property("triangleIndexBytes")
            .map(decode_cpp_mesh_triangle_indices)
    }

    pub fn file_asset_cdn_uuid_string(&self) -> Option<String> {
        let definition = definition_by_type_key(self.type_key)?;
        if !definition.is_a("FileAsset") {
            return None;
        }

        Some(format_cpp_file_asset_cdn_uuid(
            self.bytes_property("cdnUuid").unwrap_or(&[]),
        ))
    }

    pub fn file_asset_extension(&self) -> Option<&'static str> {
        cpp_file_asset_extension(self.type_name)
    }

    pub fn file_asset_unique_name(&self) -> Option<String> {
        String::from_utf8(self.file_asset_unique_name_bytes()?).ok()
    }

    pub fn file_asset_unique_name_bytes(&self) -> Option<Vec<u8>> {
        self.file_asset_extension()?;
        let name = self.string_property_bytes("name").unwrap_or_default();
        let stem_end = name
            .iter()
            .rposition(|byte| *byte == b'.')
            .unwrap_or(name.len());
        let mut unique_name = name[..stem_end].to_vec();
        unique_name.extend_from_slice(b"-");
        unique_name.extend_from_slice(
            self.uint_property("assetId")
                .unwrap_or(0)
                .to_string()
                .as_bytes(),
        );
        Some(unique_name)
    }

    pub fn file_asset_unique_filename(&self) -> Option<String> {
        String::from_utf8(self.file_asset_unique_filename_bytes()?).ok()
    }

    pub fn file_asset_unique_filename_bytes(&self) -> Option<Vec<u8>> {
        let extension = self.file_asset_extension()?;
        let mut filename = self.file_asset_unique_name_bytes()?;
        filename.extend_from_slice(b".");
        filename.extend_from_slice(extension.as_bytes());
        Some(filename)
    }

    pub fn uint_property(&self, name: &str) -> Option<u64> {
        if let Some(property) = self.property(name) {
            return property.value.as_uint();
        }

        if let Some(value) = self.bitmask_passthrough_value(name) {
            return Some(value);
        }

        match self.stored_field_initializer(name)? {
            StoredFieldInitializer::Uint(value) => Some(value),
            _ => None,
        }
    }

    pub fn bool_property(&self, name: &str) -> Option<bool> {
        if let Some(property) = self.property(name) {
            return property.value.as_bool();
        }

        if let Some(value) = self.bitmask_passthrough_value(name) {
            return Some(value != 0);
        }

        match self.stored_field_initializer(name)? {
            StoredFieldInitializer::Bool(value) => Some(value),
            _ => None,
        }
    }

    pub fn color_property(&self, name: &str) -> Option<u32> {
        if let Some(property) = self.property(name) {
            return property.value.as_color();
        }

        match self.stored_field_initializer(name)? {
            StoredFieldInitializer::Color(value) => Some(value),
            _ => None,
        }
    }

    pub fn double_property(&self, name: &str) -> Option<f32> {
        if let Some(property) = self.property(name) {
            return property.value.as_double();
        }

        match self.stored_field_initializer(name)? {
            StoredFieldInitializer::Double(value) => Some(value),
            _ => None,
        }
    }

    fn stored_field_initializer(&self, name: &str) -> Option<StoredFieldInitializer> {
        // C++ Artboard::Artboard overrides the inherited LayoutComponent
        // default so artboards clip to their bounds unless serialized otherwise.
        if self.type_name == "Artboard" && name == "clip" {
            return Some(StoredFieldInitializer::Bool(true));
        }

        let definition = definition_by_type_key(self.type_key)?;
        let property = property_by_name_in_hierarchy(definition, name)?;
        (*property).stored_field_initializer()
    }

    fn bitmask_passthrough_value(&self, name: &str) -> Option<u64> {
        let definition = definition_by_type_key(self.type_key)?;
        let property = property_by_name_in_hierarchy(definition, name)?;
        let passthrough = property.bitmask_passthrough?;
        let target = self.uint_property(passthrough.target)?;
        let mask = 1u64
            .wrapping_shl(u32::from(passthrough.width))
            .wrapping_sub(1);
        Some(target.wrapping_shr(u32::from(passthrough.bit)) & mask)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeProperty {
    pub key: u16,
    pub name: &'static str,
    pub owner: &'static str,
    pub value: FieldValue,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkippedProperty {
    pub key: u16,
    pub name: Option<&'static str>,
    pub owner: Option<&'static str>,
    pub reason: SkipReason,
    pub field: Option<&'static str>,
    pub value: Option<FieldValue>,
    pub bitmask_passthrough: Option<SkippedBitmaskPassthrough>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SkippedBitmaskPassthrough {
    pub target: &'static str,
    pub bit: u8,
    pub width: u8,
}

impl From<BitmaskPassthrough> for SkippedBitmaskPassthrough {
    fn from(value: BitmaskPassthrough) -> Self {
        Self {
            target: value.target,
            bit: value.bit,
            width: value.width,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SkipReason {
    UnknownObject,
    UnknownProperty,
    NonStoredProperty,
    PassthroughProperty,
    BitmaskPassthroughProperty,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum FieldValue {
    Bool(bool),
    Bytes(BytesValue),
    Callback,
    Color(u32),
    Double(f32),
    String(StringValue),
    Uint(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BytesValue {
    pub len: usize,
    pub preview_hex: String,
    pub raw: Vec<u8>,
}

impl BytesValue {
    pub fn new(raw: Vec<u8>) -> Self {
        Self {
            len: raw.len(),
            preview_hex: preview_hex(&raw),
            raw,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.raw
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StringValue {
    pub value: Option<String>,
    pub raw: Vec<u8>,
}

impl StringValue {
    pub fn as_str(&self) -> Option<&str> {
        self.value.as_deref()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.raw
    }
}

impl FieldValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(value) => value.as_str(),
            _ => None,
        }
    }

    pub fn as_string_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::String(value) => Some(value.as_bytes()),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(value) => Some(value.as_bytes()),
            _ => None,
        }
    }

    pub fn as_uint(&self) -> Option<u64> {
        match self {
            Self::Uint(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_color(&self) -> Option<u32> {
        match self {
            Self::Color(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_double(&self) -> Option<f32> {
        match self {
            Self::Double(value) => Some(*value),
            _ => None,
        }
    }
}

pub fn read_runtime_file(bytes: &[u8]) -> Result<RuntimeFile> {
    read_runtime_file_with_error_kind(bytes).map_err(anyhow::Error::new)
}

/// Reads a runtime file with the FileAsset importers provided by a
/// scripting-enabled Rive build.
///
/// [`read_runtime_file`] deliberately remains the non-scripting C++
/// conformance profile. Callers that will execute scripts must opt into this
/// profile so adjacent `FileAssetContents` records are retained for
/// `ScriptAsset` and `ShaderAsset` records.
pub fn read_runtime_file_with_scripting(bytes: &[u8]) -> Result<RuntimeFile> {
    read_runtime_file_with_profile(bytes, true).map_err(anyhow::Error::new)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeReadErrorKind {
    UnsupportedVersion,
    Malformed,
}

#[derive(Debug, Clone)]
pub struct RuntimeReadError {
    kind: RuntimeReadErrorKind,
    message: String,
}

impl RuntimeReadError {
    pub fn kind(&self) -> RuntimeReadErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    fn unsupported_version(message: String) -> Self {
        Self {
            kind: RuntimeReadErrorKind::UnsupportedVersion,
            message,
        }
    }

    fn malformed(error: anyhow::Error) -> Self {
        Self {
            kind: RuntimeReadErrorKind::Malformed,
            message: error.to_string(),
        }
    }
}

impl fmt::Display for RuntimeReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RuntimeReadError {}

pub fn read_runtime_file_with_error_kind(
    bytes: &[u8],
) -> std::result::Result<RuntimeFile, RuntimeReadError> {
    read_runtime_file_with_profile(bytes, false)
}

fn read_runtime_file_with_profile(
    bytes: &[u8],
    script_assets_create_importers: bool,
) -> std::result::Result<RuntimeFile, RuntimeReadError> {
    let mut reader = BinaryReader::new(bytes);
    let header = RuntimeHeader::read(&mut reader).map_err(RuntimeReadError::malformed)?;

    if header.major_version != SUPPORTED_MAJOR_VERSION {
        return Err(RuntimeReadError::unsupported_version(format!(
            "unsupported major version {}.{}; expected {}.{}",
            header.major_version,
            header.minor_version,
            SUPPORTED_MAJOR_VERSION,
            SUPPORTED_MINOR_VERSION
        )));
    }

    let mut objects = Vec::new();
    while !reader.reached_end() {
        let id = objects.len() as u32;
        let object = read_runtime_object(&mut reader, &header, id)
            .with_context(|| format!("reading object {id}"))
            .map_err(RuntimeReadError::malformed)?;
        objects.push(object);
    }

    finalize_runtime_file_with_script_assets(header, objects, script_assets_create_importers)
        .map_err(RuntimeReadError::malformed)
}

fn authoring_record_to_runtime_object(
    id: usize,
    record: AuthoringRecord,
    property_field_ids: &mut BTreeMap<u32, HeaderFieldKind>,
) -> Result<RuntimeObject> {
    let definition = definition_by_type_key(record.type_key)
        .with_context(|| format!("unknown authoring object type key {}", record.type_key))?;
    if definition.abstract_ {
        bail!(
            "authoring object type key {} ({}) is abstract",
            record.type_key,
            definition.name
        );
    }

    let object_id = u32::try_from(id).context("authoring object id does not fit in u32")?;
    let mut seen_property_keys = BTreeSet::new();
    let mut properties = Vec::with_capacity(record.properties.len());
    let mut authored_properties = record.properties;
    authored_properties.sort_by_key(|property| property.key);
    for authored_property in authored_properties {
        if !seen_property_keys.insert(authored_property.key) {
            bail!(
                "duplicate authoring property key {} on object {} ({})",
                authored_property.key,
                object_id,
                definition.name
            );
        }

        let (owner, property) =
            property_by_primary_key_in_hierarchy(definition, authored_property.key).with_context(
                || {
                    format!(
                        "authoring property key {} is not allowed on object {} ({})",
                        authored_property.key, object_id, definition.name
                    )
                },
            )?;
        if !property.deserializes {
            bail!(
                "authoring property key {} on object {} ({}) is not deserializable",
                authored_property.key,
                object_id,
                definition.name
            );
        }
        if !authored_property
            .value
            .matches_field_kind(property.runtime_type)
        {
            bail!(
                "authoring property key {} on object {} ({}) expects {:?}, got {}",
                authored_property.key,
                object_id,
                definition.name,
                property.runtime_type,
                authored_property.value.kind_name()
            );
        }
        if property.uint_storage() != Some(UintStorage::Uint64)
            && let AuthoringValue::Uint(value) = &authored_property.value
            && u32::try_from(*value).is_err()
        {
            bail!(
                "authoring property key {} on object {} ({}) uint value {} does not fit in C++ unsigned int",
                authored_property.key,
                object_id,
                definition.name,
                value
            );
        }

        if let Some(header_kind) = header_field_kind_for_property(authored_property.key, property)?
        {
            let property_key = u32::from(authored_property.key);
            if let Some(existing) = property_field_ids.insert(property_key, header_kind)
                && existing != header_kind
            {
                bail!(
                    "authoring property key {} has conflicting runtime field kinds",
                    authored_property.key
                );
            }
        }

        let mut value = authored_property.value.into_field_value();
        if property.uint_storage() == Some(UintStorage::Uint8)
            && let FieldValue::Uint(uint) = &mut value
        {
            *uint = u64::from(*uint as u8);
        }

        properties.push(RuntimeProperty {
            key: authored_property.key,
            name: property.name,
            owner,
            value,
        });
    }

    Ok(RuntimeObject {
        id: object_id,
        type_key: record.type_key,
        type_name: definition.name,
        rust_variant: definition.rust_variant,
        properties,
        skipped_properties: Vec::new(),
    })
}

impl AuthoringValue {
    fn matches_field_kind(&self, kind: FieldKind) -> bool {
        matches!(
            (self, kind),
            (Self::Bool(_), FieldKind::Bool)
                | (Self::Bytes(_), FieldKind::Bytes)
                | (Self::Color(_), FieldKind::Color)
                | (Self::Double(_), FieldKind::Double)
                | (Self::String(_), FieldKind::String)
                | (Self::Uint(_), FieldKind::Uint)
        )
    }

    fn kind_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Bytes(_) => "bytes",
            Self::Color(_) => "color",
            Self::Double(_) => "double",
            Self::String(_) => "string",
            Self::Uint(_) => "uint",
        }
    }

    fn into_field_value(self) -> FieldValue {
        match self {
            Self::Bool(value) => FieldValue::Bool(value),
            Self::Bytes(value) => FieldValue::Bytes(BytesValue::new(value)),
            Self::Color(value) => FieldValue::Color(value),
            Self::Double(value) => FieldValue::Double(value),
            Self::String(value) => {
                let raw = value.as_bytes().to_vec();
                FieldValue::String(StringValue {
                    value: Some(value),
                    raw,
                })
            }
            Self::Uint(value) => FieldValue::Uint(value),
        }
    }
}

fn header_field_kind_for_property(
    key: u16,
    property: &Property,
) -> Result<Option<HeaderFieldKind>> {
    let Some(core_kind) = core_registry_field_kind_by_property_key(key) else {
        return Ok(None);
    };

    let header_kind = match (property.runtime_type, core_kind) {
        (FieldKind::Uint, CoreRegistryFieldKind::Uint) => Some(HeaderFieldKind::Uint),
        (FieldKind::String, CoreRegistryFieldKind::StringOrBytes) => {
            Some(HeaderFieldKind::StringOrBytes)
        }
        (FieldKind::Double, CoreRegistryFieldKind::Double) => Some(HeaderFieldKind::Double),
        (FieldKind::Color, CoreRegistryFieldKind::Color) => Some(HeaderFieldKind::Color),
        (FieldKind::Bytes, CoreRegistryFieldKind::StringOrBytes)
        | (FieldKind::Bool, CoreRegistryFieldKind::Bool) => None,
        (runtime_kind, registry_kind) => {
            bail!(
                "authoring property key {key} has conflicting schema field kind {runtime_kind:?} and core-registry field kind {registry_kind:?}"
            )
        }
    };
    Ok(header_kind)
}

fn finalize_runtime_file_with_script_assets(
    header: RuntimeHeader,
    mut objects: Vec<Option<RuntimeObject>>,
    script_assets_create_importers: bool,
) -> Result<RuntimeFile> {
    let import_statuses = compute_import_statuses(&objects, script_assets_create_importers);
    validate_cpp_import_resolution(&objects, &import_statuses)?;
    apply_cpp_import_mutations(&mut objects, &import_statuses);

    Ok(RuntimeFile {
        header,
        objects,
        import_statuses,
    })
}

fn validate_authoring_import_statuses(file: &RuntimeFile) -> Result<()> {
    for (id, (object, status)) in file
        .objects
        .iter()
        .zip(file.import_statuses.iter())
        .enumerate()
    {
        match status {
            RuntimeImportStatus::Imported => {}
            RuntimeImportStatus::NullObject => {
                bail!("authored object {id} would import as a null object")
            }
            RuntimeImportStatus::Dropped { reason } => {
                let type_name = object
                    .as_ref()
                    .map(|object| object.type_name)
                    .unwrap_or("unknown");
                bail!(
                    "authored object {id} ({type_name}) would be dropped during import: {reason:?}"
                );
            }
        }
    }

    if file.objects.len() != file.import_statuses.len() {
        bail!("authored object/import-status counts do not match");
    }

    Ok(())
}

fn validate_authoring_artboard_local_objects(file: &RuntimeFile) -> Result<()> {
    for range in runtime_artboard_ranges(&file.objects) {
        let original_slots =
            runtime_artboard_local_slots(&file.objects, &file.import_statuses, range);
        let mut validated_slots = original_slots.clone();
        validate_cpp_artboard_local_slots(&mut validated_slots, &file.objects);

        for (local_id, (original, validated)) in original_slots
            .iter()
            .zip(validated_slots.iter())
            .enumerate()
        {
            let Some(file_id) = *original else {
                continue;
            };
            if validated.is_some() {
                continue;
            }

            let type_name = file
                .object(file_id)
                .map(|object| object.type_name)
                .unwrap_or("unknown");
            bail!(
                "authored object {file_id} ({type_name}) is an invalid artboard-local object at local id {local_id}"
            );
        }
    }

    Ok(())
}

#[derive(Default)]
struct ImportContext {
    import_stack: CppImportStack,
    latest_layer_state_accepts_blend_animation: bool,
    state_machine_inputs: Vec<Option<StateMachineInputKind>>,
    artboard_local_nested_inputs: Vec<Option<StateMachineInputKind>>,
}

#[derive(Default)]
struct CppImportStack {
    latest: BTreeSet<ImportStackKey>,
    last_added: Vec<ImportStackKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ImportStackKey {
    Backboard,
    Artboard,
    FileAsset,
    LinearAnimation,
    KeyedObject,
    KeyedProperty,
    StateMachine,
    StateMachineLayer,
    LayerState,
    StateTransition,
    StateMachineLayerComponent,
    StateMachineListener,
    ListenerInputTypeGamepad,
    ListenerInputTypeKeyboard,
    ListenerInputTypeSemantic,
    DataEnumCustom,
    ViewModel,
    ViewModelInstance,
    ViewModelInstanceList,
    TransitionViewModelCondition,
    BindableProperty,
    DataConverterGroup,
    DataConverterFormula,
    DataBindPath,
    ScriptedObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StateMachineInputKind {
    Bool,
    Number,
    Trigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NullObjectConsumer {
    Artboard,
    KeyedProperty,
    StateMachine,
    StateMachineLayer,
}

#[derive(Debug, Clone, Copy)]
struct CppDataBindTarget<'a> {
    file_index: usize,
    object: &'a RuntimeObject,
}

impl ImportContext {
    fn latest(&self, key: ImportStackKey) -> bool {
        self.import_stack.latest(key)
    }

    fn make_latest(&mut self, key: ImportStackKey) {
        self.import_stack.make_latest(key);
    }

    fn read_null_object(&mut self) {
        match self.import_stack.latest_null_object_consumer() {
            Some(NullObjectConsumer::Artboard) => {
                self.artboard_local_nested_inputs.push(None);
            }
            Some(NullObjectConsumer::StateMachine) => self.state_machine_inputs.push(None),
            _ => {}
        }
    }

    fn read_dropped_object(&mut self, definition: &'static Definition) {
        if definition_is_cpp_artboard_local(definition) {
            self.artboard_local_nested_inputs.push(None);
        }
    }
}

impl CppImportStack {
    fn latest(&self, key: ImportStackKey) -> bool {
        self.latest.contains(&key)
    }

    fn make_latest(&mut self, key: ImportStackKey) {
        if let Some(index) = self
            .last_added
            .iter()
            .rposition(|candidate| *candidate == key)
        {
            self.last_added.remove(index);
        }
        self.latest.insert(key);
        self.last_added.push(key);
    }

    fn latest_null_object_consumer(&self) -> Option<NullObjectConsumer> {
        self.last_added
            .iter()
            .rev()
            .find_map(|key| key.null_object_consumer())
    }
}

impl ImportStackKey {
    fn null_object_consumer(self) -> Option<NullObjectConsumer> {
        match self {
            Self::Artboard => Some(NullObjectConsumer::Artboard),
            Self::KeyedProperty => Some(NullObjectConsumer::KeyedProperty),
            Self::StateMachine => Some(NullObjectConsumer::StateMachine),
            Self::StateMachineLayer => Some(NullObjectConsumer::StateMachineLayer),
            _ => None,
        }
    }
}

fn compute_import_statuses(
    objects: &[Option<RuntimeObject>],
    script_assets_create_importers: bool,
) -> Vec<RuntimeImportStatus> {
    let mut context = ImportContext::default();
    objects
        .iter()
        .map(|object| {
            let Some(object) = object.as_ref() else {
                context.read_null_object();
                return RuntimeImportStatus::NullObject;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                context.read_null_object();
                return RuntimeImportStatus::NullObject;
            };

            if let Some(reason) = object_import_failure_reason(object, definition, &context) {
                context.read_dropped_object(definition);
                return RuntimeImportStatus::Dropped { reason };
            }

            update_import_context(
                object,
                definition,
                &mut context,
                script_assets_create_importers,
            );
            RuntimeImportStatus::Imported
        })
        .collect()
}

fn object_import_failure_reason(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<RuntimeImportDropReason> {
    if !object_imports_successfully(object, definition, context) {
        return Some(RuntimeImportDropReason::MissingObject);
    }

    if transition_input_condition_is_invalid(object, definition, context) {
        return Some(RuntimeImportDropReason::InvalidObject);
    }

    if listener_input_change_is_invalid(object, definition, context) {
        return Some(RuntimeImportDropReason::InvalidObject);
    }

    if blend_input_is_invalid(object, definition, context) {
        return Some(RuntimeImportDropReason::InvalidObject);
    }

    if definition.is_a("BlendAnimation") && !context.latest_layer_state_accepts_blend_animation {
        return Some(RuntimeImportDropReason::InvalidObject);
    }

    None
}

fn object_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> bool {
    match definition.name {
        "Backboard" | "DataEnum" | "DataEnumSystem" | "DataEnumCustom" | "ViewModel" => {
            return true;
        }
        "Artboard" => return context.latest(ImportStackKey::Backboard),
        "FileAssetContents" => return context.latest(ImportStackKey::FileAsset),
        "LinearAnimation" => return context.latest(ImportStackKey::Artboard),
        "KeyedObject" => return context.latest(ImportStackKey::LinearAnimation),
        "KeyedProperty" => return context.latest(ImportStackKey::KeyedObject),
        "StateMachine" => return context.latest(ImportStackKey::Artboard),
        "StateMachineLayer" => return context.latest(ImportStackKey::StateMachine),
        "BlendState1DViewModel" => {
            return context.latest(ImportStackKey::StateMachineLayer)
                && context.latest(ImportStackKey::BindableProperty);
        }
        "BlendAnimationDirect" => {
            if object.uint_property("blendSource") == Some(2)
                && !context.latest(ImportStackKey::BindableProperty)
            {
                return false;
            }
        }
        "ListenerViewModelChange" => {
            return context.latest(ImportStackKey::BindableProperty)
                && listener_action_imports_successfully(object, context);
        }
        "TransitionPropertyViewModelComparator" => {
            return context.latest(ImportStackKey::TransitionViewModelCondition)
                && context.latest(ImportStackKey::BindableProperty);
        }
        "ScriptInputArtboard" => {
            return context.latest(ImportStackKey::Backboard)
                && context.latest(ImportStackKey::ScriptedObject);
        }
        "GamepadInput" => {
            return context.latest(ImportStackKey::Artboard)
                && context.latest(ImportStackKey::ListenerInputTypeGamepad);
        }
        "KeyboardInput" => {
            return context.latest(ImportStackKey::Artboard)
                && context.latest(ImportStackKey::ListenerInputTypeKeyboard);
        }
        "SemanticInput" => {
            return context.latest(ImportStackKey::Artboard)
                && context.latest(ImportStackKey::ListenerInputTypeSemantic);
        }
        "DataEnumValue" => return context.latest(ImportStackKey::DataEnumCustom),
        "ViewModelInstance" => return context.latest(ImportStackKey::Backboard),
        "ViewModelInstanceListItem" => {
            return context.latest(ImportStackKey::ViewModelInstanceList);
        }
        "ViewModelInstanceAsset" | "ViewModelInstanceAssetImage" | "ViewModelInstanceAssetFont" => {
            return context.latest(ImportStackKey::Backboard)
                && context.latest(ImportStackKey::ViewModelInstance);
        }
        "DataConverterGroupItem" => {
            return context.latest(ImportStackKey::Backboard)
                && context.latest(ImportStackKey::DataConverterGroup);
        }
        _ => {}
    }

    if definition.name.starts_with("ScriptInput") {
        return context.latest(ImportStackKey::ScriptedObject);
    }

    if definition.is_a("FileAsset") {
        return definition.name == "ManifestAsset" || context.latest(ImportStackKey::Backboard);
    }

    if definition.is_a("KeyFrame") {
        return context.latest(ImportStackKey::KeyedProperty);
    }

    if definition.is_a("StateTransition") {
        return context.latest(ImportStackKey::LayerState);
    }

    if definition.is_a("TransitionCondition") {
        return context.latest(ImportStackKey::StateTransition);
    }

    if definition.is_a("TransitionComparator") {
        return context.latest(ImportStackKey::TransitionViewModelCondition);
    }

    if definition.is_a("StateMachineFireAction") {
        return context.latest(ImportStackKey::StateMachineLayerComponent);
    }

    if definition.is_a("StateMachineInput") || definition.is_a("StateMachineListener") {
        return context.latest(ImportStackKey::StateMachine);
    }

    if definition.is_a("LayerState") || definition.is_a("BlendAnimation") {
        return if definition.is_a("BlendAnimation") {
            context.latest(ImportStackKey::Artboard) && context.latest(ImportStackKey::LayerState)
        } else {
            context.latest(ImportStackKey::StateMachineLayer)
        };
    }

    if definition.is_a("ListenerAction") {
        return listener_action_imports_successfully(object, context);
    }

    if definition.is_a("ListenerInputType") {
        return context.latest(ImportStackKey::StateMachineListener);
    }

    if definition.is_a("ViewModelProperty") {
        return context.latest(ImportStackKey::ViewModel);
    }

    if definition.is_a("ViewModelInstanceValue") {
        return context.latest(ImportStackKey::ViewModelInstance);
    }

    if definition.is_a("DataBind")
        || definition.is_a("DataConverter")
        || definition.is_a("DataBindPath")
    {
        return context.latest(ImportStackKey::Backboard);
    }

    if definition.is_a("FormulaToken") {
        return context.latest(ImportStackKey::DataConverterFormula);
    }

    if definition.is_a("ScrollPhysics") {
        return context.latest(ImportStackKey::Backboard);
    }

    if definition.is_a("KeyFrameInterpolator") {
        return context.latest(ImportStackKey::Artboard)
            || context.latest(ImportStackKey::Backboard);
    }

    if definition.is_a("Component") {
        return context.latest(ImportStackKey::Artboard);
    }

    true
}

fn update_import_context(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &mut ImportContext,
    script_assets_create_importers: bool,
) {
    match definition.name {
        "Backboard" => context.make_latest(ImportStackKey::Backboard),
        "Artboard" => {
            context.artboard_local_nested_inputs.clear();
            context.make_latest(ImportStackKey::Artboard);
        }
        "LinearAnimation" => context.make_latest(ImportStackKey::LinearAnimation),
        "KeyedObject" => context.make_latest(ImportStackKey::KeyedObject),
        "KeyedProperty" => context.make_latest(ImportStackKey::KeyedProperty),
        "StateMachine" => {
            context.state_machine_inputs.clear();
            context.make_latest(ImportStackKey::StateMachine);
        }
        "StateMachineLayer" => context.make_latest(ImportStackKey::StateMachineLayer),
        "ListenerInputTypeGamepad" => {
            context.make_latest(ImportStackKey::ListenerInputTypeGamepad);
        }
        "ListenerInputTypeKeyboard" => {
            context.make_latest(ImportStackKey::ListenerInputTypeKeyboard);
        }
        "ListenerInputTypeSemantic" => {
            context.make_latest(ImportStackKey::ListenerInputTypeSemantic);
        }
        "ViewModel" => context.make_latest(ImportStackKey::ViewModel),
        "ViewModelInstance" => context.make_latest(ImportStackKey::ViewModelInstance),
        "ViewModelInstanceList" => context.make_latest(ImportStackKey::ViewModelInstanceList),
        "DataEnumCustom" => context.make_latest(ImportStackKey::DataEnumCustom),
        "DataConverterGroup" => context.make_latest(ImportStackKey::DataConverterGroup),
        "DataConverterFormula" => context.make_latest(ImportStackKey::DataConverterFormula),
        _ => {}
    }

    if file_asset_creates_importer(definition.name, script_assets_create_importers) {
        context.make_latest(ImportStackKey::FileAsset);
    }
    if definition.is_a("StateMachineLayerComponent") {
        context.make_latest(ImportStackKey::StateMachineLayerComponent);
    }
    if definition.is_a("StateTransition") {
        context.make_latest(ImportStackKey::StateTransition);
    }
    if definition.is_a("LayerState") {
        context.make_latest(ImportStackKey::LayerState);
        context.latest_layer_state_accepts_blend_animation = definition.is_a("BlendState");
    }
    if definition.is_a("StateMachineListener") {
        context.make_latest(ImportStackKey::StateMachineListener);
    }
    if let Some(kind) = state_machine_input_kind(definition) {
        context.state_machine_inputs.push(Some(kind));
    }
    if definition_is_cpp_artboard_local(definition) {
        context
            .artboard_local_nested_inputs
            .push(nested_input_kind(definition));
    }
    if definition.is_a("TransitionViewModelCondition") {
        context.make_latest(ImportStackKey::TransitionViewModelCondition);
    }
    if definition.is_a("BindableProperty") {
        context.make_latest(ImportStackKey::BindableProperty);
    }
    if definition.is_a("DataBindPath") {
        context.make_latest(ImportStackKey::DataBindPath);
    }
    if definition_is_cpp_scripted_object(definition) {
        context.make_latest(ImportStackKey::ScriptedObject);
    }

    let _ = object;
}

fn file_asset_creates_importer(type_name: &str, script_assets_create_importers: bool) -> bool {
    matches!(
        type_name,
        "ImageAsset" | "FontAsset" | "AudioAsset" | "BlobAsset" | "ManifestAsset"
    ) || (script_assets_create_importers && matches!(type_name, "ScriptAsset" | "ShaderAsset"))
}

fn cpp_file_assets_contains(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| {
        definition.is_a("FileAsset") && definition.name != "ManifestAsset"
    })
}

fn cpp_artboard_referencer_index(object: &RuntimeObject) -> Option<u64> {
    let definition = definition_by_type_key(object.type_key)?;
    (definition.is_a("NestedArtboard") || definition.name == "ScriptInputArtboard")
        .then(|| object.uint_property("artboardId"))
        .flatten()
}

fn cpp_data_bind_is_name_based(object: &RuntimeObject) -> bool {
    const DATA_BIND_NAME_BASED_FLAG: u64 = 1 << 4;

    object
        .uint_property("flags")
        .is_some_and(|flags| flags & DATA_BIND_NAME_BASED_FLAG != 0)
}

fn cpp_data_bind_binds_once(object: &RuntimeObject) -> bool {
    const DATA_BIND_ONCE_FLAG: u64 = 1 << 2;

    object
        .uint_property("flags")
        .is_some_and(|flags| flags & DATA_BIND_ONCE_FLAG != 0)
}

fn cpp_data_bind_is_main_to_source(object: &RuntimeObject) -> bool {
    const DATA_BIND_TO_SOURCE_FLAG: u64 = 1 << 0;

    object
        .uint_property("flags")
        .is_some_and(|flags| flags & DATA_BIND_TO_SOURCE_FLAG != 0)
}

fn cpp_data_bind_source_to_target_runs_first(object: &RuntimeObject) -> bool {
    const DATA_BIND_SOURCE_TO_TARGET_RUNS_FIRST_FLAG: u64 = 1 << 3;

    object
        .uint_property("flags")
        .is_some_and(|flags| flags & DATA_BIND_SOURCE_TO_TARGET_RUNS_FIRST_FLAG != 0)
}

fn cpp_data_bind_reconcile_dirt(data_bind: &RuntimeObject) -> RuntimeComponentDirt {
    let mut dirt = RuntimeComponentDirt::NONE;
    if cpp_data_bind_to_target(data_bind) {
        dirt |= RuntimeComponentDirt::BINDINGS;
    }
    if cpp_data_bind_to_source(data_bind) {
        dirt |= RuntimeComponentDirt::BINDINGS_TARGET;
    }
    dirt
}

fn cpp_data_bind_to_source(object: &RuntimeObject) -> bool {
    const DATA_BIND_TO_SOURCE_FLAG: u64 = 1 << 0;
    const DATA_BIND_TWO_WAY_FLAG: u64 = 1 << 1;

    object.uint_property("flags").unwrap_or(0) & (DATA_BIND_TO_SOURCE_FLAG | DATA_BIND_TWO_WAY_FLAG)
        != 0
}

fn cpp_data_bind_to_target(object: &RuntimeObject) -> bool {
    const DATA_BIND_TO_SOURCE_FLAG: u64 = 1 << 0;
    const DATA_BIND_TWO_WAY_FLAG: u64 = 1 << 1;

    let flags = object.uint_property("flags").unwrap_or(0);
    flags & DATA_BIND_TWO_WAY_FLAG != 0 || flags & DATA_BIND_TO_SOURCE_FLAG == 0
}

fn cpp_data_bind_can_skip(
    data_bind: &RuntimeObject,
    target: Option<&RuntimeObject>,
    target_is_collapsed: bool,
) -> bool {
    let Some(target) = target else {
        return false;
    };
    let Some(target_definition) = definition_by_type_key(target.type_key) else {
        return false;
    };
    let display_value_property_key = cpp_property_key("LayoutComponentStyle", "displayValue");

    target_definition.is_a("Component")
        && target_is_collapsed
        && data_bind.uint_property("propertyKey").unwrap_or(0) != display_value_property_key
}

fn cpp_data_bind_collapse_effect(
    data_bind: &RuntimeObject,
    target_supports_push: bool,
    is_collapsed: bool,
    requested_is_collapsed: bool,
    has_dirt: bool,
    has_container: bool,
) -> RuntimeDataBindCollapseEffect {
    let display_value_property_key = cpp_property_key("LayoutComponentStyle", "displayValue");
    if is_collapsed == requested_is_collapsed
        || data_bind.uint_property("propertyKey").unwrap_or(0) == display_value_property_key
        || !target_supports_push
    {
        return RuntimeDataBindCollapseEffect {
            is_collapsed,
            changed: false,
            requests_dirty_update: false,
        };
    }

    RuntimeDataBindCollapseEffect {
        is_collapsed: requested_is_collapsed,
        changed: true,
        requests_dirty_update: !requested_is_collapsed && has_dirt && has_container,
    }
}

fn cpp_data_bind_add_dirt_effect(
    current_dirt: RuntimeComponentDirt,
    current_target_origin: bool,
    added_dirt: RuntimeComponentDirt,
    source_to_target_runs_first: bool,
    suppress_dirt: bool,
    is_collapsed: bool,
    has_context_value: bool,
    has_container: bool,
) -> RuntimeDataBindAddDirtEffect {
    if suppress_dirt || current_dirt.contains(added_dirt) {
        return RuntimeDataBindAddDirtEffect {
            dirt: current_dirt,
            target_origin: current_target_origin,
            changed: false,
            invalidates_context_value: false,
            requests_dirty_update: false,
        };
    }

    let added_source_dirt = added_dirt.contains(RuntimeComponentDirt::BINDINGS);
    let added_target_dirt = added_dirt.contains(RuntimeComponentDirt::BINDINGS_TARGET);
    let target_origin = match (added_source_dirt, added_target_dirt) {
        (true, true) => !source_to_target_runs_first,
        (false, true) => true,
        (true, false) => false,
        (false, false) => current_target_origin,
    };
    let dirt = current_dirt | added_dirt;
    RuntimeDataBindAddDirtEffect {
        dirt,
        target_origin,
        changed: true,
        invalidates_context_value: dirt.contains(RuntimeComponentDirt::DEPENDENTS)
            && has_context_value,
        requests_dirty_update: has_container && !is_collapsed,
    }
}

fn cpp_data_bind_add_effect(
    data_bind: &RuntimeObject,
    target_supports_push: bool,
    is_data_bind_context: bool,
    is_processing: bool,
    has_data_context: bool,
) -> RuntimeDataBindAddEffect {
    if is_processing {
        return RuntimeDataBindAddEffect {
            queues_pending_addition: true,
            appends_to_data_binds: false,
            appends_to_persisting_list: false,
            sets_persisting_list_flag: false,
            sets_container: false,
            binds_from_data_context: false,
            runs_initial_update: false,
            initial_update_applies_target_to_source: false,
        };
    }

    let uses_persisting_list = cpp_data_bind_to_source(data_bind) && !target_supports_push;
    let binds_from_data_context = has_data_context && is_data_bind_context;
    RuntimeDataBindAddEffect {
        queues_pending_addition: false,
        appends_to_data_binds: true,
        appends_to_persisting_list: uses_persisting_list,
        sets_persisting_list_flag: uses_persisting_list,
        sets_container: true,
        binds_from_data_context,
        runs_initial_update: binds_from_data_context,
        initial_update_applies_target_to_source: binds_from_data_context,
    }
}

fn cpp_data_bind_source_effect(
    data_bind: &RuntimeObject,
    target: Option<&RuntimeObject>,
    source_data_type: RuntimeDataType,
) -> RuntimeDataBindSourceEffect {
    let target_is_artboard_component_list = target.is_some_and(|target| {
        definition_by_type_key(target.type_key)
            .is_some_and(|definition| definition.is_a("ArtboardComponentList"))
    });
    RuntimeDataBindSourceEffect {
        adds_source_dependent: !cpp_data_bind_binds_once(data_bind),
        sets_source: true,
        updates_artboard_component_list_reset: target_is_artboard_component_list,
        artboard_component_list_should_reset_instances: target_is_artboard_component_list
            && source_data_type == RuntimeDataType::Number,
    }
}

fn cpp_data_bind_clear_source_effect(
    data_bind: &RuntimeObject,
    has_source: bool,
) -> RuntimeDataBindClearSourceEffect {
    RuntimeDataBindClearSourceEffect {
        removes_source_dependent: has_source && !cpp_data_bind_binds_once(data_bind),
        clears_source: has_source,
    }
}

fn cpp_data_bind_effective_output_type(
    converter_output_type: Option<RuntimeDataType>,
    source_data_type: RuntimeDataType,
) -> RuntimeDataType {
    if let Some(converter_output_type) = converter_output_type {
        if converter_output_type != RuntimeDataType::Input
            && converter_output_type != RuntimeDataType::None
        {
            return converter_output_type;
        }
    }
    source_data_type
}

fn cpp_data_bind_context_value_type(output_type: RuntimeDataType) -> Option<RuntimeDataType> {
    match output_type {
        RuntimeDataType::Number
        | RuntimeDataType::String
        | RuntimeDataType::Boolean
        | RuntimeDataType::Color
        | RuntimeDataType::EnumType
        | RuntimeDataType::List
        | RuntimeDataType::Trigger
        | RuntimeDataType::SymbolListIndex
        | RuntimeDataType::AssetImage
        | RuntimeDataType::AssetFont
        | RuntimeDataType::Artboard
        | RuntimeDataType::ViewModel
        | RuntimeDataType::Any => Some(output_type),
        RuntimeDataType::None | RuntimeDataType::Integer | RuntimeDataType::Input => None,
    }
}

fn cpp_data_bind_bind_effect(
    data_bind: &RuntimeObject,
    has_converter: bool,
    converter_output_type: Option<RuntimeDataType>,
    source_data_type: RuntimeDataType,
    has_target: bool,
    is_observing: bool,
    has_context_value: bool,
    current_dirt: RuntimeComponentDirt,
    current_target_origin: bool,
    is_collapsed: bool,
    has_container: bool,
    target_supports_push: bool,
) -> RuntimeDataBindBindEffect {
    let context_value_type = cpp_data_bind_context_value_type(cpp_data_bind_effective_output_type(
        converter_output_type,
        source_data_type,
    ));
    let removes_existing_target_observer = is_observing && has_target;
    let adds_target_observer =
        cpp_data_bind_to_source(data_bind) && has_target && target_supports_push;
    RuntimeDataBindBindEffect {
        clears_existing_context_value: has_context_value,
        context_value_type,
        resets_converter: has_converter,
        removes_existing_target_observer,
        adds_target_observer,
        observing_after: if adds_target_observer {
            true
        } else if removes_existing_target_observer {
            false
        } else {
            is_observing
        },
        add_dirt_effect: cpp_data_bind_add_dirt_effect(
            current_dirt,
            current_target_origin,
            cpp_data_bind_reconcile_dirt(data_bind),
            cpp_data_bind_source_to_target_runs_first(data_bind),
            false,
            is_collapsed,
            context_value_type.is_some(),
            has_container,
        ),
    }
}

fn cpp_data_bind_target_effect(
    data_bind: &RuntimeObject,
    current_target_is_same: bool,
    has_current_target: bool,
    has_new_target: bool,
    is_observing: bool,
    new_target_supports_push: bool,
) -> RuntimeDataBindTargetEffect {
    if current_target_is_same {
        return RuntimeDataBindTargetEffect {
            unchanged: true,
            removes_existing_target_observer: false,
            sets_target: false,
            adds_target_observer: false,
            observing_after: is_observing,
        };
    }

    let removes_existing_target_observer = is_observing && has_current_target;
    let adds_target_observer =
        cpp_data_bind_to_source(data_bind) && has_new_target && new_target_supports_push;
    RuntimeDataBindTargetEffect {
        unchanged: false,
        removes_existing_target_observer,
        sets_target: true,
        adds_target_observer,
        observing_after: if adds_target_observer {
            true
        } else if removes_existing_target_observer {
            false
        } else {
            is_observing
        },
    }
}

fn cpp_data_bind_unbind_effect(
    data_bind: &RuntimeObject,
    has_converter: bool,
    has_source: bool,
    has_target: bool,
    is_observing: bool,
    has_context_value: bool,
) -> RuntimeDataBindUnbindEffect {
    let removes_target_observer = is_observing && has_target;
    RuntimeDataBindUnbindEffect {
        clear_source_effect: cpp_data_bind_clear_source_effect(data_bind, has_source),
        removes_target_observer,
        observing_after: if removes_target_observer {
            false
        } else {
            is_observing
        },
        unbinds_converter: has_converter,
        clears_context_value: has_context_value,
    }
}

fn cpp_data_bind_initialize_effect(
    data_bind: &RuntimeObject,
    target: Option<&RuntimeObject>,
    target_supports_push: bool,
    already_collapsable: bool,
    is_collapsed: bool,
    target_is_collapsed: bool,
    has_dirt: bool,
    has_container: bool,
) -> RuntimeDataBindInitializeEffect {
    let target_is_component = target.is_some_and(|target| {
        definition_by_type_key(target.type_key)
            .is_some_and(|definition| definition.is_a("Component"))
    });
    let adds_component_collapsable = target_is_component && !already_collapsable;
    RuntimeDataBindInitializeEffect {
        target_is_component,
        adds_component_collapsable,
        runs_collapse: adds_component_collapsable,
        collapse_effect: adds_component_collapsable.then(|| {
            cpp_data_bind_collapse_effect(
                data_bind,
                target_supports_push,
                is_collapsed,
                target_is_collapsed,
                has_dirt,
                has_container,
            )
        }),
    }
}

fn cpp_data_bind_relink_effect(
    is_data_bind_context: bool,
    has_container: bool,
    has_data_context: bool,
) -> RuntimeDataBindRelinkEffect {
    RuntimeDataBindRelinkEffect {
        calls_container_rebuild: has_container,
        rebuild_binds_from_context: has_container && is_data_bind_context,
        rebuild_has_data_context: has_container && is_data_bind_context && has_data_context,
    }
}

#[allow(clippy::too_many_arguments)]
fn cpp_data_bind_context_bind_effect(
    data_bind: &RuntimeObject,
    target: Option<&RuntimeObject>,
    has_converter: bool,
    converter_output_type: Option<RuntimeDataType>,
    source_path_ids: &[u32],
    resolved_source_path_ids: &[u32],
    source_path_is_resolved: bool,
    has_data_context: bool,
    lookup_has_source: bool,
    source_matches_lookup: bool,
    has_source: bool,
    source_data_type: RuntimeDataType,
    has_target: bool,
    is_observing: bool,
    has_context_value: bool,
    current_dirt: RuntimeComponentDirt,
    current_target_origin: bool,
    is_collapsed: bool,
    has_container: bool,
    target_supports_push: bool,
) -> RuntimeDataBindContextBindEffect {
    let is_name_based = cpp_data_bind_is_name_based(data_bind);
    let resolves_path = has_data_context && is_name_based && !source_path_is_resolved;
    let uses_relative_view_model_property_lookup = has_data_context && is_name_based;
    let uses_view_model_property_lookup = has_data_context && !is_name_based;

    let mut effect = RuntimeDataBindContextBindEffect {
        branch: RuntimeDataBindContextBindBranch::NoDataContext,
        resolves_path,
        marks_path_resolved: resolves_path,
        updates_source_path_ids: resolves_path && source_path_ids != resolved_source_path_ids,
        uses_relative_view_model_property_lookup,
        uses_view_model_property_lookup,
        clear_source_effect: None,
        source_effect: None,
        bind_effect: None,
        unbind_effect: None,
        add_dirt_effect: None,
        binds_converter_from_context: has_data_context && has_converter,
    };

    if !has_data_context {
        return effect;
    }

    let source_is_unchanged = has_source && lookup_has_source && source_matches_lookup;
    if source_is_unchanged {
        effect.branch = RuntimeDataBindContextBindBranch::AddDirtExistingSource;
        effect.add_dirt_effect = Some(cpp_data_bind_add_dirt_effect(
            current_dirt,
            current_target_origin,
            cpp_data_bind_reconcile_dirt(data_bind),
            cpp_data_bind_source_to_target_runs_first(data_bind),
            false,
            is_collapsed,
            has_context_value,
            has_container,
        ));
        return effect;
    }

    if lookup_has_source {
        effect.branch = RuntimeDataBindContextBindBranch::BindSource;
        effect.clear_source_effect = Some(cpp_data_bind_clear_source_effect(data_bind, has_source));
        effect.source_effect = Some(cpp_data_bind_source_effect(
            data_bind,
            target,
            source_data_type,
        ));
        effect.bind_effect = Some(cpp_data_bind_bind_effect(
            data_bind,
            has_converter,
            converter_output_type,
            source_data_type,
            has_target,
            is_observing,
            has_context_value,
            current_dirt,
            current_target_origin,
            is_collapsed,
            has_container,
            target_supports_push,
        ));
    } else {
        effect.branch = RuntimeDataBindContextBindBranch::UnbindMissingSource;
        effect.unbind_effect = Some(cpp_data_bind_unbind_effect(
            data_bind,
            has_converter,
            has_source,
            has_target,
            is_observing,
            has_context_value,
        ));
    }

    effect
}

fn cpp_data_bind_update_effect(
    data_bind: &RuntimeObject,
    has_converter: bool,
    dirt: RuntimeComponentDirt,
    apply_target_to_source: bool,
    in_persisting_list: bool,
    has_source: bool,
    has_context_value: bool,
    has_target: bool,
) -> RuntimeDataBindUpdateEffect {
    let calls_update_dependents = dirt.contains(RuntimeComponentDirt::DEPENDENTS);
    let wants_target_to_source = apply_target_to_source
        && (in_persisting_list || dirt.contains(RuntimeComponentDirt::BINDINGS_TARGET));
    let can_apply_target_to_source = wants_target_to_source
        && cpp_data_bind_to_source(data_bind)
        && has_target
        && has_context_value;
    let applies_target_to_source_before_update =
        can_apply_target_to_source && !cpp_data_bind_source_to_target_runs_first(data_bind);
    let applies_target_to_source_after_update =
        can_apply_target_to_source && cpp_data_bind_source_to_target_runs_first(data_bind);
    let clears_dirt = !dirt.is_empty();
    let applies_source_to_target = clears_dirt
        && dirt.contains(RuntimeComponentDirt::BINDINGS)
        && has_source
        && has_context_value
        && cpp_data_bind_to_target(data_bind);

    RuntimeDataBindUpdateEffect {
        calls_update_dependents,
        updates_converter_dependents: calls_update_dependents && has_converter,
        applies_target_to_source_before_update,
        clears_dirt,
        applies_source_to_target,
        source_to_target_is_main_direction: applies_source_to_target
            && !cpp_data_bind_is_main_to_source(data_bind),
        suppresses_dirt_while_applying_source_to_target: applies_source_to_target,
        applies_target_to_source_after_update,
        target_to_source_is_main_direction: (applies_target_to_source_before_update
            || applies_target_to_source_after_update)
            && cpp_data_bind_is_main_to_source(data_bind),
    }
}

fn cpp_data_bind_remove_effect(
    is_processing: bool,
    in_persisting_list: bool,
    in_dirty_list: bool,
) -> RuntimeDataBindRemoveEffect {
    if is_processing {
        return RuntimeDataBindRemoveEffect {
            queues_pending_removal: true,
            removes_from_data_binds: false,
            scans_persisting_list: false,
            clears_persisting_list_flag: false,
            scans_dirty_lists: false,
            clears_dirty_list_flag: false,
            clears_container: false,
        };
    }

    RuntimeDataBindRemoveEffect {
        queues_pending_removal: false,
        removes_from_data_binds: true,
        scans_persisting_list: in_persisting_list,
        clears_persisting_list_flag: in_persisting_list,
        scans_dirty_lists: in_dirty_list,
        clears_dirty_list_flag: in_dirty_list,
        clears_container: true,
    }
}

fn cpp_data_bind_container_bind_context_effect(
    data_bind_context_ids: &[usize],
    has_data_context: bool,
) -> RuntimeDataBindContainerBindContextEffect {
    RuntimeDataBindContainerBindContextEffect {
        bind_from_context_data_bind_ids: data_bind_context_ids.to_vec(),
        stores_data_context: has_data_context,
        clears_data_context: !has_data_context,
    }
}

fn cpp_data_bind_container_unbind_effect(
    data_bind_ids: &[usize],
) -> RuntimeDataBindContainerUnbindEffect {
    RuntimeDataBindContainerUnbindEffect {
        unbinds_data_bind_ids: data_bind_ids.to_vec(),
        clears_data_context: true,
    }
}

fn cpp_data_bind_container_advance_effect(
    data_bind_ids: &[usize],
    advance_results: &[bool],
) -> RuntimeDataBindContainerAdvanceEffect {
    RuntimeDataBindContainerAdvanceEffect {
        advances_data_bind_ids: data_bind_ids.to_vec(),
        did_update: !data_bind_ids.is_empty()
            && advance_results.iter().any(|did_update| *did_update),
    }
}

fn cpp_data_bind_container_add_dirty_effect(
    data_bind: &RuntimeObject,
    in_persisting_list: bool,
    in_dirty_list: bool,
    is_processing: bool,
) -> RuntimeDataBindContainerAddDirtyEffect {
    if cpp_data_bind_to_source(data_bind) && in_persisting_list {
        return RuntimeDataBindContainerAddDirtyEffect {
            skips_persisting_to_source: true,
            skips_already_dirty: false,
            queue: None,
            queues_pending: false,
            sets_dirty_list_flag: false,
        };
    }

    if in_dirty_list {
        return RuntimeDataBindContainerAddDirtyEffect {
            skips_persisting_to_source: false,
            skips_already_dirty: true,
            queue: None,
            queues_pending: false,
            sets_dirty_list_flag: false,
        };
    }

    RuntimeDataBindContainerAddDirtyEffect {
        skips_persisting_to_source: false,
        skips_already_dirty: false,
        queue: Some(if cpp_data_bind_to_source(data_bind) {
            RuntimeDataBindUpdateQueue::DirtyToSource
        } else {
            RuntimeDataBindUpdateQueue::DirtyToTarget
        }),
        queues_pending: is_processing,
        sets_dirty_list_flag: true,
    }
}

fn cpp_data_converter_bind_context_effect(
    data_converter: &RuntimeObject,
    owned_data_bind_context_ids: &[usize],
    has_data_context: bool,
    has_parent_data_bind: bool,
    parent_has_source: bool,
    operation_view_model_lookup_type: Option<RuntimeDataType>,
    group_child_converter_ids: Vec<usize>,
) -> RuntimeDataConverterBindContextEffect {
    let operation_view_model_uses_source_path_lookup =
        has_data_context && data_converter.type_name == "DataConverterOperationViewModel";
    let operation_view_model_sets_number_source = operation_view_model_uses_source_path_lookup
        && operation_view_model_lookup_type == Some(RuntimeDataType::Number);
    let formula_checks_parent_data_bind_source = data_converter.type_name == "DataConverterFormula";
    let formula_sets_source_from_parent =
        formula_checks_parent_data_bind_source && has_parent_data_bind && parent_has_source;

    RuntimeDataConverterBindContextEffect {
        stores_parent_data_bind: true,
        owned_data_bind_context_effect: cpp_data_bind_container_bind_context_effect(
            owned_data_bind_context_ids,
            has_data_context,
        ),
        group_child_bind_from_context_converter_ids: group_child_converter_ids,
        operation_view_model_uses_source_path_lookup,
        operation_view_model_sets_number_source,
        operation_view_model_adds_data_bind_dependent: operation_view_model_sets_number_source,
        formula_checks_parent_data_bind_source,
        formula_sets_source_from_parent,
        formula_adds_source_dependent: formula_sets_source_from_parent,
    }
}

fn cpp_data_converter_unbind_effect(
    data_converter: &RuntimeObject,
    owned_data_bind_unbind_effect: Option<RuntimeDataBindContainerUnbindEffect>,
    group_child_converter_ids: Vec<usize>,
    has_formula_source: bool,
) -> RuntimeDataConverterUnbindEffect {
    let formula_clears_source =
        data_converter.type_name == "DataConverterFormula" && has_formula_source;
    RuntimeDataConverterUnbindEffect {
        owned_data_bind_unbind_effect,
        group_child_unbind_converter_ids: group_child_converter_ids,
        formula_removes_source_dependent: formula_clears_source,
        formula_clears_source,
    }
}

fn cpp_data_converter_update_effect(
    data_converter: &RuntimeObject,
    owned_data_bind_update_effect: Option<RuntimeDataBindContainerUpdateEffect>,
    group_child_converter_ids: Vec<usize>,
) -> RuntimeDataConverterUpdateEffect {
    RuntimeDataConverterUpdateEffect {
        owned_data_bind_update_effect,
        group_child_update_converter_ids: if data_converter.type_name == "DataConverterGroup" {
            group_child_converter_ids
        } else {
            Vec::new()
        },
    }
}

fn cpp_data_converter_mark_dirty_effect(
    parent_add_dirt_effect: Option<RuntimeDataBindAddDirtEffect>,
) -> RuntimeDataConverterMarkDirtyEffect {
    RuntimeDataConverterMarkDirtyEffect {
        parent_add_dirt_effect,
    }
}

fn cpp_data_converter_property_change_effect(
    data_converter: &RuntimeObject,
    property_name: &str,
    mark_converter_dirty_effect: RuntimeDataConverterMarkDirtyEffect,
) -> RuntimeDataConverterPropertyChangeEffect {
    RuntimeDataConverterPropertyChangeEffect {
        clears_number_to_list_items: data_converter.type_name == "DataConverterNumberToList"
            && property_name == "viewModelId",
        mark_converter_dirty_effect,
    }
}

fn cpp_data_converter_add_dirty_data_bind_effect(
    mark_converter_dirty_effect: RuntimeDataConverterMarkDirtyEffect,
    container_add_dirty_effect: RuntimeDataBindContainerAddDirtyEffect,
) -> RuntimeDataConverterAddDirtyDataBindEffect {
    RuntimeDataConverterAddDirtyDataBindEffect {
        mark_converter_dirty_effect,
        container_add_dirty_effect,
    }
}

fn cpp_data_converter_formula_add_dirt_effect(
    random_mode_value: u64,
) -> RuntimeDataConverterFormulaAddDirtEffect {
    RuntimeDataConverterFormulaAddDirtEffect {
        clears_randoms: random_mode_value == 2,
    }
}

fn cpp_data_converter_reset_effect(
    data_converter: &RuntimeObject,
    group_child_converter_ids: Vec<usize>,
) -> RuntimeDataConverterResetEffect {
    let resets_interpolator = data_converter.type_name == "DataConverterInterpolator";
    RuntimeDataConverterResetEffect {
        group_child_reset_converter_ids: if data_converter.type_name == "DataConverterGroup" {
            group_child_converter_ids
        } else {
            Vec::new()
        },
        resets_interpolator_advance_count: resets_interpolator,
        disposes_interpolator_advancer_values: resets_interpolator,
        clears_interpolator_smoothing_animation: resets_interpolator,
        clears_interpolator_initialized: resets_interpolator,
    }
}

fn cpp_data_converter_unbinds_owned_data_binds(data_converter: &RuntimeObject) -> bool {
    !matches!(
        data_converter.type_name,
        "DataConverterGroup" | "DataConverterFormula"
    )
}

fn cpp_data_converter_updates_owned_data_binds(data_converter: &RuntimeObject) -> bool {
    data_converter.type_name != "DataConverterGroup"
}

fn cpp_data_converter_property_change_marks_dirty(
    data_converter: &RuntimeObject,
    property_name: &str,
) -> bool {
    matches!(
        (data_converter.type_name, property_name),
        (
            "DataConverterRangeMapper",
            "minInput" | "maxInput" | "minOutput" | "maxOutput"
        ) | ("DataConverterStringPad", "length" | "padType" | "text")
            | ("DataConverterStringTrim", "trimType")
            | ("DataConverterToString", "decimals" | "colorFormat")
            | ("DataConverterNumberToList", "viewModelId")
            | ("DataConverterOperationValue", "operationValue")
            | ("DataConverterInterpolator", "duration")
    )
}

#[allow(clippy::too_many_arguments)]
fn cpp_data_bind_container_update_effect(
    persisting_data_bind_ids: &[usize],
    persisting_can_skip: &[bool],
    dirty_to_source_data_bind_ids: &[usize],
    dirty_to_target_data_bind_ids: &[usize],
    pending_dirty_to_source_data_bind_ids: &[usize],
    pending_dirty_to_target_data_bind_ids: &[usize],
    pending_addition_ids: &[usize],
    pending_removal_ids: &[usize],
    is_processing: bool,
    apply_target_to_source: bool,
) -> RuntimeDataBindContainerUpdateEffect {
    let early = |return_reason| RuntimeDataBindContainerUpdateEffect {
        return_reason: Some(return_reason),
        enters_processing: false,
        update_steps: Vec::new(),
        skipped_persisting_data_bind_ids: Vec::new(),
        clears_dirty_to_source_queue: false,
        clears_dirty_to_target_queue: false,
        clears_dirty_list_flag_data_bind_ids: Vec::new(),
        next_dirty_to_source_data_bind_ids: Vec::new(),
        next_dirty_to_target_data_bind_ids: Vec::new(),
        flushes_pending_addition_ids: Vec::new(),
        flushes_pending_removal_ids: Vec::new(),
        exits_processing: false,
    };

    if is_processing {
        return early(RuntimeDataBindContainerUpdateReturnReason::AlreadyProcessing);
    }
    if persisting_data_bind_ids.is_empty()
        && dirty_to_source_data_bind_ids.is_empty()
        && dirty_to_target_data_bind_ids.is_empty()
    {
        return early(RuntimeDataBindContainerUpdateReturnReason::NoActiveQueues);
    }

    let mut update_steps = Vec::new();
    let mut skipped_persisting_data_bind_ids = Vec::new();
    for (data_bind_id, can_skip) in persisting_data_bind_ids
        .iter()
        .copied()
        .zip(persisting_can_skip.iter().copied())
    {
        if can_skip {
            skipped_persisting_data_bind_ids.push(data_bind_id);
        } else {
            update_steps.push(RuntimeDataBindContainerUpdateStep {
                data_bind_id,
                queue: RuntimeDataBindUpdateQueue::Persisting,
                apply_target_to_source,
                clears_dirty_list_flag: false,
            });
        }
    }
    for data_bind_id in dirty_to_source_data_bind_ids {
        update_steps.push(RuntimeDataBindContainerUpdateStep {
            data_bind_id: *data_bind_id,
            queue: RuntimeDataBindUpdateQueue::DirtyToSource,
            apply_target_to_source,
            clears_dirty_list_flag: true,
        });
    }
    for data_bind_id in dirty_to_target_data_bind_ids {
        update_steps.push(RuntimeDataBindContainerUpdateStep {
            data_bind_id: *data_bind_id,
            queue: RuntimeDataBindUpdateQueue::DirtyToTarget,
            apply_target_to_source,
            clears_dirty_list_flag: true,
        });
    }

    RuntimeDataBindContainerUpdateEffect {
        return_reason: None,
        enters_processing: true,
        update_steps,
        skipped_persisting_data_bind_ids,
        clears_dirty_to_source_queue: true,
        clears_dirty_to_target_queue: true,
        clears_dirty_list_flag_data_bind_ids: dirty_to_source_data_bind_ids
            .iter()
            .chain(dirty_to_target_data_bind_ids)
            .copied()
            .collect(),
        next_dirty_to_source_data_bind_ids: pending_dirty_to_source_data_bind_ids.to_vec(),
        next_dirty_to_target_data_bind_ids: pending_dirty_to_target_data_bind_ids.to_vec(),
        flushes_pending_addition_ids: pending_addition_ids.to_vec(),
        flushes_pending_removal_ids: pending_removal_ids.to_vec(),
        exits_processing: true,
    }
}

fn cpp_sort_data_bind_ids(
    data_bind_ids: &[usize],
    mut to_source: impl FnMut(&usize) -> bool,
) -> Vec<usize> {
    let mut data_bind_ids = data_bind_ids.to_vec();
    let mut current_to_source_index = 0usize;
    for i in 0..data_bind_ids.len() {
        if to_source(&data_bind_ids[i]) {
            if i != current_to_source_index {
                data_bind_ids.swap(current_to_source_index, i);
            }
            current_to_source_index += 1;
        }
    }
    data_bind_ids
}

fn cpp_data_bind_target_supports_push(
    data_bind: &RuntimeObject,
    target: Option<&RuntimeObject>,
) -> bool {
    let Some(target) = target else {
        return false;
    };
    let property_key = data_bind.uint_property("propertyKey").unwrap_or(0);
    if cpp_data_bind_polling_property_keys().contains(&property_key) {
        return false;
    }

    !matches!(
        target.type_name,
        "BindablePropertyAsset" | "BindablePropertyViewModel" | "ViewModelInstanceViewModel"
    )
}

fn cpp_data_bind_polling_property_keys() -> [u64; 16] {
    [
        cpp_property_key("Solo", "activeComponentId"),
        cpp_property_key("Node", "computedLocalX"),
        cpp_property_key("Node", "computedLocalY"),
        cpp_property_key("Node", "computedWorldX"),
        cpp_property_key("Node", "computedWorldY"),
        cpp_property_key("Node", "computedRootX"),
        cpp_property_key("Node", "computedRootY"),
        cpp_property_key("Node", "computedWidth"),
        cpp_property_key("Node", "computedHeight"),
        cpp_property_key("Shape", "length"),
        cpp_property_key("ScrollConstraint", "scrollIndex"),
        cpp_property_key("ScrollConstraint", "scrollPercentX"),
        cpp_property_key("ScrollConstraint", "scrollPercentY"),
        cpp_property_key("ScrollConstraint", "velocityX"),
        cpp_property_key("ScrollConstraint", "velocityY"),
        cpp_property_key("ScrollConstraint", "scrollActive"),
    ]
}

fn cpp_property_key(definition_name: &str, property_name: &str) -> u64 {
    definition_by_name(definition_name)
        .and_then(|definition| {
            definition
                .properties
                .iter()
                .find(|property| property.name == property_name)
        })
        .map(|property| u64::from(property.key.int))
        .unwrap_or(0)
}

fn cpp_data_converter_direct_output_type(
    data_converter: &RuntimeObject,
) -> Option<RuntimeDataType> {
    let definition = definition_by_type_key(data_converter.type_key)?;
    if !definition.is_a("DataConverter") {
        return None;
    }

    if definition.is_a("DataConverterOperation") {
        return Some(RuntimeDataType::Number);
    }

    Some(match data_converter.type_name {
        "DataConverterBooleanNegate" => RuntimeDataType::Boolean,
        "DataConverterFormula" => RuntimeDataType::Number,
        "DataConverterInterpolator" => RuntimeDataType::Input,
        "DataConverterListToLength" => RuntimeDataType::Number,
        "DataConverterNumberToList" => RuntimeDataType::List,
        "DataConverterRangeMapper" => RuntimeDataType::Number,
        "DataConverterRounder" => RuntimeDataType::Number,
        "DataConverterStringPad" => RuntimeDataType::String,
        "DataConverterStringRemoveZeros" => RuntimeDataType::String,
        "DataConverterStringTrim" => RuntimeDataType::String,
        "ScriptedDataConverter" => RuntimeDataType::Any,
        "DataConverterToNumber" => RuntimeDataType::Number,
        "DataConverterToString" => RuntimeDataType::String,
        "DataConverterTrigger" => RuntimeDataType::Trigger,
        _ => RuntimeDataType::None,
    })
}

pub fn data_converter_to_string_number_value(value: f32, flags: u64, decimals: u64) -> Vec<u8> {
    cpp_format_number_to_string(value, flags, decimals)
}

pub fn data_converter_to_string_boolean_value(value: bool) -> Vec<u8> {
    if value { b"1".to_vec() } else { b"0".to_vec() }
}

pub fn data_converter_to_string_string_value(value: &[u8]) -> Vec<u8> {
    value.to_vec()
}

pub fn data_converter_to_string_trigger_value(value: u64) -> Vec<u8> {
    value.to_string().into_bytes()
}

pub fn data_converter_to_string_symbol_list_index_value(value: u64) -> Vec<u8> {
    value.to_string().into_bytes()
}

pub fn data_converter_to_string_color_value(value: u32, color_format: &[u8]) -> Vec<u8> {
    if color_format.is_empty() {
        ((value as i32).to_string()).into_bytes()
    } else {
        cpp_format_color_to_string(value, color_format)
    }
}

pub fn data_converter_string_trim_value(value: &[u8], trim_type: u64) -> Vec<u8> {
    cpp_trim_string(value, trim_type)
}

pub fn data_converter_string_remove_zeros_value(value: &[u8]) -> Vec<u8> {
    cpp_remove_trailing_zeros(value)
}

pub fn data_converter_string_pad_value(
    value: &[u8],
    length: u64,
    text: &[u8],
    pad_type: u64,
) -> Vec<u8> {
    cpp_pad_string(value, length, text, pad_type)
}

fn cpp_format_number_to_string(value: f32, flags: u64, decimals: u64) -> Vec<u8> {
    const ROUND: u64 = 1 << 0;
    const TRAILING_ZEROS: u64 = 1 << 1;
    const FORMAT_WITH_COMMAS: u64 = 1 << 2;

    let mut value = if value.is_nan() {
        "nan".to_owned()
    } else if flags & ROUND != 0 {
        format!("{:.*}", decimals as usize, value)
    } else {
        format!("{value:.6}")
    };

    if flags & TRAILING_ZEROS != 0 {
        value = String::from_utf8(cpp_remove_trailing_zeros(value.as_bytes()))
            .expect("trailing-zero removal preserves UTF-8");
    }

    if flags & FORMAT_WITH_COMMAS != 0 {
        value = cpp_format_with_commas(&value);
    }

    value.into_bytes()
}

fn cpp_format_with_commas(value: &str) -> String {
    let (mut int_part, frac_part) = match value.find('.') {
        Some(dot_index) => (value[..dot_index].to_owned(), value[dot_index..].to_owned()),
        None => (value.to_owned(), String::new()),
    };

    let mut insert_position = int_part.len().saturating_sub(3);
    while insert_position > 0
        && int_part
            .as_bytes()
            .get(insert_position - 1)
            .is_some_and(u8::is_ascii_digit)
    {
        int_part.insert(insert_position, ',');
        insert_position = insert_position.saturating_sub(3);
    }

    int_part + &frac_part
}

fn cpp_remove_trailing_zeros(value: &[u8]) -> Vec<u8> {
    let Some(dot_index) = value.iter().position(|byte| *byte == b'.') else {
        return value.to_vec();
    };

    let mut end = value.len();
    while end > dot_index && value[end - 1] == b'0' {
        end -= 1;
    }
    if end > 0 && value[end - 1] == b'.' {
        end -= 1;
    }
    value[..end].to_vec()
}

fn cpp_format_color_to_string(color: u32, format: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut is_escaped = false;
    let mut is_marker = false;

    for byte in format {
        if is_escaped {
            output.push(*byte);
            is_escaped = false;
        } else if *byte == b'\\' {
            if is_marker {
                output.push(b'%');
                is_marker = false;
            }
            is_escaped = true;
        } else if *byte == b'%' {
            if is_marker {
                output.push(b'%');
            }
            is_marker = true;
        } else if is_marker {
            match *byte {
                b'r' => output.extend_from_slice(cpp_color_red(color).to_string().as_bytes()),
                b'g' => output.extend_from_slice(cpp_color_green(color).to_string().as_bytes()),
                b'b' => output.extend_from_slice(cpp_color_blue(color).to_string().as_bytes()),
                b'a' => output.extend_from_slice(cpp_color_alpha(color).to_string().as_bytes()),
                b'R' => output.extend_from_slice(cpp_color_hex(cpp_color_red(color)).as_bytes()),
                b'G' => output.extend_from_slice(cpp_color_hex(cpp_color_green(color)).as_bytes()),
                b'B' => output.extend_from_slice(cpp_color_hex(cpp_color_blue(color)).as_bytes()),
                b'A' => output.extend_from_slice(cpp_color_hex(cpp_color_alpha(color)).as_bytes()),
                b'h' => output.extend_from_slice(cpp_color_hsl(color).0.to_string().as_bytes()),
                b'l' => output.extend_from_slice(cpp_color_hsl(color).1.to_string().as_bytes()),
                b's' => output.extend_from_slice(cpp_color_hsl(color).2.to_string().as_bytes()),
                _ => {
                    output.push(b'%');
                    output.push(*byte);
                }
            }
            is_marker = false;
        } else {
            output.push(*byte);
        }
    }

    output
}

fn cpp_color_alpha(color: u32) -> u8 {
    ((color >> 24) & 0xff) as u8
}

fn cpp_color_red(color: u32) -> u8 {
    ((color >> 16) & 0xff) as u8
}

fn cpp_color_green(color: u32) -> u8 {
    ((color >> 8) & 0xff) as u8
}

fn cpp_color_blue(color: u32) -> u8 {
    (color & 0xff) as u8
}

fn cpp_color_argb(alpha: u8, red: u8, green: u8, blue: u8) -> u32 {
    (u32::from(alpha) << 24) | (u32::from(red) << 16) | (u32::from(green) << 8) | u32::from(blue)
}

fn cpp_color_lerp(from: u32, to: u32, mix: f32) -> u32 {
    cpp_color_argb(
        cpp_lerp_color_channel(cpp_color_alpha(from), cpp_color_alpha(to), mix),
        cpp_lerp_color_channel(cpp_color_red(from), cpp_color_red(to), mix),
        cpp_lerp_color_channel(cpp_color_green(from), cpp_color_green(to), mix),
        cpp_lerp_color_channel(cpp_color_blue(from), cpp_color_blue(to), mix),
    )
}

fn cpp_lerp_color_channel(from: u8, to: u8, mix: f32) -> u8 {
    (f32::from(from) * (1.0 - mix) + f32::from(to) * mix)
        .clamp(0.0, 255.0)
        .round() as u8
}

fn cpp_color_hex(value: u8) -> String {
    format!("{value:02X}")
}

fn cpp_color_hsl(color: u32) -> (i32, i32, i32) {
    let r = f32::from(cpp_color_red(color)) / 255.0;
    let g = f32::from(cpp_color_green(color)) / 255.0;
    let b = f32::from(cpp_color_blue(color)) / 255.0;
    let max_component = r.max(g).max(b);
    let min_component = r.min(g).min(b);
    let delta = max_component - min_component;

    let mut hue = 0.0;
    if delta != 0.0 {
        if max_component == r {
            hue = ((g - b) / delta) % 6.0;
        } else if max_component == g {
            hue = (b - r) / delta + 2.0;
        } else {
            hue = (r - g) / delta + 4.0;
        }
    }

    let mut h = (hue * 60.0).round() as i32;
    if h < 0 {
        h += 360;
    }

    let lum = (max_component + min_component) / 2.0;
    let sat = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * lum - 1.0).abs())
    };

    let l = (lum * 100.0).round() as i32;
    let s = (sat * 100.0).round() as i32;
    (h, l, s)
}

fn cpp_trim_string(value: &[u8], trim_type: u64) -> Vec<u8> {
    let mut start = 0usize;
    let mut end = value.len();

    if trim_type == 1 || trim_type == 3 {
        while start < end && value[start].is_ascii_whitespace() {
            start += 1;
        }
    }

    if trim_type == 2 || trim_type == 3 {
        while end > start && value[end - 1].is_ascii_whitespace() {
            end -= 1;
        }
    }

    value[start..end].to_vec()
}

fn cpp_pad_string(value: &[u8], length: u64, text: &[u8], pad_type: u64) -> Vec<u8> {
    let Ok(length) = usize::try_from(length) else {
        return value.to_vec();
    };
    if value.len() >= length || text.is_empty() {
        return value.to_vec();
    }

    let pad_text_size = length - value.len();
    let mut pad_text = Vec::with_capacity(pad_text_size);
    while pad_text.len() < pad_text_size {
        let remaining = pad_text_size - pad_text.len();
        let max_length = remaining.min(text.len());
        pad_text.extend_from_slice(&text[..max_length]);
    }

    let mut output = Vec::with_capacity(length);
    if pad_type == 1 {
        output.extend_from_slice(value);
        output.extend_from_slice(&pad_text);
    } else {
        output.extend_from_slice(&pad_text);
        output.extend_from_slice(value);
    }
    output
}

pub fn data_converter_to_number_string_value(value: &[u8]) -> f32 {
    cpp_atof_f32(value)
}

fn cpp_atof_f32(value: &[u8]) -> f32 {
    let mut start = 0usize;
    while start < value.len() && value[start].is_ascii_whitespace() {
        start += 1;
    }

    let mut number_start = start;
    let sign = match value.get(number_start) {
        Some(b'-') => {
            number_start += 1;
            -1.0
        }
        Some(b'+') => {
            number_start += 1;
            1.0
        }
        _ => 1.0,
    };
    if value.get(number_start) == Some(&b'0')
        && value
            .get(number_start + 1)
            .is_some_and(|byte| matches!(*byte, b'x' | b'X'))
    {
        return cpp_atof_hex_f32(value, number_start, sign);
    }

    let mut end = number_start;
    if value
        .get(end)
        .is_some_and(|byte| matches!(*byte, b'+' | b'-'))
    {
        end += 1;
    }

    let mut digits = 0usize;
    while value.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
        digits += 1;
    }

    if value.get(end) == Some(&b'.') {
        end += 1;
        while value.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
            digits += 1;
        }
    }

    if digits == 0 {
        return 0.0;
    }

    let mantissa_end = end;
    if value
        .get(end)
        .is_some_and(|byte| matches!(*byte, b'e' | b'E'))
    {
        end += 1;
        if value
            .get(end)
            .is_some_and(|byte| matches!(*byte, b'+' | b'-'))
        {
            end += 1;
        }
        let exponent_start = end;
        while value.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        if exponent_start == end {
            end = mantissa_end;
        }
    }

    let Ok(parsed) = std::str::from_utf8(&value[start..end])
        .expect("numeric prefix is ASCII")
        .parse::<f64>()
    else {
        return 0.0;
    };

    if parsed.is_finite() {
        parsed as f32
    } else {
        0.0
    }
}

fn cpp_atof_hex_f32(value: &[u8], number_start: usize, sign: f64) -> f32 {
    let mut end = number_start + 2;
    let mut mantissa = 0.0f64;
    let mut digits = 0usize;
    while let Some(digit) = value.get(end).and_then(|byte| ascii_hex_digit_value(*byte)) {
        mantissa = mantissa * 16.0 + f64::from(digit);
        end += 1;
        digits += 1;
    }

    if value.get(end) == Some(&b'.') {
        end += 1;
        let mut place = 1.0 / 16.0;
        while let Some(digit) = value.get(end).and_then(|byte| ascii_hex_digit_value(*byte)) {
            mantissa += f64::from(digit) * place;
            place /= 16.0;
            end += 1;
            digits += 1;
        }
    }

    if digits == 0 {
        return 0.0;
    }

    let mut exponent = 0i32;
    if value
        .get(end)
        .is_some_and(|byte| matches!(*byte, b'p' | b'P'))
    {
        end += 1;
        let exponent_sign = match value.get(end) {
            Some(b'-') => {
                end += 1;
                -1
            }
            Some(b'+') => {
                end += 1;
                1
            }
            _ => 1,
        };
        let exponent_start = end;
        let mut exponent_value = 0i32;
        while let Some(digit) = value
            .get(end)
            .filter(|byte| byte.is_ascii_digit())
            .map(|byte| i32::from(*byte - b'0'))
        {
            exponent_value = exponent_value.saturating_mul(10).saturating_add(digit);
            end += 1;
        }
        if exponent_start != end {
            exponent = exponent_sign * exponent_value;
        }
    }

    let parsed = sign * mantissa * 2.0f64.powi(exponent);
    if parsed.is_finite() {
        parsed as f32
    } else {
        0.0
    }
}

fn ascii_hex_digit_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn cpp_convert_operation_value(
    input: &RuntimeConvertedDataValue<'_>,
    operation_type: u64,
    operation_value: f32,
) -> f32 {
    let Some(input_value) = (match input {
        RuntimeConvertedDataValue::Number(value) => Some(*value),
        RuntimeConvertedDataValue::SymbolListIndex(value) => Some(*value as f32),
        _ => None,
    }) else {
        return 0.0;
    };

    match operation_type {
        0 => input_value + operation_value,
        1 => input_value - operation_value,
        2 => input_value * operation_value,
        3 => input_value / operation_value,
        4 => cpp_positive_mod(input_value, operation_value),
        5 => input_value.sqrt(),
        6 => input_value.powf(operation_value),
        7 => input_value.exp(),
        8 => input_value.ln(),
        9 => input_value.cos(),
        10 => input_value.sin(),
        11 => input_value.tan(),
        12 => input_value.acos(),
        13 => input_value.asin(),
        14 => input_value.atan(),
        15 => input_value.atan2(operation_value),
        16 => input_value.round(),
        17 => input_value.floor(),
        18 => input_value.ceil(),
        _ => operation_value,
    }
}

fn cpp_reverse_convert_operation_value(
    input: &RuntimeConvertedDataValue<'_>,
    operation_type: u64,
    operation_value: f32,
) -> f32 {
    let RuntimeConvertedDataValue::Number(input_value) = input else {
        return 0.0;
    };
    let input_value = *input_value;

    match operation_type {
        0 => input_value - operation_value,
        1 => input_value + operation_value,
        2 => input_value / operation_value,
        3 => input_value * operation_value,
        4 => input_value,
        5 => input_value.powf(2.0),
        6 => input_value.powf(1.0 / operation_value),
        7 => input_value.ln(),
        8 => input_value.exp(),
        9 => input_value.acos(),
        10 => input_value.asin(),
        11 => input_value.atan(),
        12 => input_value.cos(),
        13 => input_value.sin(),
        14 => input_value.tan(),
        15..=18 => input_value,
        _ => operation_value,
    }
}

fn cpp_apply_formula_operation(left: f32, right: f32, operation_type: u64) -> f32 {
    match operation_type {
        0 => left + right,
        1 => left - right,
        2 => left * right,
        3 => left / right,
        4 => cpp_positive_mod(left, right),
        _ => 0.0,
    }
}

fn cpp_apply_formula_function(
    stack: &mut Vec<f32>,
    function_type: u64,
    total_arguments: usize,
    random_mode: u64,
    formula_randoms: Option<&mut RuntimeFormulaRandomSource<'_>>,
) -> Option<f32> {
    let mut function_arguments = Vec::new();
    for _ in 0..total_arguments {
        if let Some(function_argument) = stack.pop() {
            function_arguments.push(function_argument);
        }
    }

    let value = match function_type {
        0 => {
            if function_arguments.is_empty() {
                0.0
            } else {
                let mut min_value = function_arguments[0];
                for value in function_arguments.iter().skip(1) {
                    if *value < min_value {
                        min_value = *value;
                    }
                }
                min_value
            }
        }
        1 => {
            if function_arguments.is_empty() {
                0.0
            } else {
                let mut max_value = function_arguments[0];
                for value in function_arguments.iter().skip(1) {
                    if *value > max_value {
                        max_value = *value;
                    }
                }
                max_value
            }
        }
        2 => function_arguments
            .last()
            .copied()
            .map(f32::round)
            .unwrap_or(0.0),
        3 => function_arguments
            .last()
            .copied()
            .map(f32::ceil)
            .unwrap_or(0.0),
        4 => function_arguments
            .last()
            .copied()
            .map(f32::floor)
            .unwrap_or(0.0),
        5 => function_arguments
            .last()
            .copied()
            .map(f32::sqrt)
            .unwrap_or(0.0),
        6 => {
            if function_arguments.len() > 1 {
                let exponent = function_arguments[function_arguments.len() - 2];
                let x = function_arguments[function_arguments.len() - 1];
                x.powf(exponent)
            } else {
                0.0
            }
        }
        7 => function_arguments
            .last()
            .copied()
            .map(f32::exp)
            .unwrap_or(0.0),
        8 => function_arguments
            .last()
            .copied()
            .map(f32::ln)
            .unwrap_or(0.0),
        9 => function_arguments
            .last()
            .copied()
            .map(f32::cos)
            .unwrap_or(0.0),
        10 => function_arguments
            .last()
            .copied()
            .map(f32::sin)
            .unwrap_or(0.0),
        11 => function_arguments
            .last()
            .copied()
            .map(f32::tan)
            .unwrap_or(0.0),
        12 => function_arguments
            .last()
            .copied()
            .map(f32::acos)
            .unwrap_or(0.0),
        13 => function_arguments
            .last()
            .copied()
            .map(f32::asin)
            .unwrap_or(0.0),
        14 => function_arguments
            .last()
            .copied()
            .map(f32::atan)
            .unwrap_or(0.0),
        15 => {
            if function_arguments.len() > 1 {
                let argument1 = function_arguments[function_arguments.len() - 1];
                let argument2 = function_arguments[function_arguments.len() - 2];
                argument1.atan2(argument2)
            } else {
                0.0
            }
        }
        16 => {
            let random_value = formula_randoms?.next(random_mode, 0)?;
            let mut lower_bound = 0.0;
            let mut upper_bound = 1.0;
            if function_arguments.len() == 1 {
                upper_bound = function_arguments[function_arguments.len() - 1];
            } else if function_arguments.len() > 1 {
                lower_bound = function_arguments[function_arguments.len() - 1];
                upper_bound = function_arguments[function_arguments.len() - 2];
            }
            lower_bound + (upper_bound - lower_bound) * random_value
        }
        _ => 0.0,
    };
    Some(value)
}

fn cpp_positive_mod(value: f32, mut range: f32) -> f32 {
    if range < 0.0 {
        range = -range;
    }
    let mut value = value % range;
    if value < 0.0 {
        value += range;
    }
    value
}

fn cpp_key_frame_interpolator_transform(interpolator: &RuntimeObject, factor: f32) -> Option<f32> {
    match interpolator.type_name {
        "CubicEaseInterpolator" => Some(cpp_cubic_interpolator_transform(interpolator, factor)),
        "ElasticInterpolator" => Some(cpp_elastic_interpolator_transform(interpolator, factor)),
        _ => None,
    }
}

fn cpp_interpolator_state_factor(
    file: &RuntimeFile,
    data_converter: &RuntimeObject,
    elapsed_seconds: f32,
) -> Option<f32> {
    let duration = data_converter.double_property("duration").unwrap_or(1.0);
    let mut factor = if duration > 0.0 {
        f32::min(1.0, elapsed_seconds / duration)
    } else {
        1.0
    };
    if let Some(interpolator) = file.resolved_interpolator_for_data_converter_object(data_converter)
    {
        factor = cpp_key_frame_interpolator_transform(interpolator, factor)?;
    }
    Some(factor)
}

fn cpp_cubic_interpolator_transform(interpolator: &RuntimeObject, factor: f32) -> f32 {
    let x1 = interpolator.double_property("x1").unwrap_or(0.42);
    let y1 = interpolator.double_property("y1").unwrap_or(0.0);
    let x2 = interpolator.double_property("x2").unwrap_or(0.58);
    let y2 = interpolator.double_property("y2").unwrap_or(1.0);
    let t = cpp_cubic_interpolator_get_t(factor, x1, x2);
    cpp_cubic_interpolator_calc_bezier(t, y1, y2)
}

fn cpp_cubic_interpolator_calc_bezier(t: f32, a1: f32, a2: f32) -> f32 {
    (((1.0 - 3.0 * a2 + 3.0 * a1) * t + (3.0 * a2 - 6.0 * a1)) * t + (3.0 * a1)) * t
}

fn cpp_cubic_interpolator_slope(t: f32, a1: f32, a2: f32) -> f32 {
    3.0 * (1.0 - 3.0 * a2 + 3.0 * a1) * t * t + 2.0 * (3.0 * a2 - 6.0 * a1) * t + (3.0 * a1)
}

fn cpp_cubic_interpolator_get_t(x: f32, x1: f32, x2: f32) -> f32 {
    const SPLINE_TABLE_SIZE: usize = 11;
    const SAMPLE_STEP_SIZE: f32 = 1.0 / (SPLINE_TABLE_SIZE as f32 - 1.0);
    const NEWTON_ITERATIONS: usize = 4;
    const NEWTON_MIN_SLOPE: f32 = 0.001;
    const SUBDIVISION_PRECISION: f32 = 0.0000001;
    const SUBDIVISION_MAX_ITERATIONS: usize = 10;

    let mut values = [0.0; SPLINE_TABLE_SIZE];
    for (i, value) in values.iter_mut().enumerate() {
        *value = cpp_cubic_interpolator_calc_bezier(i as f32 * SAMPLE_STEP_SIZE, x1, x2);
    }

    let mut interval_start = 0.0;
    let mut current_sample = 1;
    let last_sample = SPLINE_TABLE_SIZE - 1;
    while current_sample != last_sample && values[current_sample] <= x {
        interval_start += SAMPLE_STEP_SIZE;
        current_sample += 1;
    }
    current_sample -= 1;

    let dist = (x - values[current_sample]) / (values[current_sample + 1] - values[current_sample]);
    let mut guess_for_t = interval_start + dist * SAMPLE_STEP_SIZE;
    let initial_slope = cpp_cubic_interpolator_slope(guess_for_t, x1, x2);
    if initial_slope >= NEWTON_MIN_SLOPE {
        for _ in 0..NEWTON_ITERATIONS {
            let current_slope = cpp_cubic_interpolator_slope(guess_for_t, x1, x2);
            if current_slope == 0.0 {
                return guess_for_t;
            }
            let current_x = cpp_cubic_interpolator_calc_bezier(guess_for_t, x1, x2) - x;
            guess_for_t -= current_x / current_slope;
        }
        guess_for_t
    } else if initial_slope == 0.0 {
        guess_for_t
    } else {
        let mut upper_bound = interval_start + SAMPLE_STEP_SIZE;
        let mut iterations = 0;
        loop {
            let current_t = interval_start + (upper_bound - interval_start) / 2.0;
            let current_x = cpp_cubic_interpolator_calc_bezier(current_t, x1, x2) - x;
            if current_x > 0.0 {
                upper_bound = current_t;
            } else {
                interval_start = current_t;
            }
            iterations += 1;
            if current_x.abs() <= SUBDIVISION_PRECISION || iterations >= SUBDIVISION_MAX_ITERATIONS
            {
                return current_t;
            }
        }
    }
}

fn cpp_elastic_interpolator_transform(interpolator: &RuntimeObject, factor: f32) -> f32 {
    let amplitude = interpolator.double_property("amplitude").unwrap_or(1.0);
    let serialized_period = interpolator.double_property("period").unwrap_or(1.0);
    let period = if serialized_period == 0.0 {
        0.5
    } else {
        serialized_period
    };
    let shift = if amplitude < 1.0 {
        period / 4.0
    } else {
        period / (2.0 * std::f32::consts::PI) * (1.0 / amplitude).asin()
    };

    match interpolator.uint_property("easingValue").unwrap_or(1) {
        0 => cpp_elastic_ease_in(factor, amplitude, period, shift),
        1 => cpp_elastic_ease_out(factor, amplitude, period, shift),
        2 => cpp_elastic_ease_in_out(factor, amplitude, period, shift),
        _ => factor,
    }
}

fn cpp_elastic_actual_amplitude(time: f32, amplitude: f32, shift: f32) -> f32 {
    if amplitude < 1.0 {
        let shift_abs = shift.abs();
        let time_abs = time.abs();
        if time_abs < shift_abs {
            let l = time_abs / shift_abs;
            return (amplitude * l) + (1.0 - l);
        }
    }

    amplitude
}

fn cpp_elastic_ease_out(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor;
    let actual_amplitude = cpp_elastic_actual_amplitude(time, amplitude, shift);
    actual_amplitude
        * 2.0_f32.powf(10.0 * -time)
        * ((time - shift) * (2.0 * std::f32::consts::PI) / period).sin()
        + 1.0
}

fn cpp_elastic_ease_in(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor - 1.0;
    let actual_amplitude = cpp_elastic_actual_amplitude(time, amplitude, shift);
    -(actual_amplitude
        * 2.0_f32.powf(10.0 * time)
        * ((-time - shift) * (2.0 * std::f32::consts::PI) / period).sin())
}

fn cpp_elastic_ease_in_out(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor * 2.0 - 1.0;
    let actual_amplitude = cpp_elastic_actual_amplitude(time, amplitude, shift);
    if time < 0.0 {
        -0.5 * actual_amplitude
            * 2.0_f32.powf(10.0 * time)
            * ((-time - shift) * (2.0 * std::f32::consts::PI) / period).sin()
    } else {
        0.5 * (actual_amplitude
            * 2.0_f32.powf(10.0 * -time)
            * ((time - shift) * (2.0 * std::f32::consts::PI) / period).sin())
            + 1.0
    }
}

fn cpp_view_model_instance_value_data_type(type_name: &str) -> RuntimeDataType {
    match type_name {
        "ViewModelInstanceNumber" => RuntimeDataType::Number,
        "ViewModelInstanceString" => RuntimeDataType::String,
        "ViewModelInstanceBoolean" => RuntimeDataType::Boolean,
        "ViewModelInstanceColor" => RuntimeDataType::Color,
        "ViewModelInstanceList" => RuntimeDataType::List,
        "ViewModelInstanceEnum" => RuntimeDataType::EnumType,
        "ViewModelInstanceTrigger" => RuntimeDataType::Trigger,
        "ViewModelInstanceViewModel" => RuntimeDataType::ViewModel,
        "ViewModelInstanceSymbolListIndex" => RuntimeDataType::SymbolListIndex,
        "ViewModelInstanceAssetImage" => RuntimeDataType::AssetImage,
        "ViewModelInstanceAssetFont" => RuntimeDataType::AssetFont,
        "ViewModelInstanceArtboard" => RuntimeDataType::Artboard,
        _ => RuntimeDataType::None,
    }
}

fn cpp_view_model_property_instance_type_key(type_name: &str) -> Option<u16> {
    let instance_type_name = match type_name {
        "ViewModelPropertyNumber" => "ViewModelInstanceNumber",
        "ViewModelPropertyString" => "ViewModelInstanceString",
        "ViewModelPropertyBoolean" => "ViewModelInstanceBoolean",
        "ViewModelPropertyColor" => "ViewModelInstanceColor",
        "ViewModelPropertyList" => "ViewModelInstanceList",
        "ViewModelPropertyEnum" | "ViewModelPropertyEnumCustom" | "ViewModelPropertyEnumSystem" => {
            "ViewModelInstanceEnum"
        }
        "ViewModelPropertyTrigger" => "ViewModelInstanceTrigger",
        "ViewModelPropertyViewModel" => "ViewModelInstanceViewModel",
        "ViewModelPropertySymbolListIndex" => "ViewModelInstanceSymbolListIndex",
        "ViewModelPropertyAssetImage" => "ViewModelInstanceAssetImage",
        "ViewModelPropertyAssetFont" => "ViewModelInstanceAssetFont",
        "ViewModelPropertyArtboard" => "ViewModelInstanceArtboard",
        _ => return None,
    };

    definition_by_name(instance_type_name).map(|definition| definition.type_key.int)
}

fn cpp_is_data_bind_path_referencer(object: &RuntimeObject) -> bool {
    let Some(definition) = definition_by_type_key(object.type_key) else {
        return false;
    };

    definition.is_a("NestedArtboard")
        || matches!(
            definition.name,
            "StateMachineListenerSingle"
                | "StateMachineFireTrigger"
                | "ListenerInputTypeViewModel"
                | "ScriptInputViewModelProperty"
        )
}

fn cpp_claims_latest_data_bind_path(object: &RuntimeObject) -> bool {
    let Some(definition) = definition_by_type_key(object.type_key) else {
        return false;
    };

    definition.is_a("NestedArtboard")
        || matches!(
            definition.name,
            "StateMachineListenerSingle"
                | "StateMachineFireTrigger"
                | "ScriptInputViewModelProperty"
        )
}

fn cpp_inline_data_bind_path_property(object: &RuntimeObject) -> Option<&'static str> {
    let definition = definition_by_type_key(object.type_key)?;

    if definition.is_a("NestedArtboard") || definition.name == "ScriptInputViewModelProperty" {
        return Some("dataBindPathIds");
    }

    matches!(
        definition.name,
        "StateMachineListenerSingle" | "StateMachineFireTrigger" | "ListenerInputTypeViewModel"
    )
    .then_some("viewModelPathIds")
}

fn cpp_file_asset_extension(type_name: &str) -> Option<&'static str> {
    match type_name {
        "ImageAsset" => Some("png"),
        "FontAsset" => Some("ttf"),
        "AudioAsset" => Some("wav"),
        "BlobAsset" => Some("blob"),
        "ScriptAsset" => Some("lua"),
        "ShaderAsset" => Some("rstb"),
        "ManifestAsset" => Some("man"),
        _ => None,
    }
}

fn cpp_file_asset_referencer_index(object: &RuntimeObject) -> Option<u64> {
    let definition = definition_by_type_key(object.type_key)?;
    match definition.name {
        "Image" | "AudioEvent" => object.uint_property("assetId"),
        "TextStyle" => object.uint_property("fontAssetId"),
        _ if definition_is_cpp_scripted_object(definition) => object.uint_property("scriptAssetId"),
        _ => None,
    }
}

fn cpp_file_asset_matches_referencer(referencer: &RuntimeObject, asset: &RuntimeObject) -> bool {
    let Some(definition) = definition_by_type_key(referencer.type_key) else {
        return false;
    };

    match definition.name {
        "Image" => asset.type_name == "ImageAsset",
        "AudioEvent" => asset.type_name == "AudioAsset",
        "TextStyle" => asset.type_name == "FontAsset",
        _ if definition_is_cpp_scripted_object(definition) => asset.type_name == "ScriptAsset",
        _ => false,
    }
}

fn cpp_resolved_state_transition_interpolator<'a>(
    transition: &RuntimeObject,
    artboard_range: (usize, usize),
    objects: &'a [Option<RuntimeObject>],
    import_statuses: &[RuntimeImportStatus],
) -> Option<&'a RuntimeObject> {
    let interpolator_id = transition.uint_property("interpolatorId")?;
    if interpolator_id == u64::from(u32::MAX) {
        return None;
    }

    let local_id = usize::try_from(interpolator_id).ok()?;
    let slots = runtime_artboard_local_slots(objects, import_statuses, artboard_range);
    let interpolator = local_object_reference(&slots, objects, Some(local_id as u64))?;
    definition_by_type_key(interpolator.type_key)
        .is_some_and(|definition| definition.is_a("KeyFrameInterpolator"))
        .then_some(interpolator)
}

fn cpp_resolved_animation_state_animation<'a>(
    state: &RuntimeObject,
    artboard_animations: &[&'a RuntimeObject],
) -> Option<&'a RuntimeObject> {
    if state.type_name != "AnimationState" {
        return None;
    }

    let animation_index = usize::try_from(state.uint_property("animationId")?).ok()?;
    artboard_animations.get(animation_index).copied()
}

fn definition_is_cpp_scripted_object(definition: &'static Definition) -> bool {
    matches!(
        definition.name,
        "ScriptedDataConverter"
            | "ScriptedDrawable"
            | "ScriptedLayout"
            | "ScriptedPathEffect"
            | "ScriptedListenerAction"
            | "ScriptedTransitionCondition"
            | "ScriptedInterpolator"
    )
}

fn definition_adds_cpp_state_machine_scripted_object(definition: &'static Definition) -> bool {
    matches!(
        definition.name,
        "ScriptedListenerAction" | "ScriptedTransitionCondition"
    )
}

fn listener_action_imports_successfully(action: &RuntimeObject, context: &ImportContext) -> bool {
    if listener_action_parent_kind_is_listener(action) {
        context.latest(ImportStackKey::StateMachineListener)
    } else {
        context.latest(ImportStackKey::StateMachineLayerComponent)
    }
}

fn state_machine_input_kind(definition: &'static Definition) -> Option<StateMachineInputKind> {
    match definition.name {
        "StateMachineBool" => Some(StateMachineInputKind::Bool),
        "StateMachineNumber" => Some(StateMachineInputKind::Number),
        "StateMachineTrigger" => Some(StateMachineInputKind::Trigger),
        _ => None,
    }
}

fn nested_input_kind(definition: &'static Definition) -> Option<StateMachineInputKind> {
    match definition.name {
        "NestedBool" => Some(StateMachineInputKind::Bool),
        "NestedNumber" => Some(StateMachineInputKind::Number),
        "NestedTrigger" => Some(StateMachineInputKind::Trigger),
        _ => None,
    }
}

fn transition_input_condition_is_invalid(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> bool {
    let Some(expected_kind) = transition_input_condition_kind(definition) else {
        return false;
    };
    let Some(input_id) = object.uint_property("inputId") else {
        return true;
    };
    let Some(actual_kind) = usize::try_from(input_id)
        .ok()
        .and_then(|index| context.state_machine_inputs.get(index))
    else {
        return true;
    };

    match actual_kind {
        Some(actual_kind) => *actual_kind != expected_kind,
        None => false,
    }
}

fn listener_input_change_is_invalid(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> bool {
    let Some(expected_kind) = listener_input_change_kind(definition) else {
        return false;
    };

    if let Some(nested_input_id) = object.uint_property("nestedInputId") {
        if nested_input_id != u64::from(u32::MAX) {
            if let Some(Some(actual_kind)) = usize::try_from(nested_input_id)
                .ok()
                .and_then(|index| context.artboard_local_nested_inputs.get(index))
            {
                return *actual_kind != expected_kind;
            }
        }
    }

    let Some(input_id) = object.uint_property("inputId") else {
        return false;
    };

    let Some(Some(actual_kind)) = usize::try_from(input_id)
        .ok()
        .and_then(|index| context.state_machine_inputs.get(index))
    else {
        return false;
    };

    *actual_kind != expected_kind
}

fn blend_input_is_invalid(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> bool {
    if definition.name == "BlendState1DInput" {
        let Some(input_id) = object.uint_property("inputId") else {
            return false;
        };
        if input_id == u64::from(u32::MAX) {
            return false;
        }
        return state_machine_input_is_not_number(context, input_id);
    }

    if definition.name == "BlendAnimationDirect" && object.uint_property("blendSource") == Some(0) {
        let Some(input_id) = object.uint_property("inputId") else {
            return true;
        };
        return state_machine_input_is_not_number(context, input_id);
    }

    false
}

fn state_machine_input_is_not_number(context: &ImportContext, input_id: u64) -> bool {
    let Some(input_kind) = usize::try_from(input_id)
        .ok()
        .and_then(|index| context.state_machine_inputs.get(index))
    else {
        return true;
    };

    *input_kind != Some(StateMachineInputKind::Number)
}

fn transition_input_condition_kind(
    definition: &'static Definition,
) -> Option<StateMachineInputKind> {
    match definition.name {
        "TransitionBoolCondition" => Some(StateMachineInputKind::Bool),
        "TransitionNumberCondition" => Some(StateMachineInputKind::Number),
        "TransitionTriggerCondition" => Some(StateMachineInputKind::Trigger),
        _ => None,
    }
}

fn listener_input_change_kind(definition: &'static Definition) -> Option<StateMachineInputKind> {
    match definition.name {
        "ListenerBoolChange" => Some(StateMachineInputKind::Bool),
        "ListenerNumberChange" => Some(StateMachineInputKind::Number),
        "ListenerTriggerChange" => Some(StateMachineInputKind::Trigger),
        _ => None,
    }
}

fn listener_action_parent_kind_is_listener(action: &RuntimeObject) -> bool {
    let raw = (action.uint_property("flags").unwrap_or(0) >> 1) & 0x3;
    raw == 0 || raw > 2
}

fn data_bind_target_is_cpp_state_machine_owned(target: Option<&RuntimeObject>) -> bool {
    let Some(target) = target else {
        return false;
    };
    matches!(
        target.type_name,
        "BindablePropertyNumber"
            | "BindablePropertyString"
            | "BindablePropertyBoolean"
            | "BindablePropertyEnum"
            | "BindablePropertyArtboard"
            | "BindablePropertyColor"
            | "BindablePropertyTrigger"
            | "BindablePropertyInteger"
            | "BindablePropertyAsset"
            | "BindablePropertyViewModel"
            | "BindablePropertyList"
            | "TransitionPropertyViewModelComparator"
            | "StateTransition"
    )
}

fn cpp_data_bind_artboard_owner(
    target: Option<CppDataBindTarget<'_>>,
    latest_artboard_index: Option<usize>,
    artboard_local_owners: &[Option<(usize, usize)>],
) -> Option<usize> {
    let Some(target) = target else {
        return None;
    };
    let Some(definition) = definition_by_type_key(target.object.type_key) else {
        return None;
    };

    if data_bind_target_is_cpp_state_machine_owned(Some(target.object))
        || definition.is_a("DataConverter")
        || definition.is_a("FormulaToken")
        || definition.name.starts_with("ScriptInput")
    {
        return None;
    }

    if definition.is_a("Component")
        && let Some(Some((artboard_index, _))) = artboard_local_owners.get(target.file_index)
    {
        return Some(*artboard_index);
    }

    latest_artboard_index
}

fn validate_cpp_import_resolution(
    objects: &[Option<RuntimeObject>],
    import_statuses: &[RuntimeImportStatus],
) -> Result<()> {
    for range in runtime_artboard_ranges(objects) {
        let mut slots = runtime_artboard_local_slots(objects, import_statuses, range);
        validate_cpp_artboard_local_slots(&mut slots, objects);

        validate_cpp_state_machine_layers(objects, import_statuses, range)?;
        validate_cpp_constraint_parentage(&slots, objects)?;
        validate_cpp_text_parentage(&slots, objects)?;
        validate_cpp_paint_effects(&slots, objects)?;

        for (local_index, slot) in slots.iter().enumerate() {
            let Some(file_index) = *slot else {
                continue;
            };
            let Some(object) = objects[file_index].as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };
            if definition.is_a("Drawable") {
                let raw_blend_mode = object.uint_property("blendModeValue").unwrap_or(3);
                let blend_mode = raw_blend_mode as u8;
                if !cpp_drawable_blend_mode_is_valid(blend_mode) {
                    bail!(
                        "drawable object {} ({}) has invalid blendModeValue {}",
                        object.id,
                        object.type_name,
                        raw_blend_mode
                    );
                }
            }

            if definition.name == "Mesh" {
                validate_cpp_mesh_indices(object, local_index, &slots, objects)?;
            }
        }
    }

    Ok(())
}

fn validate_cpp_constraint_parentage(
    slots: &[Option<usize>],
    objects: &[Option<RuntimeObject>],
) -> Result<()> {
    for slot in slots {
        let Some(file_index) = *slot else {
            continue;
        };
        let Some(object) = objects[file_index].as_ref() else {
            continue;
        };
        let Some(definition) = definition_by_type_key(object.type_key) else {
            continue;
        };
        if !definition.is_a("Constraint") {
            continue;
        }

        let Some(parent) = local_object_reference(slots, objects, object.uint_property("parentId"))
        else {
            continue;
        };
        if !runtime_object_is_cpp_transform_component(parent) {
            bail!(
                "constraint object {} ({}) has parent that is not TransformComponent",
                object.id,
                object.type_name
            );
        }

        if definition.name == "IKConstraint" && !runtime_object_is_cpp_bone(parent) {
            bail!(
                "IK constraint object {} ({}) has parent that is not Bone",
                object.id,
                object.type_name
            );
        }
    }

    Ok(())
}

fn validate_cpp_text_parentage(
    slots: &[Option<usize>],
    objects: &[Option<RuntimeObject>],
) -> Result<()> {
    for slot in slots {
        let Some(file_index) = *slot else {
            continue;
        };
        let Some(object) = objects[file_index].as_ref() else {
            continue;
        };
        let Some(definition) = definition_by_type_key(object.type_key) else {
            continue;
        };

        if matches!(definition.name, "TextStyleAxis" | "TextStyleFeature") {
            let Some(parent) =
                local_object_reference(slots, objects, object.uint_property("parentId"))
            else {
                continue;
            };
            if !runtime_object_is_cpp_text_style(parent) {
                bail!(
                    "text style child object {} ({}) has parent that is not TextStyle",
                    object.id,
                    object.type_name
                );
            }
        }

        if definition.is_a("TextInputDrawable") {
            let Some(parent) =
                local_object_reference(slots, objects, object.uint_property("parentId"))
            else {
                continue;
            };
            if !runtime_object_is_cpp_text_input(parent) {
                bail!(
                    "text input drawable object {} ({}) has parent that is not TextInput",
                    object.id,
                    object.type_name
                );
            }
        }
    }

    Ok(())
}

fn validate_cpp_paint_effects(
    slots: &[Option<usize>],
    objects: &[Option<RuntimeObject>],
) -> Result<()> {
    let mut paint_mutators = BTreeSet::new();

    for slot in slots {
        let Some(file_index) = *slot else {
            continue;
        };
        let Some(object) = objects[file_index].as_ref() else {
            continue;
        };
        let Some(definition) = definition_by_type_key(object.type_key) else {
            continue;
        };

        if definition.name == "Dash" {
            let Some(parent) =
                local_object_reference(slots, objects, object.uint_property("parentId"))
            else {
                continue;
            };
            if !runtime_object_is_cpp_dash_path(parent) {
                bail!("dash object {} has parent that is not DashPath", object.id);
            }
        }

        if cpp_stroke_effect_requires_effects_container(definition) {
            let Some(parent) =
                local_object_reference(slots, objects, object.uint_property("parentId"))
            else {
                continue;
            };
            if !runtime_object_is_cpp_effects_container(parent) {
                bail!(
                    "stroke effect object {} ({}) has parent that is not an effects container",
                    object.id,
                    object.type_name
                );
            }
        }

        if definition.name == "TrimPath" {
            let mode_value = object.uint_property("modeValue").unwrap_or(0);
            if !cpp_trim_path_mode_is_valid(mode_value) {
                bail!(
                    "trim path object {} has invalid modeValue {}",
                    object.id,
                    mode_value
                );
            }
        }

        if definition_is_cpp_shape_paint_mutator(definition) {
            let Some((parent_local_index, parent)) = local_object_reference_with_local_index(
                slots,
                objects,
                object.uint_property("parentId"),
            ) else {
                continue;
            };
            if !runtime_object_is_cpp_shape_paint(parent) {
                continue;
            }
            if !paint_mutators.insert(parent_local_index) {
                bail!(
                    "shape paint mutator object {} ({}) is a duplicate mutator for shape paint local slot {}",
                    object.id,
                    object.type_name,
                    parent_local_index
                );
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct CppStateMachineLayerResolution {
    object_id: u32,
    state_count: usize,
    has_any_state: bool,
    has_entry_state: bool,
    has_exit_state: bool,
    transitions: Vec<CppStateTransitionResolution>,
}

#[derive(Debug)]
struct CppStateTransitionResolution {
    object_id: u32,
    type_name: &'static str,
    state_to_id: u64,
}

fn validate_cpp_state_machine_layers(
    objects: &[Option<RuntimeObject>],
    import_statuses: &[RuntimeImportStatus],
    range: (usize, usize),
) -> Result<()> {
    let mut current_layer: Option<CppStateMachineLayerResolution> = None;

    for file_index in range.0..range.1 {
        let Some(object) = objects[file_index].as_ref() else {
            if import_statuses.get(file_index) == Some(&RuntimeImportStatus::NullObject) {
                if let Some(layer) = current_layer.as_mut() {
                    layer.state_count += 1;
                }
            }
            continue;
        };

        if import_statuses.get(file_index) != Some(&RuntimeImportStatus::Imported) {
            continue;
        }

        let Some(definition) = definition_by_type_key(object.type_key) else {
            continue;
        };

        if definition.name == "StateMachineLayer" {
            if let Some(layer) = current_layer.take() {
                validate_cpp_state_machine_layer_transitions(&layer)?;
            }
            current_layer = Some(CppStateMachineLayerResolution {
                object_id: object.id,
                state_count: 0,
                has_any_state: false,
                has_entry_state: false,
                has_exit_state: false,
                transitions: Vec::new(),
            });
            continue;
        }

        if definition.is_a("LayerState") {
            if let Some(layer) = current_layer.as_mut() {
                layer.state_count += 1;
                match definition.name {
                    "AnyState" => layer.has_any_state = true,
                    "EntryState" => layer.has_entry_state = true,
                    "ExitState" => layer.has_exit_state = true,
                    _ => {}
                }
            }
        }

        if definition.is_a("StateTransition") {
            if let Some(layer) = current_layer.as_mut() {
                layer.transitions.push(CppStateTransitionResolution {
                    object_id: object.id,
                    type_name: object.type_name,
                    state_to_id: object.uint_property("stateToId").unwrap_or(u64::MAX),
                });
            }
        }
    }

    if let Some(layer) = current_layer {
        validate_cpp_state_machine_layer_transitions(&layer)?;
    }

    Ok(())
}

fn validate_cpp_state_machine_layer_transitions(
    layer: &CppStateMachineLayerResolution,
) -> Result<()> {
    if !layer.has_any_state || !layer.has_entry_state || !layer.has_exit_state {
        bail!(
            "state machine layer {} is missing required AnyState/EntryState/ExitState objects",
            layer.object_id
        );
    }

    for transition in &layer.transitions {
        let Ok(state_to_id) = usize::try_from(transition.state_to_id) else {
            bail!(
                "state transition object {} ({}) targets state {} outside {} states in state machine layer {}",
                transition.object_id,
                transition.type_name,
                transition.state_to_id,
                layer.state_count,
                layer.object_id
            );
        };

        if state_to_id >= layer.state_count {
            bail!(
                "state transition object {} ({}) targets state {} outside {} states in state machine layer {}",
                transition.object_id,
                transition.type_name,
                transition.state_to_id,
                layer.state_count,
                layer.object_id
            );
        }
    }

    Ok(())
}

fn validate_cpp_mesh_indices(
    mesh: &RuntimeObject,
    mesh_local_index: usize,
    slots: &[Option<usize>],
    objects: &[Option<RuntimeObject>],
) -> Result<()> {
    let Some(index_bytes) = mesh.bytes_property("triangleIndexBytes") else {
        bail!(
            "mesh object {} ({}) is missing triangleIndexBytes",
            mesh.id,
            mesh.type_name
        );
    };

    let vertex_count = cpp_mesh_vertex_count(mesh_local_index, slots, objects);
    for index in decode_cpp_mesh_triangle_indices(index_bytes) {
        if usize::from(index) >= vertex_count {
            bail!(
                "mesh object {} ({}) has triangle index {} outside {} mesh vertices",
                mesh.id,
                mesh.type_name,
                index,
                vertex_count
            );
        }
    }

    Ok(())
}

fn cpp_skin_tendons<'a>(
    skin_local_id: usize,
    slots: &[Option<usize>],
    objects: &'a [Option<RuntimeObject>],
) -> Vec<RuntimeTendon<'a>> {
    slots
        .iter()
        .enumerate()
        .filter_map(|(local_id, slot)| {
            let object = slot.and_then(|file_index| objects[file_index].as_ref())?;
            if object.type_name != "Tendon"
                || object.uint_property("parentId") != Some(skin_local_id as u64)
            {
                return None;
            }

            let (bone_local_id, bone) = local_object_reference_with_local_index(
                slots,
                objects,
                object.uint_property("boneId"),
            )
            .filter(|(_, bone)| runtime_object_is_cpp_bone(bone))
            .map(|(local, bone)| (Some(local), Some(bone)))
            .unwrap_or((None, None));

            Some(RuntimeTendon {
                local_id,
                object,
                bone_local_id,
                bone,
            })
        })
        .collect()
}

struct RuntimeArtboardIndex<'a> {
    runtime_objects: &'a [Option<RuntimeObject>],
    local_slots: Vec<Option<usize>>,
    children_by_parent: Vec<Vec<usize>>,
    paths_by_shape: Vec<Vec<usize>>,
}

impl<'a> RuntimeArtboardIndex<'a> {
    fn new(
        runtime_objects: &'a [Option<RuntimeObject>],
        import_statuses: &[RuntimeImportStatus],
        range: (usize, usize),
    ) -> Self {
        let mut local_slots = runtime_artboard_local_slots(runtime_objects, import_statuses, range);
        validate_cpp_artboard_local_slots(&mut local_slots, runtime_objects);

        let mut children_by_parent = vec![Vec::new(); local_slots.len()];
        for (local_id, slot) in local_slots.iter().enumerate() {
            let Some(object) = slot.and_then(|file_index| runtime_objects[file_index].as_ref())
            else {
                continue;
            };
            let Some(parent) = object
                .uint_property("parentId")
                .and_then(|parent| usize::try_from(parent).ok())
                .filter(|parent| *parent < local_slots.len())
            else {
                continue;
            };
            children_by_parent[parent].push(local_id);
        }

        let mut index = Self {
            runtime_objects,
            paths_by_shape: vec![Vec::new(); local_slots.len()],
            local_slots,
            children_by_parent,
        };
        let path_owners = index
            .local_objects()
            .filter(|(_, object)| runtime_object_is_cpp_path(object))
            .filter_map(|(local_id, _)| {
                index
                    .path_owner_shape_local(local_id)
                    .map(|shape_local_id| (shape_local_id, local_id))
            })
            .collect::<Vec<_>>();
        for (shape_local_id, local_id) in path_owners {
            index.paths_by_shape[shape_local_id].push(local_id);
        }
        index
    }

    fn object(&self, local_id: usize) -> Option<&'a RuntimeObject> {
        let file_index = self.local_slots.get(local_id).copied().flatten()?;
        self.runtime_objects.get(file_index)?.as_ref()
    }

    fn local_objects(&self) -> impl Iterator<Item = (usize, &'a RuntimeObject)> + '_ {
        self.local_slots
            .iter()
            .enumerate()
            .filter_map(|(local_id, _)| self.object(local_id).map(|object| (local_id, object)))
    }

    fn children(
        &self,
        parent_local_id: usize,
    ) -> impl Iterator<Item = (usize, &'a RuntimeObject)> + '_ {
        self.children_by_parent
            .get(parent_local_id)
            .into_iter()
            .flatten()
            .filter_map(|local_id| self.object(*local_id).map(|object| (*local_id, object)))
    }

    fn path_owner_shape_local(&self, path_local_id: usize) -> Option<usize> {
        let mut current_local_id =
            usize::try_from(self.object(path_local_id)?.uint_property("parentId")?).ok()?;

        for _ in 0..100 {
            let object = self.object(current_local_id)?;
            if runtime_object_is_cpp_shape(object) {
                return Some(current_local_id);
            }

            let next_local_id = usize::try_from(object.uint_property("parentId")?).ok()?;
            if next_local_id == current_local_id {
                return None;
            }
            current_local_id = next_local_id;
        }

        None
    }

    fn meshes(&self) -> Vec<RuntimeMesh<'a>> {
        self.local_objects()
            .filter_map(|(local_id, object)| {
                (object.type_name == "Mesh").then(|| RuntimeMesh {
                    local_id,
                    object,
                    vertices: cpp_mesh_vertices(local_id, self),
                })
            })
            .collect()
    }

    fn paths(&self) -> Vec<RuntimePath<'a>> {
        self.local_objects()
            .filter_map(|(local_id, object)| {
                runtime_object_is_cpp_path(object).then(|| RuntimePath {
                    local_id,
                    object,
                    vertices: cpp_path_vertices(local_id, self),
                })
            })
            .collect()
    }

    fn shapes(&self) -> Vec<RuntimeShape<'a>> {
        self.local_objects()
            .filter_map(|(local_id, object)| {
                runtime_object_is_cpp_shape(object).then(|| RuntimeShape {
                    local_id,
                    object,
                    paths: cpp_shape_paths(local_id, self),
                    paints: cpp_shape_paints(local_id, self),
                })
            })
            .collect()
    }

    fn shape_paint_containers(&self) -> Vec<RuntimeShapePaintContainer<'a>> {
        self.local_objects()
            .filter_map(|(local_id, object)| {
                if !runtime_object_is_cpp_shape_paint_container(object) {
                    return None;
                }
                let paints = cpp_shape_paint_container_paints(local_id, self);
                (!paints.is_empty()).then_some(RuntimeShapePaintContainer {
                    local_id,
                    object,
                    paints,
                })
            })
            .collect()
    }

    fn n_slicer_details(&self) -> Vec<RuntimeNSlicerDetails<'a>> {
        self.local_objects()
            .filter_map(|(local_id, object)| {
                runtime_object_is_cpp_n_slicer_details(object).then(|| RuntimeNSlicerDetails {
                    local_id,
                    object,
                    x_axes: cpp_n_slicer_axes(local_id, "AxisX", self),
                    y_axes: cpp_n_slicer_axes(local_id, "AxisY", self),
                    tile_modes: cpp_n_slicer_tile_modes(local_id, self),
                })
            })
            .collect()
    }
}

fn cpp_mesh_vertices<'a>(
    mesh_local_id: usize,
    index: &RuntimeArtboardIndex<'a>,
) -> Vec<RuntimeMeshVertex<'a>> {
    index
        .children(mesh_local_id)
        .filter_map(|(local_id, object)| {
            if !runtime_object_is_cpp_mesh_vertex(object) {
                return None;
            }

            let (weight_local_id, weight) = cpp_vertex_weight(local_id, index)
                .map(|(local, weight)| (Some(local), Some(weight)))
                .unwrap_or((None, None));

            Some(RuntimeMeshVertex {
                local_id,
                object,
                weight_local_id,
                weight,
            })
        })
        .collect()
}

fn cpp_path_vertices<'a>(
    path_local_id: usize,
    index: &RuntimeArtboardIndex<'a>,
) -> Vec<RuntimePathVertex<'a>> {
    index
        .children(path_local_id)
        .filter_map(|(local_id, object)| {
            if !runtime_object_is_cpp_path_vertex(object) {
                return None;
            }

            let (weight_local_id, weight) = cpp_vertex_weight(local_id, index)
                .map(|(local, weight)| (Some(local), Some(weight)))
                .unwrap_or((None, None));

            Some(RuntimePathVertex {
                local_id,
                object,
                weight_local_id,
                weight,
            })
        })
        .collect()
}

fn cpp_n_slicer_axes<'a>(
    details_local_id: usize,
    axis_type_name: &'static str,
    index: &RuntimeArtboardIndex<'a>,
) -> Vec<RuntimeNSlicerAxis<'a>> {
    index
        .children(details_local_id)
        .filter_map(|(local_id, object)| {
            if object.type_name != axis_type_name {
                return None;
            }

            Some(RuntimeNSlicerAxis { local_id, object })
        })
        .collect()
}

fn cpp_n_slicer_tile_modes<'a>(
    details_local_id: usize,
    index: &RuntimeArtboardIndex<'a>,
) -> Vec<RuntimeNSlicerTileMode<'a>> {
    let mut tile_modes = BTreeMap::<u64, RuntimeNSlicerTileMode<'a>>::new();

    for (local_id, object) in index.children(details_local_id) {
        if object.type_name != "NSlicerTileMode" {
            continue;
        }

        let patch_index = object.uint_property("patchIndex").unwrap_or(0);
        tile_modes.insert(
            patch_index,
            RuntimeNSlicerTileMode {
                local_id,
                object,
                patch_index,
                style: object.uint_property("style").unwrap_or(0),
            },
        );
    }

    tile_modes.into_values().collect()
}

fn cpp_shape_paths<'a>(
    shape_local_id: usize,
    index: &RuntimeArtboardIndex<'a>,
) -> Vec<RuntimePath<'a>> {
    index
        .paths_by_shape
        .get(shape_local_id)
        .into_iter()
        .flatten()
        .filter_map(|local_id| {
            let object = index.object(*local_id)?;

            Some(RuntimePath {
                local_id: *local_id,
                object,
                vertices: cpp_path_vertices(*local_id, index),
            })
        })
        .collect()
}

fn cpp_shape_paints<'a>(
    shape_local_id: usize,
    index: &RuntimeArtboardIndex<'a>,
) -> Vec<RuntimeShapePaint<'a>> {
    cpp_shape_paint_container_paints(shape_local_id, index)
}

fn cpp_shape_paint_container_paints<'a>(
    container_local_id: usize,
    index: &RuntimeArtboardIndex<'a>,
) -> Vec<RuntimeShapePaint<'a>> {
    index
        .children(container_local_id)
        .filter_map(|(local_id, object)| {
            if !runtime_object_is_cpp_shape_paint(object) {
                return None;
            }

            let (mutator_local_id, mutator) = cpp_shape_paint_mutator(local_id, index)
                .map(|(local, mutator)| (Some(local), Some(mutator)))
                .unwrap_or((None, None));
            let mutator_local = mutator_local_id?;
            let (feather_local_id, feather) = cpp_shape_paint_feather(local_id, index)
                .map(|(local, feather)| (Some(local), Some(feather)))
                .unwrap_or((None, None));

            Some(RuntimeShapePaint {
                local_id,
                object,
                mutator_local_id,
                mutator,
                gradient_stops: cpp_gradient_stops(mutator_local, index),
                feather_local_id,
                feather,
                effects: cpp_shape_paint_effects(local_id, index),
            })
        })
        .collect()
}

fn cpp_shape_paint_mutator<'a>(
    shape_paint_local_id: usize,
    index: &RuntimeArtboardIndex<'a>,
) -> Option<(usize, &'a RuntimeObject)> {
    index
        .children(shape_paint_local_id)
        .find(|(_, object)| runtime_object_is_cpp_shape_paint_mutator(object))
}

fn cpp_shape_paint_feather<'a>(
    shape_paint_local_id: usize,
    index: &RuntimeArtboardIndex<'a>,
) -> Option<(usize, &'a RuntimeObject)> {
    index
        .children(shape_paint_local_id)
        .filter(|(_, object)| runtime_object_is_cpp_feather(object))
        .last()
}

fn cpp_shape_paint_effects<'a>(
    shape_paint_local_id: usize,
    index: &RuntimeArtboardIndex<'a>,
) -> Vec<RuntimeStrokeEffect<'a>> {
    let mut group_stack = BTreeSet::new();
    index
        .children(shape_paint_local_id)
        .filter_map(|(local_id, object)| {
            if !runtime_object_is_cpp_registered_stroke_effect(object) {
                return None;
            }

            Some(cpp_stroke_effect(local_id, object, index, &mut group_stack))
        })
        .collect()
}

fn cpp_stroke_effect<'a>(
    local_id: usize,
    object: &'a RuntimeObject,
    index: &RuntimeArtboardIndex<'a>,
    group_stack: &mut BTreeSet<usize>,
) -> RuntimeStrokeEffect<'a> {
    let target = cpp_target_effect_group_effect(object, index);
    let group_effects = target
        .filter(|(group_local, _)| group_stack.insert(*group_local))
        .map(|(group_local, _)| {
            let effects = cpp_group_effects(group_local, index, group_stack);
            group_stack.remove(&group_local);
            effects
        })
        .unwrap_or_default();

    RuntimeStrokeEffect {
        local_id,
        object,
        target_group_effect_local_id: target.map(|(local, _)| local),
        target_group_effect: target.map(|(_, group)| group),
        group_effects,
    }
}

fn cpp_group_effects<'a>(
    group_local_id: usize,
    index: &RuntimeArtboardIndex<'a>,
    group_stack: &mut BTreeSet<usize>,
) -> Vec<RuntimeStrokeEffect<'a>> {
    index
        .children(group_local_id)
        .filter_map(|(local_id, object)| {
            runtime_object_is_cpp_registered_stroke_effect(object)
                .then(|| cpp_stroke_effect(local_id, object, index, group_stack))
        })
        .collect()
}

fn cpp_target_effect_group_effect<'a>(
    effect: &RuntimeObject,
    index: &RuntimeArtboardIndex<'a>,
) -> Option<(usize, &'a RuntimeObject)> {
    if effect.type_name != "TargetEffect" {
        return None;
    }

    let local_id = usize::try_from(effect.uint_property("targetId")?).ok()?;
    let target = index.object(local_id)?;
    runtime_object_is_cpp_group_effect(target).then_some((local_id, target))
}

fn cpp_gradient_stops<'a>(
    gradient_local_id: usize,
    index: &RuntimeArtboardIndex<'a>,
) -> Vec<RuntimeGradientStop<'a>> {
    let Some(gradient) = index.object(gradient_local_id) else {
        return Vec::new();
    };
    if !runtime_object_is_cpp_linear_gradient(gradient) {
        return Vec::new();
    }

    index
        .children(gradient_local_id)
        .filter_map(|(local_id, object)| {
            if !runtime_object_is_cpp_gradient_stop(object) {
                return None;
            }

            Some(RuntimeGradientStop { local_id, object })
        })
        .collect()
}

fn cpp_vertex_weight<'a>(
    vertex_local_id: usize,
    index: &RuntimeArtboardIndex<'a>,
) -> Option<(usize, &'a RuntimeObject)> {
    index
        .children(vertex_local_id)
        .filter(|(_, object)| runtime_object_is_cpp_weight(object))
        .last()
}

fn cpp_mesh_vertex_count(
    mesh_local_index: usize,
    slots: &[Option<usize>],
    objects: &[Option<RuntimeObject>],
) -> usize {
    slots
        .iter()
        .flatten()
        .filter_map(|file_index| objects[*file_index].as_ref())
        .filter(|object| {
            runtime_object_is_cpp_mesh_vertex(object)
                && object.uint_property("parentId") == Some(mesh_local_index as u64)
        })
        .count()
}

fn decode_cpp_mesh_triangle_indices(bytes: &[u8]) -> Vec<u16> {
    let mut indices = Vec::new();
    let mut offset = 0usize;

    while offset < bytes.len() {
        let mut result = 0u64;
        let mut shift = 0u8;

        loop {
            let Some(byte) = bytes.get(offset) else {
                indices.push(0);
                offset = bytes.len();
                break;
            };
            offset += 1;
            result |= u64::from(byte & 0x7f).wrapping_shl(u32::from(shift));

            if byte & 0x80 == 0 {
                if result > u64::from(u16::MAX) {
                    indices.push(0);
                    offset = bytes.len();
                } else {
                    indices.push(result as u16);
                }
                break;
            }

            shift = shift.wrapping_add(7);
        }
    }

    indices
}

fn runtime_artboard_ranges(objects: &[Option<RuntimeObject>]) -> Vec<(usize, usize)> {
    let starts = objects
        .iter()
        .enumerate()
        .filter_map(|(index, object)| match object {
            Some(object) if object.type_name == "Artboard" => Some(index),
            _ => None,
        })
        .collect::<Vec<_>>();

    starts
        .iter()
        .enumerate()
        .map(|(index, start)| {
            (
                *start,
                starts.get(index + 1).copied().unwrap_or(objects.len()),
            )
        })
        .collect()
}

fn runtime_artboard_local_slots(
    objects: &[Option<RuntimeObject>],
    import_statuses: &[RuntimeImportStatus],
    range: (usize, usize),
) -> Vec<Option<usize>> {
    objects[range.0..range.1]
        .iter()
        .enumerate()
        .filter_map(|(relative_index, object)| {
            let file_index = range.0 + relative_index;
            match object {
                None => Some(None),
                Some(object) if runtime_object_is_cpp_artboard_local(object) => {
                    if import_statuses.get(file_index) == Some(&RuntimeImportStatus::Imported) {
                        Some(Some(file_index))
                    } else {
                        Some(None)
                    }
                }
                Some(_) => None,
            }
        })
        .collect()
}

fn validate_cpp_artboard_local_slots(
    slots: &mut [Option<usize>],
    objects: &[Option<RuntimeObject>],
) {
    for _ in 0..100 {
        let mut changed = false;
        for index in 1..slots.len() {
            if !cpp_artboard_local_slot_is_valid(index, slots, objects) {
                slots[index] = None;
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }
}

fn cpp_artboard_local_slot_is_valid(
    index: usize,
    slots: &[Option<usize>],
    objects: &[Option<RuntimeObject>],
) -> bool {
    let Some(file_index) = slots[index] else {
        return true;
    };
    let Some(object) = objects[file_index].as_ref() else {
        return true;
    };
    let Some(definition) = definition_by_type_key(object.type_key) else {
        return false;
    };

    if definition.name == "Artboard" {
        return true;
    }

    if definition.is_a("Component") {
        let Some(parent) = local_object_reference(slots, objects, object.uint_property("parentId"))
        else {
            return false;
        };
        if !runtime_object_is_container_component(parent) {
            return false;
        }
    }

    if definition.is_a("TargetedConstraint") {
        return match local_object_reference(slots, objects, object.uint_property("targetId")) {
            Some(target) => runtime_object_is_cpp_transform_component(target),
            None => !cpp_targeted_constraint_requires_target(definition.name),
        };
    }

    if definition.is_a("NestedAnimation") {
        let Some(parent) = local_object_reference(slots, objects, object.uint_property("parentId"))
        else {
            return false;
        };
        return runtime_object_is_cpp_nested_artboard(parent);
    }

    if definition.is_a("TextStyle") {
        let Some(parent) = local_object_reference(slots, objects, object.uint_property("parentId"))
        else {
            return false;
        };
        return runtime_object_is_cpp_text_interface(parent);
    }

    if definition.name == "ScrollBarConstraint" {
        let Some(scroll_constraint) =
            local_object_reference(slots, objects, object.uint_property("scrollConstraintId"))
        else {
            return false;
        };
        return runtime_object_is_cpp_scroll_constraint(scroll_constraint);
    }

    if definition.name == "Feather" {
        let Some(parent) = local_object_reference(slots, objects, object.uint_property("parentId"))
        else {
            return false;
        };
        return runtime_object_is_cpp_shape_paint(parent);
    }

    if definition.name == "ArtboardListMapRule" {
        let Some(parent) = local_object_reference(slots, objects, object.uint_property("parentId"))
        else {
            return false;
        };
        return runtime_object_is_cpp_artboard_component_list(parent);
    }

    true
}

fn local_object_reference<'a>(
    slots: &[Option<usize>],
    objects: &'a [Option<RuntimeObject>],
    id: Option<u64>,
) -> Option<&'a RuntimeObject> {
    let id = usize::try_from(id?).ok()?;
    let file_index = slots.get(id).and_then(|slot| *slot)?;
    objects.get(file_index).and_then(|object| object.as_ref())
}

fn local_object_reference_with_local_index<'a>(
    slots: &[Option<usize>],
    objects: &'a [Option<RuntimeObject>],
    id: Option<u64>,
) -> Option<(usize, &'a RuntimeObject)> {
    let local_index = usize::try_from(id?).ok()?;
    let file_index = slots.get(local_index).and_then(|slot| *slot)?;
    let object = objects.get(file_index).and_then(|object| object.as_ref())?;
    Some((local_index, object))
}

fn cpp_keyed_object_target<'a>(
    keyed_object: &RuntimeObject,
    slots: &[Option<usize>],
    objects: &'a [Option<RuntimeObject>],
) -> Option<&'a RuntimeObject> {
    local_object_reference(slots, objects, keyed_object.uint_property("objectId"))
}

fn cpp_keyed_object_supports_property(
    keyed_object: &RuntimeObject,
    keyed_property: &RuntimeObject,
    slots: &[Option<usize>],
    objects: &[Option<RuntimeObject>],
) -> bool {
    let Some(property_key) = keyed_property.uint_property("propertyKey") else {
        return false;
    };
    let Ok(property_key) = u16::try_from(property_key) else {
        return false;
    };
    let Some(target) = cpp_keyed_object_target(keyed_object, slots, objects) else {
        return false;
    };

    object_supports_property(target.type_key, property_key)
}

fn runtime_object_is_cpp_artboard_local(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(definition_is_cpp_artboard_local)
}

fn definition_is_cpp_artboard_local(definition: &'static Definition) -> bool {
    // Component-owned ScriptInputs call Component::import in C++ and occupy
    // artboard slots; inputs owned by non-components fail parent validation.
    (definition.is_a("Component") && !definition.is_a("ScrollPhysics"))
        || definition.is_a("KeyFrameInterpolator")
        || definition.is_a("UserInput")
}

fn runtime_object_is_container_component(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("ContainerComponent"))
}

fn runtime_object_is_cpp_artboard_component_list(object: &RuntimeObject) -> bool {
    object.type_name == "ArtboardComponentList"
}

fn runtime_object_is_cpp_transform_component(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("TransformComponent"))
}

fn runtime_object_is_cpp_bone(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("Bone"))
}

fn runtime_object_is_cpp_mesh_vertex(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("MeshVertex"))
}

fn runtime_object_is_cpp_path(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("Path"))
}

fn runtime_object_is_cpp_path_vertex(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("PathVertex"))
}

fn runtime_object_is_cpp_shape(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("Shape"))
}

fn runtime_object_is_cpp_weight(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("Weight"))
}

fn runtime_object_is_cpp_skinnable(object: &RuntimeObject) -> bool {
    matches!(object.type_name, "Mesh" | "PointsPath")
}

fn runtime_object_is_cpp_nested_artboard(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("NestedArtboard"))
}

fn runtime_object_is_cpp_dash_path(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.name == "DashPath")
}

fn runtime_object_is_cpp_effects_container(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("ShapePaint") || definition.name == "GroupEffect")
}

fn runtime_object_is_cpp_shape_paint(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("ShapePaint"))
}

fn runtime_object_is_cpp_shape_paint_container(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| {
        matches!(
            definition.name,
            "Artboard"
                | "LayoutComponent"
                | "Shape"
                | "TextStylePaint"
                | "ForegroundLayoutDrawable"
                | "TextInputCursor"
                | "TextInputSelection"
                | "TextInputText"
                | "TextInputSelectedText"
        )
    })
}

fn runtime_object_is_cpp_n_slicer_details(object: &RuntimeObject) -> bool {
    cpp_type_name_is_n_slicer_details(object.type_name)
}

fn cpp_type_name_is_n_slicer_details(type_name: &'static str) -> bool {
    matches!(type_name, "NSlicer" | "NSlicedNode")
}

fn runtime_object_is_cpp_shape_paint_mutator(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(definition_is_cpp_shape_paint_mutator)
}

fn runtime_object_is_cpp_linear_gradient(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("LinearGradient"))
}

fn runtime_object_is_cpp_gradient_stop(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.name == "GradientStop")
}

fn runtime_object_is_cpp_feather(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.name == "Feather")
}

fn runtime_object_is_cpp_registered_stroke_effect(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(cpp_stroke_effect_requires_effects_container)
}

fn runtime_object_is_cpp_group_effect(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.name == "GroupEffect")
}

fn runtime_object_is_cpp_text_style(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("TextStyle"))
}

fn runtime_object_is_cpp_text_interface(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| matches!(definition.name, "Text" | "TextInput"))
}

fn runtime_object_is_cpp_text_input(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.name == "TextInput")
}

fn runtime_object_is_cpp_scroll_constraint(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.name == "ScrollConstraint")
}

fn cpp_targeted_constraint_requires_target(type_name: &str) -> bool {
    !matches!(
        type_name,
        "RotationConstraint" | "ScaleConstraint" | "TranslationConstraint"
    )
}

fn cpp_stroke_effect_requires_effects_container(definition: &'static Definition) -> bool {
    matches!(
        definition.name,
        "DashPath" | "TargetEffect" | "TrimPath" | "ScriptedPathEffect"
    )
}

fn definition_is_cpp_shape_paint_mutator(definition: &'static Definition) -> bool {
    definition.name == "SolidColor" || definition.is_a("LinearGradient")
}

fn cpp_trim_path_mode_is_valid(value: u64) -> bool {
    matches!(value, 1 | 2)
}

fn cpp_drawable_blend_mode_is_valid(value: u8) -> bool {
    matches!(value, 3 | 14..=28)
}

fn apply_cpp_import_mutations(
    objects: &mut [Option<RuntimeObject>],
    import_statuses: &[RuntimeImportStatus],
) {
    normalize_file_asset_ids(objects, import_statuses);
}

fn normalize_file_asset_ids(
    objects: &mut [Option<RuntimeObject>],
    import_statuses: &[RuntimeImportStatus],
) {
    let mut file_asset_ids = Vec::new();
    for index in 0..objects.len() {
        if import_statuses.get(index) != Some(&RuntimeImportStatus::Imported) {
            continue;
        }

        let Some(object) = objects[index].as_ref() else {
            continue;
        };
        let is_file_asset = definition_by_type_key(object.type_key)
            .is_some_and(|definition| definition.is_a("FileAsset"));
        if !is_file_asset {
            continue;
        }

        file_asset_ids.push(index);
        normalize_file_asset_ids_for_imported_assets(objects, &file_asset_ids);
    }
}

fn normalize_file_asset_ids_for_imported_assets(
    objects: &mut [Option<RuntimeObject>],
    file_asset_ids: &[usize],
) {
    let mut ids = std::collections::BTreeSet::new();
    let mut next_id = 1u32;

    for object_id in file_asset_ids {
        let object = objects[*object_id]
            .as_mut()
            .expect("file_asset_ids only contains present objects");
        let asset_id = object.uint_property("assetId").unwrap_or(0) as u32;
        if ids.contains(&asset_id) {
            set_runtime_uint_property(object, 204, "assetId", "FileAsset", u64::from(next_id));
        } else {
            ids.insert(asset_id);
            if asset_id >= next_id {
                next_id = asset_id.wrapping_add(1);
            }
        }
    }
}

fn set_runtime_uint_property(
    object: &mut RuntimeObject,
    key: u16,
    name: &'static str,
    owner: &'static str,
    value: u64,
) {
    upsert_runtime_property(
        &mut object.properties,
        RuntimeProperty {
            key,
            name,
            owner,
            value: FieldValue::Uint(value),
        },
    );
}

fn read_runtime_object(
    reader: &mut BinaryReader<'_>,
    header: &RuntimeHeader,
    id: u32,
) -> Result<Option<RuntimeObject>> {
    let raw_type_key = read_cpp_int_var_uint(reader, "object type key")?;
    let definition_type_key = u16::try_from(raw_type_key).ok();
    let definition = definition_type_key
        .and_then(definition_by_type_key)
        .filter(|definition| !definition.abstract_);

    let mut properties = Vec::new();
    let mut skipped_properties = Vec::new();

    loop {
        let property_key = to_u16(reader.read_var_uint()?, "property key")?;
        if property_key == 0 {
            break;
        }

        let Some(definition) = definition else {
            let field = match skip_unknown_property(reader, header, property_key)
                .with_context(|| format!("skipping property {property_key} on unknown object"))?
            {
                UnknownPropertySkip::Skipped { field }
                | UnknownPropertySkip::UnhandledKnownField { field } => field,
                UnknownPropertySkip::MissingToc => return Ok(None),
            };
            skipped_properties.push(SkippedProperty {
                key: property_key,
                name: None,
                owner: None,
                reason: SkipReason::UnknownObject,
                field,
                value: None,
                bitmask_passthrough: None,
            });
            continue;
        };

        let Some((owner, property)) =
            property_by_primary_key_in_hierarchy(definition, property_key)
        else {
            let field = match skip_unknown_property(reader, header, property_key)? {
                UnknownPropertySkip::Skipped { field }
                | UnknownPropertySkip::UnhandledKnownField { field } => field,
                UnknownPropertySkip::MissingToc => return Ok(None),
            };
            skipped_properties.push(SkippedProperty {
                key: property_key,
                name: None,
                owner: None,
                reason: SkipReason::UnknownProperty,
                field,
                value: None,
                bitmask_passthrough: None,
            });
            continue;
        };

        if property.deserializes {
            let value = read_field_value(reader, property)?;
            upsert_runtime_property(
                &mut properties,
                RuntimeProperty {
                    key: property_key,
                    name: property.name,
                    owner,
                    value,
                },
            );
        } else {
            let skipped =
                skip_known_non_deserialized_property(reader, header, property_key, property)?;
            let (field, value) = match skipped {
                KnownPropertySkip::Skipped { field, value } => (field, value),
                KnownPropertySkip::UnhandledKnownField { field } => (field, None),
                KnownPropertySkip::MissingToc => return Ok(None),
            };
            skipped_properties.push(SkippedProperty {
                key: property_key,
                name: Some(property.name),
                owner: Some(owner),
                reason: skip_reason_for_property(property),
                field,
                value,
                bitmask_passthrough: property.bitmask_passthrough.map(Into::into),
            });
        }
    }

    let Some(definition) = definition else {
        return Ok(None);
    };

    Ok(Some(RuntimeObject {
        id,
        type_key: definition_type_key.expect("known generated type keys fit in u16"),
        type_name: definition.name,
        rust_variant: definition.rust_variant,
        properties,
        skipped_properties,
    }))
}

fn upsert_runtime_property(properties: &mut Vec<RuntimeProperty>, property: RuntimeProperty) {
    if let Some(existing) = properties
        .iter_mut()
        .find(|existing| existing.key == property.key)
    {
        *existing = property;
    } else {
        properties.push(property);
    }
}

fn property_by_primary_key_in_hierarchy(
    definition: &'static Definition,
    key: u16,
) -> Option<(&'static str, &'static Property)> {
    definition
        .properties
        .iter()
        .find(|property| property.key.int == key)
        .map(|property| (definition.name, property))
        .or_else(|| {
            definition.ancestors.iter().find_map(|ancestor| {
                let definition = nuxie_schema::definition_by_name(ancestor)?;
                definition
                    .properties
                    .iter()
                    .find(|property| property.key.int == key)
                    .map(|property| (*ancestor, property))
            })
        })
}

fn property_by_name_in_hierarchy(
    definition: &'static Definition,
    name: &str,
) -> Option<&'static Property> {
    definition
        .properties
        .iter()
        .find(|property| property.name == name)
        .or_else(|| {
            definition.ancestors.iter().find_map(|ancestor| {
                let definition = nuxie_schema::definition_by_name(ancestor)?;
                definition
                    .properties
                    .iter()
                    .find(|property| property.name == name)
            })
        })
}

enum UnknownPropertySkip {
    Skipped { field: Option<&'static str> },
    UnhandledKnownField { field: Option<&'static str> },
    MissingToc,
}

enum KnownPropertySkip {
    Skipped {
        field: Option<&'static str>,
        value: Option<FieldValue>,
    },
    UnhandledKnownField {
        field: Option<&'static str>,
    },
    MissingToc,
}

fn skip_known_non_deserialized_property(
    reader: &mut BinaryReader<'_>,
    header: &RuntimeHeader,
    key: u16,
    property: &Property,
) -> Result<KnownPropertySkip> {
    if let Some(field) = core_registry_field_kind_by_property_key(key) {
        return read_core_registry_fallback_value(reader, field, property);
    }

    let Some(field) = header.field_for_property(key) else {
        return Ok(KnownPropertySkip::MissingToc);
    };
    let value = read_header_fallback_value(reader, field, property)?;
    Ok(KnownPropertySkip::Skipped {
        field: Some(header_field_name(field)),
        value: Some(value),
    })
}

fn skip_unknown_property(
    reader: &mut BinaryReader<'_>,
    header: &RuntimeHeader,
    key: u16,
) -> Result<UnknownPropertySkip> {
    if let Some(field) = core_registry_field_kind_by_property_key(key) {
        return skip_core_registry_value(reader, field);
    }

    let Some(field) = header.field_for_property(key) else {
        return Ok(UnknownPropertySkip::MissingToc);
    };
    skip_header_value(reader, field)?;
    Ok(UnknownPropertySkip::Skipped {
        field: Some(header_field_name(field)),
    })
}

fn skip_core_registry_value(
    reader: &mut BinaryReader<'_>,
    field: CoreRegistryFieldKind,
) -> Result<UnknownPropertySkip> {
    match field {
        CoreRegistryFieldKind::Uint => {
            // Uint64 deliberately shares the uint field id. Unknown fields do
            // not carry enough schema to select a narrower C++ storage type,
            // so consume the full raw varuint64 just like File::readRuntimeObject.
            reader.read_var_uint()?;
        }
        CoreRegistryFieldKind::StringOrBytes => {
            reader.read_string()?;
        }
        CoreRegistryFieldKind::Double => {
            reader.read_f32()?;
        }
        CoreRegistryFieldKind::Color => {
            reader.read_u32()?;
        }
        CoreRegistryFieldKind::Bool => {
            return Ok(UnknownPropertySkip::UnhandledKnownField {
                field: Some(core_registry_field_name(field)),
            });
        }
    }

    Ok(UnknownPropertySkip::Skipped {
        field: Some(core_registry_field_name(field)),
    })
}

fn read_core_registry_fallback_value(
    reader: &mut BinaryReader<'_>,
    field: CoreRegistryFieldKind,
    property: &Property,
) -> Result<KnownPropertySkip> {
    let value = match field {
        CoreRegistryFieldKind::Uint => {
            FieldValue::Uint(read_known_uint_field(reader, property, "uint field")?)
        }
        CoreRegistryFieldKind::StringOrBytes => read_string_or_bytes_value(reader, property)?,
        CoreRegistryFieldKind::Double => FieldValue::Double(reader.read_f32()?),
        CoreRegistryFieldKind::Color => FieldValue::Color(reader.read_u32()?),
        CoreRegistryFieldKind::Bool => {
            return Ok(KnownPropertySkip::UnhandledKnownField {
                field: Some(core_registry_field_name(field)),
            });
        }
    };

    Ok(KnownPropertySkip::Skipped {
        field: Some(core_registry_field_name(field)),
        value: Some(value),
    })
}

fn read_header_fallback_value(
    reader: &mut BinaryReader<'_>,
    field: HeaderFieldKind,
    property: &Property,
) -> Result<FieldValue> {
    Ok(match field {
        HeaderFieldKind::Uint => FieldValue::Uint(read_known_uint_field(
            reader,
            property,
            "header uint field",
        )?),
        HeaderFieldKind::StringOrBytes => read_string_or_bytes_value(reader, property)?,
        HeaderFieldKind::Double => FieldValue::Double(reader.read_f32()?),
        HeaderFieldKind::Color => FieldValue::Color(reader.read_u32()?),
    })
}

fn read_string_or_bytes_value(
    reader: &mut BinaryReader<'_>,
    property: &Property,
) -> Result<FieldValue> {
    if property.runtime_type == FieldKind::Bytes {
        let bytes = reader.read_length_prefixed_bytes()?;
        Ok(FieldValue::Bytes(BytesValue::new(bytes.to_vec())))
    } else {
        Ok(FieldValue::String(reader.read_string()?))
    }
}

fn read_field_value(reader: &mut BinaryReader<'_>, property: &Property) -> Result<FieldValue> {
    Ok(match property.runtime_type {
        FieldKind::Bool => FieldValue::Bool(reader.read_byte()? == 1),
        FieldKind::Bytes => {
            let bytes = reader.read_length_prefixed_bytes()?;
            FieldValue::Bytes(BytesValue::new(bytes.to_vec()))
        }
        FieldKind::Callback => FieldValue::Callback,
        FieldKind::Color => FieldValue::Color(reader.read_u32()?),
        FieldKind::Double => FieldValue::Double(reader.read_f32()?),
        FieldKind::String => FieldValue::String(reader.read_string()?),
        FieldKind::Uint => FieldValue::Uint(read_known_uint_field(reader, property, "uint field")?),
    })
}

fn skip_header_value(reader: &mut BinaryReader<'_>, kind: HeaderFieldKind) -> Result<()> {
    match kind {
        HeaderFieldKind::Uint => {
            // Header field id 0 is shared by uint32 and uint64. Unknown
            // properties therefore have to consume the full raw varuint.
            reader.read_var_uint()?;
        }
        HeaderFieldKind::StringOrBytes => {
            reader.read_length_prefixed_bytes()?;
        }
        HeaderFieldKind::Double => {
            reader.read_f32()?;
        }
        HeaderFieldKind::Color => {
            reader.read_u32()?;
        }
    }
    Ok(())
}

fn skip_reason_for_property(property: &Property) -> SkipReason {
    if property.bitmask_passthrough.is_some() {
        SkipReason::BitmaskPassthroughProperty
    } else if property.passthrough {
        SkipReason::PassthroughProperty
    } else {
        SkipReason::NonStoredProperty
    }
}

fn preview_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(12)
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn decode_cpp_u32_id_list(bytes: &[u8]) -> Vec<u32> {
    let mut ids = Vec::new();
    let mut offset = 0;

    while offset < bytes.len() {
        let (value, len) = read_cpp_embedded_var_uint64(&bytes[offset..]);
        if len == 0 {
            ids.push(0);
            break;
        }
        offset += len;

        if value > u64::from(u32::MAX) {
            ids.push(0);
            break;
        }

        ids.push(value as u32);
    }

    ids
}

fn read_cpp_embedded_var_uint64(bytes: &[u8]) -> (u64, usize) {
    let mut result = 0u64;
    let mut shift = 0u8;

    for (index, byte) in bytes.iter().copied().enumerate() {
        result |= u64::from(byte & 0x7f).wrapping_shl(u32::from(shift));
        shift = shift.wrapping_add(7);

        if byte & 0x80 == 0 {
            return (result, index + 1);
        }
    }

    (0, 0)
}

fn format_cpp_file_asset_cdn_uuid(bytes: &[u8]) -> String {
    if bytes.len() != 16 {
        return String::new();
    }

    const INDICES: [usize; 16] = [3, 2, 1, 0, 5, 4, 7, 6, 9, 8, 15, 14, 13, 12, 11, 10];

    let mut uuid = String::with_capacity(36);
    for index in INDICES {
        uuid.push_str(&format!("{:02x}", bytes[index]));
        if matches!(index, 0 | 4 | 6 | 8) {
            uuid.push('-');
        }
    }
    uuid
}

fn parse_cpp_manifest_asset(bytes: &[u8]) -> RuntimeManifest {
    let mut manifest = RuntimeManifest::default();
    if bytes.is_empty() {
        return manifest;
    }

    let mut reader = BinaryReader::new(bytes);
    while !reader.reached_end() {
        let Ok(section) = reader.read_var_uint() else {
            return manifest;
        };
        let section_size = match reader
            .read_var_uint()
            .ok()
            .and_then(|value| usize::try_from(value).ok())
        {
            Some(value) => value,
            None => return manifest,
        };
        let section_start = reader.offset;

        let decoded = match section {
            0 => decode_cpp_manifest_names(&mut reader, &mut manifest),
            1 => decode_cpp_manifest_paths(&mut reader, &mut manifest),
            _ => {
                if reader.read_bytes_exact(section_size).is_err() {
                    return manifest;
                }
                continue;
            }
        };

        if decoded.is_err() {
            return manifest;
        }

        let bytes_read = reader.offset - section_start;
        if bytes_read != section_size {
            return manifest;
        }
    }

    manifest
}

fn decode_cpp_manifest_names(
    reader: &mut BinaryReader<'_>,
    manifest: &mut RuntimeManifest,
) -> Result<()> {
    let count = reader.read_var_uint()?;
    for _ in 0..count {
        let id = cpp_manifest_key(reader.read_var_uint()?);
        let value = reader.read_string()?;
        manifest.names.insert(id, value);
    }
    Ok(())
}

fn decode_cpp_manifest_paths(
    reader: &mut BinaryReader<'_>,
    manifest: &mut RuntimeManifest,
) -> Result<()> {
    let count = reader.read_var_uint()?;
    for _ in 0..count {
        let id = cpp_manifest_key(reader.read_var_uint()?);
        let path_len = reader.read_var_uint()?;
        let mut path = Vec::new();
        for _ in 0..path_len {
            path.push(read_cpp_manifest_path_id(reader));
        }
        manifest.paths.insert(id, path);
    }
    Ok(())
}

// Manifest name/path maps are keyed by a *signed* int in C++
// (`DataResolver::resolveName(int id)`, include/rive/data_resolver.hpp). The
// runtime id arrives as an unsigned var-uint, so we deliberately reinterpret the
// low 32 bits as i32 (`as i32` is a bit-preserving truncate/reinterpret in Rust,
// NOT saturating) to match C++'s key space exactly -- an id above i32::MAX must
// wrap to the same negative key on both insert and lookup. Insert path.
fn cpp_manifest_key(value: u64) -> i32 {
    value as i32
}

// Lookup counterpart to cpp_manifest_key: same intentional u32->i32
// reinterpret, so `resolve_*` finds keys inserted by decode_cpp_manifest_*.
// See cpp_manifest_key above and the pinning test cpp_manifest_key_reinterpret.
fn cpp_manifest_resolver_key(value: u32) -> i32 {
    value as i32
}

fn read_cpp_manifest_path_id(reader: &mut BinaryReader<'_>) -> u32 {
    match reader.read_var_uint() {
        Ok(value) => value as u32,
        Err(_) => {
            reader.offset = reader.bytes.len();
            0
        }
    }
}

fn core_registry_field_name(kind: CoreRegistryFieldKind) -> &'static str {
    match kind {
        CoreRegistryFieldKind::Uint => "uint",
        CoreRegistryFieldKind::StringOrBytes => "stringOrBytes",
        CoreRegistryFieldKind::Double => "double",
        CoreRegistryFieldKind::Color => "color",
        CoreRegistryFieldKind::Bool => "bool",
    }
}

fn header_field_name(kind: HeaderFieldKind) -> &'static str {
    match kind {
        HeaderFieldKind::Uint => "uint",
        HeaderFieldKind::StringOrBytes => "stringOrBytes",
        HeaderFieldKind::Double => "double",
        HeaderFieldKind::Color => "color",
    }
}

fn to_u16(value: u64, label: &str) -> Result<u16> {
    u16::try_from(value).with_context(|| format!("{label} {value} does not fit in u16"))
}

fn read_cpp_int_var_uint(reader: &mut BinaryReader<'_>, label: &str) -> Result<u64> {
    let value = reader.read_var_uint()?;
    if value > i32::MAX as u64 {
        bail!("{label} {value} does not fit in C++ int");
    }
    Ok(value)
}

fn read_cpp_unsigned_int_var_uint(reader: &mut BinaryReader<'_>, label: &str) -> Result<u64> {
    let value = reader.read_var_uint()?;
    if value > u32::MAX as u64 {
        bail!("{label} {value} does not fit in C++ unsigned int");
    }
    Ok(value)
}

fn read_known_uint_field(
    reader: &mut BinaryReader<'_>,
    property: &Property,
    label: &str,
) -> Result<u64> {
    match property.uint_storage() {
        Some(UintStorage::Uint64) => reader.read_var_uint(),
        Some(UintStorage::Uint8) => {
            read_cpp_unsigned_int_var_uint(reader, label).map(|value| u64::from(value as u8))
        }
        Some(UintStorage::Uint32) => read_cpp_unsigned_int_var_uint(reader, label),
        None => bail!("{label} schema property is not uint-like"),
    }
}

struct BinaryReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> BinaryReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn reached_end(&self) -> bool {
        self.offset == self.bytes.len()
    }

    fn read_byte(&mut self) -> Result<u8> {
        let byte = *self
            .bytes
            .get(self.offset)
            .with_context(|| format!("read past end at byte {}", self.offset))?;
        self.offset += 1;
        Ok(byte)
    }

    fn read_bytes_exact(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(len)
            .context("byte offset overflow")?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .with_context(|| format!("read {len} bytes past end at byte {}", self.offset))?;
        self.offset = end;
        Ok(bytes)
    }

    fn read_length_prefixed_bytes(&mut self) -> Result<&'a [u8]> {
        let len = usize::try_from(self.read_var_uint()?).context("length does not fit in usize")?;
        self.read_bytes_exact(len)
    }

    fn read_string(&mut self) -> Result<StringValue> {
        let bytes = self.read_length_prefixed_bytes()?;
        let raw = bytes.to_vec();
        let value = String::from_utf8(raw.clone()).ok();
        Ok(StringValue { value, raw })
    }

    fn read_f32(&mut self) -> Result<f32> {
        let bytes: [u8; 4] = self.read_bytes_exact(4)?.try_into().unwrap();
        Ok(f32::from_le_bytes(bytes))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let bytes: [u8; 4] = self.read_bytes_exact(4)?.try_into().unwrap();
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_var_uint(&mut self) -> Result<u64> {
        let mut result = 0u64;
        let mut shift = 0u8;

        loop {
            let byte = self.read_byte()?;
            result |= u64::from(byte & 0x7f).wrapping_shl(u32::from(shift));

            if byte & 0x80 == 0 {
                return Ok(result);
            }

            shift = shift.wrapping_add(7);
        }
    }
}

#[cfg(test)]
mod uint_wire_tests {
    use super::{
        BinaryReader, CoreRegistryFieldKind, HeaderFieldKind, definition_by_name,
        read_known_uint_field, read_runtime_file_with_error_kind, skip_core_registry_value,
        skip_header_value,
    };

    fn encoded_var_uint(mut value: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                return bytes;
            }
        }
    }

    #[test]
    fn unknown_uint_skips_consume_full_varuint64() {
        let mut bytes = encoded_var_uint(u64::MAX);
        bytes.push(0x2a);

        let mut core_reader = BinaryReader::new(&bytes);
        skip_core_registry_value(&mut core_reader, CoreRegistryFieldKind::Uint)
            .expect("unknown core uint64 should be skippable");
        assert_eq!(core_reader.read_byte().expect("core sentinel"), 0x2a);

        let mut header_reader = BinaryReader::new(&bytes);
        skip_header_value(&mut header_reader, HeaderFieldKind::Uint)
            .expect("unknown header uint64 should be skippable");
        assert_eq!(header_reader.read_byte().expect("header sentinel"), 0x2a);
    }

    #[test]
    fn known_uint_width_controls_value_validation_without_changing_wire_family() {
        let file_asset = definition_by_name("FileAsset").expect("FileAsset schema");
        let uint32_property = file_asset
            .properties
            .iter()
            .find(|property| property.name == "assetId")
            .expect("FileAsset.assetId schema");
        let uint64_property = file_asset
            .properties
            .iter()
            .find(|property| property.name == "scopeLibraryId")
            .expect("FileAsset.scopeLibraryId schema");
        let uint8_property = definition_by_name("LayoutComponentStyle")
            .expect("LayoutComponentStyle schema")
            .properties
            .iter()
            .find(|property| property.name == "displayValue")
            .expect("LayoutComponentStyle.displayValue schema");

        let over_u32 = encoded_var_uint(u64::from(u32::MAX) + 1);
        let error = read_known_uint_field(
            &mut BinaryReader::new(&over_u32),
            uint32_property,
            "uint field",
        )
        .expect_err("known uint32 must retain C++ unsigned-int validation");
        assert!(
            error
                .to_string()
                .contains("does not fit in C++ unsigned int")
        );

        let wide_bytes = encoded_var_uint(u64::MAX);
        let mut wide_reader = BinaryReader::new(&wide_bytes);
        assert_eq!(
            read_known_uint_field(&mut wide_reader, uint64_property, "uint64 field")
                .expect("known uint64"),
            u64::MAX
        );

        // uint8 changes only generated member storage. Registry dispatch and
        // deserialization accept the complete uint32 wire range, then the
        // generated uint8_t member assignment truncates to its low byte.
        let compact_bytes = encoded_var_uint(u64::from(u32::MAX));
        let mut compact_reader = BinaryReader::new(&compact_bytes);
        assert_eq!(
            read_known_uint_field(&mut compact_reader, uint8_property, "uint8 field")
                .expect("known uint8 alias"),
            u64::from(u8::MAX)
        );
    }

    #[test]
    fn runtime_object_decode_preserves_known_uint64_values() {
        let mut bytes = b"RIVE".to_vec();
        // A legacy 7.0 header remains importable after advertising 7.2.
        bytes.extend_from_slice(&[7, 0, 0, 0]); // version, file id, empty header ToC.

        bytes.extend(encoded_var_uint(23)); // Backboard.
        bytes.push(0); // End Backboard properties.

        bytes.extend(encoded_var_uint(558)); // LibraryAsset.
        bytes.extend(encoded_var_uint(798)); // libraryId.
        bytes.extend(encoded_var_uint(u64::MAX));
        bytes.extend(encoded_var_uint(799)); // libraryVersionId.
        bytes.extend(encoded_var_uint(u64::from(u32::MAX) + 1));
        bytes.push(0); // End LibraryAsset properties.

        let file = read_runtime_file_with_error_kind(&bytes)
            .expect("known uint64 fields should import through the full runtime reader");
        let library = file.object(1).expect("decoded LibraryAsset");
        assert_eq!(library.uint_property("libraryId"), Some(u64::MAX));
        assert_eq!(
            library.uint_property("libraryVersionId"),
            Some(u64::from(u32::MAX) + 1)
        );
    }
}

#[cfg(test)]
mod manifest_key_tests {
    use super::{cpp_manifest_key, cpp_manifest_resolver_key};

    // Pins the intentional u32->i32 (and u64->i32) manifest-key reinterpret.
    // C++ keys its manifest name/path maps by a signed `int`
    // (DataResolver::resolveName(int)), so we reinterpret the unsigned runtime id
    // bit-for-bit rather than saturating. The insert key (cpp_manifest_key) and
    // the lookup key (cpp_manifest_resolver_key) MUST agree for any id, including
    // ids above i32::MAX that wrap negative -- otherwise resolve_name/_path would
    // miss entries decode_cpp_manifest_* inserted.
    #[test]
    fn manifest_key_reinterpret_is_bit_preserving_and_consistent() {
        // In-range ids are unchanged.
        assert_eq!(cpp_manifest_key(0), 0);
        assert_eq!(cpp_manifest_resolver_key(0), 0);
        assert_eq!(cpp_manifest_key(1), 1);
        assert_eq!(cpp_manifest_resolver_key(7), 7);
        assert_eq!(cpp_manifest_key(u64::from(i32::MAX as u32)), i32::MAX);

        // Ids above i32::MAX reinterpret (wrap) to a negative key, NOT saturate.
        assert_eq!(cpp_manifest_key(u64::from(u32::MAX)), -1);
        assert_eq!(cpp_manifest_resolver_key(u32::MAX), -1);
        assert_eq!(cpp_manifest_key(0x8000_0000), i32::MIN);
        assert_eq!(cpp_manifest_resolver_key(0x8000_0000), i32::MIN);

        // Insert key and lookup key agree for every u32 id (the load-bearing
        // property): decode inserts with cpp_manifest_key, resolve reads with
        // cpp_manifest_resolver_key.
        for id in [0u32, 1, 42, i32::MAX as u32, 0x8000_0000, u32::MAX] {
            assert_eq!(
                cpp_manifest_key(u64::from(id)),
                cpp_manifest_resolver_key(id)
            );
        }

        // cpp_manifest_key only inspects the low 32 bits (mirrors C++ truncating
        // the var-uint into an int); the high bits of the u64 do not shift it.
        assert_eq!(cpp_manifest_key(0xFFFF_FFFF_0000_0001), 1);
    }
}
