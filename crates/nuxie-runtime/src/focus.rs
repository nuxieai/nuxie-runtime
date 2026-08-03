use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::parent_traversal::{ParentTraversal, ParentTraversalFrame};
use crate::properties::property_key_for_name;
use crate::{ArtboardInstance, Mat2D};

/// Stable identity for one node in a focus tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FocusNodeId(u64);

impl FocusNodeId {
    fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusPoint {
    pub x: f32,
    pub y: f32,
}

impl FocusPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl FocusBounds {
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(x, y, x + width, y + height)
    }

    pub fn center(self) -> FocusPoint {
        FocusPoint::new(
            (self.min_x + self.max_x) * 0.5,
            (self.min_y + self.max_y) * 0.5,
        )
    }

    fn is_valid(self) -> bool {
        [self.min_x, self.min_y, self.max_x, self.max_y]
            .into_iter()
            .all(f32::is_finite)
            && self.min_x < self.max_x
            && self.min_y < self.max_y
    }
}

/// Runtime state for one authored focus target.
#[derive(Debug, Clone)]
pub struct FocusNode {
    parent: Option<FocusNodeId>,
    children: Vec<FocusNodeId>,
    has_focusable: bool,
    can_focus: bool,
    can_touch: bool,
    can_traverse: bool,
    eligible: bool,
    tab_index: i16,
    edge_behavior: FocusEdgeBehavior,
    bounds: Option<FocusBounds>,
    position: Option<FocusPoint>,
    has_focus: bool,
}

impl FocusNode {
    pub fn new() -> Self {
        Self {
            parent: None,
            children: Vec::new(),
            has_focusable: false,
            can_focus: true,
            can_touch: true,
            can_traverse: true,
            eligible: true,
            tab_index: 0,
            edge_behavior: FocusEdgeBehavior::ParentScope,
            bounds: None,
            position: None,
            has_focus: false,
        }
    }

    pub fn structural_scope() -> Self {
        Self {
            has_focusable: false,
            can_focus: false,
            can_touch: false,
            can_traverse: false,
            ..Self::new()
        }
    }

    pub fn can_focus(&self) -> bool {
        self.can_focus
    }

    pub fn set_can_focus(&mut self, value: bool) {
        self.can_focus = value;
    }

    pub fn can_touch(&self) -> bool {
        self.can_touch
    }

    pub fn set_can_touch(&mut self, value: bool) {
        self.can_touch = value;
    }

    pub fn can_traverse(&self) -> bool {
        self.can_traverse
    }

    pub fn set_can_traverse(&mut self, value: bool) {
        self.can_traverse = value;
    }

    pub fn is_eligible(&self) -> bool {
        self.eligible
    }

    pub fn set_eligible(&mut self, value: bool) {
        self.eligible = value;
    }

    pub fn tab_index(&self) -> i16 {
        self.tab_index
    }

    pub fn set_tab_index(&mut self, value: i16) {
        self.tab_index = value;
    }

    pub fn edge_behavior(&self) -> FocusEdgeBehavior {
        self.edge_behavior
    }

    pub fn set_edge_behavior(&mut self, value: FocusEdgeBehavior) {
        self.edge_behavior = value;
    }

    pub fn bounds(&self) -> Option<FocusBounds> {
        self.bounds
    }

    pub fn set_bounds(&mut self, value: Option<FocusBounds>) {
        self.bounds = value.filter(|bounds| bounds.is_valid());
    }

    pub fn position(&self) -> Option<FocusPoint> {
        self.position
    }

    pub fn set_position(&mut self, value: Option<FocusPoint>) {
        self.position = value.filter(|point| point.x.is_finite() && point.y.is_finite());
    }

    pub fn has_focus(&self) -> bool {
        self.has_focus
    }
}

impl Default for FocusNode {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FocusEdgeBehavior {
    #[default]
    ParentScope,
    ClosedLoop,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusEventKind {
    Focused,
    Blurred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusEvent {
    pub node_id: FocusNodeId,
    pub kind: FocusEventKind,
}

impl FocusEvent {
    pub fn new(node_id: FocusNodeId, kind: FocusEventKind) -> Self {
        Self { node_id, kind }
    }
}

/// Owns focus topology and focus state for one mounted focus domain.
#[derive(Debug, Clone, Default)]
pub struct FocusManager {
    nodes: BTreeMap<FocusNodeId, FocusNode>,
    roots: Vec<FocusNodeId>,
    primary_focus: Option<FocusNodeId>,
    pending_events: Vec<FocusEvent>,
    // High-level runtimes poll this every frame. Cache the tree walk until an
    // input to the predicate changes, matching C++ FocusManager.
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

        // RuntimeFocusTree projects the retained authored tree every advance.
        // Preserve the upstream O(1) cached path when that projection did not
        // actually change this node's parent or sibling position.
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
        self.nodes.get_mut(&child).expect("validated child").parent = parent;
        if let Some(parent) = parent {
            let children = &mut self
                .nodes
                .get_mut(&parent)
                .expect("validated parent")
                .children;
            children.insert(index.min(children.len()), child);
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
        self.nodes.get_mut(&node_id).expect("validated node").parent = None;
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
        // Callers with unrestricted mutable access can change can_focus or
        // focusable backing. Invalidate conservatively; hot synchronization
        // uses update_node below and only invalidates on predicate changes.
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

    pub fn set_node_has_focusable(&mut self, node_id: FocusNodeId, value: bool) -> bool {
        let Some(node) = self.nodes.get_mut(&node_id) else {
            return false;
        };
        if node.has_focusable == value {
            return false;
        }
        node.has_focusable = value;
        self.mark_focusable_content_dirty();
        true
    }

    fn update_node(&mut self, node_id: FocusNodeId, replacement: &FocusNode) -> bool {
        let Some(node) = self.nodes.get_mut(&node_id) else {
            return false;
        };
        let predicate_changed = node.has_focusable != replacement.has_focusable
            || node.can_focus() != replacement.can_focus();
        node.has_focusable = replacement.has_focusable;
        node.set_can_focus(replacement.can_focus());
        node.set_can_touch(replacement.can_touch());
        node.set_can_traverse(replacement.can_traverse());
        node.set_eligible(replacement.is_eligible());
        node.set_tab_index(replacement.tab_index());
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

    fn unlink(&mut self, node_id: FocusNodeId) {
        let parent = self.nodes.get(&node_id).and_then(|node| node.parent);
        let mut removed = false;
        if let Some(parent) = parent {
            if let Some(parent) = self.nodes.get_mut(&parent) {
                let old_len = parent.children.len();
                parent.children.retain(|child| *child != node_id);
                removed = parent.children.len() != old_len;
            }
        } else {
            let old_len = self.roots.len();
            self.roots.retain(|root| *root != node_id);
            removed = self.roots.len() != old_len;
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

    fn ancestor_chain(&self, node_id: FocusNodeId) -> Vec<FocusNodeId> {
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
        node.has_focusable
            || node.can_focus
            || node
                .children
                .iter()
                .copied()
                .any(|child| self.subtree_has_focusable_content(child))
    }

    fn node_eligible_for_focus(&self, node_id: FocusNodeId) -> bool {
        self.nodes
            .get(&node_id)
            .is_some_and(|node| node.can_focus && node.eligible)
    }

    fn node_eligible_for_traversal(&self, node_id: FocusNodeId) -> bool {
        self.nodes
            .get(&node_id)
            .is_some_and(|node| node.can_traverse && node.can_focus && node.eligible)
    }

    fn node_traversable(&self, node_id: FocusNodeId) -> bool {
        let Some(node) = self.nodes.get(&node_id) else {
            return false;
        };
        self.node_eligible_for_traversal(node_id)
            || (!node.has_focusable
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
        traversable.sort_by_key(|node_id| self.nodes.get(node_id).map_or(0, |node| node.tab_index));
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
            match node.edge_behavior {
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
            .map_or(FocusEdgeBehavior::ParentScope, |node| node.edge_behavior)
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
        let current_bounds = self.nodes.get(&current).and_then(|node| node.bounds);
        let current_position = current_bounds
            .map(FocusBounds::center)
            .or_else(|| self.nodes.get(&current).and_then(|node| node.position))?;
        let mut candidates = Vec::new();
        self.collect_traversable_leaves(&self.roots, &mut candidates);

        let mut best = None;
        let mut best_score = f32::INFINITY;
        for candidate in candidates {
            if candidate == current {
                continue;
            }
            let candidate_node = self.nodes.get(&candidate).expect("collected node");
            let score = match (current_bounds, candidate_node.bounds) {
                (Some(current), Some(candidate)) => {
                    score_directional_bounds(current, candidate, direction)
                }
                _ => {
                    let Some(candidate_position) = candidate_node
                        .bounds
                        .map(FocusBounds::center)
                        .or(candidate_node.position)
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

// Runtime projection of authored FocusData. FocusManager deliberately owns no
// Rive-object knowledge; this layer mirrors Artboard::buildFocusTree and keeps
// occurrence identity stable while nested artboards and component-list rows
// are rebuilt or reordered.

const FOCUS_KEY_ROOT: u64 = 1;
const FOCUS_KEY_NESTED: u64 = 2;
const FOCUS_KEY_LIST_SCOPE: u64 = 3;
const FOCUS_KEY_LIST_ROW: u64 = 4;
const FOCUS_KEY_AUTHORED: u64 = 5;
const FOCUS_KEY_NESTED_CHILD: u64 = 6;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RuntimeFocusOccurrenceKey(Vec<u64>);

impl RuntimeFocusOccurrenceKey {
    fn root(graph_global_id: u32) -> Self {
        Self(vec![FOCUS_KEY_ROOT, u64::from(graph_global_id)])
    }

    fn child(&self, tag: u64, first: u64, second: u64) -> Self {
        let mut value = self.0.clone();
        value.extend([tag, first, second]);
        Self(value)
    }
}

#[derive(Debug, Clone)]
struct RuntimeFocusDescriptor {
    key: RuntimeFocusOccurrenceKey,
    parent: Option<RuntimeFocusOccurrenceKey>,
    sibling_index: usize,
    node: FocusNode,
    /// Owning Artboard occurrence, target Node, and the exact direct
    /// FocusData child selected in authored order.
    target: Option<(u64, usize, usize)>,
}

#[derive(Debug, Clone, Default)]
struct RuntimeFocusDomain {
    manager: FocusManager,
    targets: BTreeMap<(u64, usize), FocusNodeId>,
    focus_data_by_target: BTreeMap<(u64, usize), usize>,
}

/// One state-machine instance's authored focus domain.
///
/// The keys describe concrete mounted occurrences, not just file-global
/// objects. A component-list row therefore retains its FocusNode when moved,
/// while a genuinely removed row is blurred and discarded.
#[derive(Debug)]
pub(crate) struct RuntimeFocusTree {
    inert: bool,
    domain: Rc<RefCell<RuntimeFocusDomain>>,
    owner_identity: u64,
    owns_projection: bool,
    nodes_by_key: BTreeMap<RuntimeFocusOccurrenceKey, FocusNodeId>,
    parents_by_key: BTreeMap<RuntimeFocusOccurrenceKey, Option<RuntimeFocusOccurrenceKey>>,
}

impl Default for RuntimeFocusTree {
    fn default() -> Self {
        Self {
            inert: false,
            domain: Rc::new(RefCell::new(RuntimeFocusDomain::default())),
            owner_identity: 0,
            owns_projection: true,
            nodes_by_key: BTreeMap::new(),
            parents_by_key: BTreeMap::new(),
        }
    }
}

impl Clone for RuntimeFocusTree {
    fn clone(&self) -> Self {
        // A public Rust state-machine snapshot is a new occurrence. Copy the
        // retained focus domain rather than aliasing focus mutations back to
        // the source occurrence. Nested machines are reattached to the new
        // root domain when their owning parent instance is constructed.
        let domain = self.domain.borrow().clone();
        // Public Clone is Rust's explicit state snapshot. Preserve owned
        // pending focus/blur values in the new non-aliased manager, just as
        // StateMachineInstance preserves callbacks already translated into
        // its own queue. A cold remount still starts empty through `default`.
        Self {
            inert: self.inert,
            domain: Rc::new(RefCell::new(domain)),
            owner_identity: self.owner_identity,
            owns_projection: self.owns_projection,
            nodes_by_key: self.nodes_by_key.clone(),
            parents_by_key: self.parents_by_key.clone(),
        }
    }
}

impl RuntimeFocusTree {
    pub(crate) fn owner_identity(&self) -> u64 {
        self.owner_identity
    }

    /// Owner-safe identity comparison for the StateMachineInstance selection
    /// seam. This observes shared ownership only; it does not expose or port
    /// RECORDED focus-manager internals from manifest row B6-0238.
    pub(crate) fn shares_manager(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.domain, &other.domain)
    }

    /// Create the focus-manager identity used by one state-machine occurrence
    /// without building the authored focus topology yet.
    ///
    /// Pinned C++ constructs `m_focusManager` with the
    /// `StateMachineInstance`, runs every layer's initial entry callbacks, and
    /// only then calls `Artboard::buildFocusTree`
    /// (`state_machine_instance.cpp:1747-1752,2123-2127`). Entry-time focus
    /// actions therefore see an empty manager. `FocusActionTarget` may still
    /// lazily create its one target `FocusNode` through
    /// `FocusData::focusNode`, but that node has no attached descendants until
    /// the final build pass (`focus_action_target.cpp:14-40`;
    /// `focus_data.cpp:55-69`).
    pub(crate) fn new_unsynchronized(artboard: &ArtboardInstance) -> Self {
        Self {
            owner_identity: artboard.instance_identity(),
            ..Self::default()
        }
    }

    /// Perform the first complete authored-tree build after initial layer
    /// callbacks have finished.
    pub(crate) fn synchronize_after_layer_initialization(&mut self, artboard: &ArtboardInstance) {
        self.sync(artboard);
        // An empty projection cannot gain authored focus content later: lists
        // and data-bound nested hosts contribute persistent structural scopes
        // even while empty. Keep the common no-focus advance path O(1).
        self.inert = self.nodes_by_key.is_empty();
    }

    /// Install the same manager used by the parent occurrence while retaining
    /// the child occurrence's own authored target namespace. Pinned C++ calls
    /// `setExternalFocusManager` before `syncNestedStateMachine` so the child
    /// contributes to one traversal domain without copying manager state.
    pub(crate) fn external_for_owner(&self, owner_identity: u64) -> Self {
        Self {
            inert: self.inert,
            domain: Rc::clone(&self.domain),
            owner_identity,
            owns_projection: false,
            nodes_by_key: BTreeMap::new(),
            parents_by_key: BTreeMap::new(),
        }
    }

    #[inline]
    pub(crate) fn is_inert(&self) -> bool {
        self.inert
    }

    pub(crate) fn sync(&mut self, artboard: &ArtboardInstance) {
        if self.inert || !self.owns_projection {
            return;
        }
        let mut descriptors = Vec::new();
        let root_key = RuntimeFocusOccurrenceKey::root(artboard.graph_global_id);
        collect_artboard_focus_descriptors(
            artboard,
            &root_key,
            None,
            Some(artboard.instance_identity()),
            true,
            Mat2D::IDENTITY,
            &mut descriptors,
        );
        let mut sibling_counts = BTreeMap::new();
        for descriptor in &mut descriptors {
            let sibling_index = sibling_counts.entry(descriptor.parent.clone()).or_insert(0);
            descriptor.sibling_index = *sibling_index;
            *sibling_index += 1;
        }

        let desired = descriptors
            .iter()
            .map(|descriptor| descriptor.key.clone())
            .collect::<BTreeSet<_>>();
        let removed = self
            .nodes_by_key
            .keys()
            .filter(|key| !desired.contains(*key))
            .cloned()
            .collect::<BTreeSet<_>>();
        let removed_roots = removed
            .iter()
            .filter(|key| {
                self.parents_by_key
                    .get(*key)
                    .and_then(Option::as_ref)
                    .is_none_or(|parent| !removed.contains(parent))
            })
            .cloned()
            .collect::<Vec<_>>();
        for key in removed_roots {
            if let Some(node_id) = self.nodes_by_key.get(&key).copied() {
                self.domain.borrow_mut().manager.remove_subtree(node_id);
            }
        }
        self.nodes_by_key
            .retain(|_, node_id| self.domain.borrow().manager.contains(*node_id));

        for descriptor in &descriptors {
            let node_id = match self.nodes_by_key.get(&descriptor.key).copied() {
                Some(node_id) => {
                    self.domain
                        .borrow_mut()
                        .manager
                        .update_node(node_id, &descriptor.node);
                    node_id
                }
                None => {
                    let node_id = self
                        .domain
                        .borrow_mut()
                        .manager
                        .create_node(descriptor.node.clone());
                    self.nodes_by_key.insert(descriptor.key.clone(), node_id);
                    node_id
                }
            };
            let parent = descriptor
                .parent
                .as_ref()
                .and_then(|key| self.nodes_by_key.get(key))
                .copied();
            self.domain.borrow_mut().manager.insert_child(
                parent,
                node_id,
                descriptor.sibling_index,
            );
        }

        self.parents_by_key = descriptors
            .iter()
            .map(|descriptor| (descriptor.key.clone(), descriptor.parent.clone()))
            .collect();
        let mut domain = self.domain.borrow_mut();
        domain.targets.clear();
        domain.focus_data_by_target.clear();
        for descriptor in descriptors {
            let Some((owner_identity, target_local, focus_data_local)) = descriptor.target else {
                continue;
            };
            let Some(node_id) = self.nodes_by_key.get(&descriptor.key).copied() else {
                continue;
            };
            let target = (owner_identity, target_local);
            // `FocusActionTarget` scans direct children in authored order and
            // stops at the first FocusData. Retain that first occurrence even
            // when malformed input authored a second direct FocusData child.
            domain.targets.entry(target).or_insert(node_id);
            domain
                .focus_data_by_target
                .entry(target)
                .or_insert(focus_data_local);
        }
        domain.manager.drop_focus_if_ineligible();
    }

    pub(crate) fn has_focusable_content(&self) -> bool {
        self.domain.borrow().manager.has_focusable_content()
    }

    pub(crate) fn set_focus_target_before_topology(
        &mut self,
        artboard: &ArtboardInstance,
        target_local: usize,
        focus_data_local: usize,
    ) -> bool {
        self.ensure_unattached_target(artboard, target_local, focus_data_local);
        self.set_focus_target(target_local)
    }

    pub(crate) fn set_focus_target(&mut self, target_local: usize) -> bool {
        let mut domain = self.domain.borrow_mut();
        domain
            .targets
            .get(&(self.owner_identity, target_local))
            .copied()
            .is_some_and(|node_id| domain.manager.set_focus(node_id))
    }

    /// Mirror the constructor-time `FocusData::focusNode()` path without
    /// attaching any other authored node or descendant. The later full sync
    /// reuses this exact occurrence identity and places it into the completed
    /// tree.
    fn ensure_unattached_target(
        &mut self,
        artboard: &ArtboardInstance,
        target_local: usize,
        focus_data_local: usize,
    ) {
        let target = (self.owner_identity, target_local);
        if self
            .domain
            .borrow()
            .focus_data_by_target
            .get(&target)
            .copied()
            == Some(focus_data_local)
        {
            return;
        }

        let mut descriptors = Vec::new();
        let root_key = RuntimeFocusOccurrenceKey::root(artboard.graph_global_id);
        collect_artboard_focus_descriptors(
            artboard,
            &root_key,
            None,
            Some(artboard.instance_identity()),
            true,
            Mat2D::IDENTITY,
            &mut descriptors,
        );
        let Some(descriptor) = descriptors
            .into_iter()
            .find(|descriptor| descriptor.target == Some((target.0, target.1, focus_data_local)))
        else {
            return;
        };
        let Some((_, _, focus_data_local)) = descriptor.target else {
            return;
        };

        let node_id = match self.nodes_by_key.get(&descriptor.key).copied() {
            Some(node_id) => node_id,
            None => {
                let node_id = self
                    .domain
                    .borrow_mut()
                    .manager
                    .create_node(descriptor.node);
                self.nodes_by_key.insert(descriptor.key.clone(), node_id);
                node_id
            }
        };
        self.parents_by_key.insert(descriptor.key, None);
        let mut domain = self.domain.borrow_mut();
        domain.targets.insert(target, node_id);
        domain.focus_data_by_target.insert(target, focus_data_local);
    }

    pub(crate) fn clear_focus(&mut self) -> bool {
        self.domain.borrow_mut().manager.clear_focus()
    }

    pub(crate) fn traverse(&mut self, traversal_kind: u64) -> bool {
        let mut domain = self.domain.borrow_mut();
        match traversal_kind {
            0 => domain.manager.focus_next(),
            1 => domain.manager.focus_previous(),
            2 => domain.manager.focus_up(),
            3 => domain.manager.focus_down(),
            4 => domain.manager.focus_left(),
            5 => domain.manager.focus_right(),
            _ => domain.manager.focus_next(),
        }
    }

    pub(crate) fn target_has_focus(&self, target_local: usize) -> bool {
        let domain = self.domain.borrow();
        domain
            .targets
            .get(&(self.owner_identity, target_local))
            .copied()
            .is_some_and(|node_id| domain.manager.has_focus(node_id))
    }

    /// Owner-safe existence projection for StateMachineInstance::setFocus.
    ///
    /// This does not expose or implement the RECORDED focus-manager tree/node
    /// seam owned by manifest row B6-0238. It only distinguishes a retained
    /// FocusData occurrence from the C++ null-FocusData/null-node branch.
    pub(crate) fn has_focus_target(&self, target_local: usize) -> bool {
        self.domain
            .borrow()
            .targets
            .contains_key(&(self.owner_identity, target_local))
    }

    /// Cheap owner-safe projection used by host FocusState polling.
    pub(crate) fn has_primary_focus(&self) -> bool {
        self.domain.borrow().manager.primary_focus().is_some()
    }

    /// Return focused listener targets from the primary leaf toward the root.
    ///
    /// C++ `FocusManager::{keyInput,textInput,gamepadDispatch}` bubbles along
    /// this exact node chain and only then applies registration order within
    /// each `FocusData` (`focus_manager.cpp:702-751`).
    pub(crate) fn focused_listener_chain(&self) -> Vec<(u64, usize, usize)> {
        let domain = self.domain.borrow();
        let Some(primary) = domain.manager.primary_focus() else {
            return Vec::new();
        };
        domain
            .manager
            .ancestor_chain(primary)
            .into_iter()
            .filter_map(|node_id| {
                let (owner_identity, target_local) = domain
                    .targets
                    .iter()
                    .find_map(|(target, candidate)| (*candidate == node_id).then_some(*target))?;
                let focus_data_local = domain
                    .focus_data_by_target
                    .get(&(owner_identity, target_local))
                    .copied()?;
                Some((owner_identity, target_local, focus_data_local))
            })
            .collect()
    }

    /// Drain this occurrence's focus callbacks as authored target ids.
    ///
    /// `FocusManager` owns the concrete node identities while listener groups
    /// retain `FocusData` occurrences. Pinned C++ delivers the manager
    /// callback to those groups, which enqueue an occurrence-owned record on
    /// the state machine. Translating back through the retained target table
    /// preserves that ownership without rediscovering the artboard graph.
    pub(crate) fn take_owner_events(&mut self) -> Vec<(usize, usize, FocusEventKind)> {
        let mut domain = self.domain.borrow_mut();
        let events = std::mem::take(&mut domain.manager.pending_events);
        let mut owner_events = Vec::new();
        for event in events {
            let target = domain
                .targets
                .iter()
                .find_map(|(target, node_id)| (*node_id == event.node_id).then_some(*target));
            let Some((owner_identity, target_local)) = target else {
                // Structural scopes have no FocusData callback in C++.
                continue;
            };
            if owner_identity != self.owner_identity {
                domain.manager.pending_events.push(event);
                continue;
            }
            let Some(focus_data_local) = domain
                .focus_data_by_target
                .get(&(owner_identity, target_local))
                .copied()
            else {
                continue;
            };
            owner_events.push((target_local, focus_data_local, event.kind));
        }
        owner_events
    }

    /// Drop focus notifications produced before listener groups exist.
    ///
    /// Pinned C++ initializes every layer (including entry focus actions)
    /// before it constructs `FocusListenerGroup` occurrences. Those earlier
    /// callbacks therefore have no registered recipient and are not replayed
    /// after registration (`state_machine_instance.cpp:1747-1752,1829-1891`).
    pub(crate) fn discard_unregistered_events(&mut self) {
        self.domain.borrow_mut().manager.pending_events.clear();
    }
}

fn collect_artboard_focus_descriptors(
    artboard: &ArtboardInstance,
    occurrence_key: &RuntimeFocusOccurrenceKey,
    parent_focus: Option<RuntimeFocusOccurrenceKey>,
    target_owner: Option<u64>,
    inherited_eligible: bool,
    root_transform: Mat2D,
    descriptors: &mut Vec<RuntimeFocusDescriptor>,
) {
    let Some(graph) = artboard.runtime_graph() else {
        return;
    };
    let Some(root_local) = graph
        .components
        .iter()
        .find(|component| component.type_name == "Artboard" && component.parent_local.is_none())
        .map(|component| component.local_id)
    else {
        return;
    };
    collect_component_focus_descriptors(
        artboard,
        root_local,
        occurrence_key,
        parent_focus,
        target_owner,
        inherited_eligible,
        root_transform,
        descriptors,
    );
}

fn collect_component_focus_descriptors(
    artboard: &ArtboardInstance,
    local_id: usize,
    occurrence_key: &RuntimeFocusOccurrenceKey,
    parent_focus: Option<RuntimeFocusOccurrenceKey>,
    target_owner: Option<u64>,
    inherited_eligible: bool,
    root_transform: Mat2D,
    descriptors: &mut Vec<RuntimeFocusDescriptor>,
) {
    let Some(graph) = artboard.runtime_graph() else {
        return;
    };
    let Some(component) = graph
        .components
        .iter()
        .find(|component| component.local_id == local_id)
    else {
        return;
    };

    let mut host_parent = parent_focus.clone();
    if matches!(
        component.type_name,
        "NestedArtboard" | "NestedArtboardLayout" | "NestedArtboardLeaf"
    ) {
        let artboard_id_key = property_key_for_name("NestedArtboard", "artboardId");
        let data_bound = artboard_id_key.is_some_and(|property_key| {
            graph.data_binds.iter().any(|data_bind| {
                data_bind.target_local == Some(local_id)
                    && data_bind.property_key == u64::from(property_key)
            })
        });
        if data_bound {
            let scope_key = occurrence_key.child(
                FOCUS_KEY_NESTED,
                local_id as u64,
                u64::from(component.global_id),
            );
            push_focus_descriptor(
                descriptors,
                scope_key.clone(),
                parent_focus.clone(),
                FocusNode::structural_scope(),
                None,
            );
            host_parent = Some(scope_key);
        }
        if let Some(nested) = artboard.nested_artboards.get(&local_id) {
            let child_key = occurrence_key.child(
                FOCUS_KEY_NESTED_CHILD,
                local_id as u64,
                nested.child.instance_identity(),
            );
            collect_artboard_focus_descriptors(
                &nested.child,
                &child_key,
                host_parent.clone(),
                Some(nested.child.instance_identity()),
                inherited_eligible
                    && component_and_ancestors_allow_focus(artboard, local_id)
                    && !nested_host_is_paused(artboard, local_id),
                root_transform.multiply(
                    artboard
                        .component(local_id)
                        .map_or(Mat2D::IDENTITY, |host| host.transform.world_transform),
                ),
                descriptors,
            );
        }
    } else if component.type_name == "ArtboardComponentList" {
        let scope_key = occurrence_key.child(
            FOCUS_KEY_LIST_SCOPE,
            local_id as u64,
            u64::from(component.global_id),
        );
        push_focus_descriptor(
            descriptors,
            scope_key.clone(),
            parent_focus.clone(),
            FocusNode::structural_scope(),
            None,
        );
        if let Some(items) = artboard.component_list_items(local_id) {
            let host_transform_local =
                if crate::constraints::scrolling::scroll_virtualizer::component_list_virtualization(
                    artboard, local_id,
                )
                .is_some()
                {
                    component.parent_local.unwrap_or(local_id)
                } else {
                    local_id
                };
            let host_world = artboard
                .component(host_transform_local)
                .map_or(Mat2D::IDENTITY, |host| host.transform.world_transform);
            let item_transforms = artboard
                .component_list_state(local_id)
                .map(|list| &list.item_transforms);
            for (item_index, item) in items.iter().enumerate() {
                let row_key = occurrence_key.child(
                    FOCUS_KEY_LIST_ROW,
                    local_id as u64,
                    item.occurrence_identity,
                );
                push_focus_descriptor(
                    descriptors,
                    row_key.clone(),
                    Some(scope_key.clone()),
                    FocusNode::structural_scope(),
                    None,
                );
                let child_key = row_key.child(
                    FOCUS_KEY_ROOT,
                    u64::from(item.child.graph_global_id),
                    item.child.instance_identity(),
                );
                collect_artboard_focus_descriptors(
                    &item.child,
                    &child_key,
                    Some(row_key),
                    Some(item.child.instance_identity()),
                    inherited_eligible && component_and_ancestors_allow_focus(artboard, local_id),
                    root_transform.multiply(host_world).multiply(
                        item_transforms
                            .and_then(|transforms| transforms.get(item_index))
                            .copied()
                            .unwrap_or(item.transform),
                    ),
                    descriptors,
                );
            }
        }
    }

    let direct_focus = component.children.iter().copied().find(|child_local| {
        graph
            .components
            .iter()
            .find(|child| child.local_id == *child_local)
            .is_some_and(|child| child.type_name == "FocusData")
    });
    let recurse_parent = if let Some(focus_local) = direct_focus {
        let focus_key = occurrence_key.child(
            FOCUS_KEY_AUTHORED,
            focus_local as u64,
            graph
                .components
                .iter()
                .find(|child| child.local_id == focus_local)
                .map_or(0, |child| u64::from(child.global_id)),
        );
        push_focus_descriptor(
            descriptors,
            focus_key.clone(),
            parent_focus,
            authored_focus_node(artboard, focus_local, inherited_eligible, root_transform),
            target_owner.map(|owner| (owner, local_id, focus_local)),
        );
        Some(focus_key)
    } else {
        parent_focus
    };

    for child_local in &component.children {
        let is_focus_data = graph
            .components
            .iter()
            .find(|child| child.local_id == *child_local)
            .is_some_and(|child| child.type_name == "FocusData");
        if !is_focus_data {
            collect_component_focus_descriptors(
                artboard,
                *child_local,
                occurrence_key,
                recurse_parent.clone(),
                target_owner,
                inherited_eligible,
                root_transform,
                descriptors,
            );
        }
    }
}

fn push_focus_descriptor(
    descriptors: &mut Vec<RuntimeFocusDescriptor>,
    key: RuntimeFocusOccurrenceKey,
    parent: Option<RuntimeFocusOccurrenceKey>,
    node: FocusNode,
    target: Option<(u64, usize, usize)>,
) {
    descriptors.push(RuntimeFocusDescriptor {
        key,
        parent,
        sibling_index: 0,
        node,
        target,
    });
}

fn authored_focus_node(
    artboard: &ArtboardInstance,
    focus_local: usize,
    inherited_eligible: bool,
    root_transform: Mat2D,
) -> FocusNode {
    let mut node = FocusNode::new();
    // C++ FocusData::onAddedDirty wires the authored FocusData through a
    // Focusable into its FocusNode. A bare FocusNode starts with nullptr, but
    // a node constructed from authored FocusData is therefore backed.
    node.has_focusable = true;
    let focus_flags = property_key_for_name("FocusData", "focusFlags")
        .and_then(|property_key| artboard.objects.uint_property(focus_local, property_key))
        .unwrap_or(7);
    node.set_can_focus(focus_flags & 1 != 0);
    node.set_can_touch(focus_flags & 2 != 0);
    node.set_can_traverse(focus_flags & 4 != 0);
    let edge_behavior = property_key_for_name("FocusData", "edgeBehaviorValue")
        .and_then(|property_key| artboard.objects.uint_property(focus_local, property_key))
        .unwrap_or(0);
    node.set_edge_behavior(match edge_behavior {
        1 => FocusEdgeBehavior::ClosedLoop,
        2 => FocusEdgeBehavior::Stop,
        _ => FocusEdgeBehavior::ParentScope,
    });

    let eligible = inherited_eligible
        && artboard
            .component(focus_local)
            .is_none_or(|focus_data| !focus_data.is_collapsed());
    let parent_local = artboard.component_parent_local(focus_local);
    let eligible = eligible
        && parent_local
            .is_none_or(|parent_local| component_and_ancestors_allow_focus(artboard, parent_local));
    node.set_eligible(eligible);
    if let Some(parent) = parent_local.and_then(|parent_local| artboard.component(parent_local)) {
        let (x, y) = root_transform.transform_point(
            parent.transform.world_transform.0[4],
            parent.transform.world_transform.0[5],
        );
        node.set_position(Some(FocusPoint::new(x, y)));
    }
    node
}

fn component_and_ancestors_allow_focus(artboard: &ArtboardInstance, start_local: usize) -> bool {
    let drawable_flags_key = property_key_for_name("Drawable", "drawableFlags");
    let allows_focus = |artboard: &ArtboardInstance,
                        component: crate::components::ComponentHandle| {
        let Some(local_id) = artboard.component_local_id(component) else {
            return true;
        };
        let component = artboard.component_at(component);
        let is_hidden = drawable_flags_key
            .and_then(|property_key| artboard.objects.uint_property(local_id, property_key))
            .is_some_and(|flags| flags & 1 != 0);
        !(component.is_collapsed()
            || is_hidden
            || (component.capabilities.transform && component.transform.render_opacity <= 0.0))
    };
    let Some(start) = artboard.component_handle(start_local) else {
        return true;
    };
    if !allows_focus(artboard, start) {
        return false;
    }

    let frames = [ParentTraversalFrame {
        artboard,
        host_component_in_parent: None,
    }];
    let mut traversal = ParentTraversal::new(&frames, start);
    while let Some(parent) = traversal.next() {
        if !allows_focus(parent.artboard, parent.component) {
            return false;
        }
    }
    true
}

fn nested_host_is_paused(artboard: &ArtboardInstance, local_id: usize) -> bool {
    property_key_for_name("NestedArtboard", "isPaused")
        .and_then(|property_key| artboard.objects.bool_property(local_id, property_key))
        .unwrap_or(false)
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
    fn upstream_focus_node_defaults_and_property_setters() {
        let mut node = FocusNode::new();
        assert!(node.can_focus());
        assert!(node.can_touch());
        assert!(node.can_traverse());
        assert_eq!(node.tab_index(), 0);
        assert_eq!(node.edge_behavior(), FocusEdgeBehavior::ParentScope);
        assert!(node.parent.is_none());
        assert!(node.children.is_empty());
        assert!(!node.has_focus());

        node.set_can_focus(false);
        assert!(!node.can_focus());
        node.set_can_touch(false);
        assert!(!node.can_touch());
        node.set_can_traverse(false);
        assert!(!node.can_traverse());
        node.set_tab_index(42);
        assert_eq!(node.tab_index(), 42);
        node.set_edge_behavior(FocusEdgeBehavior::ClosedLoop);
        assert_eq!(node.edge_behavior(), FocusEdgeBehavior::ClosedLoop);
        node.set_edge_behavior(FocusEdgeBehavior::Stop);
        assert_eq!(node.edge_behavior(), FocusEdgeBehavior::Stop);
    }

    #[test]
    fn upstream_focus_node_fresh_focusable_defaults_to_null() {
        let node = FocusNode::new();
        assert!(
            !node.has_focusable,
            "focus_test.cpp:88 expects a fresh FocusNode::focusable() to be null"
        );
    }

    #[test]
    #[ignore = "coverage finding: docs/runtime-frame-loop-test-backfill-bc.md#finding-focus-fixture-surface"]
    fn upstream_focusable_identity_and_fixture_swap_contracts_need_runtime_occurrence_surface() {
        panic!(
            "focus_test.cpp exposes Focusable pointer identity/delegation and exact bindable \
             artboard/list occurrence swaps that nuxie-runtime's public focus API does not expose"
        );
    }

    #[test]
    fn focusing_child_notifies_leaf_and_ancestors() {
        let mut manager = FocusManager::new();
        let parent = manager.create_node(FocusNode::new());
        let child = manager.create_node(FocusNode::new());

        assert!(manager.add_child(None, parent));
        assert!(manager.add_child(Some(parent), child));
        assert!(manager.set_focus(child));

        assert_eq!(manager.primary_focus(), Some(child));
        assert!(!manager.has_primary_focus(parent));
        assert!(manager.has_primary_focus(child));
        assert!(manager.has_focus(child));
        assert!(manager.has_focus(parent));
        assert_eq!(
            manager.take_events(),
            vec![
                FocusEvent::new(child, FocusEventKind::Focused),
                FocusEvent::new(parent, FocusEventKind::Focused),
            ]
        );
    }

    #[test]
    fn clearing_focus_blurs_leaf_and_ancestors() {
        let mut manager = FocusManager::new();
        let parent = manager.create_node(FocusNode::new());
        let child = manager.create_node(FocusNode::new());
        manager.add_child(None, parent);
        manager.add_child(Some(parent), child);
        manager.set_focus(child);
        manager.take_events();

        assert!(manager.clear_focus());

        assert_eq!(manager.primary_focus(), None);
        assert!(!manager.has_focus(child));
        assert!(!manager.has_focus(parent));
        assert_eq!(
            manager.take_events(),
            vec![
                FocusEvent::new(child, FocusEventKind::Blurred),
                FocusEvent::new(parent, FocusEventKind::Blurred),
            ]
        );
    }

    #[test]
    fn moving_between_siblings_does_not_renotify_the_common_ancestor() {
        let mut manager = FocusManager::new();
        let parent = manager.create_node(FocusNode::new());
        let first = manager.create_node(FocusNode::new());
        let second = manager.create_node(FocusNode::new());
        manager.add_child(None, parent);
        manager.add_child(Some(parent), first);
        manager.add_child(Some(parent), second);
        manager.set_focus(first);
        manager.take_events();

        assert!(manager.set_focus(second));

        assert_eq!(manager.primary_focus(), Some(second));
        assert!(manager.has_focus(parent));
        assert!(!manager.has_focus(first));
        assert!(manager.has_focus(second));
        assert_eq!(
            manager.take_events(),
            vec![
                FocusEvent::new(first, FocusEventKind::Blurred),
                FocusEvent::new(second, FocusEventKind::Focused),
            ]
        );
    }

    #[test]
    fn inserting_an_existing_subtree_reorders_without_blurring() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::new());
        let first = manager.create_node(FocusNode::new());
        let second = manager.create_node(FocusNode::new());
        manager.add_child(None, scope);
        manager.add_child(Some(scope), first);
        manager.add_child(Some(scope), second);
        manager.set_focus(second);
        manager.take_events();

        assert!(manager.insert_child(Some(scope), second, 0));

        assert_eq!(manager.children(scope), Some(&[second, first][..]));
        assert_eq!(manager.primary_focus(), Some(second));
        assert!(manager.take_events().is_empty());
    }

    #[test]
    fn inserting_an_ancestor_below_its_descendant_is_rejected_without_mutation() {
        let mut manager = FocusManager::new();
        let root = manager.create_node(FocusNode::new());
        let middle = manager.create_node(FocusNode::new());
        let leaf = manager.create_node(FocusNode::new());
        manager.add_child(None, root);
        manager.add_child(Some(root), middle);
        manager.add_child(Some(middle), leaf);

        assert!(!manager.insert_child(Some(leaf), root, 0));

        assert_eq!(manager.roots(), &[root]);
        assert_eq!(manager.parent(root), None);
        assert_eq!(manager.parent(middle), Some(root));
        assert_eq!(manager.parent(leaf), Some(middle));
    }

    #[test]
    fn detaching_a_focused_subtree_preserves_focus_for_reattachment() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::new());
        let row = manager.create_node(FocusNode::new());
        let leaf = manager.create_node(FocusNode::new());
        manager.add_child(None, scope);
        manager.add_child(Some(scope), row);
        manager.add_child(Some(row), leaf);
        manager.set_focus(leaf);
        manager.take_events();

        assert!(manager.detach_subtree(row));
        assert!(!manager.is_attached(row));
        assert_eq!(manager.primary_focus(), Some(leaf));
        assert!(manager.take_events().is_empty());

        assert!(manager.insert_child(Some(scope), row, 0));
        assert!(manager.is_attached(row));
        assert_eq!(manager.primary_focus(), Some(leaf));
        assert!(manager.take_events().is_empty());
    }

    #[test]
    fn removing_a_focused_subtree_blurs_and_invalidates_every_node() {
        let mut manager = FocusManager::new();
        let parent = manager.create_node(FocusNode::new());
        let child = manager.create_node(FocusNode::new());
        manager.add_child(None, parent);
        manager.add_child(Some(parent), child);
        manager.set_focus(child);
        manager.take_events();

        assert!(manager.remove_subtree(parent));

        assert_eq!(manager.primary_focus(), None);
        assert!(!manager.contains(parent));
        assert!(!manager.contains(child));
        assert_eq!(
            manager.take_events(),
            vec![
                FocusEvent::new(child, FocusEventKind::Blurred),
                FocusEvent::new(parent, FocusEventKind::Blurred),
            ]
        );
    }

    #[test]
    fn migrating_a_subtree_preserves_ids_after_the_old_manager_is_dropped() {
        let mut parent_manager = FocusManager::new();
        let parent = parent_manager.create_node(FocusNode::new());
        parent_manager.add_child(None, parent);

        let (scope, leaf) = {
            let mut internal_manager = FocusManager::new();
            let scope = internal_manager.create_node(FocusNode::new());
            let leaf = internal_manager.create_node(FocusNode::new());
            internal_manager.add_child(None, scope);
            internal_manager.add_child(Some(scope), leaf);

            assert!(parent_manager.migrate_subtree_from(
                &mut internal_manager,
                scope,
                Some(parent),
                0,
            ));
            assert!(internal_manager.roots().is_empty());
            assert!(!internal_manager.contains(scope));
            (scope, leaf)
        };

        assert!(parent_manager.contains(scope));
        assert!(parent_manager.contains(leaf));
        assert_eq!(parent_manager.parent(scope), Some(parent));
        assert_eq!(parent_manager.children(scope), Some(&[leaf][..]));
    }

    #[test]
    fn migrating_a_focused_subtree_transfers_focus_and_ancestry_events() {
        let mut source = FocusManager::new();
        let scope = source.create_node(FocusNode::new());
        let leaf = source.create_node(FocusNode::new());
        source.add_child(None, scope);
        source.add_child(Some(scope), leaf);
        source.set_focus(leaf);
        source.take_events();

        let mut target = FocusManager::new();
        let parent = target.create_node(FocusNode::new());
        target.add_child(None, parent);

        assert!(target.migrate_subtree_from(&mut source, scope, Some(parent), 0));

        assert_eq!(source.primary_focus(), None);
        assert_eq!(target.primary_focus(), Some(leaf));
        assert_eq!(
            source.take_events(),
            vec![
                FocusEvent::new(leaf, FocusEventKind::Blurred),
                FocusEvent::new(scope, FocusEventKind::Blurred),
            ]
        );
        assert_eq!(
            target.take_events(),
            vec![
                FocusEvent::new(leaf, FocusEventKind::Focused),
                FocusEvent::new(scope, FocusEventKind::Focused),
                FocusEvent::new(parent, FocusEventKind::Focused),
            ]
        );
    }

    #[test]
    fn focusable_content_ignores_empty_structural_scopes_but_counts_authored_nodes() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::structural_scope());
        manager.add_child(None, scope);
        assert!(!manager.has_focusable_content());

        let mut authored = FocusNode::new();
        authored.has_focusable = true;
        authored.set_can_focus(false);
        authored.set_can_traverse(false);
        authored.set_eligible(false);
        let authored = manager.create_node(authored);
        manager.add_child(Some(scope), authored);

        assert!(manager.has_focusable_content());
    }

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

        assert!(manager.set_node_has_focusable(child, true));
        assert!(manager.has_focusable_content());

        assert!(manager.set_node_has_focusable(child, false));
        assert!(!manager.has_focusable_content());
    }

    #[test]
    fn focusable_content_cache_invalidates_when_a_backed_node_is_added_then_removed() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::structural_scope());
        manager.add_child(None, scope);

        assert!(!manager.has_focusable_content());

        let mut backed = FocusNode::structural_scope();
        backed.has_focusable = true;
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
    fn unchanged_runtime_projection_preserves_focusable_content_cache() {
        let mut manager = FocusManager::new();
        let root = manager.create_node(FocusNode::new());
        manager.add_child(None, root);
        assert!(manager.has_focusable_content());

        let snapshot = manager.node(root).expect("root").clone();
        assert!(manager.update_node(root, &snapshot));
        assert!(manager.insert_child(None, root, 0));

        assert_eq!(manager.focusable_content_cache.get(), Some(true));
    }

    #[test]
    fn direct_focus_on_a_scope_resolves_to_its_first_traversable_leaf() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::new());
        let mut later = FocusNode::new();
        later.set_tab_index(1);
        let later = manager.create_node(later);
        let mut first = FocusNode::new();
        first.set_tab_index(-1);
        let first = manager.create_node(first);
        manager.add_child(None, scope);
        manager.add_child(Some(scope), later);
        manager.add_child(Some(scope), first);

        assert!(manager.set_focus(scope));

        assert_eq!(manager.primary_focus(), Some(first));
        assert!(manager.has_focus(scope));
        assert!(manager.has_focus(first));
        assert!(!manager.has_focus(later));
    }

    #[test]
    fn next_and_previous_traversal_follow_stable_tab_order_and_rest_on_leaves() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::new());
        let mut second = FocusNode::new();
        second.set_tab_index(1);
        let second = manager.create_node(second);
        let mut first = FocusNode::new();
        first.set_tab_index(-1);
        let first = manager.create_node(first);
        let tied = manager.create_node(FocusNode::new());
        manager.add_child(None, scope);
        manager.add_child(Some(scope), second);
        manager.add_child(Some(scope), first);
        manager.add_child(Some(scope), tied);

        assert!(manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(first));
        assert!(manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(tied));
        assert!(manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(second));
        assert!(manager.focus_previous());
        assert_eq!(manager.primary_focus(), Some(tied));
    }

    #[test]
    fn closed_loop_scope_wraps_at_both_edges() {
        let mut manager = FocusManager::new();
        let mut scope = FocusNode::new();
        scope.set_edge_behavior(FocusEdgeBehavior::ClosedLoop);
        let scope = manager.create_node(scope);
        let first = manager.create_node(FocusNode::new());
        let last = manager.create_node(FocusNode::new());
        manager.add_child(None, scope);
        manager.add_child(Some(scope), first);
        manager.add_child(Some(scope), last);
        manager.set_focus(last);

        assert!(manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(first));
        assert!(manager.focus_previous());
        assert_eq!(manager.primary_focus(), Some(last));
    }

    #[test]
    fn root_sequential_edges_clear_focus_like_cpp_find_next_focusable() {
        let mut manager = FocusManager::new();
        let first = manager.create_node(FocusNode::new());
        let last = manager.create_node(FocusNode::new());
        manager.add_child(None, first);
        manager.add_child(None, last);

        manager.set_focus(last);
        assert!(!manager.focus_next());
        assert_eq!(manager.primary_focus(), None);

        manager.set_focus(first);
        assert!(!manager.focus_previous());
        assert_eq!(manager.primary_focus(), None);
    }

    #[test]
    fn stop_scope_does_not_move_past_its_boundary() {
        let mut manager = FocusManager::new();
        let root = manager.create_node(FocusNode::new());
        let mut scope = FocusNode::new();
        scope.set_edge_behavior(FocusEdgeBehavior::Stop);
        let scope = manager.create_node(scope);
        let leaf = manager.create_node(FocusNode::new());
        let after = manager.create_node(FocusNode::new());
        manager.add_child(None, root);
        manager.add_child(Some(root), scope);
        manager.add_child(Some(scope), leaf);
        manager.add_child(Some(root), after);
        manager.set_focus(leaf);

        assert!(!manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(leaf));
    }

    #[test]
    fn parent_scope_edges_continue_with_the_scopes_siblings() {
        let mut manager = FocusManager::new();
        let root = manager.create_node(FocusNode::new());
        let before = manager.create_node(FocusNode::new());
        let scope = manager.create_node(FocusNode::new());
        let inner = manager.create_node(FocusNode::new());
        let after = manager.create_node(FocusNode::new());
        manager.add_child(None, root);
        manager.add_child(Some(root), before);
        manager.add_child(Some(root), scope);
        manager.add_child(Some(scope), inner);
        manager.add_child(Some(root), after);

        manager.set_focus(inner);
        assert!(manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(after));

        manager.set_focus(inner);
        assert!(manager.focus_previous());
        assert_eq!(manager.primary_focus(), Some(before));
    }

    #[test]
    fn only_unbacked_structural_scopes_are_transparent_to_traversal() {
        let mut manager = FocusManager::new();
        let mut authored_scope = FocusNode::new();
        authored_scope.has_focusable = true;
        authored_scope.set_can_focus(false);
        let authored_scope = manager.create_node(authored_scope);
        let blocked_leaf = manager.create_node(FocusNode::new());
        let structural_scope = manager.create_node(FocusNode::structural_scope());
        let reachable_leaf = manager.create_node(FocusNode::new());
        manager.add_child(None, authored_scope);
        manager.add_child(Some(authored_scope), blocked_leaf);
        manager.add_child(None, structural_scope);
        manager.add_child(Some(structural_scope), reachable_leaf);

        assert!(manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(reachable_leaf));
        assert!(!manager.has_focus(blocked_leaf));
        assert!(manager.has_focus(structural_scope));
    }

    #[test]
    fn direct_focus_on_an_ineligible_scope_does_not_reach_its_child() {
        let mut manager = FocusManager::new();
        let mut scope = FocusNode::new();
        scope.set_eligible(false);
        let scope = manager.create_node(scope);
        let child = manager.create_node(FocusNode::new());
        manager.add_child(None, scope);
        manager.add_child(Some(scope), child);

        assert!(!manager.set_focus(scope));
        assert_eq!(manager.primary_focus(), None);
        assert!(manager.take_events().is_empty());
    }

    #[test]
    fn focus_is_dropped_when_the_primary_node_becomes_ineligible() {
        let mut manager = FocusManager::new();
        let node = manager.create_node(FocusNode::new());
        manager.add_child(None, node);
        manager.set_focus(node);
        manager.take_events();
        manager
            .node_mut(node)
            .expect("focus node")
            .set_eligible(false);

        assert!(manager.drop_focus_if_ineligible());

        assert_eq!(manager.primary_focus(), None);
        assert_eq!(
            manager.take_events(),
            vec![FocusEvent::new(node, FocusEventKind::Blurred)]
        );
    }

    #[test]
    fn directional_scoring_prefers_axis_alignment_over_off_axis_nearness() {
        let mut manager = FocusManager::new();
        let mut current = FocusNode::new();
        current.set_bounds(Some(FocusBounds::from_xywh(0.0, 0.0, 10.0, 10.0)));
        let current = manager.create_node(current);
        let mut aligned = FocusNode::new();
        aligned.set_bounds(Some(FocusBounds::from_xywh(20.0, 0.0, 10.0, 10.0)));
        let aligned = manager.create_node(aligned);
        let mut off_axis = FocusNode::new();
        off_axis.set_bounds(Some(FocusBounds::from_xywh(11.0, 100.0, 10.0, 10.0)));
        let off_axis = manager.create_node(off_axis);
        manager.add_child(None, current);
        manager.add_child(None, off_axis);
        manager.add_child(None, aligned);
        manager.set_focus(current);

        assert!(manager.focus_right());
        assert_eq!(manager.primary_focus(), Some(aligned));
    }

    #[test]
    fn directional_scoring_falls_back_to_root_space_points() {
        let mut manager = FocusManager::new();
        let mut current = FocusNode::new();
        current.set_position(Some(FocusPoint::new(0.0, 0.0)));
        let current = manager.create_node(current);
        let mut aligned = FocusNode::new();
        aligned.set_position(Some(FocusPoint::new(20.0, 0.0)));
        let aligned = manager.create_node(aligned);
        let mut off_axis = FocusNode::new();
        off_axis.set_position(Some(FocusPoint::new(1.0, 100.0)));
        let off_axis = manager.create_node(off_axis);
        manager.add_child(None, current);
        manager.add_child(None, off_axis);
        manager.add_child(None, aligned);
        manager.set_focus(current);

        assert!(manager.focus_right());
        assert_eq!(manager.primary_focus(), Some(aligned));
    }

    #[test]
    fn empty_bounds_are_unavailable_for_directional_navigation() {
        let mut node = FocusNode::new();

        node.set_bounds(Some(FocusBounds::from_xywh(10.0, 20.0, 0.0, 5.0)));

        assert_eq!(node.bounds(), None);
    }

    #[test]
    fn directional_navigation_supports_all_four_directions() {
        let mut manager = FocusManager::new();
        let bounded = |x, y| {
            let mut node = FocusNode::new();
            node.set_bounds(Some(FocusBounds::from_xywh(x, y, 10.0, 10.0)));
            node
        };
        let center = manager.create_node(bounded(0.0, 0.0));
        let left = manager.create_node(bounded(-20.0, 0.0));
        let right = manager.create_node(bounded(20.0, 0.0));
        let up = manager.create_node(bounded(0.0, -20.0));
        let down = manager.create_node(bounded(0.0, 20.0));
        for node_id in [center, left, right, up, down] {
            manager.add_child(None, node_id);
        }

        manager.set_focus(center);
        assert!(manager.focus_left());
        assert_eq!(manager.primary_focus(), Some(left));
        manager.set_focus(center);
        assert!(manager.focus_right());
        assert_eq!(manager.primary_focus(), Some(right));
        manager.set_focus(center);
        assert!(manager.focus_up());
        assert_eq!(manager.primary_focus(), Some(up));
        manager.set_focus(center);
        assert!(manager.focus_down());
        assert_eq!(manager.primary_focus(), Some(down));
    }

    #[test]
    fn nested_occurrence_uses_parent_domain_but_snapshot_clone_isolated() {
        let root = RuntimeFocusTree {
            owner_identity: 11,
            ..RuntimeFocusTree::default()
        };
        let child_target = {
            let mut domain = root.domain.borrow_mut();
            let target = domain.manager.create_node(FocusNode::new());
            domain.manager.add_child(None, target);
            domain.targets.insert((22, 7), target);
            domain.focus_data_by_target.insert((22, 7), 8);
            target
        };
        let mut child = root.external_for_owner(22);

        assert!(child.set_focus_target(7));
        assert!(root.domain.borrow().manager.has_focus(child_target));
        let mut root = root;
        assert!(
            root.take_owner_events().is_empty(),
            "the parent occurrence must not consume a nested owner's callback"
        );
        assert_eq!(child.take_owner_events(), [(7, 8, FocusEventKind::Focused)]);

        let mut snapshot = child.clone();
        assert!(snapshot.clear_focus());
        assert!(root.domain.borrow().manager.has_focus(child_target));
    }

    #[test]
    fn listener_registration_does_not_replay_constructor_focus_callbacks() {
        let mut tree = RuntimeFocusTree {
            owner_identity: 11,
            ..RuntimeFocusTree::default()
        };
        let target = {
            let mut domain = tree.domain.borrow_mut();
            let target = domain.manager.create_node(FocusNode::new());
            domain.manager.add_child(None, target);
            domain.targets.insert((11, 7), target);
            domain.focus_data_by_target.insert((11, 7), 8);
            target
        };

        assert!(tree.set_focus_target(7));
        assert!(tree.domain.borrow().manager.has_focus(target));
        tree.discard_unregistered_events();

        assert!(tree.take_owner_events().is_empty());
        assert!(tree.target_has_focus(7));
    }

    #[test]
    fn snapshot_clone_preserves_untranslated_focus_callbacks_without_aliasing() {
        let mut tree = RuntimeFocusTree {
            owner_identity: 11,
            ..RuntimeFocusTree::default()
        };
        {
            let mut domain = tree.domain.borrow_mut();
            let target = domain.manager.create_node(FocusNode::new());
            domain.manager.add_child(None, target);
            domain.targets.insert((11, 7), target);
            domain.focus_data_by_target.insert((11, 7), 8);
        }
        assert!(tree.set_focus_target(7));

        let mut snapshot = tree.clone();
        assert_eq!(
            snapshot.take_owner_events(),
            [(7, 8, FocusEventKind::Focused)]
        );
        assert_eq!(
            tree.take_owner_events(),
            [(7, 8, FocusEventKind::Focused)],
            "draining the snapshot may not consume the source callback"
        );
    }
}
