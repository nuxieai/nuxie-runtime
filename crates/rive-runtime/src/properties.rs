use rive_binary::{FieldValue, RuntimeFile, RuntimeObject};
use rive_graph::ArtboardGraph;
use rive_schema::{
    FieldKind, StoredFieldInitializer, definition_by_name, property_by_key_in_hierarchy,
};
use std::sync::OnceLock;

use crate::components::TransformProperty;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeArtboardDimensions {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) origin_x: f32,
    pub(crate) origin_y: f32,
    pub(crate) clip: bool,
}

impl RuntimeArtboardDimensions {
    pub(crate) fn from_object(object: Option<&RuntimeObject>) -> Self {
        let width = object
            .and_then(|object| object.double_property("width"))
            .unwrap_or(0.0);
        let height = object
            .and_then(|object| object.double_property("height"))
            .unwrap_or(0.0);
        let origin_x = object
            .and_then(|object| object.double_property("originX"))
            .unwrap_or(0.0);
        let origin_y = object
            .and_then(|object| object.double_property("originY"))
            .unwrap_or(0.0);
        let clip = object
            .and_then(|object| object.bool_property("clip"))
            .unwrap_or(true);
        Self {
            width,
            height,
            origin_x,
            origin_y,
            clip,
        }
    }
}

pub(crate) fn artboard_index_for_graph(file: &RuntimeFile, graph: &ArtboardGraph) -> Option<usize> {
    file.artboards()
        .into_iter()
        .position(|artboard| artboard.id == graph.global_id)
}

pub(crate) fn runtime_object_field_kind_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<FieldKind> {
    property_by_key_in_hierarchy(object.type_key, property_key)
        .map(|(_, property)| property.runtime_type)
}

fn runtime_object_property_value_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<&FieldValue> {
    object
        .properties
        .iter()
        .rev()
        .find(|property| property.key == property_key)
        .map(|property| &property.value)
}

pub(crate) fn runtime_object_explicit_double_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<f32> {
    runtime_object_property_value_by_key(object, property_key).and_then(FieldValue::as_double)
}

pub(crate) fn runtime_object_explicit_uint_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<u64> {
    runtime_object_property_value_by_key(object, property_key).and_then(FieldValue::as_uint)
}

pub(crate) fn runtime_object_explicit_bool_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<bool> {
    runtime_object_property_value_by_key(object, property_key).and_then(FieldValue::as_bool)
}

fn runtime_object_stored_field_initializer_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<StoredFieldInitializer> {
    let (_, property) = property_by_key_in_hierarchy(object.type_key, property_key)?;
    if object.type_name == "Artboard" && property.name == "clip" {
        return Some(StoredFieldInitializer::Bool(true));
    }
    (*property).stored_field_initializer()
}

pub(crate) fn runtime_object_double_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<f32> {
    if let Some(value) = runtime_object_property_value_by_key(object, property_key) {
        return value.as_double();
    }

    match runtime_object_stored_field_initializer_by_key(object, property_key)? {
        StoredFieldInitializer::Double(value) => Some(value),
        _ => None,
    }
}

pub(crate) fn runtime_object_uint_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<u64> {
    if let Some(value) = runtime_object_property_value_by_key(object, property_key) {
        return value.as_uint();
    }

    match runtime_object_stored_field_initializer_by_key(object, property_key)? {
        StoredFieldInitializer::Uint(value) => Some(u64::from(value)),
        _ => None,
    }
}

pub(crate) fn runtime_object_bool_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<bool> {
    if let Some(value) = runtime_object_property_value_by_key(object, property_key) {
        return value.as_bool();
    }

    match runtime_object_stored_field_initializer_by_key(object, property_key)? {
        StoredFieldInitializer::Bool(value) => Some(value),
        _ => None,
    }
}

pub(crate) fn runtime_object_color_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<u32> {
    if let Some(value) = runtime_object_property_value_by_key(object, property_key) {
        return value.as_color();
    }

    match runtime_object_stored_field_initializer_by_key(object, property_key)? {
        StoredFieldInitializer::Color(value) => Some(value),
        _ => None,
    }
}

pub(crate) fn runtime_object_string_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<Vec<u8>> {
    runtime_object_string_property_bytes_by_key(object, property_key).map(|value| value.to_vec())
}

fn runtime_object_string_property_bytes_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<&[u8]> {
    if let Some(value) = runtime_object_property_value_by_key(object, property_key) {
        return match value {
            FieldValue::String(value) => Some(value.as_bytes()),
            FieldValue::Bytes(value) => Some(value.as_bytes()),
            _ => None,
        };
    }

    let (_, property) = property_by_key_in_hierarchy(object.type_key, property_key)?;
    match (*property).stored_field_initializer()? {
        StoredFieldInitializer::String(value) if property.runtime_type == FieldKind::String => {
            Some(value.as_bytes())
        }
        _ => None,
    }
}

pub(crate) fn transform_property_for_key(property_key: u16) -> Option<TransformProperty> {
    [
        TransformProperty::X.property_key_for_type("Node"),
        TransformProperty::Y.property_key_for_type("Node"),
        TransformProperty::X.property_key_for_type("RootBone"),
        TransformProperty::Y.property_key_for_type("RootBone"),
        TransformProperty::Rotation.property_key_for_type("TransformComponent"),
        TransformProperty::ScaleX.property_key_for_type("TransformComponent"),
        TransformProperty::ScaleY.property_key_for_type("TransformComponent"),
        TransformProperty::Opacity.property_key_for_type("TransformComponent"),
        TransformProperty::Opacity.property_key_for_type("Artboard"),
    ]
    .into_iter()
    .zip([
        TransformProperty::X,
        TransformProperty::Y,
        TransformProperty::X,
        TransformProperty::Y,
        TransformProperty::Rotation,
        TransformProperty::ScaleX,
        TransformProperty::ScaleY,
        TransformProperty::Opacity,
        TransformProperty::Opacity,
    ])
    .find_map(|(key, property)| (key == Some(property_key)).then_some(property))
}

pub(crate) fn solid_color_value_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "SolidColor", "colorValue")
}

pub(crate) fn shape_paint_is_visible_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "ShapePaint", "isVisible")
}

pub(crate) fn solo_active_component_id_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Solo", "activeComponentId")
}

pub(crate) fn layout_component_style_display_value_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "LayoutComponentStyle", "displayValue")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeLayoutComputedProperty {
    LocalX,
    LocalY,
    WorldX,
    WorldY,
    RootX,
    RootY,
    Width,
    Height,
}

pub(crate) fn layout_computed_property_for_key(
    property_key: u16,
) -> Option<RuntimeLayoutComputedProperty> {
    [
        ("computedLocalX", RuntimeLayoutComputedProperty::LocalX),
        ("computedLocalY", RuntimeLayoutComputedProperty::LocalY),
        ("computedWorldX", RuntimeLayoutComputedProperty::WorldX),
        ("computedWorldY", RuntimeLayoutComputedProperty::WorldY),
        ("computedRootX", RuntimeLayoutComputedProperty::RootX),
        ("computedRootY", RuntimeLayoutComputedProperty::RootY),
        ("computedWidth", RuntimeLayoutComputedProperty::Width),
        ("computedHeight", RuntimeLayoutComputedProperty::Height),
    ]
    .into_iter()
    .find_map(|(property_name, property)| {
        (property_key_for_name("Node", property_name) == Some(property_key)).then_some(property)
    })
}

pub(crate) const JOYSTICK_FLAG_INVERT_X: u64 = 1 << 0;
pub(crate) const JOYSTICK_FLAG_INVERT_Y: u64 = 1 << 1;

pub(crate) fn joystick_x_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Joystick", "x")
}

pub(crate) fn joystick_y_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Joystick", "y")
}

pub(crate) fn joystick_flags_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Joystick", "joystickFlags")
}

pub(crate) fn property_key_for_name(type_name: &str, property_name: &str) -> Option<u16> {
    let definition = definition_by_name(type_name)?;
    if let Some(property) = definition
        .properties
        .iter()
        .find(|property| property.name == property_name)
    {
        return Some(property.key.int);
    }

    for ancestor in definition.ancestors {
        let ancestor = definition_by_name(ancestor)?;
        if let Some(property) = ancestor
            .properties
            .iter()
            .find(|property| property.name == property_name)
        {
            return Some(property.key.int);
        }
    }

    None
}

pub(crate) fn cached_property_key_for_name(
    slot: &'static OnceLock<Option<u16>>,
    type_name: &'static str,
    property_name: &'static str,
) -> Option<u16> {
    *slot.get_or_init(|| property_key_for_name(type_name, property_name))
}

pub(crate) fn mix_value(current: f32, value: f32, mix: f32) -> f32 {
    if mix == 1.0 {
        value
    } else {
        current * (1.0 - mix) + value * mix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_bone_xy_keys_are_transform_properties() {
        let x = property_key_for_name("RootBone", "x").expect("RootBone.x key");
        let y = property_key_for_name("RootBone", "y").expect("RootBone.y key");

        assert_eq!(transform_property_for_key(x), Some(TransformProperty::X));
        assert_eq!(transform_property_for_key(y), Some(TransformProperty::Y));
    }
}
