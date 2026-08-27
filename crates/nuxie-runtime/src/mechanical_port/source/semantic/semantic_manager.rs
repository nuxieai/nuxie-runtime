use crate::mechanical_port::source::semantic::{
    semantic_dirt::SemanticDirt,
    semantic_node::{SemanticNode, SemanticNodeRef},
    semantic_role::is_interactive_role_value,
    semantic_snapshot::{
        SemanticsBoundsUpdate, SemanticsChildrenUpdate, SemanticsDiff, SemanticsDiffNode,
    },
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct SemanticManager {
    dirt: SemanticDirt,
    last_diff: SemanticsDiff,
    last_flat_snapshot: Vec<SemanticsDiffNode>,
    version: u64,
    next_local_id: u32,
    nodes_by_id: HashMap<u32, SemanticNodeRef>,
    roots: Vec<SemanticNodeRef>,
    dirty_content_nodes: HashSet<u32>,
    dirty_bounds_nodes: HashSet<u32>,
    dirty_boundary_ids: HashSet<u32>,
    derived_labels: HashMap<u32, String>,
    excluded_ids: HashSet<u32>,
    frame_number: u64,
}
impl Default for SemanticManager {
    fn default() -> Self {
        Self {
            dirt: SemanticDirt::ALL,
            last_diff: SemanticsDiff::default(),
            last_flat_snapshot: Vec::new(),
            version: 0,
            next_local_id: 1,
            nodes_by_id: HashMap::new(),
            roots: Vec::new(),
            dirty_content_nodes: HashSet::new(),
            dirty_bounds_nodes: HashSet::new(),
            dirty_boundary_ids: HashSet::new(),
            derived_labels: HashMap::new(),
            excluded_ids: HashSet::new(),
            frame_number: 0,
        }
    }
}
impl SemanticManager {
    pub fn set_frame_number(&mut self, value: u64) {
        self.frame_number = value;
    }
    pub fn mark_dirty(&mut self, dirt: SemanticDirt) {
        self.dirt |= dirt;
    }
    pub fn is_dirty(&self) -> bool {
        self.dirt != SemanticDirt::NONE
    }
    pub fn version(&self) -> u64 {
        self.version
    }
    pub fn node_by_id(&self, id: u32) -> Option<SemanticNodeRef> {
        self.nodes_by_id.get(&id).cloned()
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
    fn ensure_node_id(&mut self, node: &mut SemanticNode) {
        if node.id == 0 {
            while self.nodes_by_id.contains_key(&self.next_local_id) {
                self.next_local_id = self.next_local_id.wrapping_add(1);
            }
            node.id = self.next_local_id;
            self.next_local_id = self.next_local_id.wrapping_add(1);
        } else if node.id >= self.next_local_id {
            self.next_local_id = node.id.wrapping_add(1);
        }
    }
    pub fn add_child(&mut self, parent: Option<SemanticNodeRef>, child: SemanticNodeRef) {
        let id = child.borrow().id;
        if id != 0
            && self
                .nodes_by_id
                .get(&id)
                .is_some_and(|n| !Rc::ptr_eq(n, &child))
        {
            child.borrow_mut().id = 0;
        }
        self.ensure_node_id(&mut child.borrow_mut());
        let id = child.borrow().id;
        self.nodes_by_id.entry(id).or_insert_with(|| child.clone());
        if let Some(parent) = parent {
            self.nodes_by_id
                .entry(parent.borrow().id)
                .or_insert_with(|| parent.clone());
            child.borrow_mut().parent = Rc::downgrade(&parent);
            parent.borrow_mut().children.push(child);
        } else {
            child.borrow_mut().parent = std::rc::Weak::new();
            self.roots.push(child);
        }
        self.mark_dirty(SemanticDirt::STRUCTURE);
    }
    pub fn remove_child(&mut self, node: &SemanticNodeRef) {
        if let Some(parent) = node.borrow().parent() {
            parent
                .borrow_mut()
                .children
                .retain(|n| !Rc::ptr_eq(n, node));
        } else {
            self.roots.retain(|n| !Rc::ptr_eq(n, node));
        }
        node.borrow_mut().parent = std::rc::Weak::new();
        self.nodes_by_id.remove(&node.borrow().id);
        self.mark_dirty(SemanticDirt::STRUCTURE);
    }
    fn normalize_label(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut last_was_space = true;
        for character in input.chars() {
            if u32::from(character) <= u32::from(b' ') {
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
    fn collect_labels(
        node: &SemanticNodeRef,
        text: &mut String,
        image: &mut String,
        absorbed: &mut HashSet<u32>,
    ) {
        let n = node.borrow();
        if is_interactive_role_value(n.role) {
            return;
        }
        absorbed.insert(n.id);
        if n.role == 7 && !n.label.is_empty() {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(&n.label);
        } else if n.role == 8 && image.is_empty() && !n.label.is_empty() {
            image.push_str(&n.label);
        }
        let children = n.children.clone();
        drop(n);
        for child in children {
            Self::collect_labels(&child, text, image, absorbed);
        }
    }
    fn derive_visit(&mut self, node: &SemanticNodeRef) {
        let n = node.borrow();
        let children = n.children.clone();
        let derives = is_interactive_role_value(n.role) && n.label.is_empty();
        let id = n.id;
        drop(n);
        if derives {
            let mut text = String::new();
            let mut image = String::new();
            let mut absorbed = HashSet::new();
            for child in &children {
                Self::collect_labels(child, &mut text, &mut image, &mut absorbed);
            }
            let text = Self::normalize_label(&text);
            let label = if text.is_empty() {
                Self::normalize_label(&image)
            } else {
                text
            };
            if !label.is_empty() {
                self.derived_labels.insert(id, label);
                self.excluded_ids.extend(absorbed);
            }
        }
        for child in children {
            if !self.excluded_ids.contains(&child.borrow().id) {
                self.derive_visit(&child);
            }
        }
    }
    fn flatten_node(
        &self,
        node: &SemanticNodeRef,
        parent_id: i32,
        sibling: &mut u32,
        out: &mut Vec<SemanticsDiffNode>,
    ) {
        let n = node.borrow();
        let children = n.children.clone();
        if self.excluded_ids.contains(&n.id) || n.is_boundary_node {
            drop(n);
            for child in children {
                self.flatten_node(&child, parent_id, sibling, out);
            }
            return;
        }
        let flat = SemanticsDiffNode {
            id: n.id,
            role: n.role,
            label: self
                .derived_labels
                .get(&n.id)
                .cloned()
                .unwrap_or_else(|| n.label.clone()),
            value: n.value.clone(),
            hint: n.hint.clone(),
            state_flags: n.state_flags,
            trait_flags: n.trait_flags,
            heading_level: n.heading_level,
            min_x: n.bounds.min_x,
            min_y: n.bounds.min_y,
            max_x: n.bounds.max_x,
            max_y: n.bounds.max_y,
            parent_id,
            sibling_index: *sibling,
        };
        *sibling = (*sibling).wrapping_add(1);
        let id = n.id;
        drop(n);
        out.push(flat);
        let mut child_sibling = 0;
        for child in children {
            self.flatten_node(&child, id as i32, &mut child_sibling, out);
        }
    }
    fn sort_nodes(nodes: &mut [SemanticNodeRef]) {
        nodes.sort_by(|a, b| {
            let a = a.borrow().bounds;
            let b = b.borrow().bounds;
            let a_empty = a.is_empty_or_nan();
            let b_empty = b.is_empty_or_nan();
            if a_empty != b_empty {
                return if b_empty {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }
            if a_empty {
                return std::cmp::Ordering::Equal;
            }
            if a.min_y != b.min_y {
                return if a.min_y < b.min_y {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }
            if a.min_x < b.min_x {
                std::cmp::Ordering::Less
            } else if b.min_x < a.min_x {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
        for node in nodes {
            Self::sort_nodes(&mut node.borrow_mut().children);
        }
    }
    fn flatten(&self) -> Vec<SemanticsDiffNode> {
        let mut out = Vec::with_capacity(self.nodes_by_id.len());
        let mut sibling = 0;
        for root in &self.roots {
            self.flatten_node(root, -1, &mut sibling, &mut out);
        }
        out
    }
    fn children_map(nodes: &[SemanticsDiffNode]) -> HashMap<i32, Vec<u32>> {
        let mut grouped: HashMap<i32, Vec<(u32, u32)>> = HashMap::new();
        for n in nodes {
            grouped
                .entry(n.parent_id)
                .or_default()
                .push((n.sibling_index, n.id));
        }
        grouped
            .into_iter()
            .map(|(p, mut v)| {
                v.sort_by_key(|v| v.0);
                (p, v.into_iter().map(|v| v.1).collect())
            })
            .collect()
    }
    fn build_diff(
        current: &[SemanticsDiffNode],
        previous: &[SemanticsDiffNode],
        version: u64,
        frame: u64,
    ) -> SemanticsDiff {
        let mut d = SemanticsDiff {
            frame_number: frame,
            tree_version: version,
            ..Default::default()
        };
        let roots: Vec<_> = current.iter().filter(|n| n.parent_id == -1).collect();
        if roots.len() == 1 {
            d.root_id = roots[0].id;
        }
        let cm = Self::children_map(current);
        if previous.is_empty() {
            let mut seen = HashSet::new();
            for n in current {
                if seen.insert(n.parent_id) {
                    if let Some(ids) = cm.get(&n.parent_id) {
                        d.children_updated.push(SemanticsChildrenUpdate {
                            parent_id: n.parent_id,
                            child_ids: ids.clone(),
                        });
                    }
                }
                d.added.push(n.clone());
            }
            return d;
        }
        let ci: HashMap<_, _> = current.iter().map(|n| (n.id, n)).collect();
        let pi: HashMap<_, _> = previous.iter().map(|n| (n.id, n)).collect();
        let pm = Self::children_map(previous);
        for n in previous {
            if !ci.contains_key(&n.id) {
                d.removed.push(n.id);
            }
        }
        for n in current {
            let Some(p) = pi.get(&n.id) else {
                d.added.push(n.clone());
                continue;
            };
            if p.parent_id != n.parent_id || p.sibling_index != n.sibling_index {
                d.moved.push(n.clone());
            }
            if p.role != n.role
                || p.label != n.label
                || p.state_flags != n.state_flags
                || p.trait_flags != n.trait_flags
            {
                d.updated_semantic.push(n.clone());
            }
            if p.min_x != n.min_x || p.min_y != n.min_y || p.max_x != n.max_x || p.max_y != n.max_y
            {
                d.updated_geometry.push(SemanticsBoundsUpdate {
                    id: n.id,
                    min_x: n.min_x,
                    min_y: n.min_y,
                    max_x: n.max_x,
                    max_y: n.max_y,
                });
            }
        }
        let mut order = Vec::new();
        let mut seen = HashSet::new();
        for n in current.iter().chain(previous) {
            if seen.insert(n.parent_id) {
                order.push(n.parent_id);
            }
        }
        for p in order {
            let c = cm.get(&p).cloned().unwrap_or_default();
            let old = pm.get(&p).cloned().unwrap_or_default();
            if c != old {
                d.children_updated.push(SemanticsChildrenUpdate {
                    parent_id: p,
                    child_ids: c,
                });
            }
        }
        d
    }
    fn patch_incremental(&mut self) -> SemanticsDiff {
        let mut d = SemanticsDiff {
            frame_number: self.frame_number,
            ..Default::default()
        };
        for entry in &mut self.last_flat_snapshot {
            if let Some(node) = self.nodes_by_id.get(&entry.id) {
                let n = node.borrow();
                if self.dirty_bounds_nodes.contains(&entry.id) && (entry.bounds() != n.bounds) {
                    entry.set_bounds(n.bounds);
                    d.updated_geometry.push(SemanticsBoundsUpdate {
                        id: entry.id,
                        min_x: entry.min_x,
                        min_y: entry.min_y,
                        max_x: entry.max_x,
                        max_y: entry.max_y,
                    });
                }
                if self.dirty_content_nodes.contains(&entry.id) {
                    let label = self.derived_labels.get(&entry.id).unwrap_or(&n.label);
                    if entry.role != n.role
                        || &entry.label != label
                        || entry.value != n.value
                        || entry.hint != n.hint
                        || entry.state_flags != n.state_flags
                        || entry.trait_flags != n.trait_flags
                        || entry.heading_level != n.heading_level
                    {
                        entry.role = n.role;
                        entry.label = label.clone();
                        entry.value = n.value.clone();
                        entry.hint = n.hint.clone();
                        entry.state_flags = n.state_flags;
                        entry.trait_flags = n.trait_flags;
                        entry.heading_level = n.heading_level;
                        d.updated_semantic.push(entry.clone());
                    }
                }
            }
        }
        if self.roots.len() == 1 {
            d.root_id = self.roots[0].borrow().id;
        }
        d
    }
    fn refresh(&mut self) {
        let structure = self.dirt.contains(SemanticDirt::STRUCTURE);
        let bounds = self.dirt.contains(SemanticDirt::BOUNDS);
        let content = self.dirt.contains(SemanticDirt::CONTENT);
        if !structure && !bounds && !content {
            return;
        }
        let rederive = content
            && self
                .dirty_content_nodes
                .iter()
                .any(|id| self.excluded_ids.contains(id));
        let mut needs_reorder = false;
        if bounds && !structure && !self.last_flat_snapshot.is_empty() {
            let mut parents = Vec::new();
            let mut root_moved = false;
            for id in &self.dirty_bounds_nodes {
                let Some(node) = self.nodes_by_id.get(id) else {
                    continue;
                };
                if let Some(parent) = node.borrow().parent() {
                    if !parents
                        .iter()
                        .any(|candidate| Rc::ptr_eq(candidate, &parent))
                    {
                        parents.push(parent);
                    }
                } else {
                    root_moved = true;
                }
            }
            needs_reorder = parents.iter().any(|parent| {
                let parent = parent.borrow();
                parent.children.len() > 1 && !Self::nodes_in_visual_order(&parent.children)
            }) || (root_moved
                && self.roots.len() > 1
                && !Self::nodes_in_visual_order(&self.roots));
        }
        if structure || rederive || needs_reorder || self.last_flat_snapshot.is_empty() {
            if structure || needs_reorder || self.last_flat_snapshot.is_empty() {
                Self::sort_nodes(&mut self.roots);
            }
            self.derived_labels.clear();
            self.excluded_ids.clear();
            for root in self.roots.clone() {
                self.derive_visit(&root);
            }
            let flat = self.flatten();
            let diff = Self::build_diff(
                &flat,
                &self.last_flat_snapshot,
                self.version.wrapping_add(1),
                self.frame_number,
            );
            if !diff.is_empty() {
                self.version = self.version.wrapping_add(1);
                self.last_diff = diff;
                self.last_flat_snapshot = flat;
            }
        } else {
            let mut diff = self.patch_incremental();
            if !diff.is_empty() {
                self.version = self.version.wrapping_add(1);
                diff.tree_version = self.version;
                self.last_diff = diff;
            }
        }
        self.dirt = SemanticDirt::NONE;
        self.dirty_content_nodes.clear();
        self.dirty_bounds_nodes.clear();
        self.dirty_boundary_ids.clear();
    }

    fn nodes_in_visual_order(nodes: &[SemanticNodeRef]) -> bool {
        let mut previous = None;
        for node in nodes {
            let bounds = node.borrow().bounds;
            if bounds.is_empty_or_nan() {
                continue;
            }
            if let Some(previous_bounds) = previous {
                if bounds.min_y < previous_bounds.min_y
                    || (bounds.min_y == previous_bounds.min_y
                        && bounds.min_x < previous_bounds.min_x)
                {
                    return false;
                }
            }
            previous = Some(bounds);
        }
        true
    }
    pub fn drain_diff(&mut self) -> SemanticsDiff {
        self.refresh();
        std::mem::take(&mut self.last_diff)
    }
}
