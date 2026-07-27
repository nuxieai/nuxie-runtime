use std::collections::BTreeSet;

use super::ArtboardInstance;
use crate::components::{ComponentHandle, RuntimeComponent};
use crate::properties::layout_component_style_display_value_property_key;

impl ArtboardInstance {
    /// Direct C++ `LayoutComponent::hitTestPoint` owner. Its local-bounds
    /// check delegates to `Drawable::hitTestPoint` with
    /// `skipOnUnclipped=true` (`src/layout_component.cpp:49-80`).
    pub(super) fn layout_component_hit_test_point(
        &self,
        component: ComponentHandle,
        position: (f32, f32),
        skip_on_unclipped: bool,
        is_primary_hit: bool,
    ) -> bool {
        let Some(owner) = self.objects.component(component) else {
            return false;
        };
        let Some(layout) = owner.concrete.layout.as_ref() else {
            return false;
        };
        let world = owner.transform.world_transform;
        if world.determinant() == 0.0 {
            return false;
        }

        let clip = layout
            .clip_property_key
            .and_then(|key| self.objects.component_bool_property(component, key))
            .unwrap_or(owner.type_name == "Artboard" && self.clip);
        if !(skip_on_unclipped && !clip) {
            let mut local = world
                .invert_or_identity()
                .transform_point(position.0, position.1);
            let (_, _, width, height) = layout.constraint_bounds();
            if owner.type_name == "Artboard" && (self.origin_x != 0.0 || self.origin_y != 0.0) {
                local.0 += self.origin_x * width;
                local.1 += self.origin_y * height;
            }
            if local.0 < 0.0 || local.0 > width || local.1 < 0.0 || local.1 > height {
                return false;
            }
        }

        self.drawable_component_hit_test_point(component, position, true, is_primary_hit)
    }

    pub(super) fn propagate_layout_component_display_collapse(
        &mut self,
        layout_local: usize,
    ) -> bool {
        self.propagate_layout_component_display_collapse_with_ancestor(layout_local, false)
    }

    /// Direct C++ `LayoutComponent::propagateCollapse` owner. The propagated
    /// value folds in this occurrence's display:none state, then delegates
    /// each retained child to `ContainerComponent::collapse`
    /// (`src/layout_component.cpp:303-314`).
    fn propagate_layout_component_display_collapse_with_ancestor(
        &mut self,
        layout_local: usize,
        ancestor_changed: bool,
    ) -> bool {
        // These mutually recursive Rust adapters also guard malformed accepted
        // graphs. A valid C++ occurrence tree visits every child at most once.
        let mut visited = BTreeSet::new();
        let Some(layout) = self.component_handle(layout_local) else {
            return false;
        };
        self.propagate_layout_component_display_collapse_with_ancestor_guarded(
            layout,
            ancestor_changed,
            &mut visited,
        )
    }

    pub(super) fn propagate_layout_component_display_collapse_with_ancestor_guarded(
        &mut self,
        layout: ComponentHandle,
        ancestor_changed: bool,
        visited: &mut BTreeSet<ComponentHandle>,
    ) -> bool {
        let Some(layout_local) = self.component_local_id(layout) else {
            return false;
        };
        let display_hidden =
            self.layout_component_style_local(layout_local)
                .and_then(|style_local| {
                    layout_component_style_display_value_property_key()
                        .and_then(|key| self.uint_property(style_local, key))
                })
                == Some(1);
        let collapsed = display_hidden
            || self
                .component(layout_local)
                .is_some_and(RuntimeComponent::is_collapsed);
        let children = (0..self.component_child_len(layout))
            .filter_map(|index| self.component_child_at(layout, index))
            .collect::<Vec<_>>();

        let mut changed = false;
        for child in children {
            changed |= self.collapse_component_tree_with_ancestor_guarded(
                child,
                collapsed,
                ancestor_changed,
                visited,
            );
        }
        changed
    }

    pub(super) fn layout_component_style_local(&self, layout_local: usize) -> Option<usize> {
        self.component_handle(layout_local)
            .and_then(|layout| self.objects.component(layout))
            .and_then(|component| component.concrete.layout.as_ref())
            .and_then(|layout| layout.style)
            .and_then(|style| self.objects.component_local_id(style))
    }
}
