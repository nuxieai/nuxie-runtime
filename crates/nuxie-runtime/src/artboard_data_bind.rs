use crate::data_bind_graph::{
    DATA_BIND_FLAG_DIRECTION_TO_SOURCE, RuntimeDataBindGraphConverterState,
    RuntimeDataBindGraphRangeMapperProperty, runtime_data_bind_graph_convert_value,
    runtime_data_bind_graph_converter, runtime_data_bind_graph_converter_contains_global_id,
    runtime_data_bind_graph_converter_contains_source_change_random,
    runtime_data_bind_graph_converter_requires_persisting_custom_property_source,
    runtime_data_bind_graph_refresh_operation_view_model_converter_for_imported_context,
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
use crate::view_model::RuntimeOwnedViewModelListHandle;
use crate::{
    ArtboardInstance, Mat2D, RuntimeDataBindGraphConverter, RuntimeDataBindGraphValue,
    RuntimeDataContext, RuntimeOwnedViewModelInstance, RuntimeViewModelPointer, ScriptInstance,
    data_bind_flags_apply_source_to_target, data_bind_flags_apply_target_to_source,
};
use nuxie_binary::{RuntimeDataType, RuntimeFile, RuntimeObject};
use nuxie_graph::ArtboardGraph;
use nuxie_schema::{FieldKind, definition_by_type_key};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

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

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardImageAssetBindingInstance {
    target_local_id: usize,
    path: Vec<u32>,
    owned_context_source_path: Option<Vec<usize>>,
    default_value: RuntimeDataBindGraphValue,
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeNestedChildContextUpdate {
    Property(usize, RuntimeDataBindGraphValue),
    ImageAsset(usize, RuntimeDataBindGraphValue),
    ContextPath(Vec<u32>, RuntimeDataBindGraphValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeArtboardOwnedContextKey {
    view_model_index: usize,
    // C++ Artboard::bindViewModelInstance rebinds unconditionally; this key
    // is an invented change detector, so it must carry the instance identity
    // in addition to the mutation generation — two distinct instances of the
    // same view model both start at generation zero.
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
            view_model_index: context.view_model_index,
            instance_identity: context.instance_identity(),
            mutation_generation: context.mutation_generation(),
            context_chain: context_chain
                .iter()
                .map(|context_path| context_path.to_vec())
                .collect(),
        }
    }

    fn matches_context_chain(
        &self,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> bool {
        self.view_model_index == context.view_model_index
            && self.instance_identity == context.instance_identity()
            && self.mutation_generation == context.mutation_generation()
            && self.context_chain.len() == context_chain.len()
            && self
                .context_chain
                .iter()
                .zip(context_chain)
                .all(|(stored, current)| stored.as_slice() == *current)
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

    fn enqueue_path(&mut self, path: &[u32]) -> Vec<usize> {
        let Some(targets) = self.by_path.get(path).cloned() else {
            return Vec::new();
        };
        let mut enqueued_properties = Vec::new();
        for target in targets {
            match target {
                RuntimeArtboardDataBindTargetRef::Property(index) => {
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
            // A two-way binding initializes target from source. Only a
            // target-to-source-only binding may seed the source from the
            // serialized target before the target has observed its context.
            if !data_bind_flags_apply_source_to_target(binding.flags) {
                queues.enqueue_custom_property(index);
            }
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
    ) {
        let Self {
            by_target_property,
            dirty_custom_properties,
            dirty_custom_property_flags,
            dirty_numeric_sources,
            dirty_numeric_source_flags,
            ..
        } = self;
        let Some(sources) = by_target_property.get(&(local_id, property_key)) else {
            return;
        };
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
                }
            }
        }
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

    #[cfg(test)]
    fn persisting_numeric_sources(&self) -> &[usize] {
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
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardNumericSourceBindingInstance {
    data_bind_index: usize,
    target_local_id: usize,
    property_key: u16,
    property: RuntimeArtboardNumericSourceProperty,
    path: Vec<u32>,
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
    enum_value_names: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardNestedHostBindingInstance {
    target_local_id: usize,
    property: RuntimeArtboardNestedHostProperty,
    path: Vec<u32>,
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
    default_value: &RuntimeDataBindGraphValue,
    retained_source_path: &mut Option<Vec<usize>>,
) -> Option<RuntimeDataBindGraphValue> {
    if let Some(source_path) = retained_source_path.as_deref()
        && let Some(value) = runtime_owned_view_model_binding_value_for_property_path(
            context,
            source_path,
            default_value,
        )
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
        let value = runtime_owned_view_model_binding_value_for_property_path(
            context,
            &source_path,
            default_value,
        )?;
        Some((source_path, value))
    })?;
    *retained_source_path = Some(source_path);
    Some(value)
}

fn runtime_owned_view_model_binding_value_for_property_path(
    context: &RuntimeOwnedViewModelInstance,
    property_path: &[usize],
    default_value: &RuntimeDataBindGraphValue,
) -> Option<RuntimeDataBindGraphValue> {
    match default_value {
        RuntimeDataBindGraphValue::Number(_) => context
            .number_value_by_property_path(property_path)
            .map(RuntimeDataBindGraphValue::Number),
        RuntimeDataBindGraphValue::Boolean(_) => context
            .boolean_value_by_property_path(property_path)
            .map(RuntimeDataBindGraphValue::Boolean),
        RuntimeDataBindGraphValue::String(_) => context
            .string_value_by_property_path(property_path)
            .map(|value| RuntimeDataBindGraphValue::String(value.to_vec())),
        RuntimeDataBindGraphValue::Color(_) => context
            .color_value_by_property_path(property_path)
            .map(RuntimeDataBindGraphValue::Color),
        RuntimeDataBindGraphValue::Enum(_) => context
            .enum_value_by_property_path(property_path)
            .map(RuntimeDataBindGraphValue::Enum),
        RuntimeDataBindGraphValue::SymbolListIndex(_) => context
            .symbol_list_index_value_by_property_path(property_path)
            .map(RuntimeDataBindGraphValue::SymbolListIndex),
        RuntimeDataBindGraphValue::List { .. } => context
            .list_item_count_by_property_path(property_path)
            .map(|item_count| RuntimeDataBindGraphValue::List { item_count }),
        RuntimeDataBindGraphValue::Asset(_) => context
            .asset_value_by_property_path(property_path)
            .map(RuntimeDataBindGraphValue::Asset),
        RuntimeDataBindGraphValue::Artboard(_) => context
            .artboard_value_by_property_path(property_path)
            .map(RuntimeDataBindGraphValue::Artboard),
        RuntimeDataBindGraphValue::Trigger(_) => context
            .trigger_value_by_property_path(property_path)
            .map(RuntimeDataBindGraphValue::Trigger),
        RuntimeDataBindGraphValue::ViewModel(_) => context
            .view_model_value_by_property_path(property_path)
            .map(RuntimeDataBindGraphValue::ViewModel),
        RuntimeDataBindGraphValue::ListLength(_) => None,
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
    source_list_size: Option<usize>,
    source_number_value: Option<f32>,
    target_list_size: Option<usize>,
    should_reset_instances: bool,
    generated_items: Vec<RuntimeOwnedViewModelInstance>,
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
            let generated_view_model_index = converter
                .as_ref()
                .and_then(runtime_project_number_to_list_view_model_index);
            let source = default_instance.as_ref().and_then(|default_instance| {
                file.data_context_view_model_property_for_instance(default_instance.object, &path)
            });
            let source_is_unresolved_name_based = path_is_name_based && source.is_none();
            let default_value = source
                .and_then(|source| match converter.as_ref() {
                    Some(RuntimeDataBindGraphConverter::NumberToList { .. }) => file
                        .view_model_instance_number_value_for_object(source)
                        .map(RuntimeDataBindGraphValue::Number),
                    Some(_) if generated_view_model_index.is_some() => file
                        .view_model_instance_number_value_for_object(source)
                        .map(RuntimeDataBindGraphValue::Number),
                    None => file
                        .view_model_instance_list_size_for_object(source)
                        .map(|item_count| RuntimeDataBindGraphValue::List { item_count }),
                    _ => None,
                })
                .or_else(|| match converter.as_ref() {
                    Some(RuntimeDataBindGraphConverter::NumberToList { .. }) => {
                        Some(RuntimeDataBindGraphValue::Number(0.0))
                    }
                    Some(_) if generated_view_model_index.is_some() => {
                        Some(RuntimeDataBindGraphValue::Number(0.0))
                    }
                    None => Some(RuntimeDataBindGraphValue::List { item_count: 0 }),
                    _ => None,
                })?;

            Some(RuntimeArtboardListBindingInstance {
                data_bind_index,
                target_local_id,
                path: path.to_vec(),
                converter,
                default_value,
                source_list_size: None,
                source_number_value: None,
                target_list_size: source_is_unresolved_name_based.then_some(0),
                should_reset_instances: false,
                generated_items: Vec::new(),
            })
        })
        .collect()
}

fn runtime_project_number_to_list_view_model_index(
    converter: &RuntimeDataBindGraphConverter,
) -> Option<usize> {
    match converter {
        RuntimeDataBindGraphConverter::Project { program, .. } => {
            program.number_to_list_output_view_model_index()
        }
        RuntimeDataBindGraphConverter::Group(converters) => converters
            .last()
            .and_then(runtime_project_number_to_list_view_model_index),
        _ => None,
    }
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
            if !data_bind_flags_apply_source_to_target(
                data_bind.object.uint_property("flags").unwrap_or(0),
            ) {
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
                        if property_kind == FieldKind::String
                            && runtime_data_bind_property_key_for_name("TextValueRun", "text")
                                == Some(property_key)
                        {
                            return Some(RuntimeDataBindGraphValue::String(Vec::new()));
                        }
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
                        _ => RuntimeDataBindGraphValue::Number(0.0),
                    });
            if !artboard_property_binding_value_matches_kind(&default_value, property_kind)
                && !artboard_property_binding_allows_converted_default(
                    converter.as_ref(),
                    &default_value,
                    property_kind,
                )
            {
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
            if target.type_name != "Image" {
                return None;
            }
            if u16::try_from(data_bind.object.uint_property("propertyKey")?).ok()?
                != image_asset_id_key
            {
                return None;
            }
            let path = file.data_bind_context_source_path_ids_for_object(data_bind.object)?;
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
                })?;
            if !matches!(default_value, RuntimeDataBindGraphValue::Asset(_)) {
                return None;
            }

            Some(RuntimeArtboardImageAssetBindingInstance {
                target_local_id: data_bind.target_local_id?,
                path,
                owned_context_source_path: None,
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
                owned_context_source_path: None,
                artboard_value_applied: true,
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
    let default_instance = artboard_default_view_model_instance(file, artboard_index);

    let mut values = BTreeMap::new();
    for data_bind in file.artboard_data_binds(artboard_index) {
        let Some(path) = file.data_bind_context_source_path_ids_for_object(data_bind.object) else {
            continue;
        };
        let Some(value) = default_instance
            .as_ref()
            .and_then(|default_instance| {
                runtime_created_view_model_value_for_path(file, default_instance.object, &path)
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
    let trim_start_key = runtime_data_bind_property_key_for_name("TrimPath", "start");
    let trim_end_key = runtime_data_bind_property_key_for_name("TrimPath", "end");
    let shape_length_key = runtime_data_bind_property_key_for_name("Shape", "length");
    let parametric_width_key = runtime_data_bind_property_key_for_name("ParametricPath", "width");
    let parametric_height_key = runtime_data_bind_property_key_for_name("ParametricPath", "height");

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
                _ => {
                    let uses_specialized_numeric_source = matches!(target.type_name,
                        "TrimPath" if [trim_start_key, trim_end_key].contains(&Some(property_key))
                    ) || (target.type_name == "Shape"
                        && Some(property_key) == shape_length_key)
                        || (runtime_type_is_a(target.type_key, "ParametricPath")
                            && [parametric_width_key, parametric_height_key]
                                .contains(&Some(property_key)));
                    if uses_specialized_numeric_source {
                        return None;
                    }
                    match nuxie_schema::core_registry_setter_field_kind_by_property_key(
                        property_key,
                    )? {
                        FieldKind::Double => RuntimeArtboardDataBindValueKind::Number,
                        FieldKind::Bool => RuntimeArtboardDataBindValueKind::Boolean,
                        FieldKind::String => RuntimeArtboardDataBindValueKind::String,
                        FieldKind::Color => RuntimeArtboardDataBindValueKind::Color,
                        _ => return None,
                    }
                }
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
    let mut range_mapper_property_counts = BTreeMap::<u32, usize>::new();
    for data_bind in file
        .objects
        .iter()
        .flatten()
        .filter(|object| object.type_name == "DataBindContext")
    {
        let Some(target) = file.data_bind_target_for_object(data_bind) else {
            continue;
        };
        let Some(property_key) = data_bind
            .uint_property("propertyKey")
            .and_then(|value| u16::try_from(value).ok())
        else {
            continue;
        };
        if target.type_name == "DataConverterRangeMapper"
            && [
                range_mapper_min_input_key,
                range_mapper_max_input_key,
                range_mapper_min_output_key,
                range_mapper_max_output_key,
            ]
            .contains(&Some(property_key))
            && artboard_converters.iter().any(|converter| {
                runtime_data_bind_graph_converter_contains_global_id(converter, target.id)
            })
        {
            *range_mapper_property_counts.entry(target.id).or_default() += 1;
        }
    }

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
                    if range_mapper_property_counts
                        .get(&target.id)
                        .is_some_and(|count| *count > 1)
                    {
                        return None;
                    }
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
                enum_value_names,
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
        RuntimeDataType::AssetImage => Some(RuntimeDataBindGraphValue::Asset(u64::from(u32::MAX))),
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
        "ViewModelPropertyAssetImage" => {
            Some(RuntimeDataBindGraphValue::Asset(u64::from(u32::MAX)))
        }
        "ViewModelPropertyArtboard" => {
            Some(RuntimeDataBindGraphValue::Artboard(u64::from(u32::MAX)))
        }
        _ => None,
    }
}

impl ArtboardInstance {
    pub fn has_scripted_data_converter_instance_for_global(&self, global_id: u32) -> bool {
        self.scripted_data_converter_instances_by_global
            .contains_key(&global_id)
    }

    pub fn set_scripted_data_converter_instance_for_global(
        &mut self,
        global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) -> bool {
        let handle = RuntimeScriptInstanceHandle::new(instance);
        self.scripted_data_converter_instances_by_global
            .insert(global_id, handle.clone());
        self.refresh_artboard_converter_dependents(|converter| {
            converter.attach_scripted_instance(global_id, &handle)
        })
    }

    fn enqueue_artboard_data_bind_targets_for_path(&mut self, path: &[u32]) {
        let value = self.artboard_data_bind_values.get(path).cloned();
        let enqueued = self.artboard_data_bind_target_queues.enqueue_path(path);
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

    pub(crate) fn notify_artboard_data_bind_target_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
    ) {
        self.artboard_data_bind_source_queues
            .enqueue_target_property(
                local_id,
                property_key,
                self.artboard_data_bind_suppressed_target_data_bind,
            );
    }

    pub(crate) fn update_nested_artboard_data_binds_from_hosts(&mut self) -> bool {
        let mut changed = false;
        let mut values = std::mem::take(&mut self.artboard_context_source_values_scratch);
        self.collect_nested_artboard_context_source_values(Mat2D::IDENTITY, &mut values);
        for source in values.drain(..) {
            changed |= self.set_artboard_data_bind_value_for_path(&source.path, source.value);
        }
        self.artboard_context_source_values_scratch = values;
        changed
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
        let context_chain: [&[usize]; 1] = [&[]];
        self.bind_owned_view_model_artboard_context_chain(file, context, &context_chain, true, true)
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
        let rebind_self = self.retain_owned_view_model_context_chain(context, context_chain);
        let mut changed = if bind_self && rebind_self {
            let mut changed = self.refresh_artboard_converter_dependents(|converter| {
                runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_context(
                    converter,
                    context,
                    context_chain,
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
            changed
        } else {
            false
        };
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
                    &binding.default_value,
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
            let update = {
                let binding = &mut self.artboard_image_asset_bindings[index];
                runtime_owned_view_model_binding_value_for_retained_context_chain(
                    file,
                    context,
                    context_chain,
                    &binding.path,
                    false,
                    allow_full_context_bindings,
                    &binding.default_value,
                    &mut binding.owned_context_source_path,
                )
            };
            if let Some(value) = update {
                let path = self.artboard_image_asset_bindings[index].path.as_slice();
                if self.artboard_data_bind_values.get(path) == Some(&value) {
                    continue;
                }
                let path = self.artboard_image_asset_bindings[index].path.clone();
                changed |= self.set_artboard_data_bind_value_for_path(&path, value);
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
                    &binding.default_value,
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
            let Some(default_value) = self.artboard_data_bind_values.get(&path).cloned() else {
                continue;
            };
            let update = {
                let binding = &mut self.artboard_nested_host_bindings[index];
                runtime_owned_view_model_binding_value_for_retained_context_chain(
                    file,
                    context,
                    context_chain,
                    &binding.path,
                    false,
                    allow_full_context_bindings,
                    &default_value,
                    &mut binding.owned_context_source_path,
                )
            };
            if let Some(value) = update
                && self.artboard_data_bind_values.get(&path) != Some(&value)
            {
                changed |= self.set_artboard_data_bind_value_for_path(&path, value);
            }
        }
        for binding in &mut self.artboard_list_bindings {
            if binding
                .converter
                .as_ref()
                .and_then(runtime_project_number_to_list_view_model_index)
                .is_none()
            {
                continue;
            }
            let Some(value) = context_chain.iter().find_map(|context_path| {
                context.number_value_by_context_source_path(
                    file,
                    context_path,
                    &binding.path,
                    false,
                )
            }) else {
                continue;
            };
            let source_value = RuntimeDataBindGraphValue::Number(value);
            let target_value = binding.converter.as_ref().and_then(|converter| {
                runtime_data_bind_graph_convert_value(converter, &source_value)
            });
            binding.source_list_size = None;
            binding.source_number_value = Some(value);
            binding.should_reset_instances = true;
            binding.default_value = source_value;
            let target_list_size = match target_value {
                Some(RuntimeDataBindGraphValue::List { item_count }) => Some(item_count),
                _ => None,
            };
            if binding.target_list_size != target_list_size {
                binding.target_list_size = target_list_size;
                changed = true;
            }
        }
        changed |= self.sync_project_number_to_list_bindings(file);

        let component_list_updates = self
            .artboard_list_bindings
            .iter()
            .filter_map(|binding| {
                let source = context_chain.iter().find_map(|context_path| {
                    let property_path = context
                        .property_path_for_context_source_path_with_manifest_mode(
                            file,
                            context_path,
                            &binding.path,
                            false,
                            allow_full_context_bindings,
                        )?;
                    context.list_handle_by_property_path(&property_path)
                })?;
                Some((binding.target_local_id, source))
            })
            .collect::<Vec<_>>();
        for (list_local_id, source) in component_list_updates {
            let items = source.items();
            self.component_list_sources.insert(list_local_id, source);
            changed |= self.sync_component_list_items(file, list_local_id, items);
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
        if changed {
            changed |= self.apply_artboard_nested_host_bindings();
        }
        changed
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
            changed |= self.set_artboard_data_bind_value_for_path(&path, value);
        }
        let runtime_context = RuntimeDataContext::from_instance_object(file, view_model_instance);
        for binding in &mut self.artboard_list_bindings {
            if let (Some(context), Some(converter)) =
                (runtime_context.as_ref(), binding.converter.as_mut())
            {
                runtime_data_bind_graph_refresh_operation_view_model_converter_for_imported_context(
                    file, converter, context,
                );
            }
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
            binding.source_list_size = match &source_value {
                RuntimeDataBindGraphValue::List { item_count } => Some(*item_count),
                _ => None,
            };
            binding.source_number_value = match &source_value {
                RuntimeDataBindGraphValue::Number(value) => Some(*value),
                _ => None,
            };
            binding.should_reset_instances = binding.source_number_value.is_some();
            let target_list_size = match target_value {
                Some(RuntimeDataBindGraphValue::List { item_count }) => Some(item_count),
                _ => None,
            };
            binding.default_value = source_value;
            if binding.target_list_size != target_list_size {
                changed = true;
                binding.target_list_size = target_list_size;
            }
        }
        changed |= self.sync_project_number_to_list_bindings(file);
        changed
    }

    fn sync_project_number_to_list_bindings(&mut self, file: &RuntimeFile) -> bool {
        let mut updates = Vec::new();
        for binding in &mut self.artboard_list_bindings {
            let Some(view_model_index) = binding
                .converter
                .as_ref()
                .and_then(runtime_project_number_to_list_view_model_index)
            else {
                continue;
            };
            let count = binding.target_list_size.unwrap_or(0);
            if binding
                .generated_items
                .iter()
                .any(|item| item.view_model_index() != view_model_index)
            {
                binding.generated_items.clear();
            }
            while binding.generated_items.len() < count {
                let Some(item) = RuntimeOwnedViewModelInstance::new(file, view_model_index) else {
                    binding.generated_items.clear();
                    break;
                };
                binding.generated_items.push(item);
            }
            binding.generated_items.truncate(count);
            updates.push((binding.target_local_id, binding.generated_items.clone()));
        }
        updates
            .into_iter()
            .fold(false, |changed, (target_local_id, items)| {
                self.sync_component_list_items(file, target_local_id, items) || changed
            })
    }

    pub fn advance_artboard_data_binds(&mut self) -> bool {
        self.advance_artboard_data_binds_with_elapsed(0.0)
    }

    pub(crate) fn set_artboard_data_bind_value_for_path(
        &mut self,
        path: &[u32],
        value: RuntimeDataBindGraphValue,
    ) -> bool {
        if self.artboard_data_bind_values.get(path) == Some(&value) {
            return false;
        }
        let number_value = match &value {
            RuntimeDataBindGraphValue::Number(value) => Some(*value),
            _ => None,
        };
        self.artboard_data_bind_values.insert(path.to_vec(), value);
        self.reset_artboard_property_formula_random_state_for_path(path);
        self.enqueue_artboard_data_bind_targets_for_path(path);
        if let Some(value) = number_value {
            self.refresh_artboard_operation_view_model_number_converter_dependents_for_path(
                path, value,
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

    pub(crate) fn advance_artboard_data_binds_with_root_transform(
        &mut self,
        root_transform: Mat2D,
        elapsed_seconds: f32,
    ) -> bool {
        let mut changed = false;
        // C++ DataBindContainer::updateDataBind updates converter dependents
        // before applying a target-to-source binding.
        changed |= self.update_artboard_formula_token_bindings();
        changed |= self.update_artboard_converter_property_bindings();
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
        changed |= self.apply_artboard_property_bindings();
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
        for binding in &mut self.artboard_list_bindings {
            let target_value = match binding.converter.as_ref() {
                Some(converter) => {
                    runtime_data_bind_graph_convert_value(converter, &binding.default_value)
                }
                None => Some(binding.default_value.clone()),
            };
            let target_list_size = match target_value {
                Some(RuntimeDataBindGraphValue::List { item_count }) => Some(item_count),
                _ => None,
            };
            if binding.target_list_size != target_list_size {
                binding.target_list_size = target_list_size;
                changed = true;
            }
        }
        if let Some(file) = self.runtime_file_arc() {
            changed |= self.sync_project_number_to_list_bindings(&file);
        }
        changed |= self.apply_artboard_solo_bindings();
        changed |= self.apply_artboard_nested_host_bindings();
        changed |= self.sync_nested_child_artboard_data_contexts();
        changed
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
            let Some(binding) = self.artboard_numeric_source_bindings.get(index) else {
                continue;
            };
            let value = self.artboard_numeric_source_binding_value(
                binding.target_local_id,
                binding.property_key,
                binding.property,
            );
            let Some(value) = value else { continue };
            let value = RuntimeDataBindGraphValue::Number(value);
            let path = binding.path.clone();
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
        for index in 0..self.artboard_property_bindings.len() {
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
        for index in 0..self.artboard_property_bindings.len() {
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
            let Some(binding) = self.artboard_layout_computed_bindings.get(index) else {
                continue;
            };
            let value = self.runtime_graph().and_then(|graph| {
                self.artboard_layout_computed_binding_value(
                    binding.target_local_id,
                    binding.property,
                    graph,
                    root_transform,
                )
            });
            let Some(value) = value else { continue };
            let value = RuntimeDataBindGraphValue::Number(value);
            let path = binding.path.clone();
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
        let mut changed = false;
        for index in self
            .artboard_data_bind_target_queues
            .drain_dirty_properties()
        {
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
            let Some((target_local_id, value)) =
                self.artboard_image_asset_bindings
                    .get(index)
                    .map(|binding| {
                        let value = self
                            .artboard_data_bind_values
                            .get(&binding.path)
                            .cloned()
                            .unwrap_or_else(|| binding.default_value.clone());
                        (binding.target_local_id, value)
                    })
            else {
                continue;
            };
            changed |= self.apply_artboard_image_asset_binding_value(target_local_id, &value);
        }
        changed
    }

    fn apply_artboard_image_asset_binding_value(
        &mut self,
        target_local_id: usize,
        value: &RuntimeDataBindGraphValue,
    ) -> bool {
        // Mirrors C++ `src/data_bind/context/context_value_asset_image.cpp`:
        // applying an asset-image value to an Image target swaps the Image's
        // asset pointer. Missing/sentinel values use the view-model instance's
        // private empty ImageAsset, which makes Image::draw return early.
        let RuntimeDataBindGraphValue::Asset(value) = value else {
            return false;
        };
        let asset_global = self
            .runtime_file()
            .and_then(|file| runtime_image_asset_global_for_file_asset_index(file, *value));
        self.set_image_asset_override(target_local_id, asset_global)
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
        let converted = match binding.converter.as_ref() {
            Some(RuntimeDataBindGraphConverter::ToString { .. }) => match value {
                RuntimeDataBindGraphValue::Enum(value) => {
                    let index = usize::try_from(value).ok()?;
                    binding
                        .enum_value_names
                        .get(index)
                        .cloned()
                        .map(RuntimeDataBindGraphValue::String)
                }
                _ => binding.converter_state.convert_value_with_formula_randoms(
                    binding.converter.as_ref()?,
                    &value,
                    &mut self.artboard_formula_random_source,
                ),
            },
            Some(converter) => binding.converter_state.convert_value_with_formula_randoms(
                converter,
                &value,
                &mut self.artboard_formula_random_source,
            ),
            None => Some(value),
        }?;
        Some((
            binding.data_bind_index,
            binding.target_local_id,
            binding.property_key,
            converted,
        ))
    }

    fn reset_artboard_property_formula_random_state_for_path(&mut self, path: &[u32]) {
        for binding in &mut self.artboard_property_bindings {
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
        mut update: impl FnMut(&mut RuntimeDataBindGraphConverter) -> bool,
    ) -> bool {
        let mut changed = false;

        for index in 0..self.artboard_property_bindings.len() {
            let binding_changed = {
                let binding = &mut self.artboard_property_bindings[index];
                binding.converter.as_mut().is_some_and(&mut update)
            };
            if binding_changed {
                self.enqueue_artboard_property_binding_target(index);
                changed = true;
            }
        }

        for index in 0..self.artboard_custom_property_bindings.len() {
            let binding_changed = {
                let binding = &mut self.artboard_custom_property_bindings[index];
                binding.converter.as_mut().is_some_and(&mut update)
            };
            if binding_changed {
                if !data_bind_flags_apply_source_to_target(
                    self.artboard_custom_property_bindings[index].flags,
                ) {
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

    fn refresh_artboard_operation_view_model_number_converter_dependents_for_path(
        &mut self,
        path: &[u32],
        value: f32,
    ) -> bool {
        self.refresh_artboard_converter_dependents(|converter| {
            runtime_data_bind_graph_refresh_operation_view_model_number_converter_for_path(
                converter, path, value,
            )
        })
    }

    fn advance_artboard_property_binding_converters(&mut self, elapsed_seconds: f32) -> bool {
        let mut changed = false;
        for index in 0..self.artboard_property_bindings.len() {
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
            let Some(binding) = self.artboard_solo_source_bindings.get(index) else {
                continue;
            };
            let Some(value) = self.artboard_solo_source_binding_value(
                binding.target_local_id,
                &binding.enum_value_names,
            ) else {
                continue;
            };
            let path = binding.path.clone();
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
        let Some(target_value) = self.artboard_custom_property_binding_target_value(index) else {
            return false;
        };
        let Some((path, value)) =
            self.convert_artboard_custom_property_binding_target_value(index, &target_value)
        else {
            return false;
        };
        self.set_artboard_data_bind_value_for_path(path.as_ref(), value)
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
        let converted = match binding.converter.as_ref() {
            None => value.clone(),
            Some(converter) if binding.flags & DATA_BIND_FLAG_DIRECTION_TO_SOURCE != 0 => {
                binding.converter_state.convert_value_with_formula_randoms(
                    converter,
                    value,
                    &mut self.artboard_formula_random_source,
                )?
            }
            Some(converter) => binding
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
        let Some(parent_key) = runtime_data_bind_component_parent_id_key() else {
            return false;
        };
        let Some(property_id_key) = runtime_data_bind_view_model_instance_value_property_id_key()
        else {
            return false;
        };
        let Some(property_value_key) =
            runtime_data_bind_property_key_for_name("ViewModelInstanceViewModel", "propertyValue")
        else {
            return false;
        };

        let mut updates = BTreeMap::<usize, Vec<(usize, usize)>>::new();
        for host_local_id in &self.nested_artboard_locals {
            let Some(instance_local_id) = self
                .nested_artboards
                .get(host_local_id)
                .and_then(|nested| nested.stateful_view_model_instance_local)
            else {
                continue;
            };
            for slot in &self.slots {
                if slot.type_name != Some("ViewModelInstanceViewModel")
                    || self.uint_property(slot.local_id, parent_key)
                        != u64::try_from(instance_local_id).ok()
                {
                    continue;
                }
                let Some(property_index) = self
                    .uint_property(slot.local_id, property_id_key)
                    .and_then(|value| usize::try_from(value).ok())
                else {
                    continue;
                };
                let Some(instance_index) = self
                    .uint_property(slot.local_id, property_value_key)
                    .and_then(|value| usize::try_from(value).ok())
                else {
                    continue;
                };
                updates
                    .entry(*host_local_id)
                    .or_default()
                    .push((property_index, instance_index));
            }
        }

        let mut changed = false;
        let Some(file) = self.runtime_file_arc() else {
            return false;
        };
        for (host_local_id, updates) in updates {
            let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
                continue;
            };
            let Some(context) = nested.stateful_view_model_context.as_mut() else {
                continue;
            };
            let context_changed = updates.into_iter().fold(false, |changed, update| {
                context.set_view_model_by_property_index(update.0, update.1) || changed
            });
            if !context_changed {
                continue;
            }
            let context = context.clone();
            let context_chain: [&[usize]; 1] = [&[]];
            changed = true;
            changed |=
                nested.bind_owned_view_model_animation_contexts(&file, &context, &context_chain);
            changed |= nested.child.bind_owned_view_model_artboard_context_chain(
                &file,
                &context,
                &context_chain,
                true,
                true,
            );
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
                let property_value_key = runtime_data_bind_view_model_instance_asset_value_key()?;
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
    use crate::data_bind_graph::DATA_BIND_FLAG_TWO_WAY;

    #[test]
    fn nested_artboard_layout_is_a_nested_artboard() {
        let type_key = nuxie_schema::definition_by_name("NestedArtboardLayout")
            .expect("NestedArtboardLayout schema definition")
            .type_key
            .int;

        assert!(runtime_type_is_a(type_key, "NestedArtboard"));
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
            flags: 0,
            value_kind: RuntimeArtboardDataBindValueKind::Number,
            converter,
            converter_state: RuntimeDataBindGraphConverterState::None,
            default_value: RuntimeDataBindGraphValue::Number(0.0),
        }
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
        let mut custom_bindings = vec![
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
        custom_bindings[0].flags = DATA_BIND_FLAG_DIRECTION_TO_SOURCE;
        custom_bindings[1].flags = DATA_BIND_FLAG_TWO_WAY;
        let layout_bindings = vec![RuntimeArtboardLayoutComputedBindingInstance {
            target_local_id: 9,
            property: RuntimeLayoutComputedProperty::WorldX,
            path: shared_data_bind_path(vec![3]),
        }];
        let numeric_bindings = vec![
            numeric_binding(2, 7, 11, RuntimeArtboardNumericSourceProperty::DirectDouble),
            numeric_binding(3, 10, 13, RuntimeArtboardNumericSourceProperty::ShapeLength),
        ];
        let solo_bindings = vec![RuntimeArtboardSoloSourceBindingInstance {
            target_local_id: 14,
            path: shared_data_bind_path(vec![4]),
            enum_value_names: vec![b"first".to_vec()],
        }];
        let mut queues = RuntimeArtboardDataBindSourceQueues::new(
            &custom_bindings,
            &layout_bindings,
            &numeric_bindings,
            &solo_bindings,
        );

        assert_eq!(queues.drain_custom_property_update_indices(), vec![0, 7]);
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

        queues.enqueue_target_property(8, 12, None);

        assert_eq!(queues.drain_custom_property_update_indices(), vec![1, 7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

        queues.enqueue_target_property(8, 12, Some(1));

        assert_eq!(queues.drain_custom_property_update_indices(), vec![7]);
        assert_eq!(queues.drain_dirty_numeric_sources(), Vec::<usize>::new());

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

        queues.enqueue_path(&[2]);
        queues.enqueue_path(&[2]);
        queues.enqueue_path(&[99]);

        assert_eq!(queues.drain_dirty_converter_properties(), vec![1]);
        assert_eq!(
            queues.drain_dirty_converter_properties(),
            Vec::<usize>::new()
        );

        queues.enqueue_path(&[3]);

        assert_eq!(queues.drain_dirty_converter_properties(), vec![2]);

        queues.enqueue_path(&[4]);

        assert_eq!(queues.drain_dirty_converter_properties(), vec![3]);

        queues.enqueue_path(&[5]);

        assert_eq!(queues.drain_dirty_converter_properties(), vec![4]);
    }
}
