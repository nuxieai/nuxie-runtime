use anyhow::Result;
use nuxie_binary::RuntimeObject;
use nuxie_render_api::{StrokeCap as RenderStrokeCap, StrokeJoin as RenderStrokeJoin};

use crate::draw::runtime_draw_property_key_for_name;
use crate::properties::{
    property_key_for_name, runtime_object_explicit_double_property_by_key,
    runtime_object_explicit_uint_property_by_key,
};
use crate::{ArtboardInstance, ComponentDirt};

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("Stroke", "thickness") != Some(property_key) {
        return None;
    }
    Some(artboard.add_dirt(local_id, ComponentDirt::PAINT, false))
}

pub(crate) fn uint_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if !["cap", "join"]
        .into_iter()
        .any(|name| property_key_for_name("Stroke", name) == Some(property_key))
    {
        return None;
    }
    Some(artboard.add_dirt(local_id, ComponentDirt::PAINT, false))
}

pub(crate) fn runtime_stroke_cap(value: u64) -> Result<RenderStrokeCap> {
    Ok(match value {
        0 => RenderStrokeCap::Butt,
        1 => RenderStrokeCap::Round,
        2 => RenderStrokeCap::Square,
        _ => anyhow::bail!("unsupported stroke cap {value}"),
    })
}

pub(crate) fn runtime_stroke_join(value: u64) -> Result<RenderStrokeJoin> {
    Ok(match value {
        0 => RenderStrokeJoin::Miter,
        1 => RenderStrokeJoin::Round,
        2 => RenderStrokeJoin::Bevel,
        _ => anyhow::bail!("unsupported stroke join {value}"),
    })
}

pub(crate) fn runtime_stroke_thickness(
    instance: &ArtboardInstance,
    object: &RuntimeObject,
    local_id: usize,
) -> f32 {
    let thickness_key = runtime_draw_property_key_for_name("Stroke", "thickness");
    runtime_stroke_thickness_for_local(instance, local_id)
        .or_else(|| {
            thickness_key
                .and_then(|key| runtime_object_explicit_double_property_by_key(object, key))
        })
        .unwrap_or(1.0)
}

pub(crate) fn runtime_stroke_thickness_for_local(
    instance: &ArtboardInstance,
    local_id: usize,
) -> Option<f32> {
    runtime_draw_property_key_for_name("Stroke", "thickness")
        .and_then(|key| instance.double_property(local_id, key))
}

pub(crate) fn runtime_stroke_uint_property(
    instance: &ArtboardInstance,
    object: &RuntimeObject,
    local_id: usize,
    property_name: &str,
    fallback: u64,
) -> u64 {
    let property_key = runtime_draw_property_key_for_name("Stroke", property_name);
    property_key
        .and_then(|key| instance.uint_property(local_id, key))
        .or_else(|| {
            property_key.and_then(|key| runtime_object_explicit_uint_property_by_key(object, key))
        })
        .unwrap_or(fallback)
}
