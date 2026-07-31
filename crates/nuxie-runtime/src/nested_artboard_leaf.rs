// Direct owner for pinned C++ `src/nested_artboard_leaf.cpp`.

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
    let alignment = crate::layout::Alignment::new(alignment_x, alignment_y);
    let alignment_x = alignment.x;
    let alignment_y = alignment.y;
    let (frame_left, frame_top, frame_width, frame_height) = frame;
    let (content_left, content_top, content_width, content_height) = content;
    let x = -content_left - content_width * 0.5 - alignment_x * content_width * 0.5;
    let y = -content_top - content_height * 0.5 - alignment_y * content_height * 0.5;

    let (scale_x, scale_y) = match fit {
        0 => (frame_width / content_width, frame_height / content_height),
        1 => {
            let scale = (frame_width / content_width).min(frame_height / content_height);
            (scale, scale)
        }
        2 => {
            let scale = (frame_width / content_width).max(frame_height / content_height);
            (scale, scale)
        }
        3 => {
            let scale = frame_width / content_width;
            (scale, scale)
        }
        4 => {
            let scale = frame_height / content_height;
            (scale, scale)
        }
        6 => {
            let scale = (frame_width / content_width)
                .min(frame_height / content_height)
                .min(1.0);
            (scale, scale)
        }
        _ => (1.0, 1.0),
    };

    let translation = Mat2D([
        1.0,
        0.0,
        0.0,
        1.0,
        frame_left + frame_width * 0.5 + alignment_x * frame_width * 0.5,
        frame_top + frame_height * 0.5 + alignment_y * frame_height * 0.5,
    ]);
    let scale = Mat2D([scale_x, 0.0, 0.0, scale_y, 0.0, 0.0]);
    translation
        .multiply(scale)
        .multiply(Mat2D([1.0, 0.0, 0.0, 1.0, x, y]))
}
