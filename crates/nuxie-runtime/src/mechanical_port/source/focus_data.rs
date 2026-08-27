use std::{any::Any, cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation,
    artboard::{Artboard, ArtboardInstance},
    artboard_host::ArtboardHost,
    component::Component,
    component_dirt::ComponentDirt,
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    generated::{
        component_base::ComponentBaseCallbacks,
        focus_data_base::{FocusDataBase, FocusDataBaseCallbacks},
    },
    input::{
        focus_listener::FocusListener,
        focus_manager::FocusManager,
        focus_node::{EdgeBehavior, FocusNode, FocusNodeRef, FocusableRef},
        focusable::{Focusable, Key, KeyModifiers},
        gamepad_listener::GamepadListener,
        keyboard_listener::KeyboardListener,
    },
    layout_component::LayoutComponent,
    math::{aabb::Aabb, vec2d::Vec2D},
    parent_traversal::ParentTraversal,
    scripted::scripted_drawable::ScriptedDrawable,
    semantic::{semantic_data::SemanticData, semantic_snapshot::Bounds},
};

type KeyboardListenerObject = dyn KeyboardListener<Key, KeyModifiers>;
type GamepadListenerObject = dyn GamepadListener<ListenerInvocation, ScriptedDrawable>;

struct FocusDataFocusable {
    owner: *mut FocusData,
}

fn component_allows_focus_traversal(component: *const Component) -> bool {
    let Some(component) = (unsafe { component.as_ref() }) else {
        return false;
    };
    if component.is_collapsed() {
        return false;
    }
    if let Some(drawable) = component.as_drawable() {
        if drawable.is_hidden() {
            return false;
        }
    }
    if let Some(transform_component) = component.as_transform_component() {
        if transform_component.render_opacity() <= 0.0 {
            return false;
        }
    }
    true
}

impl FocusData {
    pub fn is_eligible_for_focus_traversal(&self) -> bool {
        if self.component().is_collapsed() {
            return false;
        }
        let Some(start) = self.component().parent() else {
            return true;
        };
        let start_component = &start.base.base as *const Component;
        if !component_allows_focus_traversal(start_component) {
            return false;
        }
        let mut traversal =
            ParentTraversal::new(Some(unsafe { &mut *(start_component as *mut Component) }));
        loop {
            let Some(parent) = traversal.next() else {
                break;
            };
            if !component_allows_focus_traversal(&parent.base.base) {
                return false;
            }
            if traversal.did_cross_boundary() {
                if let Some(host) = traversal.crossing_host() {
                    let host = host as *const dyn ArtboardHost as *mut dyn ArtboardHost;
                    if let Some(host_component) = unsafe { &mut *host }.host_component() {
                        if !component_allows_focus_traversal(host_component) {
                            return false;
                        }
                        if let Some(nested_artboard) = host_component.as_nested_artboard() {
                            if nested_artboard.base.is_paused() {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    }

    pub fn world_position(&mut self, out_position: &mut Vec2D) -> bool {
        let Some(parent) = self.component_mut().parent_mut() else {
            return false;
        };
        let Some(world_transform_component) = parent.base.base.as_world_transform_component_mut()
        else {
            return false;
        };
        let local_position = world_transform_component.world_translation();
        *out_position = self
            .component_mut()
            .artboard_mut()
            .map_or(local_position, |artboard| {
                artboard.root_transform(local_position)
            });
        true
    }

    pub fn accepts_keyboard_input(&self) -> bool {
        !self.keyboard_listeners.is_empty() || !self.text_input_listeners.is_empty()
    }

    pub fn focusable_artboard(&mut self) -> Option<&mut Artboard> {
        self.component_mut().artboard_mut()
    }

    pub fn name_changed(&mut self) {
        if let Some(node) = &self.focus_node {
            node.borrow_mut().name = self.base.base.base.name().to_owned();
        }
    }

    pub fn build_dependencies(&mut self) {
        self.component_mut().build_dependencies();
        let this = self.component_mut() as *mut Component;
        if let Some(parent) = self.component_mut().parent_mut() {
            parent.base.base.add_dependent(this);
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
        let layout = self
            .component_mut()
            .parent_mut()
            .and_then(|parent| parent.base.base.as_layout_component_mut())
            .map(|layout| layout.world_bounds());
        if let Some(mut bounds) = layout {
            if let Some(artboard) = self.component_mut().artboard_mut() {
                let minimum = artboard.root_transform(Vec2D::new(bounds.left(), bounds.top()));
                let maximum = artboard.root_transform(Vec2D::new(bounds.right(), bounds.bottom()));
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

    fn gamepad_dispatch(&mut self, invocation: &dyn Any) -> bool {
        invocation
            .downcast_ref::<ListenerInvocation>()
            .is_some_and(|invocation| FocusData::gamepad_dispatch(self, invocation, None))
    }

    fn focused(&mut self) {
        FocusData::focused(self);
    }

    fn blurred(&mut self) {
        FocusData::blurred(self);
    }

    fn world_position(&self) -> Option<(f32, f32)> {
        let this = self as *const Self as *mut Self;
        let mut position = Vec2D::default();
        unsafe { &mut *this }
            .world_position(&mut position)
            .then_some((position.x, position.y))
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
        unsafe { &mut *self.owner }.key_input(key, modifiers, is_pressed, is_repeat)
    }

    fn text_input(&mut self, text: &str) -> bool {
        unsafe { &mut *self.owner }.text_input(text)
    }

    fn gamepad_dispatch(&mut self, invocation: &dyn Any) -> bool {
        invocation
            .downcast_ref::<ListenerInvocation>()
            .is_some_and(|invocation| {
                unsafe { &mut *self.owner }.gamepad_dispatch(invocation, None)
            })
    }

    fn focused(&mut self) {
        unsafe { &mut *self.owner }.focused();
    }

    fn blurred(&mut self) {
        unsafe { &mut *self.owner }.blurred();
    }

    fn world_position(&self) -> Option<(f32, f32)> {
        let mut position = Vec2D::default();
        unsafe { &mut *self.owner }
            .world_position(&mut position)
            .then_some((position.x, position.y))
    }

    fn is_eligible_for_focus_traversal(&self) -> bool {
        unsafe { &*self.owner }.is_eligible_for_focus_traversal()
    }

    fn accepts_keyboard_input(&self) -> bool {
        unsafe { &*self.owner }.accepts_keyboard_input()
    }
}

pub struct FocusData {
    pub base: FocusDataBase,
    focus_node: Option<FocusNodeRef>,
    focus_listeners: Vec<*mut dyn FocusListener>,
    keyboard_listeners: Vec<*mut KeyboardListenerObject>,
    text_input_listeners: Vec<*mut KeyboardListenerObject>,
    gamepad_listeners: Vec<*mut GamepadListenerObject>,
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
        node.borrow_mut().focusable = None;
        if let Some(parent) = node.borrow().parent() {
            FocusNode::remove_child(&parent, &node);
            return;
        }
        let manager = self
            .component()
            .artboard()
            .and_then(Artboard::focus_manager)
            .map_or(std::ptr::null(), |value| value);
        if !manager.is_null() {
            unsafe { &mut *(manager as *mut FocusManager) }.remove_child(&node);
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
            let focusable: FocusableRef = Rc::new(RefCell::new(FocusDataFocusable {
                owner: self as *mut FocusData,
            }));
            let node = FocusNode::new(Some(focusable));
            {
                let mut node_ref = node.borrow_mut();
                node_ref.can_focus(self.base.focus_flags() & FocusDataBase::CAN_FOCUS_BITMASK != 0);
                node_ref.can_touch(self.base.focus_flags() & FocusDataBase::CAN_TOUCH_BITMASK != 0);
                node_ref.can_traverse(
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

    pub fn add_focus_listener(&mut self, listener: *mut dyn FocusListener) {
        self.focus_listeners.push(listener);
    }

    pub fn remove_focus_listener(&mut self, listener: *mut dyn FocusListener) {
        if let Some(index) = self
            .focus_listeners
            .iter()
            .position(|value| std::ptr::addr_eq(*value, listener))
        {
            self.focus_listeners.remove(index);
        }
    }

    pub fn add_keyboard_listener(&mut self, listener: *mut KeyboardListenerObject) {
        self.keyboard_listeners.push(listener);
    }

    pub fn remove_keyboard_listener(&mut self, listener: *mut KeyboardListenerObject) {
        if let Some(index) = self
            .keyboard_listeners
            .iter()
            .position(|value| std::ptr::addr_eq(*value, listener))
        {
            self.keyboard_listeners.remove(index);
        }
    }

    pub fn add_text_input_listener(&mut self, listener: *mut KeyboardListenerObject) {
        self.text_input_listeners.push(listener);
    }

    pub fn remove_text_input_listener(&mut self, listener: *mut KeyboardListenerObject) {
        if let Some(index) = self
            .text_input_listeners
            .iter()
            .position(|value| std::ptr::addr_eq(*value, listener))
        {
            self.text_input_listeners.remove(index);
        }
    }

    pub fn add_gamepad_listener(&mut self, listener: *mut GamepadListenerObject) {
        self.gamepad_listeners.push(listener);
    }

    pub fn remove_gamepad_listener(&mut self, listener: *mut GamepadListenerObject) {
        if let Some(index) = self
            .gamepad_listeners
            .iter()
            .position(|value| std::ptr::addr_eq(*value, listener))
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
        for listener in self.keyboard_listeners.iter().copied() {
            if unsafe { &mut *listener }.key_input(value, modifiers, is_pressed, is_repeat) {
                return true;
            }
        }
        false
    }

    pub fn text_input(&mut self, text: &str) -> bool {
        for listener in self.text_input_listeners.iter().copied() {
            if unsafe { &mut *listener }.text_input(text) {
                return true;
            }
        }
        false
    }

    pub fn gamepad_dispatch(
        &mut self,
        invocation: &ListenerInvocation,
        out_dispatched_scripted_drawable: Option<&mut Option<*mut ScriptedDrawable>>,
    ) -> bool {
        let output = out_dispatched_scripted_drawable
            .map(|value| value as *mut Option<*mut ScriptedDrawable>);
        for listener in self.gamepad_listeners.iter().copied() {
            let output = output.map(|value| unsafe { &mut *value });
            if unsafe { &mut *listener }.gamepad_dispatch(invocation, output) {
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

fn find_sibling_semantic_data(component: &mut Component) -> Option<&mut SemanticData> {
    let parent = component.parent_mut()?;
    parent.base.base.as_node_mut()?;
    for child in parent.children() {
        if let Some(semantic_data) = unsafe { &mut **child }.as_semantic_data_mut() {
            return Some(semantic_data);
        }
    }
    None
}

impl FocusData {
    fn scroll_into_view(&mut self) {
        let mut layout_component = self
            .component_mut()
            .parent_mut()
            .map(|value| value as *mut _);
        while let Some(component) = layout_component {
            if unsafe { &mut *component }
                .base
                .base
                .as_layout_component_mut()
                .is_some()
            {
                break;
            }
            layout_component = unsafe { &mut *component }
                .base
                .base
                .parent_mut()
                .map(|value| value as *mut _);
        }
        let Some(layout_component) = layout_component else {
            return;
        };
        let mut element_bounds = unsafe { &mut *layout_component }
            .base
            .base
            .as_layout_component_mut()
            .unwrap()
            .world_bounds();

        let this = self.component_mut() as *mut Component;
        let mut traversal = ParentTraversal::new(Some(unsafe { &mut *this }));
        while let Some(parent) = traversal.next() {
            let parent = parent as *mut _;
            if traversal.did_cross_boundary() {
                if let (Some(host), Some(source_artboard)) =
                    (traversal.crossing_host(), traversal.source_artboard())
                {
                    let instance = if source_artboard.is_instance() {
                        source_artboard as *const Artboard as *mut ArtboardInstance
                    } else {
                        std::ptr::null_mut()
                    };
                    let host_transform = host.world_transform_for_artboard(instance);
                    let minimum =
                        host_transform * Vec2D::new(element_bounds.left(), element_bounds.top());
                    let maximum = host_transform
                        * Vec2D::new(element_bounds.right(), element_bounds.bottom());
                    element_bounds = Aabb::new(minimum.x, minimum.y, maximum.x, maximum.y);
                }
            }
            let parent_component = unsafe { &mut *parent }
                .base
                .base
                .as_transform_component_mut();
            let Some(transform_component) = parent_component else {
                continue;
            };
            let constraints = transform_component.constraints().to_vec();
            for constraint in constraints {
                let component = unsafe { &mut *constraint }.as_component_mut_ptr();
                let Some(scroll_constraint) =
                    (unsafe { &mut *component }).as_scroll_constraint_mut()
                else {
                    continue;
                };
                self.scroll_constraint_to_show_bounds(scroll_constraint, element_bounds);
            }
        }
    }

    fn scroll_constraint_to_show_bounds(
        &mut self,
        constraint: &mut ScrollConstraint,
        element_bounds: Aabb,
    ) {
        let content = constraint.content() as *const LayoutComponent;
        let viewport = constraint.viewport() as *const LayoutComponent;
        if content.is_null() || viewport.is_null() {
            return;
        }
        let viewport_transform = unsafe { &*viewport }.world_transform();
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
        for listener in self.focus_listeners.iter().copied() {
            unsafe { &mut *listener }.on_focused();
        }
        if let Some(sibling) = find_sibling_semantic_data(self.component_mut()) {
            sibling.set_focused_state(true);
        }
        if let Some(parent) = self.component_mut().parent_mut() {
            if let Some(text_input) = parent.base.base.as_text_input_mut() {
                text_input.focused();
            }
        }
    }

    pub fn blurred(&mut self) {
        for listener in self.focus_listeners.iter().copied() {
            unsafe { &mut *listener }.on_blurred();
        }
        if let Some(sibling) = find_sibling_semantic_data(self.component_mut()) {
            sibling.set_focused_state(false);
        }
        if let Some(parent) = self.component_mut().parent_mut() {
            if let Some(text_input) = parent.base.base.as_text_input_mut() {
                text_input.blurred();
            }
        }
    }

    pub fn focus_flags_changed(&mut self) {
        if let Some(node) = &self.focus_node {
            let mut node = node.borrow_mut();
            node.can_focus(self.base.focus_flags() & FocusDataBase::CAN_FOCUS_BITMASK != 0);
            node.can_touch(self.base.focus_flags() & FocusDataBase::CAN_TOUCH_BITMASK != 0);
            node.can_traverse(self.base.focus_flags() & FocusDataBase::CAN_TRAVERSE_BITMASK != 0);
        }
    }

    pub fn edge_behavior_value_changed(&mut self) {
        if let Some(node) = &self.focus_node {
            node.borrow_mut()
                .set_edge_behavior(edge_behavior(self.base.edge_behavior_value()));
        }
    }

    pub fn find_parent_focus_data(&self) -> Option<&mut FocusData> {
        let mut current = self
            .component()
            .parent()
            .map(|value| value as *const _ as *mut _);
        while let Some(parent) = current {
            let parent_ref = unsafe { &mut *parent };
            if parent_ref.base.base.as_node_mut().is_some() {
                for child in parent_ref.children() {
                    if let Some(focus_data) = unsafe { &mut **child }.as_focus_data_mut() {
                        if !std::ptr::eq(focus_data, self) {
                            return Some(focus_data);
                        }
                    }
                }
            }
            current = parent_ref
                .base
                .base
                .parent_mut()
                .map(|value| value as *mut _);
        }
        None
    }

    pub fn find_closest_focus_node(component: *mut Component) -> Option<FocusNodeRef> {
        let Some(component) = (unsafe { component.as_mut() }) else {
            return None;
        };
        let mut current = component.parent_mut().map(|value| value as *mut _);
        while let Some(parent) = current {
            let parent_ref = unsafe { &mut *parent };
            if let Some(artboard) = parent_ref.base.base.as_artboard_mut() {
                if let Some(host) = artboard.host() {
                    if let Some(host_component) = unsafe { &mut *host }.host_component() {
                        if let Some(container) = host_component.as_container_component_mut() {
                            current = Some(container);
                            continue;
                        }
                    }
                }
                #[cfg(feature = "rive_tools")]
                if let Some(external_node) = artboard.external_parent_focus_node() {
                    return Some(external_node);
                }
                return None;
            }
            if parent_ref.base.base.as_node_mut().is_some() {
                for child in parent_ref.children() {
                    if let Some(focus_data) = unsafe { &mut **child }.as_focus_data_mut() {
                        return Some(focus_data.focus_node());
                    }
                }
            }
            current = parent_ref
                .base
                .base
                .parent_mut()
                .map(|value| value as *mut _);
        }
        None
    }
}
