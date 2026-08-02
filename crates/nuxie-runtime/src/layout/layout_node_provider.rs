use crate::{artboard::ArtboardInstance, layout_component};

/// Direct `Node::markLayoutNodeDirty`: every retained LayoutComponent in the
/// parent chain owns a separate layout node and receives the callback.
pub(crate) fn mark_layout_node_dirty(
    instance: &mut ArtboardInstance,
    node_local_id: usize,
) -> bool {
    let retained_layout_ancestor_count = instance
        .component(node_local_id)
        .map_or(0, |component| component.layout_ancestors.len());
    // Cycle guard: a malformed-but-accepted file can make `parentId` form a
    // parent cycle (A -> B -> A), and C++ hangs on this walk. We deliberately
    // DIVERGE and terminate, mirroring C++'s own cycle-guard idiom -- the
    // visited-set from DependencySorter::visit (src/dependency_sorter.cpp) --
    // so the walk ends as if the chain did. Marks already made stay; the
    // retained node's dirty transition is idempotent. Unreachable on any
    // valid file. See runtime_layout_ancestors (components.rs) and
    // fuzz/regressions/README.md.
    let mut visited = std::collections::BTreeSet::new();
    let mut parent = instance.component_parent_local(node_local_id);
    let mut changed = false;
    while let Some(local_id) = parent {
        if !visited.insert(local_id) {
            break;
        }
        parent = instance.component_parent_local(local_id);
        if instance
            .component(local_id)
            .is_some_and(|component| component.concrete.layout.is_some())
        {
            changed |= layout_component::mark_layout_node_dirty(instance, local_id);
        }
    }
    // Imported graphs retain this same parent-walk result for ownership that
    // passes through non-Node Core objects. Duplicate owners are harmless:
    // the retained node's dirty transition is idempotent until the next solve.
    for index in 0..retained_layout_ancestor_count {
        let Some(layout) = instance
            .component(node_local_id)
            .and_then(|component| component.layout_ancestors.get(index).copied())
        else {
            continue;
        };
        if let Some(local_id) = instance.component_local_id(layout) {
            changed |= layout_component::mark_layout_node_dirty(instance, local_id);
        }
    }
    changed
}
