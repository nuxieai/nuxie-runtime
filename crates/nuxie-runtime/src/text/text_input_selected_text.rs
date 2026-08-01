//! Selected-text drawable bridge from `src/text/text_input_selected_text.cpp`.

use crate::ArtboardInstance;

pub(crate) fn on_added_clean(instance: &ArtboardInstance, text_input_local: usize) -> bool {
    instance
        .component_handle(text_input_local)
        .is_some_and(|owner| {
            (0..instance.component_child_len(owner)).any(|index| {
                instance
                    .component_child_at(owner, index)
                    .and_then(|child| instance.component_local_id(child))
                    .is_some_and(|child| {
                        instance.runtime_object_type_name(child) == Some("TextInputSelectedText")
                    })
            })
        })
}
