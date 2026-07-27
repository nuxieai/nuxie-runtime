use std::collections::BTreeSet;

use super::ArtboardInstance;
use crate::components::{ComponentDirt, ComponentHandle};

impl ArtboardInstance {
    pub(crate) fn collapse_component_tree(&mut self, local_id: usize, collapsed: bool) -> bool {
        self.collapse_component_tree_with_ancestor(local_id, collapsed, false)
    }

    pub(crate) fn collapse_component_tree_with_ancestor(
        &mut self,
        local_id: usize,
        collapsed: bool,
        ancestor_changed: bool,
    ) -> bool {
        // Cycle guard entry point: see
        // LayoutComponent::propagateCollapse. A valid C++ occurrence tree has
        // one parent per Component, so this is inert for valid files and
        // fail-closed for malformed accepted graphs.
        let mut visited = BTreeSet::new();
        let Some(handle) = self.component_handle(local_id) else {
            return false;
        };
        self.collapse_component_tree_with_ancestor_guarded(
            handle,
            collapsed,
            ancestor_changed,
            &mut visited,
        )
    }

    pub(super) fn collapse_component_tree_with_ancestor_guarded(
        &mut self,
        handle: ComponentHandle,
        collapsed: bool,
        ancestor_changed: bool,
        visited: &mut BTreeSet<ComponentHandle>,
    ) -> bool {
        // C++ `ContainerComponent::collapse` walks each concrete child in
        // retained order after applying the Component base (`src/
        // container_component.cpp:39-56`). Skip a repeated occurrence only
        // when a malformed graph would otherwise recurse forever.
        if !visited.insert(handle) {
            return false;
        }
        let changed_here = self.collapse_component_handle(handle, collapsed);
        let mut changed = changed_here;
        if ancestor_changed && !collapsed {
            changed |= self.add_component_dirt(handle, ComponentDirt::FILTHY, false);
        }
        let type_name = self
            .objects
            .component(handle)
            .map(|component| component.type_name);
        match type_name {
            // C++ Solo::collapse intentionally skips the blind
            // ContainerComponent child walk. Solo::propagateCollapse
            // re-collapses inactive children even while the Solo becomes
            // visible (`src/solo.cpp:44-81`).
            Some("Solo") => changed,
            // C++ LayoutComponent::collapse folds local display:none into the
            // value propagated to its children.
            Some("Artboard" | "LayoutComponent") => {
                changed
                    | self.propagate_layout_component_display_collapse_with_ancestor_guarded(
                        handle,
                        ancestor_changed || changed_here,
                        visited,
                    )
            }
            _ => {
                let children = (0..self.component_child_len(handle))
                    .filter_map(|index| self.component_child_at(handle, index))
                    .collect::<Vec<_>>();
                for child in children {
                    changed |= self.collapse_component_tree_with_ancestor_guarded(
                        child,
                        collapsed,
                        ancestor_changed || changed_here,
                        visited,
                    );
                }
                changed |= self.collapse_constrained_transform_dependents(handle);
                changed
            }
        }
    }
}
