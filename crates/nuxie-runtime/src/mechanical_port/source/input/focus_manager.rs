use crate::mechanical_port::source::input::{
    focus_node::{EdgeBehavior, FocusNode, FocusNodeRef},
    focusable::{Key, KeyModifiers},
};
use crate::mechanical_port::source::semantic::semantic_snapshot::Bounds;
use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation, artboard::Artboard, core::CoreHandle,
};
use std::cell::{Cell, Ref, RefCell};
use std::rc::{Rc, Weak};

#[derive(Clone)]
pub struct RuntimeFocusManagerHandle(Rc<RefCell<FocusManager>>, Rc<Cell<bool>>);

#[derive(Clone, Default)]
pub struct RuntimeFocusManagerWeakHandle {
    manager: Weak<RefCell<FocusManager>>,
    focusable_content_dirty: Weak<Cell<bool>>,
    root_nodes: Weak<RefCell<Vec<FocusNodeRef>>>,
}

impl RuntimeFocusManagerWeakHandle {
    pub fn upgrade(&self) -> Option<RuntimeFocusManagerHandle> {
        let manager = self.manager.upgrade()?;
        let dirty = self
            .focusable_content_dirty
            .upgrade()
            .expect("live FocusManager owns its content dirty flag");
        Some(RuntimeFocusManagerHandle(manager, dirty))
    }

    pub(crate) fn invalidate_focusable_content(&self) {
        if let Some(dirty) = self.focusable_content_dirty.upgrade() {
            dirty.set(true);
        }
    }

    fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.root_nodes, &other.root_nodes)
    }

    fn is_attached(&self) -> bool {
        self.root_nodes.strong_count() != 0
    }

    fn erase_root(&self, child: &FocusNodeRef) {
        let Some(root_nodes) = self.root_nodes.upgrade() else {
            return;
        };
        let mut root_nodes = root_nodes.borrow_mut();
        if let Some(index) = root_nodes.iter().position(|node| Rc::ptr_eq(node, child)) {
            root_nodes.remove(index);
            self.invalidate_focusable_content();
        }
    }
}

impl RuntimeFocusManagerHandle {
    pub fn new(mut manager: FocusManager) -> Self {
        let dirty = manager.focusable_content_dirty.clone();
        let owner = Rc::new_cyclic(|owner| {
            manager.runtime_self = RuntimeFocusManagerWeakHandle {
                manager: owner.clone(),
                focusable_content_dirty: Rc::downgrade(&dirty),
                root_nodes: Rc::downgrade(&manager.root_nodes),
            };
            RefCell::new(manager)
        });
        Self(owner, dirty)
    }

    pub fn downgrade(&self) -> RuntimeFocusManagerWeakHandle {
        let root_nodes = Rc::downgrade(&self.0.borrow().root_nodes);
        RuntimeFocusManagerWeakHandle {
            manager: Rc::downgrade(&self.0),
            focusable_content_dirty: Rc::downgrade(&self.1),
            root_nodes,
        }
    }

    pub fn with_focus_manager<R>(&self, use_manager: impl FnOnce(&FocusManager) -> R) -> R {
        use_manager(&self.0.borrow())
    }

    pub fn with_focus_manager_mut<R>(&self, use_manager: impl FnOnce(&mut FocusManager) -> R) -> R {
        use_manager(&mut self.0.borrow_mut())
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[cfg(feature = "tools")]
pub type FocusChangedCallback = fn();

#[cfg(feature = "tools")]
pub type ScrollIntoViewCallback = fn(&Bounds, &CoreHandle);

pub struct FocusManager {
    runtime_self: RuntimeFocusManagerWeakHandle,
    primary_focus: Option<FocusNodeRef>,
    root_nodes: Rc<RefCell<Vec<FocusNodeRef>>>,
    has_focusable_content: bool,
    // The same canonical flag is reachable by attached nodes even while a
    // manager method is mutably borrowed; invalidation has no user callbacks.
    focusable_content_dirty: Rc<Cell<bool>>,
    #[cfg(feature = "tools")]
    focus_changed_callback: Option<FocusChangedCallback>,
    #[cfg(feature = "tools")]
    scroll_into_view_callback: Option<ScrollIntoViewCallback>,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self {
            runtime_self: RuntimeFocusManagerWeakHandle::default(),
            primary_focus: None,
            root_nodes: Rc::new(RefCell::new(Vec::new())),
            has_focusable_content: false,
            focusable_content_dirty: Rc::new(Cell::new(true)),
            #[cfg(feature = "tools")]
            focus_changed_callback: None,
            #[cfg(feature = "tools")]
            scroll_into_view_callback: None,
        }
    }
}

fn eligible_for_focus(node: &FocusNodeRef) -> bool {
    let node = node.borrow();
    if !node.can_focus() {
        return false;
    }
    #[cfg(feature = "tools")]
    if node.is_collapsed {
        return false;
    }
    node.focusable
        .as_ref()
        .is_none_or(|focusable| focusable.borrow().is_eligible_for_focus_traversal())
}

fn eligible_for_traversal(node: &FocusNodeRef) -> bool {
    node.borrow().can_traverse() && eligible_for_focus(node)
}

fn has_eligible_traversable_child(node: &FocusNodeRef) -> bool {
    node.borrow().children().iter().any(focus_node_traversable)
}

fn focus_node_traversable(node: &FocusNodeRef) -> bool {
    if eligible_for_traversal(node) {
        return true;
    }
    if node.borrow().focusable.is_some() {
        return false;
    }
    has_eligible_traversable_child(node)
}

fn is_leaf(node: &FocusNodeRef) -> bool {
    !node.borrow().children().iter().any(focus_node_traversable)
}

fn collect_all_traversable_nodes(nodes: &[FocusNodeRef], result: &mut Vec<FocusNodeRef>) {
    for node in nodes {
        if node.borrow().can_focus()
            && node.borrow().can_traverse()
            && is_leaf(node)
            && eligible_for_traversal(node)
        {
            result.push(node.clone());
        }
        let children = node.borrow().children().to_vec();
        collect_all_traversable_nodes(&children, result);
    }
}

fn subtree_has_focusable_content(nodes: &[FocusNodeRef]) -> bool {
    nodes.iter().any(|node| {
        let node = node.borrow();
        node.focusable.is_some()
            || node.can_focus()
            || subtree_has_focusable_content(node.children())
    })
}

fn cpp_max(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}

fn cpp_min(a: f32, b: f32) -> f32 {
    if b < a { b } else { a }
}

fn calculate_overlap(a_min: f32, a_max: f32, b_min: f32, b_max: f32) -> f32 {
    let overlap_min = cpp_max(a_min, b_min);
    let overlap_max = cpp_min(a_max, b_max);
    cpp_max(0.0, overlap_max - overlap_min)
}

fn bounds_center(bounds: Bounds) -> (f32, f32) {
    (
        (bounds.min_x + bounds.max_x) * 0.5,
        (bounds.min_y + bounds.max_y) * 0.5,
    )
}

fn root_bounds(node: &FocusNodeRef) -> Option<Bounds> {
    let node = node.borrow();
    if node.has_world_bounds() {
        return Some(node.world_bounds);
    }
    node.focusable
        .as_ref()
        .and_then(|focusable| focusable.borrow().world_bounds())
}

fn root_artboard(mut artboard: CoreHandle) -> CoreHandle {
    loop {
        let host = artboard
            .with_downcast::<Artboard, _>(Artboard::host)
            .flatten();
        let Some(host) = host else { break };
        let parent = host
            .with(|host| host.as_artboard_host()?.parent_artboard())
            .flatten();
        let Some(parent) = parent else { break };
        artboard = parent;
    }
    artboard
}

fn root_position(node: &FocusNodeRef) -> Option<(f32, f32)> {
    let node = node.borrow();
    if node.has_world_bounds() {
        return Some(bounds_center(node.world_bounds));
    }
    node.focusable
        .as_ref()
        .and_then(|focusable| focusable.borrow().world_position())
}

fn score_candidate_bounds(current: Bounds, candidate: Bounds, direction: Direction) -> f32 {
    let (displacement, orthogonal_distance, overlap, orthogonal_weight) = match direction {
        Direction::Left => {
            let displacement = current.min_x - candidate.max_x;
            if displacement < 0.0 {
                return f32::MAX;
            }
            (
                displacement,
                cpp_max(
                    0.0,
                    cpp_max(
                        candidate.min_y - current.max_y,
                        current.min_y - candidate.max_y,
                    ),
                ),
                calculate_overlap(
                    current.min_y,
                    current.max_y,
                    candidate.min_y,
                    candidate.max_y,
                ),
                30.0,
            )
        }
        Direction::Right => {
            let displacement = candidate.min_x - current.max_x;
            if displacement < 0.0 {
                return f32::MAX;
            }
            (
                displacement,
                cpp_max(
                    0.0,
                    cpp_max(
                        candidate.min_y - current.max_y,
                        current.min_y - candidate.max_y,
                    ),
                ),
                calculate_overlap(
                    current.min_y,
                    current.max_y,
                    candidate.min_y,
                    candidate.max_y,
                ),
                30.0,
            )
        }
        Direction::Up => {
            let displacement = current.min_y - candidate.max_y;
            if displacement < 0.0 {
                return f32::MAX;
            }
            (
                displacement,
                cpp_max(
                    0.0,
                    cpp_max(
                        candidate.min_x - current.max_x,
                        current.min_x - candidate.max_x,
                    ),
                ),
                calculate_overlap(
                    current.min_x,
                    current.max_x,
                    candidate.min_x,
                    candidate.max_x,
                ),
                2.0,
            )
        }
        Direction::Down => {
            let displacement = candidate.min_y - current.max_y;
            if displacement < 0.0 {
                return f32::MAX;
            }
            (
                displacement,
                cpp_max(
                    0.0,
                    cpp_max(
                        candidate.min_x - current.max_x,
                        current.min_x - candidate.max_x,
                    ),
                ),
                calculate_overlap(
                    current.min_x,
                    current.max_x,
                    candidate.min_x,
                    candidate.max_x,
                ),
                2.0,
            )
        }
    };
    displacement + orthogonal_weight * orthogonal_distance - overlap.sqrt()
}

fn score_candidate_point(current: (f32, f32), candidate: (f32, f32), direction: Direction) -> f32 {
    let delta = (candidate.0 - current.0, candidate.1 - current.1);
    let (primary, orthogonal, weight) = match direction {
        Direction::Left => (-delta.0, delta.1.abs(), 30.0),
        Direction::Right => (delta.0, delta.1.abs(), 30.0),
        Direction::Up => (-delta.1, delta.0.abs(), 2.0),
        Direction::Down => (delta.1, delta.0.abs(), 2.0),
    };
    if primary <= 0.0 {
        f32::MAX
    } else {
        primary + weight * orthogonal
    }
}

impl FocusManager {
    fn node_manager_handle(&self) -> RuntimeFocusManagerWeakHandle {
        RuntimeFocusManagerWeakHandle {
            manager: self.runtime_self.manager.clone(),
            focusable_content_dirty: Rc::downgrade(&self.focusable_content_dirty),
            root_nodes: Rc::downgrade(&self.root_nodes),
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn primary_focus(&self) -> Option<FocusNodeRef> {
        self.primary_focus.clone()
    }

    pub fn primary_focus_immediate_artboard(&self) -> Option<CoreHandle> {
        let focusable = self.primary_focus.as_ref()?.borrow().focusable()?;
        let artboard = focusable.borrow().focusable_artboard();
        artboard
    }

    pub fn primary_focus_artboard(&self) -> Option<CoreHandle> {
        self.primary_focus_immediate_artboard().map(root_artboard)
    }

    #[cfg(feature = "tools")]
    pub fn set_focus_changed_callback(&mut self, callback: Option<FocusChangedCallback>) {
        self.focus_changed_callback = callback;
    }

    #[cfg(feature = "tools")]
    pub fn set_scroll_into_view_callback(&mut self, callback: Option<ScrollIntoViewCallback>) {
        self.scroll_into_view_callback = callback;
    }

    pub fn primary_focus_bounds(&self) -> Option<Bounds> {
        self.primary_focus.as_ref().and_then(root_bounds)
    }

    pub fn has_primary_focus(&self, node: &FocusNodeRef) -> bool {
        self.primary_focus
            .as_ref()
            .is_some_and(|focus| Rc::ptr_eq(focus, node))
    }

    pub fn has_focus(&self, node: &FocusNodeRef) -> bool {
        node.borrow().has_focus()
    }

    pub fn drop_focus_if_focus_target_hidden(&mut self) {
        if self
            .primary_focus
            .as_ref()
            .is_some_and(|node| !eligible_for_traversal(node))
        {
            self.clear_focus();
        }
    }

    pub fn set_focus(&mut self, mut node: FocusNodeRef) {
        if eligible_for_focus(&node)
            && let Some(leaf) = self.get_first_leaf(&node)
        {
            node = leaf;
        }
        if self.has_primary_focus(&node) || !node.borrow().can_focus() || !eligible_for_focus(&node)
        {
            return;
        }
        let old_focus = self.primary_focus.replace(node.clone());
        self.notify_focus_change(old_focus.as_ref(), Some(&node));
    }

    pub fn clear_focus(&mut self) {
        if let Some(old_focus) = self.primary_focus.take() {
            self.notify_focus_change(Some(&old_focus), None);
        }
    }

    fn notify_focus_change(
        &mut self,
        old_focus: Option<&FocusNodeRef>,
        new_focus: Option<&FocusNodeRef>,
    ) {
        let common_ancestor = if let (Some(old_focus), Some(new_focus)) = (old_focus, new_focus) {
            let mut old_ancestors = Vec::new();
            let mut current = Some(old_focus.clone());
            while let Some(node) = current {
                old_ancestors.push(node.clone());
                current = node.borrow().parent();
            }
            let mut current = Some(new_focus.clone());
            let mut common = None;
            while let Some(node) = current {
                if old_ancestors
                    .iter()
                    .any(|ancestor| Rc::ptr_eq(ancestor, &node))
                {
                    common = Some(node);
                    break;
                }
                current = node.borrow().parent();
            }
            common
        } else {
            None
        };

        let mut current = old_focus.cloned();
        while let Some(node) = current {
            if common_ancestor
                .as_ref()
                .is_some_and(|common| Rc::ptr_eq(common, &node))
                || !node.borrow().has_focus()
            {
                break;
            }
            {
                let mut node = node.borrow_mut();
                node.set_has_focus(false);
                node.blurred();
            }
            current = node.borrow().parent();
        }

        let mut current = new_focus.cloned();
        while let Some(node) = current {
            if common_ancestor
                .as_ref()
                .is_some_and(|common| Rc::ptr_eq(common, &node))
                || node.borrow().has_focus()
            {
                break;
            }
            {
                let mut node = node.borrow_mut();
                node.set_has_focus(true);
                node.focused();
            }
            current = node.borrow().parent();
        }

        #[cfg(feature = "tools")]
        {
            if let Some(callback) = self.focus_changed_callback {
                callback();
            }

            if let (Some(callback), Some(new_focus)) = (self.scroll_into_view_callback, new_focus)
                && let Some(bounds) = root_bounds(new_focus)
                && let Some(focusable) = new_focus.borrow().focusable()
                && let Some(artboard) = focusable.borrow().focusable_artboard()
            {
                let artboard = root_artboard(artboard);
                let is_hostless = artboard
                    .with_downcast::<Artboard, _>(|artboard| artboard.host().is_none())
                    .unwrap_or(false);
                if is_hostless {
                    callback(&bounds, &artboard);
                }
            }
        }
    }

    pub fn add_child(
        &mut self,
        parent: Option<FocusNodeRef>,
        child: FocusNodeRef,
        index: Option<usize>,
    ) {
        let manager = self.node_manager_handle();
        let index = index.unwrap_or_else(|| {
            parent.as_ref().map_or_else(
                || self.root_nodes.borrow().len(),
                |parent| parent.borrow().children().len(),
            )
        });
        self.mark_focusable_content_dirty();
        let (previous_parent, previous_manager) = {
            let child = child.borrow();
            (child.parent(), child.manager.clone())
        };
        if previous_parent.is_some() {
            FocusNode::remove_from_parent(&child);
        } else if previous_manager.is_attached() && !previous_manager.ptr_eq(&manager) {
            previous_manager.erase_root(&child);
        } else {
            self.erase_root(&child);
        }
        // Source assigns only this child. Do not recursively rewrite its
        // descendants' manager identities during a hierarchy reorder.
        child.borrow_mut().manager = manager;
        if let Some(parent) = parent {
            FocusNode::insert_child(&parent, index, child);
        } else {
            let mut root_nodes = self.root_nodes.borrow_mut();
            let index = index.min(root_nodes.len());
            root_nodes.insert(index, child);
        }
    }

    pub fn remove_child(&mut self, child: &FocusNodeRef) {
        if self.has_focus(child) {
            self.clear_focus();
        }
        self.detach_child(child);
    }

    pub fn detach_child(&mut self, child: &FocusNodeRef) {
        self.mark_focusable_content_dirty();
        Self::remove_manager(child);
        let has_parent = child.borrow().parent().is_some();
        if has_parent {
            FocusNode::remove_from_parent(child);
        } else {
            self.erase_root(child);
        }
    }

    fn remove_manager(node: &FocusNodeRef) {
        let children = node.borrow().children().to_vec();
        for child in children {
            Self::remove_manager(&child);
        }
        node.borrow_mut().manager = RuntimeFocusManagerWeakHandle::default();
    }

    fn erase_root(&mut self, child: &FocusNodeRef) {
        let removed = {
            let mut root_nodes = self.root_nodes.borrow_mut();
            if let Some(index) = root_nodes.iter().position(|node| Rc::ptr_eq(node, child)) {
                root_nodes.remove(index);
                true
            } else {
                false
            }
        };
        if removed {
            self.mark_focusable_content_dirty();
        }
    }

    pub fn root_nodes(&self) -> Ref<'_, [FocusNodeRef]> {
        Ref::map(self.root_nodes.borrow(), Vec::as_slice)
    }

    pub fn mark_focusable_content_dirty(&mut self) {
        self.focusable_content_dirty.set(true);
    }

    pub fn has_focusable_content(&mut self) -> bool {
        if self.focusable_content_dirty.get() {
            self.has_focusable_content = subtree_has_focusable_content(&self.root_nodes.borrow());
            self.focusable_content_dirty.set(false);
        }
        self.has_focusable_content
    }

    pub fn get_traversable_nodes(&self, scope: Option<&FocusNodeRef>) -> Vec<FocusNodeRef> {
        let mut result: Vec<_> = match scope {
            Some(scope) => scope
                .borrow()
                .children()
                .iter()
                .filter(|child| focus_node_traversable(child))
                .cloned()
                .collect(),
            None => self
                .root_nodes
                .borrow()
                .iter()
                .filter(|child| focus_node_traversable(child))
                .cloned()
                .collect(),
        };
        result.sort_by_key(|node| node.borrow().tab_index());
        result
    }

    fn get_first_leaf(&self, node: &FocusNodeRef) -> Option<FocusNodeRef> {
        for child in self.get_traversable_nodes(Some(node)) {
            if let Some(leaf) = self.get_first_leaf(&child) {
                return Some(leaf);
            }
        }
        (eligible_for_traversal(node) && !has_eligible_traversable_child(node))
            .then(|| node.clone())
    }

    fn get_last_leaf(&self, node: &FocusNodeRef) -> Option<FocusNodeRef> {
        for child in self.get_traversable_nodes(Some(node)).into_iter().rev() {
            if let Some(leaf) = self.get_last_leaf(&child) {
                return Some(leaf);
            }
        }
        (eligible_for_traversal(node) && !has_eligible_traversable_child(node))
            .then(|| node.clone())
    }

    fn first_eligible_leaf_from(
        &self,
        traversable: &[FocusNodeRef],
        forward: bool,
    ) -> Option<FocusNodeRef> {
        if forward {
            traversable
                .iter()
                .find_map(|node| self.get_first_leaf(node))
        } else {
            traversable
                .iter()
                .rev()
                .find_map(|node| self.get_last_leaf(node))
        }
    }

    fn find_next_focusable(
        &mut self,
        current: Option<FocusNodeRef>,
        forward: bool,
    ) -> Option<FocusNodeRef> {
        let scope = current.as_ref().and_then(|node| node.borrow().parent());
        let traversable = self.get_traversable_nodes(scope.as_ref());
        if traversable.is_empty() {
            return if scope.is_some() {
                self.find_next_focusable(scope, forward)
            } else {
                None
            };
        }

        let current_index = current.as_ref().and_then(|current| {
            traversable
                .iter()
                .position(|node| Rc::ptr_eq(node, current))
        });
        let next = if let Some(index) = current_index {
            let direct = if forward {
                traversable[index + 1..]
                    .iter()
                    .find_map(|node| self.get_first_leaf(node))
            } else {
                traversable[..index]
                    .iter()
                    .rev()
                    .find_map(|node| self.get_last_leaf(node))
            };
            if direct.is_some() {
                direct
            } else {
                match scope.as_ref().map_or(EdgeBehavior::ParentScope, |scope| {
                    scope.borrow().edge_behavior()
                }) {
                    EdgeBehavior::ClosedLoop => {
                        let wrapped = if forward {
                            traversable[..index]
                                .iter()
                                .find_map(|node| self.get_first_leaf(node))
                        } else {
                            traversable[index + 1..]
                                .iter()
                                .rev()
                                .find_map(|node| self.get_last_leaf(node))
                        };
                        wrapped.or_else(|| self.first_eligible_leaf_from(&traversable, forward))
                    }
                    EdgeBehavior::Stop => current.clone(),
                    EdgeBehavior::Unknown => None,
                    EdgeBehavior::ParentScope => {
                        if scope.is_some() {
                            return self.find_next_focusable(scope, forward);
                        }
                        None
                    }
                }
            }
        } else {
            self.first_eligible_leaf_from(&traversable, forward)
        };

        let changed = match (next.as_ref(), current.as_ref()) {
            (Some(next), Some(current)) => !Rc::ptr_eq(next, current),
            (None, None) => false,
            _ => true,
        };
        if changed {
            if let Some(next_node) = next.as_ref() {
                self.set_focus(next_node.clone());
            } else {
                // Source setFocus(nullptr) at the root edge still clears the
                // previous focus, although focusNext/Previous returns false.
                self.clear_focus();
            }
            next
        } else {
            None
        }
    }

    pub fn focus_next(&mut self) -> bool {
        self.drop_focus_if_focus_target_hidden();
        self.find_next_focusable(self.primary_focus.clone(), true)
            .is_some()
    }

    pub fn focus_previous(&mut self) -> bool {
        self.drop_focus_if_focus_target_hidden();
        self.find_next_focusable(self.primary_focus.clone(), false)
            .is_some()
    }

    fn find_node_in_direction(
        &self,
        current: &FocusNodeRef,
        direction: Direction,
    ) -> Option<FocusNodeRef> {
        let mut candidates = Vec::new();
        collect_all_traversable_nodes(&self.root_nodes.borrow(), &mut candidates);
        let current_bounds = root_bounds(current);
        let current_position = if current_bounds.is_none() {
            Some(root_position(current)?)
        } else {
            None
        };
        let mut best = None;
        let mut best_score = f32::MAX;
        for candidate in candidates {
            if Rc::ptr_eq(&candidate, current) {
                continue;
            }
            let score = if let (Some(current_bounds), Some(candidate_bounds)) =
                (current_bounds, root_bounds(&candidate))
            {
                score_candidate_bounds(current_bounds, candidate_bounds, direction)
            } else {
                let Some(candidate_position) = root_position(&candidate) else {
                    continue;
                };
                score_candidate_point(
                    current_bounds.map_or_else(
                        || current_position.expect("a current position was required"),
                        bounds_center,
                    ),
                    candidate_position,
                    direction,
                )
            };
            if score < best_score {
                best_score = score;
                best = Some(candidate);
            }
        }
        best
    }

    fn focus_direction(&mut self, direction: Direction) -> bool {
        self.drop_focus_if_focus_target_hidden();
        let Some(current) = self.primary_focus.clone() else {
            return false;
        };
        let Some(next) = self.find_node_in_direction(&current, direction) else {
            return false;
        };
        self.set_focus(next);
        true
    }

    pub fn focus_left(&mut self) -> bool {
        self.focus_direction(Direction::Left)
    }

    pub fn focus_right(&mut self) -> bool {
        self.focus_direction(Direction::Right)
    }

    pub fn focus_up(&mut self) -> bool {
        self.focus_direction(Direction::Up)
    }

    pub fn focus_down(&mut self) -> bool {
        self.focus_direction(Direction::Down)
    }

    pub fn key_input(
        &mut self,
        key: Key,
        modifiers: KeyModifiers,
        is_pressed: bool,
        is_repeat: bool,
    ) -> bool {
        self.drop_focus_if_focus_target_hidden();
        let mut node = self.primary_focus.clone();
        while let Some(current) = node {
            if current
                .borrow_mut()
                .key_input(key, modifiers, is_pressed, is_repeat)
            {
                return true;
            }
            node = current.borrow().parent();
        }
        false
    }

    pub fn text_input(&mut self, text: &str) -> bool {
        self.drop_focus_if_focus_target_hidden();
        let mut node = self.primary_focus.clone();
        while let Some(current) = node {
            if current.borrow_mut().text_input(text) {
                return true;
            }
            node = current.borrow().parent();
        }
        false
    }

    pub fn gamepad_dispatch(
        &mut self,
        invocation: &ListenerInvocation,
        out_dispatched_scripted_drawable: Option<&mut Option<CoreHandle>>,
    ) -> bool {
        self.drop_focus_if_focus_target_hidden();
        let mut node = self.primary_focus.clone();
        let mut out_dispatched_scripted_drawable = out_dispatched_scripted_drawable;
        while let Some(current) = node {
            if current
                .borrow()
                .focusable
                .as_ref()
                .is_some_and(|focusable| {
                    focusable.borrow_mut().gamepad_dispatch(
                        invocation,
                        out_dispatched_scripted_drawable.as_deref_mut(),
                    )
                })
            {
                return true;
            }
            node = current.borrow().parent();
        }
        false
    }
}

impl Drop for FocusManager {
    fn drop(&mut self) {
        let root_nodes = self.root_nodes.borrow().clone();
        for node in &root_nodes {
            Self::remove_manager(node);
        }
        // No focus/blur notifications during destruction, as in the source.
        self.primary_focus = None;
    }
}
