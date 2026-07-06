use crate::data_bind_graph::{
    DATA_BIND_FLAG_DIRECTION_TO_SOURCE, RuntimeDataBindGraphConverterState,
    runtime_data_bind_graph_convert_value, runtime_data_bind_graph_converter,
    runtime_data_bind_graph_converter_contains_source_change_random,
};
use crate::objects::InstanceObjectArena;
use crate::properties::{
    RuntimeLayoutComputedProperty, artboard_index_for_graph, layout_computed_property_for_key,
    property_key_for_name, solid_color_value_property_key, solo_active_component_id_property_key,
};
use crate::{
    ArtboardInstance, RuntimeDataBindGraphConverter, RuntimeDataBindGraphValue,
    data_bind_flags_apply_source_to_target, data_bind_flags_apply_target_to_source,
};
use rive_binary::{RuntimeDataType, RuntimeFile, RuntimeObject};
use rive_graph::ArtboardGraph;
use rive_schema::FieldKind;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardPropertyBindingInstance {
    target_local_id: usize,
    property_key: u16,
    path: Vec<u32>,
    enum_value_names: Vec<Vec<u8>>,
    converter: Option<RuntimeDataBindGraphConverter>,
    converter_state: RuntimeDataBindGraphConverterState,
    default_value: RuntimeDataBindGraphValue,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardCustomPropertyBindingInstance {
    target_local_id: usize,
    property_key: u16,
    path: Vec<u32>,
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
    path: Vec<u32>,
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
    path: Vec<u32>,
    enum_value_names: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtboardNestedHostBindingInstance {
    target_local_id: usize,
    property: RuntimeArtboardNestedHostProperty,
    path: Vec<u32>,
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
}

pub(super) fn build_artboard_list_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardListBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let Some(default_instance) = artboard_default_view_model_instance(file, artboard_index) else {
        return Vec::new();
    };

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
            let source =
                file.data_context_view_model_property_for_instance(default_instance.object, &path);
            let source_is_unresolved_name_based = path_is_name_based && source.is_none();
            let default_value = source
                .and_then(|source| match converter.as_ref() {
                    Some(RuntimeDataBindGraphConverter::NumberToList { .. }) => file
                        .view_model_instance_number_value_for_object(source)
                        .map(RuntimeDataBindGraphValue::Number),
                    None => file
                        .view_model_instance_list_size_for_object(source)
                        .map(|item_count| RuntimeDataBindGraphValue::List { item_count }),
                    _ => None,
                })
                .or_else(|| {
                    if !path_is_name_based {
                        return None;
                    }
                    match converter.as_ref() {
                        Some(RuntimeDataBindGraphConverter::NumberToList { .. }) => {
                            Some(RuntimeDataBindGraphValue::Number(0.0))
                        }
                        None => Some(RuntimeDataBindGraphValue::List { item_count: 0 }),
                        _ => None,
                    }
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

    file.artboard_data_binds(artboard_index)
        .into_iter()
        .filter_map(|data_bind| {
            if !data_bind_flags_apply_source_to_target(
                data_bind.object.uint_property("flags").unwrap_or(0),
            ) {
                return None;
            }
            let target = data_bind.target?;
            if matches!(
                target.type_name,
                "ArtboardComponentList" | "NestedArtboard" | "Solo"
            ) {
                return None;
            }
            let target_local_id = data_bind.target_local_id?;
            let property_key =
                u16::try_from(data_bind.object.uint_property("propertyKey")?).ok()?;
            let Some(property_kind) =
                rive_schema::core_registry_setter_field_kind_by_property_key(property_key)
            else {
                return None;
            };
            if !matches!(
                property_kind,
                FieldKind::Double | FieldKind::Uint | FieldKind::Color | FieldKind::String
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
                .unwrap_or(RuntimeDataBindGraphValue::Number(0.0));
            if !artboard_property_binding_value_matches_kind(&default_value, property_kind)
                && !artboard_property_binding_allows_converted_default(
                    converter.as_ref(),
                    &default_value,
                    property_kind,
                )
            {
                return None;
            }

            Some(RuntimeArtboardPropertyBindingInstance {
                target_local_id,
                property_key,
                path: path.to_vec(),
                enum_value_names,
                converter_state: RuntimeDataBindGraphConverterState::for_converter(
                    converter.as_ref(),
                ),
                converter,
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
        ) | (RuntimeDataBindGraphValue::Color(_), FieldKind::Color)
            | (RuntimeDataBindGraphValue::String(_), FieldKind::String)
            | (RuntimeDataBindGraphValue::Enum(_), FieldKind::Uint)
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

pub(super) fn build_artboard_nested_host_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardNestedHostBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let artboard_id_key = property_key_for_name("NestedArtboard", "artboardId");
    let is_paused_key = property_key_for_name("NestedArtboard", "isPaused");
    let speed_key = property_key_for_name("NestedArtboard", "speed");
    let quantize_key = property_key_for_name("NestedArtboard", "quantize");

    file.artboard_data_binds(artboard_index)
        .into_iter()
        .filter_map(|data_bind| {
            if !data_bind_flags_apply_source_to_target(
                data_bind.object.uint_property("flags").unwrap_or(0),
            ) {
                return None;
            }
            let target = data_bind.target?;
            if target.type_name != "NestedArtboard" {
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
            })
        })
        .collect()
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

pub(super) fn apply_artboard_unbound_color_data_bind_defaults(
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
        .filter_map(|data_bind| {
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
                    if property_key_for_name("CustomPropertyNumber", "propertyValue")
                        == Some(property_key) =>
                {
                    RuntimeArtboardDataBindValueKind::Number
                }
                "CustomPropertyBoolean"
                    if property_key_for_name("CustomPropertyBoolean", "propertyValue")
                        == Some(property_key) =>
                {
                    RuntimeArtboardDataBindValueKind::Boolean
                }
                "CustomPropertyString"
                    if property_key_for_name("CustomPropertyString", "propertyValue")
                        == Some(property_key) =>
                {
                    RuntimeArtboardDataBindValueKind::String
                }
                "CustomPropertyColor"
                    if property_key_for_name("CustomPropertyColor", "propertyValue")
                        == Some(property_key) =>
                {
                    RuntimeArtboardDataBindValueKind::Color
                }
                "CustomPropertyEnum"
                    if property_key_for_name("CustomPropertyEnum", "propertyValue")
                        == Some(property_key) =>
                {
                    RuntimeArtboardDataBindValueKind::Enum
                }
                "CustomPropertyTrigger"
                    if property_key_for_name("CustomPropertyTrigger", "propertyValue")
                        == Some(property_key) =>
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
                target_local_id,
                property_key,
                path,
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

pub(super) fn build_artboard_layout_computed_bindings(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeArtboardLayoutComputedBindingInstance> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };

    file.artboard_data_binds(artboard_index)
        .into_iter()
        .filter_map(|data_bind| {
            if !data_bind_flags_apply_target_to_source(
                data_bind.object.uint_property("flags").unwrap_or(0),
            ) {
                return None;
            }
            let target = data_bind.target?;
            if target.type_name != "LayoutComponent" {
                return None;
            }
            let property_key =
                u16::try_from(data_bind.object.uint_property("propertyKey")?).ok()?;
            Some(RuntimeArtboardLayoutComputedBindingInstance {
                target_local_id: data_bind.target_local_id?,
                property: layout_computed_property_for_key(property_key)?,
                path: file.data_bind_context_source_path_ids_for_object(data_bind.object)?,
            })
        })
        .collect()
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
                path,
                enum_value_names,
            })
        })
        .collect()
}

fn runtime_enum_value_names_for_data_bind_path(
    file: &RuntimeFile,
    default_instance: Option<&rive_binary::RuntimeViewModelInstanceReference<'_>>,
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
) -> Option<rive_binary::RuntimeViewModelInstanceReference<'_>> {
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

    pub(crate) fn clear_default_text_property_context(&mut self) -> bool {
        let Some(text_property_key) = property_key_for_name("TextValueRun", "text") else {
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
        let mut changed = false;
        let paths = self
            .artboard_property_bindings
            .iter()
            .map(|binding| binding.path.clone())
            .chain(
                self.artboard_custom_property_bindings
                    .iter()
                    .map(|binding| binding.path.clone()),
            )
            .chain(
                self.artboard_solo_bindings
                    .iter()
                    .map(|binding| binding.path.clone()),
            )
            .chain(
                self.artboard_solo_source_bindings
                    .iter()
                    .map(|binding| binding.path.clone()),
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
                .find(|binding| binding.path == path)
                .map(|binding| binding.default_value.clone())
                .or_else(|| {
                    runtime_created_view_model_value_for_path(file, view_model_instance, &path)
                })
            else {
                continue;
            };
            if self.artboard_data_bind_values.get(&path) != Some(&value) {
                self.reset_artboard_property_formula_random_state_for_path(&path);
                self.artboard_data_bind_values.insert(path, value);
                changed = true;
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
            binding.source_list_size = match &source_value {
                RuntimeDataBindGraphValue::List { item_count } => Some(*item_count),
                _ => None,
            };
            binding.source_number_value = match source_value {
                RuntimeDataBindGraphValue::Number(value) => Some(value),
                _ => None,
            };
            binding.should_reset_instances = binding.source_number_value.is_some();
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

    pub(crate) fn set_artboard_data_bind_value_for_path(
        &mut self,
        path: &[u32],
        value: RuntimeDataBindGraphValue,
    ) -> bool {
        if self.artboard_data_bind_values.get(path) == Some(&value) {
            return false;
        }
        self.artboard_data_bind_values.insert(path.to_vec(), value);
        self.reset_artboard_property_formula_random_state_for_path(path);
        true
    }

    pub fn advance_artboard_data_binds_with_elapsed(&mut self, elapsed_seconds: f32) -> bool {
        let mut changed = false;
        for index in 0..self.artboard_custom_property_bindings.len() {
            changed |= self.update_artboard_custom_property_binding(index);
        }
        changed |= self.update_artboard_layout_computed_bindings();
        changed |= self.update_artboard_solo_source_bindings();
        changed |= self.apply_artboard_property_bindings();
        changed |= self.advance_artboard_property_binding_converters(elapsed_seconds);
        changed |= self.apply_artboard_property_bindings();
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
        changed |= self.apply_artboard_solo_bindings();
        changed |= self.apply_artboard_nested_host_bindings();
        changed |= self.sync_nested_child_artboard_data_contexts();
        changed
    }

    fn update_artboard_layout_computed_bindings(&mut self) -> bool {
        let Some(graph) = self.runtime_graph().cloned() else {
            return false;
        };
        let mut changed = false;
        for binding in self.artboard_layout_computed_bindings.clone() {
            changed |= self.update_artboard_layout_computed_binding(&binding, &graph);
        }
        changed
    }

    fn update_artboard_layout_computed_binding(
        &mut self,
        binding: &RuntimeArtboardLayoutComputedBindingInstance,
        graph: &ArtboardGraph,
    ) -> bool {
        // Mirrors C++ `src/data_bind/data_bind.cpp` targetSupportsPush:
        // Node computed* data binds are polled after layout settles.
        let Some(value) =
            self.runtime_layout_computed_property(binding.target_local_id, binding.property, graph)
        else {
            return false;
        };
        let value = RuntimeDataBindGraphValue::Number(value);
        if self.artboard_data_bind_values.get(&binding.path) == Some(&value) {
            return false;
        }
        self.artboard_data_bind_values
            .insert(binding.path.clone(), value);
        self.reset_artboard_property_formula_random_state_for_path(&binding.path);
        true
    }

    fn apply_artboard_property_bindings(&mut self) -> bool {
        let mut changed = false;
        for index in 0..self.artboard_property_bindings.len() {
            let Some((target_local_id, property_key, value)) =
                self.converted_artboard_property_binding_value(index)
            else {
                continue;
            };
            changed |=
                self.apply_artboard_property_binding_value(target_local_id, property_key, &value);
        }
        changed
    }

    fn converted_artboard_property_binding_value(
        &mut self,
        index: usize,
    ) -> Option<(usize, u16, RuntimeDataBindGraphValue)> {
        let binding = self.artboard_property_bindings.get_mut(index)?;
        let value = self.artboard_data_bind_values.get(&binding.path).cloned()?;
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
        Some((binding.target_local_id, binding.property_key, converted))
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

    fn advance_artboard_property_binding_converters(&mut self, elapsed_seconds: f32) -> bool {
        let mut changed = false;
        for binding in &mut self.artboard_property_bindings {
            let advance = binding
                .converter_state
                .advance_converter(binding.converter.as_ref(), elapsed_seconds);
            changed |= advance.changed;
        }
        changed
    }

    fn update_artboard_solo_source_bindings(&mut self) -> bool {
        let mut changed = false;
        for binding in self.artboard_solo_source_bindings.clone() {
            let Some(value) = self.artboard_solo_source_binding_value(&binding) else {
                continue;
            };
            if self.artboard_data_bind_values.get(&binding.path) == Some(&value) {
                continue;
            }
            self.artboard_data_bind_values
                .insert(binding.path.clone(), value);
            self.reset_artboard_property_formula_random_state_for_path(&binding.path);
            changed = true;
        }
        changed
    }

    fn artboard_solo_source_binding_value(
        &self,
        binding: &RuntimeArtboardSoloSourceBindingInstance,
    ) -> Option<RuntimeDataBindGraphValue> {
        let solo = self
            .solos
            .iter()
            .find(|solo| solo.local_id == binding.target_local_id)?;
        let active_component_id = usize::try_from(
            self.uint_property(binding.target_local_id, solo.active_component_property_key)?,
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
        let index = binding
            .enum_value_names
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

    fn update_artboard_custom_property_binding(&mut self, index: usize) -> bool {
        let Some(target_value) = self.artboard_custom_property_binding_target_value(index) else {
            return false;
        };
        let Some((path, value)) =
            self.convert_artboard_custom_property_binding_target_value(index, &target_value)
        else {
            return false;
        };
        if self.artboard_data_bind_values.get(&path) == Some(&value) {
            return false;
        }
        self.artboard_data_bind_values.insert(path.clone(), value);
        self.reset_artboard_property_formula_random_state_for_path(&path);
        true
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
    ) -> Option<(Vec<u32>, RuntimeDataBindGraphValue)> {
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
        let mut changed = false;
        for binding in self.artboard_solo_bindings.clone() {
            let Some(value) = self.artboard_data_bind_values.get(&binding.path).cloned() else {
                continue;
            };
            changed |= self.apply_artboard_solo_binding_value(&binding, &value);
        }
        changed
    }

    fn apply_artboard_solo_binding_value(
        &mut self,
        binding: &RuntimeArtboardSoloBindingInstance,
        value: &RuntimeDataBindGraphValue,
    ) -> bool {
        match value {
            RuntimeDataBindGraphValue::Number(value) => {
                self.set_solo_active_child_by_index(binding.target_local_id, *value)
            }
            RuntimeDataBindGraphValue::String(value) => {
                self.set_solo_active_child_by_name(binding.target_local_id, value)
            }
            RuntimeDataBindGraphValue::Enum(value) => {
                let Ok(index) = usize::try_from(*value) else {
                    return false;
                };
                let Some(name) = binding.enum_value_names.get(index) else {
                    return false;
                };
                self.set_solo_active_child_by_name(binding.target_local_id, name)
            }
            _ => false,
        }
    }

    fn apply_artboard_nested_host_bindings(&mut self) -> bool {
        let mut changed = false;
        for binding in self.artboard_nested_host_bindings.clone() {
            let Some(value) = self.artboard_data_bind_values.get(&binding.path).cloned() else {
                continue;
            };
            changed |= self.apply_artboard_nested_host_binding_value(&binding, &value);
        }
        changed
    }

    fn apply_artboard_nested_host_binding_value(
        &mut self,
        binding: &RuntimeArtboardNestedHostBindingInstance,
        value: &RuntimeDataBindGraphValue,
    ) -> bool {
        match (binding.property, value) {
            (
                RuntimeArtboardNestedHostProperty::ArtboardId { property_key },
                RuntimeDataBindGraphValue::Artboard(value),
            ) => {
                let changed = self.set_uint_property(binding.target_local_id, property_key, *value);
                changed || self.set_nested_artboard_artboard_id(binding.target_local_id, *value)
            }
            (
                RuntimeArtboardNestedHostProperty::IsPaused { property_key },
                RuntimeDataBindGraphValue::Boolean(value),
            ) => self.set_bool_property(binding.target_local_id, property_key, *value),
            (
                RuntimeArtboardNestedHostProperty::Speed { property_key },
                RuntimeDataBindGraphValue::Number(value),
            )
            | (
                RuntimeArtboardNestedHostProperty::Quantize { property_key },
                RuntimeDataBindGraphValue::Number(value),
            ) => self.set_double_property(binding.target_local_id, property_key, *value),
            _ => false,
        }
    }

    fn sync_nested_child_artboard_data_contexts(&mut self) -> bool {
        let host_locals = self.nested_artboards.keys().copied().collect::<Vec<_>>();
        let mut changed = false;
        for host_local_id in host_locals {
            let Some(bindings) = self
                .nested_artboards
                .get(&host_local_id)
                .map(|nested| nested.child.artboard_property_bindings.clone())
            else {
                continue;
            };
            if bindings.is_empty() {
                continue;
            }
            let updates = bindings
                .iter()
                .filter_map(|binding| {
                    self.stateful_nested_host_binding_value(host_local_id, binding)
                        .map(|value| (binding.path.clone(), value))
                })
                .collect::<Vec<_>>();
            if updates.is_empty() {
                continue;
            }
            let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
                continue;
            };
            let mut child_context_changed = false;
            for (path, value) in updates {
                if nested.child.artboard_data_bind_values.get(&path) == Some(&value) {
                    continue;
                }
                nested.child.artboard_data_bind_values.insert(path, value);
                child_context_changed = true;
            }
            if child_context_changed {
                changed = true;
                changed |= nested.child.advance_artboard_data_binds();
                nested.child.update_pass();
            }
        }
        changed
    }

    fn stateful_nested_host_binding_value(
        &self,
        host_local_id: usize,
        binding: &RuntimeArtboardPropertyBindingInstance,
    ) -> Option<RuntimeDataBindGraphValue> {
        let source_local = self.stateful_nested_host_value_local(host_local_id, &binding.path)?;
        match binding.default_value {
            RuntimeDataBindGraphValue::Number(_) => {
                let property_value_key =
                    property_key_for_name("ViewModelInstanceNumber", "propertyValue")?;
                self.double_property(source_local, property_value_key)
                    .map(RuntimeDataBindGraphValue::Number)
            }
            RuntimeDataBindGraphValue::String(_) => {
                let property_value_key =
                    property_key_for_name("ViewModelInstanceString", "propertyValue")?;
                self.string_property(source_local, property_value_key)
                    .map(|value| RuntimeDataBindGraphValue::String(value.to_vec()))
            }
            RuntimeDataBindGraphValue::Color(_) => {
                let property_value_key =
                    property_key_for_name("ViewModelInstanceColor", "propertyValue")?;
                self.color_property(source_local, property_value_key)
                    .map(RuntimeDataBindGraphValue::Color)
            }
            _ => None,
        }
    }

    fn stateful_nested_host_value_local(
        &self,
        host_local_id: usize,
        path: &[u32],
    ) -> Option<usize> {
        let (view_model_id, property_path) = path.split_first()?;
        let mut current_local =
            self.stateful_nested_host_view_model_instance_local(host_local_id, *view_model_id)?;
        for property_id in property_path {
            current_local =
                self.view_model_instance_value_child_local(current_local, *property_id)?;
        }
        Some(current_local)
    }

    fn stateful_nested_host_view_model_instance_local(
        &self,
        host_local_id: usize,
        view_model_id: u32,
    ) -> Option<usize> {
        let parent_key = property_key_for_name("Component", "parentId")?;
        let view_model_key = property_key_for_name("ViewModelInstance", "viewModelId")?;
        self.slots.iter().find_map(|slot| {
            (slot.type_name == Some("ViewModelInstance")
                && self.uint_property(slot.local_id, parent_key) == Some(host_local_id as u64)
                && self.uint_property(slot.local_id, view_model_key)
                    == Some(u64::from(view_model_id)))
            .then_some(slot.local_id)
        })
    }

    fn view_model_instance_value_child_local(
        &self,
        parent_local_id: usize,
        view_model_property_id: u32,
    ) -> Option<usize> {
        let parent_key = property_key_for_name("Component", "parentId")?;
        let property_key = property_key_for_name("ViewModelInstanceValue", "viewModelPropertyId")?;
        self.slots.iter().find_map(|slot| {
            let type_name = slot.type_name?;
            (type_name.starts_with("ViewModelInstance")
                && type_name != "ViewModelInstance"
                && self.uint_property(slot.local_id, parent_key) == Some(parent_local_id as u64)
                && self.uint_property(slot.local_id, property_key)
                    == Some(u64::from(view_model_property_id)))
            .then_some(slot.local_id)
        })
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
