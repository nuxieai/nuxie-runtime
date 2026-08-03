// Pinned C++ correspondence (d788e8ec):
// src/semantic/semantic_data.cpp:1-572 and
// include/rive/semantic/semantic_data.hpp:1-79.

use std::rc::Rc;

use crate::ArtboardInstance;
use crate::semantic_manager::SemanticManager;
use crate::semantic_node::{
    SemanticDirt, SemanticNodeHandle, SemanticState, SemanticTrait, has_semantic_state,
};
use crate::semantic_provider::{
    ResolvedSemanticData, SemanticProvider, semantic_string_property, semantic_uint_property,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SemanticActionType {
    Tap = 0,
    Increase = 1,
    Decrease = 2,
}

impl SemanticActionType {
    pub fn from_raw(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::Tap,
            1 => Self::Increase,
            2 => Self::Decrease,
            _ => return None,
        })
    }
}

pub trait SemanticListener: std::fmt::Debug {
    fn on_semantic_tap(&self);
    fn on_semantic_increase(&self);
    fn on_semantic_decrease(&self);
}

#[derive(Debug, Clone)]
pub struct RuntimeSemanticData {
    pub local_id: usize,
    pub parent_local_id: Option<usize>,
    role: u32,
    label: String,
    value: String,
    hint: String,
    heading_level: u32,
    trait_flags: u32,
    state_flags: u32,
    semantic_node: Option<SemanticNodeHandle>,
    semantic_manager_identity: Option<u64>,
    tree_parent: Option<SemanticNodeHandle>,
    semantic_listeners: Vec<Rc<dyn SemanticListener>>,
    bounds_retry_pending: bool,
    excluded_from_tree: bool,
}

impl RuntimeSemanticData {
    pub fn new(local_id: usize, parent_local_id: Option<usize>) -> Self {
        Self {
            local_id,
            parent_local_id,
            role: 0,
            label: String::new(),
            value: String::new(),
            hint: String::new(),
            heading_level: 0,
            trait_flags: 0,
            state_flags: 0,
            semantic_node: None,
            semantic_manager_identity: None,
            tree_parent: None,
            semantic_listeners: Vec::new(),
            bounds_retry_pending: false,
            excluded_from_tree: false,
        }
    }

    pub(crate) fn from_artboard(artboard: &ArtboardInstance, local_id: usize) -> Self {
        let mut data = Self::new(local_id, artboard.component_parent_local(local_id));
        data.role = semantic_uint_property(artboard, local_id, "role");
        data.label = semantic_string_property(artboard, local_id, "label");
        data.value = semantic_string_property(artboard, local_id, "value");
        data.hint = semantic_string_property(artboard, local_id, "hint");
        data.heading_level = semantic_uint_property(artboard, local_id, "headingLevel");
        data.trait_flags = semantic_uint_property(artboard, local_id, "traitFlags");
        data.state_flags = semantic_uint_property(artboard, local_id, "stateFlags");
        data
    }

    pub fn has_semantic_node(&self) -> bool {
        self.semantic_node.is_some()
    }

    pub fn semantic_id(&self) -> u32 {
        self.semantic_node
            .as_ref()
            .map_or(0, |node| node.borrow().id())
    }

    pub fn semantic_node(&mut self, artboard: &mut ArtboardInstance) -> SemanticNodeHandle {
        if let Some(node) = &self.semantic_node {
            return node.clone();
        }
        let node = SemanticNodeHandle::new(0);
        {
            let mut node = node.borrow_mut();
            node.set_core_owner_local_id(self.parent_local_id);
            node.set_semantic_data_local_id(Some(self.local_id));
            node.set_role(self.role);
            node.set_label(self.label.clone());
            node.set_value(self.value.clone());
            node.set_hint(self.hint.clone());
            node.set_heading_level(self.heading_level);
            node.set_state_flags(self.state_flags);
            let mut traits = self.trait_flags;
            if self.parent_has_focus_data(artboard) {
                traits |= SemanticTrait::FOCUSABLE.0;
            }
            node.set_trait_flags(traits);
        }
        self.semantic_node = Some(node.clone());
        self.apply_inferred_semantics_if_needed(artboard, None);
        self.bounds_retry_pending = true;
        self.update_world_bounds(artboard, None);
        node
    }

    pub fn node_handle(&self) -> Option<SemanticNodeHandle> {
        self.semantic_node.clone()
    }

    fn parent_has_focus_data(&self, artboard: &ArtboardInstance) -> bool {
        let Some(parent) = self
            .parent_local_id
            .and_then(|local| artboard.component(local))
        else {
            return false;
        };
        parent.children.iter().any(|child| {
            artboard
                .component_local_id(*child)
                .is_some_and(|local| artboard.runtime_object_type_name(local) == Some("FocusData"))
        })
    }

    pub fn attach(
        &mut self,
        manager: &mut SemanticManager,
        parent: Option<&SemanticNodeHandle>,
        artboard: &mut ArtboardInstance,
    ) -> u32 {
        let node = self.semantic_node(artboard);
        self.tree_parent = parent.cloned();
        self.semantic_manager_identity = Some(manager.identity());
        self.excluded_from_tree = self.should_exclude_from_tree(artboard);
        if self.excluded_from_tree {
            return node.borrow().id();
        }
        let id = manager.add_child(parent, node);
        id
    }

    pub fn detach(&mut self, manager: &mut SemanticManager) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        if node.borrow().manager_identity() == Some(manager.identity()) {
            manager.remove_child(node);
        }
        self.semantic_manager_identity = None;
        self.tree_parent = None;
    }

    pub fn set_focused_state(&mut self, focused: bool, manager: Option<&mut SemanticManager>) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        let mut flags = node.borrow().state_flags();
        if focused {
            flags |= SemanticState::FOCUSED.0;
        } else {
            flags &= !SemanticState::FOCUSED.0;
        }
        node.borrow_mut().set_state_flags(flags);
        if let Some(manager) = manager {
            manager.mark_node_dirty(node.borrow().id(), SemanticDirt::CONTENT);
        }
    }

    pub fn set_role(&mut self, value: u32, manager: Option<&mut SemanticManager>) -> bool {
        if self.role == value {
            return false;
        }
        self.role = value;
        self.update_node_content(manager, |node| node.set_role(value));
        true
    }

    pub fn set_label(
        &mut self,
        value: impl Into<String>,
        manager: Option<&mut SemanticManager>,
    ) -> bool {
        let value = value.into();
        if self.label == value {
            return false;
        }
        self.label = value.clone();
        self.update_node_content(manager, |node| node.set_label(value));
        true
    }

    pub fn set_value(
        &mut self,
        value: impl Into<String>,
        manager: Option<&mut SemanticManager>,
    ) -> bool {
        let value = value.into();
        if self.value == value {
            return false;
        }
        self.value = value.clone();
        self.update_node_content(manager, |node| node.set_value(value));
        true
    }

    pub fn set_hint(
        &mut self,
        value: impl Into<String>,
        manager: Option<&mut SemanticManager>,
    ) -> bool {
        let value = value.into();
        if self.hint == value {
            return false;
        }
        self.hint = value.clone();
        self.update_node_content(manager, |node| node.set_hint(value));
        true
    }

    pub fn set_heading_level(&mut self, value: u32, manager: Option<&mut SemanticManager>) -> bool {
        if self.heading_level == value {
            return false;
        }
        self.heading_level = value;
        self.update_node_content(manager, |node| node.set_heading_level(value));
        true
    }

    pub fn set_trait_flags(&mut self, value: u32, manager: Option<&mut SemanticManager>) -> bool {
        if self.trait_flags == value {
            return false;
        }
        self.trait_flags = value;
        self.update_node_content(manager, |node| node.set_trait_flags(value));
        true
    }

    pub fn set_state_flags(
        &mut self,
        value: u32,
        mut manager: Option<&mut SemanticManager>,
        artboard: &mut ArtboardInstance,
    ) -> bool {
        if self.state_flags == value {
            return false;
        }
        let hidden_changed = (self.state_flags ^ value) & SemanticState::HIDDEN.0 != 0;
        self.state_flags = value;
        self.update_node_content(manager.as_deref_mut(), |node| node.set_state_flags(value));
        if hidden_changed && let Some(manager) = manager {
            self.sync_tree_visibility(manager, artboard);
        }
        true
    }

    fn update_node_content<F>(&mut self, manager: Option<&mut SemanticManager>, update: F)
    where
        F: FnOnce(&mut crate::SemanticNode),
    {
        let Some(node) = &self.semantic_node else {
            return;
        };
        update(&mut node.borrow_mut());
        if let Some(manager) = manager {
            manager.mark_node_dirty(node.borrow().id(), SemanticDirt::CONTENT);
        }
    }

    pub fn apply_inferred_semantics_if_needed(
        &mut self,
        artboard: &ArtboardInstance,
        manager: Option<&mut SemanticManager>,
    ) {
        if self.role != 0 || !self.label.is_empty() {
            return;
        }
        let (Some(node), Some(parent_local)) = (&self.semantic_node, self.parent_local_id) else {
            return;
        };
        let mut inferred = ResolvedSemanticData::default();
        if !crate::semantic_inference_registry::resolve_inferred_semantics(
            artboard,
            parent_local,
            &mut inferred,
        ) {
            return;
        }
        if node.borrow().role() == inferred.role && node.borrow().label() == inferred.label {
            return;
        }
        {
            let mut node = node.borrow_mut();
            node.set_role(inferred.role);
            node.set_label(inferred.label);
        }
        if let Some(manager) = manager {
            manager.mark_node_dirty(node.borrow().id(), SemanticDirt::CONTENT);
        }
    }

    pub fn update_world_bounds(
        &mut self,
        artboard: &mut ArtboardInstance,
        manager: Option<&mut SemanticManager>,
    ) {
        let (Some(node), Some(parent_local)) = (&self.semantic_node, self.parent_local_id) else {
            return;
        };
        let bounds = SemanticProvider::semantic_bounds(artboard, parent_local);
        if bounds.is_empty_or_nan() && self.bounds_retry_pending {
            artboard.add_dirt(
                self.local_id,
                crate::components::ComponentDirt::WORLD_TRANSFORM,
                false,
            );
            return;
        }
        self.bounds_retry_pending = false;
        if node.borrow().bounds() == bounds {
            return;
        }
        node.borrow_mut().set_bounds(bounds);
        if let Some(manager) = manager {
            manager.mark_node_dirty(node.borrow().id(), SemanticDirt::BOUNDS);
        }
    }

    fn should_exclude_from_tree(&self, artboard: &ArtboardInstance) -> bool {
        if has_semantic_state(self.state_flags, SemanticState::HIDDEN) {
            return true;
        }
        let Some(parent_local) = self.parent_local_id else {
            return true;
        };
        if artboard.component(parent_local).is_none() {
            return true;
        }
        artboard.runtime_component_is_collapsed_for_draw(parent_local)
    }

    pub fn sync_tree_visibility(
        &mut self,
        manager: &mut SemanticManager,
        artboard: &mut ArtboardInstance,
    ) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        let exclude = self.should_exclude_from_tree(artboard);
        if exclude == self.excluded_from_tree {
            return;
        }
        self.excluded_from_tree = exclude;
        if exclude {
            if node.borrow().manager_identity() == Some(manager.identity()) {
                manager.remove_child(node);
            }
            self.semantic_manager_identity = Some(manager.identity());
            return;
        }
        if node.borrow().manager_identity().is_none()
            && self.semantic_manager_identity == Some(manager.identity())
        {
            manager.add_child(self.tree_parent.as_ref(), node.clone());
            self.bounds_retry_pending = true;
            self.update_world_bounds(artboard, Some(manager));
            self.apply_inferred_semantics_if_needed(artboard, Some(manager));
        }
    }

    pub fn add_semantic_listener(&mut self, listener: Rc<dyn SemanticListener>) {
        self.semantic_listeners.push(listener);
    }

    pub fn remove_semantic_listener(&mut self, listener: &Rc<dyn SemanticListener>) {
        if let Some(index) = self
            .semantic_listeners
            .iter()
            .position(|candidate| Rc::ptr_eq(candidate, listener))
        {
            self.semantic_listeners.remove(index);
        }
    }

    pub fn fire(&self, action: SemanticActionType) {
        for listener in &self.semantic_listeners {
            match action {
                SemanticActionType::Tap => listener.on_semantic_tap(),
                SemanticActionType::Increase => listener.on_semantic_increase(),
                SemanticActionType::Decrease => listener.on_semantic_decrease(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[derive(Debug, Default)]
    struct CountingListener {
        tap: Cell<usize>,
        increase: Cell<usize>,
        decrease: Cell<usize>,
    }

    impl SemanticListener for CountingListener {
        fn on_semantic_tap(&self) {
            self.tap.set(self.tap.get() + 1);
        }

        fn on_semantic_increase(&self) {
            self.increase.set(self.increase.get() + 1);
        }

        fn on_semantic_decrease(&self) {
            self.decrease.set(self.decrease.get() + 1);
        }
    }

    #[test]
    fn listeners_preserve_duplicates_remove_first_and_dispatch_exact_action() {
        let mut data = RuntimeSemanticData::new(1, Some(0));
        let listener = Rc::new(CountingListener::default());
        let erased: Rc<dyn SemanticListener> = listener.clone();
        data.add_semantic_listener(erased.clone());
        data.add_semantic_listener(erased.clone());
        data.fire(SemanticActionType::Tap);
        assert_eq!(listener.tap.get(), 2);
        assert_eq!(listener.increase.get(), 0);
        data.remove_semantic_listener(&erased);
        data.fire(SemanticActionType::Increase);
        assert_eq!(listener.increase.get(), 1);
        data.fire(SemanticActionType::Decrease);
        assert_eq!(listener.decrease.get(), 1);
    }

    #[test]
    fn focused_state_preserves_other_bits_and_is_noop_before_node_creation() {
        let mut data = RuntimeSemanticData::new(1, Some(0));
        data.set_focused_state(true, None);
        assert!(!data.has_semantic_node());
        let node = SemanticNodeHandle::new(1);
        node.borrow_mut().set_state_flags(SemanticState::SELECTED.0);
        data.semantic_node = Some(node.clone());
        data.set_focused_state(true, None);
        assert_eq!(
            node.borrow().state_flags(),
            SemanticState::SELECTED.0 | SemanticState::FOCUSED.0
        );
        data.set_focused_state(false, None);
        assert_eq!(node.borrow().state_flags(), SemanticState::SELECTED.0);
    }

    #[test]
    fn generated_style_setters_early_out_and_preserve_node_identity() {
        let mut data = RuntimeSemanticData::new(1, Some(0));
        let node = SemanticNodeHandle::new(1);
        data.semantic_node = Some(node.clone());
        assert!(data.set_label("hello", None));
        assert!(!data.set_label("hello", None));
        assert!(data.node_handle().is_some_and(|same| same.ptr_eq(&node)));
        assert_eq!(node.borrow().label(), "hello");
    }
}
