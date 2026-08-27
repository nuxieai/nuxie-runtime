use std::collections::HashMap;

use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
        state_machine_listener::StateMachineListener,
    },
    component::Component,
    core::Core,
    gesture_click_phase::GestureClickPhase,
    listener_type::ListenerType,
    math::vec2d::Vec2D,
    process_event_result::ProcessEventResult,
};

pub struct PointerData {
    pub is_hovered: bool,
    pub is_prev_hovered: bool,
    pub phase: GestureClickPhase,
    previous_position: Vec2D,
}

impl Default for PointerData {
    fn default() -> Self {
        Self {
            is_hovered: false,
            is_prev_hovered: false,
            phase: GestureClickPhase::Out,
            previous_position: Vec2D::new(0.0, 0.0),
        }
    }
}

impl PointerData {
    pub fn previous_position(&mut self) -> &mut Vec2D {
        &mut self.previous_position
    }
}

pub struct ListenerGroup {
    is_consumed: bool,
    has_dragged: bool,
    listener: *const StateMachineListener,
    pointers: HashMap<i32, Box<PointerData>>,
    pointers_pool: Vec<Box<PointerData>>,
}

impl ListenerGroup {
    pub fn new(listener: *const StateMachineListener) -> Self {
        Self {
            is_consumed: false,
            has_dragged: false,
            listener,
            pointers: HashMap::new(),
            pointers_pool: Vec::new(),
        }
    }

    pub fn pointer_data(&mut self, id: i32) -> &mut PointerData {
        self.pointers.entry(id).or_insert_with(|| {
            self.pointers_pool
                .pop()
                .unwrap_or_else(|| Box::new(PointerData::default()))
        })
    }

    pub fn consume(&mut self) {
        self.is_consumed = true;
    }

    pub fn hover(&mut self, id: i32) {
        self.pointer_data(id).is_hovered = true;
    }

    pub fn reset(&mut self, pointer_id: i32) {
        let pointer = self.pointer_data(pointer_id);
        if pointer.phase != GestureClickPhase::Disabled {
            self.is_consumed = false;
            pointer.is_prev_hovered = pointer.is_hovered;
            pointer.is_hovered = false;
        }
        if pointer.phase == GestureClickPhase::Clicked {
            pointer.phase = GestureClickPhase::Out;
        }
    }

    pub fn release_event(&mut self, pointer_id: i32) {
        if let Some(mut pointer) = self.pointers.remove(&pointer_id) {
            pointer.is_hovered = false;
            pointer.is_prev_hovered = false;
            pointer.phase = GestureClickPhase::Out;
            *pointer.previous_position() = Vec2D::new(0.0, 0.0);
            self.pointers_pool.push(pointer);
        }
    }

    pub fn enable(&mut self, pointer_id: i32) {
        self.pointer_data(pointer_id).phase = GestureClickPhase::Out;
    }

    pub fn disable(&mut self, pointer_id: i32) {
        self.pointer_data(pointer_id).phase = GestureClickPhase::Disabled;
        self.consume();
    }

    pub fn is_consumed(&self) -> bool {
        self.is_consumed
    }

    pub fn can_early_out(&self, _drawable: &Component) -> bool {
        let listener = unsafe { &*self.listener };
        !(listener.has_listener(ListenerType::Enter)
            || listener.has_listener(ListenerType::Exit)
            || listener.has_listener(ListenerType::Move)
            || listener.has_listener(ListenerType::Drag))
    }

    pub fn needs_down_listener(&self, _drawable: &Component) -> bool {
        let listener = unsafe { &*self.listener };
        listener.has_listener(ListenerType::Down)
            || listener.has_listener(ListenerType::Click)
            || listener.has_listener(ListenerType::Drag)
    }

    pub fn needs_up_listener(&self, _drawable: &Component) -> bool {
        let listener = unsafe { &*self.listener };
        listener.has_listener(ListenerType::Up)
            || listener.has_listener(ListenerType::Click)
            || listener.has_listener(ListenerType::Drag)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn process_event(
        &mut self,
        _component: &mut Component,
        position: Vec2D,
        pointer_id: i32,
        hit_event: ListenerType,
        can_hit: bool,
        time_stamp: f32,
        state_machine_instance: &mut StateMachineInstance,
    ) -> ProcessEventResult {
        let pointer = self.pointer_data(pointer_id) as *mut PointerData;
        let pointer = unsafe { &mut *pointer };
        let previous_phase = pointer.phase;
        if !can_hit && pointer.is_hovered {
            pointer.is_hovered = false;
        }

        let is_group_hovered = can_hit && pointer.is_hovered;
        let hover_change = pointer.is_prev_hovered != is_group_hovered;
        if hover_change && is_group_hovered {
            pointer.previous_position = position;
        }

        if is_group_hovered {
            if hit_event == ListenerType::Down {
                pointer.phase = GestureClickPhase::Down;
            } else if hit_event == ListenerType::Up && pointer.phase == GestureClickPhase::Down {
                pointer.phase = GestureClickPhase::Clicked;
            }
        } else if hit_event == ListenerType::Down || hit_event == ListenerType::Up {
            pointer.phase = GestureClickPhase::Out;
        }

        if previous_phase == GestureClickPhase::Down
            && matches!(
                pointer.phase,
                GestureClickPhase::Clicked | GestureClickPhase::Out
            )
            && self.has_dragged
        {
            state_machine_instance.drag_end(position, time_stamp, pointer_id);
            self.has_dragged = false;
        }

        let listener = unsafe { &*self.listener };
        let mut should_perform_changes = false;
        let mut listener_type_matched = hit_event;
        if hover_change {
            if is_group_hovered && listener.has_listener(ListenerType::Enter) {
                should_perform_changes = true;
                listener_type_matched = ListenerType::Enter;
            } else if !is_group_hovered && listener.has_listener(ListenerType::Exit) {
                should_perform_changes = true;
                listener_type_matched = ListenerType::Exit;
            }
        }
        if pointer.phase == GestureClickPhase::Clicked && listener.has_listener(ListenerType::Click)
        {
            should_perform_changes = true;
            listener_type_matched = ListenerType::Click;
        } else if is_group_hovered && listener.has_listener(hit_event) {
            should_perform_changes = true;
        }
        if pointer.phase == GestureClickPhase::Down
            && listener.has_listener(ListenerType::Drag)
            && hit_event == ListenerType::Move
        {
            should_perform_changes = true;
            listener_type_matched = ListenerType::Drag;
            if !self.has_dragged {
                state_machine_instance.drag_start(position, time_stamp, false, pointer_id);
                self.has_dragged = true;
            }
        }

        if should_perform_changes {
            listener.perform_changes(
                state_machine_instance,
                ListenerInvocation::pointer(
                    position,
                    pointer.previous_position,
                    pointer_id,
                    listener_type_matched,
                    time_stamp,
                ),
            );
            state_machine_instance.mark_needs_advance();
            self.consume();
        }
        pointer.previous_position = position;
        ProcessEventResult::Pointer
    }

    pub fn listener(&self) -> &StateMachineListener {
        unsafe { &*self.listener }
    }
}

pub struct HitTarget {
    component: *mut Component,
    is_opaque: bool,
}

impl HitTarget {
    pub fn new(component: *mut Component, is_opaque: bool) -> Self {
        Self {
            component,
            is_opaque,
        }
    }
    pub fn component(&mut self) -> &mut Component {
        unsafe { &mut *self.component }
    }
    pub fn is_opaque(&self) -> bool {
        self.is_opaque
    }
}

pub struct ListenerGroupWithTargets {
    group: *mut ListenerGroup,
    targets: Vec<*mut HitTarget>,
}

impl ListenerGroupWithTargets {
    pub fn new(group: *mut ListenerGroup, targets: Vec<*mut HitTarget>) -> Self {
        Self { group, targets }
    }
    pub fn group(&mut self) -> &mut ListenerGroup {
        unsafe { &mut *self.group }
    }
    pub fn targets(&self) -> &[*mut HitTarget] {
        &self.targets
    }
}

pub trait ListenerGroupProvider {
    fn listener_groups(&mut self) -> Vec<*mut ListenerGroupWithTargets>;
    fn hit_components(
        &mut self,
        state_machine: &mut StateMachineInstance,
    ) -> Vec<*mut crate::mechanical_port::source::animation::hit_component::HitComponent>;
}

impl dyn ListenerGroupProvider {
    pub fn from(component: &mut Core) -> Option<&mut dyn ListenerGroupProvider> {
        match component.core_type() {
            crate::mechanical_port::source::generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase::TYPE_KEY => component.as_scroll_constraint_mut().map(|value| value as &mut dyn ListenerGroupProvider),
            crate::mechanical_port::source::generated::constraints::scrolling::scroll_bar_constraint_base::ScrollBarConstraintBase::TYPE_KEY => component.as_scroll_bar_constraint_mut().map(|value| value as &mut dyn ListenerGroupProvider),
            crate::mechanical_port::source::generated::scripted::scripted_layout_base::ScriptedLayoutBase::TYPE_KEY => component.as_scripted_layout_mut().map(|value| value as &mut dyn ListenerGroupProvider),
            crate::mechanical_port::source::generated::scripted::scripted_drawable_base::ScriptedDrawableBase::TYPE_KEY => component.as_scripted_drawable_mut().map(|value| value as &mut dyn ListenerGroupProvider),
            _ => None,
        }
    }
}
