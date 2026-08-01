//! Selection drawable bridge from `src/text/text_input_selection.cpp`.

use crate::{ArtboardInstance, RuntimePathCommand};

pub(super) fn local_clockwise_path(
    instance: &ArtboardInstance,
    text_input_local: usize,
) -> Vec<RuntimePathCommand> {
    instance.text_input_selection_path(text_input_local)
}
