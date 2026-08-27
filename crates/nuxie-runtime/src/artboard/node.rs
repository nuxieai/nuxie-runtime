use super::ArtboardInstance;
use crate::components::{ComponentHandle, Mat2D};
use crate::layout_component;
use crate::properties::RuntimeLayoutComputedProperty;
use nuxie_graph::ArtboardGraph;

impl ArtboardInstance {
    /// Direct `Node::{xChanged,yChanged}`.
    pub(in crate::artboard) fn runtime_node_position_changed(
        &mut self,
        node: ComponentHandle,
    ) -> bool {
        self.mark_transform_dirty_handle(node)
    }

    pub(crate) fn runtime_node_computed_local_transform(&self, local_id: usize) -> Option<Mat2D> {
        let handle = self.component_handle(local_id)?;
        let component = self.objects.component(handle)?;
        let node = component.concrete.node.as_ref()?;
        let parent_world = component
            .parent_transform
            .and_then(|parent| self.objects.component(parent))
            .map(|parent| parent.transform.world_transform);
        Some(node.computed_local_transform(parent_world, component.transform.world_transform))
    }

    /// Direct `Node::{computedRootX,computedRootY}`: map the settled world
    /// translation through the Artboard host/root transform.
    pub(crate) fn runtime_node_computed_root_position(
        &self,
        local_id: usize,
        graph: &ArtboardGraph,
        root_transform: Mat2D,
    ) -> Option<(f32, f32)> {
        let x = self.runtime_layout_computed_property(
            local_id,
            RuntimeLayoutComputedProperty::WorldX,
            graph,
        )?;
        let y = self.runtime_layout_computed_property(
            local_id,
            RuntimeLayoutComputedProperty::WorldY,
            graph,
        )?;
        Some(root_transform.transform_point(x, y))
    }

    /// Direct `Node::markLayoutNodeDirty`: every retained LayoutComponent in
    /// the parent chain owns a separate layout node and receives the callback.
    pub(crate) fn runtime_node_mark_layout_node_dirty(&mut self, node_local_id: usize) -> bool {
        let retained_layout_ancestor_count = self
            .component(node_local_id)
            .map_or(0, |component| component.layout_ancestors.len());
        // Cycle guard: a malformed-but-accepted file can make `parentId` form
        // a parent cycle (A -> B -> A), and C++ hangs on this walk. We
        // deliberately DIVERGE and terminate, mirroring C++'s own cycle-guard
        // idiom -- the visited-set from DependencySorter::visit
        // (src/dependency_sorter.cpp) -- so the walk ends as if the chain did.
        // Marks already made stay; the retained node's dirty transition is
        // idempotent. Unreachable on any valid file. See
        // runtime_layout_ancestors (components.rs) and
        // fuzz/regressions/README.md.
        let mut visited = std::collections::BTreeSet::new();
        let mut parent = self.component_parent_local(node_local_id);
        let mut changed = false;
        while let Some(local_id) = parent {
            if !visited.insert(local_id) {
                break;
            }
            parent = self.component_parent_local(local_id);
            if self
                .component(local_id)
                .is_some_and(|component| component.concrete.layout.is_some())
            {
                changed |= layout_component::mark_layout_node_dirty(self, local_id);
            }
        }
        // Imported graphs retain this same parent-walk result for ownership
        // that passes through non-Node Core objects. Duplicate owners are
        // harmless: the retained node's dirty transition is idempotent until
        // the next solve.
        for index in 0..retained_layout_ancestor_count {
            let Some(layout) = self
                .component(node_local_id)
                .and_then(|component| component.layout_ancestors.get(index).copied())
            else {
                continue;
            };
            if let Some(local_id) = self.component_local_id(layout) {
                changed |= layout_component::mark_layout_node_dirty(self, local_id);
            }
        }
        changed
    }
}
