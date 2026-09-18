//! Decoder demand uses scene geometry even before the first decoded frame.
use crate::source::{
    core::CoreHandle,
    semantic::{
        semantic_provider::{SemanticGeometryError, rendered_geometry_intersects_host_clip},
        semantic_snapshot::Bounds,
    },
};

/// `viewport` is in root-artboard coordinates after the host undoes its fit
/// transform. Call after the scene update; this never creates a media resource.
pub fn is_visible_in(video: &CoreHandle, viewport: Bounds) -> Result<bool, SemanticGeometryError> {
    is_visible_in_host_clip(video, viewport, None)
}

/// The optional host clip uses root-artboard coordinates and the exact drawing fill rule.
pub fn is_visible_in_host_clip(
    video: &CoreHandle,
    viewport: Bounds,
    host_clip: Option<(
        &crate::source::math::raw_path::RawPath,
        nuxie_render_api::FillRule,
    )>,
) -> Result<bool, SemanticGeometryError> {
    if !video.is_type_of(super::Video::TYPE_KEY) {
        return Ok(false);
    }
    rendered_geometry_intersects_host_clip(video, viewport, host_clip)
}
