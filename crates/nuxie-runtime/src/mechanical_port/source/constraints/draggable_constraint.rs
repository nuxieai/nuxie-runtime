use std::{
    cell::{Cell, RefCell},
    ops::{Deref, DerefMut},
};

use crate::mechanical_port::source::{
    animation::{
        state_machine_instance::StateMachineInstance, state_machine_listener::ListenerType,
        state_machine_listener_single::StateMachineListenerSingle,
    },
    component::Component,
    core::{Core, CoreHandle},
    drawable::RuntimeDrawableOccurrence,
    generated::{
        component_base::ComponentBaseCallbacks,
        constraints::{
            constraint_base::ConstraintBaseCallbacks,
            draggable_constraint_base::{
                DraggableConstraintBase, DraggableConstraintBaseCallbacks,
            },
        },
    },
    listener_group::{GestureClickPhase, HitTarget, ListenerGroup, ListenerGroupWithTargets},
    math::vec2d::Vec2D,
    process_event_result::ProcessEventResult,
};

impl ComponentBaseCallbacks for DraggableConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

impl ConstraintBaseCallbacks for DraggableConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }

    fn strength_changed(&mut self) {
        crate::mechanical_port::source::constraints::constraint::Constraint::strength_changed(self);
    }
}

impl DraggableConstraintBaseCallbacks for DraggableConstraint {
    fn notify_property_changed(&mut self, property_key: u16) {
        Core::notify_property_changed(self, property_key);
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DraggableConstraintDirection {
    Horizontal,
    Vertical,
    All,
}

pub trait DraggableProxy {
    fn is_opaque(&self) -> bool {
        false
    }
    fn start_drag(&mut self, mouse_position: Vec2D, time_stamp: f32) -> bool;
    fn drag(&mut self, mouse_position: Vec2D, time_stamp: f32) -> bool;
    fn end_drag(&mut self, mouse_position: Vec2D, time_stamp: f32) -> bool;
    fn hittable(&self) -> Option<CoreHandle>;
}

#[derive(Default)]
pub struct DraggableConstraint {
    pub base: DraggableConstraintBase,
}

impl Deref for DraggableConstraint {
    type Target = DraggableConstraintBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for DraggableConstraint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl DraggableConstraint {
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DraggableConstraintBaseCallbacks) {
        self.base.copy(&object.base, callbacks);
    }

    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut crate::mechanical_port::source::core::binary_reader::BinaryReader<'_>,
        callbacks: &mut impl DraggableConstraintBaseCallbacks,
    ) -> bool {
        self.base.deserialize(property_key, reader, callbacks)
    }

    pub fn direction(&self) -> DraggableConstraintDirection {
        match self.direction_value() {
            0 => DraggableConstraintDirection::Horizontal,
            1 => DraggableConstraintDirection::Vertical,
            _ => DraggableConstraintDirection::All,
        }
    }

    pub fn constrains_horizontal(&self) -> bool {
        matches!(
            self.direction(),
            DraggableConstraintDirection::Horizontal | DraggableConstraintDirection::All
        )
    }

    pub fn constrains_vertical(&self) -> bool {
        matches!(
            self.direction(),
            DraggableConstraintDirection::Vertical | DraggableConstraintDirection::All
        )
    }

    pub fn listener_groups(
        constraint: CoreHandle,
        draggables: Vec<Box<dyn DraggableProxy>>,
    ) -> Vec<ListenerGroupWithTargets> {
        let mut result = Vec::new();
        for drag_proxy in draggables {
            let mut listener = StateMachineListenerSingle::default();
            listener.set_listener_type_value(ListenerType::ComponentProvided as u32);
            let Some(listener) = constraint.insert_sibling(listener) else {
                continue;
            };
            let target = drag_proxy
                .hittable()
                .map(|hittable| HitTarget::new(hittable, drag_proxy.is_opaque()));
            let listener_group =
                DraggableConstraintListenerGroup::new(listener, constraint.clone(), drag_proxy);
            if let Some(target) = target {
                result.push(ListenerGroupWithTargets::new(
                    Box::new(listener_group),
                    vec![Box::new(target)],
                ));
            }
        }
        result
    }
}

pub struct DraggableConstraintListenerGroup {
    base: ListenerGroup,
    constraint: CoreHandle,
    draggable: RefCell<Box<dyn DraggableProxy>>,
    has_scrolled: Cell<bool>,
}

impl DraggableConstraintListenerGroup {
    pub fn new(
        listener: CoreHandle,
        constraint: CoreHandle,
        draggable: Box<dyn DraggableProxy>,
    ) -> Self {
        Self {
            base: ListenerGroup::new(listener),
            constraint,
            draggable: RefCell::new(draggable),
            has_scrolled: Cell::new(false),
        }
    }

    pub fn enable(&self, _pointer_id: i32) {}
    pub fn disable(&self, _pointer_id: i32) {}
    pub fn reset(&self, pointer_id: i32) {
        self.base.reset(pointer_id);
    }
    pub fn release_event(&self, pointer_id: i32) {
        self.base.release_event(pointer_id);
    }
    pub fn hover(&self, pointer_id: i32) {
        self.base.hover(pointer_id);
    }
    pub fn is_consumed(&self) -> bool {
        self.base.is_consumed()
    }
    pub fn constraint(&self) -> CoreHandle {
        self.constraint.clone()
    }
    pub fn can_early_out(&self, _drawable: &Component) -> bool {
        false
    }
    pub fn needs_down_listener(&self, _drawable: &Component) -> bool {
        true
    }
    pub fn needs_up_listener(&self, _drawable: &Component) -> bool {
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub fn process_event(
        &self,
        component: &RuntimeDrawableOccurrence,
        position: Vec2D,
        pointer_id: i32,
        hit_event: ListenerType,
        can_hit: bool,
        time_stamp: f32,
        state_machine_instance: &mut StateMachineInstance,
    ) -> ProcessEventResult {
        let pointer = self.base.pointer_data(pointer_id);
        let previous_phase = pointer.phase.get();
        self.base.process_event(
            component,
            position,
            pointer_id,
            hit_event,
            can_hit,
            time_stamp,
            state_machine_instance,
        );
        let phase = pointer.phase.get();
        if previous_phase == GestureClickPhase::Down
            && matches!(phase, GestureClickPhase::Clicked | GestureClickPhase::Out)
        {
            self.draggable.borrow_mut().end_drag(position, time_stamp);
            if self.has_scrolled.get() {
                state_machine_instance.drag_end(position, time_stamp, pointer_id);
                self.has_scrolled.set(false);
                return ProcessEventResult::Scroll;
            }
        } else if previous_phase != GestureClickPhase::Down && phase == GestureClickPhase::Down {
            self.draggable.borrow_mut().start_drag(position, time_stamp);
            self.has_scrolled.set(false);
        } else if hit_event == ListenerType::Move && phase == GestureClickPhase::Down {
            let has_dragged = self.draggable.borrow_mut().drag(position, time_stamp);
            if has_dragged {
                if !self.has_scrolled.get() {
                    state_machine_instance.drag_start(position, time_stamp, false, pointer_id);
                }
                self.has_scrolled.set(true);
                return ProcessEventResult::Scroll;
            }
        }
        ProcessEventResult::None
    }
}
