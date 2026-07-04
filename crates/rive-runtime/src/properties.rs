use rive_binary::{RuntimeFile, RuntimeObject};
use rive_graph::ArtboardGraph;
use rive_schema::{FieldKind, definition_by_name, definition_by_type_key};

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

fn runtime_property_name_by_key(object: &RuntimeObject, property_key: u16) -> Option<&'static str> {
    definition_by_type_key(object.type_key)?
        .property_by_key_in_hierarchy(property_key)
        .map(|property| property.name)
}

pub(crate) fn runtime_object_field_kind_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<FieldKind> {
    definition_by_type_key(object.type_key)?
        .property_by_key_in_hierarchy(property_key)
        .map(|property| property.runtime_type)
}

pub(crate) fn runtime_object_double_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<f32> {
    object.double_property(runtime_property_name_by_key(object, property_key)?)
}

pub(crate) fn runtime_object_uint_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<u64> {
    object.uint_property(runtime_property_name_by_key(object, property_key)?)
}

pub(crate) fn runtime_object_bool_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<bool> {
    object.bool_property(runtime_property_name_by_key(object, property_key)?)
}

pub(crate) fn runtime_object_color_property_by_key(
    object: &RuntimeObject,
    property_key: u16,
) -> Option<u32> {
    object.color_property(runtime_property_name_by_key(object, property_key)?)
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
    let property =
        definition_by_type_key(object.type_key)?.property_by_key_in_hierarchy(property_key)?;
    match property.runtime_type {
        FieldKind::String => object.string_property_bytes(property.name),
        FieldKind::Bytes => object.bytes_property(property.name),
        _ => None,
    }
}

pub(crate) fn transform_property_for_key(property_key: u16) -> Option<TransformProperty> {
    [
        ("Node", "x", TransformProperty::X),
        ("Node", "y", TransformProperty::Y),
        (
            "TransformComponent",
            "rotation",
            TransformProperty::Rotation,
        ),
        ("TransformComponent", "scaleX", TransformProperty::ScaleX),
        ("TransformComponent", "scaleY", TransformProperty::ScaleY),
        ("TransformComponent", "opacity", TransformProperty::Opacity),
        ("Artboard", "opacity", TransformProperty::Opacity),
    ]
    .into_iter()
    .find_map(|(type_name, property_name, property)| {
        (property_key_for_name(type_name, property_name) == Some(property_key)).then_some(property)
    })
}

pub(crate) fn solid_color_value_property_key() -> Option<u16> {
    property_key_for_name("SolidColor", "colorValue")
}

pub(crate) fn shape_paint_is_visible_property_key() -> Option<u16> {
    property_key_for_name("ShapePaint", "isVisible")
}

pub(crate) fn solo_active_component_id_property_key() -> Option<u16> {
    property_key_for_name("Solo", "activeComponentId")
}

pub(crate) const JOYSTICK_FLAG_INVERT_X: u64 = 1 << 0;
pub(crate) const JOYSTICK_FLAG_INVERT_Y: u64 = 1 << 1;

pub(crate) fn joystick_x_property_key() -> Option<u16> {
    property_key_for_name("Joystick", "x")
}

pub(crate) fn joystick_y_property_key() -> Option<u16> {
    property_key_for_name("Joystick", "y")
}

pub(crate) fn joystick_flags_property_key() -> Option<u16> {
    property_key_for_name("Joystick", "joystickFlags")
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

pub(crate) fn mix_value(current: f32, value: f32, mix: f32) -> f32 {
    if mix == 1.0 {
        value
    } else {
        current * (1.0 - mix) + value * mix
    }
}
