#[cfg(any(test, feature = "tools"))]
use std::sync::Arc;

use super::{ArtboardInstance, RuntimeAdvancingComponent};
#[cfg(any(test, feature = "tools"))]
use crate::components::ComponentDirt;

impl ArtboardInstance {
    #[cfg(any(test, feature = "tools"))]
    pub fn debug_set_text_input_layout_size(
        &mut self,
        text_input_local: usize,
        width: f32,
        height: f32,
    ) -> bool {
        let Some(parent_local) = self
            .runtime_graph()
            .and_then(|graph| {
                graph.components.iter().find(|component| {
                    component.local_id == text_input_local && component.type_name == "TextInput"
                })
            })
            .and_then(|component| component.parent_local)
        else {
            return false;
        };
        let mut bounds = self
            .layout_constraint_bounds
            .as_deref()
            .cloned()
            .unwrap_or_default();
        let authored = self.runtime_authored_layout_component_bounds(parent_local);
        bounds.insert(
            parent_local,
            crate::draw::RuntimeLayoutBounds {
                x: authored.x,
                y: authored.y,
                width,
                height,
            },
        );
        self.layout_constraint_bounds = Some(Arc::new(bounds));
        self.solved_layout_bounds = self.layout_constraint_bounds.clone();
        if let Some(text_input) = self
            .component(text_input_local)
            .and_then(|component| component.concrete.text_input.as_ref())
        {
            text_input.raw.borrow_mut().mark_geometry_dirty();
        }
        self.add_dirt(text_input_local, ComponentDirt::TEXT_SHAPE, false);
        true
    }

    pub(in crate::artboard) fn advance_text_input_entry(
        &mut self,
        entry: RuntimeAdvancingComponent,
        elapsed_seconds: f32,
    ) -> bool {
        let Some(component) = entry.component else {
            return false;
        };
        let Some((is_dragging, scroll_constraint, scroll_x, scroll_y, last_position)) =
            self.objects.component(component).and_then(|component| {
                let state = component.concrete.text_input.as_ref()?;
                Some((
                    state.is_dragging,
                    state.scroll_constraint,
                    state.scroll_x,
                    state.scroll_y,
                    state.last_drag_world_position,
                ))
            })
        else {
            return false;
        };
        if !is_dragging {
            if let Some(state) = self
                .objects
                .component_mut(component)
                .and_then(|component| component.concrete.text_input.as_mut())
            {
                state.scroll_x = 0.0;
                state.scroll_y = 0.0;
            }
            return false;
        }
        let Some(scroll_constraint) = scroll_constraint else {
            return false;
        };
        if scroll_x == 0.0 && scroll_y == 0.0 {
            return false;
        }

        crate::constraints::advance_text_input_scroll(
            self,
            scroll_constraint,
            scroll_x,
            scroll_y,
            elapsed_seconds,
        );
        if last_position.0.is_finite() && last_position.1.is_finite() {
            self.text_input_drag(entry.local_id, last_position);
        }
        true
    }
}
