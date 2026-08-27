//! Direct Rust owner for pinned C++ `src/custom_property_container.cpp`.
//!
//! Owns imported container membership and the occurrence-local custom-property
//! DataBind collection; scalar storage and dirt remain in their generic owners.

use std::sync::Arc;

use nuxie_binary::RuntimeFile;
use nuxie_graph::ArtboardGraph;
use nuxie_schema::FieldKind;

use crate::artboard_data_bind::{
    RuntimeArtboardDataBindValueKind, artboard_default_view_model_instance,
    runtime_artboard_data_bind_default_value_for_kind,
    runtime_created_view_model_value_for_declared_path,
    runtime_created_view_model_value_for_source, runtime_data_bind_property_key_for_name,
    runtime_type_is_a, shared_data_bind_path,
};
use crate::data_bind_graph::{
    RuntimeDataBindGraphConverterBuildCache, RuntimeDataBindGraphConverterState,
    runtime_data_bind_graph_converter_with_cache,
};
use crate::properties::{artboard_index_for_graph, property_key_for_name};
use crate::{
    ArtboardInstance, RuntimeDataBindGraphConverter, RuntimeDataBindGraphValue,
    data_bind_flags_apply_target_to_source,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeEventPropertyValue {
    Number(f32),
    Bool(bool),
    String(Vec<u8>),
    Color(u32),
    Enum(u64),
    Trigger(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeEventProperty {
    pub name: Option<String>,
    pub value: RuntimeEventPropertyValue,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeArtboardCustomPropertyBindingInstance {
    pub(crate) data_bind_index: usize,
    pub(crate) target_local_id: usize,
    pub(crate) property_key: u16,
    pub(crate) path: Arc<[u32]>,
    pub(crate) path_is_name_based: bool,
    pub(crate) owned_context_source_path: Option<Vec<usize>>,
    pub(crate) flags: u64,
    pub(crate) value_kind: RuntimeArtboardDataBindValueKind,
    pub(crate) converter: Option<RuntimeDataBindGraphConverter>,
    pub(crate) converter_state: RuntimeDataBindGraphConverterState,
    pub(crate) default_value: RuntimeDataBindGraphValue,
}

impl ArtboardInstance {
    /// Rust graph correspondence for `syncCustomProperties` and
    /// `customProperties`: rebuild the occurrence-local collection from the
    /// container's direct children, preserving child order and accepting every
    /// type derived from `CustomProperty`.
    ///
    /// The C++ mixin keeps a second pointer vector and mutates it through
    /// `addProperty`/`removeProperty`. Rust's component graph is the retained
    /// owner, so structural insertion/removal is observed by rebuilding this
    /// projection instead of duplicating membership state.
    fn synced_custom_property_local_ids(&self, container_local_id: usize) -> Vec<usize> {
        let Some(container) = self.component_handle(container_local_id) else {
            return Vec::new();
        };
        (0..self.component_child_len(container))
            .filter_map(|index| self.component_child_at(container, index))
            .filter_map(|child| {
                let component = self.objects.component(child)?;
                nuxie_schema::definition_by_name(component.type_name)
                    .is_some_and(|definition| definition.is_a("CustomProperty"))
                    .then_some(component.local_id)
            })
            .collect()
    }

    /// Snapshot authored custom properties attached to one event, preserving
    /// their component/local order.
    pub fn event_properties(&self, event_local_id: usize) -> Vec<RuntimeEventProperty> {
        self.synced_custom_property_local_ids(event_local_id)
            .into_iter()
            .filter_map(|local_id| self.component_handle(local_id))
            .filter_map(|handle| self.objects.component(handle))
            .filter_map(|component| {
                let key = property_key_for_name(component.type_name, "propertyValue")?;
                let definition = nuxie_schema::definition_by_name(component.type_name)?;
                let value = if definition.is_a("CustomPropertyNumber") {
                    RuntimeEventPropertyValue::Number(
                        self.double_property(component.local_id, key)?,
                    )
                } else if definition.is_a("CustomPropertyBoolean") {
                    RuntimeEventPropertyValue::Bool(self.bool_property(component.local_id, key)?)
                } else if definition.is_a("CustomPropertyString") {
                    RuntimeEventPropertyValue::String(
                        self.string_property(component.local_id, key)?.to_vec(),
                    )
                } else if definition.is_a("CustomPropertyColor") {
                    RuntimeEventPropertyValue::Color(self.color_property(component.local_id, key)?)
                } else if definition.is_a("CustomPropertyEnum") {
                    RuntimeEventPropertyValue::Enum(self.uint_property(component.local_id, key)?)
                } else if definition.is_a("CustomPropertyTrigger") {
                    RuntimeEventPropertyValue::Trigger(self.uint_property(component.local_id, key)?)
                } else {
                    return None;
                };
                let name_key = property_key_for_name(component.type_name, "name")?;
                Some(RuntimeEventProperty {
                    name: self
                        .string_property(component.local_id, name_key)
                        .map(|value| String::from_utf8_lossy(value).into_owned())
                        .filter(|name| !name.is_empty()),
                    value,
                })
            })
            .collect()
    }
}

pub(crate) fn build_artboard_custom_property_bindings<'a>(
    file: &'a RuntimeFile,
    graph: &ArtboardGraph,
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
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
                "CustomPropertyTrigger" | "ViewModelInstanceTrigger"
                    if runtime_data_bind_property_key_for_name(
                        target.type_name,
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
            let converter = runtime_data_bind_graph_converter_with_cache(
                file,
                data_bind.object,
                converter_cache,
            );
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
