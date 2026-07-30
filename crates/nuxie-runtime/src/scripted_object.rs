//! Occurrence-owned property state shared by pinned C++ `ScriptedObject`
//! implementations.
//!
//! `ScriptedObject::cloneProperties` clones each `ScriptInput` before binding
//! it to a concrete state-machine occurrence.  The cloned Core object remains
//! the DataBind target; the Lua table is only a projection updated by the
//! generated `*Changed` callback.

use crate::data_bind_graph::RuntimeDataBindGraphValue;
use crate::script_input_artboard::{
    RuntimeScriptInputArtboardApply, RuntimeScriptInputArtboardOccurrence,
};
use crate::script_input_viewmodel_property::ScriptInputViewModelPropertyPath;
use crate::scripting::{ScriptCoreString, ScriptListenerInputKind};
use crate::view_model_cell::RuntimeViewModelCellValue;
use nuxie_binary::{RuntimeFile, RuntimeObject};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeScriptInputTargetProperty {
    Name,
    ParentId,
    Value,
    Unsupported(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RuntimeScriptInputProperties {
    name: ScriptCoreString,
    parent_id: u32,
    /// Generated Core value. For ScriptInputArtboard this is the authored/live
    /// `artboardId`, not the separately retained referenced Artboard.
    value: Option<RuntimeDataBindGraphValue>,
    artboard: Option<RuntimeScriptInputArtboardOccurrence>,
    view_model_path: Option<ScriptInputViewModelPropertyPath>,
}

impl RuntimeScriptInputProperties {
    pub(crate) fn from_object(
        file: &RuntimeFile,
        input: &RuntimeObject,
        kind: ScriptListenerInputKind,
    ) -> Self {
        let value_key = value_property_key(kind);
        Self {
            name: ScriptCoreString::from_bytes(
                input.string_property_bytes("name").unwrap_or_default(),
            ),
            parent_id: input
                .uint_property("parentId")
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(u32::MAX),
            value: value_key.and_then(|key| authored_value(input, kind, key)),
            artboard: (kind == ScriptListenerInputKind::Artboard)
                .then(|| RuntimeScriptInputArtboardOccurrence::from_imported(file, input)),
            view_model_path: (kind == ScriptListenerInputKind::ViewModelProperty)
                .then(|| ScriptInputViewModelPropertyPath::from_imported(file, input))
                .flatten(),
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        name: impl Into<String>,
        parent_id: u32,
        value: Option<RuntimeDataBindGraphValue>,
    ) -> Self {
        Self {
            name: ScriptCoreString::from(name.into()),
            parent_id,
            value,
            artboard: None,
            view_model_path: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test_artboard(
        name: impl Into<String>,
        parent_id: u32,
        generated_artboard_id: u64,
        referenced_artboard_id: Option<u64>,
        file_attached: bool,
    ) -> Self {
        Self {
            name: ScriptCoreString::from(name.into()),
            parent_id,
            value: Some(RuntimeDataBindGraphValue::Artboard(generated_artboard_id)),
            artboard: Some(RuntimeScriptInputArtboardOccurrence::for_test(
                referenced_artboard_id,
                file_attached,
            )),
            view_model_path: None,
        }
    }

    /// Clone one fresh C++ CustomProperty occurrence.
    ///
    /// Most generated properties are ordinary value copies. The handwritten
    /// ScriptInputArtboard owner has asymmetric pointer/File clone behavior,
    /// so a derived `Clone` is reserved for transactional rehoming.
    pub(crate) fn clone_for_scripted_object(&self) -> Self {
        Self {
            name: self.name.clone(),
            parent_id: self.parent_id,
            value: self.value.clone(),
            artboard: self
                .artboard
                .as_ref()
                .map(RuntimeScriptInputArtboardOccurrence::clone_for_scripted_object),
            view_model_path: self.view_model_path.clone(),
        }
    }

    pub(crate) fn name(&self) -> &ScriptCoreString {
        &self.name
    }

    pub(crate) fn value(&self) -> Option<&RuntimeDataBindGraphValue> {
        self.value.as_ref()
    }

    /// Value projected into the ScriptedObject table.
    ///
    /// ScriptInputArtboard projects its retained reference, while reverse
    /// binding still reads the distinct generated Core `artboardId`.
    pub(crate) fn projection_value(
        &self,
        kind: ScriptListenerInputKind,
    ) -> Option<RuntimeDataBindGraphValue> {
        if kind == ScriptListenerInputKind::Artboard {
            return self
                .artboard
                .as_ref()
                .and_then(RuntimeScriptInputArtboardOccurrence::referenced_artboard_id)
                .map(RuntimeDataBindGraphValue::Artboard);
        }
        self.value.clone()
    }

    pub(crate) fn artboard_referenced_id(&self) -> Option<u64> {
        self.artboard
            .as_ref()
            .and_then(RuntimeScriptInputArtboardOccurrence::referenced_artboard_id)
    }

    pub(crate) fn view_model_path(&self) -> Option<&ScriptInputViewModelPropertyPath> {
        self.view_model_path.as_ref()
    }

    pub(crate) fn property_for_key(
        kind: ScriptListenerInputKind,
        property_key: u32,
    ) -> RuntimeScriptInputTargetProperty {
        if property_key_for_name(script_input_type_name(kind), "name") == Some(property_key) {
            RuntimeScriptInputTargetProperty::Name
        } else if property_key_for_name(script_input_type_name(kind), "parentId")
            == Some(property_key)
        {
            RuntimeScriptInputTargetProperty::ParentId
        } else if value_property_key(kind) == Some(property_key) {
            RuntimeScriptInputTargetProperty::Value
        } else {
            RuntimeScriptInputTargetProperty::Unsupported(property_key)
        }
    }

    pub(crate) fn target_value(
        &self,
        property: RuntimeScriptInputTargetProperty,
        source: Option<&RuntimeViewModelCellValue>,
    ) -> Option<RuntimeDataBindGraphValue> {
        match property {
            RuntimeScriptInputTargetProperty::Name => Some(RuntimeDataBindGraphValue::String(
                self.name.as_bytes().to_vec(),
            )),
            RuntimeScriptInputTargetProperty::ParentId => {
                Some(uint_target_value(self.parent_id, source))
            }
            RuntimeScriptInputTargetProperty::Value => self.value.clone(),
            RuntimeScriptInputTargetProperty::Unsupported(_) => None,
        }
    }

    /// Apply a converted DataBind value to the cloned Core target.
    ///
    /// The return value says whether the generated ScriptInput value callback
    /// must project the new value into the Lua table. `name` and `parentId`
    /// remain ordinary inherited Core properties and do not invoke a typed
    /// ScriptInput callback.
    pub(crate) fn apply_target(
        &mut self,
        file: &RuntimeFile,
        kind: ScriptListenerInputKind,
        property: RuntimeScriptInputTargetProperty,
        value: RuntimeDataBindGraphValue,
    ) -> RuntimeScriptInputTargetApply {
        match property {
            RuntimeScriptInputTargetProperty::Name => {
                let RuntimeDataBindGraphValue::String(value) = value else {
                    return RuntimeScriptInputTargetApply::Rejected;
                };
                let value = ScriptCoreString::from_bytes(value);
                if self.name == value {
                    RuntimeScriptInputTargetApply::Unchanged
                } else {
                    self.name = value;
                    RuntimeScriptInputTargetApply::ChangedWithoutTableProjection
                }
            }
            RuntimeScriptInputTargetProperty::ParentId => {
                let Some(value) = uint_from_graph_value(&value) else {
                    return RuntimeScriptInputTargetApply::Rejected;
                };
                if self.parent_id == value {
                    RuntimeScriptInputTargetApply::Unchanged
                } else {
                    self.parent_id = value;
                    RuntimeScriptInputTargetApply::ChangedWithoutTableProjection
                }
            }
            RuntimeScriptInputTargetProperty::Value => {
                let Some(next) = coerce_script_input_value(kind, value) else {
                    return RuntimeScriptInputTargetApply::Rejected;
                };
                let Some(current) = self.value.as_ref() else {
                    return RuntimeScriptInputTargetApply::Rejected;
                };
                if current == &next {
                    RuntimeScriptInputTargetApply::Unchanged
                } else {
                    self.value = Some(next.clone());
                    if kind == ScriptListenerInputKind::Artboard {
                        let RuntimeDataBindGraphValue::Artboard(artboard_id) = next else {
                            unreachable!("ScriptInputArtboard coercion returns an Artboard value")
                        };
                        self.artboard
                            .as_mut()
                            .map(|artboard| {
                                map_artboard_apply(
                                    artboard.apply_artboard_id_changed(file, artboard_id),
                                )
                            })
                            .unwrap_or(RuntimeScriptInputTargetApply::ChangedWithoutTableProjection)
                    } else {
                        RuntimeScriptInputTargetApply::ChangedWithTableProjection
                    }
                }
            }
            RuntimeScriptInputTargetProperty::Unsupported(_) => {
                RuntimeScriptInputTargetApply::Rejected
            }
        }
    }

    /// Apply the Artboard-specialized ContextValue path. This updates only
    /// the retained reference; the generated Core `artboardId` remains
    /// untouched and therefore remains the target-to-source value.
    pub(crate) fn apply_artboard_source(
        &mut self,
        file: &RuntimeFile,
        artboard_id: u64,
    ) -> RuntimeScriptInputTargetApply {
        self.artboard
            .as_mut()
            .map(|artboard| map_artboard_apply(artboard.apply_artboard_source(file, artboard_id)))
            .unwrap_or(RuntimeScriptInputTargetApply::Rejected)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeScriptInputTargetApply {
    Rejected,
    Unchanged,
    ChangedWithoutTableProjection,
    ChangedWithTableProjection,
}

fn map_artboard_apply(apply: RuntimeScriptInputArtboardApply) -> RuntimeScriptInputTargetApply {
    match apply {
        RuntimeScriptInputArtboardApply::Rejected => RuntimeScriptInputTargetApply::Rejected,
        RuntimeScriptInputArtboardApply::ChangedWithoutProjection => {
            RuntimeScriptInputTargetApply::ChangedWithoutTableProjection
        }
        RuntimeScriptInputArtboardApply::Project(_) => {
            RuntimeScriptInputTargetApply::ChangedWithTableProjection
        }
    }
}

fn property_key_for_name(type_name: &str, name: &str) -> Option<u32> {
    crate::properties::property_key_for_name(type_name, name).map(u32::from)
}

fn value_property_key(kind: ScriptListenerInputKind) -> Option<u32> {
    match kind {
        ScriptListenerInputKind::Boolean => crate::script_input_boolean::value_property_key(),
        ScriptListenerInputKind::Number => crate::script_input_number::value_property_key(),
        ScriptListenerInputKind::Color => crate::script_input_color::value_property_key(),
        ScriptListenerInputKind::String => crate::script_input_string::value_property_key(),
        ScriptListenerInputKind::Trigger => crate::script_input_trigger::value_property_key(),
        ScriptListenerInputKind::Artboard => crate::script_input_artboard::value_property_key(),
        ScriptListenerInputKind::ViewModelProperty => {
            crate::script_input_viewmodel_property::value_property_key()
        }
    }
    .map(u32::from)
}

fn authored_value(
    input: &RuntimeObject,
    kind: ScriptListenerInputKind,
    property_key: u32,
) -> Option<RuntimeDataBindGraphValue> {
    match kind {
        ScriptListenerInputKind::Boolean => {
            crate::script_input_boolean::authored_target(input, property_key)
        }
        ScriptListenerInputKind::Number => {
            crate::script_input_number::authored_target(input, property_key)
        }
        ScriptListenerInputKind::Color => {
            crate::script_input_color::authored_target(input, property_key)
        }
        ScriptListenerInputKind::String => {
            crate::script_input_string::authored_target(input, property_key)
        }
        ScriptListenerInputKind::Trigger => {
            crate::script_input_trigger::authored_target(input, property_key)
        }
        ScriptListenerInputKind::Artboard => {
            crate::script_input_artboard::authored_target(input, property_key)
        }
        ScriptListenerInputKind::ViewModelProperty => {
            crate::script_input_viewmodel_property::authored_target(input, property_key)
        }
    }
}

fn script_input_type_name(kind: ScriptListenerInputKind) -> &'static str {
    match kind {
        ScriptListenerInputKind::Boolean => "ScriptInputBoolean",
        ScriptListenerInputKind::Number => "ScriptInputNumber",
        ScriptListenerInputKind::Color => "ScriptInputColor",
        ScriptListenerInputKind::String => "ScriptInputString",
        ScriptListenerInputKind::Trigger => "ScriptInputTrigger",
        ScriptListenerInputKind::Artboard => "ScriptInputArtboard",
        ScriptListenerInputKind::ViewModelProperty => "ScriptInputViewModelProperty",
    }
}

fn uint_target_value(
    value: u32,
    source: Option<&RuntimeViewModelCellValue>,
) -> RuntimeDataBindGraphValue {
    match source {
        Some(RuntimeViewModelCellValue::Number(_)) => {
            RuntimeDataBindGraphValue::Number(value as f32)
        }
        Some(RuntimeViewModelCellValue::SymbolListIndex(_)) => {
            RuntimeDataBindGraphValue::SymbolListIndex(u64::from(value))
        }
        Some(RuntimeViewModelCellValue::AssetImage(_))
        | Some(RuntimeViewModelCellValue::AssetFont(_)) => {
            RuntimeDataBindGraphValue::Asset(u64::from(value))
        }
        Some(RuntimeViewModelCellValue::Artboard(_)) => {
            RuntimeDataBindGraphValue::Artboard(u64::from(value))
        }
        Some(RuntimeViewModelCellValue::Trigger(_)) => {
            RuntimeDataBindGraphValue::Trigger(u64::from(value))
        }
        _ => RuntimeDataBindGraphValue::Enum(u64::from(value)),
    }
}

fn uint_from_graph_value(value: &RuntimeDataBindGraphValue) -> Option<u32> {
    let value = match value {
        RuntimeDataBindGraphValue::Number(value) => {
            if value.is_nan() || *value <= 0.0 {
                0
            } else {
                value.round().min(u32::MAX as f32) as u64
            }
        }
        RuntimeDataBindGraphValue::Enum(value)
        | RuntimeDataBindGraphValue::SymbolListIndex(value)
        | RuntimeDataBindGraphValue::Asset(value)
        | RuntimeDataBindGraphValue::Artboard(value)
        | RuntimeDataBindGraphValue::Trigger(value) => *value,
        _ => return None,
    };
    Some(u32::try_from(value).unwrap_or(u32::MAX))
}

/// Convert the source `DataValue` payload according to the generated Core
/// field owned by the concrete ScriptInput. This is deliberately not a
/// source-kind equality check:
///
/// - `ContextValueSymbolListIndex` writes either `float` or `uint32_t`;
/// - Number writes either `float` or rounded/clamped `uint32_t`;
/// - Enum, Trigger, Asset, and non-referencer Artboard sources write
///   `uint32_t`.
///
/// (`context_value_number.cpp:16-39`;
/// `context_value_symbol_list_index.cpp:17-35`;
/// `context_value_{enum,trigger,asset_image,asset_font,artboard}.cpp`).
fn coerce_script_input_value(
    kind: ScriptListenerInputKind,
    value: RuntimeDataBindGraphValue,
) -> Option<RuntimeDataBindGraphValue> {
    match kind {
        ScriptListenerInputKind::Boolean => match value {
            RuntimeDataBindGraphValue::Boolean(value) => {
                Some(RuntimeDataBindGraphValue::Boolean(value))
            }
            _ => None,
        },
        ScriptListenerInputKind::Number => match value {
            RuntimeDataBindGraphValue::Number(value) => {
                Some(RuntimeDataBindGraphValue::Number(value))
            }
            RuntimeDataBindGraphValue::SymbolListIndex(value) => {
                Some(RuntimeDataBindGraphValue::Number(value as f32))
            }
            _ => None,
        },
        ScriptListenerInputKind::Color => match value {
            RuntimeDataBindGraphValue::Color(value) => {
                Some(RuntimeDataBindGraphValue::Color(value))
            }
            _ => None,
        },
        ScriptListenerInputKind::String => match value {
            RuntimeDataBindGraphValue::String(value) => {
                Some(RuntimeDataBindGraphValue::String(value))
            }
            _ => None,
        },
        ScriptListenerInputKind::Trigger => uint_from_graph_value(&value)
            .map(u64::from)
            .map(RuntimeDataBindGraphValue::Trigger),
        ScriptListenerInputKind::Artboard => uint_from_graph_value(&value)
            .map(u64::from)
            .map(RuntimeDataBindGraphValue::Artboard),
        ScriptListenerInputKind::ViewModelProperty => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_input_number_accepts_symbol_list_index_like_cpp_core_double() {
        assert_eq!(
            coerce_script_input_value(
                ScriptListenerInputKind::Number,
                RuntimeDataBindGraphValue::SymbolListIndex(16_777_217),
            ),
            Some(RuntimeDataBindGraphValue::Number(16_777_217_u64 as f32)),
        );
    }

    #[test]
    fn script_input_uint_fields_accept_every_cpp_uint_source_family() {
        let sources = [
            RuntimeDataBindGraphValue::Number(2.6),
            RuntimeDataBindGraphValue::Enum(3),
            RuntimeDataBindGraphValue::SymbolListIndex(4),
            RuntimeDataBindGraphValue::Trigger(5),
            RuntimeDataBindGraphValue::Asset(6),
            RuntimeDataBindGraphValue::Artboard(7),
        ];
        let expected = [3, 3, 4, 5, 6, 7];
        for (source, expected) in sources.into_iter().zip(expected) {
            assert_eq!(
                coerce_script_input_value(ScriptListenerInputKind::Trigger, source.clone()),
                Some(RuntimeDataBindGraphValue::Trigger(expected)),
            );
            assert_eq!(
                coerce_script_input_value(ScriptListenerInputKind::Artboard, source),
                Some(RuntimeDataBindGraphValue::Artboard(expected)),
            );
        }
    }

    #[test]
    fn script_input_generated_field_rejects_incompatible_core_field_kind() {
        assert_eq!(
            coerce_script_input_value(
                ScriptListenerInputKind::Boolean,
                RuntimeDataBindGraphValue::Number(1.0),
            ),
            None,
        );
        assert_eq!(
            coerce_script_input_value(
                ScriptListenerInputKind::Number,
                RuntimeDataBindGraphValue::Enum(1),
            ),
            None,
        );
        assert_eq!(
            coerce_script_input_value(
                ScriptListenerInputKind::String,
                RuntimeDataBindGraphValue::Color(1),
            ),
            None,
        );
    }

    #[test]
    fn view_model_property_path_is_deep_cloned_per_scripted_object_occurrence() {
        let definition = RuntimeScriptInputProperties {
            name: ScriptCoreString::from("child"),
            parent_id: 7,
            value: None,
            artboard: None,
            view_model_path: Some(ScriptInputViewModelPropertyPath {
                path_ids: vec![1, 2, 3],
                resolved_path_ids: vec![4, 5, 6],
                is_relative: true,
            }),
        };
        let mut first = definition.clone_for_scripted_object();
        let second = definition.clone_for_scripted_object();

        first
            .view_model_path
            .as_mut()
            .expect("first cloned path")
            .path_ids[0] = 99;

        assert_eq!(
            definition
                .view_model_path()
                .expect("definition path")
                .path_ids,
            vec![1, 2, 3]
        );
        assert_eq!(
            second
                .view_model_path()
                .expect("second cloned path")
                .path_ids,
            vec![1, 2, 3],
            "ScriptInputViewModelProperty::copyDataBindPathIds deep-clones the path for each occurrence (`script_input_viewmodel_property_base.hpp:41-45`; `data_bind_path_referencer.cpp:15-21`)"
        );
    }
}
