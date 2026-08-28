use crate::mechanical_port::source::{
    artboard::Artboard,
    component::Component,
    component_dirt::ComponentDirt,
    core::{CoreHandle, CoreObject},
    focus_data::FocusData,
    generated::{focus_data_base::FocusDataBase, semantic::semantic_data_base::SemanticDataBase},
    semantic::{
        semantic_dirt::SemanticDirt,
        semantic_inference_registry::resolve_inferred_semantics,
        semantic_listener::SemanticListener,
        semantic_manager::RuntimeSemanticManagerHandle,
        semantic_node::{SemanticNode, SemanticNodeRef},
        semantic_provider::semantic_bounds,
        semantic_state::SemanticState,
        semantic_trait::SemanticTrait,
    },
};
use std::{rc::Rc, vec::Vec};

pub type SemanticListenerRef = Rc<dyn SemanticListener>;

macro_rules! semantic_trait_flag {
    ($get:ident, $set:ident, $mask:ident, $property:ident) => {
        pub fn $get(&self) -> bool {
            self.base.trait_flags() & SemanticDataBase::$mask != 0
        }
        pub fn $set(&mut self, value: bool) {
            let flags = if value {
                self.base.trait_flags() | SemanticDataBase::$mask
            } else {
                self.base.trait_flags() & !SemanticDataBase::$mask
            };
            if !self.base.set_trait_flags_value(flags) {
                return;
            }
            self.trait_flags_changed();
            self.notify_property_changed(SemanticDataBase::$property);
        }
    };
}

macro_rules! semantic_state_flag {
    ($get:ident, $set:ident, $mask:ident, $property:ident) => {
        pub fn $get(&self) -> bool {
            self.base.state_flags() & SemanticDataBase::$mask != 0
        }
        pub fn $set(&mut self, value: bool) {
            let flags = if value {
                self.base.state_flags() | SemanticDataBase::$mask
            } else {
                self.base.state_flags() & !SemanticDataBase::$mask
            };
            if !self.base.set_state_flags_value(flags) {
                return;
            }
            self.state_flags_changed();
            self.notify_property_changed(SemanticDataBase::$property);
        }
    };
}

pub struct SemanticData {
    pub base: SemanticDataBase,
    semantic_node: Option<SemanticNodeRef>,
    semantic_manager: Option<RuntimeSemanticManagerHandle>,
    semantic_listeners: Vec<SemanticListenerRef>,
    bounds_retry_pending: bool,
    excluded_from_tree: bool,
}

impl Default for SemanticData {
    fn default() -> Self {
        Self {
            base: SemanticDataBase::default(),
            semantic_node: None,
            semantic_manager: None,
            semantic_listeners: Vec::new(),
            bounds_retry_pending: false,
            excluded_from_tree: false,
        }
    }
}

impl SemanticData {
    fn component(&self) -> &Component {
        &self.base.base
    }

    fn component_mut(&mut self) -> &mut Component {
        &mut self.base.base
    }

    fn self_handle(&self) -> Option<CoreHandle> {
        CoreObject::core(self).handle()
    }

    pub fn existing_semantic_node(&self) -> Option<SemanticNodeRef> {
        self.semantic_node.clone()
    }

    pub fn has_semantic_node(&self) -> bool {
        self.semantic_node.is_some()
    }

    pub fn is_collapsed(&self) -> bool {
        self.component().is_collapsed()
    }

    pub fn manager_is(&self, manager: &RuntimeSemanticManagerHandle) -> bool {
        self.semantic_node
            .as_ref()
            .and_then(|node| node.borrow().manager())
            .as_ref()
            .is_some_and(|current| current.ptr_eq(manager))
    }

    pub fn find_parent_semantic_data_handle(&self) -> Option<CoreHandle> {
        let this = self.self_handle();
        let mut current = self.component().parent_handle();
        while let Some(component) = current {
            let found = component
                .with(|component| {
                    component.as_node()?.children().iter().find_map(|child| {
                        if this.as_ref() == Some(child) {
                            return None;
                        }
                        child
                            .with(|child| child.as_semantic_data().is_some())
                            .unwrap_or(false)
                            .then(|| child.clone())
                    })
                })
                .flatten();
            if found.is_some() {
                return found;
            }
            current = component
                .with(|component| component.component_parent_handle())
                .flatten();
        }
        None
    }

    pub fn find_closest_semantic_node_handle(start: Option<CoreHandle>) -> Option<SemanticNodeRef> {
        let mut current = start;
        while let Some(component) = current {
            if component.is_type_of(
                crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
            ) {
                let host_component = component
                    .with_downcast::<Artboard, _>(|artboard| artboard.host())
                    .flatten()
                    .and_then(|host| {
                        host.with(|host| host.as_artboard_host()?.host_component())
                            .flatten()
                    });
                if let Some(host_component) = host_component
                    && host_component
                        .with(|host| host.as_container_component().is_some())
                        .unwrap_or(false)
                {
                    current = Some(host_component);
                    continue;
                }
                return None;
            }

            let semantic_data = component
                .with(|component| {
                    component.as_node()?.children().iter().find_map(|child| {
                        child
                            .with(|child| child.as_semantic_data().is_some())
                            .unwrap_or(false)
                            .then(|| child.clone())
                    })
                })
                .flatten();
            if let Some(semantic_data) = semantic_data {
                return semantic_data
                    .with_mut(|semantic_data| {
                        semantic_data
                            .as_semantic_data_mut()
                            .map(SemanticData::semantic_node)
                    })
                    .flatten();
            }
            current = component
                .with(|component| component.component_parent_handle())
                .flatten();
        }
        None
    }

    fn sibling_focus_data(&self) -> Option<CoreHandle> {
        self.component().parent_handle().and_then(|parent| {
            parent
                .with(|parent| {
                    parent.as_node()?.children().iter().find_map(|child| {
                        child
                            .is_type_of(FocusDataBase::TYPE_KEY)
                            .then(|| child.clone())
                    })
                })
                .flatten()
        })
    }

    pub fn semantic_node(&mut self) -> SemanticNodeRef {
        if let Some(node) = &self.semantic_node {
            return node.clone();
        }

        let node = SemanticNode::new(0);
        {
            let mut node = node.borrow_mut();
            node.core_owner = self.component().parent_handle();
            node.semantic_data = self.self_handle();
            node.role = self.base.role();
            node.label = self.base.label().to_owned();
            node.value = self.base.value().to_owned();
            node.hint = self.base.hint().to_owned();
            node.heading_level = self.base.heading_level();
            node.state_flags = self.base.state_flags();
            node.trait_flags = self.base.trait_flags()
                | if self.sibling_focus_data().is_some() {
                    SemanticTrait::FOCUSABLE.0
                } else {
                    0
                };
        }
        self.semantic_node = Some(node.clone());
        self.apply_inferred_semantics_if_needed();
        self.bounds_retry_pending = true;
        self.update_world_bounds();
        node
    }

    pub fn semantic_id(&self) -> u32 {
        self.semantic_node
            .as_ref()
            .map_or(0, |node| node.borrow().id())
    }

    pub fn set_focused_state(&mut self, focused: bool) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        let mut node = node.borrow_mut();
        if focused {
            node.state_flags |= SemanticState::FOCUSED.0;
        } else {
            node.state_flags &= !SemanticState::FOCUSED.0;
        }
        drop(node);
        self.mark_content_dirty();
    }

    pub fn request_focus(&mut self) -> bool {
        let Some(focus_data) = self.sibling_focus_data() else {
            return false;
        };
        let Some(focus_node) = focus_data.with_downcast_mut::<FocusData, _>(FocusData::focus_node)
        else {
            return false;
        };
        let Some(focus_manager) = self
            .component()
            .with_artboard(Artboard::focus_manager_handle)
            .flatten()
        else {
            return false;
        };
        focus_manager.with_focus_manager_mut(|manager| manager.set_focus(focus_node));
        true
    }

    fn resolve_parent_node(&mut self) -> Option<SemanticNodeRef> {
        if let Some(parent) = self.find_parent_semantic_data_handle() {
            let parent = parent
                .with_mut(|parent| {
                    parent
                        .as_semantic_data_mut()
                        .map(SemanticData::semantic_node)
                })
                .flatten();
            if parent.is_some() {
                return parent;
            }
        }
        self.component()
            .with_artboard(Artboard::semantic_boundary_node)
            .flatten()
    }

    pub fn register_with_manager(
        &mut self,
        manager: RuntimeSemanticManagerHandle,
        fallback_parent: Option<SemanticNodeRef>,
    ) -> SemanticNodeRef {
        let parent = self.resolve_parent_node().or(fallback_parent);
        let node = self.semantic_node();
        manager.add_child(parent, node.clone());
        self.sync_semantic_tree_visibility();
        node
    }

    pub fn detach_if_managed_by(&mut self, manager: &RuntimeSemanticManagerHandle) {
        let Some(node) = self.semantic_node.as_ref() else {
            return;
        };
        if node
            .borrow()
            .manager()
            .as_ref()
            .is_some_and(|current| current.ptr_eq(manager))
        {
            manager.remove_child(node);
        }
    }

    pub fn collapse_after_component(&mut self, value: bool) {
        let Some(node) = self.semantic_node.clone() else {
            return;
        };
        if value {
            let manager = { node.borrow().manager() };
            if let Some(manager) = manager {
                self.semantic_manager = Some(manager.clone());
                manager.remove_child(&node);
            }
            return;
        }

        let manager = self
            .semantic_manager
            .clone()
            .or_else(|| node.borrow().manager());
        let Some(manager) = manager else {
            return;
        };
        self.semantic_manager = Some(manager.clone());
        let parent = self.resolve_parent_node();
        manager.add_child(parent, node);
        self.bounds_retry_pending = true;
        self.update_world_bounds();
        self.component_mut()
            .add_dirt(ComponentDirt::WORLD_TRANSFORM, false);
        self.apply_inferred_semantics_if_needed();
    }

    pub fn build_dependencies(&mut self) {
        self.component_mut().build_dependencies();
        let (Some(parent), Some(this)) = (self.component().parent_handle(), self.self_handle())
        else {
            return;
        };
        parent.with_mut(|parent| {
            parent.component_add_dependent(this);
        });
    }

    pub fn update(&mut self, value: ComponentDirt) {
        if Component::has_dirt_in(value, ComponentDirt::COLLAPSED) {
            self.sync_semantic_tree_visibility();
        }
        if Component::has_dirt_in(value, ComponentDirt::WORLD_TRANSFORM) {
            self.apply_inferred_semantics_if_needed();
            self.update_world_bounds();
        }
        if Component::has_dirt_in(value, ComponentDirt::PATH) {
            self.update_world_bounds();
        }
    }

    fn apply_inferred_semantics_if_needed(&mut self) {
        if self.base.role() != 0 || !self.base.label().is_empty() || self.semantic_node.is_none() {
            return;
        }
        let parent = self.component().parent_handle();
        let mut inferred = Default::default();
        if !resolve_inferred_semantics(parent.as_ref(), &mut inferred) {
            return;
        }
        let Some(node) = &self.semantic_node else {
            return;
        };
        if node.borrow().role == inferred.role && node.borrow().label == inferred.label {
            return;
        }
        let mut node = node.borrow_mut();
        node.role = inferred.role;
        node.label = inferred.label;
        drop(node);
        self.mark_content_dirty();
    }

    fn mark_content_dirty(&self) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        let Some(manager) = node.borrow().manager() else {
            return;
        };
        if self.component().parent_handle().is_none() {
            return;
        }
        manager.with_semantic_manager_mut(|manager| {
            manager.mark_node_dirty(self.semantic_id(), SemanticDirt::CONTENT)
        });
    }

    fn should_exclude_from_semantic_tree(&self) -> bool {
        if self.base.state_flags() & SemanticState::HIDDEN.0 != 0 {
            return true;
        }
        let Some(parent) = self.component().parent_handle() else {
            return true;
        };
        parent
            .with(|parent| {
                parent.as_component().is_some_and(Component::is_collapsed)
                    || parent
                        .as_drawable()
                        .is_some_and(|drawable| drawable.is_hidden())
            })
            .unwrap_or(true)
    }

    pub fn sync_semantic_tree_visibility(&mut self) {
        let Some(node) = self.semantic_node.clone() else {
            return;
        };
        let should_exclude = self.should_exclude_from_semantic_tree();
        if should_exclude == self.excluded_from_tree {
            return;
        }
        self.excluded_from_tree = should_exclude;
        let manager = node.borrow().manager();
        if should_exclude {
            if let Some(manager) = manager {
                self.semantic_manager = Some(manager.clone());
                manager.remove_child(&node);
            }
            return;
        }
        if manager.is_some() {
            return;
        }
        let Some(manager) = self.semantic_manager.clone() else {
            return;
        };
        let parent = self.resolve_parent_node();
        manager.add_child(parent, node);
        self.bounds_retry_pending = true;
        self.update_world_bounds();
        self.apply_inferred_semantics_if_needed();
    }

    fn update_world_bounds(&mut self) {
        if self.semantic_node.is_none() {
            return;
        }
        let Some(parent) = self.component().parent_handle() else {
            return;
        };
        let bounds = semantic_bounds(Some(&parent));
        if bounds.is_empty_or_nan() && self.bounds_retry_pending {
            self.component_mut()
                .add_dirt(ComponentDirt::WORLD_TRANSFORM, false);
            return;
        }
        self.bounds_retry_pending = false;
        let Some(node) = &self.semantic_node else {
            return;
        };
        if node.borrow().bounds == bounds {
            return;
        }
        node.borrow_mut().bounds = bounds;
        let manager = { node.borrow().manager() };
        if let Some(manager) = manager {
            manager.with_semantic_manager_mut(|manager| {
                manager.mark_node_dirty(self.semantic_id(), SemanticDirt::BOUNDS)
            });
        }
    }

    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    pub fn role_changed(&mut self) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        if node.borrow().role == self.base.role() {
            return;
        }
        node.borrow_mut().role = self.base.role();
        self.mark_content_dirty();
    }

    pub fn label_changed(&mut self) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        if node.borrow().label == self.base.label() {
            return;
        }
        node.borrow_mut().label = self.base.label().to_owned();
        self.mark_content_dirty();
    }

    pub fn value_changed(&mut self) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        if node.borrow().value == self.base.value() {
            return;
        }
        node.borrow_mut().value = self.base.value().to_owned();
        self.mark_content_dirty();
    }

    pub fn hint_changed(&mut self) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        if node.borrow().hint == self.base.hint() {
            return;
        }
        node.borrow_mut().hint = self.base.hint().to_owned();
        self.mark_content_dirty();
    }

    pub fn heading_level_changed(&mut self) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        if node.borrow().heading_level == self.base.heading_level() {
            return;
        }
        node.borrow_mut().heading_level = self.base.heading_level();
        self.mark_content_dirty();
    }

    pub fn trait_flags_changed(&mut self) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        if node.borrow().trait_flags == self.base.trait_flags() {
            return;
        }
        node.borrow_mut().trait_flags = self.base.trait_flags();
        self.mark_content_dirty();
    }

    pub fn state_flags_changed(&mut self) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        if node.borrow().state_flags == self.base.state_flags() {
            return;
        }
        node.borrow_mut().state_flags = self.base.state_flags();
        self.mark_content_dirty();
        self.sync_semantic_tree_visibility();
    }

    pub fn set_role(&mut self, value: u32) {
        if self.base.set_role_value(value) {
            self.role_changed();
            self.notify_property_changed(SemanticDataBase::ROLE_PROPERTY_KEY);
        }
    }

    pub fn set_label(&mut self, value: String) {
        if self.base.set_label_value(value) {
            self.label_changed();
            self.notify_property_changed(SemanticDataBase::LABEL_PROPERTY_KEY);
        }
    }

    pub fn set_value(&mut self, value: String) {
        if self.base.set_value_value(value) {
            self.value_changed();
            self.notify_property_changed(SemanticDataBase::VALUE_PROPERTY_KEY);
        }
    }

    pub fn set_hint(&mut self, value: String) {
        if self.base.set_hint_value(value) {
            self.hint_changed();
            self.notify_property_changed(SemanticDataBase::HINT_PROPERTY_KEY);
        }
    }

    pub fn set_heading_level(&mut self, value: u32) {
        if self.base.set_heading_level_value(value) {
            self.heading_level_changed();
            self.notify_property_changed(SemanticDataBase::HEADING_LEVEL_PROPERTY_KEY);
        }
    }

    pub fn set_trait_flags(&mut self, value: u32) {
        if self.base.set_trait_flags_value(value) {
            self.trait_flags_changed();
            self.notify_property_changed(SemanticDataBase::TRAIT_FLAGS_PROPERTY_KEY);
        }
    }

    pub fn set_state_flags(&mut self, value: u32) {
        if self.base.set_state_flags_value(value) {
            self.state_flags_changed();
            self.notify_property_changed(SemanticDataBase::STATE_FLAGS_PROPERTY_KEY);
        }
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

    pub fn add_semantic_listener(&mut self, listener: SemanticListenerRef) {
        self.semantic_listeners.push(listener);
    }

    pub fn remove_semantic_listener(&mut self, listener: &SemanticListenerRef) {
        if let Some(index) = self
            .semantic_listeners
            .iter()
            .position(|current| Rc::ptr_eq(current, listener))
        {
            self.semantic_listeners.remove(index);
        }
    }

    pub fn fire_semantic_tap(&self) {
        for listener in &self.semantic_listeners {
            listener.on_semantic_tap();
        }
    }

    pub fn fire_semantic_increase(&self) {
        for listener in &self.semantic_listeners {
            listener.on_semantic_increase();
        }
    }

    pub fn fire_semantic_decrease(&self) {
        for listener in &self.semantic_listeners {
            listener.on_semantic_decrease();
        }
    }
}

impl Drop for SemanticData {
    fn drop(&mut self) {
        let Some(node) = self.semantic_node.as_ref() else {
            return;
        };
        let manager = { node.borrow().manager() };
        if let Some(manager) = manager {
            manager.remove_child(node);
        }
    }
}
