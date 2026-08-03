// Pinned C++ correspondence (d788e8ec):
// src/semantic/semantic_manager.cpp:1-1109 and
// include/rive/semantic/semantic_manager.hpp:1-100.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

use crate::semantic_node::{
    SemanticBounds, SemanticDirt, SemanticNode, SemanticNodeHandle, is_interactive_role,
};
use crate::semantic_snapshot::{
    SemanticsBoundsUpdate, SemanticsChildrenUpdate, SemanticsDiff, SemanticsDiffNode,
};

type ChildrenByParent = BTreeMap<i32, Vec<u32>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticDrainError {
    BoundaryResolutionRequired,
}

impl std::fmt::Display for SemanticDrainError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BoundaryResolutionRequired => {
                formatter.write_str("semantic boundary bounds must be reconciled before draining")
            }
        }
    }
}

impl std::error::Error for SemanticDrainError {}

fn next_manager_identity() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, AtomicOrdering::Relaxed)
}

fn normalize_label(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut last_was_space = true;
    for character in input.chars() {
        if character <= ' ' {
            if !last_was_space && !result.is_empty() {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(character);
            last_was_space = false;
        }
    }
    if result.ends_with(' ') {
        result.pop();
    }
    result
}

#[derive(Default)]
struct LabelDerivationScratch {
    text_label: String,
    image_label: String,
    absorbed: BTreeSet<u32>,
}

fn collect_labels_from_subtree(node: &SemanticNodeHandle, scratch: &mut LabelDerivationScratch) {
    let (id, role, label, children) = {
        let node = node.borrow();
        (
            node.id(),
            node.role(),
            node.label().to_owned(),
            node.children().to_vec(),
        )
    };
    if is_interactive_role(role) {
        return;
    }

    scratch.absorbed.insert(id);
    match role {
        7 if !label.is_empty() => {
            if !scratch.text_label.is_empty() {
                scratch.text_label.push(' ');
            }
            scratch.text_label.push_str(&label);
        }
        8 if !label.is_empty() && scratch.image_label.is_empty() => {
            scratch.image_label = label;
        }
        _ => {}
    }

    for child in children {
        collect_labels_from_subtree(&child, scratch);
    }
}

fn derive_label_visit(
    node: &SemanticNodeHandle,
    derived_labels: &mut BTreeMap<u32, String>,
    excluded_ids: &mut BTreeSet<u32>,
    scratch: &mut LabelDerivationScratch,
) {
    let (id, role, label, children) = {
        let node = node.borrow();
        (
            node.id(),
            node.role(),
            node.label().to_owned(),
            node.children().to_vec(),
        )
    };
    if is_interactive_role(role) && label.is_empty() {
        scratch.text_label.clear();
        scratch.image_label.clear();
        scratch.absorbed.clear();
        for child in &children {
            collect_labels_from_subtree(child, scratch);
        }
        let mut derived = normalize_label(&scratch.text_label);
        if derived.is_empty() {
            derived = normalize_label(&scratch.image_label);
        }
        if !derived.is_empty() {
            derived_labels.insert(id, derived);
            excluded_ids.extend(scratch.absorbed.iter().copied());
        }
    }

    for child in children {
        if !excluded_ids.contains(&child.borrow().id()) {
            derive_label_visit(&child, derived_labels, excluded_ids, scratch);
        }
    }
}

fn derive_labels_for_interactive_nodes(
    roots: &[SemanticNodeHandle],
    derived_labels: &mut BTreeMap<u32, String>,
    excluded_ids: &mut BTreeSet<u32>,
) {
    let mut scratch = LabelDerivationScratch::default();
    for root in roots {
        derive_label_visit(root, derived_labels, excluded_ids, &mut scratch);
    }
}

fn flatten_semantic_node(
    node: &SemanticNodeHandle,
    parent_id: i32,
    sibling_counter: &mut u32,
    out: &mut Vec<SemanticsDiffNode>,
    excluded_ids: &BTreeSet<u32>,
    derived_labels: &BTreeMap<u32, String>,
) {
    let (id, is_boundary, children) = {
        let node = node.borrow();
        (node.id(), node.is_boundary_node(), node.children().to_vec())
    };
    if excluded_ids.contains(&id) || is_boundary {
        for child in children {
            flatten_semantic_node(
                &child,
                parent_id,
                sibling_counter,
                out,
                excluded_ids,
                derived_labels,
            );
        }
        return;
    }

    let flat = {
        let node = node.borrow();
        let bounds = node.bounds();
        SemanticsDiffNode {
            id,
            role: node.role(),
            label: derived_labels
                .get(&id)
                .cloned()
                .unwrap_or_else(|| node.label().to_owned()),
            value: node.value().to_owned(),
            hint: node.hint().to_owned(),
            state_flags: node.state_flags(),
            trait_flags: node.trait_flags(),
            heading_level: node.heading_level(),
            min_x: bounds.min_x,
            min_y: bounds.min_y,
            max_x: bounds.max_x,
            max_y: bounds.max_y,
            parent_id,
            sibling_index: *sibling_counter,
        }
    };
    *sibling_counter = sibling_counter.wrapping_add(1);
    let flat_id = flat.id;
    out.push(flat);

    let mut child_sibling_counter = 0;
    for child in children {
        flatten_semantic_node(
            &child,
            flat_id as i32,
            &mut child_sibling_counter,
            out,
            excluded_ids,
            derived_labels,
        );
    }
}

fn flatten_from_semantic_nodes(
    roots: &[SemanticNodeHandle],
    reserve_hint: usize,
    excluded_ids: &BTreeSet<u32>,
    derived_labels: &BTreeMap<u32, String>,
) -> Vec<SemanticsDiffNode> {
    let mut out = Vec::with_capacity(reserve_hint);
    let mut root_sibling_counter = 0;
    for root in roots {
        flatten_semantic_node(
            root,
            -1,
            &mut root_sibling_counter,
            &mut out,
            excluded_ids,
            derived_labels,
        );
    }
    out
}

fn build_children_by_parent(nodes: &[SemanticsDiffNode]) -> ChildrenByParent {
    let mut grouped = BTreeMap::<i32, Vec<(u32, u32)>>::new();
    for node in nodes {
        grouped
            .entry(node.parent_id)
            .or_default()
            .push((node.sibling_index, node.id));
    }
    grouped
        .into_iter()
        .map(|(parent, mut children)| {
            children.sort_by_key(|(sibling, _)| *sibling);
            (
                parent,
                children.into_iter().map(|(_, id)| id).collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn build_diff_from_flats(
    current: &[SemanticsDiffNode],
    previous: &[SemanticsDiffNode],
    tree_version: u64,
) -> SemanticsDiff {
    let mut diff = SemanticsDiff {
        frame_number: crate::artboard_draw_frame_id(),
        tree_version,
        ..SemanticsDiff::default()
    };
    let roots = current
        .iter()
        .filter(|node| node.parent_id == -1)
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if roots.len() == 1 {
        diff.root_id = roots[0];
    }

    if previous.is_empty() {
        let current_children = build_children_by_parent(current);
        let mut seen = BTreeSet::new();
        for node in current {
            if !seen.insert(node.parent_id) {
                continue;
            }
            if let Some(children) = current_children.get(&node.parent_id) {
                diff.children_updated.push(SemanticsChildrenUpdate {
                    parent_id: node.parent_id,
                    child_ids: children.clone(),
                });
            }
        }
        diff.added.extend_from_slice(current);
        return diff;
    }

    let current_by_id = current
        .iter()
        .map(|node| (node.id, node))
        .collect::<BTreeMap<_, _>>();
    let previous_by_id = previous
        .iter()
        .map(|node| (node.id, node))
        .collect::<BTreeMap<_, _>>();
    let current_children = build_children_by_parent(current);
    let previous_children = build_children_by_parent(previous);

    for node in previous {
        if !current_by_id.contains_key(&node.id) {
            diff.removed.push(node.id);
        }
    }

    for node in current {
        let Some(previous_node) = previous_by_id.get(&node.id) else {
            diff.added.push(node.clone());
            continue;
        };
        if previous_node.parent_id != node.parent_id
            || previous_node.sibling_index != node.sibling_index
        {
            diff.moved.push(node.clone());
        }
        // This intentionally mirrors the pinned full-diff predicate. Value,
        // hint, and heading are copied into payloads but are only compared by
        // the incremental content patch in d788e8ec.
        if previous_node.role != node.role
            || previous_node.label != node.label
            || previous_node.state_flags != node.state_flags
            || previous_node.trait_flags != node.trait_flags
        {
            diff.updated_semantic.push(node.clone());
        }
        if previous_node.bounds() != node.bounds() {
            diff.updated_geometry.push(SemanticsBoundsUpdate {
                id: node.id,
                min_x: node.min_x,
                min_y: node.min_y,
                max_x: node.max_x,
                max_y: node.max_y,
            });
        }
    }

    let mut ordered_parents = Vec::new();
    let mut seen = BTreeSet::new();
    for node in current.iter().chain(previous) {
        if seen.insert(node.parent_id) {
            ordered_parents.push(node.parent_id);
        }
    }
    for parent_id in ordered_parents {
        let current_children = current_children
            .get(&parent_id)
            .map_or(&[][..], Vec::as_slice);
        let previous_children = previous_children
            .get(&parent_id)
            .map_or(&[][..], Vec::as_slice);
        if current_children != previous_children {
            diff.children_updated.push(SemanticsChildrenUpdate {
                parent_id,
                child_ids: current_children.to_vec(),
            });
        }
    }
    diff
}

fn compare_visual_position(a: &SemanticNodeHandle, b: &SemanticNodeHandle) -> Ordering {
    let a = a.borrow().bounds();
    let b = b.borrow().bounds();
    let a_empty = a.is_empty_or_nan();
    let b_empty = b.is_empty_or_nan();
    if a_empty != b_empty {
        return if b_empty {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    if a_empty {
        return Ordering::Equal;
    }
    if a.min_y < b.min_y {
        Ordering::Less
    } else if a.min_y > b.min_y {
        Ordering::Greater
    } else if a.min_x < b.min_x {
        Ordering::Less
    } else if a.min_x > b.min_x {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

fn children_in_visual_order(children: &[SemanticNodeHandle]) -> bool {
    let mut previous = None;
    for child in children {
        let bounds = child.borrow().bounds();
        if bounds.is_empty_or_nan() {
            continue;
        }
        if let Some(previous_bounds) = previous {
            let previous_bounds: SemanticBounds = previous_bounds;
            if bounds.min_y < previous_bounds.min_y
                || (bounds.min_y == previous_bounds.min_y && bounds.min_x < previous_bounds.min_x)
            {
                return false;
            }
        }
        previous = Some(bounds);
    }
    true
}

fn sort_children_by_visual_position(nodes: &mut Vec<SemanticNodeHandle>) {
    if nodes.len() > 1 {
        nodes.sort_by(compare_visual_position);
    }
    for node in nodes {
        let children = node.borrow().children().to_vec();
        let mut sorted = children;
        sort_children_by_visual_position(&mut sorted);
        *node.borrow_mut().children_mut() = sorted;
    }
}

/// Owns one semantic tree and produces incremental platform deltas.
#[derive(Debug)]
pub struct SemanticManager {
    identity: u64,
    dirt: SemanticDirt,
    last_diff: SemanticsDiff,
    last_flat_snapshot: Vec<SemanticsDiffNode>,
    version: u64,
    next_local_id: u32,
    nodes_by_id: BTreeMap<u32, SemanticNodeHandle>,
    roots: Vec<SemanticNodeHandle>,
    dirty_content_nodes: BTreeSet<u32>,
    dirty_bounds_nodes: BTreeSet<u32>,
    dirty_boundary_ids: BTreeSet<u32>,
    derived_labels: BTreeMap<u32, String>,
    excluded_ids: BTreeSet<u32>,
}

impl Default for SemanticManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticManager {
    pub fn new() -> Self {
        Self {
            identity: next_manager_identity(),
            dirt: SemanticDirt::ALL,
            last_diff: SemanticsDiff::default(),
            last_flat_snapshot: Vec::new(),
            version: 0,
            next_local_id: 1,
            nodes_by_id: BTreeMap::new(),
            roots: Vec::new(),
            dirty_content_nodes: BTreeSet::new(),
            dirty_bounds_nodes: BTreeSet::new(),
            dirty_boundary_ids: BTreeSet::new(),
            derived_labels: BTreeMap::new(),
            excluded_ids: BTreeSet::new(),
        }
    }

    pub fn identity(&self) -> u64 {
        self.identity
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn is_dirty(&self) -> bool {
        self.dirt != SemanticDirt::NONE
    }

    pub fn node_by_id(&self, id: u32) -> Option<SemanticNodeHandle> {
        self.nodes_by_id.get(&id).cloned()
    }

    /// Resolve a node's retained SemanticData identity and ask its owner to
    /// focus it. Boundary and unknown nodes are silent false results.
    pub fn request_focus<F>(&self, id: u32, mut request: F) -> bool
    where
        F: FnMut(usize) -> bool,
    {
        self.nodes_by_id
            .get(&id)
            .and_then(|node| node.borrow().semantic_data_local_id())
            .is_some_and(&mut request)
    }

    pub fn roots(&self) -> &[SemanticNodeHandle] {
        &self.roots
    }

    pub fn mark_dirty(&mut self, dirt: SemanticDirt) {
        self.dirt |= dirt;
    }

    pub fn mark_node_dirty(&mut self, id: u32, dirt: SemanticDirt) {
        self.mark_dirty(dirt);
        if id == 0 {
            return;
        }
        if dirt.contains(SemanticDirt::CONTENT) {
            self.dirty_content_nodes.insert(id);
        }
        if dirt.contains(SemanticDirt::BOUNDS) {
            self.dirty_bounds_nodes.insert(id);
        }
    }

    pub fn mark_boundary_dirty(&mut self, id: u32) {
        self.dirty_boundary_ids.insert(id);
        self.mark_dirty(SemanticDirt::BOUNDS);
    }

    fn ensure_node_id(&mut self, node: &SemanticNodeHandle) {
        if node.borrow().id() == 0 {
            while self.nodes_by_id.contains_key(&self.next_local_id) {
                self.next_local_id = self.next_local_id.wrapping_add(1);
            }
            node.borrow_mut().set_id(self.next_local_id);
            self.next_local_id = self.next_local_id.wrapping_add(1);
            return;
        }
        let id = node.borrow().id();
        if id >= self.next_local_id {
            self.next_local_id = id.wrapping_add(1);
        }
    }

    pub fn add_child(
        &mut self,
        parent: Option<&SemanticNodeHandle>,
        child: SemanticNodeHandle,
    ) -> u32 {
        let child_id = child.borrow().id();
        if child_id != 0
            && self
                .nodes_by_id
                .get(&child_id)
                .is_some_and(|existing| !existing.ptr_eq(&child))
        {
            child.borrow_mut().set_id(0);
        }
        self.ensure_node_id(&child);
        let child_id = child.borrow().id();
        self.nodes_by_id
            .entry(child_id)
            .or_insert_with(|| child.clone());
        child.borrow_mut().set_manager_identity(Some(self.identity));

        if let Some(parent) = parent {
            let parent_id = parent.borrow().id();
            self.nodes_by_id
                .entry(parent_id)
                .or_insert_with(|| parent.clone());
            child.borrow_mut().set_parent(Some(parent.downgrade()));
            parent.borrow_mut().children_mut().push(child);
        } else {
            child.borrow_mut().set_parent(None);
            self.roots.push(child);
        }
        self.mark_dirty(SemanticDirt::STRUCTURE);
        child_id
    }

    pub fn remove_child(&mut self, node: &SemanticNodeHandle) {
        let id = node.borrow().id();
        let parent = node
            .borrow()
            .parent_id()
            .and_then(|parent_id| self.nodes_by_id.get(&parent_id).cloned());
        if let Some(parent) = parent {
            parent
                .borrow_mut()
                .children_mut()
                .retain(|child| !child.ptr_eq(node));
            node.borrow_mut().set_parent(None);
        } else {
            self.roots.retain(|root| !root.ptr_eq(node));
        }
        node.borrow_mut().set_manager_identity(None);
        self.nodes_by_id.remove(&id);
        self.mark_dirty(SemanticDirt::STRUCTURE);
    }

    /// Re-read live bounds for one dirty boundary subtree.
    ///
    /// The callback is the Rust owner-mediated form of C++ node back-pointers:
    /// it receives each retained node and returns its current root-space bounds.
    pub fn reconcile_boundary_bounds<F>(&mut self, boundary_id: u32, mut resolve: F)
    where
        F: FnMut(&SemanticNode) -> Option<SemanticBounds>,
    {
        let Some(root) = self.nodes_by_id.get(&boundary_id).cloned() else {
            self.dirty_boundary_ids.remove(&boundary_id);
            return;
        };
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            let children = node.borrow().children().to_vec();
            let resolved_bounds = {
                let node = node.borrow();
                resolve(&node)
            };
            if let Some(bounds) = resolved_bounds {
                let changed = node.borrow().bounds() != bounds;
                if changed {
                    let id = node.borrow().id();
                    node.borrow_mut().set_bounds(bounds);
                    self.dirty_bounds_nodes.insert(id);
                }
            }
            stack.extend(children.into_iter().rev());
        }
        self.dirty_boundary_ids.remove(&boundary_id);
        if !self.dirty_bounds_nodes.is_empty() {
            self.mark_dirty(SemanticDirt::BOUNDS);
        }
    }

    /// Reconcile every pending boundary from its live Artboard owner before
    /// producing the diff, matching the pinned refresh-time ordering.
    pub fn drain_diff_with_boundary_resolver<F>(&mut self, mut resolve: F) -> SemanticsDiff
    where
        F: FnMut(&SemanticNode) -> Option<SemanticBounds>,
    {
        let boundary_ids = self.dirty_boundary_ids.iter().copied().collect::<Vec<_>>();
        for boundary_id in boundary_ids {
            self.reconcile_boundary_bounds(boundary_id, &mut resolve);
        }
        self.drain_reconciled_diff()
    }

    fn patch_bounds_node(&self, entry: &mut SemanticsDiffNode, diff: &mut SemanticsDiff) {
        let Some(node) = self.nodes_by_id.get(&entry.id) else {
            return;
        };
        let bounds = node.borrow().bounds();
        if entry.bounds() != bounds {
            entry.set_bounds(bounds);
            diff.updated_geometry.push(SemanticsBoundsUpdate {
                id: entry.id,
                min_x: entry.min_x,
                min_y: entry.min_y,
                max_x: entry.max_x,
                max_y: entry.max_y,
            });
        }
    }

    fn patch_content_node(&self, entry: &mut SemanticsDiffNode, diff: &mut SemanticsDiff) {
        let Some(node) = self.nodes_by_id.get(&entry.id) else {
            return;
        };
        let node = node.borrow();
        let effective_label = self
            .derived_labels
            .get(&entry.id)
            .map_or(node.label(), String::as_str);
        if entry.role != node.role()
            || entry.label != effective_label
            || entry.value != node.value()
            || entry.hint != node.hint()
            || entry.state_flags != node.state_flags()
            || entry.trait_flags != node.trait_flags()
            || entry.heading_level != node.heading_level()
        {
            entry.role = node.role();
            entry.label = effective_label.to_owned();
            entry.value = node.value().to_owned();
            entry.hint = node.hint().to_owned();
            entry.state_flags = node.state_flags();
            entry.trait_flags = node.trait_flags();
            entry.heading_level = node.heading_level();
            diff.updated_semantic.push(entry.clone());
        }
    }

    fn refresh(&mut self) {
        let structure_dirty = self.dirt.contains(SemanticDirt::STRUCTURE);
        let mut bounds_dirty = self.dirt.contains(SemanticDirt::BOUNDS);
        let content_dirty = self.dirt.contains(SemanticDirt::CONTENT);

        // Boundary resolution is owner-mediated through reconcile_boundary_bounds.
        // Unresolved ids remain dirty so a later side-channel drain with live
        // artboard access can reconcile them without losing the request.
        if !self.dirty_boundary_ids.is_empty() && !self.dirty_bounds_nodes.is_empty() {
            bounds_dirty = true;
        }
        if !structure_dirty && !bounds_dirty && !content_dirty {
            return;
        }

        let mut needs_reorder = false;
        if bounds_dirty && !structure_dirty && !self.last_flat_snapshot.is_empty() {
            let mut parents = BTreeSet::new();
            let mut root_moved = false;
            for id in &self.dirty_bounds_nodes {
                let Some(node) = self.nodes_by_id.get(id) else {
                    continue;
                };
                if let Some(parent_id) = node.borrow().parent_id() {
                    parents.insert(parent_id);
                } else {
                    root_moved = true;
                }
            }
            for parent_id in parents {
                let Some(parent) = self.nodes_by_id.get(&parent_id) else {
                    continue;
                };
                let parent = parent.borrow();
                if parent.children().len() > 1 && !children_in_visual_order(parent.children()) {
                    needs_reorder = true;
                    break;
                }
            }
            if !needs_reorder
                && root_moved
                && self.roots.len() > 1
                && !children_in_visual_order(&self.roots)
            {
                needs_reorder = true;
            }
        }

        let needs_rederivation = content_dirty
            && !self.excluded_ids.is_empty()
            && self
                .dirty_content_nodes
                .iter()
                .any(|id| self.excluded_ids.contains(id));

        if structure_dirty
            || needs_rederivation
            || needs_reorder
            || self.last_flat_snapshot.is_empty()
        {
            if structure_dirty || needs_reorder || self.last_flat_snapshot.is_empty() {
                sort_children_by_visual_position(&mut self.roots);
            }
            self.derived_labels.clear();
            self.excluded_ids.clear();
            derive_labels_for_interactive_nodes(
                &self.roots,
                &mut self.derived_labels,
                &mut self.excluded_ids,
            );
            let current = flatten_from_semantic_nodes(
                &self.roots,
                self.nodes_by_id.len(),
                &self.excluded_ids,
                &self.derived_labels,
            );
            let next = build_diff_from_flats(
                &current,
                &self.last_flat_snapshot,
                self.version.wrapping_add(1),
            );
            if !next.is_empty() {
                self.version = self.version.wrapping_add(1);
                self.last_diff = next;
                self.last_flat_snapshot = current;
            }
        } else {
            let mut next = SemanticsDiff {
                frame_number: crate::artboard_draw_frame_id(),
                ..SemanticsDiff::default()
            };
            let mut snapshot = std::mem::take(&mut self.last_flat_snapshot);
            for entry in &mut snapshot {
                if bounds_dirty && self.dirty_bounds_nodes.contains(&entry.id) {
                    self.patch_bounds_node(entry, &mut next);
                }
                if content_dirty && self.dirty_content_nodes.contains(&entry.id) {
                    self.patch_content_node(entry, &mut next);
                }
            }
            self.last_flat_snapshot = snapshot;
            if self.roots.len() == 1 {
                next.root_id = self.roots[0].borrow().id();
            }
            if !next.is_empty() {
                self.version = self.version.wrapping_add(1);
                next.tree_version = self.version;
                self.last_diff = next;
            }
        }

        self.dirt = SemanticDirt::NONE;
        self.dirty_content_nodes.clear();
        self.dirty_bounds_nodes.clear();
    }

    pub fn drain_diff(&mut self) -> Result<SemanticsDiff, SemanticDrainError> {
        if !self.dirty_boundary_ids.is_empty() {
            return Err(SemanticDrainError::BoundaryResolutionRequired);
        }
        Ok(self.drain_reconciled_diff())
    }

    fn drain_reconciled_diff(&mut self) -> SemanticsDiff {
        self.refresh();
        std::mem::take(&mut self.last_diff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SemanticRole;

    fn node(
        id: u32,
        role: SemanticRole,
        label: &str,
        bounds: SemanticBounds,
    ) -> SemanticNodeHandle {
        let node = SemanticNodeHandle::new(id);
        {
            let mut node = node.borrow_mut();
            node.set_role(role as u32);
            node.set_label(label);
            node.set_bounds(bounds);
        }
        node
    }

    fn drain(manager: &mut SemanticManager) -> SemanticsDiff {
        manager
            .drain_diff()
            .expect("focused manager tree has no unresolved boundary dirt")
    }

    #[test]
    fn button_derives_label_from_child_text_and_absorbs_the_child() {
        let mut manager = SemanticManager::new();
        let button = node(
            1,
            SemanticRole::Button,
            "",
            SemanticBounds::new(0.0, 0.0, 10.0, 10.0),
        );
        let text = node(
            2,
            SemanticRole::Text,
            "Press me",
            SemanticBounds::new(1.0, 1.0, 9.0, 9.0),
        );
        manager.add_child(None, button.clone());
        manager.add_child(Some(&button), text);
        let diff = drain(&mut manager);
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].id, 1);
        assert_eq!(diff.added[0].label, "Press me");
    }

    #[test]
    fn explicit_label_wins_and_children_remain_exposed() {
        let mut manager = SemanticManager::new();
        let button = node(
            1,
            SemanticRole::Button,
            "Explicit",
            SemanticBounds::new(0.0, 0.0, 10.0, 10.0),
        );
        let text = node(
            2,
            SemanticRole::Text,
            "Child",
            SemanticBounds::new(1.0, 1.0, 9.0, 9.0),
        );
        manager.add_child(None, button.clone());
        manager.add_child(Some(&button), text);
        let diff = drain(&mut manager);
        assert_eq!(
            diff.added.iter().map(|node| node.id).collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(diff.added[0].label, "Explicit");
    }

    #[test]
    fn text_priority_and_whitespace_normalization_match_pinned_derivation() {
        let mut manager = SemanticManager::new();
        let button = node(1, SemanticRole::Button, "", SemanticBounds::default());
        let image = node(
            2,
            SemanticRole::Image,
            "Fallback",
            SemanticBounds::default(),
        );
        let first = node(
            3,
            SemanticRole::Text,
            "  first\n",
            SemanticBounds::default(),
        );
        let second = node(
            4,
            SemanticRole::Text,
            "\tsecond  ",
            SemanticBounds::default(),
        );
        manager.add_child(None, button.clone());
        manager.add_child(Some(&button), image);
        manager.add_child(Some(&button), first);
        manager.add_child(Some(&button), second);
        let diff = drain(&mut manager);
        assert_eq!(diff.added[0].label, "first second");
    }

    #[test]
    fn label_normalization_preserves_utf8_bytes_as_text() {
        let mut manager = SemanticManager::new();
        let button = node(1, SemanticRole::Button, "", SemanticBounds::default());
        let text = node(
            2,
            SemanticRole::Text,
            "  Zażółć\n日本語  ",
            SemanticBounds::default(),
        );
        manager.add_child(None, button.clone());
        manager.add_child(Some(&button), text);
        assert_eq!(drain(&mut manager).added[0].label, "Zażółć 日本語");
    }

    #[test]
    fn image_is_the_fallback_when_no_text_label_exists() {
        let mut manager = SemanticManager::new();
        let button = node(1, SemanticRole::Button, "", SemanticBounds::default());
        let image = node(2, SemanticRole::Image, "Artwork", SemanticBounds::default());
        manager.add_child(None, button.clone());
        manager.add_child(Some(&button), image);
        let diff = drain(&mut manager);
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].label, "Artwork");
    }

    #[test]
    fn group_and_text_field_do_not_derive_or_absorb_labels() {
        for role in [SemanticRole::Group, SemanticRole::TextField] {
            let mut manager = SemanticManager::new();
            let parent = node(1, role, "", SemanticBounds::default());
            let text = node(2, SemanticRole::Text, "Child", SemanticBounds::default());
            manager.add_child(None, parent.clone());
            manager.add_child(Some(&parent), text);
            let diff = drain(&mut manager);
            assert_eq!(diff.added.len(), 2);
            assert!(diff.added[0].label.is_empty());
        }
    }

    #[test]
    fn every_pinned_interactive_role_derives_a_child_label() {
        for role in [
            SemanticRole::Button,
            SemanticRole::Link,
            SemanticRole::Checkbox,
            SemanticRole::SwitchControl,
            SemanticRole::Slider,
            SemanticRole::ListItem,
            SemanticRole::Tab,
            SemanticRole::RadioButton,
        ] {
            let mut manager = SemanticManager::new();
            let parent = node(1, role, "", SemanticBounds::default());
            let text = node(2, SemanticRole::Text, "Child", SemanticBounds::default());
            manager.add_child(None, parent.clone());
            manager.add_child(Some(&parent), text);
            let diff = drain(&mut manager);
            assert_eq!(diff.added.len(), 1, "role {role:?}");
            assert_eq!(diff.added[0].label, "Child", "role {role:?}");
        }
    }

    #[test]
    fn nested_interactive_child_is_not_absorbed() {
        let mut manager = SemanticManager::new();
        let outer = node(1, SemanticRole::Button, "", SemanticBounds::default());
        let inner = node(2, SemanticRole::Link, "Inner", SemanticBounds::default());
        manager.add_child(None, outer.clone());
        manager.add_child(Some(&outer), inner);
        let diff = drain(&mut manager);
        assert_eq!(
            diff.added.iter().map(|node| node.id).collect::<Vec<_>>(),
            [1, 2]
        );
        assert!(diff.added[0].label.is_empty());
    }

    #[test]
    fn children_sort_by_y_then_x_and_empty_bounds_stay_last() {
        let mut manager = SemanticManager::new();
        let root = node(1, SemanticRole::Group, "", SemanticBounds::default());
        let first_empty = node(
            2,
            SemanticRole::Text,
            "first empty",
            SemanticBounds::for_expansion(),
        );
        let second_empty = node(
            5,
            SemanticRole::Text,
            "second empty",
            SemanticBounds::for_expansion(),
        );
        let right = node(
            3,
            SemanticRole::Text,
            "right",
            SemanticBounds::new(20.0, 5.0, 30.0, 6.0),
        );
        let left = node(
            4,
            SemanticRole::Text,
            "left",
            SemanticBounds::new(10.0, 5.0, 15.0, 6.0),
        );
        manager.add_child(None, root.clone());
        manager.add_child(Some(&root), first_empty);
        manager.add_child(Some(&root), second_empty);
        manager.add_child(Some(&root), right);
        manager.add_child(Some(&root), left);
        let diff = drain(&mut manager);
        assert_eq!(
            diff.added.iter().map(|node| node.id).collect::<Vec<_>>(),
            [1, 4, 3, 2, 5]
        );
    }

    #[test]
    fn bounds_crossing_reorders_children_and_emits_authoritative_order() {
        let mut manager = SemanticManager::new();
        let root = node(1, SemanticRole::Group, "", SemanticBounds::default());
        let first = node(
            2,
            SemanticRole::Text,
            "first",
            SemanticBounds::new(0.0, 0.0, 1.0, 1.0),
        );
        let second = node(
            3,
            SemanticRole::Text,
            "second",
            SemanticBounds::new(0.0, 10.0, 1.0, 11.0),
        );
        manager.add_child(None, root.clone());
        manager.add_child(Some(&root), first.clone());
        manager.add_child(Some(&root), second.clone());
        drain(&mut manager);
        second
            .borrow_mut()
            .set_bounds(SemanticBounds::new(0.0, -1.0, 1.0, 0.0));
        manager.mark_node_dirty(3, SemanticDirt::BOUNDS);
        let diff = drain(&mut manager);
        assert_eq!(
            diff.moved.iter().map(|node| node.id).collect::<Vec<_>>(),
            [3, 2]
        );
        assert_eq!(
            diff.children_updated,
            [SemanticsChildrenUpdate {
                parent_id: 1,
                child_ids: vec![3, 2],
            }]
        );
    }

    #[test]
    fn bounds_change_without_crossing_stays_geometry_only() {
        let mut manager = SemanticManager::new();
        let root = node(1, SemanticRole::Group, "", SemanticBounds::default());
        let child = node(2, SemanticRole::Text, "child", SemanticBounds::default());
        manager.add_child(None, root.clone());
        manager.add_child(Some(&root), child.clone());
        drain(&mut manager);
        child
            .borrow_mut()
            .set_bounds(SemanticBounds::new(1.0, 2.0, 3.0, 4.0));
        manager.mark_node_dirty(2, SemanticDirt::BOUNDS);
        let diff = drain(&mut manager);
        assert_eq!(diff.updated_geometry.len(), 1);
        assert!(diff.added.is_empty());
        assert!(diff.moved.is_empty());
        assert!(diff.children_updated.is_empty());
    }

    #[test]
    fn boundary_nodes_flatten_away_and_reparent_children() {
        let mut manager = SemanticManager::new();
        let boundary = node(1, SemanticRole::None, "", SemanticBounds::default());
        boundary.borrow_mut().set_boundary_node(true);
        let child = node(2, SemanticRole::Text, "child", SemanticBounds::default());
        manager.add_child(None, boundary.clone());
        manager.add_child(Some(&boundary), child);
        let diff = drain(&mut manager);
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].id, 2);
        assert_eq!(diff.added[0].parent_id, -1);
        assert_eq!(diff.root_id, 2);
    }

    #[test]
    fn boundary_drain_reconciles_live_subtree_bounds_and_clears_unknown_ids() {
        let mut manager = SemanticManager::new();
        let boundary = node(1, SemanticRole::None, "", SemanticBounds::default());
        boundary.borrow_mut().set_boundary_node(true);
        let child = node(2, SemanticRole::Text, "child", SemanticBounds::default());
        manager.add_child(None, boundary.clone());
        manager.add_child(Some(&boundary), child);
        drain(&mut manager);

        manager.mark_boundary_dirty(1);
        let diff = manager.drain_diff_with_boundary_resolver(|node| {
            (node.id() == 2).then_some(SemanticBounds::new(1.0, 2.0, 3.0, 4.0))
        });
        assert_eq!(diff.updated_geometry.len(), 1);
        assert_eq!(diff.updated_geometry[0].id, 2);
        assert!(manager.dirty_boundary_ids.is_empty());

        manager.mark_boundary_dirty(99);
        assert!(
            manager
                .drain_diff_with_boundary_resolver(|_| None)
                .is_empty()
        );
        assert!(manager.dirty_boundary_ids.is_empty());
    }

    #[test]
    fn drain_without_resolver_returns_structured_error_for_pending_boundaries() {
        let mut manager = SemanticManager::new();
        let root = node(1, SemanticRole::Group, "", SemanticBounds::default());
        manager.add_child(None, root);
        manager.mark_boundary_dirty(1);
        assert_eq!(
            manager.drain_diff(),
            Err(SemanticDrainError::BoundaryResolutionRequired)
        );
    }

    #[test]
    fn first_diff_is_preorder_and_contains_authoritative_child_lists() {
        let mut manager = SemanticManager::new();
        let root = node(1, SemanticRole::Group, "", SemanticBounds::default());
        let child = node(2, SemanticRole::Text, "child", SemanticBounds::default());
        manager.add_child(None, root.clone());
        manager.add_child(Some(&root), child);
        let diff = drain(&mut manager);
        assert_eq!(diff.root_id, 1);
        assert_eq!(diff.tree_version, 1);
        assert_eq!(
            diff.added.iter().map(|node| node.id).collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            diff.children_updated,
            [
                SemanticsChildrenUpdate {
                    parent_id: -1,
                    child_ids: vec![1]
                },
                SemanticsChildrenUpdate {
                    parent_id: 1,
                    child_ids: vec![2]
                },
            ]
        );
    }

    #[test]
    fn manager_local_ids_are_independent_and_explicit_ids_advance_the_watermark() {
        let mut first = SemanticManager::new();
        let mut second = SemanticManager::new();
        let first_auto = SemanticNodeHandle::new(0);
        let second_auto = SemanticNodeHandle::new(0);
        assert_eq!(first.add_child(None, first_auto), 1);
        assert_eq!(second.add_child(None, second_auto), 1);
        assert_eq!(first.add_child(None, SemanticNodeHandle::new(10)), 10);
        assert_eq!(first.add_child(None, SemanticNodeHandle::new(0)), 11);
    }

    #[test]
    fn resident_id_collision_reassigns_only_the_new_node() {
        let mut manager = SemanticManager::new();
        let first = SemanticNodeHandle::new(7);
        let second = SemanticNodeHandle::new(7);
        assert_eq!(manager.add_child(None, first.clone()), 7);
        assert_eq!(manager.add_child(None, second.clone()), 8);
        assert!(
            manager
                .node_by_id(7)
                .is_some_and(|node| node.ptr_eq(&first))
        );
        assert!(
            manager
                .node_by_id(8)
                .is_some_and(|node| node.ptr_eq(&second))
        );
    }

    #[test]
    fn incremental_content_and_bounds_updates_are_separate_and_tree_ordered() {
        let mut manager = SemanticManager::new();
        let root = node(1, SemanticRole::Group, "root", SemanticBounds::default());
        let child = node(2, SemanticRole::Text, "before", SemanticBounds::default());
        manager.add_child(None, root.clone());
        manager.add_child(Some(&root), child.clone());
        drain(&mut manager);

        child.borrow_mut().set_label("after");
        child
            .borrow_mut()
            .set_bounds(SemanticBounds::new(1.0, 2.0, 3.0, 4.0));
        manager.mark_node_dirty(2, SemanticDirt::CONTENT | SemanticDirt::BOUNDS);
        let diff = drain(&mut manager);
        assert_eq!(diff.updated_semantic.len(), 1);
        assert_eq!(diff.updated_semantic[0].label, "after");
        assert_eq!(diff.updated_geometry.len(), 1);
        assert_eq!(
            diff.updated_geometry[0].bounds(),
            SemanticBounds::new(1.0, 2.0, 3.0, 4.0)
        );
    }

    #[test]
    fn no_op_and_unknown_dirty_marks_emit_no_delta() {
        let mut manager = SemanticManager::new();
        let root = node(1, SemanticRole::Group, "root", SemanticBounds::default());
        manager.add_child(None, root);
        drain(&mut manager);
        manager.mark_node_dirty(1, SemanticDirt::BOUNDS);
        assert!(drain(&mut manager).is_empty());
        manager.mark_node_dirty(99, SemanticDirt::CONTENT | SemanticDirt::BOUNDS);
        assert!(drain(&mut manager).is_empty());
    }

    #[test]
    fn request_focus_routes_only_semantic_data_nodes() {
        let mut manager = SemanticManager::new();
        let data = SemanticNodeHandle::new(1);
        data.borrow_mut().set_semantic_data_local_id(Some(42));
        let boundary = SemanticNodeHandle::new(2);
        boundary.borrow_mut().set_boundary_node(true);
        manager.add_child(None, data);
        manager.add_child(None, boundary);
        assert!(manager.request_focus(1, |local| local == 42));
        assert!(!manager.request_focus(2, |_| true));
        assert!(!manager.request_focus(99, |_| true));
    }

    #[test]
    fn node_lookup_drops_removed_semantic_data_and_rejects_unknown_ids() {
        let mut manager = SemanticManager::new();
        let node = SemanticNodeHandle::new(7);
        node.borrow_mut().set_semantic_data_local_id(Some(42));
        manager.add_child(None, node.clone());
        assert!(
            manager
                .node_by_id(7)
                .is_some_and(|found| found.ptr_eq(&node))
        );
        assert!(manager.node_by_id(99).is_none());
        manager.remove_child(&node);
        assert!(manager.node_by_id(7).is_none());
    }

    #[test]
    fn removed_ids_follow_previous_tree_preorder() {
        let mut manager = SemanticManager::new();
        let root = node(1, SemanticRole::Group, "", SemanticBounds::default());
        let child = node(2, SemanticRole::Group, "", SemanticBounds::default());
        let leaf = node(3, SemanticRole::Text, "leaf", SemanticBounds::default());
        manager.add_child(None, root.clone());
        manager.add_child(Some(&root), child.clone());
        manager.add_child(Some(&child), leaf);
        drain(&mut manager);
        manager.remove_child(&root);
        let diff = drain(&mut manager);
        assert_eq!(diff.removed, [1, 2, 3]);
    }

    #[test]
    fn absorbed_child_content_change_rederives_parent_label() {
        let mut manager = SemanticManager::new();
        let button = node(1, SemanticRole::Button, "", SemanticBounds::default());
        let child = node(2, SemanticRole::Text, "before", SemanticBounds::default());
        manager.add_child(None, button.clone());
        manager.add_child(Some(&button), child.clone());
        drain(&mut manager);
        child.borrow_mut().set_label("after");
        manager.mark_node_dirty(2, SemanticDirt::CONTENT);
        let diff = drain(&mut manager);
        assert_eq!(diff.updated_semantic.len(), 1);
        assert_eq!(diff.updated_semantic[0].id, 1);
        assert_eq!(diff.updated_semantic[0].label, "after");
    }
}
