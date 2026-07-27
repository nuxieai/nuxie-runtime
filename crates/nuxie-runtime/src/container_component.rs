use std::collections::BTreeSet;

use super::ArtboardInstance;
use crate::components::ComponentHandle;

impl ArtboardInstance {
    /// The retained-child loop owned by C++ `ContainerComponent::collapse`
    /// (`src/container_component.cpp:9-19`).
    ///
    /// The thin virtual-dispatch adapter remains on `ArtboardInstance`; it
    /// decides whether this concrete owner, `Solo`, or `LayoutComponent` runs.
    pub(super) fn collapse_container_component_children_with_ancestor_guarded(
        &mut self,
        handle: ComponentHandle,
        collapsed: bool,
        ancestor_changed: bool,
        visited: &mut BTreeSet<ComponentHandle>,
    ) -> bool {
        let children = (0..self.component_child_len(handle))
            .filter_map(|index| self.component_child_at(handle, index))
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
}
