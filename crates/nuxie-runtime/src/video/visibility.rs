//! Decoder demand uses scene geometry even before the first decoded frame.
use crate::source::{
    core::CoreHandle,
    semantic::{
        semantic_provider::{SemanticGeometryError, rendered_geometry_intersects_viewport},
        semantic_snapshot::Bounds,
    },
};

/// `viewport` is in root-artboard coordinates after the host undoes its fit
/// transform. Call after the scene update; this never creates a media resource.
pub fn is_visible_in(video: &CoreHandle, viewport: Bounds) -> Result<bool, SemanticGeometryError> {
    if !video.is_type_of(super::Video::TYPE_KEY) {
        return Ok(false);
    }
    rendered_geometry_intersects_viewport(video, viewport)
}
