// Direct owner for pinned C++ `src/nested_artboard_leaf.cpp`.

pub(crate) fn nested_artboard_leaf_uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("NestedArtboardLeaf")
        && property_key_for_name("NestedArtboardLeaf", "fit") == Some(property_key))
    .then(|| instance.add_dirt(local_id, ComponentDirt::WORLD_TRANSFORM, true))
}

fn is_nested_artboard_occurrence_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "NestedArtboard" | "NestedArtboardLayout" | "NestedArtboardLeaf"
    )
}

/// Direct transform update owned by C++ `NestedArtboardLeaf::update`.
///
/// Bounds discovery remains a renderer projection; the leaf owns fit and
/// alignment semantics and returns the exact view transform multiplied into
/// its world transform.
pub(crate) fn runtime_nested_artboard_leaf_alignment(
    fit: u64,
    alignment_x: f32,
    alignment_y: f32,
    frame: (f32, f32, f32, f32),
    content: (f32, f32, f32, f32),
) -> Mat2D {
    let (frame_left, frame_top, frame_width, frame_height) = frame;
    let (content_left, content_top, content_width, content_height) = content;
    let fit = nuxie_render_api::Fit::from_u64(fit).unwrap_or(nuxie_render_api::Fit::None);
    let alignment = crate::layout::Alignment::new(alignment_x, alignment_y);
    let matrix = nuxie_render_api::compute_alignment_from_origin_size(
        fit,
        nuxie_render_api::Vec2D::new(alignment.x, alignment.y),
        nuxie_render_api::Vec2D::new(frame_left, frame_top),
        nuxie_render_api::Vec2D::new(frame_width, frame_height),
        nuxie_render_api::Vec2D::new(content_left, content_top),
        nuxie_render_api::Vec2D::new(content_width, content_height),
        1.0,
    );
    Mat2D(matrix.0)
}
