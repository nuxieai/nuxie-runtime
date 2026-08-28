use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::{
        component_base::ComponentBaseCallbacks,
        semantic::semantic_data_base::{SemanticDataBase, SemanticDataBaseCallbacks},
    },
    semantic::{
        semantic_dirt::SemanticDirt,
        semantic_listener::SemanticListener,
        semantic_manager::SemanticManager,
        semantic_node::{SemanticNode, SemanticNodeRef},
        semantic_snapshot::Bounds,
        semantic_state::SemanticState,
        semantic_trait::SemanticTrait,
    },
};
use std::{cell::RefCell, rc::Rc};

pub type SemanticManagerRef = Rc<RefCell<SemanticManager>>;
pub type SemanticListenerRef = Rc<dyn SemanticListener>;
macro_rules! semantic_trait_flag {
    ($get:ident, $set:ident, $mask:ident, $property:ident) => {
        pub fn $get(&self) -> bool {
            self.base.trait_flags() & SemanticDataBase::$mask != 0
        }
        pub fn $set(&mut self, value: bool) {
            self.set_trait_bit(SemanticDataBase::$mask, SemanticDataBase::$property, value);
        }
    };
}
macro_rules! semantic_state_flag {
    ($get:ident, $set:ident, $mask:ident, $property:ident) => {
        pub fn $get(&self) -> bool {
            self.base.state_flags() & SemanticDataBase::$mask != 0
        }
        pub fn $set(&mut self, value: bool) {
            self.set_state_bit(SemanticDataBase::$mask, SemanticDataBase::$property, value);
        }
    };
}
struct SilentSemanticDataCallbacks;
impl ComponentBaseCallbacks for SilentSemanticDataCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}
impl SemanticDataBaseCallbacks for SilentSemanticDataCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}
pub struct SemanticData {
    pub base: SemanticDataBase,
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
            base: SemanticDataBase::default(),
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
    pub fn existing_semantic_node(&self) -> Option<SemanticNodeRef> {
        self.semantic_node.clone()
    }

    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    pub fn manager_is(&self, manager: &SemanticManagerRef) -> bool {
        self.semantic_manager
            .as_ref()
            .is_some_and(|current| Rc::ptr_eq(current, manager))
    }

    pub fn find_closest_semantic_node_handle(start: Option<CoreHandle>) -> Option<SemanticNodeRef> {
        let mut current = start;
        while let Some(component) = current {
            let direct_semantic = component
                .with(|component| {
                    if let Some(semantic) = component.as_semantic_data() {
                        return semantic.existing_semantic_node();
                    }
                    component.as_container_component().and_then(|container| {
                        container.children().iter().find_map(|child| {
                            child
                                .with(|child| {
                                    child
                                        .as_semantic_data()
                                        .and_then(SemanticData::existing_semantic_node)
                                })
                                .flatten()
                        })
                    })
                })
                .flatten();
            if direct_semantic.is_some() {
                return direct_semantic;
            }
            current = component
                .with(|component| component.component_parent_handle())
                .flatten();
        }
        None
    }

    fn set_trait_bit(&mut self, mask: u32, property_key: u16, value: bool) {
        let flags = if value {
            self.base.trait_flags() | mask
        } else {
            self.base.trait_flags() & !mask
        };
        if flags == self.base.trait_flags() {
            return;
        }
        let mut callbacks = SilentSemanticDataCallbacks;
        self.base.set_trait_flags(flags, &mut callbacks);
        if let Some(node) = &self.semantic_node {
            node.borrow_mut().trait_flags = flags;
            self.mark_content_dirty();
        }
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn set_state_bit(&mut self, mask: u32, property_key: u16, value: bool) {
        let flags = if value {
            self.base.state_flags() | mask
        } else {
            self.base.state_flags() & !mask
        };
        if flags == self.base.state_flags() {
            return;
        }
        let mut callbacks = SilentSemanticDataCallbacks;
        self.base.set_state_flags(flags, &mut callbacks);
        if let Some(node) = &self.semantic_node {
            node.borrow_mut().state_flags = flags;
            self.mark_content_dirty();
        }
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    semantic_trait_flag!(
        is_expandable,
        set_is_expandable,
        IS_EXPANDABLE_BITMASK,
        IS_EXPANDABLE_PROPERTY_KEY
    );
    semantic_trait_flag!(
        is_selectable,
        set_is_selectable,
        IS_SELECTABLE_BITMASK,
        IS_SELECTABLE_PROPERTY_KEY
    );
    semantic_trait_flag!(
        is_checkable,
        set_is_checkable,
        IS_CHECKABLE_BITMASK,
        IS_CHECKABLE_PROPERTY_KEY
    );
    semantic_trait_flag!(
        is_toggleable,
        set_is_toggleable,
        IS_TOGGLEABLE_BITMASK,
        IS_TOGGLEABLE_PROPERTY_KEY
    );
    semantic_trait_flag!(
        is_requirable,
        set_is_requirable,
        IS_REQUIRABLE_BITMASK,
        IS_REQUIRABLE_PROPERTY_KEY
    );
    semantic_trait_flag!(
        is_enablable,
        set_is_enablable,
        IS_ENABLABLE_BITMASK,
        IS_ENABLABLE_PROPERTY_KEY
    );
    semantic_trait_flag!(
        is_focusable,
        set_is_focusable,
        IS_FOCUSABLE_BITMASK,
        IS_FOCUSABLE_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_expanded,
        set_is_expanded,
        IS_EXPANDED_BITMASK,
        IS_EXPANDED_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_selected,
        set_is_selected,
        IS_SELECTED_BITMASK,
        IS_SELECTED_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_checked,
        set_is_checked,
        IS_CHECKED_BITMASK,
        IS_CHECKED_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_mixed,
        set_is_mixed,
        IS_MIXED_BITMASK,
        IS_MIXED_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_toggled,
        set_is_toggled,
        IS_TOGGLED_BITMASK,
        IS_TOGGLED_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_required,
        set_is_required,
        IS_REQUIRED_BITMASK,
        IS_REQUIRED_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_disabled,
        set_is_disabled,
        IS_DISABLED_BITMASK,
        IS_DISABLED_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_focused,
        set_is_focused,
        IS_FOCUSED_BITMASK,
        IS_FOCUSED_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_hidden,
        set_is_hidden,
        IS_HIDDEN_BITMASK,
        IS_HIDDEN_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_live_region,
        set_is_live_region,
        IS_LIVE_REGION_BITMASK,
        IS_LIVE_REGION_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_read_only,
        set_is_read_only,
        IS_READ_ONLY_BITMASK,
        IS_READ_ONLY_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_modal,
        set_is_modal,
        IS_MODAL_BITMASK,
        IS_MODAL_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_obscured,
        set_is_obscured,
        IS_OBSCURED_BITMASK,
        IS_OBSCURED_PROPERTY_KEY
    );
    semantic_state_flag!(
        is_multiline,
        set_is_multiline,
        IS_MULTILINE_BITMASK,
        IS_MULTILINE_PROPERTY_KEY
    );

    pub fn semantic_node(
        &mut self,
        has_focus_data: bool,
        core_owner: Option<CoreHandle>,
        semantic_data: CoreHandle,
        initial_bounds: Bounds,
    ) -> SemanticNodeRef {
        if self.semantic_node.is_none() {
            let node = SemanticNode::new(0);
            {
                let mut n = node.borrow_mut();
                n.core_owner = core_owner;
                n.semantic_data = Some(semantic_data);
                n.role = self.base.role();
                n.label = self.base.label().to_owned();
                n.value = self.base.value().to_owned();
                n.hint = self.base.hint().to_owned();
                n.heading_level = self.base.heading_level();
                n.state_flags = self.base.state_flags();
                n.trait_flags = self.base.trait_flags()
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
        let exclude = self.base.state_flags() & SemanticState::HIDDEN.0 != 0
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
        if self.base.role() == v {
            return;
        }
        let mut callbacks = SilentSemanticDataCallbacks;
        self.base.set_role(v, &mut callbacks);
        if let Some(n) = &self.semantic_node {
            if n.borrow().role != v {
                n.borrow_mut().role = v;
                self.mark_content_dirty();
            }
        }
        self.base
            .base
            .base
            .base
            .notify_property_changed(SemanticDataBase::ROLE_PROPERTY_KEY);
    }
    pub fn set_label(&mut self, v: String) {
        if self.base.label() == v {
            return;
        }
        let mut callbacks = SilentSemanticDataCallbacks;
        self.base.set_label(v.clone(), &mut callbacks);
        if let Some(n) = &self.semantic_node {
            if n.borrow().label != v {
                n.borrow_mut().label = v;
                self.mark_content_dirty();
            }
        }
        self.base
            .base
            .base
            .base
            .notify_property_changed(SemanticDataBase::LABEL_PROPERTY_KEY);
    }
    pub fn set_value(&mut self, v: String) {
        if self.base.value() == v {
            return;
        }
        let mut callbacks = SilentSemanticDataCallbacks;
        self.base.set_value(v.clone(), &mut callbacks);
        if let Some(n) = &self.semantic_node {
            if n.borrow().value != v {
                n.borrow_mut().value = v;
                self.mark_content_dirty();
            }
        }
        self.base
            .base
            .base
            .base
            .notify_property_changed(SemanticDataBase::VALUE_PROPERTY_KEY);
    }
    pub fn set_hint(&mut self, v: String) {
        if self.base.hint() == v {
            return;
        }
        let mut callbacks = SilentSemanticDataCallbacks;
        self.base.set_hint(v.clone(), &mut callbacks);
        if let Some(n) = &self.semantic_node {
            if n.borrow().hint != v {
                n.borrow_mut().hint = v;
                self.mark_content_dirty();
            }
        }
        self.base
            .base
            .base
            .base
            .notify_property_changed(SemanticDataBase::HINT_PROPERTY_KEY);
    }
    pub fn set_heading_level(&mut self, v: u32) {
        if self.base.heading_level() == v {
            return;
        }
        let mut callbacks = SilentSemanticDataCallbacks;
        self.base.set_heading_level(v, &mut callbacks);
        if let Some(n) = &self.semantic_node {
            if n.borrow().heading_level != v {
                n.borrow_mut().heading_level = v;
                self.mark_content_dirty();
            }
        }
        self.base
            .base
            .base
            .base
            .notify_property_changed(SemanticDataBase::HEADING_LEVEL_PROPERTY_KEY);
    }
    pub fn set_state_flags(&mut self, v: u32) {
        if self.base.state_flags() == v {
            return;
        }
        let mut callbacks = SilentSemanticDataCallbacks;
        self.base.set_state_flags(v, &mut callbacks);
        if let Some(n) = &self.semantic_node {
            if n.borrow().state_flags != v {
                n.borrow_mut().state_flags = v;
                self.mark_content_dirty();
            }
        }
        self.base
            .base
            .base
            .base
            .notify_property_changed(SemanticDataBase::STATE_FLAGS_PROPERTY_KEY);
    }
    pub fn set_trait_flags(&mut self, v: u32) {
        if self.base.trait_flags() == v {
            return;
        }
        let mut callbacks = SilentSemanticDataCallbacks;
        self.base.set_trait_flags(v, &mut callbacks);
        if let Some(n) = &self.semantic_node {
            if n.borrow().trait_flags != v {
                n.borrow_mut().trait_flags = v;
                self.mark_content_dirty();
            }
        }
        self.base
            .base
            .base
            .base
            .notify_property_changed(SemanticDataBase::TRAIT_FLAGS_PROPERTY_KEY);
    }
    pub fn apply_inferred_semantics(&mut self, role: u32, label: String) {
        if self.base.role() != 0 || !self.base.label().is_empty() {
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
            l.on_semantic_tap();
        }
    }
    pub fn fire_semantic_increase(&mut self) {
        for l in &self.listeners {
            l.on_semantic_increase();
        }
    }
    pub fn fire_semantic_decrease(&mut self) {
        for l in &self.listeners {
            l.on_semantic_decrease();
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
