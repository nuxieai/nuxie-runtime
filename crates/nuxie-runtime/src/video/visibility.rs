//! Decoder demand uses scene geometry even before the first decoded frame.
use crate::source::{
    core::CoreHandle,
    semantic::{
        semantic_provider::{
            SemanticGeometryError, node_has_area, rendered_geometry_intersects_host_clip,
            source_ancestors_are_visible,
        },
        semantic_snapshot::Bounds,
    },
};

/// `viewport` is in root-artboard coordinates after the host undoes its fit
/// transform. Call after the scene update; this never creates a media resource.
pub fn is_visible_in(video: &CoreHandle, viewport: Bounds) -> Result<bool, SemanticGeometryError> {
    is_visible_in_host_clip(video, viewport, None)
}

/// The optional host clip uses root-artboard coordinates and the exact drawing fill rule.
/// A video is on screen where it draws itself, and wherever a script that
/// reads it draws.
pub fn is_visible_in_host_clip(
    video: &CoreHandle,
    viewport: Bounds,
    host_clip: Option<(
        &crate::source::math::raw_path::RawPath,
        nuxie_render_api::FillRule,
    )>,
) -> Result<bool, SemanticGeometryError> {
    let Some((drawn_by_script, readers)) = video
        .with_downcast::<super::Video, _>(|video| (video.drawn_by_script, video.script_readers()))
    else {
        return Ok(false);
    };
    if !drawn_by_script && rendered_geometry_intersects_host_clip(video, viewport, host_clip)? {
        return Ok(true);
    }
    for reader in readers {
        if reader_is_visible_in(&reader, viewport, host_clip)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// A drawing script has no box of its own, so the nearest box around it says
/// where it draws.
fn reader_is_visible_in(
    reader: &CoreHandle,
    viewport: Bounds,
    host_clip: Option<(
        &crate::source::math::raw_path::RawPath,
        nuxie_render_api::FillRule,
    )>,
) -> Result<bool, SemanticGeometryError> {
    if !source_ancestors_are_visible(reader) {
        return Ok(false);
    }
    let mut current = Some(reader.clone());
    while let Some(node) = current {
        if node_has_area(&node) {
            return rendered_geometry_intersects_host_clip(&node, viewport, host_clip);
        }
        current = node
            .with(|object| object.component_parent_handle())
            .flatten();
    }
    Ok(false)
}
