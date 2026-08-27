use crate::mechanical_port::source::semantic::{
    semantic_dirt::SemanticDirt,
    semantic_listener::SemanticListener,
    semantic_manager::SemanticManager,
    semantic_node::{SemanticNode, SemanticNodeRef},
    semantic_snapshot::Bounds,
    semantic_state::SemanticState,
    semantic_trait::SemanticTrait,
};
use std::{cell::RefCell, rc::Rc};

pub type SemanticManagerRef = Rc<RefCell<SemanticManager>>;
pub type SemanticListenerRef = Rc<RefCell<dyn SemanticListener>>;
pub struct SemanticData {
    pub role: u32,
    pub label: String,
    pub value: String,
    pub hint: String,
    pub heading_level: u32,
    pub state_flags: u32,
    pub trait_flags: u32,
    semantic_node: Option<SemanticNodeRef>,
    semantic_manager: Option<SemanticManagerRef>,
    listeners: Vec<SemanticListenerRef>,
    bounds_retry_pending: bool,
    excluded_from_tree: bool,
    collapsed: bool,
}
impl Default for SemanticData {
    fn default() -> Self {
        Self {
            role: 0,
            label: String::new(),
            value: String::new(),
            hint: String::new(),
            heading_level: 0,
            state_flags: 0,
            trait_flags: 0,
            semantic_node: None,
            semantic_manager: None,
            listeners: Vec::new(),
            bounds_retry_pending: false,
            excluded_from_tree: false,
            collapsed: false,
        }
    }
}
impl SemanticData {
    pub fn semantic_node(
        &mut self,
        has_focus_data: bool,
        core_owner: Option<usize>,
        initial_bounds: Bounds,
    ) -> SemanticNodeRef {
        if self.semantic_node.is_none() {
            let node = SemanticNode::new(0);
            {
                let mut n = node.borrow_mut();
                n.core_owner = core_owner;
                n.semantic_data = Some(self as *mut Self as usize);
                n.role = self.role;
                n.label = self.label.clone();
                n.value = self.value.clone();
                n.hint = self.hint.clone();
                n.heading_level = self.heading_level;
                n.state_flags = self.state_flags;
                n.trait_flags = self.trait_flags
                    | if has_focus_data {
                        SemanticTrait::FOCUSABLE.0
                    } else {
                        0
                    };
                n.bounds = initial_bounds;
            }
            self.bounds_retry_pending = initial_bounds.is_empty_or_nan();
            self.semantic_node = Some(node);
        }
        self.semantic_node.as_ref().unwrap().clone()
    }
    pub fn has_semantic_node(&self) -> bool {
        self.semantic_node.is_some()
    }
    pub fn semantic_id(&self) -> u32 {
        self.semantic_node.as_ref().map_or(0, |n| n.borrow().id())
    }
    pub fn set_focused_state(&mut self, focused: bool) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        let mut n = node.borrow_mut();
        if focused {
            n.state_flags |= SemanticState::FOCUSED.0
        } else {
            n.state_flags &= !SemanticState::FOCUSED.0
        }
        drop(n);
        self.mark_content_dirty();
    }
    pub fn attach(&mut self, manager: SemanticManagerRef, parent: Option<SemanticNodeRef>) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        manager.borrow_mut().add_child(parent, node.clone());
        self.semantic_manager = Some(manager);
    }
    pub fn collapse(&mut self, value: bool, parent: Option<SemanticNodeRef>) -> bool {
        if self.collapsed == value {
            return false;
        }
        self.collapsed = value;
        let Some(node) = &self.semantic_node else {
            return true;
        };
        if value {
            if let Some(manager) = &self.semantic_manager {
                manager.borrow_mut().remove_child(node);
            }
        } else if let Some(manager) = &self.semantic_manager {
            manager.borrow_mut().add_child(parent, node.clone());
            self.bounds_retry_pending = true;
        }
        true
    }
    pub fn sync_semantic_tree_visibility(
        &mut self,
        component_missing: bool,
        component_collapsed: bool,
        drawable_hidden: bool,
        parent: Option<SemanticNodeRef>,
    ) {
        let exclude = self.state_flags & SemanticState::HIDDEN.0 != 0
            || component_missing
            || component_collapsed
            || drawable_hidden;
        if exclude == self.excluded_from_tree {
            return;
        }
        self.excluded_from_tree = exclude;
        let Some(node) = &self.semantic_node else {
            return;
        };
        if exclude {
            if let Some(manager) = &self.semantic_manager {
                manager.borrow_mut().remove_child(node);
            }
        } else if let Some(manager) = &self.semantic_manager {
            manager.borrow_mut().add_child(parent, node.clone());
            self.bounds_retry_pending = true;
        }
    }
    pub fn update_world_bounds(&mut self, bounds: Bounds) -> bool {
        let Some(node) = &self.semantic_node else {
            return false;
        };
        if bounds.is_empty_or_nan() && self.bounds_retry_pending {
            return true;
        }
        self.bounds_retry_pending = false;
        if node.borrow().bounds == bounds {
            return false;
        }
        node.borrow_mut().bounds = bounds;
        if let Some(manager) = &self.semantic_manager {
            manager
                .borrow_mut()
                .mark_node_dirty(self.semantic_id(), SemanticDirt::BOUNDS);
        }
        false
    }
    fn mark_content_dirty(&self) {
        if let (Some(node), Some(manager)) = (&self.semantic_node, &self.semantic_manager) {
            manager
                .borrow_mut()
                .mark_node_dirty(node.borrow().id(), SemanticDirt::CONTENT);
        }
    }
    pub fn set_role(&mut self, v: u32) {
        self.role = v;
        if let Some(n) = &self.semantic_node {
            if n.borrow().role != v {
                n.borrow_mut().role = v;
                self.mark_content_dirty();
            }
        }
    }
    pub fn set_label(&mut self, v: String) {
        self.label = v.clone();
        if let Some(n) = &self.semantic_node {
            if n.borrow().label != v {
                n.borrow_mut().label = v;
                self.mark_content_dirty();
            }
        }
    }
    pub fn set_value(&mut self, v: String) {
        self.value = v.clone();
        if let Some(n) = &self.semantic_node {
            if n.borrow().value != v {
                n.borrow_mut().value = v;
                self.mark_content_dirty();
            }
        }
    }
    pub fn set_hint(&mut self, v: String) {
        self.hint = v.clone();
        if let Some(n) = &self.semantic_node {
            if n.borrow().hint != v {
                n.borrow_mut().hint = v;
                self.mark_content_dirty();
            }
        }
    }
    pub fn set_heading_level(&mut self, v: u32) {
        self.heading_level = v;
        if let Some(n) = &self.semantic_node {
            if n.borrow().heading_level != v {
                n.borrow_mut().heading_level = v;
                self.mark_content_dirty();
            }
        }
    }
    pub fn set_state_flags(&mut self, v: u32) {
        self.state_flags = v;
        if let Some(n) = &self.semantic_node {
            if n.borrow().state_flags != v {
                n.borrow_mut().state_flags = v;
                self.mark_content_dirty();
            }
        }
    }
    pub fn set_trait_flags(&mut self, v: u32) {
        self.trait_flags = v;
        if let Some(n) = &self.semantic_node {
            if n.borrow().trait_flags != v {
                n.borrow_mut().trait_flags = v;
                self.mark_content_dirty();
            }
        }
    }
    pub fn apply_inferred_semantics(&mut self, role: u32, label: String) {
        if self.role != 0 || !self.label.is_empty() {
            return;
        }
        if let Some(n) = &self.semantic_node {
            let mut n = n.borrow_mut();
            if n.role != role || n.label != label {
                n.role = role;
                n.label = label;
                drop(n);
                self.mark_content_dirty();
            }
        }
    }
    pub fn add_semantic_listener(&mut self, l: SemanticListenerRef) {
        self.listeners.push(l)
    }
    pub fn remove_semantic_listener(&mut self, l: &SemanticListenerRef) {
        if let Some(i) = self.listeners.iter().position(|v| Rc::ptr_eq(v, l)) {
            self.listeners.remove(i);
        }
    }
    pub fn fire_semantic_tap(&mut self) {
        for l in &self.listeners {
            l.borrow_mut().on_semantic_tap();
        }
    }
    pub fn fire_semantic_increase(&mut self) {
        for l in &self.listeners {
            l.borrow_mut().on_semantic_increase();
        }
    }
    pub fn fire_semantic_decrease(&mut self) {
        for l in &self.listeners {
            l.borrow_mut().on_semantic_decrease();
        }
    }
}
impl Drop for SemanticData {
    fn drop(&mut self) {
        if let (Some(node), Some(manager)) = (&self.semantic_node, &self.semantic_manager) {
            manager.borrow_mut().remove_child(node);
        }
    }
}
