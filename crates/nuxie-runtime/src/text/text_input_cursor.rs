//! Cursor drawable bridge ported from `src/text/text_input_cursor.cpp`.

use crate::{ArtboardInstance, RuntimePathCommand};

pub(super) fn local_clockwise_path(
    instance: &ArtboardInstance,
    text_input_local: usize,
    fallback_height: f32,
) -> Vec<RuntimePathCommand> {
    if !instance.text_input_is_focused(text_input_local) {
        return Vec::new();
    }
    let Some((top, bottom)) = instance.text_input_cursor_geometry(text_input_local) else {
        return caret_rect(0.0, 0.0, fallback_height);
    };
    caret_rect(top.0, top.1, bottom.1)
}

fn caret_rect(x: f32, top: f32, bottom: f32) -> Vec<RuntimePathCommand> {
    vec![
        RuntimePathCommand::Move { x, y: top },
        RuntimePathCommand::Line { x: x + 1.0, y: top },
        RuntimePathCommand::Line {
            x: x + 1.0,
            y: bottom,
        },
        RuntimePathCommand::Line { x, y: bottom },
        RuntimePathCommand::Close,
    ]
}
