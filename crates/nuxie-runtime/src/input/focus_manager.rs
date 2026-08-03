//! Direct retained-tree manager port of pinned src/input/focus_manager.cpp (B6-0238).

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use super::focus_node::{
    FocusBounds, FocusDirection, FocusEdgeBehavior, FocusEvent, FocusEventKind, FocusNode,
    FocusNodeId, FocusPoint,
};
use super::focusable::RuntimeFocusable;

/// Owns focus topology and focus state for one mounted focus domain.
#[derive(Debug, Clone, Default)]
pub struct FocusManager {
    pub(crate) nodes: BTreeMap<FocusNodeId, FocusNode>,
    roots: Vec<FocusNodeId>,
    primary_focus: Option<FocusNodeId>,
    pub(crate) pending_events: Vec<FocusEvent>,
    // High-level runtimes poll this every frame. Recompute the retained-tree
    // predicate only after an input to it changes, matching pinned C++.
    focusable_content_cache: Cell<Option<bool>>,
}

impl FocusManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_node(&mut self, node: FocusNode) -> FocusNodeId {
        let node_id = FocusNodeId::next();
        self.nodes.insert(node_id, node);
        node_id
    }

    pub fn add_child(&mut self, parent: Option<FocusNodeId>, child: FocusNodeId) -> bool {
        self.insert_child(parent, child, usize::MAX)
    }

    pub fn insert_child(
        &mut self,
        parent: Option<FocusNodeId>,
        child: FocusNodeId,
        index: usize,
    ) -> bool {
        if !self.nodes.contains_key(&child)
            || parent.is_some_and(|parent| !self.nodes.contains_key(&parent))
            || parent == Some(child)
            || parent.is_some_and(|parent| self.ancestor_chain(parent).contains(&child))
        {
            return false;
        }

        let siblings = parent
            .and_then(|parent| self.nodes.get(&parent).map(|node| &node.children))
            .unwrap_or(&self.roots);
        if self.nodes.get(&child).and_then(|node| node.parent) == parent
            && siblings
                .iter()
                .position(|node| *node == child)
                .is_some_and(|position| position == index.min(siblings.len().saturating_sub(1)))
        {
            return true;
        }

        self.unlink(child);
        self.nodes
            .get_mut(&child)
            .expect("validated child")
            .set_parent(parent);
        if let Some(parent) = parent {
            self.nodes
                .get_mut(&parent)
                .expect("validated parent")
                .insert_child(index, child);
        } else {
            self.roots.insert(index.min(self.roots.len()), child);
        }
        self.mark_focusable_content_dirty();
        true
    }

    pub fn set_focus(&mut self, node_id: FocusNodeId) -> bool {
        if !self.node_eligible_for_focus(node_id) {
            return false;
        }
        let node_id = self.first_eligible_leaf(node_id).unwrap_or(node_id);
        if self.primary_focus == Some(node_id) {
            return false;
        }
        let old_focus = self.primary_focus.replace(node_id);
        self.notify_focus_change(old_focus, Some(node_id));
        true
    }

    pub fn clear_focus(&mut self) -> bool {
        let Some(old_focus) = self.primary_focus.take() else {
            return false;
        };
        self.notify_focus_change(Some(old_focus), None);
        true
    }

    pub fn focus_next(&mut self) -> bool {
        self.move_focus_sequentially(true)
    }

    pub fn focus_previous(&mut self) -> bool {
        self.move_focus_sequentially(false)
    }

    pub fn focus_left(&mut self) -> bool {
        self.focus_direction(FocusDirection::Left)
    }

    pub fn focus_right(&mut self) -> bool {
        self.focus_direction(FocusDirection::Right)
    }

    pub fn focus_up(&mut self) -> bool {
        self.focus_direction(FocusDirection::Up)
    }

    pub fn focus_down(&mut self) -> bool {
        self.focus_direction(FocusDirection::Down)
    }

    pub fn focus_direction(&mut self, direction: FocusDirection) -> bool {
        self.drop_focus_if_ineligible();
        let Some(current) = self.primary_focus else {
            return false;
        };
        let Some(next) = self.node_in_direction(current, direction) else {
            return false;
        };
        self.set_focus(next)
    }

    pub fn detach_subtree(&mut self, node_id: FocusNodeId) -> bool {
        if !self.nodes.contains_key(&node_id) {
            return false;
        }
        self.unlink(node_id);
        self.nodes
            .get_mut(&node_id)
            .expect("validated node")
            .set_parent(None);
        true
    }

    pub fn remove_subtree(&mut self, node_id: FocusNodeId) -> bool {
        let Some(subtree) = self.subtree_ids(node_id) else {
            return false;
        };
        if self
            .primary_focus
            .is_some_and(|primary| subtree.contains(&primary))
        {
            self.clear_focus();
        }
        self.detach_subtree(node_id);
        for descendant in subtree {
            self.nodes.remove(&descendant);
        }
        true
    }

    pub fn migrate_subtree_from(
        &mut self,
        source: &mut Self,
        node_id: FocusNodeId,
        parent: Option<FocusNodeId>,
        index: usize,
    ) -> bool {
        if parent.is_some_and(|parent| !self.nodes.contains_key(&parent)) {
            return false;
        }
        let Some(subtree) = source.subtree_ids(node_id) else {
            return false;
        };
        if subtree
            .iter()
            .any(|descendant| self.nodes.contains_key(descendant))
        {
            return false;
        }
        let focused = source
            .primary_focus
            .filter(|primary| subtree.contains(primary));
        if focused.is_some() {
            source.clear_focus();
        }
        source.detach_subtree(node_id);
        for descendant in &subtree {
            let node = source
                .nodes
                .remove(descendant)
                .expect("validated source subtree");
            self.nodes.insert(*descendant, node);
        }
        self.nodes
            .get_mut(&node_id)
            .expect("migrated subtree root")
            .parent = None;
        let inserted = self.insert_child(parent, node_id, index);
        debug_assert!(inserted);
        if let Some(focused) = focused {
            self.set_focus(focused);
        }
        true
    }

    pub fn primary_focus(&self) -> Option<FocusNodeId> {
        self.primary_focus
    }

    pub fn has_primary_focus(&self, node_id: FocusNodeId) -> bool {
        self.primary_focus == Some(node_id)
    }

    pub fn contains(&self, node_id: FocusNodeId) -> bool {
        self.nodes.contains_key(&node_id)
    }

    pub fn node(&self, node_id: FocusNodeId) -> Option<&FocusNode> {
        self.nodes.get(&node_id)
    }

    pub fn node_mut(&mut self, node_id: FocusNodeId) -> Option<&mut FocusNode> {
        // Unrestricted mutation can change can_focus or focusable backing.
        // Retained hot paths use update_node/set_node_focusable below, which
        // invalidate only when the cached predicate can actually change.
        self.mark_focusable_content_dirty();
        self.nodes.get_mut(&node_id)
    }

    pub fn set_node_can_focus(&mut self, node_id: FocusNodeId, value: bool) -> bool {
        let Some(node) = self.nodes.get_mut(&node_id) else {
            return false;
        };
        if node.can_focus() == value {
            return false;
        }
        node.set_can_focus(value);
        self.mark_focusable_content_dirty();
        true
    }

    pub(crate) fn set_node_focusable(
        &mut self,
        node_id: FocusNodeId,
        value: Option<RuntimeFocusable>,
    ) -> bool {
        let Some(node) = self.nodes.get_mut(&node_id) else {
            return false;
        };
        let backing_changed = node.focusable.is_some() != value.is_some();
        let changed = node.focusable != value;
        node.focusable = value;
        if backing_changed {
            self.mark_focusable_content_dirty();
        }
        changed
    }

    pub(crate) fn update_node(&mut self, node_id: FocusNodeId, replacement: &FocusNode) -> bool {
        let Some(node) = self.nodes.get_mut(&node_id) else {
            return false;
        };
        let predicate_changed = node.can_focus() != replacement.can_focus();
        node.set_can_focus(replacement.can_focus());
        node.set_can_touch(replacement.can_touch());
        node.set_can_traverse(replacement.can_traverse());
        node.set_eligible(replacement.is_eligible());
        node.set_tab_index(replacement.tab_index());
        node.set_name(replacement.name());
        node.set_edge_behavior(replacement.edge_behavior());
        node.set_bounds(replacement.bounds());
        node.set_position(replacement.position());
        if predicate_changed {
            self.mark_focusable_content_dirty();
        }
        true
    }

    pub fn has_focus(&self, node_id: FocusNodeId) -> bool {
        self.nodes.get(&node_id).is_some_and(|node| node.has_focus)
    }

    pub fn children(&self, node_id: FocusNodeId) -> Option<&[FocusNodeId]> {
        self.nodes
            .get(&node_id)
            .map(|node| node.children.as_slice())
    }

    pub fn parent(&self, node_id: FocusNodeId) -> Option<FocusNodeId> {
        self.nodes.get(&node_id).and_then(|node| node.parent)
    }

    pub fn roots(&self) -> &[FocusNodeId] {
        &self.roots
    }

    pub fn is_attached(&self, node_id: FocusNodeId) -> bool {
        let mut current = node_id;
        loop {
            let Some(node) = self.nodes.get(&current) else {
                return false;
            };
            let Some(parent) = node.parent else {
                return self.roots.contains(&current);
            };
            current = parent;
        }
    }

    pub fn has_focusable_content(&self) -> bool {
        if let Some(cached) = self.focusable_content_cache.get() {
            return cached;
        }
        let has_focusable_content = self
            .roots
            .iter()
            .copied()
            .any(|root| self.subtree_has_focusable_content(root));
        self.focusable_content_cache
            .set(Some(has_focusable_content));
        has_focusable_content
    }

    pub fn mark_focusable_content_dirty(&self) {
        self.focusable_content_cache.set(None);
    }

    pub fn take_events(&mut self) -> Vec<FocusEvent> {
        std::mem::take(&mut self.pending_events)
    }

    pub(crate) fn focusable(&self, node_id: FocusNodeId) -> Option<RuntimeFocusable> {
        self.nodes.get(&node_id).and_then(FocusNode::focusable)
    }

    fn unlink(&mut self, node_id: FocusNodeId) {
        let parent = self.nodes.get(&node_id).and_then(|node| node.parent);
        let mut removed = false;
        if let Some(parent) = parent {
            if let Some(parent) = self.nodes.get_mut(&parent) {
                let previous_len = parent.children.len();
                parent.remove_child(node_id);
                removed = parent.children.len() != previous_len;
            }
        } else {
            let previous_len = self.roots.len();
            self.roots.retain(|root| *root != node_id);
            removed = self.roots.len() != previous_len;
        }
        if removed {
            self.mark_focusable_content_dirty();
        }
    }

    fn notify_focus_change(
        &mut self,
        old_focus: Option<FocusNodeId>,
        new_focus: Option<FocusNodeId>,
    ) {
        let old_ancestors: BTreeSet<_> = old_focus
            .map(|node_id| self.ancestor_chain(node_id).into_iter().collect())
            .unwrap_or_default();
        let common_ancestor = new_focus.and_then(|node_id| {
            self.ancestor_chain(node_id)
                .into_iter()
                .find(|ancestor| old_ancestors.contains(ancestor))
        });

        let mut current = old_focus;
        while current != common_ancestor {
            let Some(node_id) = current else {
                break;
            };
            let node = self.nodes.get_mut(&node_id).expect("focus node exists");
            if !node.has_focus {
                break;
            }
            node.has_focus = false;
            current = node.parent;
            self.pending_events
                .push(FocusEvent::new(node_id, FocusEventKind::Blurred));
        }

        let mut current = new_focus;
        while current != common_ancestor {
            let Some(node_id) = current else {
                break;
            };
            let node = self.nodes.get_mut(&node_id).expect("focus node exists");
            if node.has_focus {
                break;
            }
            node.has_focus = true;
            current = node.parent;
            self.pending_events
                .push(FocusEvent::new(node_id, FocusEventKind::Focused));
        }
    }

    pub(crate) fn ancestor_chain(&self, node_id: FocusNodeId) -> Vec<FocusNodeId> {
        let mut ancestors = Vec::new();
        let mut current = Some(node_id);
        while let Some(node_id) = current {
            ancestors.push(node_id);
            current = self.nodes.get(&node_id).and_then(|node| node.parent);
        }
        ancestors
    }

    fn subtree_ids(&self, node_id: FocusNodeId) -> Option<Vec<FocusNodeId>> {
        let mut descendants = Vec::new();
        let mut pending = vec![node_id];
        while let Some(descendant) = pending.pop() {
            let node = self.nodes.get(&descendant)?;
            descendants.push(descendant);
            pending.extend(node.children.iter().rev().copied());
        }
        Some(descendants)
    }

    fn subtree_has_focusable_content(&self, node_id: FocusNodeId) -> bool {
        let Some(node) = self.nodes.get(&node_id) else {
            return false;
        };
        node.focusable.is_some()
            || node.can_focus()
            || node
                .children
                .iter()
                .copied()
                .any(|child| self.subtree_has_focusable_content(child))
    }

    fn node_eligible_for_focus(&self, node_id: FocusNodeId) -> bool {
        self.nodes
            .get(&node_id)
            .is_some_and(|node| node.can_focus() && node.is_eligible())
    }

    fn node_eligible_for_traversal(&self, node_id: FocusNodeId) -> bool {
        self.nodes
            .get(&node_id)
            .is_some_and(|node| node.can_traverse() && node.can_focus() && node.is_eligible())
    }

    fn node_traversable(&self, node_id: FocusNodeId) -> bool {
        let Some(node) = self.nodes.get(&node_id) else {
            return false;
        };
        self.node_eligible_for_traversal(node_id)
            || (node.focusable.is_none()
                && node
                    .children
                    .iter()
                    .copied()
                    .any(|child| self.node_traversable(child)))
    }

    fn traversable_children(&self, parent: Option<FocusNodeId>) -> Vec<FocusNodeId> {
        let children = parent
            .and_then(|parent| self.nodes.get(&parent).map(|node| node.children.as_slice()))
            .unwrap_or(self.roots.as_slice());
        let mut traversable = children
            .iter()
            .copied()
            .filter(|child| self.node_traversable(*child))
            .collect::<Vec<_>>();
        traversable
            .sort_by_key(|node_id| self.nodes.get(node_id).map_or(0, |node| node.tab_index()));
        traversable
    }

    fn first_eligible_leaf(&self, node_id: FocusNodeId) -> Option<FocusNodeId> {
        let children = self.traversable_children(Some(node_id));
        for child in &children {
            if let Some(leaf) = self.first_eligible_leaf(*child) {
                return Some(leaf);
            }
        }
        (children.is_empty() && self.node_eligible_for_traversal(node_id)).then_some(node_id)
    }

    fn last_eligible_leaf(&self, node_id: FocusNodeId) -> Option<FocusNodeId> {
        let children = self.traversable_children(Some(node_id));
        for child in children.iter().rev() {
            if let Some(leaf) = self.last_eligible_leaf(*child) {
                return Some(leaf);
            }
        }
        (children.is_empty() && self.node_eligible_for_traversal(node_id)).then_some(node_id)
    }

    fn move_focus_sequentially(&mut self, forward: bool) -> bool {
        self.drop_focus_if_ineligible();
        let current = self.primary_focus;
        let Some(next) = self.next_focusable_from(current, forward) else {
            if current.is_some_and(|current| self.clears_at_sequential_edge(current)) {
                self.clear_focus();
            }
            return false;
        };
        if Some(next) == current {
            return false;
        }
        self.set_focus(next)
    }

    fn clears_at_sequential_edge(&self, current: FocusNodeId) -> bool {
        let mut scope = self.parent(current);
        while let Some(scope_id) = scope {
            let Some(node) = self.nodes.get(&scope_id) else {
                return false;
            };
            match node.edge_behavior() {
                FocusEdgeBehavior::Stop => return false,
                FocusEdgeBehavior::ClosedLoop => return false,
                FocusEdgeBehavior::ParentScope => scope = node.parent,
            }
        }
        true
    }

    fn next_focusable_from(
        &self,
        current: Option<FocusNodeId>,
        forward: bool,
    ) -> Option<FocusNodeId> {
        let scope = current.and_then(|current| self.parent(current));
        let traversable = self.traversable_children(scope);
        if traversable.is_empty() {
            return scope.and_then(|scope| self.next_focusable_from(Some(scope), forward));
        }

        let Some(current) = current else {
            return self.first_leaf_from(&traversable, forward);
        };
        let Some(index) = traversable.iter().position(|node_id| *node_id == current) else {
            return self.first_leaf_from(&traversable, forward);
        };

        let next = if forward {
            traversable
                .iter()
                .skip(index.saturating_add(1))
                .find_map(|node_id| self.first_eligible_leaf(*node_id))
        } else {
            traversable
                .iter()
                .take(index)
                .rev()
                .find_map(|node_id| self.last_eligible_leaf(*node_id))
        };
        if next.is_some() {
            return next;
        }

        match scope
            .and_then(|scope| self.nodes.get(&scope))
            .map_or(FocusEdgeBehavior::ParentScope, |node| node.edge_behavior())
        {
            FocusEdgeBehavior::ClosedLoop => self.first_leaf_from(&traversable, forward),
            FocusEdgeBehavior::Stop => None,
            FocusEdgeBehavior::ParentScope => {
                scope.and_then(|scope| self.next_focusable_from(Some(scope), forward))
            }
        }
    }

    fn first_leaf_from(&self, traversable: &[FocusNodeId], forward: bool) -> Option<FocusNodeId> {
        if forward {
            traversable
                .iter()
                .find_map(|node_id| self.first_eligible_leaf(*node_id))
        } else {
            traversable
                .iter()
                .rev()
                .find_map(|node_id| self.last_eligible_leaf(*node_id))
        }
    }

    pub fn drop_focus_if_ineligible(&mut self) -> bool {
        if self
            .primary_focus
            .is_some_and(|primary| !self.node_eligible_for_traversal(primary))
        {
            self.clear_focus()
        } else {
            false
        }
    }

    fn node_in_direction(
        &self,
        current: FocusNodeId,
        direction: FocusDirection,
    ) -> Option<FocusNodeId> {
        let current_bounds = self.nodes.get(&current).and_then(FocusNode::bounds);
        let current_position = current_bounds
            .map(FocusBounds::center)
            .or_else(|| self.nodes.get(&current).and_then(FocusNode::position))?;
        let mut candidates = Vec::new();
        self.collect_traversable_leaves(&self.roots, &mut candidates);

        let mut best = None;
        let mut best_score = f32::INFINITY;
        for candidate in candidates {
            if candidate == current {
                continue;
            }
            let candidate_node = self.nodes.get(&candidate).expect("collected node");
            let score = match (current_bounds, candidate_node.bounds()) {
                (Some(current), Some(candidate)) => {
                    score_directional_bounds(current, candidate, direction)
                }
                _ => {
                    let Some(candidate_position) = candidate_node
                        .bounds()
                        .map(FocusBounds::center)
                        .or(candidate_node.position())
                    else {
                        continue;
                    };
                    score_directional_points(current_position, candidate_position, direction)
                }
            };
            if score < best_score {
                best_score = score;
                best = Some(candidate);
            }
        }
        best
    }

    fn collect_traversable_leaves(&self, nodes: &[FocusNodeId], result: &mut Vec<FocusNodeId>) {
        for node_id in nodes {
            let Some(node) = self.nodes.get(node_id) else {
                continue;
            };
            let is_leaf = !node
                .children
                .iter()
                .copied()
                .any(|child| self.node_traversable(child));
            if is_leaf && self.node_eligible_for_traversal(*node_id) {
                result.push(*node_id);
            }
            self.collect_traversable_leaves(&node.children, result);
        }
    }
}

fn score_directional_bounds(
    current: FocusBounds,
    candidate: FocusBounds,
    direction: FocusDirection,
) -> f32 {
    let (displacement, orthogonal_distance, overlap, orthogonal_weight) = match direction {
        FocusDirection::Left => (
            current.min_x - candidate.max_x,
            (candidate.min_y - current.max_y)
                .max(current.min_y - candidate.max_y)
                .max(0.0),
            axis_overlap(
                current.min_y,
                current.max_y,
                candidate.min_y,
                candidate.max_y,
            ),
            30.0,
        ),
        FocusDirection::Right => (
            candidate.min_x - current.max_x,
            (candidate.min_y - current.max_y)
                .max(current.min_y - candidate.max_y)
                .max(0.0),
            axis_overlap(
                current.min_y,
                current.max_y,
                candidate.min_y,
                candidate.max_y,
            ),
            30.0,
        ),
        FocusDirection::Up => (
            current.min_y - candidate.max_y,
            (candidate.min_x - current.max_x)
                .max(current.min_x - candidate.max_x)
                .max(0.0),
            axis_overlap(
                current.min_x,
                current.max_x,
                candidate.min_x,
                candidate.max_x,
            ),
            2.0,
        ),
        FocusDirection::Down => (
            candidate.min_y - current.max_y,
            (candidate.min_x - current.max_x)
                .max(current.min_x - candidate.max_x)
                .max(0.0),
            axis_overlap(
                current.min_x,
                current.max_x,
                candidate.min_x,
                candidate.max_x,
            ),
            2.0,
        ),
    };
    if displacement < 0.0 {
        return f32::INFINITY;
    }
    displacement + orthogonal_weight * orthogonal_distance - overlap.sqrt()
}

fn score_directional_points(
    current: FocusPoint,
    candidate: FocusPoint,
    direction: FocusDirection,
) -> f32 {
    let delta_x = candidate.x - current.x;
    let delta_y = candidate.y - current.y;
    let (primary, orthogonal, orthogonal_weight) = match direction {
        FocusDirection::Left => (-delta_x, delta_y.abs(), 30.0),
        FocusDirection::Right => (delta_x, delta_y.abs(), 30.0),
        FocusDirection::Up => (-delta_y, delta_x.abs(), 2.0),
        FocusDirection::Down => (delta_y, delta_x.abs(), 2.0),
    };
    if primary <= 0.0 {
        return f32::INFINITY;
    }
    primary + orthogonal_weight * orthogonal
}

fn axis_overlap(a_min: f32, a_max: f32, b_min: f32, b_max: f32) -> f32 {
    (a_max.min(b_max) - a_min.max(b_min)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focusable_content_cache_invalidates_when_can_focus_toggles_after_caching() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::structural_scope());
        let child = manager.create_node(FocusNode::structural_scope());
        manager.add_child(None, scope);
        manager.add_child(Some(scope), child);

        assert!(!manager.has_focusable_content());
        assert_eq!(manager.focusable_content_cache.get(), Some(false));

        assert!(manager.set_node_can_focus(child, true));
        assert_eq!(manager.focusable_content_cache.get(), None);
        assert!(manager.has_focusable_content());

        assert!(manager.set_node_can_focus(child, false));
        assert!(!manager.has_focusable_content());
    }

    #[test]
    fn focusable_content_cache_invalidates_when_backing_toggles_after_caching() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::structural_scope());
        let child = manager.create_node(FocusNode::structural_scope());
        manager.add_child(None, scope);
        manager.add_child(Some(scope), child);

        assert!(!manager.has_focusable_content());

        assert!(manager.set_node_focusable(child, Some(RuntimeFocusable::new(1, 2, 3))));
        assert!(manager.has_focusable_content());

        assert!(manager.set_node_focusable(child, None));
        assert!(!manager.has_focusable_content());
    }

    #[test]
    fn focusable_content_cache_invalidates_when_a_backed_node_is_added_then_removed() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::structural_scope());
        manager.add_child(None, scope);

        assert!(!manager.has_focusable_content());

        let mut backed = FocusNode::structural_scope();
        backed.set_focusable(RuntimeFocusable::new(1, 2, 3));
        let backed = manager.create_node(backed);
        manager.add_child(Some(scope), backed);
        assert!(manager.has_focusable_content());

        manager.remove_subtree(backed);
        assert!(!manager.has_focusable_content());
    }

    #[test]
    fn focusable_content_cache_invalidates_when_the_last_root_migrates() {
        let mut first = FocusManager::new();
        let node = first.create_node(FocusNode::new());
        first.add_child(None, node);
        assert!(first.has_focusable_content());

        let mut second = FocusManager::new();
        assert!(second.migrate_subtree_from(&mut first, node, None, 0));

        assert!(!first.has_focusable_content());
        assert!(second.has_focusable_content());
    }

    #[test]
    fn unchanged_retained_updates_preserve_the_focusable_content_cache() {
        let mut manager = FocusManager::new();
        let root = manager.create_node(FocusNode::new());
        manager.add_child(None, root);
        assert!(manager.has_focusable_content());

        let mut replacement = manager.node(root).expect("root").clone();
        replacement.set_bounds(Some(FocusBounds::from_xywh(1.0, 2.0, 3.0, 4.0)));
        assert!(manager.update_node(root, &replacement));
        assert!(manager.insert_child(None, root, 0));

        assert_eq!(manager.focusable_content_cache.get(), Some(true));
    }
}
