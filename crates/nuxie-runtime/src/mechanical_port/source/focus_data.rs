use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{
    animation::{
        focus_listener_group::RuntimeFocusListenerGroupWeakHandle,
        keyboard_listener_group::RuntimeKeyboardListenerGroupWeakHandle,
        listener_invocation::ListenerInvocation,
        state_machine_instance::{RuntimeObjectHandle, RuntimeStateMachineInstanceWeakHandle},
    },
    artboard::Artboard,
    component::Component,
    component_dirt::ComponentDirt,
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    core::CoreHandle,
    generated::{
        component_base::ComponentBaseCallbacks,
        focus_data_base::{FocusDataBase, FocusDataBaseCallbacks},
    },
    input::{
        focus_node::{EdgeBehavior, FocusNode, FocusNodeRef, FocusableRef},
        focusable::{Focusable, Key, KeyModifiers},
    },
    layout_component::LayoutComponent,
    math::{aabb::Aabb, vec2d::Vec2D},
    parent_traversal::ParentTraversal,
    semantic::{semantic_data::SemanticData, semantic_snapshot::Bounds},
};

#[derive(Clone)]
pub struct RuntimeFocusListenerHandle {
    group: RuntimeFocusListenerGroupWeakHandle,
}

impl RuntimeFocusListenerHandle {
    pub fn new(group: RuntimeFocusListenerGroupWeakHandle) -> Self {
        Self { group }
    }

    fn is_alive(&self) -> bool {
        self.group.upgrade().is_some()
    }

    fn ptr_eq(&self, other: &Self) -> bool {
        self.group.ptr_eq(&other.group)
    }

    fn notify(&self, focused: bool) {
        if let Some(group) = self.group.upgrade() {
            group.with_group_mut(|group| {
                if focused {
                    group.on_focused();
                } else {
                    group.on_blurred();
                }
            });
        }
    }
}

#[derive(Clone)]
pub struct RuntimeKeyboardListenerHandle {
    group: RuntimeKeyboardListenerGroupWeakHandle,
}

impl RuntimeKeyboardListenerHandle {
    pub fn new(group: RuntimeKeyboardListenerGroupWeakHandle) -> Self {
        Self { group }
    }

    fn is_alive(&self) -> bool {
        self.group.upgrade().is_some()
    }

    fn ptr_eq(&self, other: &Self) -> bool {
        self.group.ptr_eq(&other.group)
    }

    fn key_input(
        &self,
        key: Key,
        modifiers: KeyModifiers,
        is_pressed: bool,
        is_repeat: bool,
    ) -> bool {
        self.group
            .upgrade()
            .map(|group| {
                group.with_group_mut(|group| group.key_input(key, modifiers, is_pressed, is_repeat))
            })
            .unwrap_or(false)
    }

    fn text_input(&self, text: &str) -> bool {
        self.group
            .upgrade()
            .map(|group| group.with_group_mut(|group| group.text_input(text)))
            .unwrap_or(false)
    }
}

#[derive(Clone)]
pub struct RuntimeGamepadListenerHandle {
    machine: RuntimeStateMachineInstanceWeakHandle,
    group: RuntimeObjectHandle,
}

impl RuntimeGamepadListenerHandle {
    pub fn new(machine: RuntimeStateMachineInstanceWeakHandle, group: RuntimeObjectHandle) -> Self {
        Self { machine, group }
    }

    fn is_alive(&self) -> bool {
        self.machine.upgrade().is_some()
    }

    fn ptr_eq(&self, other: &Self) -> bool {
        self.group == other.group && self.machine.ptr_eq(&other.machine)
    }

    fn dispatch(
        &self,
        invocation: &ListenerInvocation,
        out_dispatched_scripted_drawable: Option<&mut Option<CoreHandle>>,
    ) -> bool {
        self.machine
            .with_instance_mut(|machine| {
                machine.dispatch_gamepad_listener_group(
                    self.group,
                    invocation,
                    out_dispatched_scripted_drawable,
                )
            })
            .unwrap_or(false)
    }
}

struct FocusDataFocusable {
    owner: Option<CoreHandle>,
}

fn component_allows_focus_traversal(
    component: &crate::mechanical_port::source::core::CoreHandle,
) -> bool {
    component
        .with(|component| {
            let Some(base) = component.as_component() else {
                return false;
            };
            if base.is_collapsed() {
                return false;
            }
            if component
                .as_drawable()
                .is_some_and(|drawable| drawable.is_hidden())
            {
                return false;
            }
            if component
                .as_transform_component()
                .is_some_and(|component| component.render_opacity() <= 0.0)
            {
                return false;
            }
            true
        })
        .unwrap_or(false)
}

impl FocusData {
    pub fn is_eligible_for_focus_traversal(&self) -> bool {
        if self.component().is_collapsed() {
            return false;
        }
        let Some(this) = crate::mechanical_port::source::core::CoreObject::core(self).handle()
        else {
            return true;
        };
        let mut traversal = ParentTraversal::new(Some(this));
        loop {
            let Some(parent) = traversal.next() else {
                break;
            };
            if !component_allows_focus_traversal(&parent) {
                return false;
            }
            if traversal.did_cross_boundary() {
                if let Some(host) = traversal.crossing_host() {
                    let allowed = host
                        .with(|host| {
                            let host = host.as_artboard_host()?;
                            let component = host.host_component()?;
                            Some(
                                component_allows_focus_traversal(&component)
                                    && !component
                                        .with(|component| {
                                            component.nested_artboard_is_paused().unwrap_or(false)
                                        })
                                        .unwrap_or(false),
                            )
                        })
                        .flatten()
                        .unwrap_or(false);
                    if !allowed {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn world_position(&self, out_position: &mut Vec2D) -> bool {
        let Some(local_position) = self
            .component()
            .parent_handle()
            .and_then(|parent| {
                parent.with(|parent| {
                    parent
                        .as_world_transform_component()
                        .map(|component| component.world_translation())
                })
            })
            .flatten()
        else {
            return false;
        };
        *out_position = self
            .component()
            .with_artboard_mut(|artboard| artboard.root_transform(local_position))
            .unwrap_or(local_position);
        true
    }

    pub fn accepts_keyboard_input(&self) -> bool {
        self.keyboard_listeners
            .iter()
            .any(RuntimeKeyboardListenerHandle::is_alive)
            || self
                .text_input_listeners
                .iter()
                .any(RuntimeKeyboardListenerHandle::is_alive)
    }

    pub fn focusable_artboard(&self) -> Option<CoreHandle> {
        self.component().artboard_handle()
    }

    pub fn name_changed(&mut self) {
        if let Some(node) = &self.focus_node {
            node.borrow_mut().name = self.base.base.base.name().to_owned();
        }
    }

    pub fn build_dependencies(&mut self) {
        self.component_mut().build_dependencies();
        let Some(this) = crate::mechanical_port::source::core::CoreObject::core(self).handle()
        else {
            return;
        };
        if let Some(parent) = self.component().parent_handle() {
            parent.with_mut(|parent| parent.component_add_dependent(this));
        }
    }

    pub fn update(&mut self, value: ComponentDirt) {
        if value.contains(ComponentDirt::WORLD_TRANSFORM) {
            self.update_world_bounds();
        }
    }

    fn update_world_bounds(&mut self) {
        let Some(node) = self.focus_node.clone() else {
            return;
        };
        let layout = self.component().parent_handle().and_then(|parent| {
            parent
                .with(|parent| {
                    parent
                        .as_layout_component()
                        .map(LayoutComponent::world_bounds)
                })
                .flatten()
        });
        if let Some(mut bounds) = layout {
            if let Some((minimum, maximum)) = self.component().with_artboard_mut(|artboard| {
                (
                    artboard.root_transform(Vec2D::new(bounds.left(), bounds.top())),
                    artboard.root_transform(Vec2D::new(bounds.right(), bounds.bottom())),
                )
            }) {
                bounds = Aabb::new(minimum.x, minimum.y, maximum.x, maximum.y);
            }
            node.borrow_mut().world_bounds = Bounds {
                min_x: bounds.left(),
                min_y: bounds.top(),
                max_x: bounds.right(),
                max_y: bounds.bottom(),
            };
        } else {
            node.borrow_mut().clear_world_bounds();
        }
    }
}

impl Focusable for FocusData {
    fn key_input(
        &mut self,
        key: Key,
        modifiers: KeyModifiers,
        is_pressed: bool,
        is_repeat: bool,
    ) -> bool {
        FocusData::key_input(self, key, modifiers, is_pressed, is_repeat)
    }

    fn text_input(&mut self, text: &str) -> bool {
        FocusData::text_input(self, text)
    }

    fn gamepad_dispatch(
        &mut self,
        invocation: &ListenerInvocation,
        out_dispatched_scripted_drawable: Option<&mut Option<CoreHandle>>,
    ) -> bool {
        FocusData::gamepad_dispatch(self, invocation, out_dispatched_scripted_drawable)
    }

    fn focused(&mut self) {
        FocusData::focused(self);
    }

    fn blurred(&mut self) {
        FocusData::blurred(self);
    }

    fn world_position(&self) -> Option<(f32, f32)> {
        let mut position = Vec2D::default();
        FocusData::world_position(self, &mut position).then_some((position.x, position.y))
    }

    fn is_eligible_for_focus_traversal(&self) -> bool {
        FocusData::is_eligible_for_focus_traversal(self)
    }

    fn accepts_keyboard_input(&self) -> bool {
        FocusData::accepts_keyboard_input(self)
    }
}

impl FocusDataBaseCallbacks for FocusData {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn focus_flags_changed(&mut self) {
        FocusData::focus_flags_changed(self);
    }

    fn edge_behavior_value_changed(&mut self) {
        FocusData::edge_behavior_value_changed(self);
    }
}

impl ComponentBaseCallbacks for FocusData {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn name_changed(&mut self) {
        FocusData::name_changed(self);
    }
}

impl Focusable for FocusDataFocusable {
    fn key_input(
        &mut self,
        key: Key,
        modifiers: KeyModifiers,
        is_pressed: bool,
        is_repeat: bool,
    ) -> bool {
        self.owner
            .as_ref()
            .and_then(|owner| {
                owner.with_downcast_mut::<FocusData, _>(|owner| {
                    owner.key_input(key, modifiers, is_pressed, is_repeat)
                })
            })
            .unwrap_or(false)
    }

    fn text_input(&mut self, text: &str) -> bool {
        self.owner
            .as_ref()
            .and_then(|owner| {
                owner.with_downcast_mut::<FocusData, _>(|owner| owner.text_input(text))
            })
            .unwrap_or(false)
    }

    fn gamepad_dispatch(
        &mut self,
        invocation: &ListenerInvocation,
        out_dispatched_scripted_drawable: Option<&mut Option<CoreHandle>>,
    ) -> bool {
        self.owner
            .as_ref()
            .and_then(|owner| {
                owner.with_downcast_mut::<FocusData, _>(|owner| {
                    owner.gamepad_dispatch(invocation, out_dispatched_scripted_drawable)
                })
            })
            .unwrap_or(false)
    }

    fn focused(&mut self) {
        if let Some(owner) = &self.owner {
            owner.with_downcast_mut::<FocusData, _>(FocusData::focused);
        }
    }

    fn blurred(&mut self) {
        if let Some(owner) = &self.owner {
            owner.with_downcast_mut::<FocusData, _>(FocusData::blurred);
        }
    }

    fn world_position(&self) -> Option<(f32, f32)> {
        let mut position = Vec2D::default();
        self.owner
            .as_ref()
            .and_then(|owner| {
                owner.with_downcast::<FocusData, _>(|owner| owner.world_position(&mut position))
            })
            .unwrap_or(false)
            .then_some((position.x, position.y))
    }

    fn is_eligible_for_focus_traversal(&self) -> bool {
        self.owner
            .as_ref()
            .and_then(|owner| {
                owner.with_downcast::<FocusData, _>(FocusData::is_eligible_for_focus_traversal)
            })
            .unwrap_or(false)
    }

    fn accepts_keyboard_input(&self) -> bool {
        self.owner
            .as_ref()
            .and_then(|owner| {
                owner.with_downcast::<FocusData, _>(FocusData::accepts_keyboard_input)
            })
            .unwrap_or(false)
    }
}

pub struct FocusData {
    pub base: FocusDataBase,
    focus_node: Option<FocusNodeRef>,
    focus_listeners: Vec<RuntimeFocusListenerHandle>,
    keyboard_listeners: Vec<RuntimeKeyboardListenerHandle>,
    text_input_listeners: Vec<RuntimeKeyboardListenerHandle>,
    gamepad_listeners: Vec<RuntimeGamepadListenerHandle>,
}

impl Default for FocusData {
    fn default() -> Self {
        Self {
            base: FocusDataBase::default(),
            focus_node: None,
            focus_listeners: Vec::new(),
            keyboard_listeners: Vec::new(),
            text_input_listeners: Vec::new(),
            gamepad_listeners: Vec::new(),
        }
    }
}

impl Drop for FocusData {
    fn drop(&mut self) {
        let Some(node) = self.focus_node.take() else {
            return;
        };
        node.borrow_mut().clear_focusable();
        let parent = node.borrow().parent();
        let manager = self
            .component()
            .with_artboard(Artboard::focus_manager_handle)
            .flatten();
        if let Some(manager) = manager {
            manager.with_focus_manager_mut(|manager| manager.remove_child(&node));
        } else if let Some(parent) = parent {
            FocusNode::remove_child(&parent, &node);
        }
    }
}

impl FocusData {
    pub const TYPE_KEY: u16 = FocusDataBase::TYPE_KEY;

    fn component(&self) -> &Component {
        &self.base.base
    }

    fn component_mut(&mut self) -> &mut Component {
        &mut self.base.base
    }

    pub fn focus_node(&mut self) -> FocusNodeRef {
        if self.focus_node.is_none() {
            let owner = crate::mechanical_port::source::core::CoreObject::core(self).handle();
            let focusable: FocusableRef = Rc::new(RefCell::new(FocusDataFocusable { owner }));
            let node = FocusNode::new(Some(focusable));
            {
                let mut node_ref = node.borrow_mut();
                node_ref
                    .set_can_focus(self.base.focus_flags() & FocusDataBase::CAN_FOCUS_BITMASK != 0);
                node_ref
                    .set_can_touch(self.base.focus_flags() & FocusDataBase::CAN_TOUCH_BITMASK != 0);
                node_ref.set_can_traverse(
                    self.base.focus_flags() & FocusDataBase::CAN_TRAVERSE_BITMASK != 0,
                );
                node_ref.set_edge_behavior(edge_behavior(self.base.edge_behavior_value()));
                node_ref.name = self.base.base.base.name().to_owned();
            }
            self.focus_node = Some(node);
            self.update_world_bounds();
        }
        self.focus_node.as_ref().unwrap().clone()
    }

    pub fn existing_focus_node(&self) -> Option<FocusNodeRef> {
        self.focus_node.clone()
    }

    pub fn add_focus_listener(&mut self, listener: RuntimeFocusListenerHandle) {
        self.focus_listeners.push(listener);
    }

    pub fn remove_focus_listener(&mut self, listener: RuntimeFocusListenerHandle) {
        if let Some(index) = self
            .focus_listeners
            .iter()
            .position(|value| value.ptr_eq(&listener))
        {
            self.focus_listeners.remove(index);
        }
    }

    pub fn add_keyboard_listener(&mut self, listener: RuntimeKeyboardListenerHandle) {
        self.keyboard_listeners.push(listener);
    }

    pub fn remove_keyboard_listener(&mut self, listener: RuntimeKeyboardListenerHandle) {
        if let Some(index) = self
            .keyboard_listeners
            .iter()
            .position(|value| value.ptr_eq(&listener))
        {
            self.keyboard_listeners.remove(index);
        }
    }

    pub fn add_text_input_listener(&mut self, listener: RuntimeKeyboardListenerHandle) {
        self.text_input_listeners.push(listener);
    }

    pub fn remove_text_input_listener(&mut self, listener: RuntimeKeyboardListenerHandle) {
        if let Some(index) = self
            .text_input_listeners
            .iter()
            .position(|value| value.ptr_eq(&listener))
        {
            self.text_input_listeners.remove(index);
        }
    }

    pub fn add_gamepad_listener(&mut self, listener: RuntimeGamepadListenerHandle) {
        self.gamepad_listeners.push(listener);
    }

    pub fn remove_gamepad_listener(&mut self, listener: RuntimeGamepadListenerHandle) {
        if let Some(index) = self
            .gamepad_listeners
            .iter()
            .position(|value| value.ptr_eq(&listener))
        {
            self.gamepad_listeners.remove(index);
        }
    }

    pub fn focus(&mut self) {}

    pub fn key_input(
        &mut self,
        value: Key,
        modifiers: KeyModifiers,
        is_pressed: bool,
        is_repeat: bool,
    ) -> bool {
        self.keyboard_listeners
            .retain(RuntimeKeyboardListenerHandle::is_alive);
        for listener in &self.keyboard_listeners {
            if listener.key_input(value, modifiers, is_pressed, is_repeat) {
                return true;
            }
        }
        false
    }

    pub fn text_input(&mut self, text: &str) -> bool {
        self.text_input_listeners
            .retain(RuntimeKeyboardListenerHandle::is_alive);
        for listener in &self.text_input_listeners {
            if listener.text_input(text) {
                return true;
            }
        }
        false
    }

    pub fn gamepad_dispatch(
        &mut self,
        invocation: &ListenerInvocation,
        out_dispatched_scripted_drawable: Option<&mut Option<CoreHandle>>,
    ) -> bool {
        self.gamepad_listeners
            .retain(RuntimeGamepadListenerHandle::is_alive);
        let mut output = out_dispatched_scripted_drawable;
        for listener in &self.gamepad_listeners {
            if listener.dispatch(invocation, output.as_deref_mut()) {
                return true;
            }
        }
        false
    }
}

fn edge_behavior(value: u32) -> EdgeBehavior {
    match value {
        1 => EdgeBehavior::ClosedLoop,
        2 => EdgeBehavior::Stop,
        _ => EdgeBehavior::ParentScope,
    }
}

fn find_sibling_semantic_data(component: &Component) -> Option<CoreHandle> {
    component
        .parent_handle()?
        .with(|parent| {
            parent
                .as_node()?
                .children()
                .iter()
                .find(|child| {
                    child
                        .with(|child| child.as_semantic_data().is_some())
                        .unwrap_or(false)
                })
                .cloned()
        })
        .flatten()
}

impl FocusData {
    fn scroll_into_view(&mut self) {
        let mut layout_component = self.component().parent_handle();
        while let Some(component) = layout_component.clone() {
            if component
                .with(|component| component.as_layout_component().is_some())
                .unwrap_or(false)
            {
                break;
            }
            layout_component = component
                .with(|component| component.as_component()?.parent_handle())
                .flatten();
        }
        let Some(layout_component) = layout_component else {
            return;
        };
        let Some(mut element_bounds) = layout_component
            .with(|component| {
                component
                    .as_layout_component()
                    .map(LayoutComponent::world_bounds)
            })
            .flatten()
        else {
            return;
        };

        let Some(this) = crate::mechanical_port::source::core::CoreObject::core(self).handle()
        else {
            return;
        };
        let mut traversal = ParentTraversal::new(Some(this));
        while let Some(parent) = traversal.next() {
            if traversal.did_cross_boundary() {
                if let (Some(host), Some(source_artboard)) =
                    (traversal.crossing_host(), traversal.source_artboard())
                {
                    if let Some(host_transform) = host
                        .with(|host| {
                            host.as_artboard_host().map(|host| {
                                host.world_transform_for_artboard(source_artboard.clone())
                            })
                        })
                        .flatten()
                    {
                        let minimum = host_transform
                            * Vec2D::new(element_bounds.left(), element_bounds.top());
                        let maximum = host_transform
                            * Vec2D::new(element_bounds.right(), element_bounds.bottom());
                        element_bounds = Aabb::new(minimum.x, minimum.y, maximum.x, maximum.y);
                    }
                }
            }
            let constraints = parent
                .with(|parent| {
                    parent
                        .as_transform_component()
                        .map(|component| component.constraints().to_vec())
                })
                .flatten();
            let Some(constraints) = constraints else {
                continue;
            };
            for constraint in constraints {
                constraint.with_downcast_mut::<ScrollConstraint, _>(|scroll_constraint| {
                    self.scroll_constraint_to_show_bounds(scroll_constraint, element_bounds);
                });
            }
        }
    }

    fn scroll_constraint_to_show_bounds(
        &mut self,
        constraint: &mut ScrollConstraint,
        element_bounds: Aabb,
    ) {
        let Some(_content) = constraint.content_handle() else {
            return;
        };
        let Some(viewport) = constraint.viewport_handle() else {
            return;
        };
        let Some(viewport_transform) = viewport
            .with(|viewport| {
                viewport
                    .as_layout_component()
                    .map(|viewport| *viewport.world_transform())
            })
            .flatten()
        else {
            return;
        };
        let viewport_world_x = viewport_transform[4];
        let viewport_world_y = viewport_transform[5];
        let viewport_width = constraint.viewport_width();
        let viewport_height = constraint.viewport_height();
        let viewport_left = element_bounds.left() - viewport_world_x;
        let viewport_top = element_bounds.top() - viewport_world_y;
        let viewport_right = element_bounds.right() - viewport_world_x;
        let viewport_bottom = element_bounds.bottom() - viewport_world_y;
        let effective_scroll_x = constraint.effective_scroll_offset_x();
        let effective_scroll_y = constraint.effective_scroll_offset_y();
        let mut delta_x = 0.0;
        let mut delta_y = 0.0;
        if constraint.base.constrains_horizontal() {
            let element_width = viewport_right - viewport_left;
            if element_width > viewport_width {
                delta_x = -viewport_left;
            } else if viewport_left < 0.0 {
                delta_x = -viewport_left;
            } else if viewport_right > viewport_width {
                delta_x = -(viewport_right - viewport_width);
            }
        }
        if constraint.base.constrains_vertical() {
            let element_height = viewport_bottom - viewport_top;
            if element_height > viewport_height {
                delta_y = -viewport_top;
            } else if viewport_top < 0.0 {
                delta_y = -viewport_top;
            } else if viewport_bottom > viewport_height {
                delta_y = -(viewport_bottom - viewport_height);
            }
        }
        if delta_x != 0.0 || delta_y != 0.0 {
            let current = Vec2D::new(effective_scroll_x, effective_scroll_y);
            let target = Vec2D::new(effective_scroll_x + delta_x, effective_scroll_y + delta_y);
            let snapped = constraint.nearest_snap_offset_in_direction(current, target);
            constraint.scroll_to_position(snapped.x, snapped.y);
        }
    }

    pub fn focused(&mut self) {
        self.scroll_into_view();
        self.focus_listeners
            .retain(RuntimeFocusListenerHandle::is_alive);
        for listener in &self.focus_listeners {
            listener.notify(true);
        }
        if let Some(sibling) = find_sibling_semantic_data(self.component()) {
            sibling.with_downcast_mut::<SemanticData, _>(|sibling| sibling.set_focused_state(true));
        }
        if let Some(parent) = self.component().parent_handle() {
            parent.with_mut(|parent| {
                if let Some(text_input) = parent.as_text_input_mut() {
                    text_input.focused();
                }
            });
        }
    }

    pub fn blurred(&mut self) {
        self.focus_listeners
            .retain(RuntimeFocusListenerHandle::is_alive);
        for listener in &self.focus_listeners {
            listener.notify(false);
        }
        if let Some(sibling) = find_sibling_semantic_data(self.component()) {
            sibling
                .with_downcast_mut::<SemanticData, _>(|sibling| sibling.set_focused_state(false));
        }
        if let Some(parent) = self.component().parent_handle() {
            parent.with_mut(|parent| {
                if let Some(text_input) = parent.as_text_input_mut() {
                    text_input.blurred();
                }
            });
        }
    }

    pub fn focus_flags_changed(&mut self) {
        if let Some(node) = &self.focus_node {
            let mut node = node.borrow_mut();
            node.set_can_focus(self.base.focus_flags() & FocusDataBase::CAN_FOCUS_BITMASK != 0);
            node.set_can_touch(self.base.focus_flags() & FocusDataBase::CAN_TOUCH_BITMASK != 0);
            node.set_can_traverse(
                self.base.focus_flags() & FocusDataBase::CAN_TRAVERSE_BITMASK != 0,
            );
        }
    }

    pub fn edge_behavior_value_changed(&mut self) {
        if let Some(node) = &self.focus_node {
            node.borrow_mut()
                .set_edge_behavior(edge_behavior(self.base.edge_behavior_value()));
        }
    }

    pub fn find_parent_focus_data(&self) -> Option<CoreHandle> {
        let this = crate::mechanical_port::source::core::CoreObject::core(self).handle();
        let mut current = self.component().parent_handle();
        while let Some(parent) = current {
            let children = parent
                .with(|parent| parent.as_node().map(|parent| parent.children().to_vec()))
                .flatten()
                .unwrap_or_default();
            for child in children {
                if Some(&child) != this.as_ref()
                    && child.with_downcast::<FocusData, _>(|_| ()).is_some()
                {
                    return Some(child);
                }
            }
            current = parent
                .with(|parent| parent.as_component()?.parent_handle())
                .flatten();
        }
        None
    }

    pub fn find_closest_focus_node_handle(
        component: crate::mechanical_port::source::core::CoreHandle,
    ) -> Option<FocusNodeRef> {
        let mut current = component
            .with(|component| component.as_component()?.parent_handle())
            .flatten();
        while let Some(parent) = current {
            if parent.with_downcast::<Artboard, _>(|_| ()).is_some() {
                if let Some(host) = parent
                    .with_downcast::<Artboard, _>(Artboard::host)
                    .flatten()
                {
                    let host_component = host
                        .with(|host| host.as_artboard_host()?.host_component())
                        .flatten();
                    if host_component.as_ref().is_some_and(|component| {
                        component
                            .with(|component| component.as_container_component().is_some())
                            .unwrap_or(false)
                    }) {
                        current = host_component;
                        continue;
                    }
                }
                #[cfg(feature = "tools")]
                if let Some(external_node) = parent
                    .with_downcast::<Artboard, _>(Artboard::external_parent_focus_node)
                    .flatten()
                {
                    return Some(external_node);
                }
                return None;
            }
            let children = parent
                .with(|parent| parent.as_node().map(|node| node.children().to_vec()))
                .flatten()
                .unwrap_or_default();
            for child in children {
                if let Some(node) = child.with_downcast_mut::<FocusData, _>(FocusData::focus_node) {
                    return Some(node);
                }
            }
            current = parent
                .with(|parent| parent.as_component()?.parent_handle())
                .flatten();
        }
        None
    }
}

impl std::ops::Deref for FocusData {
    type Target = FocusDataBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FocusData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
