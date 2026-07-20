use crate::data_bind_graph::{
    DATA_BIND_FLAG_DIRECTION_TO_SOURCE, RuntimeDataBindGraphConverterState,
    RuntimeDataBindGraphFormulaRandomSource, RuntimeDataBindGraphRangeMapperProperty,
    RuntimeKeyFrameDataBindTarget, RuntimeKeyFrameDataBindTemplate,
    data_bind_flags_source_to_target_runs_first, runtime_data_bind_graph_convert_value,
    runtime_data_bind_graph_converter, runtime_data_bind_graph_converter_contains_global_id,
    runtime_data_bind_graph_converter_contains_source_change_random,
    runtime_data_bind_graph_converter_requires_persisting_custom_property_source,
    runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_context,
    runtime_data_bind_graph_refresh_operation_view_model_number_converter_for_path,
};
use crate::objects::{InstanceObjectArena, InstanceSlot};
use crate::properties::{
    RuntimeLayoutComputedProperty, artboard_index_for_graph, cached_property_key_for_name,
    layout_computed_property_for_key, property_key_for_name, solid_color_value_property_key,
    solo_active_component_id_property_key,
};
use crate::scripting::RuntimeScriptInstanceHandle;
use crate::view_model::{RuntimeFontAssetValue, RuntimeOwnedViewModelListHandle};
use crate::{
    ArtboardInstance, Mat2D, RuntimeDataBindGraphConverter, RuntimeDataBindGraphValue,
    RuntimeOwnedViewModelContext, RuntimeOwnedViewModelContextHandle, RuntimeOwnedViewModelHandle,
    RuntimeOwnedViewModelInstance, RuntimeViewModelPointer, ScriptInstance,
    data_bind_flags_apply_source_to_target, data_bind_flags_apply_target_to_source,
};
use nuxie_binary::{RuntimeDataType, RuntimeFile, RuntimeObject};
use nuxie_graph::ArtboardGraph;
use nuxie_schema::{FieldKind, definition_by_type_key};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

pub(crate) fn build_key_frame_data_bind_templates(
    file: &RuntimeFile,
    artboard_index: usize,
) -> Vec<RuntimeKeyFrameDataBindTemplate> {
    let default_instance = artboard_default_view_model_instance(file, artboard_index);
    let mut claimed_targets = BTreeSet::new();
    let mut templates = Vec::new();

    for (data_bind_index, data_bind) in file
        .artboard_data_binds(artboard_index)
        .into_iter()
        .enumerate()
    {
        let Some(target) = data_bind.target else {
            continue;
        };
        let holder_target = match target.type_name {
            "KeyFrameDouble" => RuntimeKeyFrameDataBindTarget::Number,
            "KeyFrameColor" => RuntimeKeyFrameDataBindTarget::Color,
            "KeyFrameBool" => RuntimeKeyFrameDataBindTarget::Boolean,
            "KeyFrameString" => RuntimeKeyFrameDataBindTarget::String,
            // C++ intentionally leaves KeyFrameUint/KeyFrameId unbound.
            _ => continue,
        };
        // C++ firstBindByTarget.emplace keeps the first authored bind even if
        // another bind to the same shared keyframe follows it.
        if !claimed_targets.insert(target.id) {
            continue;
        }
        let Some(path) = file.data_bind_context_source_path_ids_for_object(data_bind.object) else {
            continue;
        };
        let holder_default = match holder_target {
            RuntimeKeyFrameDataBindTarget::Number => RuntimeDataBindGraphValue::Number(0.0),
            RuntimeKeyFrameDataBindTarget::Color => RuntimeDataBindGraphValue::Color(0xFF1D1D1D),
            RuntimeKeyFrameDataBindTarget::Boolean => RuntimeDataBindGraphValue::Boolean(false),
            RuntimeKeyFrameDataBindTarget::String => RuntimeDataBindGraphValue::String(Vec::new()),
        };
        let default_value = default_instance
            .as_ref()
            .and_then(|default_instance| {
                file.data_context_view_model_property_for_instance(default_instance.object, &path)
                    .and_then(|source| runtime_created_view_model_value_for_source(file, source))
            })
            .or_else(|| runtime_created_view_model_value_for_declared_path(file, &path))
            .unwrap_or(holder_default);

        templates.push(RuntimeKeyFrameDataBindTemplate {
            data_bind_index,
            key_frame_global_id: target.id,
            target: holder_target,
            path: path.to_vec(),
            flags: data_bind.object.uint_property("flags").unwrap_or(0),
            converter: runtime_data_bind_graph_converter(file, data_bind.object),
            default_value,
        });
    }
    templates
}

macro_rules! cached_runtime_data_bind_property_key {
    ($type_name:literal, $property_name:literal) => {{
        static KEY: OnceLock<Option<u16>> = OnceLock::new();
        cached_property_key_for_name(&KEY, $type_name, $property_name)
    }};
}

fn runtime_data_bind_property_key_for_name(type_name: &str, property_name: &str) -> Option<u16> {
    match (type_name, property_name) {
        ("Component", "parentId") => {
            cached_runtime_data_bind_property_key!("Component", "parentId")
        }
        ("TextValueRun", "text") => {
            cached_runtime_data_bind_property_key!("TextValueRun", "text")
        }
        ("Image", "assetId") => cached_runtime_data_bind_property_key!("Image", "assetId"),
        ("NestedArtboard", "artboardId") => {
            cached_runtime_data_bind_property_key!("NestedArtboard", "artboardId")
        }
        ("NestedArtboard", "isPaused") => {
            cached_runtime_data_bind_property_key!("NestedArtboard", "isPaused")
        }
        ("NestedArtboard", "speed") => {
            cached_runtime_data_bind_property_key!("NestedArtboard", "speed")
        }
        ("NestedArtboard", "quantize") => {
            cached_runtime_data_bind_property_key!("NestedArtboard", "quantize")
        }
        ("CustomPropertyNumber", "propertyValue") => {
            cached_runtime_data_bind_property_key!("CustomPropertyNumber", "propertyValue")
        }
        ("CustomPropertyBoolean", "propertyValue") => {
            cached_runtime_data_bind_property_key!("CustomPropertyBoolean", "propertyValue")
        }
        ("CustomPropertyString", "propertyValue") => {
            cached_runtime_data_bind_property_key!("CustomPropertyString", "propertyValue")
        }
        ("CustomPropertyColor", "propertyValue") => {
            cached_runtime_data_bind_property_key!("CustomPropertyColor", "propertyValue")
        }
        ("CustomPropertyEnum", "propertyValue") => {
            cached_runtime_data_bind_property_key!("CustomPropertyEnum", "propertyValue")
        }
        ("CustomPropertyTrigger", "propertyValue") => {
            cached_runtime_data_bind_property_key!("CustomPropertyTrigger", "propertyValue")
        }
        ("TrimPath", "start") => cached_runtime_data_bind_property_key!("TrimPath", "start"),
        ("TrimPath", "end") => cached_runtime_data_bind_property_key!("TrimPath", "end"),
        ("Shape", "length") => cached_runtime_data_bind_property_key!("Shape", "length"),
        ("ParametricPath", "width") => {
            cached_runtime_data_bind_property_key!("ParametricPath", "width")
        }
        ("ParametricPath", "height") => {
            cached_runtime_data_bind_property_key!("ParametricPath", "height")
        }
        ("FormulaTokenValue", "operationValue") => {
            cached_runtime_data_bind_property_key!("FormulaTokenValue", "operationValue")
        }
        ("DataConverterOperationValue", "operationValue") => {
            cached_runtime_data_bind_property_key!("DataConverterOperationValue", "operationValue")
        }
        ("ViewModelInstanceNumber", "propertyValue") => {
            cached_runtime_data_bind_property_key!("ViewModelInstanceNumber", "propertyValue")
        }
        ("ViewModelInstanceString", "propertyValue") => {
            cached_runtime_data_bind_property_key!("ViewModelInstanceString", "propertyValue")
        }
        ("ViewModelInstanceColor", "propertyValue") => {
            cached_runtime_data_bind_property_key!("ViewModelInstanceColor", "propertyValue")
        }
        ("ViewModelInstanceBoolean", "propertyValue") => {
            cached_runtime_data_bind_property_key!("ViewModelInstanceBoolean", "propertyValue")
        }
        ("ViewModelInstanceEnum", "propertyValue") => {
            cached_runtime_data_bind_property_key!("ViewModelInstanceEnum", "propertyValue")
        }
        ("ViewModelInstanceAssetImage", "propertyValue") => {
            cached_runtime_data_bind_property_key!("ViewModelInstanceAssetImage", "propertyValue")
        }
        ("ViewModelInstanceAssetFont", "propertyValue") => {
            cached_runtime_data_bind_property_key!("ViewModelInstanceAssetFont", "propertyValue")
        }
        ("ViewModelInstance", "viewModelId") => {
            cached_runtime_data_bind_property_key!("ViewModelInstance", "viewModelId")
        }
        ("ViewModelInstanceValue", "viewModelPropertyId") => {
            cached_runtime_data_bind_property_key!("ViewModelInstanceValue", "viewModelPropertyId")
        }
        _ => property_key_for_name(type_name, property_name),
    }
}

fn shared_data_bind_path(path: Vec<u32>) -> Arc<[u32]> {
    Arc::from(path.into_boxed_slice())
}

fn runtime_data_bind_component_parent_id_key() -> Option<u16> {
    cached_runtime_data_bind_property_key!("Component", "parentId")
}

fn runtime_data_bind_view_model_instance_view_model_id_key() -> Option<u16> {
    cached_runtime_data_bind_property_key!("ViewModelInstance", "viewModelId")
}

fn runtime_data_bind_view_model_instance_value_property_id_key() -> Option<u16> {
    cached_runtime_data_bind_property_key!("ViewModelInstanceValue", "viewModelPropertyId")
}

fn runtime_data_bind_view_model_instance_number_value_key() -> Option<u16> {
    cached_runtime_data_bind_property_key!("ViewModelInstanceNumber", "propertyValue")
}

fn runtime_data_bind_view_model_instance_string_value_key() -> Option<u16> {
    cached_runtime_data_bind_property_key!("ViewModelInstanceString", "propertyValue")
}

fn runtime_data_bind_view_model_instance_color_value_key() -> Option<u16> {
    cached_runtime_data_bind_property_key!("ViewModelInstanceColor", "propertyValue")
}

fn runtime_data_bind_view_model_instance_boolean_value_key() -> Option<u16> {
    cached_runtime_data_bind_property_key!("ViewModelInstanceBoolean", "propertyValue")
}

fn runtime_data_bind_view_model_instance_enum_value_key() -> Option<u16> {
    cached_runtime_data_bind_property_key!("ViewModelInstanceEnum", "propertyValue")
}

fn runtime_data_bind_view_model_instance_asset_value_key() -> Option<u16> {
    cached_runtime_data_bind_property_key!("ViewModelInstanceAssetImage", "propertyValue")
}

fn runtime_data_bind_view_model_instance_font_asset_value_key() -> Option<u16> {
    cached_runtime_data_bind_property_key!("ViewModelInstanceAssetFont", "propertyValue")
}

pub(crate) fn build_nested_host_data_bind_source_locals(
    slots: &[InstanceSlot],
    objects: &InstanceObjectArena,
    host_local_id: usize,
    view_model_instance_locals_by_id: &BTreeMap<u32, usize>,
    child: &ArtboardInstance,
) -> BTreeMap<Vec<u32>, usize> {
    if child.artboard_property_bindings.is_empty()
        && child.artboard_image_asset_bindings.is_empty()
        && child.artboard_formula_token_bindings.is_empty()
        && child.artboard_converter_property_bindings.is_empty()
    {
        return BTreeMap::new();
    }

    let mut source_locals = BTreeMap::new();
    for path in child
        .artboard_property_bindings
        .iter()
        .map(|binding| binding.path.as_slice())
        .chain(
            child
                .artboard_image_asset_bindings
                .iter()
                .map(|binding| binding.path.as_slice()),
        )
        .chain(
            child
                .artboard_formula_token_bindings
                .iter()
                .filter(|binding| binding.artboard_converter_reachable)
                .map(|binding| binding.path.as_slice()),
        )
        .chain(
            child
                .artboard_converter_property_bindings
                .iter()
                .map(|binding| binding.path.as_slice()),
        )
    {
        if source_locals.contains_key(path) {
            continue;
        }
        if let Some(source_local) = stateful_nested_host_value_local_for_slots(
            slots,
            objects,
            host_local_id,
            Some(view_model_instance_locals_by_id),
            path,
        ) {
            source_locals.insert(path.to_vec(), source_local);
        }
    }
    source_locals
}

pub(crate) fn build_nested_host_data_bind_source_local_slots(
    child: &ArtboardInstance,
    source_locals_by_path: &BTreeMap<Vec<u32>, usize>,
) -> (Vec<Option<usize>>, Vec<Option<usize>>) {
    let property_source_locals = child
        .artboard_property_bindings
        .iter()
        .map(|binding| source_locals_by_path.get(binding.path.as_slice()).copied())
        .collect();
    let image_source_locals = child
        .artboard_image_asset_bindings
        .iter()
        .map(|binding| source_locals_by_path.get(binding.path.as_slice()).copied())
        .collect();
    (property_source_locals, image_source_locals)
}

pub(crate) fn build_nested_host_view_model_instance_locals(
    slots: &[InstanceSlot],
    objects: &InstanceObjectArena,
    host_local_id: usize,
) -> BTreeMap<u32, usize> {
    let Some(parent_key) = runtime_data_bind_component_parent_id_key() else {
        return BTreeMap::new();
    };
    let Some(view_model_key) = runtime_data_bind_view_model_instance_view_model_id_key() else {
        return BTreeMap::new();
    };
    let mut locals_by_id = BTreeMap::new();
    for slot in slots {
        if slot.type_name != Some("ViewModelInstance")
            || objects.uint_property(slot.local_id, parent_key) != Some(host_local_id as u64)
        {
            continue;
        }
        let Some(view_model_id) = objects
            .uint_property(slot.local_id, view_model_key)
            .and_then(|value| u32::try_from(value).ok())
        else {
            continue;
        };
        locals_by_id.entry(view_model_id).or_insert(slot.local_id);
    }
    locals_by_id
}

fn stateful_nested_host_value_local_for_slots(
    slots: &[InstanceSlot],
    objects: &InstanceObjectArena,
    host_local_id: usize,
    view_model_instance_locals_by_id: Option<&BTreeMap<u32, usize>>,
    path: &[u32],
) -> Option<usize> {
    let (view_model_id, property_path) = path.split_first()?;
    let mut current_local = match view_model_instance_locals_by_id {
        Some(view_model_instance_locals_by_id) => view_model_instance_locals_by_id
            .get(view_model_id)
            .copied()?,
        None => stateful_nested_host_view_model_instance_local_for_slots(
            slots,
            objects,
            host_local_id,
            *view_model_id,
        )?,
    };
    for property_id in property_path {
        current_local = view_model_instance_value_child_local_for_slots(
            slots,
            objects,
            current_local,
            *property_id,
        )?;
    }
    Some(current_local)
}

fn stateful_nested_host_view_model_instance_local_for_slots(
    slots: &[InstanceSlot],
    objects: &InstanceObjectArena,
    host_local_id: usize,
    view_model_id: u32,
) -> Option<usize> {
    let parent_key = runtime_data_bind_component_parent_id_key()?;
    let view_model_key = runtime_data_bind_view_model_instance_view_model_id_key()?;
    slots.iter().find_map(|slot| {
        (slot.type_name == Some("ViewModelInstance")
            && objects.uint_property(slot.local_id, parent_key) == Some(host_local_id as u64)
            && objects.uint_property(slot.local_id, view_model_key)
                == Some(u64::from(view_model_id)))
        .then_some(slot.local_id)
    })
}

fn view_model_instance_value_child_local_for_slots(
    slots: &[InstanceSlot],
    objects: &InstanceObjectArena,
    parent_local_id: usize,
    view_model_property_id: u32,
) -> Option<usize> {
    let parent_key = runtime_data_bind_component_parent_id_key()?;
    let property_key = runtime_data_bind_view_model_instance_value_property_id_key()?;
    slots.iter().find_map(|slot| {
        let type_name = slot.type_name?;
        (type_name.starts_with("ViewModelInstance")
            && type_name != "ViewModelInstance"
            && objects.uint_property(slot.local_id, parent_key) == Some(parent_local_id as u64)
            && objects.uint_property(slot.local_id, property_key)
                == Some(u64::from(view_model_property_id)))
        .then_some(slot.local_id)
    })
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardPropertyBindingInstance {
    data_bind_index: usize,
    flags: u64,
    target_local_id: usize,
    property_key: u16,
    path: Vec<u32>,
    path_is_name_based: bool,
    owned_context_source_path: Option<Vec<usize>>,
    enum_value_names: Vec<Vec<u8>>,
    converter: Option<RuntimeDataBindGraphConverter>,
    converter_state: RuntimeDataBindGraphConverterState,
    default_value: RuntimeDataBindGraphValue,
    default_value_is_resolved: bool,
    snapshots_source_value: bool,
    pending_value: Option<RuntimeDataBindGraphValue>,
}

/// One C++ `DataBind` owns one converter and one direction latch even when the
/// Rust execution plan materializes separate source-to-target and
/// target-to-source binding records. Keep that shared state keyed by the
/// authored data-bind index so a stateful reverse conversion cannot advance a
/// second interpolator or re-dirty the opposite direction.
#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardSharedDataBindConverterState {
    converter: RuntimeDataBindGraphConverter,
    converter_state: RuntimeDataBindGraphConverterState,
    target_origin: bool,
}

pub(super) fn build_artboard_shared_data_bind_converter_states(
    property_bindings: &[RuntimeArtboardPropertyBindingInstance],
    custom_property_bindings: &[RuntimeArtboardCustomPropertyBindingInstance],
) -> BTreeMap<usize, RuntimeArtboardSharedDataBindConverterState> {
    property_bindings
        .iter()
        .filter_map(|property| {
            custom_property_bindings
                .iter()
                .any(|custom| custom.data_bind_index == property.data_bind_index)
                .then_some(())?;
            let converter = property.converter.clone()?;
            Some((
                property.data_bind_index,
                RuntimeArtboardSharedDataBindConverterState {
                    converter_state: RuntimeDataBindGraphConverterState::for_converter(Some(
                        &converter,
                    )),
                    converter,
                    // A reconcile carries both dirt bits; C++ latches the
                    // direction that runs last (the favored final value).
                    target_origin: !data_bind_flags_source_to_target_runs_first(property.flags),
                },
            ))
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeArtboardAssetBindingTarget {
    Image(usize),
    Font(usize),
}

impl RuntimeArtboardAssetBindingTarget {
    fn is_font(self) -> bool {
        matches!(self, Self::Font(_))
    }
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardImageAssetBindingInstance {
    target: RuntimeArtboardAssetBindingTarget,
    path: Vec<u32>,
    path_is_name_based: bool,
    owned_context_source_path: Option<Vec<usize>>,
    default_value: RuntimeDataBindGraphValue,
    font_value: Option<RuntimeFontAssetValue>,
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeNestedChildContextUpdate {
    Property(usize, RuntimeDataBindGraphValue),
    ImageAsset(usize, RuntimeDataBindGraphValue),
    ContextPath(Vec<u32>, RuntimeDataBindGraphValue),
}

#[derive(Debug, Clone)]
enum RuntimeStatefulViewModelValueUpdate {
    Value(RuntimeDataBindGraphValue),
    FontAsset(u64),
    ViewModelInstance(usize),
}

#[derive(Debug, Clone)]
struct RuntimeStatefulViewModelUpdate {
    instance_local_id: usize,
    view_model_index: usize,
    property_path: Vec<usize>,
    value: RuntimeStatefulViewModelValueUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeArtboardOwnedContextKey {
    instances: Vec<RuntimeArtboardOwnedContextInstanceKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeArtboardOwnedContextInstanceKey {
    view_model_index: usize,
    instance_identity: u64,
    mutation_generation: u64,
    context_chain: Vec<Vec<usize>>,
}

impl RuntimeArtboardOwnedContextKey {
    fn from_context_chain(
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> Self {
        Self {
            instances: vec![RuntimeArtboardOwnedContextInstanceKey {
                view_model_index: context.view_model_index,
                instance_identity: context.instance_identity(),
                mutation_generation: context.mutation_generation(),
                context_chain: context_chain
                    .iter()
                    .map(|context_path| context_path.to_vec())
                    .collect(),
            }],
        }
    }

    fn matches_context_chain(
        &self,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> bool {
        self.instances.len() == 1
            && self.instances[0].view_model_index == context.view_model_index
            && self.instances[0].instance_identity == context.instance_identity()
            && self.instances[0].mutation_generation == context.mutation_generation()
            && self.instances[0].context_chain.len() == context_chain.len()
            && self.instances[0]
                .context_chain
                .iter()
                .zip(context_chain)
                .all(|(stored, current)| stored.as_slice() == *current)
    }

    fn from_candidates(candidates: &[RuntimeOwnedViewModelBindingCandidate]) -> Self {
        Self {
            instances: candidates
                .iter()
                .map(|candidate| {
                    let context = candidate.context.borrow();
                    RuntimeArtboardOwnedContextInstanceKey {
                        view_model_index: context.view_model_index(),
                        instance_identity: context.instance_identity(),
                        mutation_generation: context.mutation_generation(),
                        context_chain: candidate.context_chain.clone(),
                    }
                })
                .collect(),
        }
    }

    fn matches_candidates(&self, candidates: &[RuntimeOwnedViewModelBindingCandidate]) -> bool {
        self.instances.len() == candidates.len()
            && self
                .instances
                .iter()
                .zip(candidates)
                .all(|(stored, current)| {
                    let context = current.context.borrow();
                    stored.view_model_index == context.view_model_index()
                        && stored.instance_identity == context.instance_identity()
                        && stored.mutation_generation == context.mutation_generation()
                        && stored.context_chain == current.context_chain
                })
    }

    /// The retained candidate vector is replaced only through the full
    /// structural check above. Steady-state frame polling therefore only
    /// needs to observe mutations of those already-bound shared instances.
    fn matches_candidate_mutations(
        &self,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
    ) -> bool {
        self.instances.len() == candidates.len()
            && self
                .instances
                .iter()
                .zip(candidates)
                .all(|(stored, current)| {
                    stored.mutation_generation == current.context.borrow().mutation_generation()
                })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeOwnedViewModelBindingCandidate {
    pub(crate) context: RuntimeOwnedViewModelHandle,
    pub(crate) context_chain: Vec<Vec<usize>>,
}

impl RuntimeOwnedViewModelBindingCandidate {
    pub(crate) fn root(context: &RuntimeOwnedViewModelInstance) -> Self {
        Self {
            context: RuntimeOwnedViewModelHandle::new(context.clone()),
            context_chain: vec![Vec::new()],
        }
    }

    pub(crate) fn root_handle(context: &RuntimeOwnedViewModelHandle) -> Self {
        Self {
            context: context.clone(),
            context_chain: vec![Vec::new()],
        }
    }

    fn context_chain(&self) -> Vec<&[usize]> {
        self.context_chain.iter().map(Vec::as_slice).collect()
    }
}

/// The concrete view-model value selected when a data bind resolves its
/// source. C++ retains this as a `ContextValue`; keeping both the shared
/// instance and resolved property path prevents a target-to-source update
/// from being redirected through a different fallback context later.
#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelBindingSource {
    context: RuntimeOwnedViewModelHandle,
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelBindingSource {
    fn sync_value(&self, value: &RuntimeDataBindGraphValue) -> bool {
        let mut context = self.context.borrow_mut();
        match value {
            RuntimeDataBindGraphValue::Number(value) => {
                context.sync_number_by_property_path(&self.property_path, *value)
            }
            RuntimeDataBindGraphValue::Boolean(value) => {
                context.sync_boolean_by_property_path(&self.property_path, *value)
            }
            RuntimeDataBindGraphValue::String(value) => {
                context.sync_string_by_property_path(&self.property_path, value)
            }
            RuntimeDataBindGraphValue::Color(value) => {
                context.sync_color_by_property_path(&self.property_path, *value)
            }
            RuntimeDataBindGraphValue::Enum(value) => {
                context.sync_enum_by_property_path(&self.property_path, *value)
            }
            RuntimeDataBindGraphValue::SymbolListIndex(value) => {
                context.sync_symbol_list_index_by_property_path(&self.property_path, *value)
            }
            RuntimeDataBindGraphValue::Asset(value) => {
                context.sync_asset_by_property_path(&self.property_path, *value)
            }
            RuntimeDataBindGraphValue::Artboard(value) => {
                context.sync_artboard_by_property_path(&self.property_path, *value)
            }
            RuntimeDataBindGraphValue::Trigger(value) => {
                context.sync_trigger_by_property_path(&self.property_path, *value)
            }
            RuntimeDataBindGraphValue::List { .. }
            | RuntimeDataBindGraphValue::ListLength(_)
            | RuntimeDataBindGraphValue::ViewModel(_) => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum RuntimeArtboardDataBindTargetRef {
    Property(usize),
    ImageAsset(usize),
    ConverterProperty(usize),
}

#[derive(Debug, Clone, Default)]
pub(super) struct RuntimeArtboardDataBindTargetQueues {
    by_path: BTreeMap<Vec<u32>, Vec<RuntimeArtboardDataBindTargetRef>>,
    dirty_properties: Vec<usize>,
    dirty_property_flags: Vec<bool>,
    dirty_image_assets: Vec<usize>,
    dirty_image_asset_flags: Vec<bool>,
    dirty_converter_properties: Vec<usize>,
    dirty_converter_property_flags: Vec<bool>,
}

impl RuntimeArtboardDataBindTargetQueues {
    pub(super) fn new(
        property_bindings: &[RuntimeArtboardPropertyBindingInstance],
        image_asset_bindings: &[RuntimeArtboardImageAssetBindingInstance],
        converter_property_bindings: &[RuntimeArtboardConverterPropertyBindingInstance],
    ) -> Self {
        let mut queues = Self {
            dirty_property_flags: vec![false; property_bindings.len()],
            dirty_image_asset_flags: vec![false; image_asset_bindings.len()],
            dirty_converter_property_flags: vec![false; converter_property_bindings.len()],
            ..Self::default()
        };
        for (index, binding) in property_bindings.iter().enumerate() {
            queues
                .by_path
                .entry(binding.path.clone())
                .or_default()
                .push(RuntimeArtboardDataBindTargetRef::Property(index));
            queues.enqueue_property(index);
        }
        for (index, binding) in image_asset_bindings.iter().enumerate() {
            queues
                .by_path
                .entry(binding.path.clone())
                .or_default()
                .push(RuntimeArtboardDataBindTargetRef::ImageAsset(index));
            queues.enqueue_image_asset(index);
        }
        for (index, binding) in converter_property_bindings.iter().enumerate() {
            queues
                .by_path
                .entry(binding.path.clone())
                .or_default()
                .push(RuntimeArtboardDataBindTargetRef::ConverterProperty(index));
            queues.enqueue_converter_property(index);
        }
        queues
    }

    fn enqueue_path(
        &mut self,
        path: &[u32],
        suppressed_property_index: Option<usize>,
    ) -> Vec<usize> {
        let Some(targets) = self.by_path.get(path).cloned() else {
            return Vec::new();
        };
        let mut enqueued_properties = Vec::new();
        for target in targets {
            match target {
                RuntimeArtboardDataBindTargetRef::Property(index) => {
                    if Some(index) == suppressed_property_index {
                        continue;
                    }
                    if self.enqueue_property(index) {
                        enqueued_properties.push(index);
                    }
                }
                RuntimeArtboardDataBindTargetRef::ImageAsset(index) => {
                    self.enqueue_image_asset(index);
                }
                RuntimeArtboardDataBindTargetRef::ConverterProperty(index) => {
                    self.enqueue_converter_property(index);
                }
            }
        }
        enqueued_properties
    }

    fn enqueue_property(&mut self, index: usize) -> bool {
        let Some(flag) = self.dirty_property_flags.get_mut(index) else {
            return false;
        };
        if *flag {
            return false;
        }
        *flag = true;
        self.dirty_properties.push(index);
        true
    }

    fn enqueue_image_asset(&mut self, index: usize) {
        let Some(flag) = self.dirty_image_asset_flags.get_mut(index) else {
            return;
        };
        if *flag {
            return;
        }
        *flag = true;
        self.dirty_image_assets.push(index);
    }

    fn enqueue_converter_property(&mut self, index: usize) {
        let Some(flag) = self.dirty_converter_property_flags.get_mut(index) else {
            return;
        };
        if *flag {
            return;
        }
        *flag = true;
        self.dirty_converter_properties.push(index);
    }

    fn drain_dirty_properties(&mut self) -> Vec<usize> {
        let dirty = std::mem::take(&mut self.dirty_properties);
        for index in &dirty {
            if let Some(flag) = self.dirty_property_flags.get_mut(*index) {
                *flag = false;
            }
        }
        dirty
    }

    fn drain_dirty_properties_for_precedence(
        &mut self,
        bindings: &[RuntimeArtboardPropertyBindingInstance],
        source_to_target_runs_first: bool,
    ) -> Vec<usize> {
        let mut selected = Vec::new();
        for index in self.drain_dirty_properties() {
            let runs_first = bindings.get(index).is_some_and(|binding| {
                data_bind_flags_apply_target_to_source(binding.flags)
                    && data_bind_flags_source_to_target_runs_first(binding.flags)
            });
            if runs_first == source_to_target_runs_first {
                selected.push(index);
            } else {
                self.enqueue_property(index);
            }
        }
        selected
    }

    fn drain_dirty_image_assets(&mut self) -> Vec<usize> {
        let dirty = std::mem::take(&mut self.dirty_image_assets);
        for index in &dirty {
            if let Some(flag) = self.dirty_image_asset_flags.get_mut(*index) {
                *flag = false;
            }
        }
        dirty
    }

    fn drain_dirty_converter_properties(&mut self) -> Vec<usize> {
        let dirty = std::mem::take(&mut self.dirty_converter_properties);
        for index in &dirty {
            if let Some(flag) = self.dirty_converter_property_flags.get_mut(*index) {
                *flag = false;
            }
        }
        dirty
    }
}

#[derive(Debug, Clone, Copy)]
enum RuntimeArtboardDataBindSourceRef {
    CustomProperty {
        index: usize,
        data_bind_index: usize,
    },
    NumericSource {
        index: usize,
        data_bind_index: usize,
    },
}

#[derive(Debug, Clone, Default)]
pub(super) struct RuntimeArtboardDataBindSourceQueues {
    by_target_property: BTreeMap<(usize, u16), Vec<RuntimeArtboardDataBindSourceRef>>,
    dirty_custom_properties: Vec<usize>,
    dirty_custom_property_flags: Vec<bool>,
    persisting_custom_properties: Vec<usize>,
    custom_property_update_indices: Vec<usize>,
    custom_property_update_flags: Vec<bool>,
    dirty_numeric_sources: Vec<usize>,
    dirty_numeric_source_flags: Vec<bool>,
    push_numeric_sources: Vec<usize>,
    persisting_layout_computed: Vec<usize>,
    persisting_solo_sources: Vec<usize>,
    persisting_numeric_sources: Vec<usize>,
    numeric_source_update_indices: Vec<usize>,
}

impl RuntimeArtboardDataBindSourceQueues {
    #[inline]
    fn has_target_properties(&self) -> bool {
        !self.by_target_property.is_empty()
    }

    pub(crate) fn observes_target_property(&self, local_id: usize, property_key: u16) -> bool {
        self.by_target_property
            .contains_key(&(local_id, property_key))
    }

    pub(super) fn new(
        custom_property_bindings: &[RuntimeArtboardCustomPropertyBindingInstance],
        layout_computed_bindings: &[RuntimeArtboardLayoutComputedBindingInstance],
        numeric_source_bindings: &[RuntimeArtboardNumericSourceBindingInstance],
        solo_source_bindings: &[RuntimeArtboardSoloSourceBindingInstance],
    ) -> Self {
        let mut queues = Self {
            dirty_custom_property_flags: vec![false; custom_property_bindings.len()],
            custom_property_update_flags: vec![false; custom_property_bindings.len()],
            dirty_numeric_source_flags: vec![false; numeric_source_bindings.len()],
            persisting_layout_computed: (0..layout_computed_bindings.len()).collect(),
            persisting_solo_sources: (0..solo_source_bindings.len()).collect(),
            ..Self::default()
        };
        for (index, binding) in custom_property_bindings.iter().enumerate() {
            queues
                .by_target_property
                .entry((binding.target_local_id, binding.property_key))
                .or_default()
                .push(RuntimeArtboardDataBindSourceRef::CustomProperty {
                    index,
                    data_bind_index: binding.data_bind_index,
                });
            queues.enqueue_custom_property(index);
            if binding.converter.as_ref().is_some_and(
                runtime_data_bind_graph_converter_requires_persisting_custom_property_source,
            ) {
                // C++ data converters dirty their parent DataBind through
                // converter-owned dependencies. Keep only converter families
                // with unmodeled dirt edges on the conservative polling lane.
                queues.persisting_custom_properties.push(index);
            }
        }
        for (index, binding) in numeric_source_bindings.iter().enumerate() {
            match binding.property {
                RuntimeArtboardNumericSourceProperty::DirectDouble => {
                    queues
                        .by_target_property
                        .entry((binding.target_local_id, binding.property_key))
                        .or_default()
                        .push(RuntimeArtboardDataBindSourceRef::NumericSource {
                            index,
                            data_bind_index: binding.data_bind_index,
                        });
                    queues.enqueue_numeric_source(index);
                    queues.push_numeric_sources.push(index);
                }
                RuntimeArtboardNumericSourceProperty::ShapeLength => {
                    queues.persisting_numeric_sources.push(index);
                }
            }
        }
        queues
    }

    fn enqueue_target_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        suppressed_data_bind_index: Option<usize>,
    ) -> Vec<usize> {
        let Self {
            by_target_property,
            dirty_custom_properties,
            dirty_custom_property_flags,
            dirty_numeric_sources,
            dirty_numeric_source_flags,
            ..
        } = self;
        let Some(sources) = by_target_property.get(&(local_id, property_key)) else {
            return Vec::new();
        };
        let mut enqueued_data_binds = Vec::new();
        for source in sources.iter().copied() {
            match source {
                RuntimeArtboardDataBindSourceRef::CustomProperty {
                    index,
                    data_bind_index,
                } => {
                    if Some(data_bind_index) == suppressed_data_bind_index {
                        continue;
                    }
                    let Some(flag) = dirty_custom_property_flags.get_mut(index) else {
                        continue;
                    };
                    if *flag {
                        continue;
                    }
                    *flag = true;
                    dirty_custom_properties.push(index);
                    enqueued_data_binds.push(data_bind_index);
                }
                RuntimeArtboardDataBindSourceRef::NumericSource {
                    index,
                    data_bind_index,
                } => {
                    if Some(data_bind_index) == suppressed_data_bind_index {
                        continue;
                    }
                    let Some(flag) = dirty_numeric_source_flags.get_mut(index) else {
                        continue;
                    };
                    if *flag {
                        continue;
                    }
                    *flag = true;
                    dirty_numeric_sources.push(index);
                    enqueued_data_binds.push(data_bind_index);
                }
            }
        }
        enqueued_data_binds
    }

    fn enqueue_custom_property(&mut self, index: usize) {
        let Some(flag) = self.dirty_custom_property_flags.get_mut(index) else {
            return;
        };
        if *flag {
            return;
        }
        *flag = true;
        self.dirty_custom_properties.push(index);
    }

    fn enqueue_numeric_source(&mut self, index: usize) {
        let Some(flag) = self.dirty_numeric_source_flags.get_mut(index) else {
            return;
        };
        if *flag {
            return;
        }
        *flag = true;
        self.dirty_numeric_sources.push(index);
    }

    fn enqueue_numeric_push_sources(&mut self) {
        for index in self.push_numeric_sources.clone() {
            self.enqueue_numeric_source(index);
        }
    }

    fn has_custom_property_update_indices(&self) -> bool {
        !self.dirty_custom_properties.is_empty() || !self.persisting_custom_properties.is_empty()
    }

    fn has_numeric_source_update_indices(&self) -> bool {
        !self.dirty_numeric_sources.is_empty() || !self.persisting_numeric_sources.is_empty()
    }

    fn take_custom_property_update_indices(&mut self) -> Vec<usize> {
        let mut indices = std::mem::take(&mut self.custom_property_update_indices);
        indices.clear();
        let mut included_indices = std::mem::take(&mut self.custom_property_update_flags);
        for index in self.dirty_custom_properties.drain(..) {
            if let Some(flag) = self.dirty_custom_property_flags.get_mut(index) {
                *flag = false;
            }
            let Some(included) = included_indices.get_mut(index) else {
                continue;
            };
            if !*included {
                *included = true;
                indices.push(index);
            }
        }
        for index in &self.persisting_custom_properties {
            let Some(included) = included_indices.get_mut(*index) else {
                continue;
            };
            if !*included {
                *included = true;
                indices.push(*index);
            }
        }
        for index in &indices {
            if let Some(included) = included_indices.get_mut(*index) {
                *included = false;
            }
        }
        self.custom_property_update_flags = included_indices;
        indices
    }

    fn recycle_custom_property_update_indices(&mut self, mut indices: Vec<usize>) {
        indices.clear();
        self.custom_property_update_indices = indices;
    }

    fn take_numeric_source_update_indices(&mut self) -> Vec<usize> {
        let mut indices = std::mem::take(&mut self.numeric_source_update_indices);
        indices.clear();
        indices.extend(self.persisting_numeric_sources.iter().copied());
        for index in self.dirty_numeric_sources.drain(..) {
            if let Some(flag) = self.dirty_numeric_source_flags.get_mut(index) {
                *flag = false;
            }
            indices.push(index);
        }
        indices
    }

    fn recycle_numeric_source_update_indices(&mut self, mut indices: Vec<usize>) {
        indices.clear();
        self.numeric_source_update_indices = indices;
    }

    #[cfg(test)]
    fn drain_custom_property_update_indices(&mut self) -> Vec<usize> {
        self.take_custom_property_update_indices()
    }

    #[cfg(test)]
    fn drain_dirty_numeric_sources(&mut self) -> Vec<usize> {
        let mut dirty = Vec::new();
        for index in self.dirty_numeric_sources.drain(..) {
            if let Some(flag) = self.dirty_numeric_source_flags.get_mut(index) {
                *flag = false;
            }
            dirty.push(index);
        }
        dirty
    }

    fn persisting_layout_computed(&self) -> &[usize] {
        &self.persisting_layout_computed
    }

    fn persisting_solo_sources(&self) -> &[usize] {
        &self.persisting_solo_sources
    }

    pub(super) fn persisting_numeric_sources(&self) -> &[usize] {
        &self.persisting_numeric_sources
    }
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardCustomPropertyBindingInstance {
    data_bind_index: usize,
    target_local_id: usize,
    property_key: u16,
    path: Arc<[u32]>,
    path_is_name_based: bool,
    owned_context_source_path: Option<Vec<usize>>,
    owned_context_source: Option<RuntimeOwnedViewModelBindingSource>,
    flags: u64,
    value_kind: RuntimeArtboardDataBindValueKind,
    converter: Option<RuntimeDataBindGraphConverter>,
    converter_state: RuntimeDataBindGraphConverterState,
    default_value: RuntimeDataBindGraphValue,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardLayoutComputedBindingInstance {
    target_local_id: usize,
    property: RuntimeLayoutComputedProperty,
    path: Arc<[u32]>,
    owned_context_source: Option<RuntimeOwnedViewModelBindingSource>,
    path_is_name_based: bool,
    owned_context_source_path: Option<Vec<usize>>,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardNumericSourceBindingInstance {
    data_bind_index: usize,
    target_local_id: usize,
    property_key: u16,
    property: RuntimeArtboardNumericSourceProperty,
    path: Vec<u32>,
    owned_context_source: Option<RuntimeOwnedViewModelBindingSource>,
    path_is_name_based: bool,
    owned_context_source_path: Option<Vec<usize>>,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardFormulaTokenBindingInstance {
    target: RuntimeArtboardFormulaBindingTarget,
    path: Vec<u32>,
    artboard_converter_reachable: bool,
    converter: Option<RuntimeDataBindGraphConverter>,
    converter_state: RuntimeDataBindGraphConverterState,
    default_value: RuntimeDataBindGraphValue,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum RuntimeArtboardFormulaBindingTarget {
    FormulaToken { global_id: u32 },
    OperationValue { global_id: u32 },
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardConverterPropertyBindingInstance {
    target: RuntimeArtboardConverterPropertyBindingTarget,
    path: Vec<u32>,
    converter: Option<RuntimeDataBindGraphConverter>,
    converter_state: RuntimeDataBindGraphConverterState,
    default_value: RuntimeDataBindGraphValue,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum RuntimeArtboardConverterPropertyBindingTarget {
    ToStringDecimals {
        global_id: u32,
    },
    ToStringColorFormat {
        global_id: u32,
    },
    StringTrimTrimType {
        global_id: u32,
    },
    StringPadLength {
        global_id: u32,
    },
    StringPadText {
        global_id: u32,
    },
    StringPadPadType {
        global_id: u32,
    },
    InterpolatorDuration {
        global_id: u32,
    },
    RangeMapper {
        global_id: u32,
        property: RuntimeDataBindGraphRangeMapperProperty,
    },
    NumberToListViewModelId {
        global_id: u32,
    },
}

enum RuntimeArtboardConverterPropertyBindingUpdate {
    ToStringDecimals {
        global_id: u32,
        value: u64,
    },
    ToStringColorFormat {
        global_id: u32,
        value: Vec<u8>,
    },
    StringTrimTrimType {
        global_id: u32,
        value: u64,
    },
    StringPadLength {
        global_id: u32,
        value: u64,
    },
    StringPadText {
        global_id: u32,
        value: Vec<u8>,
    },
    StringPadPadType {
        global_id: u32,
        value: u64,
    },
    InterpolatorDuration {
        global_id: u32,
        value: f32,
    },
    RangeMapper {
        global_id: u32,
        property: RuntimeDataBindGraphRangeMapperProperty,
        value: f32,
    },
    NumberToListViewModelId {
        global_id: u32,
        value: u64,
    },
}

#[derive(Debug, Clone, Copy)]
enum RuntimeArtboardNumericSourceProperty {
    DirectDouble,
    ShapeLength,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardContextSourceValue {
    path: Arc<[u32]>,
    value: RuntimeDataBindGraphValue,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardSoloBindingInstance {
    target_local_id: usize,
    path: Vec<u32>,
    enum_value_names: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardSoloSourceBindingInstance {
    target_local_id: usize,
    path: Arc<[u32]>,
    path_is_name_based: bool,
    owned_context_source_path: Option<Vec<usize>>,
    enum_value_names: Vec<Vec<u8>>,
    owned_context_source: Option<RuntimeOwnedViewModelBindingSource>,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardNestedHostBindingInstance {
    target_local_id: usize,
    property: RuntimeArtboardNestedHostProperty,
    path: Vec<u32>,
    path_is_name_based: bool,
    owned_context_source_path: Option<Vec<usize>>,
    artboard_value_applied: bool,
}

#[derive(Debug, Clone, Copy)]
enum RuntimeArtboardNestedHostProperty {
    ArtboardId { property_key: u16 },
    IsPaused { property_key: u16 },
    Speed { property_key: u16 },
    Quantize { property_key: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeArtboardDataBindValueKind {
    Number,
    Boolean,
    String,
    Color,
    Enum,
    Trigger,
}

fn runtime_artboard_data_bind_default_value_for_kind(
    kind: RuntimeArtboardDataBindValueKind,
) -> RuntimeDataBindGraphValue {
    match kind {
        RuntimeArtboardDataBindValueKind::Number => RuntimeDataBindGraphValue::Number(0.0),
        RuntimeArtboardDataBindValueKind::Boolean => RuntimeDataBindGraphValue::Boolean(false),
        RuntimeArtboardDataBindValueKind::String => RuntimeDataBindGraphValue::String(Vec::new()),
        RuntimeArtboardDataBindValueKind::Color => RuntimeDataBindGraphValue::Color(0xFF000000),
        RuntimeArtboardDataBindValueKind::Enum => RuntimeDataBindGraphValue::Enum(0),
        RuntimeArtboardDataBindValueKind::Trigger => RuntimeDataBindGraphValue::Trigger(0),
    }
}

fn artboard_data_bind_values_have_same_kind(
    source: &RuntimeDataBindGraphValue,
    value: &RuntimeDataBindGraphValue,
) -> bool {
    matches!(
        (source, value),
        (
            RuntimeDataBindGraphValue::Number(_),
            RuntimeDataBindGraphValue::Number(_)
        ) | (
            RuntimeDataBindGraphValue::Boolean(_),
            RuntimeDataBindGraphValue::Boolean(_)
        ) | (
            RuntimeDataBindGraphValue::String(_),
            RuntimeDataBindGraphValue::String(_)
        ) | (
            RuntimeDataBindGraphValue::Color(_),
            RuntimeDataBindGraphValue::Color(_)
        ) | (
            RuntimeDataBindGraphValue::Enum(_),
            RuntimeDataBindGraphValue::Enum(_)
        ) | (
            RuntimeDataBindGraphValue::SymbolListIndex(_),
            RuntimeDataBindGraphValue::SymbolListIndex(_)
        ) | (
            RuntimeDataBindGraphValue::List { .. },
            RuntimeDataBindGraphValue::List { .. }
        ) | (
            RuntimeDataBindGraphValue::ListLength(_),
            RuntimeDataBindGraphValue::ListLength(_)
        ) | (
            RuntimeDataBindGraphValue::Asset(_),
            RuntimeDataBindGraphValue::Asset(_)
        ) | (
            RuntimeDataBindGraphValue::Artboard(_),
            RuntimeDataBindGraphValue::Artboard(_)
        ) | (
            RuntimeDataBindGraphValue::Trigger(_),
            RuntimeDataBindGraphValue::Trigger(_)
        ) | (
            RuntimeDataBindGraphValue::ViewModel(_),
            RuntimeDataBindGraphValue::ViewModel(_)
        )
    )
}

fn runtime_owned_view_model_context_path_for_context_chain<'a>(
    context: &RuntimeOwnedViewModelInstance,
    context_chain: &'a [&'a [usize]],
    path: &[u32],
) -> Option<RuntimeOwnedViewModelContextPathStorage<'a>> {
    context_chain.iter().find_map(|context_path| {
        let property_path = RuntimeOwnedViewModelContextPathStorage::from_context_source_path(
            context,
            context_path,
            path,
        )?;
        context.view_model_index_by_property_path(property_path.as_slice())?;
        Some(property_path)
    })
}

fn runtime_owned_view_model_binding_value_for_retained_context_chain(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelInstance,
    context_chain: &[&[usize]],
    path: &[u32],
    path_is_name_based: bool,
    scripting_manifest: bool,
    retained_source_path: &mut Option<Vec<usize>>,
) -> Option<RuntimeDataBindGraphValue> {
    if let Some(source_path) = retained_source_path.as_deref()
        && let Some(value) =
            runtime_owned_view_model_binding_value_for_property_path(context, source_path)
    {
        return Some(value);
    }

    let (source_path, value) = context_chain.iter().find_map(|context_path| {
        let source_path = context.property_path_for_context_source_path_with_manifest_mode(
            file,
            context_path,
            path,
            path_is_name_based,
            scripting_manifest,
        )?;
        let value =
            runtime_owned_view_model_binding_value_for_property_path(context, &source_path)?;
        Some((source_path, value))
    })?;
    *retained_source_path = Some(source_path);
    Some(value)
}

fn runtime_owned_view_model_binding_value_for_candidates(
    file: &RuntimeFile,
    candidates: &[RuntimeOwnedViewModelBindingCandidate],
    path: &[u32],
    path_is_name_based: bool,
    scripting_manifest: bool,
) -> Option<RuntimeDataBindGraphValue> {
    candidates.iter().find_map(|candidate| {
        let context = candidate.context.borrow();
        candidate.context_chain.iter().find_map(|context_path| {
            let source_path = context.property_path_for_context_source_path_with_manifest_mode(
                file,
                context_path,
                path,
                path_is_name_based,
                scripting_manifest,
            )?;
            runtime_owned_view_model_binding_value_for_property_path(&context, &source_path)
        })
    })
}

fn runtime_owned_view_model_binding_source_for_candidates(
    file: &RuntimeFile,
    candidates: &[RuntimeOwnedViewModelBindingCandidate],
    path: &[u32],
    path_is_name_based: bool,
    scripting_manifest: bool,
) -> Option<RuntimeOwnedViewModelBindingSource> {
    candidates.iter().find_map(|candidate| {
        let context = candidate.context.borrow();
        let property_path = candidate.context_chain.iter().find_map(|context_path| {
            context.property_path_for_context_source_path_with_manifest_mode(
                file,
                context_path,
                path,
                path_is_name_based,
                scripting_manifest,
            )
        })?;
        Some(RuntimeOwnedViewModelBindingSource {
            context: candidate.context.clone(),
            property_path,
        })
    })
}

fn runtime_owned_view_model_sync_value_for_candidates(
    file: &RuntimeFile,
    candidates: &[RuntimeOwnedViewModelBindingCandidate],
    path: &[u32],
    path_is_name_based: bool,
    value: &RuntimeDataBindGraphValue,
) -> Option<bool> {
    runtime_owned_view_model_binding_source_for_candidates(
        file,
        candidates,
        path,
        path_is_name_based,
        true,
    )
    .or_else(|| {
        runtime_owned_view_model_binding_source_for_candidates(
            file,
            candidates,
            path,
            path_is_name_based,
            false,
        )
    })
    .map(|source| source.sync_value(value))
}

fn runtime_owned_view_model_value_for_candidates(
    file: &RuntimeFile,
    candidates: &[RuntimeOwnedViewModelBindingCandidate],
    path: &[u32],
    default_value: &RuntimeDataBindGraphValue,
) -> Option<RuntimeDataBindGraphValue> {
    candidates.iter().find_map(|candidate| {
        let context = candidate.context.borrow();
        candidate.context_chain.iter().find_map(|context_path| {
            default_value.resolve_from_owned_view_model_context_path(
                file,
                &context,
                context_path,
                path,
            )
        })
    })
}

fn runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_candidates(
    converter: &mut RuntimeDataBindGraphConverter,
    candidates: &[RuntimeOwnedViewModelBindingCandidate],
) -> bool {
    match converter {
        RuntimeDataBindGraphConverter::OperationViewModel {
            operation_value,
            source_path: Some(source_path),
            ..
        } => {
            let value = candidates
                .iter()
                .find_map(|candidate| {
                    let context = candidate.context.borrow();
                    let value = RuntimeDataBindGraphValue::Number(0.0)
                        .resolve_from_owned_view_model_instance(&context, source_path)?;
                    match value {
                        RuntimeDataBindGraphValue::Number(value) => Some(value),
                        _ => None,
                    }
                })
                .unwrap_or(0.0);
            if *operation_value == value {
                return false;
            }
            *operation_value = value;
            true
        }
        RuntimeDataBindGraphConverter::Group(converters) => {
            converters.iter_mut().fold(false, |changed, converter| {
                runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_candidates(
                    converter, candidates,
                ) || changed
            })
        }
        _ => false,
    }
}

fn runtime_owned_view_model_binding_value_for_property_path(
    context: &RuntimeOwnedViewModelInstance,
    property_path: &[usize],
) -> Option<RuntimeDataBindGraphValue> {
    context
        .number_value_by_property_path(property_path)
        .map(RuntimeDataBindGraphValue::Number)
        .or_else(|| {
            context
                .boolean_value_by_property_path(property_path)
                .map(RuntimeDataBindGraphValue::Boolean)
        })
        .or_else(|| {
            context
                .string_value_by_property_path(property_path)
                .map(|value| RuntimeDataBindGraphValue::String(value.to_vec()))
        })
        .or_else(|| {
            context
                .color_value_by_property_path(property_path)
                .map(RuntimeDataBindGraphValue::Color)
        })
        .or_else(|| {
            context
                .enum_value_by_property_path(property_path)
                .map(RuntimeDataBindGraphValue::Enum)
        })
        .or_else(|| {
            context
                .symbol_list_index_value_by_property_path(property_path)
                .map(RuntimeDataBindGraphValue::SymbolListIndex)
        })
        .or_else(|| {
            context
                .list_item_count_by_property_path(property_path)
                .map(|item_count| RuntimeDataBindGraphValue::List { item_count })
        })
        .or_else(|| {
            context
                .asset_value_by_property_path(property_path)
                .map(RuntimeDataBindGraphValue::Asset)
        })
        .or_else(|| {
            context
                .font_asset_value_by_property_path(property_path)
                .map(|value| RuntimeDataBindGraphValue::Asset(value.file_asset_index()))
        })
        .or_else(|| {
            context
                .artboard_value_by_property_path(property_path)
                .map(RuntimeDataBindGraphValue::Artboard)
        })
        .or_else(|| {
            context
                .trigger_value_by_property_path(property_path)
                .map(RuntimeDataBindGraphValue::Trigger)
        })
        .or_else(|| {
            context
                .view_model_value_by_property_path(property_path)
                .map(RuntimeDataBindGraphValue::ViewModel)
        })
}

fn runtime_font_asset_values_equal(
    current: &RuntimeFontAssetValue,
    next: &RuntimeFontAssetValue,
) -> bool {
    if current.file_asset_index() != next.file_asset_index() {
        return false;
    }
    match (current.live_font_bytes_arc(), next.live_font_bytes_arc()) {
        (Some(current), Some(next)) => {
            Arc::ptr_eq(current, next) || current.as_ref() == next.as_ref()
        }
        (None, None) => true,
        _ => false,
    }
}

fn runtime_owned_view_model_font_value_for_retained_context_chain(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelInstance,
    context_chain: &[&[usize]],
    path: &[u32],
    path_is_name_based: bool,
    scripting_manifest: bool,
    retained_source_path: &mut Option<Vec<usize>>,
) -> Option<RuntimeFontAssetValue> {
    if let Some(source_path) = retained_source_path.as_deref()
        && let Some(value) = context.font_asset_value_by_property_path(source_path)
    {
        return Some(value.clone());
    }

    let (source_path, value) = context_chain.iter().find_map(|context_path| {
        let source_path = context.property_path_for_context_source_path_with_manifest_mode(
            file,
            context_path,
            path,
            path_is_name_based,
            scripting_manifest,
        )?;
        let value = context.font_asset_value_by_property_path(&source_path)?;
        Some((source_path, value.clone()))
    })?;
    *retained_source_path = Some(source_path);
    Some(value)
}

fn runtime_owned_view_model_font_value_for_candidates(
    file: &RuntimeFile,
    candidates: &[RuntimeOwnedViewModelBindingCandidate],
    path: &[u32],
    path_is_name_based: bool,
    scripting_manifest: bool,
) -> Option<RuntimeFontAssetValue> {
    candidates.iter().find_map(|candidate| {
        let context = candidate.context.borrow();
        candidate.context_chain.iter().find_map(|context_path| {
            let source_path = context.property_path_for_context_source_path_with_manifest_mode(
                file,
                context_path,
                path,
                path_is_name_based,
                scripting_manifest,
            )?;
            context
                .font_asset_value_by_property_path(&source_path)
                .cloned()
        })
    })
}

fn runtime_owned_view_model_list_source_for_property_path(
    context: &RuntimeOwnedViewModelInstance,
    property_path: &[usize],
) -> Option<RuntimeArtboardListResolvedSource> {
    context
        .list_handle_by_property_path(property_path)
        .map(RuntimeArtboardListResolvedSource::List)
        .or_else(|| {
            runtime_owned_view_model_binding_value_for_property_path(context, property_path)
                .map(RuntimeArtboardListResolvedSource::Value)
        })
}

fn runtime_owned_view_model_set_binding_value_for_property_path(
    context: &mut RuntimeOwnedViewModelInstance,
    property_path: &[usize],
    value: &RuntimeDataBindGraphValue,
) -> Option<bool> {
    match value {
        RuntimeDataBindGraphValue::Number(value) => context
            .number_value_by_property_path(property_path)
            .map(|_| context.set_number_by_property_path(property_path, *value)),
        RuntimeDataBindGraphValue::Boolean(value) => context
            .boolean_value_by_property_path(property_path)
            .map(|_| context.set_boolean_by_property_path(property_path, *value)),
        RuntimeDataBindGraphValue::String(value) => {
            if context
                .string_value_by_property_path(property_path)
                .is_some()
            {
                Some(context.set_string_by_property_path(property_path, value))
            } else {
                None
            }
        }
        RuntimeDataBindGraphValue::Color(value) => context
            .color_value_by_property_path(property_path)
            .map(|_| context.set_color_by_property_path(property_path, *value)),
        RuntimeDataBindGraphValue::Enum(value) => context
            .enum_value_by_property_path(property_path)
            .map(|_| context.set_enum_by_property_path(property_path, *value)),
        RuntimeDataBindGraphValue::SymbolListIndex(value) => context
            .symbol_list_index_value_by_property_path(property_path)
            .map(|_| context.set_symbol_list_index_by_property_path(property_path, *value)),
        RuntimeDataBindGraphValue::Asset(value) => context
            .asset_value_by_property_path(property_path)
            .map(|_| context.set_asset_by_property_path(property_path, *value)),
        RuntimeDataBindGraphValue::Artboard(value) => context
            .artboard_value_by_property_path(property_path)
            .map(|_| context.set_artboard_by_property_path(property_path, *value)),
        RuntimeDataBindGraphValue::Trigger(value) => context
            .trigger_value_by_property_path(property_path)
            .map(|_| context.set_trigger_by_property_path(property_path, *value)),
        RuntimeDataBindGraphValue::List { .. }
        | RuntimeDataBindGraphValue::ListLength(_)
        | RuntimeDataBindGraphValue::ViewModel(_) => None,
    }
}

fn runtime_owned_view_model_missing_binding_value_for_context_chain(
    context_chain: &[&[usize]],
    binding: &RuntimeArtboardPropertyBindingInstance,
) -> Option<RuntimeDataBindGraphValue> {
    let text_property_key = runtime_data_bind_property_key_for_name("TextValueRun", "text")?;
    if binding.property_key != text_property_key {
        return None;
    }
    if !binding.path_is_name_based || !context_chain.iter().any(|path| !path.is_empty()) {
        return None;
    }
    match binding.default_value {
        RuntimeDataBindGraphValue::String(_) => Some(RuntimeDataBindGraphValue::String(Vec::new())),
        _ => None,
    }
}

const RUNTIME_OWNED_VIEW_MODEL_CONTEXT_CHAIN_INLINE: usize = 8;
const RUNTIME_OWNED_VIEW_MODEL_CONTEXT_PATH_INLINE: usize = 8;

enum RuntimeOwnedViewModelContextPathStorage<'a> {
    Borrowed(&'a [usize]),
    Inline {
        path: [usize; RUNTIME_OWNED_VIEW_MODEL_CONTEXT_PATH_INLINE],
        len: usize,
    },
    Heap(Vec<usize>),
}

impl<'a> RuntimeOwnedViewModelContextPathStorage<'a> {
    fn from_context_source_path(
        context: &RuntimeOwnedViewModelInstance,
        context_path: &'a [usize],
        source_path: &[u32],
    ) -> Option<Self> {
        if source_path.is_empty() {
            return None;
        }
        let view_model_index = context.view_model_index_by_property_path(context_path)?;
        if usize::try_from(source_path[0]).ok()? != view_model_index {
            return None;
        }
        let source_tail = &source_path[1..];
        if source_tail.is_empty() {
            return Some(Self::Borrowed(context_path));
        }
        let len = context_path.len() + source_tail.len();
        if len <= RUNTIME_OWNED_VIEW_MODEL_CONTEXT_PATH_INLINE {
            let mut path = [0; RUNTIME_OWNED_VIEW_MODEL_CONTEXT_PATH_INLINE];
            path[..context_path.len()].copy_from_slice(context_path);
            for (index, property_index) in source_tail.iter().enumerate() {
                path[context_path.len() + index] = usize::try_from(*property_index).ok()?;
            }
            return Some(Self::Inline { path, len });
        }
        let mut path = Vec::with_capacity(len);
        path.extend_from_slice(context_path);
        for property_index in source_tail {
            path.push(usize::try_from(*property_index).ok()?);
        }
        Some(Self::Heap(path))
    }

    fn as_slice(&self) -> &[usize] {
        match self {
            Self::Borrowed(path) => path,
            Self::Inline { path, len } => &path[..*len],
            Self::Heap(path) => path.as_slice(),
        }
    }
}

enum RuntimeOwnedViewModelContextChainStorage<'a> {
    Borrowed(&'a [&'a [usize]]),
    Inline {
        paths: [&'a [usize]; RUNTIME_OWNED_VIEW_MODEL_CONTEXT_CHAIN_INLINE],
        len: usize,
    },
    Heap(Vec<&'a [usize]>),
}

impl<'a> RuntimeOwnedViewModelContextChainStorage<'a> {
    fn with_child_context(
        context_chain: &'a [&'a [usize]],
        child_context: Option<&'a [usize]>,
    ) -> Self {
        let Some(child_context) = child_context else {
            return Self::Borrowed(context_chain);
        };
        let len = context_chain.len() + 1;
        if len <= RUNTIME_OWNED_VIEW_MODEL_CONTEXT_CHAIN_INLINE {
            let empty: &'a [usize] = &[];
            let mut paths = [empty; RUNTIME_OWNED_VIEW_MODEL_CONTEXT_CHAIN_INLINE];
            paths[0] = child_context;
            for (index, context_path) in context_chain.iter().enumerate() {
                paths[index + 1] = *context_path;
            }
            return Self::Inline { paths, len };
        }
        let mut paths = Vec::with_capacity(len);
        paths.push(child_context);
        paths.extend_from_slice(context_chain);
        Self::Heap(paths)
    }

    fn as_slice(&self) -> &[&'a [usize]] {
        match self {
            Self::Borrowed(paths) => paths,
            Self::Inline { paths, len } => &paths[..*len],
            Self::Heap(paths) => paths.as_slice(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardListBindingInstance {
    data_bind_index: usize,
    target_local_id: usize,
    path: Vec<u32>,
    converter: Option<RuntimeDataBindGraphConverter>,
    default_value: RuntimeDataBindGraphValue,
    source_value: Option<RuntimeDataBindGraphValue>,
    source_list_size: Option<usize>,
    source_number_value: Option<f32>,
    target_list_size: Option<usize>,
    should_reset_instances: bool,
    generated_view_model_id: Option<usize>,
    generated_items: Vec<RuntimeOwnedViewModelHandle>,
}

enum RuntimeArtboardListResolvedSource {
    List(RuntimeOwnedViewModelListHandle),
    Value(RuntimeDataBindGraphValue),
}

struct RuntimeArtboardListResolvedUpdate {
    target_local_id: usize,
    source: Option<RuntimeOwnedViewModelListHandle>,
    items: Option<Vec<RuntimeOwnedViewModelHandle>>,
    binding_changed: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct RuntimeArtboardListConvertedValue {
    value: RuntimeDataBindGraphValue,
    generated_view_model_id: Option<u64>,
}

fn runtime_artboard_convert_list_value(
    converter: &RuntimeDataBindGraphConverter,
    converted: RuntimeArtboardListConvertedValue,
) -> Option<RuntimeArtboardListConvertedValue> {
    if let RuntimeDataBindGraphConverter::Group(converters) = converter {
        return converters
            .iter()
            .try_fold(converted, |converted, converter| {
                runtime_artboard_convert_list_value(converter, converted)
            });
    }

    let input_was_number = matches!(converted.value, RuntimeDataBindGraphValue::Number(_));
    let input_was_list = matches!(converted.value, RuntimeDataBindGraphValue::List { .. });
    let value = runtime_data_bind_graph_convert_value(converter, &converted.value)?;
    let generated_view_model_id = match converter {
        RuntimeDataBindGraphConverter::NumberToList { view_model_id, .. } if input_was_number => {
            Some(*view_model_id)
        }
        _ if input_was_list && matches!(value, RuntimeDataBindGraphValue::List { .. }) => {
            converted.generated_view_model_id
        }
        _ => None,
    };
    Some(RuntimeArtboardListConvertedValue {
        value,
        generated_view_model_id,
    })
}

impl RuntimeArtboardListBindingInstance {
    fn update_metadata(
        &mut self,
        source_list_size: Option<usize>,
        source_number_value: Option<f32>,
        target_list_size: Option<usize>,
        should_reset_instances: bool,
        source_value: Option<RuntimeDataBindGraphValue>,
    ) -> bool {
        let changed = self.source_list_size != source_list_size
            || self.source_number_value != source_number_value
            || self.target_list_size != target_list_size
            || self.should_reset_instances != should_reset_instances
            || self.source_value != source_value;
        self.source_value = source_value;
        self.source_list_size = source_list_size;
        self.source_number_value = source_number_value;
        self.target_list_size = target_list_size;
        self.should_reset_instances = should_reset_instances;
        changed
    }

    /// Mirrors C++ `DataConverterNumberToList::convert`: generated instances
    /// are cached, the common prefix survives size changes, and changing the
    /// converter's view-model id invalidates the complete cache.
    fn reconcile_number_to_list_items(
        &mut self,
        file: &RuntimeFile,
        source_value: RuntimeDataBindGraphValue,
        target_size: usize,
        generated_view_model_id: u64,
    ) -> RuntimeArtboardListResolvedUpdate {
        let view_model_id = usize::try_from(generated_view_model_id)
            .ok()
            .filter(|&index| index < file.view_models().len());

        let mut cache_changed = false;
        if self.generated_view_model_id != view_model_id {
            self.generated_items.clear();
            self.generated_view_model_id = view_model_id;
            cache_changed = true;
        }

        let requested_size = if view_model_id.is_some() {
            target_size
        } else {
            0
        };
        if self.generated_items.len() > requested_size {
            self.generated_items.truncate(requested_size);
            cache_changed = true;
        }
        if let Some(view_model_id) = view_model_id {
            while self.generated_items.len() < requested_size {
                let Some(instance) =
                    RuntimeOwnedViewModelInstance::from_instance(file, view_model_id, 0)
                        .or_else(|| RuntimeOwnedViewModelInstance::new(file, view_model_id))
                else {
                    break;
                };
                self.generated_items
                    .push(RuntimeOwnedViewModelHandle::new(instance));
                cache_changed = true;
            }
        }

        let source_number_value = match &source_value {
            RuntimeDataBindGraphValue::Number(value) => Some(*value),
            _ => None,
        };
        let source_list_size = match &source_value {
            RuntimeDataBindGraphValue::List { item_count } => Some(*item_count),
            _ => None,
        };
        let binding_changed = self.update_metadata(
            source_list_size,
            source_number_value,
            Some(self.generated_items.len()),
            true,
            Some(source_value),
        ) || cache_changed;
        RuntimeArtboardListResolvedUpdate {
            target_local_id: self.target_local_id,
            source: None,
            items: cache_changed.then(|| self.generated_items.clone()),
            binding_changed,
        }
    }

    fn apply_resolved_source(
        &mut self,
        file: &RuntimeFile,
        source: RuntimeArtboardListResolvedSource,
    ) -> RuntimeArtboardListResolvedUpdate {
        let (source_handle, source_value) = match source {
            RuntimeArtboardListResolvedSource::List(source) => {
                let item_count = source.items().len();
                (Some(source), RuntimeDataBindGraphValue::List { item_count })
            }
            RuntimeArtboardListResolvedSource::Value(source_value) => (None, source_value),
        };
        let converted = match self.converter.as_ref() {
            Some(converter) => runtime_artboard_convert_list_value(
                converter,
                RuntimeArtboardListConvertedValue {
                    value: source_value.clone(),
                    generated_view_model_id: None,
                },
            ),
            None => Some(RuntimeArtboardListConvertedValue {
                value: source_value.clone(),
                generated_view_model_id: None,
            }),
        };

        if let Some(RuntimeArtboardListConvertedValue {
            value: RuntimeDataBindGraphValue::List { item_count },
            generated_view_model_id: Some(view_model_id),
        }) = converted.as_ref()
        {
            return self.reconcile_number_to_list_items(
                file,
                source_value,
                *item_count,
                *view_model_id,
            );
        }

        if let (
            Some(source),
            Some(RuntimeArtboardListConvertedValue {
                value: RuntimeDataBindGraphValue::List { item_count },
                generated_view_model_id: None,
            }),
        ) = (source_handle, converted)
        {
            let items = source.items();
            let source_item_count = match source_value {
                RuntimeDataBindGraphValue::List { item_count } => item_count,
                _ => items.len(),
            };
            let binding_changed = self.update_metadata(
                Some(source_item_count),
                None,
                Some(item_count),
                false,
                Some(source_value),
            );
            return RuntimeArtboardListResolvedUpdate {
                target_local_id: self.target_local_id,
                source: Some(source),
                items: Some(items),
                binding_changed,
            };
        }

        let cache_changed =
            !self.generated_items.is_empty() || self.generated_view_model_id.take().is_some();
        self.generated_items.clear();
        let source_list_size = match &source_value {
            RuntimeDataBindGraphValue::List { item_count } => Some(*item_count),
            _ => None,
        };
        let source_number_value = match &source_value {
            RuntimeDataBindGraphValue::Number(value) => Some(*value),
            _ => None,
        };
        let binding_changed = self.update_metadata(
            source_list_size,
            source_number_value,
            None,
            false,
            Some(source_value),
        ) || cache_changed;
        RuntimeArtboardListResolvedUpdate {
            target_local_id: self.target_local_id,
            source: None,
            items: Some(Vec::new()),
            binding_changed,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardTextListBindingInstance {
    target_local_id: usize,
    path: Vec<u32>,
    path_is_name_based: bool,
    source: Option<RuntimeOwnedViewModelListHandle>,
}

impl RuntimeArtboardTextListBindingInstance {
    pub(super) fn target_local_id(&self) -> usize {
        self.target_local_id
    }

    pub(super) fn text_runs(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.source
            .as_ref()
            .map(RuntimeOwnedViewModelListHandle::text_runs)
            .unwrap_or_default()
    }
}

pub(super) fn build_artboard_text_list_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardTextListBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let Some(text_run_list_source_key) = property_key_for_name("Text", "textRunListSource") else {
        return Vec::new();
    };
    file.artboard_data_binds(artboard_index)
        .into_iter()
        .filter_map(|data_bind| {
            let target = data_bind.target?;
            if !(target.type_name == "Text"
                && data_bind.object.uint_property("propertyKey")
                    == Some(u64::from(text_run_list_source_key))
                && data_bind_flags_apply_source_to_target(
                    data_bind.object.uint_property("flags").unwrap_or(0),
                ))
            {
                return None;
            }
            Some(RuntimeArtboardTextListBindingInstance {
                target_local_id: data_bind.target_local_id?,
                path: file
                    .data_bind_context_source_path_ids_for_object(data_bind.object)?
                    .to_vec(),
                path_is_name_based: file
                    .data_bind_is_name_based_for_object(data_bind.object)
                    .unwrap_or(false),
                source: None,
            })
        })
        .collect()
}

pub(super) fn build_artboard_list_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardListBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let default_instance = artboard_default_view_model_instance(file, artboard_index);

    file.artboard_data_binds(artboard_index)
        .into_iter()
        .enumerate()
        .filter_map(|(data_bind_index, data_bind)| {
            let target = data_bind.target?;
            if target.type_name != "ArtboardComponentList" {
                return None;
            }
            let target_local_id = data_bind.target_local_id?;
            let path_is_name_based = file
                .data_bind_is_name_based_for_object(data_bind.object)
                .unwrap_or(false);
            let path = file.data_bind_context_source_path_ids_for_object(data_bind.object)?;
            let converter = runtime_data_bind_graph_converter(file, data_bind.object);
            let source = default_instance.as_ref().and_then(|default_instance| {
                file.data_context_view_model_property_for_instance(default_instance.object, &path)
            });
            let source_is_unresolved_name_based = path_is_name_based && source.is_none();
            let converts_number_to_list = converter
                .as_ref()
                .and_then(RuntimeDataBindGraphConverter::number_to_list_view_model_id)
                .is_some();
            let default_value = source
                .and_then(|source| {
                    if converts_number_to_list {
                        runtime_created_view_model_value_for_source(file, source)
                    } else if converter.is_none() {
                        file.view_model_instance_list_size_for_object(source)
                            .map(|item_count| RuntimeDataBindGraphValue::List { item_count })
                    } else {
                        None
                    }
                })
                .or_else(|| {
                    if converts_number_to_list {
                        Some(RuntimeDataBindGraphValue::Number(0.0))
                    } else if converter.is_none() {
                        Some(RuntimeDataBindGraphValue::List { item_count: 0 })
                    } else {
                        None
                    }
                })?;

            Some(RuntimeArtboardListBindingInstance {
                data_bind_index,
                target_local_id,
                path: path.to_vec(),
                converter,
                default_value,
                source_value: None,
                source_list_size: None,
                source_number_value: None,
                target_list_size: source_is_unresolved_name_based.then_some(0),
                should_reset_instances: false,
                generated_view_model_id: None,
                generated_items: Vec::new(),
            })
        })
        .collect()
}

pub(super) fn build_artboard_property_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardPropertyBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let default_instance = artboard_default_view_model_instance(file, artboard_index);
    let target_to_source_paths = file
        .artboard_data_binds(artboard_index)
        .into_iter()
        .filter(|data_bind| {
            data_bind_flags_apply_target_to_source(
                data_bind.object.uint_property("flags").unwrap_or(0),
            )
        })
        .filter_map(|data_bind| file.data_bind_context_source_path_ids_for_object(data_bind.object))
        .collect::<BTreeSet<_>>();

    file.artboard_data_binds(artboard_index)
        .into_iter()
        .enumerate()
        .filter_map(|(data_bind_index, data_bind)| {
            let flags = data_bind.object.uint_property("flags").unwrap_or(0);
            if !data_bind_flags_apply_source_to_target(flags) {
                return None;
            }
            let target = data_bind.target?;
            if matches!(target.type_name, "ArtboardComponentList" | "Solo") {
                return None;
            }
            let target_local_id = data_bind.target_local_id?;
            let property_key =
                u16::try_from(data_bind.object.uint_property("propertyKey")?).ok()?;
            let target_is_nested_artboard = runtime_type_is_a(target.type_key, "NestedArtboard");
            if target_is_nested_artboard
                && [
                    runtime_data_bind_property_key_for_name("NestedArtboard", "artboardId"),
                    runtime_data_bind_property_key_for_name("NestedArtboard", "isPaused"),
                    runtime_data_bind_property_key_for_name("NestedArtboard", "speed"),
                    runtime_data_bind_property_key_for_name("NestedArtboard", "quantize"),
                ]
                .contains(&Some(property_key))
            {
                return None;
            }
            let Some(property_kind) =
                nuxie_schema::core_registry_setter_field_kind_by_property_key(property_key)
            else {
                return None;
            };
            if !matches!(
                property_kind,
                FieldKind::Double
                    | FieldKind::Uint
                    | FieldKind::Bool
                    | FieldKind::Color
                    | FieldKind::String
            ) {
                return None;
            }
            let converter = runtime_data_bind_graph_converter(file, data_bind.object);
            if matches!(converter, Some(RuntimeDataBindGraphConverter::Unsupported)) {
                return None;
            }
            let path = file.data_bind_context_source_path_ids_for_object(data_bind.object)?;
            let enum_value_names = runtime_enum_value_names_for_data_bind_path(
                file,
                default_instance.as_ref(),
                data_bind.object,
                &path,
            );
            let path_is_name_based = file
                .data_bind_is_name_based_for_object(data_bind.object)
                .unwrap_or(false);
            let resolved_default_value = default_instance
                .as_ref()
                .and_then(|default_instance| {
                    file.data_context_view_model_property_for_instance(
                        default_instance.object,
                        &path,
                    )
                    .and_then(|source| runtime_created_view_model_value_for_source(file, source))
                })
                .or_else(|| {
                    if path_is_name_based {
                        return None;
                    }
                    runtime_created_view_model_value_for_declared_path(file, &path)
                });
            if resolved_default_value.is_none() && !path_is_name_based {
                return None;
            }
            let default_value =
                resolved_default_value
                    .clone()
                    .unwrap_or_else(|| match property_kind {
                        FieldKind::Bool => RuntimeDataBindGraphValue::Boolean(false),
                        FieldKind::Color => RuntimeDataBindGraphValue::Color(0xFF000000),
                        FieldKind::String => RuntimeDataBindGraphValue::String(Vec::new()),
                        FieldKind::Double | FieldKind::Uint => {
                            RuntimeDataBindGraphValue::Number(0.0)
                        }
                        _ => unreachable!("property kind was filtered above"),
                    });
            if !artboard_property_binding_accepts_default(
                converter.as_ref(),
                &default_value,
                property_kind,
            ) {
                return None;
            }
            let snapshots_source_value = converter.is_none()
                && matches!(target.type_name, "LinearGradient" | "RadialGradient")
                && target_to_source_paths.contains(&path);
            let pending_value = snapshots_source_value
                .then(|| resolved_default_value.clone())
                .flatten();

            Some(RuntimeArtboardPropertyBindingInstance {
                data_bind_index,
                flags,
                target_local_id,
                property_key,
                path: path.to_vec(),
                path_is_name_based,
                owned_context_source_path: None,
                enum_value_names,
                converter_state: RuntimeDataBindGraphConverterState::for_converter(
                    converter.as_ref(),
                ),
                converter,
                pending_value,
                default_value,
                default_value_is_resolved: resolved_default_value.is_some(),
                snapshots_source_value,
            })
        })
        .collect()
}

pub(super) fn build_artboard_image_asset_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardImageAssetBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let Some(image_asset_id_key) = runtime_data_bind_property_key_for_name("Image", "assetId")
    else {
        return Vec::new();
    };
    let font_asset_id_key = runtime_data_bind_property_key_for_name("TextStyle", "fontAssetId");
    let default_instance = artboard_default_view_model_instance(file, artboard_index);

    file.artboard_data_binds(artboard_index)
        .into_iter()
        .filter_map(|data_bind| {
            if !data_bind_flags_apply_source_to_target(
                data_bind.object.uint_property("flags").unwrap_or(0),
            ) {
                return None;
            }
            let target = data_bind.target?;
            let target_local_id = data_bind.target_local_id?;
            let property_key =
                u16::try_from(data_bind.object.uint_property("propertyKey")?).ok()?;
            let target = if target.type_name == "Image" && property_key == image_asset_id_key {
                RuntimeArtboardAssetBindingTarget::Image(target_local_id)
            } else if matches!(target.type_name, "TextStyle" | "TextStylePaint")
                && Some(property_key) == font_asset_id_key
            {
                RuntimeArtboardAssetBindingTarget::Font(target_local_id)
            } else {
                return None;
            };
            let path = file.data_bind_context_source_path_ids_for_object(data_bind.object)?;
            let path_is_name_based = file
                .data_bind_is_name_based_for_object(data_bind.object)
                .unwrap_or(false);
            let default_value = default_instance
                .as_ref()
                .and_then(|default_instance| {
                    file.data_context_view_model_property_for_instance(
                        default_instance.object,
                        &path,
                    )
                    .and_then(|source| runtime_created_view_model_value_for_source(file, source))
                })
                .or_else(|| {
                    if path_is_name_based {
                        return None;
                    }
                    runtime_created_view_model_value_for_declared_path(file, &path)
                })?;
            if !matches!(default_value, RuntimeDataBindGraphValue::Asset(_)) {
                return None;
            }

            Some(RuntimeArtboardImageAssetBindingInstance {
                target,
                path,
                path_is_name_based,
                owned_context_source_path: None,
                font_value: target.is_font().then(|| {
                    let RuntimeDataBindGraphValue::Asset(index) = &default_value else {
                        unreachable!("asset binding default was checked above")
                    };
                    RuntimeFontAssetValue::from_file_asset_index(*index)
                }),
                default_value,
            })
        })
        .collect()
}

fn artboard_property_binding_value_matches_kind(
    value: &RuntimeDataBindGraphValue,
    property_kind: FieldKind,
) -> bool {
    matches!(
        (value, property_kind),
        (
            RuntimeDataBindGraphValue::Number(_),
            FieldKind::Double | FieldKind::Uint
        ) | (RuntimeDataBindGraphValue::Boolean(_), FieldKind::Bool)
            | (RuntimeDataBindGraphValue::Color(_), FieldKind::Color)
            | (RuntimeDataBindGraphValue::String(_), FieldKind::String)
            | (RuntimeDataBindGraphValue::Enum(_), FieldKind::Uint)
            | (RuntimeDataBindGraphValue::ViewModel(_), FieldKind::Uint)
    )
}

fn artboard_property_binding_allows_converted_default(
    converter: Option<&RuntimeDataBindGraphConverter>,
    default_value: &RuntimeDataBindGraphValue,
    property_kind: FieldKind,
) -> bool {
    let Some(converter) = converter else {
        return false;
    };
    let mut state = RuntimeDataBindGraphConverterState::for_converter(Some(converter));
    state
        .convert_value(converter, default_value)
        .as_ref()
        .is_some_and(|value| artboard_property_binding_value_matches_kind(value, property_kind))
}

fn artboard_property_binding_accepts_default(
    converter: Option<&RuntimeDataBindGraphConverter>,
    default_value: &RuntimeDataBindGraphValue,
    property_kind: FieldKind,
) -> bool {
    artboard_property_binding_value_matches_kind(default_value, property_kind)
        || artboard_property_binding_allows_converted_default(
            converter,
            default_value,
            property_kind,
        )
        || converter.is_some_and(RuntimeDataBindGraphConverter::can_change_output_kind)
}

fn runtime_artboard_convert_property_binding_value(
    converter: &RuntimeDataBindGraphConverter,
    converter_state: &mut RuntimeDataBindGraphConverterState,
    value: RuntimeDataBindGraphValue,
    enum_value_names: &[Vec<u8>],
    formula_random_source: &mut RuntimeDataBindGraphFormulaRandomSource,
) -> Option<RuntimeDataBindGraphValue> {
    if matches!(converter, RuntimeDataBindGraphConverter::ToString { .. })
        && let RuntimeDataBindGraphValue::Enum(value) = &value
    {
        return enum_value_names
            .get(usize::try_from(*value).ok()?)
            .cloned()
            .map(RuntimeDataBindGraphValue::String);
    }
    converter_state.convert_value_with_formula_randoms(converter, &value, formula_random_source)
}

fn runtime_image_asset_global_for_file_asset_index(
    file: &RuntimeFile,
    asset_index: u64,
) -> Option<u32> {
    let asset = file.file_asset(usize::try_from(asset_index).ok()?)?;
    (asset.type_name == "ImageAsset").then_some(asset.id)
}

pub(super) fn build_artboard_nested_host_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardNestedHostBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let artboard_id_key = runtime_data_bind_property_key_for_name("NestedArtboard", "artboardId");
    let is_paused_key = runtime_data_bind_property_key_for_name("NestedArtboard", "isPaused");
    let speed_key = runtime_data_bind_property_key_for_name("NestedArtboard", "speed");
    let quantize_key = runtime_data_bind_property_key_for_name("NestedArtboard", "quantize");

    file.artboard_data_binds(artboard_index)
        .into_iter()
        .filter_map(|data_bind| {
            if !data_bind_flags_apply_source_to_target(
                data_bind.object.uint_property("flags").unwrap_or(0),
            ) {
                return None;
            }
            let target = data_bind.target?;
            if !runtime_type_is_a(target.type_key, "NestedArtboard") {
                return None;
            }
            let property_key =
                u16::try_from(data_bind.object.uint_property("propertyKey")?).ok()?;
            let property = if Some(property_key) == artboard_id_key {
                RuntimeArtboardNestedHostProperty::ArtboardId { property_key }
            } else if Some(property_key) == is_paused_key {
                RuntimeArtboardNestedHostProperty::IsPaused { property_key }
            } else if Some(property_key) == speed_key {
                RuntimeArtboardNestedHostProperty::Speed { property_key }
            } else if Some(property_key) == quantize_key {
                RuntimeArtboardNestedHostProperty::Quantize { property_key }
            } else {
                return None;
            };
            Some(RuntimeArtboardNestedHostBindingInstance {
                target_local_id: data_bind.target_local_id?,
                property,
                path: file.data_bind_context_source_path_ids_for_object(data_bind.object)?,
                path_is_name_based: file
                    .data_bind_is_name_based_for_object(data_bind.object)
                    .unwrap_or(false),
                owned_context_source_path: None,
                // C++ DataBindViewModel::update bypasses the generated
                // NestedArtboard::artboardId equality guard and calls
                // NestedArtboard::updateArtboard even when the first bound
                // value equals the authored source. Preserve that first
                // clone so instance-owned paints and state have the same
                // lifetime as C++.
                artboard_value_applied: false,
            })
        })
        .collect()
}

fn runtime_type_is_a(type_key: u16, type_name: &str) -> bool {
    definition_by_type_key(type_key).is_some_and(|definition| definition.is_a(type_name))
}

pub(super) fn build_artboard_default_view_model_values(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> BTreeMap<Vec<u32>, RuntimeDataBindGraphValue> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return BTreeMap::new();
    };
    // C++ leaves authored targets untouched when createViewModelInstance()
    // returns null. A declared ViewModel describes the value kinds, but it is
    // not itself a bound DataContext and must not seed source-to-target work.
    let Some(default_instance) = artboard_default_view_model_instance(file, artboard_index) else {
        return BTreeMap::new();
    };

    let mut values = BTreeMap::new();
    for data_bind in file.artboard_data_binds(artboard_index) {
        let Some(path) = file.data_bind_context_source_path_ids_for_object(data_bind.object) else {
            continue;
        };
        let Some(value) =
            runtime_created_view_model_value_for_path(file, default_instance.object, &path)
                .or_else(|| {
                    if file
                        .data_bind_is_name_based_for_object(data_bind.object)
                        .unwrap_or(false)
                    {
                        return None;
                    }
                    runtime_created_view_model_value_for_declared_path(file, &path)
                })
        else {
            continue;
        };
        values.entry(path).or_insert(value);
    }
    values
}

pub(super) fn apply_artboard_name_based_color_data_bind_defaults(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    objects: &mut InstanceObjectArena,
) -> bool {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return false;
    };
    let Some(color_key) = solid_color_value_property_key() else {
        return false;
    };

    let mut changed = false;
    for data_bind in file.artboard_data_binds(artboard_index) {
        if !data_bind_flags_apply_source_to_target(
            data_bind.object.uint_property("flags").unwrap_or(0),
        ) {
            continue;
        }
        if !file
            .data_bind_is_name_based_for_object(data_bind.object)
            .unwrap_or(false)
        {
            continue;
        }
        if data_bind
            .target
            .is_none_or(|target| target.type_name != "SolidColor")
        {
            continue;
        }
        if u16::try_from(data_bind.object.uint_property("propertyKey").unwrap_or(0)).ok()
            != Some(color_key)
        {
            continue;
        }
        let Some(target_local_id) = data_bind.target_local_id else {
            continue;
        };
        changed |= objects.set_color_property(target_local_id, color_key, 0xFF000000);
    }
    changed
}

pub(super) fn build_artboard_custom_property_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardCustomPropertyBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let default_instance = artboard_default_view_model_instance(file, artboard_index);

    file.artboard_data_binds(artboard_index)
        .into_iter()
        .enumerate()
        .filter_map(|(data_bind_index, data_bind)| {
            let flags = data_bind.object.uint_property("flags").unwrap_or(0);
            if !data_bind_flags_apply_target_to_source(flags) {
                return None;
            }
            let target = data_bind.target?;
            let target_local_id = data_bind.target_local_id?;
            let property_key =
                u16::try_from(data_bind.object.uint_property("propertyKey")?).ok()?;
            let value_kind = match target.type_name {
                "CustomPropertyNumber"
                    if runtime_data_bind_property_key_for_name(
                        "CustomPropertyNumber",
                        "propertyValue",
                    ) == Some(property_key) =>
                {
                    RuntimeArtboardDataBindValueKind::Number
                }
                "CustomPropertyBoolean"
                    if runtime_data_bind_property_key_for_name(
                        "CustomPropertyBoolean",
                        "propertyValue",
                    ) == Some(property_key) =>
                {
                    RuntimeArtboardDataBindValueKind::Boolean
                }
                "CustomPropertyString"
                    if runtime_data_bind_property_key_for_name(
                        "CustomPropertyString",
                        "propertyValue",
                    ) == Some(property_key) =>
                {
                    RuntimeArtboardDataBindValueKind::String
                }
                "CustomPropertyColor"
                    if runtime_data_bind_property_key_for_name(
                        "CustomPropertyColor",
                        "propertyValue",
                    ) == Some(property_key) =>
                {
                    RuntimeArtboardDataBindValueKind::Color
                }
                "CustomPropertyEnum"
                    if runtime_data_bind_property_key_for_name(
                        "CustomPropertyEnum",
                        "propertyValue",
                    ) == Some(property_key) =>
                {
                    RuntimeArtboardDataBindValueKind::Enum
                }
                "CustomPropertyTrigger"
                    if runtime_data_bind_property_key_for_name(
                        "CustomPropertyTrigger",
                        "propertyValue",
                    ) == Some(property_key) =>
                {
                    RuntimeArtboardDataBindValueKind::Trigger
                }
                _ => return None,
            };
            let path = file.data_bind_context_source_path_ids_for_object(data_bind.object)?;
            let converter = runtime_data_bind_graph_converter(file, data_bind.object);
            if matches!(converter, Some(RuntimeDataBindGraphConverter::Unsupported)) {
                return None;
            }
            let default_value = default_instance
                .as_ref()
                .and_then(|default_instance| {
                    file.data_context_view_model_property_for_instance(
                        default_instance.object,
                        &path,
                    )
                    .and_then(|source| runtime_created_view_model_value_for_source(file, source))
                })
                .or_else(|| {
                    if file
                        .data_bind_is_name_based_for_object(data_bind.object)
                        .unwrap_or(false)
                    {
                        return None;
                    }
                    runtime_created_view_model_value_for_declared_path(file, &path)
                })
                .unwrap_or_else(|| runtime_artboard_data_bind_default_value_for_kind(value_kind));
            Some(RuntimeArtboardCustomPropertyBindingInstance {
                data_bind_index,
                target_local_id,
                property_key,
                path: shared_data_bind_path(path),
                path_is_name_based: file
                    .data_bind_is_name_based_for_object(data_bind.object)
                    .unwrap_or(false),
                owned_context_source_path: None,
                owned_context_source: None,
                flags,
                value_kind,
                converter_state: RuntimeDataBindGraphConverterState::for_converter(
                    converter.as_ref(),
                ),
                converter,
                default_value,
            })
        })
        .collect()
}

pub(super) fn build_artboard_numeric_source_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardNumericSourceBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let trim_start_key = runtime_data_bind_property_key_for_name("TrimPath", "start");
    let trim_end_key = runtime_data_bind_property_key_for_name("TrimPath", "end");
    let shape_length_key = runtime_data_bind_property_key_for_name("Shape", "length");
    let parametric_width_key = runtime_data_bind_property_key_for_name("ParametricPath", "width");
    let parametric_height_key = runtime_data_bind_property_key_for_name("ParametricPath", "height");

    file.artboard_data_binds(artboard_index)
        .into_iter()
        .enumerate()
        .filter_map(|(data_bind_index, data_bind)| {
            if !data_bind_flags_apply_target_to_source(
                data_bind.object.uint_property("flags").unwrap_or(0),
            ) {
                return None;
            }
            let target = data_bind.target?;
            let property_key =
                u16::try_from(data_bind.object.uint_property("propertyKey")?).ok()?;
            let source_path =
                file.data_bind_context_source_path_ids_for_object(data_bind.object)?;
            let property = match target.type_name {
                "TrimPath" if Some(property_key) == trim_start_key => {
                    RuntimeArtboardNumericSourceProperty::DirectDouble
                }
                "TrimPath" if Some(property_key) == trim_end_key => {
                    RuntimeArtboardNumericSourceProperty::DirectDouble
                }
                "Shape" if Some(property_key) == shape_length_key => {
                    RuntimeArtboardNumericSourceProperty::ShapeLength
                }
                _ if runtime_type_is_a(target.type_key, "ParametricPath")
                    && [parametric_width_key, parametric_height_key]
                        .contains(&Some(property_key)) =>
                {
                    RuntimeArtboardNumericSourceProperty::DirectDouble
                }
                _ => return None,
            };
            Some(RuntimeArtboardNumericSourceBindingInstance {
                data_bind_index,
                target_local_id: data_bind.target_local_id?,
                property_key,
                property,
                path: source_path,
                owned_context_source: None,
                path_is_name_based: file
                    .data_bind_is_name_based_for_object(data_bind.object)
                    .unwrap_or(false),
                owned_context_source_path: None,
            })
        })
        .collect()
}

pub(super) fn build_artboard_formula_token_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardFormulaTokenBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let formula_token_operation_value_key =
        runtime_data_bind_property_key_for_name("FormulaTokenValue", "operationValue");
    let converter_operation_value_key =
        runtime_data_bind_property_key_for_name("DataConverterOperationValue", "operationValue");
    if formula_token_operation_value_key.is_none() && converter_operation_value_key.is_none() {
        return Vec::new();
    }
    let default_instance = artboard_default_view_model_instance(file, artboard_index);
    let artboard_converters = file
        .artboard_data_binds(artboard_index)
        .into_iter()
        .filter_map(|data_bind| runtime_data_bind_graph_converter(file, data_bind.object))
        .collect::<Vec<_>>();
    file.objects
        .iter()
        .flatten()
        .filter(|object| object.type_name == "DataBindContext")
        .filter_map(|data_bind| {
            let data_bind_id = usize::try_from(data_bind.id).ok()?;
            if file.import_status(data_bind_id) != Some(nuxie_binary::RuntimeImportStatus::Imported)
            {
                return None;
            }
            if !data_bind_flags_apply_source_to_target(
                data_bind.uint_property("flags").unwrap_or(0),
            ) {
                return None;
            }
            let target = file.data_bind_target_for_object(data_bind)?;
            let artboard_converter_reachable = artboard_converters.iter().any(|converter| {
                runtime_data_bind_graph_converter_contains_global_id(converter, target.id)
            });
            let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
            let target = match target.type_name {
                "FormulaTokenValue" if Some(property_key) == formula_token_operation_value_key => {
                    RuntimeArtboardFormulaBindingTarget::FormulaToken {
                        global_id: target.id,
                    }
                }
                "DataConverterOperationValue"
                | "DataConverterSystemDegsToRads"
                | "DataConverterSystemNormalizer"
                    if Some(property_key) == converter_operation_value_key =>
                {
                    RuntimeArtboardFormulaBindingTarget::OperationValue {
                        global_id: target.id,
                    }
                }
                _ => return None,
            };
            let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
            let converter = runtime_data_bind_graph_converter(file, data_bind);
            if matches!(converter, Some(RuntimeDataBindGraphConverter::Unsupported)) {
                return None;
            }
            let default_value = default_instance
                .as_ref()
                .and_then(|default_instance| {
                    file.data_context_view_model_property_for_instance(
                        default_instance.object,
                        &path,
                    )
                    .and_then(|source| runtime_created_view_model_value_for_source(file, source))
                })
                .or_else(|| runtime_created_view_model_value_for_declared_path(file, &path))
                .unwrap_or(RuntimeDataBindGraphValue::Number(0.0));
            let default_is_number = match converter.as_ref() {
                Some(converter) => runtime_data_bind_graph_convert_value(converter, &default_value)
                    .is_some_and(|value| matches!(value, RuntimeDataBindGraphValue::Number(_))),
                None => matches!(default_value, RuntimeDataBindGraphValue::Number(_)),
            };
            if !default_is_number {
                return None;
            }

            Some(RuntimeArtboardFormulaTokenBindingInstance {
                target,
                path,
                artboard_converter_reachable,
                converter_state: RuntimeDataBindGraphConverterState::for_converter(
                    converter.as_ref(),
                ),
                converter,
                default_value,
            })
        })
        .collect()
}

pub(super) fn build_artboard_converter_property_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardConverterPropertyBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let decimals_key = runtime_data_bind_property_key_for_name("DataConverterToString", "decimals");
    let color_format_key =
        runtime_data_bind_property_key_for_name("DataConverterToString", "colorFormat");
    let string_trim_trim_type_key =
        runtime_data_bind_property_key_for_name("DataConverterStringTrim", "trimType");
    let string_pad_length_key =
        runtime_data_bind_property_key_for_name("DataConverterStringPad", "length");
    let string_pad_text_key =
        runtime_data_bind_property_key_for_name("DataConverterStringPad", "text");
    let string_pad_pad_type_key =
        runtime_data_bind_property_key_for_name("DataConverterStringPad", "padType");
    let interpolator_duration_key =
        runtime_data_bind_property_key_for_name("DataConverterInterpolator", "duration");
    let range_mapper_min_input_key =
        runtime_data_bind_property_key_for_name("DataConverterRangeMapper", "minInput");
    let range_mapper_max_input_key =
        runtime_data_bind_property_key_for_name("DataConverterRangeMapper", "maxInput");
    let range_mapper_min_output_key =
        runtime_data_bind_property_key_for_name("DataConverterRangeMapper", "minOutput");
    let range_mapper_max_output_key =
        runtime_data_bind_property_key_for_name("DataConverterRangeMapper", "maxOutput");
    let number_to_list_view_model_id_key =
        runtime_data_bind_property_key_for_name("DataConverterNumberToList", "viewModelId");
    if decimals_key.is_none()
        && color_format_key.is_none()
        && string_trim_trim_type_key.is_none()
        && string_pad_length_key.is_none()
        && string_pad_text_key.is_none()
        && string_pad_pad_type_key.is_none()
        && interpolator_duration_key.is_none()
        && range_mapper_min_input_key.is_none()
        && range_mapper_max_input_key.is_none()
        && range_mapper_min_output_key.is_none()
        && range_mapper_max_output_key.is_none()
        && number_to_list_view_model_id_key.is_none()
    {
        return Vec::new();
    }
    let default_instance = artboard_default_view_model_instance(file, artboard_index);
    let artboard_converters = file
        .artboard_data_binds(artboard_index)
        .into_iter()
        .filter_map(|data_bind| runtime_data_bind_graph_converter(file, data_bind.object))
        .collect::<Vec<_>>();
    file.objects
        .iter()
        .flatten()
        .filter(|object| object.type_name == "DataBindContext")
        .filter_map(|data_bind| {
            let data_bind_id = usize::try_from(data_bind.id).ok()?;
            if file.import_status(data_bind_id) != Some(nuxie_binary::RuntimeImportStatus::Imported)
            {
                return None;
            }
            if !data_bind_flags_apply_source_to_target(
                data_bind.uint_property("flags").unwrap_or(0),
            ) {
                return None;
            }
            let target = file.data_bind_target_for_object(data_bind)?;
            if !artboard_converters.iter().any(|converter| {
                runtime_data_bind_graph_converter_contains_global_id(converter, target.id)
            }) {
                return None;
            }
            let property_key = u16::try_from(data_bind.uint_property("propertyKey")?).ok()?;
            let target = match target.type_name {
                "DataConverterToString" if Some(property_key) == decimals_key => {
                    RuntimeArtboardConverterPropertyBindingTarget::ToStringDecimals {
                        global_id: target.id,
                    }
                }
                "DataConverterToString" if Some(property_key) == color_format_key => {
                    RuntimeArtboardConverterPropertyBindingTarget::ToStringColorFormat {
                        global_id: target.id,
                    }
                }
                "DataConverterStringTrim" if Some(property_key) == string_trim_trim_type_key => {
                    RuntimeArtboardConverterPropertyBindingTarget::StringTrimTrimType {
                        global_id: target.id,
                    }
                }
                "DataConverterStringPad" if Some(property_key) == string_pad_length_key => {
                    RuntimeArtboardConverterPropertyBindingTarget::StringPadLength {
                        global_id: target.id,
                    }
                }
                "DataConverterStringPad" if Some(property_key) == string_pad_text_key => {
                    RuntimeArtboardConverterPropertyBindingTarget::StringPadText {
                        global_id: target.id,
                    }
                }
                "DataConverterStringPad" if Some(property_key) == string_pad_pad_type_key => {
                    RuntimeArtboardConverterPropertyBindingTarget::StringPadPadType {
                        global_id: target.id,
                    }
                }
                "DataConverterInterpolator" if Some(property_key) == interpolator_duration_key => {
                    RuntimeArtboardConverterPropertyBindingTarget::InterpolatorDuration {
                        global_id: target.id,
                    }
                }
                "DataConverterRangeMapper" => {
                    let property = if Some(property_key) == range_mapper_min_input_key {
                        RuntimeDataBindGraphRangeMapperProperty::MinInput
                    } else if Some(property_key) == range_mapper_max_input_key {
                        RuntimeDataBindGraphRangeMapperProperty::MaxInput
                    } else if Some(property_key) == range_mapper_min_output_key {
                        RuntimeDataBindGraphRangeMapperProperty::MinOutput
                    } else if Some(property_key) == range_mapper_max_output_key {
                        RuntimeDataBindGraphRangeMapperProperty::MaxOutput
                    } else {
                        return None;
                    };
                    RuntimeArtboardConverterPropertyBindingTarget::RangeMapper {
                        global_id: target.id,
                        property,
                    }
                }
                "DataConverterNumberToList"
                    if Some(property_key) == number_to_list_view_model_id_key =>
                {
                    RuntimeArtboardConverterPropertyBindingTarget::NumberToListViewModelId {
                        global_id: target.id,
                    }
                }
                _ => return None,
            };
            let path = file.data_bind_context_source_path_ids_for_object(data_bind)?;
            let converter = runtime_data_bind_graph_converter(file, data_bind);
            if matches!(converter, Some(RuntimeDataBindGraphConverter::Unsupported)) {
                return None;
            }
            let default_value = default_instance
                .as_ref()
                .and_then(|default_instance| {
                    file.data_context_view_model_property_for_instance(
                        default_instance.object,
                        &path,
                    )
                    .and_then(|source| runtime_created_view_model_value_for_source(file, source))
                })
                .or_else(|| runtime_created_view_model_value_for_declared_path(file, &path))
                .unwrap_or_else(|| match target {
                    RuntimeArtboardConverterPropertyBindingTarget::ToStringDecimals { .. } => {
                        RuntimeDataBindGraphValue::Number(0.0)
                    }
                    RuntimeArtboardConverterPropertyBindingTarget::ToStringColorFormat {
                        ..
                    } => RuntimeDataBindGraphValue::String(Vec::new()),
                    RuntimeArtboardConverterPropertyBindingTarget::StringTrimTrimType {
                        ..
                    } => RuntimeDataBindGraphValue::Number(1.0),
                    RuntimeArtboardConverterPropertyBindingTarget::StringPadLength { .. } => {
                        RuntimeDataBindGraphValue::Number(1.0)
                    }
                    RuntimeArtboardConverterPropertyBindingTarget::StringPadText { .. } => {
                        RuntimeDataBindGraphValue::String(Vec::new())
                    }
                    RuntimeArtboardConverterPropertyBindingTarget::StringPadPadType { .. } => {
                        RuntimeDataBindGraphValue::Number(0.0)
                    }
                    RuntimeArtboardConverterPropertyBindingTarget::InterpolatorDuration {
                        ..
                    } => RuntimeDataBindGraphValue::Number(1.0),
                    RuntimeArtboardConverterPropertyBindingTarget::RangeMapper { .. } => {
                        RuntimeDataBindGraphValue::Number(1.0)
                    }
                    RuntimeArtboardConverterPropertyBindingTarget::NumberToListViewModelId {
                        ..
                    } => RuntimeDataBindGraphValue::Enum(u64::from(u32::MAX)),
                });
            if !runtime_artboard_converter_property_binding_target_accepts_value(
                target,
                converter
                    .as_ref()
                    .and_then(|converter| {
                        runtime_data_bind_graph_convert_value(converter, &default_value)
                    })
                    .as_ref()
                    .unwrap_or(&default_value),
            ) {
                return None;
            }

            Some(RuntimeArtboardConverterPropertyBindingInstance {
                target,
                path,
                converter_state: RuntimeDataBindGraphConverterState::for_converter(
                    converter.as_ref(),
                ),
                converter,
                default_value,
            })
        })
        .collect()
}

fn runtime_artboard_converter_property_binding_target_accepts_value(
    target: RuntimeArtboardConverterPropertyBindingTarget,
    value: &RuntimeDataBindGraphValue,
) -> bool {
    match target {
        RuntimeArtboardConverterPropertyBindingTarget::ToStringDecimals { .. }
        | RuntimeArtboardConverterPropertyBindingTarget::StringTrimTrimType { .. }
        | RuntimeArtboardConverterPropertyBindingTarget::StringPadLength { .. }
        | RuntimeArtboardConverterPropertyBindingTarget::StringPadPadType { .. }
        | RuntimeArtboardConverterPropertyBindingTarget::NumberToListViewModelId { .. } => {
            matches!(
                value,
                RuntimeDataBindGraphValue::Number(_) | RuntimeDataBindGraphValue::Enum(_)
            )
        }
        RuntimeArtboardConverterPropertyBindingTarget::InterpolatorDuration { .. }
        | RuntimeArtboardConverterPropertyBindingTarget::RangeMapper { .. } => {
            matches!(value, RuntimeDataBindGraphValue::Number(_))
        }
        RuntimeArtboardConverterPropertyBindingTarget::ToStringColorFormat { .. }
        | RuntimeArtboardConverterPropertyBindingTarget::StringPadText { .. } => {
            matches!(value, RuntimeDataBindGraphValue::String(_))
        }
    }
}

fn runtime_artboard_converter_property_binding_update(
    target: RuntimeArtboardConverterPropertyBindingTarget,
    value: RuntimeDataBindGraphValue,
) -> Option<RuntimeArtboardConverterPropertyBindingUpdate> {
    match (target, value) {
        (
            RuntimeArtboardConverterPropertyBindingTarget::ToStringDecimals { global_id },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(
            RuntimeArtboardConverterPropertyBindingUpdate::ToStringDecimals {
                global_id,
                value: value.max(0.0).round() as u64,
            },
        ),
        (
            RuntimeArtboardConverterPropertyBindingTarget::ToStringDecimals { global_id },
            RuntimeDataBindGraphValue::Enum(value),
        ) => Some(
            RuntimeArtboardConverterPropertyBindingUpdate::ToStringDecimals { global_id, value },
        ),
        (
            RuntimeArtboardConverterPropertyBindingTarget::ToStringColorFormat { global_id },
            RuntimeDataBindGraphValue::String(value),
        ) => Some(
            RuntimeArtboardConverterPropertyBindingUpdate::ToStringColorFormat { global_id, value },
        ),
        (
            RuntimeArtboardConverterPropertyBindingTarget::StringTrimTrimType { global_id },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(
            RuntimeArtboardConverterPropertyBindingUpdate::StringTrimTrimType {
                global_id,
                value: value.max(0.0).round() as u64,
            },
        ),
        (
            RuntimeArtboardConverterPropertyBindingTarget::StringTrimTrimType { global_id },
            RuntimeDataBindGraphValue::Enum(value),
        ) => Some(
            RuntimeArtboardConverterPropertyBindingUpdate::StringTrimTrimType { global_id, value },
        ),
        (
            RuntimeArtboardConverterPropertyBindingTarget::StringPadLength { global_id },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(
            RuntimeArtboardConverterPropertyBindingUpdate::StringPadLength {
                global_id,
                value: value.max(0.0).round() as u64,
            },
        ),
        (
            RuntimeArtboardConverterPropertyBindingTarget::StringPadLength { global_id },
            RuntimeDataBindGraphValue::Enum(value),
        ) => Some(
            RuntimeArtboardConverterPropertyBindingUpdate::StringPadLength { global_id, value },
        ),
        (
            RuntimeArtboardConverterPropertyBindingTarget::StringPadText { global_id },
            RuntimeDataBindGraphValue::String(value),
        ) => {
            Some(RuntimeArtboardConverterPropertyBindingUpdate::StringPadText { global_id, value })
        }
        (
            RuntimeArtboardConverterPropertyBindingTarget::StringPadPadType { global_id },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(
            RuntimeArtboardConverterPropertyBindingUpdate::StringPadPadType {
                global_id,
                value: value.max(0.0).round() as u64,
            },
        ),
        (
            RuntimeArtboardConverterPropertyBindingTarget::StringPadPadType { global_id },
            RuntimeDataBindGraphValue::Enum(value),
        ) => Some(
            RuntimeArtboardConverterPropertyBindingUpdate::StringPadPadType { global_id, value },
        ),
        (
            RuntimeArtboardConverterPropertyBindingTarget::InterpolatorDuration { global_id },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(
            RuntimeArtboardConverterPropertyBindingUpdate::InterpolatorDuration {
                global_id,
                value,
            },
        ),
        (
            RuntimeArtboardConverterPropertyBindingTarget::RangeMapper {
                global_id,
                property,
            },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(RuntimeArtboardConverterPropertyBindingUpdate::RangeMapper {
            global_id,
            property,
            value,
        }),
        (
            RuntimeArtboardConverterPropertyBindingTarget::NumberToListViewModelId { global_id },
            RuntimeDataBindGraphValue::Number(value),
        ) => Some(
            RuntimeArtboardConverterPropertyBindingUpdate::NumberToListViewModelId {
                global_id,
                value: value.max(0.0).round() as u64,
            },
        ),
        (
            RuntimeArtboardConverterPropertyBindingTarget::NumberToListViewModelId { global_id },
            RuntimeDataBindGraphValue::Enum(value),
        ) => Some(
            RuntimeArtboardConverterPropertyBindingUpdate::NumberToListViewModelId {
                global_id,
                value,
            },
        ),
        _ => None,
    }
}

pub(super) fn build_artboard_layout_computed_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardLayoutComputedBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };

    let bindings = file
        .artboard_data_binds(artboard_index)
        .into_iter()
        .filter_map(|data_bind| {
            if !data_bind_flags_apply_target_to_source(
                data_bind.object.uint_property("flags").unwrap_or(0),
            ) {
                return None;
            }
            data_bind.target?;
            let property_key =
                u16::try_from(data_bind.object.uint_property("propertyKey")?).ok()?;
            let property = layout_computed_property_for_key(property_key)?;
            Some(RuntimeArtboardLayoutComputedBindingInstance {
                target_local_id: data_bind.target_local_id?,
                property,
                path: shared_data_bind_path(
                    file.data_bind_context_source_path_ids_for_object(data_bind.object)?,
                ),
                owned_context_source: None,
                path_is_name_based: file
                    .data_bind_is_name_based_for_object(data_bind.object)
                    .unwrap_or(false),
                owned_context_source_path: None,
            })
        })
        .collect::<Vec<_>>();
    bindings
}

pub(super) fn build_artboard_solo_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardSoloBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let Some(active_component_id_key) = solo_active_component_id_property_key() else {
        return Vec::new();
    };
    let default_instance = artboard_default_view_model_instance(file, artboard_index);

    file.artboard_data_binds(artboard_index)
        .into_iter()
        .filter_map(|data_bind| {
            if !data_bind_flags_apply_source_to_target(
                data_bind.object.uint_property("flags").unwrap_or(0),
            ) {
                return None;
            }
            let target = data_bind.target?;
            if target.type_name != "Solo" {
                return None;
            }
            if u16::try_from(data_bind.object.uint_property("propertyKey")?).ok()?
                != active_component_id_key
            {
                return None;
            }
            let path = file.data_bind_context_source_path_ids_for_object(data_bind.object)?;
            let enum_value_names = runtime_enum_value_names_for_data_bind_path(
                file,
                default_instance.as_ref(),
                data_bind.object,
                &path,
            );
            Some(RuntimeArtboardSoloBindingInstance {
                target_local_id: data_bind.target_local_id?,
                path,
                enum_value_names,
            })
        })
        .collect()
}

pub(super) fn build_artboard_solo_source_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardSoloSourceBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let Some(active_component_id_key) = solo_active_component_id_property_key() else {
        return Vec::new();
    };
    let default_instance = artboard_default_view_model_instance(file, artboard_index);

    file.artboard_data_binds(artboard_index)
        .into_iter()
        .filter_map(|data_bind| {
            if !data_bind_flags_apply_target_to_source(
                data_bind.object.uint_property("flags").unwrap_or(0),
            ) {
                return None;
            }
            let target = data_bind.target?;
            if target.type_name != "Solo" {
                return None;
            }
            if u16::try_from(data_bind.object.uint_property("propertyKey")?).ok()?
                != active_component_id_key
            {
                return None;
            }
            let path = file.data_bind_context_source_path_ids_for_object(data_bind.object)?;
            let enum_value_names = runtime_enum_value_names_for_data_bind_path(
                file,
                default_instance.as_ref(),
                data_bind.object,
                &path,
            );
            if enum_value_names.is_empty() {
                return None;
            }
            Some(RuntimeArtboardSoloSourceBindingInstance {
                target_local_id: data_bind.target_local_id?,
                path: shared_data_bind_path(path),
                path_is_name_based: file
                    .data_bind_is_name_based_for_object(data_bind.object)
                    .unwrap_or(false),
                owned_context_source_path: None,
                enum_value_names,
                owned_context_source: None,
            })
        })
        .collect()
}

fn runtime_enum_value_names_for_data_bind_path(
    file: &RuntimeFile,
    default_instance: Option<&nuxie_binary::RuntimeViewModelInstanceReference<'_>>,
    data_bind: &RuntimeObject,
    path: &[u32],
) -> Vec<Vec<u8>> {
    default_instance
        .and_then(|default_instance| {
            file.data_context_view_model_property_for_instance(default_instance.object, path)
        })
        .and_then(|source| runtime_enum_value_names_for_source(file, source))
        .or_else(|| {
            if file
                .data_bind_is_name_based_for_object(data_bind)
                .unwrap_or(false)
            {
                return None;
            }
            runtime_enum_value_names_for_declared_path(file, path)
        })
        .unwrap_or_default()
}

fn runtime_enum_value_names_for_source(
    file: &RuntimeFile,
    source: &RuntimeObject,
) -> Option<Vec<Vec<u8>>> {
    let data_enum = file.data_enum_for_view_model_instance_enum_value_object(source)?;
    Some(
        data_enum
            .values
            .into_iter()
            .map(runtime_data_enum_value_name)
            .collect(),
    )
}

fn runtime_enum_value_names_for_declared_path(
    file: &RuntimeFile,
    path: &[u32],
) -> Option<Vec<Vec<u8>>> {
    let (view_model_index, property_path) = path.split_first()?;
    let mut view_model_index = usize::try_from(*view_model_index).ok()?;

    for (index, property_id) in property_path.iter().enumerate() {
        let view_model = file.view_model(view_model_index)?;
        let property_index = usize::try_from(*property_id).ok()?;
        let property = *view_model.properties.get(property_index)?;
        if index == property_path.len() - 1 {
            let data_enum = file.data_enum_for_view_model_property_object(property)?;
            return Some(
                data_enum
                    .values
                    .into_iter()
                    .map(runtime_data_enum_value_name)
                    .collect(),
            );
        }
        if property.type_name != "ViewModelPropertyViewModel" {
            return None;
        }
        view_model_index = usize::try_from(property.uint_property("viewModelReferenceId")?).ok()?;
    }

    None
}

fn runtime_data_enum_value_name(value: &RuntimeObject) -> Vec<u8> {
    let resolved_value = value.string_property_bytes("value").unwrap_or_default();
    if resolved_value.is_empty() {
        return value
            .string_property_bytes("key")
            .unwrap_or_default()
            .to_vec();
    }
    resolved_value.to_vec()
}

fn runtime_created_view_model_value_for_path(
    file: &RuntimeFile,
    default_instance: &RuntimeObject,
    path: &[u32],
) -> Option<RuntimeDataBindGraphValue> {
    let source = file.data_context_view_model_property_for_instance(default_instance, path)?;
    runtime_created_view_model_value_for_source(file, source)
}

fn artboard_default_view_model_instance(
    file: &RuntimeFile,
    artboard_index: usize,
) -> Option<nuxie_binary::RuntimeViewModelInstanceReference<'_>> {
    let artboard = file.artboard(artboard_index)?;
    let view_model_index = usize::try_from(artboard.uint_property("viewModelId")?).ok()?;
    file.view_model_default_instance(view_model_index)
}

fn runtime_created_view_model_value_for_source(
    file: &RuntimeFile,
    source: &RuntimeObject,
) -> Option<RuntimeDataBindGraphValue> {
    // The C++ golden runner binds File::createViewModelInstance(), not the
    // serialized default instance. Use generated ViewModelInstance* field
    // defaults while relying on the serialized instance only for path/type
    // resolution.
    match file.view_model_instance_value_data_type_for_object(source)? {
        RuntimeDataType::Number => Some(RuntimeDataBindGraphValue::Number(0.0)),
        RuntimeDataType::Boolean => Some(RuntimeDataBindGraphValue::Boolean(false)),
        RuntimeDataType::String => Some(RuntimeDataBindGraphValue::String(Vec::new())),
        RuntimeDataType::Color => Some(RuntimeDataBindGraphValue::Color(0xFF000000)),
        RuntimeDataType::EnumType => Some(RuntimeDataBindGraphValue::Enum(0)),
        RuntimeDataType::Trigger => Some(RuntimeDataBindGraphValue::Trigger(0)),
        RuntimeDataType::List => Some(RuntimeDataBindGraphValue::List { item_count: 0 }),
        RuntimeDataType::SymbolListIndex => Some(RuntimeDataBindGraphValue::SymbolListIndex(0)),
        RuntimeDataType::AssetImage | RuntimeDataType::AssetFont => {
            Some(RuntimeDataBindGraphValue::Asset(u64::from(u32::MAX)))
        }
        RuntimeDataType::Artboard => Some(RuntimeDataBindGraphValue::Artboard(u64::from(u32::MAX))),
        _ => None,
    }
}

fn runtime_created_view_model_value_for_declared_path(
    file: &RuntimeFile,
    path: &[u32],
) -> Option<RuntimeDataBindGraphValue> {
    let (view_model_index, property_path) = path.split_first()?;
    let mut view_model_index = usize::try_from(*view_model_index).ok()?;

    for (index, property_id) in property_path.iter().enumerate() {
        let view_model = file.view_model(view_model_index)?;
        let property_index = usize::try_from(*property_id).ok()?;
        let property = *view_model.properties.get(property_index)?;
        if index == property_path.len() - 1 {
            return runtime_created_view_model_value_for_declared_property(property);
        }
        if property.type_name != "ViewModelPropertyViewModel" {
            return None;
        }
        view_model_index = usize::try_from(property.uint_property("viewModelReferenceId")?).ok()?;
    }

    None
}

fn runtime_created_view_model_value_for_declared_property(
    property: &RuntimeObject,
) -> Option<RuntimeDataBindGraphValue> {
    match property.type_name {
        "ViewModelPropertyNumber" => Some(RuntimeDataBindGraphValue::Number(0.0)),
        "ViewModelPropertyBoolean" => Some(RuntimeDataBindGraphValue::Boolean(false)),
        "ViewModelPropertyString" => Some(RuntimeDataBindGraphValue::String(Vec::new())),
        "ViewModelPropertyColor" => Some(RuntimeDataBindGraphValue::Color(0xFF000000)),
        "ViewModelPropertyEnum" | "ViewModelPropertyEnumCustom" | "ViewModelPropertyEnumSystem" => {
            Some(RuntimeDataBindGraphValue::Enum(0))
        }
        "ViewModelPropertyTrigger" => Some(RuntimeDataBindGraphValue::Trigger(0)),
        "ViewModelPropertyList" => Some(RuntimeDataBindGraphValue::List { item_count: 0 }),
        "ViewModelPropertySymbolListIndex" => Some(RuntimeDataBindGraphValue::SymbolListIndex(0)),
        "ViewModelPropertyAssetImage" | "ViewModelPropertyAssetFont" => {
            Some(RuntimeDataBindGraphValue::Asset(u64::from(u32::MAX)))
        }
        "ViewModelPropertyArtboard" => {
            Some(RuntimeDataBindGraphValue::Artboard(u64::from(u32::MAX)))
        }
        _ => None,
    }
}

impl ArtboardInstance {
    pub fn set_scripted_data_converter_instance_for_global(
        &mut self,
        global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) -> bool {
        let handle = RuntimeScriptInstanceHandle::new(instance);
        self.scripted_data_converter_instances_by_global
            .insert(global_id, handle.clone());
        let mut attached = false;
        for state in self.artboard_shared_data_bind_converter_states.values_mut() {
            attached |= state.converter.attach_scripted_instance(global_id, &handle);
        }
        for binding in &mut self.artboard_property_bindings {
            if let Some(converter) = binding.converter.as_mut() {
                attached |= converter.attach_scripted_instance(global_id, &handle);
            }
        }
        for binding in &mut self.artboard_custom_property_bindings {
            if let Some(converter) = binding.converter.as_mut() {
                attached |= converter.attach_scripted_instance(global_id, &handle);
            }
        }
        for binding in &mut self.artboard_formula_token_bindings {
            if let Some(converter) = binding.converter.as_mut() {
                attached |= converter.attach_scripted_instance(global_id, &handle);
            }
        }
        for binding in &mut self.artboard_converter_property_bindings {
            if let Some(converter) = binding.converter.as_mut() {
                attached |= converter.attach_scripted_instance(global_id, &handle);
            }
        }
        for binding in &mut self.artboard_list_bindings {
            if let Some(converter) = binding.converter.as_mut() {
                attached |= converter.attach_scripted_instance(global_id, &handle);
            }
        }
        attached
    }

    fn enqueue_artboard_data_bind_targets_for_path(&mut self, path: &[u32]) {
        self.enqueue_artboard_data_bind_targets_for_path_with_suppressed_data_bind(path, None);
    }

    fn enqueue_artboard_data_bind_targets_for_path_with_suppressed_data_bind(
        &mut self,
        path: &[u32],
        suppressed_data_bind_index: Option<usize>,
    ) {
        let value = self.artboard_data_bind_values.get(path).cloned();
        let suppressed_property_index = suppressed_data_bind_index.and_then(|data_bind_index| {
            self.artboard_property_bindings
                .iter()
                .position(|binding| binding.data_bind_index == data_bind_index)
        });
        let enqueued = self
            .artboard_data_bind_target_queues
            .enqueue_path(path, suppressed_property_index);
        for index in &enqueued {
            let Some(data_bind_index) = self
                .artboard_property_bindings
                .get(*index)
                .map(|binding| binding.data_bind_index)
            else {
                continue;
            };
            if let Some(state) = self
                .artboard_shared_data_bind_converter_states
                .get_mut(&data_bind_index)
            {
                state.target_origin = false;
            }
        }
        if let Some(value) = value {
            for index in enqueued {
                if let Some(binding) = self.artboard_property_bindings.get_mut(index) {
                    if binding.snapshots_source_value {
                        binding.pending_value = Some(value.clone());
                    }
                }
            }
        }
    }

    fn enqueue_artboard_property_binding_target(&mut self, index: usize) {
        if !self
            .artboard_data_bind_target_queues
            .enqueue_property(index)
        {
            return;
        }
        let value = self
            .artboard_property_bindings
            .get(index)
            .and_then(|binding| self.artboard_data_bind_values.get(&binding.path))
            .cloned();
        if let (Some(binding), Some(value)) =
            (self.artboard_property_bindings.get_mut(index), value)
        {
            if binding.snapshots_source_value {
                binding.pending_value = Some(value);
            }
        }
    }

    fn enqueue_artboard_shared_converter_direction(&mut self, data_bind_index: usize) {
        let Some(target_origin) = self
            .artboard_shared_data_bind_converter_states
            .get(&data_bind_index)
            .map(|state| state.target_origin)
        else {
            return;
        };
        if target_origin {
            if let Some(index) = self
                .artboard_custom_property_bindings
                .iter()
                .position(|binding| binding.data_bind_index == data_bind_index)
            {
                self.artboard_data_bind_source_queues
                    .enqueue_custom_property(index);
            }
        } else if let Some(index) = self
            .artboard_property_bindings
            .iter()
            .position(|binding| binding.data_bind_index == data_bind_index)
        {
            self.enqueue_artboard_property_binding_target(index);
        }
    }

    pub(crate) fn notify_artboard_data_bind_target_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
    ) -> bool {
        if !self
            .artboard_data_bind_source_queues
            .has_target_properties()
        {
            return false;
        }
        let enqueued = self
            .artboard_data_bind_source_queues
            .enqueue_target_property(
                local_id,
                property_key,
                self.artboard_data_bind_suppressed_target_data_bind,
            );
        let did_enqueue = !enqueued.is_empty();
        for data_bind_index in enqueued {
            if let Some(state) = self
                .artboard_shared_data_bind_converter_states
                .get_mut(&data_bind_index)
            {
                state.target_origin = true;
            }
        }
        if did_enqueue {
            self.mark_artboard_data_bind_work_dirty();
        }
        did_enqueue
    }

    pub(crate) fn update_nested_artboard_data_binds_from_hosts(&mut self) -> bool {
        if !self.nested_artboard_tree_has_context_source_bindings() {
            return false;
        }
        let mut changed = false;
        let mut values = std::mem::take(&mut self.artboard_context_source_values_scratch);
        self.collect_nested_artboard_context_source_values(Mat2D::IDENTITY, &mut values);
        for source in values.drain(..) {
            changed |= self.set_artboard_data_bind_value_for_path(&source.path, source.value);
        }
        self.artboard_context_source_values_scratch = values;
        changed
    }

    fn nested_artboard_tree_has_context_source_bindings(&self) -> bool {
        let structure_epoch = self.nested_structure_epoch();
        if let (Some(epoch), Some((cached_epoch, cached_value))) =
            (structure_epoch, self.nested_context_source_tree_cache.get())
            && cached_epoch == epoch
        {
            return cached_value;
        }
        let has_bindings = self.nested_artboard_locals.iter().any(|host_local_id| {
            self.nested_artboards
                .get(host_local_id)
                .is_some_and(|nested| {
                    nested.child.has_artboard_context_source_bindings()
                        || nested
                            .child
                            .nested_artboard_tree_has_context_source_bindings()
                })
        }) || self.component_list_items.values().flatten().any(|item| {
            item.child.has_artboard_context_source_bindings()
                || item
                    .child
                    .nested_artboard_tree_has_context_source_bindings()
        });
        if let Some(epoch) = structure_epoch {
            self.nested_context_source_tree_cache
                .set(Some((epoch, has_bindings)));
        }
        has_bindings
    }

    fn collect_nested_artboard_context_source_values(
        &mut self,
        root_transform: Mat2D,
        values: &mut Vec<RuntimeArtboardContextSourceValue>,
    ) {
        for index in 0..self.nested_artboard_locals.len() {
            let host_local_id = self.nested_artboard_locals[index];
            let host_world = self
                .component(host_local_id)
                .map(|component| component.transform.world_transform)
                .unwrap_or(Mat2D::IDENTITY);
            let child_root_transform = root_transform.multiply(host_world);
            let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
                continue;
            };
            let child_has_direct_context_sources =
                nested.child.has_artboard_context_source_bindings();
            let child_has_nested_context_sources = !nested.child.nested_artboard_locals.is_empty();
            if !child_has_direct_context_sources && !child_has_nested_context_sources {
                continue;
            }
            let descendant_start = values.len();
            if child_has_nested_context_sources {
                nested
                    .child
                    .collect_nested_artboard_context_source_values(child_root_transform, values);
            }
            let descendant_end = values.len();
            for source in &values[descendant_start..descendant_end] {
                nested
                    .child
                    .set_artboard_data_bind_value_for_path_ref(&source.path, &source.value);
            }
            nested
                .child
                .advance_artboard_data_binds_with_root_transform(child_root_transform, 0.0);
            if child_has_direct_context_sources {
                nested.child.append_artboard_context_source_values(values);
            }
        }

        // `ArtboardComponentList` is also an ArtboardHost in C++. Row
        // artboards own their own main DataContext, so settle each row with
        // the host/root transform but do not publish its values into the
        // parent's context. Nested descendants inside the row still publish
        // into the row before its own bindings are polled.
        let row_root_transforms = self.runtime_component_list_child_root_transforms(root_transform);
        for (list_local_id, transforms) in row_root_transforms {
            let Some(items) = self.component_list_items.get_mut(&list_local_id) else {
                continue;
            };
            for (item, child_root_transform) in items.iter_mut().zip(transforms) {
                let row_values_start = values.len();
                item.child
                    .collect_nested_artboard_context_source_values(child_root_transform, values);
                let row_values_end = values.len();
                for source in &values[row_values_start..row_values_end] {
                    item.child
                        .set_artboard_data_bind_value_for_path_ref(&source.path, &source.value);
                }
                item.child
                    .advance_artboard_data_binds_with_root_transform(child_root_transform, 0.0);
                values.truncate(row_values_start);
            }
        }
    }

    fn has_artboard_context_source_bindings(&self) -> bool {
        !self.artboard_layout_computed_bindings.is_empty()
            || !self.artboard_custom_property_bindings.is_empty()
            || !self.artboard_solo_source_bindings.is_empty()
    }

    fn append_artboard_context_source_values(
        &self,
        values: &mut Vec<RuntimeArtboardContextSourceValue>,
    ) {
        values.reserve(
            self.artboard_layout_computed_bindings.len()
                + self.artboard_custom_property_bindings.len()
                + self.artboard_solo_source_bindings.len(),
        );
        for binding in &self.artboard_layout_computed_bindings {
            if let Some(value) = self
                .artboard_data_bind_values
                .get(binding.path.as_ref())
                .cloned()
            {
                values.push(RuntimeArtboardContextSourceValue {
                    path: binding.path.clone(),
                    value,
                });
            }
        }
        for binding in &self.artboard_custom_property_bindings {
            if let Some(value) = self
                .artboard_data_bind_values
                .get(binding.path.as_ref())
                .cloned()
            {
                values.push(RuntimeArtboardContextSourceValue {
                    path: binding.path.clone(),
                    value,
                });
            }
        }
        for binding in &self.artboard_solo_source_bindings {
            if let Some(value) = self
                .artboard_data_bind_values
                .get(binding.path.as_ref())
                .cloned()
            {
                values.push(RuntimeArtboardContextSourceValue {
                    path: binding.path.clone(),
                    value,
                });
            }
        }
    }

    pub fn bind_default_view_model_artboard_list_context(&mut self, file: &RuntimeFile) -> bool {
        let Some(artboard_index) = file
            .artboards()
            .into_iter()
            .position(|artboard| artboard.id == self.graph_global_id)
        else {
            return false;
        };
        let Some(default_instance) = artboard_default_view_model_instance(file, artboard_index)
        else {
            return false;
        };
        self.bind_artboard_data_context(file, default_instance.object)
    }

    pub fn bind_owned_view_model_artboard_context(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.artboard_owned_view_model_handle = None;
        self.artboard_owned_view_model_context =
            Some(RuntimeOwnedViewModelContext::from_main(context.clone()));
        self.bind_owned_view_model_artboard_context_snapshot(file, context)
    }

    /// Bind and retain one shared owned view-model graph.
    ///
    /// Mutations through any alias are refreshed before the next artboard
    /// data-bind pass, matching Rive's reference-counted data context.
    pub fn bind_owned_view_model_artboard_handle(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelHandle,
    ) -> bool {
        self.bind_owned_view_model_artboard_context_handle(
            file,
            &RuntimeOwnedViewModelContextHandle::root(file, context.clone()),
        )
    }

    pub fn bind_owned_view_model_artboard_context_handle(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelContextHandle,
    ) -> bool {
        self.artboard_owned_view_model_context = Some(
            RuntimeOwnedViewModelContext::from_main_handle(context.root_handle()),
        );
        self.artboard_owned_view_model_handle = Some(context.clone());
        let root = context.root_handle();
        let root = root.borrow();
        let context_chain = [context.scope_path()];
        self.bind_owned_view_model_artboard_context_chain(file, &root, &context_chain, true, true)
    }

    fn bind_owned_view_model_artboard_context_snapshot(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
    ) -> bool {
        let context_chain: [&[usize]; 1] = [&[]];
        self.bind_owned_view_model_artboard_context_chain(file, context, &context_chain, true, true)
    }

    /// Binds the same ordered composite context used by C++ `DataContext`:
    /// main first, followed by globals in canonical file-slot order.
    pub fn bind_owned_view_model_artboard_contexts(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelContext,
    ) -> bool {
        self.artboard_owned_view_model_handle = None;
        self.artboard_owned_view_model_context = Some(context.clone());
        let candidates = context
            .handles()
            .map(RuntimeOwnedViewModelBindingCandidate::root_handle)
            .collect::<Vec<_>>();
        self.bind_owned_view_model_artboard_context_candidates(file, &candidates, true, true)
    }

    pub(crate) fn bind_owned_view_model_artboard_context_candidates(
        &mut self,
        file: &RuntimeFile,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
        bind_self: bool,
        allow_full_context_bindings: bool,
    ) -> bool {
        if bind_self {
            self.artboard_owned_view_model_candidates = candidates.to_vec();
        }
        let rebind_self = self.retain_owned_view_model_context_candidates(candidates);
        if rebind_self {
            self.mark_artboard_data_bind_work_dirty();
            self.stateful_nested_view_model_contexts_dirty = true;
        }
        if bind_self && rebind_self {
            self.bind_owned_view_model_target_to_source_bindings(
                file,
                candidates,
                allow_full_context_bindings,
            );
        }
        let mut changed = if bind_self && rebind_self {
            let mut changed = self.refresh_artboard_converter_dependents(|converter| {
                runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_candidates(
                    converter, candidates,
                )
            });
            changed |= self.bind_owned_view_model_artboard_values_for_candidates(
                file,
                candidates,
                allow_full_context_bindings,
            );
            self.artboard_data_bind_source_queues
                .enqueue_numeric_push_sources();
            changed
        } else {
            false
        };

        for index in 0..self.nested_artboard_locals.len() {
            let host_local_id = self.nested_artboard_locals[index];
            let child_candidates = self.owned_view_model_context_candidates_for_nested_host(
                file,
                candidates,
                host_local_id,
                allow_full_context_bindings,
            );
            let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
                continue;
            };

            let mut nested_candidates = Vec::new();
            if let Some(stateful_context) = nested.stateful_view_model_context.clone() {
                nested_candidates.push(RuntimeOwnedViewModelBindingCandidate::root(
                    &stateful_context,
                ));
            }
            nested_candidates.extend(
                nested
                    .stateful_global_view_model_contexts
                    .values()
                    .map(RuntimeOwnedViewModelBindingCandidate::root),
            );
            // A nested composite has its local main/globals first and inherits
            // the complete parent context as fallback.
            nested_candidates.extend(child_candidates);

            if rebind_self {
                changed |= nested
                    .bind_owned_view_model_animation_context_candidates(file, &nested_candidates);
            }
            changed |= nested
                .child
                .bind_owned_view_model_artboard_context_candidates(
                    file,
                    &nested_candidates,
                    true,
                    allow_full_context_bindings,
                );
        }
        if bind_self && rebind_self {
            changed |= self.bind_owned_view_model_component_list_context_candidates(
                file,
                candidates,
                allow_full_context_bindings,
            );
        }
        changed
    }

    /// Rebinds a freshly mounted nested occurrence to the parent context that
    /// was active when its artboard asset changed.
    ///
    /// C++ `NestedArtboard::updateArtboard` mounts the replacement and then
    /// immediately calls `bindViewModelInstance` (or propagates its existing
    /// `DataContext`). The replacement must therefore observe the same local
    /// main/globals plus inherited-parent ordering as an initially mounted
    /// nested artboard, without waiting for the parent scene to be rebound.
    pub(crate) fn rebind_owned_view_model_context_after_nested_artboard_swap(
        &mut self,
        file: &RuntimeFile,
        host_local_id: usize,
    ) -> bool {
        if self.artboard_owned_view_model_candidates.is_empty() {
            return false;
        }
        let inherited_candidates = self.owned_view_model_context_candidates_for_nested_host(
            file,
            &self.artboard_owned_view_model_candidates,
            host_local_id,
            true,
        );
        let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
            return false;
        };

        let mut candidates = Vec::new();
        if let Some(context) = nested.stateful_view_model_context.clone() {
            candidates.push(RuntimeOwnedViewModelBindingCandidate::root(&context));
        }
        candidates.extend(
            nested
                .stateful_global_view_model_contexts
                .values()
                .map(RuntimeOwnedViewModelBindingCandidate::root),
        );
        candidates.extend(inherited_candidates);

        let mut changed =
            nested.bind_owned_view_model_animation_context_candidates(file, &candidates);
        changed |= nested
            .child
            .bind_owned_view_model_artboard_context_candidates(file, &candidates, true, true);
        changed
    }

    fn bind_owned_view_model_component_list_context_candidates(
        &mut self,
        file: &RuntimeFile,
        parent_candidates: &[RuntimeOwnedViewModelBindingCandidate],
        allow_full_context_bindings: bool,
    ) -> bool {
        let mut changed = false;
        for items in self.component_list_items.values_mut() {
            for item in items {
                // C++ gives each row a child DataContext whose main is the row
                // instance and whose parent is the complete owning artboard
                // context. Resolution is therefore row first, followed by the
                // parent's main and globals in their existing order.
                let mut child_candidates = Vec::with_capacity(parent_candidates.len() + 1);
                child_candidates.push(RuntimeOwnedViewModelBindingCandidate::root_handle(
                    &item.context,
                ));
                child_candidates.extend(parent_candidates.iter().cloned());
                changed |= item
                    .child
                    .bind_owned_view_model_artboard_context_candidates(
                        file,
                        &child_candidates,
                        true,
                        allow_full_context_bindings,
                    );
                if let Some(state_machine) = item.state_machine.as_mut() {
                    let contexts = child_candidates
                        .iter()
                        .map(|candidate| {
                            (candidate.context.clone(), candidate.context_chain.clone())
                        })
                        .collect::<Vec<_>>();
                    if state_machine.bind_owned_view_model_context_chains(file, &contexts) {
                        changed = true;
                        changed |= state_machine.advance_data_context();
                    }
                }
            }
        }
        changed
    }

    pub fn bind_owned_view_model_nested_artboard_contexts(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
    ) -> bool {
        let context_chain: [&[usize]; 1] = [&[]];
        self.bind_owned_view_model_artboard_context_chain(
            file,
            context,
            &context_chain,
            false,
            false,
        )
    }

    fn bind_owned_view_model_artboard_context_chain(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
        bind_self: bool,
        allow_full_context_bindings: bool,
    ) -> bool {
        if bind_self {
            self.artboard_owned_view_model_candidates =
                vec![RuntimeOwnedViewModelBindingCandidate {
                    context: RuntimeOwnedViewModelHandle::new(context.clone()),
                    context_chain: context_chain.iter().map(|path| path.to_vec()).collect(),
                }];
        }
        let rebind_self = self.retain_owned_view_model_context_chain(context, context_chain);
        if rebind_self {
            self.mark_artboard_data_bind_work_dirty();
            self.stateful_nested_view_model_contexts_dirty = true;
        }
        if bind_self && rebind_self {
            let candidates = self.artboard_owned_view_model_candidates.clone();
            self.bind_owned_view_model_target_to_source_bindings(
                file,
                &candidates,
                allow_full_context_bindings,
            );
        }
        // A retained graph mutation is observable work even when the changed
        // value does not alter this artboard's immediate scalar projection.
        // In particular, list-row aliases share the root mutation clock and
        // must invalidate every parent exactly once.
        let mut changed = bind_self && rebind_self;
        if bind_self && rebind_self {
            changed |= self.refresh_artboard_converter_dependents(|converter| {
                runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_context(
                    converter, context,
                )
            });
            changed |= self.bind_owned_view_model_artboard_values(
                file,
                context,
                context_chain,
                allow_full_context_bindings,
            );
            self.artboard_data_bind_source_queues
                .enqueue_numeric_push_sources();
        }
        for index in 0..self.nested_artboard_locals.len() {
            let host_local_id = self.nested_artboard_locals[index];
            let child_context = self.owned_view_model_context_chain_for_nested_host(
                file,
                context,
                context_chain,
                host_local_id,
                allow_full_context_bindings,
            );
            let child_context_chain_storage =
                RuntimeOwnedViewModelContextChainStorage::with_child_context(
                    context_chain,
                    child_context
                        .as_ref()
                        .map(RuntimeOwnedViewModelContextPathStorage::as_slice),
                );
            let child_context_chain = child_context_chain_storage.as_slice();
            let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
                continue;
            };
            if let Some(stateful_context) = nested.stateful_view_model_context.clone() {
                let stateful_context_chain: [&[usize]; 1] = [&[]];
                if rebind_self {
                    changed |= nested.bind_owned_view_model_animation_contexts(
                        file,
                        &stateful_context,
                        &stateful_context_chain,
                    );
                }
                changed |= nested.child.bind_owned_view_model_artboard_context_chain(
                    file,
                    &stateful_context,
                    &stateful_context_chain,
                    true,
                    allow_full_context_bindings,
                );
                continue;
            }
            if rebind_self {
                changed |= nested.bind_owned_view_model_animation_contexts(
                    file,
                    context,
                    child_context_chain,
                );
            }
            changed |= nested.child.bind_owned_view_model_artboard_context_chain(
                file,
                context,
                child_context_chain,
                true,
                allow_full_context_bindings,
            );
        }
        if bind_self && rebind_self {
            let parent_candidates = [RuntimeOwnedViewModelBindingCandidate {
                context: RuntimeOwnedViewModelHandle::new(context.clone()),
                context_chain: context_chain.iter().map(|path| path.to_vec()).collect(),
            }];
            changed |= self.bind_owned_view_model_component_list_context_candidates(
                file,
                &parent_candidates,
                allow_full_context_bindings,
            );
        }
        changed
    }

    fn retain_owned_view_model_context_chain(
        &mut self,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> bool {
        if self
            .artboard_owned_context_key
            .as_ref()
            .is_some_and(|key| key.matches_context_chain(context, context_chain))
        {
            return false;
        }
        let replacing_owned_context = self.artboard_owned_context_key.is_some();
        self.artboard_owned_context_key = Some(RuntimeArtboardOwnedContextKey::from_context_chain(
            context,
            context_chain,
        ));
        for binding in &mut self.artboard_property_bindings {
            binding.owned_context_source_path = None;
        }
        for binding in &mut self.artboard_image_asset_bindings {
            binding.owned_context_source_path = None;
        }
        for binding in &mut self.artboard_custom_property_bindings {
            binding.owned_context_source_path = None;
            binding.owned_context_source = None;
        }
        for binding in &mut self.artboard_layout_computed_bindings {
            binding.owned_context_source = None;
        }
        for binding in &mut self.artboard_numeric_source_bindings {
            binding.owned_context_source = None;
        }
        for binding in &mut self.artboard_solo_source_bindings {
            binding.owned_context_source = None;
        }
        for binding in &mut self.artboard_layout_computed_bindings {
            binding.owned_context_source_path = None;
        }
        for binding in &mut self.artboard_numeric_source_bindings {
            binding.owned_context_source_path = None;
        }
        for binding in &mut self.artboard_solo_source_bindings {
            binding.owned_context_source_path = None;
        }
        for binding in &mut self.artboard_nested_host_bindings {
            binding.owned_context_source_path = None;
            if replacing_owned_context {
                binding.artboard_value_applied = false;
            }
        }
        for binding in &mut self.artboard_text_list_bindings {
            binding.source = None;
        }
        true
    }

    fn retain_owned_view_model_context_candidates(
        &mut self,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
    ) -> bool {
        if self
            .artboard_owned_context_key
            .as_ref()
            .is_some_and(|key| key.matches_candidates(candidates))
        {
            return false;
        }
        let replacing_owned_context = self.artboard_owned_context_key.is_some();
        self.artboard_owned_context_key =
            Some(RuntimeArtboardOwnedContextKey::from_candidates(candidates));
        for binding in &mut self.artboard_property_bindings {
            binding.owned_context_source_path = None;
        }
        for binding in &mut self.artboard_image_asset_bindings {
            binding.owned_context_source_path = None;
        }
        for binding in &mut self.artboard_custom_property_bindings {
            binding.owned_context_source_path = None;
            binding.owned_context_source = None;
        }
        for binding in &mut self.artboard_layout_computed_bindings {
            binding.owned_context_source = None;
        }
        for binding in &mut self.artboard_numeric_source_bindings {
            binding.owned_context_source = None;
        }
        for binding in &mut self.artboard_solo_source_bindings {
            binding.owned_context_source = None;
        }
        for binding in &mut self.artboard_nested_host_bindings {
            binding.owned_context_source_path = None;
            if replacing_owned_context {
                binding.artboard_value_applied = false;
            }
        }
        for binding in &mut self.artboard_text_list_bindings {
            binding.source = None;
        }
        true
    }

    fn bind_owned_view_model_target_to_source_bindings(
        &mut self,
        file: &RuntimeFile,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
        scripting_manifest: bool,
    ) {
        for binding in &mut self.artboard_custom_property_bindings {
            binding.owned_context_source = runtime_owned_view_model_binding_source_for_candidates(
                file,
                candidates,
                binding.path.as_ref(),
                binding.path_is_name_based,
                scripting_manifest,
            );
        }
        for binding in &mut self.artboard_layout_computed_bindings {
            binding.owned_context_source = runtime_owned_view_model_binding_source_for_candidates(
                file,
                candidates,
                binding.path.as_ref(),
                false,
                scripting_manifest,
            );
        }
        for binding in &mut self.artboard_numeric_source_bindings {
            binding.owned_context_source = runtime_owned_view_model_binding_source_for_candidates(
                file,
                candidates,
                &binding.path,
                false,
                scripting_manifest,
            );
        }
        for binding in &mut self.artboard_solo_source_bindings {
            binding.owned_context_source = runtime_owned_view_model_binding_source_for_candidates(
                file,
                candidates,
                binding.path.as_ref(),
                false,
                scripting_manifest,
            );
        }
    }

    fn bind_owned_view_model_artboard_values(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
        allow_full_context_bindings: bool,
    ) -> bool {
        let mut changed = false;

        for index in 0..self.artboard_property_bindings.len() {
            let update = {
                let binding = &mut self.artboard_property_bindings[index];
                runtime_owned_view_model_binding_value_for_retained_context_chain(
                    file,
                    context,
                    context_chain,
                    &binding.path,
                    binding.path_is_name_based,
                    allow_full_context_bindings,
                    &mut binding.owned_context_source_path,
                )
                .or_else(|| {
                    runtime_owned_view_model_missing_binding_value_for_context_chain(
                        context_chain,
                        binding,
                    )
                })
            };
            if let Some(value) = update {
                let path = self.artboard_property_bindings[index].path.as_slice();
                if self.artboard_data_bind_values.get(path) == Some(&value) {
                    continue;
                }
                if self.artboard_property_bindings[index].snapshots_source_value {
                    self.artboard_property_bindings[index].pending_value = Some(value.clone());
                }
                let path = self.artboard_property_bindings[index].path.clone();
                changed |= self.set_artboard_data_bind_value_for_path(&path, value);
            }
        }
        if allow_full_context_bindings {
            changed |= self.bind_owned_name_based_color_values(file, context, context_chain);
        }

        for index in 0..self.artboard_image_asset_bindings.len() {
            let (update, font_update) = {
                let binding = &mut self.artboard_image_asset_bindings[index];
                let update = runtime_owned_view_model_binding_value_for_retained_context_chain(
                    file,
                    context,
                    context_chain,
                    &binding.path,
                    binding.path_is_name_based,
                    allow_full_context_bindings,
                    &mut binding.owned_context_source_path,
                );
                let font_update = binding.target.is_font().then(|| {
                    runtime_owned_view_model_font_value_for_retained_context_chain(
                        file,
                        context,
                        context_chain,
                        &binding.path,
                        binding.path_is_name_based,
                        allow_full_context_bindings,
                        &mut binding.owned_context_source_path,
                    )
                });
                (update, font_update.flatten())
            };
            if let Some(value) = update {
                let font_changed = font_update.as_ref().is_some_and(|font_update| {
                    self.artboard_image_asset_bindings[index]
                        .font_value
                        .as_ref()
                        .is_none_or(|current| {
                            !runtime_font_asset_values_equal(current, font_update)
                        })
                });
                if let Some(font_update) = font_update {
                    self.artboard_image_asset_bindings[index].font_value = Some(font_update);
                }
                let path = self.artboard_image_asset_bindings[index].path.as_slice();
                let value_changed = self.artboard_data_bind_values.get(path) != Some(&value);
                if !value_changed && !font_changed {
                    continue;
                }
                let path = self.artboard_image_asset_bindings[index].path.clone();
                if value_changed {
                    changed |= self.set_artboard_data_bind_value_for_path(&path, value);
                } else {
                    self.artboard_data_bind_target_queues
                        .enqueue_image_asset(index);
                    changed = true;
                }
            }
        }

        for index in 0..self.artboard_custom_property_bindings.len() {
            let update = {
                let binding = &mut self.artboard_custom_property_bindings[index];
                runtime_owned_view_model_binding_value_for_retained_context_chain(
                    file,
                    context,
                    context_chain,
                    binding.path.as_ref(),
                    binding.path_is_name_based,
                    allow_full_context_bindings,
                    &mut binding.owned_context_source_path,
                )
            };
            if let Some(value) = update {
                let path = self.artboard_custom_property_bindings[index].path.as_ref();
                if self.artboard_data_bind_values.get(path) == Some(&value) {
                    continue;
                }
                let path = self.artboard_custom_property_bindings[index].path.clone();
                changed |= self.set_artboard_data_bind_value_for_path(path.as_ref(), value);
            }
        }

        for index in 0..self.artboard_formula_token_bindings.len() {
            if !self.artboard_formula_token_bindings[index].artboard_converter_reachable {
                continue;
            }
            let update = {
                let binding = &self.artboard_formula_token_bindings[index];
                context_chain.iter().find_map(|context_path| {
                    binding
                        .default_value
                        .resolve_from_owned_view_model_context_path(
                            file,
                            context,
                            context_path,
                            &binding.path,
                        )
                })
            };
            if let Some(value) = update {
                let path = self.artboard_formula_token_bindings[index].path.clone();
                changed |= self.set_artboard_data_bind_value_for_path(&path, value);
            }
        }

        for index in 0..self.artboard_converter_property_bindings.len() {
            let update = {
                let binding = &self.artboard_converter_property_bindings[index];
                context_chain.iter().find_map(|context_path| {
                    binding
                        .default_value
                        .resolve_from_owned_view_model_context_path(
                            file,
                            context,
                            context_path,
                            &binding.path,
                        )
                })
            };
            if let Some(value) = update {
                let path = self.artboard_converter_property_bindings[index]
                    .path
                    .clone();
                changed |= self.set_artboard_data_bind_value_for_path(&path, value);
            }
        }

        for index in 0..self.artboard_nested_host_bindings.len() {
            let path = self.artboard_nested_host_bindings[index].path.clone();
            let update = {
                let binding = &mut self.artboard_nested_host_bindings[index];
                runtime_owned_view_model_binding_value_for_retained_context_chain(
                    file,
                    context,
                    context_chain,
                    &binding.path,
                    binding.path_is_name_based,
                    allow_full_context_bindings,
                    &mut binding.owned_context_source_path,
                )
            };
            if let Some(value) = update
                && self.artboard_data_bind_values.get(&path) != Some(&value)
            {
                changed |= self.set_artboard_data_bind_value_for_path(&path, value);
            }
        }
        let mut component_list_updates = Vec::new();
        for binding in &mut self.artboard_list_bindings {
            let source = context_chain.iter().find_map(|context_path| {
                let property_path = context
                    .property_path_for_context_source_path_with_manifest_mode(
                        file,
                        context_path,
                        &binding.path,
                        false,
                        allow_full_context_bindings,
                    )?;
                runtime_owned_view_model_list_source_for_property_path(context, &property_path)
            });
            if let Some(source) = source {
                component_list_updates.push(binding.apply_resolved_source(file, source));
            }
        }
        for update in component_list_updates {
            changed |= update.binding_changed;
            if let Some(source) = update.source {
                self.component_list_sources
                    .insert(update.target_local_id, source);
            } else {
                self.component_list_sources.remove(&update.target_local_id);
            }
            if let Some(items) = update.items {
                changed |= self.sync_component_list_items(file, update.target_local_id, items);
            }
        }
        let mut text_lists_changed = false;
        for binding in &mut self.artboard_text_list_bindings {
            let source = context_chain.iter().find_map(|context_path| {
                let property_path = context
                    .property_path_for_context_source_path_with_manifest_mode(
                        file,
                        context_path,
                        &binding.path,
                        binding.path_is_name_based,
                        allow_full_context_bindings,
                    )?;
                context.list_handle_by_property_path(&property_path)
            });
            text_lists_changed |= source.is_some();
            binding.source = source;
        }
        if text_lists_changed {
            self.mark_path_changed();
            self.mark_prepared_changed();
            self.mark_layout_changed();
            changed = true;
        }
        changed
    }

    fn bind_owned_view_model_artboard_values_for_candidates(
        &mut self,
        file: &RuntimeFile,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
        allow_full_context_bindings: bool,
    ) -> bool {
        let mut changed = false;

        for index in 0..self.artboard_property_bindings.len() {
            let update = {
                let binding = &self.artboard_property_bindings[index];
                runtime_owned_view_model_binding_value_for_candidates(
                    file,
                    candidates,
                    &binding.path,
                    binding.path_is_name_based,
                    allow_full_context_bindings,
                )
                .or_else(|| {
                    candidates.iter().find_map(|candidate| {
                        let context_chain = candidate.context_chain();
                        runtime_owned_view_model_missing_binding_value_for_context_chain(
                            &context_chain,
                            binding,
                        )
                    })
                })
            };
            if let Some(value) = update {
                let path = self.artboard_property_bindings[index].path.as_slice();
                if self.artboard_data_bind_values.get(path) == Some(&value) {
                    continue;
                }
                if self.artboard_property_bindings[index].snapshots_source_value {
                    self.artboard_property_bindings[index].pending_value = Some(value.clone());
                }
                let path = self.artboard_property_bindings[index].path.clone();
                changed |= self.set_artboard_data_bind_value_for_path(&path, value);
            }
        }
        if allow_full_context_bindings {
            changed |= self.bind_owned_name_based_color_values_for_candidates(file, candidates);
        }

        for index in 0..self.artboard_image_asset_bindings.len() {
            let (update, font_update) = {
                let binding = &self.artboard_image_asset_bindings[index];
                let update = runtime_owned_view_model_binding_value_for_candidates(
                    file,
                    candidates,
                    &binding.path,
                    binding.path_is_name_based,
                    allow_full_context_bindings,
                );
                let font_update = binding.target.is_font().then(|| {
                    runtime_owned_view_model_font_value_for_candidates(
                        file,
                        candidates,
                        &binding.path,
                        binding.path_is_name_based,
                        allow_full_context_bindings,
                    )
                });
                (update, font_update.flatten())
            };
            if let Some(value) = update {
                let font_changed = font_update.as_ref().is_some_and(|font_update| {
                    self.artboard_image_asset_bindings[index]
                        .font_value
                        .as_ref()
                        .is_none_or(|current| {
                            !runtime_font_asset_values_equal(current, font_update)
                        })
                });
                if let Some(font_update) = font_update {
                    self.artboard_image_asset_bindings[index].font_value = Some(font_update);
                }
                let path = self.artboard_image_asset_bindings[index].path.as_slice();
                let value_changed = self.artboard_data_bind_values.get(path) != Some(&value);
                if !value_changed && !font_changed {
                    continue;
                }
                let path = self.artboard_image_asset_bindings[index].path.clone();
                if value_changed {
                    changed |= self.set_artboard_data_bind_value_for_path(&path, value);
                } else {
                    self.artboard_data_bind_target_queues
                        .enqueue_image_asset(index);
                    changed = true;
                }
            }
        }

        for index in 0..self.artboard_custom_property_bindings.len() {
            let update = {
                let binding = &self.artboard_custom_property_bindings[index];
                runtime_owned_view_model_binding_value_for_candidates(
                    file,
                    candidates,
                    binding.path.as_ref(),
                    binding.path_is_name_based,
                    allow_full_context_bindings,
                )
            };
            if let Some(value) = update {
                let path = self.artboard_custom_property_bindings[index].path.as_ref();
                if self.artboard_data_bind_values.get(path) == Some(&value) {
                    continue;
                }
                let path = self.artboard_custom_property_bindings[index].path.clone();
                changed |= self.set_artboard_data_bind_value_for_path(path.as_ref(), value);
            }
        }

        for index in 0..self.artboard_formula_token_bindings.len() {
            if !self.artboard_formula_token_bindings[index].artboard_converter_reachable {
                continue;
            }
            let binding = &self.artboard_formula_token_bindings[index];
            let update = runtime_owned_view_model_value_for_candidates(
                file,
                candidates,
                &binding.path,
                &binding.default_value,
            );
            if let Some(value) = update {
                let path = binding.path.clone();
                changed |= self.set_artboard_data_bind_value_for_path(&path, value);
            }
        }

        for index in 0..self.artboard_converter_property_bindings.len() {
            let binding = &self.artboard_converter_property_bindings[index];
            let update = runtime_owned_view_model_value_for_candidates(
                file,
                candidates,
                &binding.path,
                &binding.default_value,
            );
            if let Some(value) = update {
                let path = binding.path.clone();
                changed |= self.set_artboard_data_bind_value_for_path(&path, value);
            }
        }

        for index in 0..self.artboard_nested_host_bindings.len() {
            let path = self.artboard_nested_host_bindings[index].path.clone();
            let binding = &self.artboard_nested_host_bindings[index];
            let update = runtime_owned_view_model_binding_value_for_candidates(
                file,
                candidates,
                &binding.path,
                binding.path_is_name_based,
                allow_full_context_bindings,
            );
            if let Some(value) = update
                && self.artboard_data_bind_values.get(&path) != Some(&value)
            {
                changed |= self.set_artboard_data_bind_value_for_path(&path, value);
            }
        }

        let mut component_list_updates = Vec::new();
        for binding in &mut self.artboard_list_bindings {
            let source = candidates.iter().find_map(|candidate| {
                let context = candidate.context.borrow();
                candidate.context_chain.iter().find_map(|context_path| {
                    let property_path = context
                        .property_path_for_context_source_path_with_manifest_mode(
                            file,
                            context_path,
                            &binding.path,
                            false,
                            allow_full_context_bindings,
                        )?;
                    runtime_owned_view_model_list_source_for_property_path(&context, &property_path)
                })
            });
            if let Some(source) = source {
                component_list_updates.push(binding.apply_resolved_source(file, source));
            }
        }
        for update in component_list_updates {
            changed |= update.binding_changed;
            if let Some(source) = update.source {
                self.component_list_sources
                    .insert(update.target_local_id, source);
            } else {
                self.component_list_sources.remove(&update.target_local_id);
            }
            if let Some(items) = update.items {
                changed |= self.sync_component_list_items(file, update.target_local_id, items);
            }
        }

        let mut text_lists_changed = false;
        for binding in &mut self.artboard_text_list_bindings {
            let source = candidates.iter().find_map(|candidate| {
                let context = candidate.context.borrow();
                candidate.context_chain.iter().find_map(|context_path| {
                    let property_path = context
                        .property_path_for_context_source_path_with_manifest_mode(
                            file,
                            context_path,
                            &binding.path,
                            binding.path_is_name_based,
                            allow_full_context_bindings,
                        )?;
                    context.list_handle_by_property_path(&property_path)
                })
            });
            text_lists_changed |= source.is_some();
            binding.source = source;
        }
        if text_lists_changed {
            self.mark_path_changed();
            self.mark_prepared_changed();
            self.mark_layout_changed();
            changed = true;
        }
        changed
    }

    fn bind_owned_name_based_color_values_for_candidates(
        &mut self,
        file: &RuntimeFile,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
    ) -> bool {
        let Some(artboard_index) = file
            .artboards()
            .into_iter()
            .position(|artboard| artboard.id == self.graph_global_id)
        else {
            return false;
        };
        let Some(color_key) = solid_color_value_property_key() else {
            return false;
        };
        let updates = file
            .artboard_data_binds(artboard_index)
            .into_iter()
            .filter_map(|data_bind| {
                if !data_bind_flags_apply_source_to_target(
                    data_bind.object.uint_property("flags").unwrap_or(0),
                ) || !file
                    .data_bind_is_name_based_for_object(data_bind.object)
                    .unwrap_or(false)
                    || data_bind.target?.type_name != "SolidColor"
                    || u16::try_from(data_bind.object.uint_property("propertyKey")?).ok()?
                        != color_key
                {
                    return None;
                }
                let path = file.data_bind_context_source_path_ids_for_object(data_bind.object)?;
                let value = candidates.iter().find_map(|candidate| {
                    let context = candidate.context.borrow();
                    candidate.context_chain.iter().find_map(|context_path| {
                        let property_path = context
                            .property_path_for_context_source_path_with_manifest_mode(
                                file,
                                context_path,
                                &path,
                                true,
                                true,
                            )?;
                        context.color_value_by_property_path(&property_path)
                    })
                })?;
                Some((data_bind.target_local_id?, value))
            })
            .collect::<Vec<_>>();
        updates.into_iter().fold(false, |changed, (target, value)| {
            self.set_color_property(target, color_key, value) || changed
        })
    }

    fn bind_owned_name_based_color_values(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> bool {
        let Some(artboard_index) = file
            .artboards()
            .into_iter()
            .position(|artboard| artboard.id == self.graph_global_id)
        else {
            return false;
        };
        let Some(color_key) = solid_color_value_property_key() else {
            return false;
        };
        let updates = file
            .artboard_data_binds(artboard_index)
            .into_iter()
            .filter_map(|data_bind| {
                if !data_bind_flags_apply_source_to_target(
                    data_bind.object.uint_property("flags").unwrap_or(0),
                ) || !file
                    .data_bind_is_name_based_for_object(data_bind.object)
                    .unwrap_or(false)
                    || data_bind.target?.type_name != "SolidColor"
                    || u16::try_from(data_bind.object.uint_property("propertyKey")?).ok()?
                        != color_key
                {
                    return None;
                }
                let path = file.data_bind_context_source_path_ids_for_object(data_bind.object)?;
                let value = context_chain.iter().find_map(|context_path| {
                    let property_path = context
                        .property_path_for_context_source_path_with_manifest_mode(
                            file,
                            context_path,
                            &path,
                            true,
                            true,
                        )?;
                    context.color_value_by_property_path(&property_path)
                })?;
                Some((data_bind.target_local_id?, value))
            })
            .collect::<Vec<_>>();
        updates.into_iter().fold(false, |changed, (target, value)| {
            self.set_color_property(target, color_key, value) || changed
        })
    }

    fn owned_view_model_context_chain_for_nested_host<'a>(
        &self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &'a [&'a [usize]],
        host_local_id: usize,
        scripting_manifest: bool,
    ) -> Option<RuntimeOwnedViewModelContextPathStorage<'a>> {
        let nested = self.nested_artboards.get(&host_local_id)?;
        let path = nested.data_bind_path_ids.as_deref()?;
        if nested.data_bind_path_is_relative {
            return context_chain.iter().find_map(|context_path| {
                let property_path = context
                    .property_path_for_context_source_path_with_manifest_mode(
                        file,
                        context_path,
                        path,
                        true,
                        scripting_manifest,
                    )?;
                context.view_model_index_by_property_path(&property_path)?;
                Some(RuntimeOwnedViewModelContextPathStorage::Heap(property_path))
            });
        }
        runtime_owned_view_model_context_path_for_context_chain(context, context_chain, path)
    }

    fn owned_view_model_context_candidates_for_nested_host(
        &self,
        file: &RuntimeFile,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
        host_local_id: usize,
        scripting_manifest: bool,
    ) -> Vec<RuntimeOwnedViewModelBindingCandidate> {
        candidates
            .iter()
            .map(|candidate| {
                let context = candidate.context.borrow();
                let context_chain = candidate.context_chain();
                let child_context = self.owned_view_model_context_chain_for_nested_host(
                    file,
                    &context,
                    &context_chain,
                    host_local_id,
                    scripting_manifest,
                );
                let mut child_context_chain = Vec::with_capacity(
                    candidate.context_chain.len() + usize::from(child_context.is_some()),
                );
                if let Some(child_context) = child_context {
                    child_context_chain.push(child_context.as_slice().to_vec());
                }
                child_context_chain.extend(candidate.context_chain.iter().cloned());
                RuntimeOwnedViewModelBindingCandidate {
                    context: candidate.context.clone(),
                    context_chain: child_context_chain,
                }
            })
            .collect()
    }

    pub(crate) fn clear_default_text_property_context(&mut self) -> bool {
        let Some(text_property_key) =
            runtime_data_bind_property_key_for_name("TextValueRun", "text")
        else {
            return false;
        };
        let mut changed = false;
        let paths = self
            .artboard_property_bindings
            .iter()
            .filter(|binding| binding.property_key == text_property_key)
            .map(|binding| binding.path.clone())
            .collect::<Vec<_>>();
        for path in paths {
            if self.artboard_data_bind_values.remove(&path).is_some() {
                self.reset_artboard_property_formula_random_state_for_path(&path);
                self.enqueue_artboard_data_bind_targets_for_path(&path);
                changed = true;
            }
        }
        changed
    }

    fn bind_artboard_data_context(
        &mut self,
        file: &RuntimeFile,
        view_model_instance: &RuntimeObject,
    ) -> bool {
        // This overwrites the same value paths the owned-context bind wrote;
        // drop the retained owned-context key so a later owned rebind is not
        // skipped against stale defaults.
        self.artboard_owned_context_key = None;
        self.artboard_owned_view_model_context = None;
        self.artboard_owned_view_model_candidates.clear();
        self.artboard_owned_view_model_handle = None;
        let mut changed = false;
        let paths = self
            .artboard_property_bindings
            .iter()
            .map(|binding| binding.path.clone())
            .chain(
                self.artboard_image_asset_bindings
                    .iter()
                    .map(|binding| binding.path.clone()),
            )
            .chain(
                self.artboard_custom_property_bindings
                    .iter()
                    .map(|binding| binding.path.as_ref().to_vec()),
            )
            .chain(
                self.artboard_solo_bindings
                    .iter()
                    .map(|binding| binding.path.clone()),
            )
            .chain(
                self.artboard_solo_source_bindings
                    .iter()
                    .map(|binding| binding.path.as_ref().to_vec()),
            )
            .chain(
                self.artboard_nested_host_bindings
                    .iter()
                    .map(|binding| binding.path.clone()),
            )
            .collect::<Vec<_>>();
        for path in paths {
            let Some(value) = self
                .artboard_property_bindings
                .iter()
                .find(|binding| binding.path == path && binding.default_value_is_resolved)
                .map(|binding| binding.default_value.clone())
                .or_else(|| {
                    self.artboard_image_asset_bindings
                        .iter()
                        .find(|binding| binding.path == path)
                        .map(|binding| binding.default_value.clone())
                })
                .or_else(|| {
                    runtime_created_view_model_value_for_path(file, view_model_instance, &path)
                })
            else {
                continue;
            };
            let mut reset_font_binding_indices = Vec::new();
            if let RuntimeDataBindGraphValue::Asset(file_asset_index) = &value {
                for (index, binding) in self
                    .artboard_image_asset_bindings
                    .iter_mut()
                    .enumerate()
                    .filter(|(_, binding)| binding.target.is_font() && binding.path == path)
                {
                    // A newly selected ViewModelInstance owns a distinct
                    // private FontAsset. Do not leak a live font retained by
                    // the previously bound instance merely because both
                    // serialized indices happen to match.
                    let next = RuntimeFontAssetValue::from_file_asset_index(*file_asset_index);
                    if binding
                        .font_value
                        .as_ref()
                        .is_none_or(|current| !runtime_font_asset_values_equal(current, &next))
                    {
                        binding.font_value = Some(next);
                        reset_font_binding_indices.push(index);
                    }
                }
            }
            let value_changed = self.set_artboard_data_bind_value_for_path(&path, value);
            changed |= value_changed;
            if !value_changed {
                for index in reset_font_binding_indices {
                    self.artboard_data_bind_target_queues
                        .enqueue_image_asset(index);
                    changed = true;
                }
            }
        }
        for binding in &mut self.artboard_list_bindings {
            let Some(source_value) = binding.default_value.resolve_from_view_model_instance(
                file,
                view_model_instance,
                &binding.path,
            ) else {
                continue;
            };
            let target_value = match binding.converter.as_ref() {
                Some(converter) => runtime_data_bind_graph_convert_value(converter, &source_value),
                None => Some(source_value.clone()),
            };
            binding.source_value = Some(source_value.clone());
            binding.source_list_size = match &source_value {
                RuntimeDataBindGraphValue::List { item_count } => Some(*item_count),
                _ => None,
            };
            binding.source_number_value = match &source_value {
                RuntimeDataBindGraphValue::Number(value) => Some(*value),
                _ => None,
            };
            binding.should_reset_instances = binding
                .converter
                .as_ref()
                .and_then(RuntimeDataBindGraphConverter::number_to_list_view_model_id)
                .is_some();
            let target_list_size = match target_value {
                Some(RuntimeDataBindGraphValue::List { .. }) => Some(0),
                _ => None,
            };
            if binding.target_list_size != target_list_size {
                changed = true;
                binding.target_list_size = target_list_size;
            }
        }
        changed
    }

    pub fn advance_artboard_data_binds(&mut self) -> bool {
        self.advance_artboard_data_binds_with_elapsed(0.0)
    }

    /// Repoint the isolated paint evaluator's owned-context writes without
    /// touching the live occurrence. Descendants participate because host
    /// collection advances their data-bind containers too.
    pub(crate) fn detach_initial_nested_layout_paint_binding_contexts(&mut self) {
        let mut detached_handles = Vec::new();
        self.detach_initial_nested_layout_paint_binding_contexts_recursive(&mut detached_handles);
    }

    fn detach_initial_nested_layout_paint_binding_contexts_recursive(
        &mut self,
        detached_handles: &mut Vec<(RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelHandle)>,
    ) {
        for state in self.artboard_shared_data_bind_converter_states.values_mut() {
            state.converter.detach_scripted_instance();
        }
        for converter in self
            .artboard_property_bindings
            .iter_mut()
            .filter_map(|binding| binding.converter.as_mut())
            .chain(
                self.artboard_custom_property_bindings
                    .iter_mut()
                    .filter_map(|binding| binding.converter.as_mut()),
            )
            .chain(
                self.artboard_formula_token_bindings
                    .iter_mut()
                    .filter_map(|binding| binding.converter.as_mut()),
            )
            .chain(
                self.artboard_converter_property_bindings
                    .iter_mut()
                    .filter_map(|binding| binding.converter.as_mut()),
            )
            .chain(
                self.artboard_list_bindings
                    .iter_mut()
                    .filter_map(|binding| binding.converter.as_mut()),
            )
        {
            converter.detach_scripted_instance();
        }
        if !self.artboard_owned_view_model_candidates.is_empty() {
            let candidates = self
                .artboard_owned_view_model_candidates
                .iter()
                .map(|candidate| {
                    let context = detached_handles
                        .iter()
                        .find(|(source, _)| source.ptr_eq(&candidate.context))
                        .map(|(_, detached)| detached.clone())
                        .unwrap_or_else(|| {
                            let detached = RuntimeOwnedViewModelHandle::new(
                                candidate.context.borrow().clone(),
                            );
                            detached_handles.push((candidate.context.clone(), detached.clone()));
                            detached
                        });
                    RuntimeOwnedViewModelBindingCandidate {
                        context,
                        context_chain: candidate.context_chain.clone(),
                    }
                })
                .collect::<Vec<_>>();
            self.artboard_owned_view_model_candidates = candidates.clone();
            if let Some(file) = self.runtime_file_arc() {
                self.bind_owned_view_model_target_to_source_bindings(&file, &candidates, true);
            }
        }

        for nested in self.nested_artboards.values_mut() {
            nested
                .child
                .detach_initial_nested_layout_paint_binding_contexts_recursive(detached_handles);
        }
        for item in self.component_list_items.values_mut().flatten() {
            item.child
                .detach_initial_nested_layout_paint_binding_contexts_recursive(detached_handles);
        }
    }

    pub(crate) fn set_artboard_data_bind_value_for_path(
        &mut self,
        path: &[u32],
        value: RuntimeDataBindGraphValue,
    ) -> bool {
        self.set_artboard_data_bind_value_for_path_with_suppressed_data_bind(path, value, None)
    }

    fn set_artboard_data_bind_value_for_path_with_suppressed_data_bind(
        &mut self,
        path: &[u32],
        value: RuntimeDataBindGraphValue,
        suppressed_data_bind_index: Option<usize>,
    ) -> bool {
        if self.artboard_data_bind_values.get(path) == Some(&value) {
            return false;
        }
        self.mark_artboard_data_bind_work_dirty();
        if let RuntimeDataBindGraphValue::Asset(file_asset_index) = &value {
            for binding in self
                .artboard_image_asset_bindings
                .iter_mut()
                .filter(|binding| binding.target.is_font() && binding.path == path)
            {
                match binding.font_value.as_mut() {
                    Some(font_value) => {
                        font_value.set_file_asset_index(*file_asset_index);
                    }
                    None => {
                        binding.font_value = Some(RuntimeFontAssetValue::from_file_asset_index(
                            *file_asset_index,
                        ));
                    }
                }
            }
        }
        let number_value = match &value {
            RuntimeDataBindGraphValue::Number(value) => Some(*value),
            _ => None,
        };
        self.artboard_data_bind_values.insert(path.to_vec(), value);
        self.reset_artboard_property_formula_random_state_for_path_with_suppressed_data_bind(
            path,
            suppressed_data_bind_index,
        );
        self.enqueue_artboard_data_bind_targets_for_path_with_suppressed_data_bind(
            path,
            suppressed_data_bind_index,
        );
        if let Some(value) = number_value {
            self.refresh_artboard_operation_view_model_number_converter_dependents_for_path_with_suppressed_data_bind(
                path,
                value,
                suppressed_data_bind_index,
            );
        }
        true
    }

    fn set_artboard_data_bind_value_for_path_ref(
        &mut self,
        path: &[u32],
        value: &RuntimeDataBindGraphValue,
    ) -> bool {
        if self.artboard_data_bind_values.get(path) == Some(value) {
            return false;
        }
        self.set_artboard_data_bind_value_for_path(path, value.clone())
    }

    pub fn advance_artboard_data_binds_with_elapsed(&mut self, elapsed_seconds: f32) -> bool {
        self.advance_artboard_data_binds_with_root_transform(Mat2D::IDENTITY, elapsed_seconds)
    }

    fn refresh_owned_view_model_artboard_context_if_mutated(&mut self) -> bool {
        if self.artboard_owned_view_model_candidates.is_empty()
            || self.artboard_owned_context_key.as_ref().is_some_and(|key| {
                key.matches_candidate_mutations(&self.artboard_owned_view_model_candidates)
            })
        {
            return false;
        }
        let Some(file) = self.runtime_file_arc() else {
            return false;
        };
        let candidates = self.artboard_owned_view_model_candidates.clone();
        // A value mutation does not relink the DataContext tree in C++.
        // Dependents are dirtied on their own DataBindContainer and observe
        // the shared source when that container reaches its scheduled
        // updateDataBinds pass. Refresh only this artboard here; recursively
        // rebinding descendants makes later siblings observe a reverse write
        // before their C++ counterpart and also resets stateful converters.
        if !self.retain_owned_view_model_context_candidates(&candidates) {
            return false;
        }
        self.mark_artboard_data_bind_work_dirty();
        self.stateful_nested_view_model_contexts_dirty = true;
        self.bind_owned_view_model_target_to_source_bindings(&file, &candidates, true);
        let mut changed = self.refresh_artboard_converter_dependents(|converter| {
            runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_candidates(
                converter,
                &candidates,
            )
        });
        changed |=
            self.bind_owned_view_model_artboard_values_for_candidates(&file, &candidates, true);
        self.artboard_data_bind_source_queues
            .enqueue_numeric_push_sources();
        changed
    }

    #[inline]
    pub(crate) fn advance_artboard_data_binds_with_root_transform(
        &mut self,
        root_transform: Mat2D,
        elapsed_seconds: f32,
    ) -> bool {
        // Match C++'s cheap clean `DataBindContainer::updateDataBinds` return
        // before entering the active reconciliation routine. An owned-context
        // refresh can itself dirty this epoch, so only bypass that refresh when
        // no candidate exists and always re-read the epoch afterwards.
        let clean_identity_pass = elapsed_seconds == 0.0
            && root_transform == Mat2D::IDENTITY
            && self.artboard_data_bind_dirty_epoch == self.artboard_data_bind_processed_epoch
            && self.artboard_list_bindings.is_empty();
        if clean_identity_pass
            && self.artboard_owned_view_model_candidates.is_empty()
            && self.artboard_owned_view_model_handle.is_none()
        {
            return false;
        }
        let refreshed_owned_context = if self.artboard_owned_view_model_handle.is_some() {
            self.refresh_retained_owned_view_model_artboard_context()
        } else {
            self.refresh_owned_view_model_artboard_context_if_mutated()
        };
        if elapsed_seconds == 0.0
            && root_transform == Mat2D::IDENTITY
            && self.artboard_data_bind_dirty_epoch == self.artboard_data_bind_processed_epoch
            && self.artboard_list_bindings.is_empty()
        {
            return refreshed_owned_context;
        }
        self.advance_active_artboard_data_binds_with_root_transform(
            root_transform,
            elapsed_seconds,
            refreshed_owned_context,
        )
    }

    #[inline(never)]
    fn advance_active_artboard_data_binds_with_root_transform(
        &mut self,
        root_transform: Mat2D,
        elapsed_seconds: f32,
        refreshed_owned_context: bool,
    ) -> bool {
        let dirty_epoch_at_start = self.artboard_data_bind_dirty_epoch;
        let mut changed = refreshed_owned_context;
        // C++ DataBindContainer::updateDataBind updates converter dependents
        // before applying a target-to-source binding.
        changed |= self.update_artboard_formula_token_bindings();
        changed |= self.update_artboard_converter_property_bindings();
        // A two-way reconcile carries both direction bits. C++ runs the
        // source-to-target half before reading the target only when the
        // authored SourceToTargetRunsFirst flag is set. Keep pure toTarget
        // binds and target-first two-way binds on the ordinary post-source
        // lane below.
        changed |= self.apply_artboard_property_bindings_for_precedence(true);
        if self
            .artboard_data_bind_source_queues
            .has_custom_property_update_indices()
        {
            let custom_property_update_indices = self
                .artboard_data_bind_source_queues
                .take_custom_property_update_indices();
            for index in custom_property_update_indices.iter().copied() {
                changed |= self.update_artboard_custom_property_binding(index);
            }
            self.artboard_data_bind_source_queues
                .recycle_custom_property_update_indices(custom_property_update_indices);
        }
        changed |= self.update_artboard_layout_computed_bindings(root_transform);
        changed |= self.update_artboard_solo_source_bindings();
        changed |= self.update_artboard_numeric_source_bindings();
        changed |= self.apply_artboard_property_bindings_for_precedence(false);
        changed |= self.apply_artboard_image_asset_bindings();
        if elapsed_seconds != 0.0 {
            let property_converters_changed =
                self.advance_artboard_property_binding_converters(elapsed_seconds);
            changed |= property_converters_changed;
            changed |= self.advance_artboard_custom_property_binding_converters(elapsed_seconds);
            if property_converters_changed {
                changed |= self.apply_artboard_property_bindings();
                changed |= self.apply_artboard_image_asset_bindings();
            }
        }
        let runtime_file = self.runtime_file_arc();
        let mut generated_list_updates = Vec::new();
        for binding in &mut self.artboard_list_bindings {
            if let (Some(source_value), Some(file)) =
                (binding.source_value.clone(), runtime_file.as_deref())
                && let Some(converter) = binding.converter.as_ref()
                && let Some(RuntimeArtboardListConvertedValue {
                    value: RuntimeDataBindGraphValue::List { item_count },
                    generated_view_model_id: Some(view_model_id),
                }) = runtime_artboard_convert_list_value(
                    converter,
                    RuntimeArtboardListConvertedValue {
                        value: source_value.clone(),
                        generated_view_model_id: None,
                    },
                )
            {
                generated_list_updates.push(binding.reconcile_number_to_list_items(
                    file,
                    source_value,
                    item_count,
                    view_model_id,
                ));
                continue;
            }
            let target_list_size = if let Some(source_list_size) = binding.source_list_size {
                Some(source_list_size)
            } else {
                let target_value = match binding.converter.as_ref() {
                    Some(converter) => {
                        runtime_data_bind_graph_convert_value(converter, &binding.default_value)
                    }
                    None => Some(binding.default_value.clone()),
                };
                match target_value {
                    Some(RuntimeDataBindGraphValue::List { item_count }) => Some(item_count),
                    _ => None,
                }
            };
            if binding.target_list_size != target_list_size {
                binding.target_list_size = target_list_size;
                changed = true;
            }
        }
        for update in generated_list_updates {
            changed |= update.binding_changed;
            self.component_list_sources.remove(&update.target_local_id);
            if let (Some(file), Some(items)) = (runtime_file.as_deref(), update.items) {
                changed |= self.sync_component_list_items(file, update.target_local_id, items);
            }
        }
        changed |= self.apply_artboard_solo_bindings();
        changed |= self.apply_artboard_nested_host_bindings();
        changed |= self.sync_nested_child_artboard_data_contexts();
        if self.artboard_data_bind_dirty_epoch == dirty_epoch_at_start {
            self.artboard_data_bind_processed_epoch = dirty_epoch_at_start;
        }
        changed
    }

    fn refresh_retained_owned_view_model_artboard_context(&mut self) -> bool {
        let Some(context) = self.artboard_owned_view_model_handle.clone() else {
            return false;
        };
        let Some(file) = self.runtime_file_arc() else {
            return false;
        };
        let root = context.root_handle();
        let root = root.borrow();
        let context_chain = [context.scope_path()];
        self.bind_owned_view_model_artboard_context_chain(&file, &root, &context_chain, true, true)
    }

    fn write_retained_owned_view_model_artboard_source(
        &self,
        path: &[u32],
        path_is_name_based: bool,
        retained_source_path: Option<&[usize]>,
        value: &RuntimeDataBindGraphValue,
    ) -> (bool, Option<Vec<usize>>) {
        let Some(handle) = self.artboard_owned_view_model_handle.clone() else {
            return (false, None);
        };
        let Some(file) = self.runtime_file_arc() else {
            return (false, None);
        };
        let Some(context_key) = self.artboard_owned_context_key.as_ref() else {
            return (false, None);
        };
        let root = handle.root_handle();
        let mut context = root.borrow_mut();

        if let Some(source_path) = retained_source_path
            && let Some(changed) = runtime_owned_view_model_set_binding_value_for_property_path(
                &mut context,
                source_path,
                value,
            )
        {
            return (changed, Some(source_path.to_vec()));
        }

        let Some(context_key) = context_key.instances.iter().find(|key| {
            key.instance_identity == context.instance_identity()
                && key.view_model_index == context.view_model_index()
        }) else {
            return (false, None);
        };
        for context_path in &context_key.context_chain {
            let Some(source_path) = context
                .property_path_for_context_source_path_with_manifest_mode(
                    &file,
                    context_path,
                    path,
                    path_is_name_based,
                    true,
                )
            else {
                continue;
            };
            if let Some(changed) = runtime_owned_view_model_set_binding_value_for_property_path(
                &mut context,
                &source_path,
                value,
            ) {
                return (changed, Some(source_path));
            }
        }
        (false, None)
    }

    fn update_artboard_numeric_source_bindings(&mut self) -> bool {
        if !self
            .artboard_data_bind_source_queues
            .has_numeric_source_update_indices()
        {
            return false;
        }
        let mut changed = false;
        let indices = self
            .artboard_data_bind_source_queues
            .take_numeric_source_update_indices();
        for index in indices.iter().copied() {
            let Some((
                target_local_id,
                property_key,
                property,
                path,
                owned_context_source,
                path_is_name_based,
                retained_source_path,
            )) = self
                .artboard_numeric_source_bindings
                .get(index)
                .map(|binding| {
                    (
                        binding.target_local_id,
                        binding.property_key,
                        binding.property,
                        binding.path.clone(),
                        binding.owned_context_source.clone(),
                        binding.path_is_name_based,
                        binding.owned_context_source_path.clone(),
                    )
                })
            else {
                continue;
            };
            // C++ `DataBindContainer::updateDataBinds` skips persisting
            // computed targets while their component is collapsed. Keep the
            // previously bound source value instead of publishing the layout
            // engine's display:none zero bounds.
            if self.runtime_component_is_collapsed_for_draw(target_local_id) {
                continue;
            }
            let value =
                self.artboard_numeric_source_binding_value(target_local_id, property_key, property);
            let Some(value) = value else { continue };
            let value = RuntimeDataBindGraphValue::Number(value);
            if self.artboard_owned_view_model_handle.is_some() {
                let (retained_changed, resolved_source_path) = self
                    .write_retained_owned_view_model_artboard_source(
                        &path,
                        path_is_name_based,
                        retained_source_path.as_deref(),
                        &value,
                    );
                if let Some(binding) = self.artboard_numeric_source_bindings.get_mut(index) {
                    binding.owned_context_source_path = resolved_source_path;
                }
                changed |= retained_changed;
            } else if let Some(source) = owned_context_source {
                changed |= source.sync_value(&value);
            } else if let Some(file) = self.runtime_file_arc() {
                changed |= runtime_owned_view_model_sync_value_for_candidates(
                    &file,
                    &self.artboard_owned_view_model_candidates,
                    &path,
                    false,
                    &value,
                )
                .unwrap_or(false);
            }
            changed |= self.set_artboard_data_bind_value_for_path(&path, value);
        }
        self.artboard_data_bind_source_queues
            .recycle_numeric_source_update_indices(indices);
        changed
    }

    fn artboard_numeric_source_binding_value(
        &self,
        target_local_id: usize,
        property_key: u16,
        property: RuntimeArtboardNumericSourceProperty,
    ) -> Option<f32> {
        match property {
            RuntimeArtboardNumericSourceProperty::DirectDouble => self
                .layout_constraint_bounds_enabled
                .then(|| {
                    let graph = self.runtime_graph()?;
                    graph.paths.iter().find(|path| {
                        path.local_id == target_local_id && path.parametric.is_some()
                    })?;
                    let (width, height) =
                        self.runtime_parametric_path_layout_control_size(target_local_id, graph)?;
                    if runtime_data_bind_property_key_for_name("ParametricPath", "width")
                        == Some(property_key)
                    {
                        Some(width)
                    } else if runtime_data_bind_property_key_for_name("ParametricPath", "height")
                        == Some(property_key)
                    {
                        Some(height)
                    } else {
                        None
                    }
                })
                .flatten()
                .or_else(|| self.double_property(target_local_id, property_key)),
            RuntimeArtboardNumericSourceProperty::ShapeLength => self
                .runtime_graph()
                .and_then(|graph| self.artboard_shape_length(target_local_id, graph)),
        }
    }

    pub(crate) fn enqueue_artboard_parametric_layout_control_sources(&mut self) {
        let Some(graph) = self.runtime_graph() else {
            return;
        };
        let indices = self
            .artboard_numeric_source_bindings
            .iter()
            .enumerate()
            .filter_map(|(index, binding)| {
                (matches!(
                    binding.property,
                    RuntimeArtboardNumericSourceProperty::DirectDouble
                ) && graph.paths.iter().any(|path| {
                    path.local_id == binding.target_local_id && path.parametric.is_some()
                }) && self
                    .runtime_parametric_path_layout_control_size(binding.target_local_id, graph)
                    .is_some())
                .then_some(index)
            })
            .collect::<Vec<_>>();
        for index in indices {
            self.artboard_data_bind_source_queues
                .enqueue_numeric_source(index);
        }
    }

    fn update_artboard_formula_token_bindings(&mut self) -> bool {
        let mut changed = false;
        for index in 0..self.artboard_formula_token_bindings.len() {
            let Some((target, value)) = self.converted_artboard_formula_token_binding_value(index)
            else {
                continue;
            };
            changed |= match target {
                RuntimeArtboardFormulaBindingTarget::FormulaToken { global_id } => {
                    self.set_artboard_formula_token_value(global_id, value)
                }
                RuntimeArtboardFormulaBindingTarget::OperationValue { global_id } => {
                    self.set_artboard_operation_value(global_id, value)
                }
            };
        }
        changed
    }

    fn converted_artboard_formula_token_binding_value(
        &mut self,
        index: usize,
    ) -> Option<(RuntimeArtboardFormulaBindingTarget, f32)> {
        let binding = self.artboard_formula_token_bindings.get_mut(index)?;
        let value = self
            .artboard_data_bind_values
            .get(&binding.path)
            .cloned()
            .unwrap_or_else(|| binding.default_value.clone());
        let converted = match binding.converter.as_ref() {
            Some(converter) => binding.converter_state.convert_value_with_formula_randoms(
                converter,
                &value,
                &mut self.artboard_formula_random_source,
            ),
            None => Some(value),
        }?;
        match converted {
            RuntimeDataBindGraphValue::Number(value) => Some((binding.target, value)),
            _ => None,
        }
    }

    fn update_artboard_converter_property_bindings(&mut self) -> bool {
        let mut changed = false;
        loop {
            let indices = self
                .artboard_data_bind_target_queues
                .drain_dirty_converter_properties();
            if indices.is_empty() {
                break;
            }
            for index in indices {
                let Some(update) = self.converted_artboard_converter_property_binding_value(index)
                else {
                    continue;
                };
                changed |= match update {
                    RuntimeArtboardConverterPropertyBindingUpdate::ToStringDecimals {
                        global_id,
                        value,
                    } => self.set_artboard_to_string_converter_decimals(global_id, value),
                    RuntimeArtboardConverterPropertyBindingUpdate::ToStringColorFormat {
                        global_id,
                        value,
                    } => self.set_artboard_to_string_converter_color_format(global_id, &value),
                    RuntimeArtboardConverterPropertyBindingUpdate::StringTrimTrimType {
                        global_id,
                        value,
                    } => self.set_artboard_string_trim_converter_trim_type(global_id, value),
                    RuntimeArtboardConverterPropertyBindingUpdate::StringPadLength {
                        global_id,
                        value,
                    } => self.set_artboard_string_pad_converter_length(global_id, value),
                    RuntimeArtboardConverterPropertyBindingUpdate::StringPadText {
                        global_id,
                        value,
                    } => self.set_artboard_string_pad_converter_text(global_id, &value),
                    RuntimeArtboardConverterPropertyBindingUpdate::StringPadPadType {
                        global_id,
                        value,
                    } => self.set_artboard_string_pad_converter_pad_type(global_id, value),
                    RuntimeArtboardConverterPropertyBindingUpdate::InterpolatorDuration {
                        global_id,
                        value,
                    } => self.set_artboard_interpolator_converter_duration(global_id, value),
                    RuntimeArtboardConverterPropertyBindingUpdate::RangeMapper {
                        global_id,
                        property,
                        value,
                    } => self.set_artboard_range_mapper_converter_value(global_id, property, value),
                    RuntimeArtboardConverterPropertyBindingUpdate::NumberToListViewModelId {
                        global_id,
                        value,
                    } => self.set_artboard_number_to_list_converter_view_model_id(global_id, value),
                };
            }
        }
        changed
    }

    fn converted_artboard_converter_property_binding_value(
        &mut self,
        index: usize,
    ) -> Option<RuntimeArtboardConverterPropertyBindingUpdate> {
        let binding = self.artboard_converter_property_bindings.get_mut(index)?;
        let value = self
            .artboard_data_bind_values
            .get(&binding.path)
            .cloned()
            .unwrap_or_else(|| binding.default_value.clone());
        let converted = match binding.converter.as_ref() {
            Some(converter) => binding.converter_state.convert_value_with_formula_randoms(
                converter,
                &value,
                &mut self.artboard_formula_random_source,
            ),
            None => Some(value),
        }?;
        runtime_artboard_converter_property_binding_update(binding.target, converted)
    }

    fn set_artboard_formula_token_value(&mut self, token_id: u32, value: f32) -> bool {
        let mut changed = false;
        let mut shared_changed = Vec::new();
        for (data_bind_index, state) in &mut self.artboard_shared_data_bind_converter_states {
            if state.converter.set_formula_token_value(token_id, value) {
                state.converter_state.reset_formula_randoms();
                shared_changed.push(*data_bind_index);
                changed = true;
            }
        }
        for data_bind_index in shared_changed {
            self.enqueue_artboard_shared_converter_direction(data_bind_index);
        }
        for index in 0..self.artboard_property_bindings.len() {
            if self
                .artboard_shared_data_bind_converter_states
                .contains_key(&self.artboard_property_bindings[index].data_bind_index)
            {
                continue;
            }
            let binding_changed = {
                let binding = &mut self.artboard_property_bindings[index];
                let Some(converter) = binding.converter.as_mut() else {
                    continue;
                };
                if converter.set_formula_token_value(token_id, value) {
                    binding.converter_state.reset_formula_randoms();
                    true
                } else {
                    false
                }
            };
            if binding_changed {
                self.enqueue_artboard_property_binding_target(index);
                changed = true;
            }
        }
        for index in 0..self.artboard_custom_property_bindings.len() {
            if self
                .artboard_shared_data_bind_converter_states
                .contains_key(&self.artboard_custom_property_bindings[index].data_bind_index)
            {
                continue;
            }
            let binding_changed = {
                let binding = &mut self.artboard_custom_property_bindings[index];
                let Some(converter) = binding.converter.as_mut() else {
                    continue;
                };
                if converter.set_formula_token_value(token_id, value) {
                    binding.converter_state.reset_formula_randoms();
                    true
                } else {
                    false
                }
            };
            if binding_changed {
                self.artboard_data_bind_source_queues
                    .enqueue_custom_property(index);
                changed = true;
            }
        }
        changed
    }

    fn set_artboard_operation_value(&mut self, target_global_id: u32, value: f32) -> bool {
        let mut changed = false;
        let mut shared_changed = Vec::new();
        for (data_bind_index, state) in &mut self.artboard_shared_data_bind_converter_states {
            if state.converter.set_operation_value(target_global_id, value) {
                state.converter_state.reset_formula_randoms();
                shared_changed.push(*data_bind_index);
                changed = true;
            }
        }
        for data_bind_index in shared_changed {
            self.enqueue_artboard_shared_converter_direction(data_bind_index);
        }
        for index in 0..self.artboard_property_bindings.len() {
            if self
                .artboard_shared_data_bind_converter_states
                .contains_key(&self.artboard_property_bindings[index].data_bind_index)
            {
                continue;
            }
            let binding_changed = {
                let binding = &mut self.artboard_property_bindings[index];
                let Some(converter) = binding.converter.as_mut() else {
                    continue;
                };
                if converter.set_operation_value(target_global_id, value) {
                    binding.converter_state.reset_formula_randoms();
                    true
                } else {
                    false
                }
            };
            if binding_changed {
                self.enqueue_artboard_property_binding_target(index);
                changed = true;
            }
        }
        for index in 0..self.artboard_custom_property_bindings.len() {
            if self
                .artboard_shared_data_bind_converter_states
                .contains_key(&self.artboard_custom_property_bindings[index].data_bind_index)
            {
                continue;
            }
            let binding_changed = {
                let binding = &mut self.artboard_custom_property_bindings[index];
                let Some(converter) = binding.converter.as_mut() else {
                    continue;
                };
                if converter.set_operation_value(target_global_id, value) {
                    binding.converter_state.reset_formula_randoms();
                    true
                } else {
                    false
                }
            };
            if binding_changed {
                self.artboard_data_bind_source_queues
                    .enqueue_custom_property(index);
                changed = true;
            }
        }
        changed
    }

    fn set_artboard_to_string_converter_decimals(
        &mut self,
        target_global_id: u32,
        value: u64,
    ) -> bool {
        self.refresh_artboard_converter_dependents(|converter| {
            converter.set_to_string_decimals(target_global_id, value)
        })
    }

    fn set_artboard_to_string_converter_color_format(
        &mut self,
        target_global_id: u32,
        value: &[u8],
    ) -> bool {
        self.refresh_artboard_converter_dependents(|converter| {
            converter.set_to_string_color_format(target_global_id, value)
        })
    }

    fn set_artboard_string_trim_converter_trim_type(
        &mut self,
        target_global_id: u32,
        value: u64,
    ) -> bool {
        self.refresh_artboard_converter_dependents(|converter| {
            converter.set_string_trim_trim_type(target_global_id, value)
        })
    }

    fn set_artboard_string_pad_converter_length(
        &mut self,
        target_global_id: u32,
        value: u64,
    ) -> bool {
        self.refresh_artboard_converter_dependents(|converter| {
            converter.set_string_pad_length(target_global_id, value)
        })
    }

    fn set_artboard_string_pad_converter_text(
        &mut self,
        target_global_id: u32,
        value: &[u8],
    ) -> bool {
        self.refresh_artboard_converter_dependents(|converter| {
            converter.set_string_pad_text(target_global_id, value)
        })
    }

    fn set_artboard_string_pad_converter_pad_type(
        &mut self,
        target_global_id: u32,
        value: u64,
    ) -> bool {
        self.refresh_artboard_converter_dependents(|converter| {
            converter.set_string_pad_pad_type(target_global_id, value)
        })
    }

    fn set_artboard_interpolator_converter_duration(
        &mut self,
        target_global_id: u32,
        value: f32,
    ) -> bool {
        self.refresh_artboard_converter_dependents(|converter| {
            converter.set_interpolator_duration(target_global_id, value)
        })
    }

    fn set_artboard_range_mapper_converter_value(
        &mut self,
        target_global_id: u32,
        property: RuntimeDataBindGraphRangeMapperProperty,
        value: f32,
    ) -> bool {
        self.refresh_artboard_converter_dependents(|converter| {
            converter.set_range_mapper_value(target_global_id, property, value)
        })
    }

    fn set_artboard_number_to_list_converter_view_model_id(
        &mut self,
        target_global_id: u32,
        value: u64,
    ) -> bool {
        self.refresh_artboard_converter_dependents(|converter| {
            converter.set_number_to_list_view_model_id(target_global_id, value)
        })
    }

    fn artboard_shape_length(&self, shape_local_id: usize, graph: &ArtboardGraph) -> Option<f32> {
        self.runtime_shape_length_with_layout(shape_local_id, graph)
    }

    fn update_artboard_layout_computed_bindings(&mut self, root_transform: Mat2D) -> bool {
        let mut changed = false;
        let indices = self
            .artboard_data_bind_source_queues
            .persisting_layout_computed()
            .to_vec();
        for index in indices {
            let Some((
                target_local_id,
                property,
                path,
                owned_context_source,
                path_is_name_based,
                retained_source_path,
            )) = self
                .artboard_layout_computed_bindings
                .get(index)
                .map(|binding| {
                    (
                        binding.target_local_id,
                        binding.property,
                        binding.path.clone(),
                        binding.owned_context_source.clone(),
                        binding.path_is_name_based,
                        binding.owned_context_source_path.clone(),
                    )
                })
            else {
                continue;
            };
            if self.runtime_component_is_collapsed_for_draw(target_local_id) {
                continue;
            }
            let value = self.runtime_graph().and_then(|graph| {
                self.artboard_layout_computed_binding_value(
                    target_local_id,
                    property,
                    graph,
                    root_transform,
                )
            });
            let Some(value) = value else { continue };
            let value = RuntimeDataBindGraphValue::Number(value);
            if self.artboard_owned_view_model_handle.is_some() {
                let (retained_changed, resolved_source_path) = self
                    .write_retained_owned_view_model_artboard_source(
                        path.as_ref(),
                        path_is_name_based,
                        retained_source_path.as_deref(),
                        &value,
                    );
                if let Some(binding) = self.artboard_layout_computed_bindings.get_mut(index) {
                    binding.owned_context_source_path = resolved_source_path;
                }
                changed |= retained_changed;
            } else if let Some(source) = owned_context_source {
                changed |= source.sync_value(&value);
            } else if let Some(file) = self.runtime_file_arc() {
                changed |= runtime_owned_view_model_sync_value_for_candidates(
                    &file,
                    &self.artboard_owned_view_model_candidates,
                    path.as_ref(),
                    false,
                    &value,
                )
                .unwrap_or(false);
            }
            changed |= self.set_artboard_data_bind_value_for_path(path.as_ref(), value);
        }
        changed
    }

    fn artboard_layout_computed_binding_value(
        &self,
        target_local_id: usize,
        property: RuntimeLayoutComputedProperty,
        graph: &ArtboardGraph,
        root_transform: Mat2D,
    ) -> Option<f32> {
        match property {
            RuntimeLayoutComputedProperty::RootX | RuntimeLayoutComputedProperty::RootY => {
                let x = self.runtime_layout_computed_property(
                    target_local_id,
                    RuntimeLayoutComputedProperty::WorldX,
                    graph,
                )?;
                let y = self.runtime_layout_computed_property(
                    target_local_id,
                    RuntimeLayoutComputedProperty::WorldY,
                    graph,
                )?;
                let (x, y) = root_transform.transform_point(x, y);
                match property {
                    RuntimeLayoutComputedProperty::RootX => Some(x),
                    RuntimeLayoutComputedProperty::RootY => Some(y),
                    _ => unreachable!(),
                }
            }
            _ => self.runtime_layout_computed_property(target_local_id, property, graph),
        }
    }

    fn apply_artboard_property_bindings(&mut self) -> bool {
        let indices = self
            .artboard_data_bind_target_queues
            .drain_dirty_properties();
        self.apply_artboard_property_binding_indices(indices)
    }

    fn apply_artboard_property_bindings_for_precedence(
        &mut self,
        source_to_target_runs_first: bool,
    ) -> bool {
        let indices = self
            .artboard_data_bind_target_queues
            .drain_dirty_properties_for_precedence(
                &self.artboard_property_bindings,
                source_to_target_runs_first,
            );
        self.apply_artboard_property_binding_indices(indices)
    }

    fn apply_artboard_property_binding_indices(&mut self, indices: Vec<usize>) -> bool {
        let mut changed = false;
        for index in indices {
            let Some((data_bind_index, target_local_id, property_key, value)) =
                self.converted_artboard_property_binding_value(index)
            else {
                continue;
            };
            let previous_suppression = self.artboard_data_bind_suppressed_target_data_bind;
            self.artboard_data_bind_suppressed_target_data_bind = Some(data_bind_index);
            changed |=
                self.apply_artboard_property_binding_value(target_local_id, property_key, &value);
            self.artboard_data_bind_suppressed_target_data_bind = previous_suppression;
        }
        changed
    }

    fn apply_artboard_image_asset_bindings(&mut self) -> bool {
        let mut changed = false;
        for index in self
            .artboard_data_bind_target_queues
            .drain_dirty_image_assets()
        {
            let Some((target, value, font_value)) = self
                .artboard_image_asset_bindings
                .get(index)
                .map(|binding| {
                    let value = self
                        .artboard_data_bind_values
                        .get(&binding.path)
                        .cloned()
                        .unwrap_or_else(|| binding.default_value.clone());
                    (binding.target, value, binding.font_value.clone())
                })
            else {
                continue;
            };
            changed |= self.apply_artboard_image_asset_binding_value(target, &value, font_value);
        }
        changed
    }

    fn apply_artboard_image_asset_binding_value(
        &mut self,
        target: RuntimeArtboardAssetBindingTarget,
        value: &RuntimeDataBindGraphValue,
        font_value: Option<RuntimeFontAssetValue>,
    ) -> bool {
        let RuntimeDataBindGraphValue::Asset(value) = value else {
            return false;
        };
        match target {
            RuntimeArtboardAssetBindingTarget::Image(target_local_id) => {
                // Mirrors C++ `context_value_asset_image.cpp`: missing values
                // use the private empty ImageAsset, so Image::draw returns.
                let asset_global = self
                    .runtime_file()
                    .and_then(|file| runtime_image_asset_global_for_file_asset_index(file, *value));
                self.set_image_asset_override(target_local_id, asset_global)
            }
            RuntimeArtboardAssetBindingTarget::Font(target_local_id) => {
                // C++ TextStyle::setAsset swaps the retained FontAsset pointer
                // without rewriting the serialized fontAssetId property.
                self.set_text_style_font_override(
                    target_local_id,
                    font_value
                        .unwrap_or_else(|| RuntimeFontAssetValue::from_file_asset_index(*value)),
                )
            }
        }
    }

    fn converted_artboard_property_binding_value(
        &mut self,
        index: usize,
    ) -> Option<(usize, usize, u16, RuntimeDataBindGraphValue)> {
        let binding = self.artboard_property_bindings.get_mut(index)?;
        let value = binding
            .pending_value
            .take()
            .or_else(|| self.artboard_data_bind_values.get(&binding.path).cloned())?;
        let converted = if let Some(shared) = self
            .artboard_shared_data_bind_converter_states
            .get_mut(&binding.data_bind_index)
        {
            runtime_artboard_convert_property_binding_value(
                &shared.converter,
                &mut shared.converter_state,
                value,
                &binding.enum_value_names,
                &mut self.artboard_formula_random_source,
            )
        } else if let Some(converter) = binding.converter.as_ref() {
            runtime_artboard_convert_property_binding_value(
                converter,
                &mut binding.converter_state,
                value,
                &binding.enum_value_names,
                &mut self.artboard_formula_random_source,
            )
        } else {
            Some(value)
        }?;
        Some((
            binding.data_bind_index,
            binding.target_local_id,
            binding.property_key,
            converted,
        ))
    }

    fn reset_artboard_property_formula_random_state_for_path(&mut self, path: &[u32]) {
        self.reset_artboard_property_formula_random_state_for_path_with_suppressed_data_bind(
            path, None,
        );
    }

    fn reset_artboard_property_formula_random_state_for_path_with_suppressed_data_bind(
        &mut self,
        path: &[u32],
        suppressed_data_bind_index: Option<usize>,
    ) {
        let shared_indices = self
            .artboard_property_bindings
            .iter()
            .filter(|binding| {
                binding.path == path && Some(binding.data_bind_index) != suppressed_data_bind_index
            })
            .map(|binding| binding.data_bind_index)
            .collect::<Vec<_>>();
        for data_bind_index in shared_indices {
            if let Some(state) = self
                .artboard_shared_data_bind_converter_states
                .get_mut(&data_bind_index)
                && runtime_data_bind_graph_converter_contains_source_change_random(&state.converter)
            {
                state.converter_state.reset_formula_randoms();
            }
        }
        for binding in &mut self.artboard_property_bindings {
            if Some(binding.data_bind_index) == suppressed_data_bind_index {
                continue;
            }
            if self
                .artboard_shared_data_bind_converter_states
                .contains_key(&binding.data_bind_index)
            {
                continue;
            }
            if binding.path == path
                && binding
                    .converter
                    .as_ref()
                    .is_some_and(runtime_data_bind_graph_converter_contains_source_change_random)
            {
                binding.converter_state.reset_formula_randoms();
            }
        }
    }

    fn refresh_artboard_converter_dependents(
        &mut self,
        update: impl FnMut(&mut RuntimeDataBindGraphConverter) -> bool,
    ) -> bool {
        self.refresh_artboard_converter_dependents_with_suppressed_data_bind(None, update)
    }

    fn refresh_artboard_converter_dependents_with_suppressed_data_bind(
        &mut self,
        suppressed_data_bind_index: Option<usize>,
        mut update: impl FnMut(&mut RuntimeDataBindGraphConverter) -> bool,
    ) -> bool {
        let mut changed = false;

        let mut shared_changed = Vec::new();
        for (data_bind_index, state) in &mut self.artboard_shared_data_bind_converter_states {
            if update(&mut state.converter) {
                shared_changed.push(*data_bind_index);
                changed = true;
            }
        }
        for data_bind_index in shared_changed {
            if Some(data_bind_index) != suppressed_data_bind_index {
                self.enqueue_artboard_shared_converter_direction(data_bind_index);
            }
        }

        for index in 0..self.artboard_property_bindings.len() {
            if self
                .artboard_shared_data_bind_converter_states
                .contains_key(&self.artboard_property_bindings[index].data_bind_index)
            {
                continue;
            }
            let binding_changed = {
                let binding = &mut self.artboard_property_bindings[index];
                binding.converter.as_mut().is_some_and(&mut update)
            };
            if binding_changed {
                if Some(self.artboard_property_bindings[index].data_bind_index)
                    != suppressed_data_bind_index
                {
                    self.enqueue_artboard_property_binding_target(index);
                }
                changed = true;
            }
        }

        for index in 0..self.artboard_custom_property_bindings.len() {
            if self
                .artboard_shared_data_bind_converter_states
                .contains_key(&self.artboard_custom_property_bindings[index].data_bind_index)
            {
                continue;
            }
            let binding_changed = {
                let binding = &mut self.artboard_custom_property_bindings[index];
                binding.converter.as_mut().is_some_and(&mut update)
            };
            if binding_changed {
                if Some(self.artboard_custom_property_bindings[index].data_bind_index)
                    != suppressed_data_bind_index
                {
                    self.artboard_data_bind_source_queues
                        .enqueue_custom_property(index);
                }
                changed = true;
            }
        }

        for binding in &mut self.artboard_formula_token_bindings {
            if binding.converter.as_mut().is_some_and(&mut update) {
                changed = true;
            }
        }

        for index in 0..self.artboard_converter_property_bindings.len() {
            let binding_changed = {
                let binding = &mut self.artboard_converter_property_bindings[index];
                binding.converter.as_mut().is_some_and(&mut update)
            };
            if binding_changed {
                self.artboard_data_bind_target_queues
                    .enqueue_converter_property(index);
                changed = true;
            }
        }

        for binding in &mut self.artboard_list_bindings {
            if binding.converter.as_mut().is_some_and(&mut update) {
                changed = true;
            }
        }

        changed
    }

    fn refresh_artboard_operation_view_model_number_converter_dependents_for_path_with_suppressed_data_bind(
        &mut self,
        path: &[u32],
        value: f32,
        suppressed_data_bind_index: Option<usize>,
    ) -> bool {
        self.refresh_artboard_converter_dependents_with_suppressed_data_bind(
            suppressed_data_bind_index,
            |converter| {
                runtime_data_bind_graph_refresh_operation_view_model_number_converter_for_path(
                    converter, path, value,
                )
            },
        )
    }

    fn advance_artboard_property_binding_converters(&mut self, elapsed_seconds: f32) -> bool {
        let mut changed = false;
        let mut shared_changed = Vec::new();
        for (data_bind_index, state) in &mut self.artboard_shared_data_bind_converter_states {
            let advance = state
                .converter_state
                .advance_converter(Some(&state.converter), elapsed_seconds);
            if advance.changed {
                shared_changed.push(*data_bind_index);
                changed = true;
            }
        }
        for data_bind_index in shared_changed {
            self.enqueue_artboard_shared_converter_direction(data_bind_index);
        }
        for index in 0..self.artboard_property_bindings.len() {
            if self
                .artboard_shared_data_bind_converter_states
                .contains_key(&self.artboard_property_bindings[index].data_bind_index)
            {
                continue;
            }
            let advance = {
                let binding = &mut self.artboard_property_bindings[index];
                binding
                    .converter_state
                    .advance_converter(binding.converter.as_ref(), elapsed_seconds)
            };
            if advance.changed {
                self.enqueue_artboard_property_binding_target(index);
                changed = true;
            }
        }
        changed
    }

    fn advance_artboard_custom_property_binding_converters(
        &mut self,
        elapsed_seconds: f32,
    ) -> bool {
        let mut changed = false;
        for index in 0..self.artboard_custom_property_bindings.len() {
            if self
                .artboard_shared_data_bind_converter_states
                .contains_key(&self.artboard_custom_property_bindings[index].data_bind_index)
            {
                continue;
            }
            let advance = {
                let binding = &mut self.artboard_custom_property_bindings[index];
                binding
                    .converter_state
                    .advance_converter(binding.converter.as_ref(), elapsed_seconds)
            };
            if advance.changed {
                self.artboard_data_bind_source_queues
                    .enqueue_custom_property(index);
                changed = true;
            }
        }
        changed
    }

    fn update_artboard_solo_source_bindings(&mut self) -> bool {
        let mut changed = false;
        let indices = self
            .artboard_data_bind_source_queues
            .persisting_solo_sources()
            .to_vec();
        for index in indices {
            let Some((
                target_local_id,
                enum_value_names,
                path,
                owned_context_source,
                path_is_name_based,
                retained_source_path,
            )) = self
                .artboard_solo_source_bindings
                .get(index)
                .map(|binding| {
                    (
                        binding.target_local_id,
                        binding.enum_value_names.clone(),
                        binding.path.clone(),
                        binding.owned_context_source.clone(),
                        binding.path_is_name_based,
                        binding.owned_context_source_path.clone(),
                    )
                })
            else {
                continue;
            };
            if self.runtime_component_is_collapsed_for_draw(target_local_id) {
                continue;
            }
            let Some(value) =
                self.artboard_solo_source_binding_value(target_local_id, &enum_value_names)
            else {
                continue;
            };
            if self.artboard_owned_view_model_handle.is_some() {
                let (retained_changed, resolved_source_path) = self
                    .write_retained_owned_view_model_artboard_source(
                        path.as_ref(),
                        path_is_name_based,
                        retained_source_path.as_deref(),
                        &value,
                    );
                if let Some(binding) = self.artboard_solo_source_bindings.get_mut(index) {
                    binding.owned_context_source_path = resolved_source_path;
                }
                changed |= retained_changed;
            } else if let Some(source) = owned_context_source {
                changed |= source.sync_value(&value);
            } else if let Some(file) = self.runtime_file_arc() {
                changed |= runtime_owned_view_model_sync_value_for_candidates(
                    &file,
                    &self.artboard_owned_view_model_candidates,
                    path.as_ref(),
                    false,
                    &value,
                )
                .unwrap_or(false);
            }
            changed |= self.set_artboard_data_bind_value_for_path(path.as_ref(), value);
        }
        changed
    }

    fn artboard_solo_source_binding_value(
        &self,
        target_local_id: usize,
        enum_value_names: &[Vec<u8>],
    ) -> Option<RuntimeDataBindGraphValue> {
        let solo = self
            .solos
            .iter()
            .find(|solo| solo.local_id == target_local_id)?;
        let active_component_id = usize::try_from(
            self.uint_property(target_local_id, solo.active_component_property_key)?,
        )
        .ok()?;
        let active_local_id = solo
            .runtime_local_by_cpp_local
            .get(&active_component_id)
            .copied()?;
        let active_name = self
            .slot(active_local_id)
            .and_then(|slot| slot.name.as_deref())?
            .as_bytes();
        let index = enum_value_names
            .iter()
            .position(|name| name.as_slice() == active_name)?;
        Some(RuntimeDataBindGraphValue::Enum(u64::try_from(index).ok()?))
    }

    fn apply_artboard_property_binding_value(
        &mut self,
        target_local_id: usize,
        property_key: u16,
        value: &RuntimeDataBindGraphValue,
    ) -> bool {
        match (
            self.objects.property_kind(target_local_id, property_key),
            Some(value.clone()),
        ) {
            (Some(FieldKind::Double), Some(RuntimeDataBindGraphValue::Number(value))) => {
                self.set_double_property(target_local_id, property_key, value)
            }
            (Some(FieldKind::Uint), Some(RuntimeDataBindGraphValue::Number(value))) => {
                let rounded = if value < 0.0 { 0 } else { value.round() as u64 };
                self.set_uint_property(target_local_id, property_key, rounded)
            }
            (Some(FieldKind::Uint), Some(RuntimeDataBindGraphValue::Enum(value))) => {
                self.set_uint_property(target_local_id, property_key, value)
            }
            (Some(FieldKind::Uint), Some(RuntimeDataBindGraphValue::ViewModel(value))) => self
                .view_model_instance_index_for_target_pointer(target_local_id, value)
                .is_some_and(|value| self.set_uint_property(target_local_id, property_key, value)),
            (Some(FieldKind::Bool), Some(RuntimeDataBindGraphValue::Boolean(value))) => {
                self.set_bool_property(target_local_id, property_key, value)
            }
            (Some(FieldKind::Color), Some(RuntimeDataBindGraphValue::Color(value))) => {
                // Mirrors C++ src/data_bind/context/context_value_color.cpp.
                self.set_color_property(target_local_id, property_key, value)
            }
            (Some(FieldKind::String), Some(RuntimeDataBindGraphValue::String(value))) => {
                self.set_string_property(target_local_id, property_key, value)
            }
            _ => false,
        }
    }

    fn view_model_instance_index_for_target_pointer(
        &self,
        target_local_id: usize,
        value: RuntimeViewModelPointer,
    ) -> Option<u64> {
        let RuntimeViewModelPointer::Imported { object_id } = value else {
            return None;
        };
        if self.slot(target_local_id)?.type_name != Some("ViewModelInstanceViewModel") {
            return None;
        }
        let parent_key = runtime_data_bind_component_parent_id_key()?;
        let property_id_key = runtime_data_bind_view_model_instance_value_property_id_key()?;
        let view_model_id_key = runtime_data_bind_view_model_instance_view_model_id_key()?;
        let parent_local_id =
            usize::try_from(self.uint_property(target_local_id, parent_key)?).ok()?;
        let property_id =
            usize::try_from(self.uint_property(target_local_id, property_id_key)?).ok()?;
        let view_model_index =
            usize::try_from(self.uint_property(parent_local_id, view_model_id_key)?).ok()?;
        let file = self.runtime_file()?;
        let view_model = file.view_model(view_model_index)?;
        let property = view_model.properties.get(property_id)?;
        if property.type_name != "ViewModelPropertyViewModel" {
            return None;
        }
        let referenced_view_model_index =
            usize::try_from(property.uint_property("viewModelReferenceId")?).ok()?;
        file.view_model(referenced_view_model_index)?
            .instances
            .iter()
            .position(|instance| instance.object.id == object_id)
            .and_then(|index| u64::try_from(index).ok())
    }

    fn update_artboard_custom_property_binding(&mut self, index: usize) -> bool {
        let Some((data_bind_index, path_is_name_based, owned_context_source, retained_source_path)) =
            self.artboard_custom_property_bindings
                .get(index)
                .map(|binding| {
                    (
                        binding.data_bind_index,
                        binding.path_is_name_based,
                        binding.owned_context_source.clone(),
                        binding.owned_context_source_path.clone(),
                    )
                })
        else {
            return false;
        };
        let Some(target_value) = self.artboard_custom_property_binding_target_value(index) else {
            return false;
        };
        let Some((path, value)) =
            self.convert_artboard_custom_property_binding_target_value(index, &target_value)
        else {
            return false;
        };
        let context_changed = if self.artboard_owned_view_model_handle.is_some() {
            let (retained_changed, resolved_source_path) = self
                .write_retained_owned_view_model_artboard_source(
                    path.as_ref(),
                    path_is_name_based,
                    retained_source_path.as_deref(),
                    &value,
                );
            if let Some(binding) = self.artboard_custom_property_bindings.get_mut(index) {
                binding.owned_context_source_path = resolved_source_path;
            }
            retained_changed
        } else if let Some(source) = owned_context_source {
            source.sync_value(&value)
        } else {
            self.runtime_file_arc()
                .and_then(|file| {
                    runtime_owned_view_model_sync_value_for_candidates(
                        &file,
                        &self.artboard_owned_view_model_candidates,
                        path.as_ref(),
                        path_is_name_based,
                        &value,
                    )
                })
                .unwrap_or(false)
        };
        self.set_artboard_data_bind_value_for_path_with_suppressed_data_bind(
            path.as_ref(),
            value,
            Some(data_bind_index),
        ) || context_changed
    }

    fn artboard_custom_property_binding_target_value(
        &self,
        index: usize,
    ) -> Option<RuntimeDataBindGraphValue> {
        let binding = self.artboard_custom_property_bindings.get(index)?;
        match binding.value_kind {
            RuntimeArtboardDataBindValueKind::Number => self
                .double_property(binding.target_local_id, binding.property_key)
                .map(RuntimeDataBindGraphValue::Number),
            RuntimeArtboardDataBindValueKind::Boolean => self
                .bool_property(binding.target_local_id, binding.property_key)
                .map(RuntimeDataBindGraphValue::Boolean),
            RuntimeArtboardDataBindValueKind::String => self
                .string_property(binding.target_local_id, binding.property_key)
                .map(|value| RuntimeDataBindGraphValue::String(value.to_vec())),
            RuntimeArtboardDataBindValueKind::Color => self
                .color_property(binding.target_local_id, binding.property_key)
                .map(RuntimeDataBindGraphValue::Color),
            RuntimeArtboardDataBindValueKind::Enum => self
                .uint_property(binding.target_local_id, binding.property_key)
                .map(RuntimeDataBindGraphValue::Enum),
            RuntimeArtboardDataBindValueKind::Trigger => self
                .uint_property(binding.target_local_id, binding.property_key)
                .map(RuntimeDataBindGraphValue::Trigger),
        }
    }

    fn convert_artboard_custom_property_binding_target_value(
        &mut self,
        index: usize,
        value: &RuntimeDataBindGraphValue,
    ) -> Option<(Arc<[u32]>, RuntimeDataBindGraphValue)> {
        let binding = self.artboard_custom_property_bindings.get_mut(index)?;
        let shared = self
            .artboard_shared_data_bind_converter_states
            .get_mut(&binding.data_bind_index);
        let converted = match (shared, binding.converter.as_ref()) {
            (Some(shared), _) if binding.flags & DATA_BIND_FLAG_DIRECTION_TO_SOURCE != 0 => {
                shared.converter_state.convert_value_with_formula_randoms(
                    &shared.converter,
                    value,
                    &mut self.artboard_formula_random_source,
                )?
            }
            (Some(shared), _) => shared
                .converter_state
                .reverse_convert_value_with_formula_randoms(
                    &shared.converter,
                    value,
                    &mut self.artboard_formula_random_source,
                )?,
            (None, None) => value.clone(),
            (None, Some(converter)) if binding.flags & DATA_BIND_FLAG_DIRECTION_TO_SOURCE != 0 => {
                binding.converter_state.convert_value_with_formula_randoms(
                    converter,
                    value,
                    &mut self.artboard_formula_random_source,
                )?
            }
            (None, Some(converter)) => binding
                .converter_state
                .reverse_convert_value_with_formula_randoms(
                    converter,
                    value,
                    &mut self.artboard_formula_random_source,
                )?,
        };
        artboard_data_bind_values_have_same_kind(&binding.default_value, &converted)
            .then(|| (binding.path.clone(), converted))
    }

    fn apply_artboard_solo_bindings(&mut self) -> bool {
        enum RuntimeSoloBindingApply {
            Index(f32),
            Name(Vec<u8>),
        }

        let mut changed = false;
        for index in 0..self.artboard_solo_bindings.len() {
            let Some((target_local_id, apply)) =
                self.artboard_solo_bindings.get(index).and_then(|binding| {
                    let value = self.artboard_data_bind_values.get(&binding.path)?;
                    let apply = match value {
                        RuntimeDataBindGraphValue::Number(value) => {
                            RuntimeSoloBindingApply::Index(*value)
                        }
                        RuntimeDataBindGraphValue::String(value) => {
                            RuntimeSoloBindingApply::Name(value.clone())
                        }
                        RuntimeDataBindGraphValue::Enum(value) => {
                            let value = usize::try_from(*value).ok()?;
                            RuntimeSoloBindingApply::Name(
                                binding.enum_value_names.get(value)?.clone(),
                            )
                        }
                        _ => return None,
                    };
                    Some((binding.target_local_id, apply))
                })
            else {
                continue;
            };
            changed |= match apply {
                RuntimeSoloBindingApply::Index(value) => {
                    self.set_solo_active_child_by_index(target_local_id, value)
                }
                RuntimeSoloBindingApply::Name(value) => {
                    self.set_solo_active_child_by_name(target_local_id, &value)
                }
            }
        }
        changed
    }

    fn apply_artboard_nested_host_bindings(&mut self) -> bool {
        let mut changed = false;
        for index in 0..self.artboard_nested_host_bindings.len() {
            let Some((target_local_id, property, value, first_artboard_apply)) = self
                .artboard_nested_host_bindings
                .get_mut(index)
                .and_then(|binding| {
                    let value = self.artboard_data_bind_values.get(&binding.path)?.clone();
                    let first_artboard_apply =
                        matches!(
                            (binding.property, &value),
                            (
                                RuntimeArtboardNestedHostProperty::ArtboardId { .. },
                                RuntimeDataBindGraphValue::Artboard(_)
                            )
                        ) && !std::mem::replace(&mut binding.artboard_value_applied, true);
                    Some((
                        binding.target_local_id,
                        binding.property,
                        value,
                        first_artboard_apply,
                    ))
                })
            else {
                continue;
            };
            changed |= self.apply_artboard_nested_host_binding_value(
                target_local_id,
                property,
                &value,
                first_artboard_apply,
            );
        }
        changed
    }

    pub(crate) fn nested_artboard_host_has_artboard_data_bind(
        &self,
        target_local_id: usize,
    ) -> bool {
        self.artboard_nested_host_bindings.iter().any(|binding| {
            binding.target_local_id == target_local_id
                && matches!(
                    binding.property,
                    RuntimeArtboardNestedHostProperty::ArtboardId { .. }
                )
        })
    }

    fn apply_artboard_nested_host_binding_value(
        &mut self,
        target_local_id: usize,
        property: RuntimeArtboardNestedHostProperty,
        value: &RuntimeDataBindGraphValue,
        first_artboard_apply: bool,
    ) -> bool {
        match (property, value) {
            (
                RuntimeArtboardNestedHostProperty::ArtboardId { property_key },
                RuntimeDataBindGraphValue::Artboard(value),
            ) => {
                let property_changed =
                    self.set_uint_property(target_local_id, property_key, *value);
                let artboard_changed = if first_artboard_apply && !property_changed {
                    self.replace_nested_artboard_artboard_id(target_local_id, *value)
                } else {
                    self.set_nested_artboard_artboard_id(target_local_id, *value)
                };
                property_changed || artboard_changed
            }
            (
                RuntimeArtboardNestedHostProperty::IsPaused { property_key },
                RuntimeDataBindGraphValue::Boolean(value),
            ) => self.set_bool_property(target_local_id, property_key, *value),
            (
                RuntimeArtboardNestedHostProperty::Speed { property_key },
                RuntimeDataBindGraphValue::Number(value),
            )
            | (
                RuntimeArtboardNestedHostProperty::Quantize { property_key },
                RuntimeDataBindGraphValue::Number(value),
            ) => self.set_double_property(target_local_id, property_key, *value),
            _ => false,
        }
    }

    fn sync_nested_child_artboard_data_contexts(&mut self) -> bool {
        let mut changed = self.sync_stateful_nested_view_model_contexts();
        let mut updates = std::mem::take(&mut self.artboard_nested_child_context_updates_scratch);
        for index in 0..self.nested_artboard_locals.len() {
            updates.clear();
            let host_local_id = self.nested_artboard_locals[index];
            let Some(nested) = self.nested_artboards.get(&host_local_id) else {
                continue;
            };
            if nested.child.artboard_property_bindings.is_empty()
                && nested.child.artboard_image_asset_bindings.is_empty()
                && nested.child.artboard_formula_token_bindings.is_empty()
                && nested.child.artboard_converter_property_bindings.is_empty()
            {
                continue;
            }
            for (index, binding) in nested.child.artboard_property_bindings.iter().enumerate() {
                let source_local = nested
                    .data_bind_property_source_locals
                    .get(index)
                    .copied()
                    .flatten();
                let value = self
                    .artboard_custom_property_bindings
                    .iter()
                    .any(|source| source.path.as_ref() == binding.path.as_slice())
                    .then(|| {
                        self.artboard_data_bind_values
                            .get(binding.path.as_slice())
                            .cloned()
                    })
                    .flatten()
                    .or_else(|| {
                        source_local.and_then(|source_local| {
                            self.stateful_nested_host_binding_value_for_local(
                                source_local,
                                &binding.default_value,
                            )
                        })
                    });
                if let Some(value) = value
                    && nested
                        .child
                        .artboard_data_bind_values
                        .get(binding.path.as_slice())
                        != Some(&value)
                {
                    updates.push(RuntimeNestedChildContextUpdate::Property(index, value));
                }
            }
            for (index, binding) in nested
                .child
                .artboard_image_asset_bindings
                .iter()
                .enumerate()
            {
                let Some(source_local) = nested
                    .data_bind_image_source_locals
                    .get(index)
                    .copied()
                    .flatten()
                else {
                    continue;
                };
                if let Some(value) = self.stateful_nested_host_binding_value_for_local(
                    source_local,
                    &binding.default_value,
                ) && nested
                    .child
                    .artboard_data_bind_values
                    .get(binding.path.as_slice())
                    != Some(&value)
                {
                    updates.push(RuntimeNestedChildContextUpdate::ImageAsset(index, value));
                }
            }
            for path in nested
                .child
                .artboard_formula_token_bindings
                .iter()
                .filter(|binding| binding.artboard_converter_reachable)
                .map(|binding| (binding.path.as_slice(), &binding.default_value))
                .chain(
                    nested
                        .child
                        .artboard_converter_property_bindings
                        .iter()
                        .map(|binding| (binding.path.as_slice(), &binding.default_value)),
                )
            {
                let (path, default_value) = path;
                let value = nested
                    .data_bind_context_source_locals_by_path
                    .get(path)
                    .copied()
                    .and_then(|source_local| {
                        self.stateful_nested_host_binding_value_for_local(
                            source_local,
                            default_value,
                        )
                    })
                    .or_else(|| self.artboard_data_bind_values.get(path).cloned());
                let Some(value) = value else {
                    continue;
                };
                if nested.child.artboard_data_bind_values.get(path) != Some(&value) {
                    updates.push(RuntimeNestedChildContextUpdate::ContextPath(
                        path.to_vec(),
                        value,
                    ));
                }
            }
            if updates.is_empty() {
                continue;
            };
            let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
                continue;
            };
            let mut child_context_changed = false;
            for update in updates.drain(..) {
                let (path, value) = match update {
                    RuntimeNestedChildContextUpdate::Property(index, value) => {
                        let Some(binding) = nested.child.artboard_property_bindings.get(index)
                        else {
                            continue;
                        };
                        (binding.path.clone(), value)
                    }
                    RuntimeNestedChildContextUpdate::ImageAsset(index, value) => {
                        let Some(binding) = nested.child.artboard_image_asset_bindings.get(index)
                        else {
                            continue;
                        };
                        (binding.path.clone(), value)
                    }
                    RuntimeNestedChildContextUpdate::ContextPath(path, value) => (path, value),
                };
                child_context_changed |= nested
                    .child
                    .set_artboard_data_bind_value_for_path(&path, value);
            }
            if child_context_changed {
                changed = true;
                changed |= nested.child.advance_artboard_data_binds();
                nested.child.update_pass();
            }
        }
        self.artboard_nested_child_context_updates_scratch = updates;
        changed
    }

    fn sync_stateful_nested_view_model_contexts(&mut self) -> bool {
        if !std::mem::replace(&mut self.stateful_nested_view_model_contexts_dirty, false) {
            return false;
        }
        let Some(parent_key) = runtime_data_bind_component_parent_id_key() else {
            return false;
        };
        let Some(property_id_key) = runtime_data_bind_view_model_instance_value_property_id_key()
        else {
            return false;
        };

        // C++ NestedArtboard retains the authored ViewModelInstance children
        // by pointer, so keyframes and data binds on those children are visible
        // to the mounted artboard immediately. Rust owns a detached context;
        // reconcile every supported live value before rebinding descendants.
        let mut updates = BTreeMap::<usize, Vec<RuntimeStatefulViewModelUpdate>>::new();
        for host_local_id in &self.nested_artboard_locals {
            let Some(nested) = self.nested_artboards.get(host_local_id) else {
                continue;
            };
            let roots_by_local = nested
                .stateful_view_model_instance_locals_by_id
                .iter()
                .filter_map(|(&view_model_id, &instance_local_id)| {
                    usize::try_from(view_model_id)
                        .ok()
                        .map(|view_model_index| (instance_local_id, view_model_index))
                })
                .collect::<BTreeMap<_, _>>();
            for slot in &self.slots {
                let Some(type_name) = slot.type_name else {
                    continue;
                };
                if !type_name.starts_with("ViewModelInstance") || type_name == "ViewModelInstance" {
                    continue;
                }

                let mut property_path = Vec::new();
                let mut current_local = slot.local_id;
                let mut visited = BTreeSet::new();
                let (instance_local_id, view_model_index) = loop {
                    if !visited.insert(current_local) {
                        break (usize::MAX, usize::MAX);
                    }
                    let Some(property_index) = self
                        .uint_property(current_local, property_id_key)
                        .and_then(|value| usize::try_from(value).ok())
                    else {
                        break (usize::MAX, usize::MAX);
                    };
                    property_path.push(property_index);
                    let Some(parent_local) = self
                        .uint_property(current_local, parent_key)
                        .and_then(|value| usize::try_from(value).ok())
                    else {
                        break (usize::MAX, usize::MAX);
                    };
                    if let Some(&view_model_index) = roots_by_local.get(&parent_local) {
                        break (parent_local, view_model_index);
                    }
                    current_local = parent_local;
                };
                if instance_local_id == usize::MAX {
                    continue;
                }
                property_path.reverse();

                let Some(value_key) =
                    runtime_data_bind_property_key_for_name(type_name, "propertyValue")
                else {
                    continue;
                };
                let value = match type_name {
                    "ViewModelInstanceNumber" => self
                        .double_property(slot.local_id, value_key)
                        .map(RuntimeDataBindGraphValue::Number)
                        .map(RuntimeStatefulViewModelValueUpdate::Value),
                    "ViewModelInstanceBoolean" => self
                        .bool_property(slot.local_id, value_key)
                        .map(RuntimeDataBindGraphValue::Boolean)
                        .map(RuntimeStatefulViewModelValueUpdate::Value),
                    "ViewModelInstanceString" => self
                        .string_property(slot.local_id, value_key)
                        .map(|value| RuntimeDataBindGraphValue::String(value.to_vec()))
                        .map(RuntimeStatefulViewModelValueUpdate::Value),
                    "ViewModelInstanceColor" => self
                        .color_property(slot.local_id, value_key)
                        .map(RuntimeDataBindGraphValue::Color)
                        .map(RuntimeStatefulViewModelValueUpdate::Value),
                    "ViewModelInstanceEnum" => self
                        .uint_property(slot.local_id, value_key)
                        .map(RuntimeDataBindGraphValue::Enum)
                        .map(RuntimeStatefulViewModelValueUpdate::Value),
                    "ViewModelInstanceSymbolListIndex" => self
                        .uint_property(slot.local_id, value_key)
                        .map(RuntimeDataBindGraphValue::SymbolListIndex)
                        .map(RuntimeStatefulViewModelValueUpdate::Value),
                    "ViewModelInstanceAssetImage" => self
                        .uint_property(slot.local_id, value_key)
                        .map(RuntimeDataBindGraphValue::Asset)
                        .map(RuntimeStatefulViewModelValueUpdate::Value),
                    "ViewModelInstanceAssetFont" => self
                        .uint_property(slot.local_id, value_key)
                        .map(RuntimeStatefulViewModelValueUpdate::FontAsset),
                    "ViewModelInstanceArtboard" => self
                        .uint_property(slot.local_id, value_key)
                        .map(RuntimeDataBindGraphValue::Artboard)
                        .map(RuntimeStatefulViewModelValueUpdate::Value),
                    "ViewModelInstanceTrigger" => self
                        .uint_property(slot.local_id, value_key)
                        .map(RuntimeDataBindGraphValue::Trigger)
                        .map(RuntimeStatefulViewModelValueUpdate::Value),
                    "ViewModelInstanceViewModel" => self
                        .uint_property(slot.local_id, value_key)
                        .and_then(|value| usize::try_from(value).ok())
                        .map(RuntimeStatefulViewModelValueUpdate::ViewModelInstance),
                    _ => None,
                };
                let Some(value) = value else { continue };
                updates
                    .entry(*host_local_id)
                    .or_default()
                    .push(RuntimeStatefulViewModelUpdate {
                        instance_local_id,
                        view_model_index,
                        property_path,
                        value,
                    });
            }
        }

        let mut changed = false;
        let Some(file) = self.runtime_file_arc() else {
            return false;
        };
        let parent_candidates = self.artboard_owned_view_model_candidates.clone();
        for (host_local_id, mut updates) in updates {
            // A parent ViewModel selection must be applied before values below
            // that pointer are reconciled into its newly active instance.
            updates.sort_by_key(|update| update.property_path.len());
            let inherited_candidates = self.owned_view_model_context_candidates_for_nested_host(
                &file,
                &parent_candidates,
                host_local_id,
                true,
            );
            let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
                continue;
            };
            let mut context_changed = false;
            for update in updates {
                let context = if nested.stateful_view_model_instance_local
                    == Some(update.instance_local_id)
                {
                    nested.stateful_view_model_context.as_mut()
                } else {
                    nested
                        .stateful_global_view_model_contexts
                        .get_mut(&update.view_model_index)
                };
                let Some(context) = context else { continue };
                let value_changed = match update.value {
                    RuntimeStatefulViewModelValueUpdate::Value(
                        RuntimeDataBindGraphValue::Number(value),
                    ) => context.sync_number_by_property_path(&update.property_path, value),
                    RuntimeStatefulViewModelValueUpdate::Value(
                        RuntimeDataBindGraphValue::Boolean(value),
                    ) => context.sync_boolean_by_property_path(&update.property_path, value),
                    RuntimeStatefulViewModelValueUpdate::Value(
                        RuntimeDataBindGraphValue::String(value),
                    ) => context.sync_string_by_property_path(&update.property_path, &value),
                    RuntimeStatefulViewModelValueUpdate::Value(
                        RuntimeDataBindGraphValue::Color(value),
                    ) => context.sync_color_by_property_path(&update.property_path, value),
                    RuntimeStatefulViewModelValueUpdate::Value(
                        RuntimeDataBindGraphValue::Enum(value),
                    ) => context.sync_enum_by_property_path(&update.property_path, value),
                    RuntimeStatefulViewModelValueUpdate::Value(
                        RuntimeDataBindGraphValue::SymbolListIndex(value),
                    ) => context
                        .sync_symbol_list_index_by_property_path(&update.property_path, value),
                    RuntimeStatefulViewModelValueUpdate::Value(
                        RuntimeDataBindGraphValue::Asset(value),
                    ) => context.sync_asset_by_property_path(&update.property_path, value),
                    RuntimeStatefulViewModelValueUpdate::FontAsset(value) => {
                        context.sync_font_asset_index_by_property_path(&update.property_path, value)
                    }
                    RuntimeStatefulViewModelValueUpdate::Value(
                        RuntimeDataBindGraphValue::Artboard(value),
                    ) => context.sync_artboard_by_property_path(&update.property_path, value),
                    RuntimeStatefulViewModelValueUpdate::Value(
                        RuntimeDataBindGraphValue::Trigger(value),
                    ) => context.sync_trigger_by_property_path(&update.property_path, value),
                    RuntimeStatefulViewModelValueUpdate::ViewModelInstance(instance_index) => {
                        context
                            .set_view_model_by_property_path(&update.property_path, instance_index)
                    }
                    RuntimeStatefulViewModelValueUpdate::Value(_) => false,
                };
                context_changed |= value_changed;
            }
            if !context_changed {
                continue;
            }

            let mut nested_candidates = Vec::new();
            if let Some(context) = nested.stateful_view_model_context.clone() {
                nested_candidates.push(RuntimeOwnedViewModelBindingCandidate::root(&context));
            }
            nested_candidates.extend(
                nested
                    .stateful_global_view_model_contexts
                    .values()
                    .map(RuntimeOwnedViewModelBindingCandidate::root),
            );
            nested_candidates.extend(inherited_candidates);
            changed = true;
            changed |= nested
                .bind_owned_view_model_animation_context_candidates(&file, &nested_candidates);
            changed |= nested
                .child
                .bind_owned_view_model_artboard_context_candidates(
                    &file,
                    &nested_candidates,
                    true,
                    true,
                );
            // C++ retains authored view-model instances by pointer, so the
            // mounted child observes the new value during the same host-first
            // updateDataBinds pass. Rust rebinds a detached owned snapshot;
            // flush its queued bindings now, before the parent measures a
            // hug-sized NestedArtboardLayout from the child.
            changed |= nested.child.advance_artboard_data_binds();
            changed |= nested.child.update_pass();
        }
        if changed {
            // The mounted child can be a hug-sized provider in this artboard's
            // layout tree, so its live text/shape size participates in the
            // parent's layout cache key.
            self.mark_layout_changed();
        }
        changed
    }

    fn stateful_nested_host_binding_value_for_local(
        &self,
        source_local: usize,
        default_value: &RuntimeDataBindGraphValue,
    ) -> Option<RuntimeDataBindGraphValue> {
        match default_value {
            RuntimeDataBindGraphValue::Number(_) => {
                let property_value_key = runtime_data_bind_view_model_instance_number_value_key()?;
                self.double_property(source_local, property_value_key)
                    .map(RuntimeDataBindGraphValue::Number)
            }
            RuntimeDataBindGraphValue::String(_) => {
                let property_value_key = runtime_data_bind_view_model_instance_string_value_key()?;
                self.string_property(source_local, property_value_key)
                    .map(|value| RuntimeDataBindGraphValue::String(value.to_vec()))
            }
            RuntimeDataBindGraphValue::Color(_) => {
                let property_value_key = runtime_data_bind_view_model_instance_color_value_key()?;
                self.color_property(source_local, property_value_key)
                    .map(RuntimeDataBindGraphValue::Color)
            }
            RuntimeDataBindGraphValue::Boolean(_) => {
                let property_value_key = runtime_data_bind_view_model_instance_boolean_value_key()?;
                self.bool_property(source_local, property_value_key)
                    .map(RuntimeDataBindGraphValue::Boolean)
            }
            RuntimeDataBindGraphValue::Enum(_) => {
                let property_value_key = runtime_data_bind_view_model_instance_enum_value_key()?;
                self.uint_property(source_local, property_value_key)
                    .map(RuntimeDataBindGraphValue::Enum)
            }
            RuntimeDataBindGraphValue::Asset(_) => {
                let property_value_key = if self.slot(source_local).and_then(|slot| slot.type_name)
                    == Some("ViewModelInstanceAssetFont")
                {
                    runtime_data_bind_view_model_instance_font_asset_value_key()?
                } else {
                    runtime_data_bind_view_model_instance_asset_value_key()?
                };
                self.uint_property(source_local, property_value_key)
                    .map(RuntimeDataBindGraphValue::Asset)
            }
            _ => None,
        }
    }

    pub fn artboard_list_binding_source_list_size_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        self.artboard_list_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| binding.source_list_size)
    }

    pub fn artboard_list_binding_source_number_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<f32> {
        self.artboard_list_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| binding.source_number_value)
    }

    pub fn artboard_list_binding_target_list_size_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        self.artboard_list_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| binding.target_list_size)
    }

    pub fn artboard_list_binding_target_local_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        self.artboard_list_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.target_local_id)
    }

    pub fn artboard_list_binding_should_reset_instances_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<bool> {
        self.artboard_list_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.should_reset_instances)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn scripted_converter_defers_default_output_kind_validation() {
        let converter = RuntimeDataBindGraphConverter::Scripted {
            global_id: 42,
            instance: None,
        };
        let numeric_default = RuntimeDataBindGraphValue::Number(10_001.0);

        assert!(!artboard_property_binding_value_matches_kind(
            &numeric_default,
            FieldKind::String
        ));
        assert!(artboard_property_binding_accepts_default(
            Some(&converter),
            &numeric_default,
            FieldKind::String
        ));
    }

    fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
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

    fn list_binding_fixture() -> RuntimeFile {
        RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Items".to_owned()),
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
        ])
        .expect("list binding fixture imports")
    }

    fn font_binding_fixture() -> RuntimeFile {
        RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record(
                "FontAsset",
                vec![property("FontAsset", "assetId", AuthoringValue::Uint(7))],
            ),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Model".to_owned()),
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
                "ViewModelInstance",
                vec![property(
                    "ViewModelInstance",
                    "viewModelId",
                    AuthoringValue::Uint(0),
                )],
            ),
            record(
                "ViewModelInstanceAssetFont",
                vec![
                    property(
                        "ViewModelInstanceAssetFont",
                        "parentId",
                        AuthoringValue::Uint(0),
                    ),
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
                "Artboard",
                vec![
                    property("Artboard", "width", AuthoringValue::Double(100.0)),
                    property("Artboard", "height", AuthoringValue::Double(100.0)),
                    property("Artboard", "viewModelId", AuthoringValue::Uint(0)),
                ],
            ),
            record(
                "Text",
                vec![property("Text", "parentId", AuthoringValue::Uint(0))],
            ),
            record(
                "TextStylePaint",
                vec![
                    property("TextStylePaint", "parentId", AuthoringValue::Uint(1)),
                    property("TextStylePaint", "fontAssetId", AuthoringValue::Uint(0)),
                ],
            ),
            record(
                "DataBindContext",
                vec![
                    property(
                        "DataBindContext",
                        "propertyKey",
                        AuthoringValue::Uint(u64::from(
                            property_key_for_name("TextStyle", "fontAssetId")
                                .expect("fontAssetId key"),
                        )),
                    ),
                    property(
                        "DataBindContext",
                        "sourcePathIds",
                        AuthoringValue::Bytes(vec![0, 0]),
                    ),
                ],
            ),
        ])
        .expect("font binding fixture imports")
    }

    fn shape_length_binding_fixture() -> RuntimeFile {
        let shape_length_key =
            property_key_for_name("Shape", "length").expect("shape length property key");
        RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Model".to_owned()),
                )],
            ),
            record(
                "ViewModelPropertyNumber",
                vec![property(
                    "ViewModelPropertyNumber",
                    "name",
                    AuthoringValue::String("length".to_owned()),
                )],
            ),
            record(
                "Artboard",
                vec![property("Artboard", "viewModelId", AuthoringValue::Uint(0))],
            ),
            record(
                "Shape",
                vec![
                    property("Shape", "parentId", AuthoringValue::Uint(0)),
                    property("Shape", "scaleX", AuthoringValue::Double(2.0)),
                    property("Shape", "scaleY", AuthoringValue::Double(3.0)),
                ],
            ),
            record(
                "DataBindContext",
                vec![
                    property(
                        "DataBindContext",
                        "propertyKey",
                        AuthoringValue::Uint(u64::from(shape_length_key)),
                    ),
                    property(
                        "DataBindContext",
                        "sourcePathIds",
                        AuthoringValue::Bytes(vec![0, 0]),
                    ),
                    property("DataBindContext", "flags", AuthoringValue::Uint(1)),
                ],
            ),
            record(
                "Rectangle",
                vec![
                    property("Rectangle", "parentId", AuthoringValue::Uint(1)),
                    property("Rectangle", "width", AuthoringValue::Double(10.0)),
                    property("Rectangle", "height", AuthoringValue::Double(20.0)),
                ],
            ),
        ])
        .expect("shape length binding fixture imports")
    }

    #[test]
    fn update_pass_repolls_shape_length_after_components_settle() {
        let file = shape_length_binding_fixture();
        let graphs =
            nuxie_graph::GraphFile::from_runtime_file(&file).expect("shape length graph builds");
        let graph = graphs.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, graph).expect("artboard builds");

        assert!(artboard.update_pass());
        assert_eq!(
            artboard.artboard_data_bind_values.get(&[0_u32, 0][..]),
            Some(&RuntimeDataBindGraphValue::Number(160.0)),
            "C++ polls computed target-to-source bindings again after component transforms settle"
        );

        let scale_x_key = property_key_for_name("Shape", "scaleX").expect("shape scaleX key");
        assert!(artboard.set_double_property(1, scale_x_key, 4.0));
        assert!(artboard.update_pass());
        assert_eq!(
            artboard.artboard_data_bind_values.get(&[0_u32, 0][..]),
            Some(&RuntimeDataBindGraphValue::Number(200.0)),
            "the clean-frame epoch guard must not suppress C++'s post-component derived-value poll"
        );
    }

    #[test]
    fn isolated_paint_binding_evaluator_does_not_publish_to_live_owned_context() {
        let file = shape_length_binding_fixture();
        let graphs =
            nuxie_graph::GraphFile::from_runtime_file(&file).expect("shape length graph builds");
        let graph = graphs.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, graph).expect("artboard builds");
        let context = RuntimeOwnedViewModelContext::from_main(
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("owned context"),
        );

        artboard.bind_owned_view_model_artboard_contexts(&file, &context);
        assert_eq!(artboard.artboard_owned_view_model_candidates.len(), 1);
        assert!(artboard.update_pass());
        let live_value = context
            .main()
            .and_then(|main| main.number_value_by_property_name("length"))
            .expect("published live shape length");
        let live_generation = context.main().expect("live context").mutation_generation();

        let mut evaluator = artboard.clone();
        evaluator.detach_initial_nested_layout_paint_binding_contexts();
        let scale_x_key = property_key_for_name("Shape", "scaleX").expect("shape scaleX key");
        assert!(evaluator.set_double_property(1, scale_x_key, 4.0));
        assert!(evaluator.update_pass());

        assert_eq!(
            context
                .main()
                .and_then(|main| main.number_value_by_property_name("length")),
            Some(live_value)
        );
        assert_eq!(
            context.main().expect("live context").mutation_generation(),
            live_generation,
            "isolated full data-bind evaluation must not publish into the mounted occurrence"
        );
        assert!(
            evaluator.artboard_owned_view_model_candidates[0]
                .context
                .borrow()
                .number_value_by_property_name("length")
                .is_some_and(|value| value > live_value),
            "the detached candidate still receives the evaluator's computed result"
        );
    }

    fn same_artboard_binding_fixture() -> RuntimeFile {
        RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record(
                "ViewModel",
                vec![property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Model".to_owned()),
                )],
            ),
            record(
                "ViewModelPropertyArtboard",
                vec![property(
                    "ViewModelPropertyArtboard",
                    "name",
                    AuthoringValue::String("child".to_owned()),
                )],
            ),
            record(
                "ViewModelInstance",
                vec![property(
                    "ViewModelInstance",
                    "viewModelId",
                    AuthoringValue::Uint(0),
                )],
            ),
            record(
                "ViewModelInstanceArtboard",
                vec![
                    property(
                        "ViewModelInstanceArtboard",
                        "parentId",
                        AuthoringValue::Uint(0),
                    ),
                    property(
                        "ViewModelInstanceArtboard",
                        "viewModelPropertyId",
                        AuthoringValue::Uint(0),
                    ),
                    property(
                        "ViewModelInstanceArtboard",
                        "propertyValue",
                        AuthoringValue::Uint(1),
                    ),
                ],
            ),
            record(
                "Artboard",
                vec![
                    property("Artboard", "width", AuthoringValue::Double(100.0)),
                    property("Artboard", "height", AuthoringValue::Double(100.0)),
                    property("Artboard", "viewModelId", AuthoringValue::Uint(0)),
                ],
            ),
            record(
                "NestedArtboard",
                vec![
                    property("NestedArtboard", "parentId", AuthoringValue::Uint(0)),
                    property("NestedArtboard", "artboardId", AuthoringValue::Uint(1)),
                ],
            ),
            record(
                "DataBindContext",
                vec![
                    property(
                        "DataBindContext",
                        "propertyKey",
                        AuthoringValue::Uint(u64::from(
                            property_key_for_name("NestedArtboard", "artboardId")
                                .expect("artboardId key"),
                        )),
                    ),
                    property(
                        "DataBindContext",
                        "sourcePathIds",
                        AuthoringValue::Bytes(vec![0, 0]),
                    ),
                ],
            ),
            record(
                "Artboard",
                vec![
                    property("Artboard", "width", AuthoringValue::Double(50.0)),
                    property("Artboard", "height", AuthoringValue::Double(50.0)),
                ],
            ),
        ])
        .expect("same-artboard binding fixture imports")
    }

    #[test]
    fn first_artboard_binding_recreates_an_equal_authored_nested_source() {
        let file = same_artboard_binding_fixture();
        let graphs =
            nuxie_graph::GraphFile::from_runtime_file(&file).expect("fixture graph builds");
        let graph = graphs
            .artboards
            .first()
            .expect("fixture has a parent artboard");
        let mut artboard =
            ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
                .expect("parent artboard builds");
        let host_local = graph.nested_artboards[0].local_id;
        let original_identity = artboard.nested_artboards[&host_local]
            .child
            .instance_identity();
        let context = RuntimeOwnedViewModelInstance::from_instance(&file, 0, 0)
            .expect("serialized view model instance builds");

        assert!(artboard.bind_owned_view_model_artboard_context(&file, &context));
        assert!(artboard.advance_artboard_data_binds());

        let replacement = &artboard.nested_artboards[&host_local];
        assert_ne!(replacement.child.instance_identity(), original_identity);
        assert_eq!(replacement.render_cache_revision, 1);
    }

    #[test]
    fn font_binding_retains_live_font_value_and_applies_text_style_override() {
        let file = font_binding_fixture();
        let graphs =
            nuxie_graph::GraphFile::from_runtime_file(&file).expect("font binding graph builds");
        let graph = graphs.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, graph).expect("artboard builds");
        let mut context = RuntimeOwnedViewModelInstance::from_instance(&file, 0, 0)
            .expect("serialized view model instance builds");

        assert!(artboard.bind_owned_view_model_artboard_context(&file, &context));
        assert!(artboard.advance_artboard_data_binds());
        let serialized = artboard
            .text_style_font_override(2)
            .expect("serialized font override was applied");
        assert_eq!(serialized.file_asset_index(), 0);
        assert_eq!(serialized.live_font_bytes(), None);

        let live: Arc<[u8]> = vec![1, 2, 3, 4].into();
        assert!(context.set_live_font_bytes_by_property_name("font", Some(Arc::clone(&live))));
        assert!(artboard.bind_owned_view_model_artboard_context(&file, &context));
        assert!(artboard.advance_artboard_data_binds());
        let value = artboard
            .text_style_font_override(2)
            .expect("font override was applied to TextStylePaint");
        assert_eq!(
            value.file_asset_index(),
            RuntimeFontAssetValue::MISSING_FILE_ASSET_INDEX
        );
        assert_eq!(value.live_font_bytes(), Some(live.as_ref()));

        assert!(context.set_font_asset_index_by_property_name("font", 0));
        assert!(artboard.bind_owned_view_model_artboard_context(&file, &context));
        assert!(artboard.advance_artboard_data_binds());
        let file_backed = artboard
            .text_style_font_override(2)
            .expect("file-backed font override was reapplied");
        assert_eq!(file_backed.file_asset_index(), 0);
        assert_eq!(
            file_backed.live_font_bytes(),
            Some(live.as_ref()),
            "assigning a file index preserves the private live font, while resolution lets the file asset win"
        );
    }

    #[test]
    fn shared_owned_context_mutations_refresh_without_an_explicit_rebind() {
        let file = font_binding_fixture();
        let graphs =
            nuxie_graph::GraphFile::from_runtime_file(&file).expect("font binding graph builds");
        let graph = graphs.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, graph).expect("artboard builds");
        let context = RuntimeOwnedViewModelContext::from_main(
            RuntimeOwnedViewModelInstance::from_instance(&file, 0, 0)
                .expect("serialized view model instance builds"),
        );

        assert!(artboard.bind_owned_view_model_artboard_contexts(&file, &context));
        assert!(artboard.advance_artboard_data_binds());

        let live: Arc<[u8]> = vec![5, 6, 7, 8].into();
        assert!(
            context
                .main_mut()
                .expect("main view model remains shared")
                .set_live_font_bytes_by_property_name("font", Some(Arc::clone(&live)))
        );
        assert!(artboard.advance_artboard_data_binds());

        let refreshed = artboard
            .text_style_font_override(2)
            .expect("shared mutation refreshed the font override");
        assert_eq!(refreshed.live_font_bytes(), Some(live.as_ref()));
    }

    fn list_binding(
        converter: RuntimeDataBindGraphConverter,
    ) -> RuntimeArtboardListBindingInstance {
        RuntimeArtboardListBindingInstance {
            data_bind_index: 0,
            target_local_id: 1,
            path: vec![0, 0],
            converter: Some(converter),
            default_value: RuntimeDataBindGraphValue::List { item_count: 0 },
            source_value: None,
            source_list_size: None,
            source_number_value: None,
            target_list_size: None,
            should_reset_instances: false,
            generated_view_model_id: None,
            generated_items: Vec::new(),
        }
    }

    fn list_handle_with_items(
        file: &RuntimeFile,
        item_count: usize,
    ) -> RuntimeOwnedViewModelListHandle {
        let mut source = RuntimeOwnedViewModelInstance::new(file, 0).expect("source instance");
        for _ in 0..item_count {
            let item = Rc::new(RefCell::new(
                RuntimeOwnedViewModelInstance::new(file, 0).expect("list item instance"),
            ));
            assert!(source.push_list_item_by_property_name("items", item));
        }
        source
            .list_handle_by_property_path(&[0])
            .expect("items list handle")
    }

    #[test]
    fn nested_artboard_layout_is_a_nested_artboard() {
        let type_key = nuxie_schema::definition_by_name("NestedArtboardLayout")
            .expect("NestedArtboardLayout schema definition")
            .type_key
            .int;

        assert!(runtime_type_is_a(type_key, "NestedArtboard"));
    }

    #[test]
    fn list_converter_group_runs_list_to_length_before_number_to_list() {
        let file = list_binding_fixture();
        let source = list_handle_with_items(&file, 3);
        let mut binding = list_binding(RuntimeDataBindGraphConverter::Group(vec![
            RuntimeDataBindGraphConverter::ListToLength,
            RuntimeDataBindGraphConverter::NumberToList {
                global_id: 1,
                view_model_id: 0,
                view_model_count: 1,
            },
        ]));

        let update =
            binding.apply_resolved_source(&file, RuntimeArtboardListResolvedSource::List(source));

        assert!(update.source.is_none());
        assert_eq!(update.items.as_ref().map(Vec::len), Some(3));
        assert_eq!(binding.source_list_size, Some(3));
        assert_eq!(binding.target_list_size, Some(3));
        assert!(binding.should_reset_instances);
        assert_eq!(binding.generated_view_model_id, Some(0));
    }

    #[test]
    fn number_to_list_preserves_a_list_input() {
        let file = list_binding_fixture();
        let source = list_handle_with_items(&file, 2);
        let mut binding = list_binding(RuntimeDataBindGraphConverter::NumberToList {
            global_id: 1,
            view_model_id: 0,
            view_model_count: 1,
        });

        let update =
            binding.apply_resolved_source(&file, RuntimeArtboardListResolvedSource::List(source));

        assert!(update.source.is_some());
        assert_eq!(update.items.as_ref().map(Vec::len), Some(2));
        assert!(!binding.should_reset_instances);
        assert!(binding.generated_items.is_empty());
    }

    fn custom_binding(
        data_bind_index: usize,
        target_local_id: usize,
        property_key: u16,
        converter: Option<RuntimeDataBindGraphConverter>,
    ) -> RuntimeArtboardCustomPropertyBindingInstance {
        RuntimeArtboardCustomPropertyBindingInstance {
            data_bind_index,
            target_local_id,
            property_key,
            path: shared_data_bind_path(vec![1]),
            path_is_name_based: false,
            owned_context_source_path: None,
            owned_context_source: None,
            flags: 0,
            value_kind: RuntimeArtboardDataBindValueKind::Number,
            converter,
            converter_state: RuntimeDataBindGraphConverterState::None,
            default_value: RuntimeDataBindGraphValue::Number(0.0),
        }
    }

    fn property_binding(
        data_bind_index: usize,
        flags: u64,
    ) -> RuntimeArtboardPropertyBindingInstance {
        RuntimeArtboardPropertyBindingInstance {
            data_bind_index,
            flags,
            target_local_id: data_bind_index + 1,
            property_key: u16::try_from(data_bind_index + 1).unwrap(),
            path: vec![u32::try_from(data_bind_index + 1).unwrap()],
            path_is_name_based: false,
            owned_context_source_path: None,
            enum_value_names: Vec::new(),
            converter: None,
            converter_state: RuntimeDataBindGraphConverterState::None,
            default_value: RuntimeDataBindGraphValue::Number(0.0),
            default_value_is_resolved: true,
            snapshots_source_value: false,
            pending_value: None,
        }
    }

    #[test]
    fn target_queues_partition_two_way_reconcile_by_cpp_precedence() {
        let bindings = vec![
            property_binding(0, (1 << 1) | (1 << 3)),
            property_binding(1, 1 << 1),
            property_binding(2, 0),
        ];
        let mut queues = RuntimeArtboardDataBindTargetQueues::new(&bindings, &[], &[]);

        assert_eq!(
            queues.drain_dirty_properties_for_precedence(&bindings, true),
            vec![0],
            "source-first two-way bindings run before target-to-source reconcile"
        );
        assert_eq!(
            queues.drain_dirty_properties_for_precedence(&bindings, false),
            vec![1, 2],
            "target-first two-way and pure toTarget bindings retain the ordinary lane"
        );
    }

    #[test]
    fn two_way_converter_uses_one_shared_state_and_latches_favored_origin() {
        let mut source_first = property_binding(7, (1 << 1) | (1 << 3));
        source_first.converter = Some(RuntimeDataBindGraphConverter::PassThrough);
        source_first.converter_state =
            RuntimeDataBindGraphConverterState::for_converter(source_first.converter.as_ref());
        let mut source_first_reverse = custom_binding(
            7,
            source_first.target_local_id,
            source_first.property_key,
            source_first.converter.clone(),
        );
        source_first_reverse.flags = source_first.flags;

        let source_first_states = build_artboard_shared_data_bind_converter_states(
            &[source_first],
            &[source_first_reverse],
        );
        assert_eq!(source_first_states.len(), 1);
        assert!(!source_first_states[&7].target_origin);

        let mut target_first = property_binding(8, 1 << 1);
        target_first.converter = Some(RuntimeDataBindGraphConverter::PassThrough);
        let mut target_first_reverse = custom_binding(
            8,
            target_first.target_local_id,
            target_first.property_key,
            target_first.converter.clone(),
        );
        target_first_reverse.flags = target_first.flags;

        let target_first_states = build_artboard_shared_data_bind_converter_states(
            &[target_first],
            &[target_first_reverse],
        );
        assert_eq!(target_first_states.len(), 1);
        assert!(target_first_states[&8].target_origin);
    }

    #[test]
    fn target_to_source_write_suppresses_only_its_own_source_to_target_binding() {
        let mut origin = property_binding(7, 1 << 1);
        origin.path = vec![41];
        let mut observer = property_binding(8, 0);
        observer.path = origin.path.clone();
        let bindings = vec![origin, observer];
        let mut queues = RuntimeArtboardDataBindTargetQueues::new(&bindings, &[], &[]);

        assert_eq!(queues.drain_dirty_properties(), vec![0, 1]);
        assert_eq!(
            queues.enqueue_path(&[41], Some(0)),
            vec![1],
            "C++ suppressDirt skips the originating DataBind but still notifies other observers"
        );
        assert_eq!(queues.drain_dirty_properties(), vec![1]);

        assert_eq!(queues.enqueue_path(&[41], None), vec![0, 1]);
        assert_eq!(queues.drain_dirty_properties(), vec![0, 1]);
    }

    fn numeric_binding(
        data_bind_index: usize,
        target_local_id: usize,
        property_key: u16,
        property: RuntimeArtboardNumericSourceProperty,
    ) -> RuntimeArtboardNumericSourceBindingInstance {
        RuntimeArtboardNumericSourceBindingInstance {
            data_bind_index,
            target_local_id,
            property_key,
            property,
            path: vec![2],
            owned_context_source: None,
            path_is_name_based: false,
            owned_context_source_path: None,
        }
    }

    fn converter_property_binding(
        path: Vec<u32>,
        target: RuntimeArtboardConverterPropertyBindingTarget,
    ) -> RuntimeArtboardConverterPropertyBindingInstance {
        RuntimeArtboardConverterPropertyBindingInstance {
            target,
            path,
            converter: None,
            converter_state: RuntimeDataBindGraphConverterState::None,
            default_value: RuntimeDataBindGraphValue::Number(0.0),
        }
    }

    #[test]
    fn source_queues_split_push_targets_from_persisting_targets() {
        let custom_bindings = vec![
            custom_binding(0, 7, 11, None),
            custom_binding(1, 8, 12, Some(RuntimeDataBindGraphConverter::PassThrough)),
            custom_binding(
                2,
                17,
                18,
                Some(RuntimeDataBindGraphConverter::OperationValue {
                    global_id: 900,
                    operation_type: 2,
                    operation_value: 2.0,
                }),
            ),
            custom_binding(
                3,
                19,
                20,
                Some(RuntimeDataBindGraphConverter::ToString {
                    global_id: 901,
                    flags: 0,
                    decimals: 2,
                    color_format: Vec::new(),
                }),
            ),
            custom_binding(
                4,
                21,
                22,
                Some(RuntimeDataBindGraphConverter::StringTrim {
                    global_id: 902,
                    trim_type: 1,
                }),
            ),
            custom_binding(
                5,
                23,
                24,
                Some(RuntimeDataBindGraphConverter::StringPad {
                    global_id: 903,
                    length: 4,
                    text: b" ".to_vec(),
                    pad_type: 0,
                }),
            ),
            custom_binding(
                6,
                25,
                26,
                Some(RuntimeDataBindGraphConverter::Rounder { decimals: 2 }),
            ),
            custom_binding(
                7,
                15,
                16,
                Some(RuntimeDataBindGraphConverter::RangeMapper {
                    global_id: 0,
                    min_input: 0.0,
                    max_input: 1.0,
                    min_output: 0.0,
                    max_output: 1.0,
                    flags: 0,
                    interpolation_type: 0,
                    interpolator: None,
                }),
            ),
            custom_binding(
                8,
                27,
                28,
                Some(RuntimeDataBindGraphConverter::Interpolator {
                    global_id: 904,
                    duration: 1.0,
                    interpolator: None,
                }),
            ),
            custom_binding(
                9,
                29,
                30,
                Some(RuntimeDataBindGraphConverter::NumberToList {
                    global_id: 905,
                    view_model_id: 0,
                    view_model_count: 1,
                }),
            ),
            custom_binding(
                10,
                31,
                32,
                Some(RuntimeDataBindGraphConverter::OperationViewModel {
                    operation_type: 2,
                    operation_value: 3.0,
                    default_operation_value: 3.0,
                    source_path: Some(vec![1]),
                }),
            ),
            custom_binding(
                11,
                33,
                34,
                Some(RuntimeDataBindGraphConverter::SystemOperationValue {
                    global_id: 906,
                    operation_type: 2,
                    operation_value: 2.0,
                    reverse: false,
                }),
            ),
        ];
        let layout_bindings = vec![RuntimeArtboardLayoutComputedBindingInstance {
            target_local_id: 9,
            property: RuntimeLayoutComputedProperty::WorldX,
            path: shared_data_bind_path(vec![3]),
            owned_context_source: None,
            path_is_name_based: false,
            owned_context_source_path: None,
        }];
        let numeric_bindings = vec![
            numeric_binding(2, 7, 11, RuntimeArtboardNumericSourceProperty::DirectDouble),
            numeric_binding(3, 10, 13, RuntimeArtboardNumericSourceProperty::ShapeLength),
        ];
        let solo_bindings = vec![RuntimeArtboardSoloSourceBindingInstance {
            target_local_id: 14,
            path: shared_data_bind_path(vec![4]),
            path_is_name_based: false,
            owned_context_source_path: None,
            enum_value_names: vec![b"first".to_vec()],
            owned_context_source: None,
        }];
        let mut queues = RuntimeArtboardDataBindSourceQueues::new(
            &custom_bindings,
            &layout_bindings,
            &numeric_bindings,
            &solo_bindings,
        );

        assert!(queues.observes_target_property(7, 11));
        assert!(!queues.observes_target_property(99, 99));

        assert_eq!(
            queues.drain_custom_property_update_indices(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
        );
        assert_eq!(queues.drain_custom_property_update_indices(), vec![7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), vec![0]);
        assert_eq!(queues.persisting_layout_computed(), &[0]);
        assert_eq!(queues.persisting_solo_sources(), &[0]);
        assert_eq!(queues.persisting_numeric_sources(), &[1]);

        queues.enqueue_numeric_push_sources();
        assert_eq!(queues.drain_custom_property_update_indices(), vec![7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), vec![0]);

        queues.enqueue_target_property(7, 11, None);
        queues.enqueue_target_property(7, 11, None);
        queues.enqueue_target_property(99, 99, None);

        assert_eq!(queues.drain_custom_property_update_indices(), vec![0, 7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), vec![0]);

        queues.enqueue_target_property(7, 11, Some(0));

        assert_eq!(queues.drain_custom_property_update_indices(), vec![7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), vec![0]);

        queues.enqueue_target_property(17, 18, None);

        assert_eq!(queues.drain_custom_property_update_indices(), vec![2, 7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(17, 18, Some(2));

        assert_eq!(queues.drain_custom_property_update_indices(), vec![7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(19, 20, None);

        assert_eq!(queues.drain_custom_property_update_indices(), vec![3, 7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(19, 20, Some(3));

        assert_eq!(queues.drain_custom_property_update_indices(), vec![7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(21, 22, None);

        assert_eq!(queues.drain_custom_property_update_indices(), vec![4, 7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(21, 22, Some(4));

        assert_eq!(queues.drain_custom_property_update_indices(), vec![7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(23, 24, None);

        assert_eq!(queues.drain_custom_property_update_indices(), vec![5, 7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(23, 24, Some(5));

        assert_eq!(queues.drain_custom_property_update_indices(), vec![7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(25, 26, None);

        assert_eq!(queues.drain_custom_property_update_indices(), vec![6, 7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(25, 26, Some(6));

        assert_eq!(queues.drain_custom_property_update_indices(), vec![7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(27, 28, None);

        assert_eq!(queues.drain_custom_property_update_indices(), vec![8, 7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(27, 28, Some(8));

        assert_eq!(queues.drain_custom_property_update_indices(), vec![7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(29, 30, None);

        assert_eq!(queues.drain_custom_property_update_indices(), vec![9, 7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(29, 30, Some(9));

        assert_eq!(queues.drain_custom_property_update_indices(), vec![7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(31, 32, None);

        assert_eq!(queues.drain_custom_property_update_indices(), vec![10, 7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(31, 32, Some(10));

        assert_eq!(queues.drain_custom_property_update_indices(), vec![7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(33, 34, None);

        assert_eq!(queues.drain_custom_property_update_indices(), vec![11, 7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(33, 34, Some(11));

        assert_eq!(queues.drain_custom_property_update_indices(), vec![7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());
    }

    #[test]
    fn target_queues_seed_and_dirty_converter_property_bindings() {
        let property_bindings = Vec::new();
        let image_asset_bindings = Vec::new();
        let converter_property_bindings = vec![
            converter_property_binding(
                vec![1],
                RuntimeArtboardConverterPropertyBindingTarget::ToStringDecimals { global_id: 901 },
            ),
            converter_property_binding(
                vec![2],
                RuntimeArtboardConverterPropertyBindingTarget::StringTrimTrimType {
                    global_id: 902,
                },
            ),
            converter_property_binding(
                vec![3],
                RuntimeArtboardConverterPropertyBindingTarget::StringPadText { global_id: 903 },
            ),
            converter_property_binding(
                vec![4],
                RuntimeArtboardConverterPropertyBindingTarget::InterpolatorDuration {
                    global_id: 904,
                },
            ),
            converter_property_binding(
                vec![5],
                RuntimeArtboardConverterPropertyBindingTarget::NumberToListViewModelId {
                    global_id: 905,
                },
            ),
        ];
        let mut queues = RuntimeArtboardDataBindTargetQueues::new(
            &property_bindings,
            &image_asset_bindings,
            &converter_property_bindings,
        );

        assert_eq!(
            queues.drain_dirty_converter_properties(),
            vec![0, 1, 2, 3, 4]
        );
        assert_eq!(
            queues.drain_dirty_converter_properties(),
            Vec::<usize>::new()
        );

        queues.enqueue_path(&[2], None);
        queues.enqueue_path(&[2], None);
        queues.enqueue_path(&[99], None);

        assert_eq!(queues.drain_dirty_converter_properties(), vec![1]);
        assert_eq!(
            queues.drain_dirty_converter_properties(),
            Vec::<usize>::new()
        );

        queues.enqueue_path(&[3], None);

        assert_eq!(queues.drain_dirty_converter_properties(), vec![2]);

        queues.enqueue_path(&[4], None);

        assert_eq!(queues.drain_dirty_converter_properties(), vec![3]);

        queues.enqueue_path(&[5], None);

        assert_eq!(queues.drain_dirty_converter_properties(), vec![4]);
    }
}
