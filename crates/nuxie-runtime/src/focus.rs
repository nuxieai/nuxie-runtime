use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

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
#[derive(Debug)]
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
            has_focusable: true,
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
#[derive(Debug, Default)]
pub struct FocusManager {
    nodes: BTreeMap<FocusNodeId, FocusNode>,
    roots: Vec<FocusNodeId>,
    primary_focus: Option<FocusNodeId>,
    pending_events: Vec<FocusEvent>,
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
        self.nodes.get_mut(&node_id)
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
        self.roots
            .iter()
            .copied()
            .any(|root| self.subtree_has_focusable_content(root))
    }

    pub fn take_events(&mut self) -> Vec<FocusEvent> {
        std::mem::take(&mut self.pending_events)
    }

    fn unlink(&mut self, node_id: FocusNodeId) {
        let parent = self.nodes.get(&node_id).and_then(|node| node.parent);
        if let Some(parent) = parent {
            if let Some(parent) = self.nodes.get_mut(&parent) {
                parent.children.retain(|child| *child != node_id);
            }
        } else {
            self.roots.retain(|root| *root != node_id);
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
            return false;
        };
        if Some(next) == current {
            return false;
        }
        self.set_focus(next)
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
        authored.set_can_focus(false);
        authored.set_can_traverse(false);
        authored.set_eligible(false);
        let authored = manager.create_node(authored);
        manager.add_child(Some(scope), authored);

        assert!(manager.has_focusable_content());
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
}
