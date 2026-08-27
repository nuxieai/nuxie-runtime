use crate::mechanical_port::source::{
    animation::{
        state_machine_instance::StateMachineInstance,
        state_machine_listener::{ListenerType, StateMachineListener},
        state_machine_listener_single::StateMachineListenerSingle,
    },
    component::Component,
    drawable::Drawable,
    generated::constraints::draggable_constraint_base::DraggableConstraintBase,
    hit_component::HitComponent,
    listener_group::{
        GestureClickPhase, HitTarget, ListenerGroup, ListenerGroupProvider,
        ListenerGroupWithTargets,
    },
    math::vec2d::Vec2D,
    process_event_result::ProcessEventResult,
};

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
    fn hittable(&mut self) -> Option<&mut Drawable>;
}

pub trait DraggableConstraint: DraggableConstraintBase + ListenerGroupProvider {
    fn draggables(&mut self) -> Vec<Box<dyn DraggableProxy>>;

    fn direction(&self) -> DraggableConstraintDirection {
        match self.direction_value() {
            0 => DraggableConstraintDirection::Horizontal,
            1 => DraggableConstraintDirection::Vertical,
            _ => DraggableConstraintDirection::All,
        }
    }

    fn constrains_horizontal(&self) -> bool {
        matches!(
            self.direction(),
            DraggableConstraintDirection::Horizontal | DraggableConstraintDirection::All
        )
    }

    fn constrains_vertical(&self) -> bool {
        matches!(
            self.direction(),
            DraggableConstraintDirection::Vertical | DraggableConstraintDirection::All
        )
    }

    fn listener_groups(&mut self) -> Vec<ListenerGroupWithTargets> {
        let mut result = Vec::new();
        for mut drag_proxy in self.draggables() {
            let mut listener = Box::new(StateMachineListenerSingle::default());
            listener.set_listener_type_value(ListenerType::ComponentProvided as u32);
            let target = drag_proxy.hittable().and_then(|hittable| {
                hittable
                    .as_component_mut()
                    .map(|component| HitTarget::new(component, drag_proxy.is_opaque()))
            });
            let listener_group = DraggableConstraintListenerGroup::new(
                listener,
                self.as_draggable_constraint_ptr(),
                drag_proxy,
            );
            if let Some(target) = target {
                result.push(ListenerGroupWithTargets::new(
                    Box::new(listener_group),
                    vec![Box::new(target)],
                ));
            }
        }
        result
    }

    fn hit_components(&self, _state_machine: &StateMachineInstance) -> Vec<&HitComponent> {
        Vec::new()
    }
}

pub struct DraggableConstraintListenerGroup {
    base: ListenerGroup,
    constraint: *mut dyn DraggableConstraint,
    draggable: Box<dyn DraggableProxy>,
    has_scrolled: bool,
}

impl DraggableConstraintListenerGroup {
    pub fn new(
        listener: Box<dyn StateMachineListener>,
        constraint: *mut dyn DraggableConstraint,
        draggable: Box<dyn DraggableProxy>,
    ) -> Self {
        Self {
            base: ListenerGroup::new(listener),
            constraint,
            draggable,
            has_scrolled: false,
        }
    }

    pub fn enable(&mut self, _pointer_id: i32) {}
    pub fn disable(&mut self, _pointer_id: i32) {}
    pub fn constraint(&mut self) -> *mut dyn DraggableConstraint {
        self.constraint
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
        &mut self,
        component: &mut Component,
        position: Vec2D,
        pointer_id: i32,
        hit_event: ListenerType,
        can_hit: bool,
        time_stamp: f32,
        state_machine_instance: &mut StateMachineInstance,
    ) -> ProcessEventResult {
        let previous_phase = self.base.pointer_data(pointer_id).phase;
        self.base.process_event(
            component,
            position,
            pointer_id,
            hit_event,
            can_hit,
            time_stamp,
            state_machine_instance,
        );
        let phase = self.base.pointer_data(pointer_id).phase;
        if previous_phase == GestureClickPhase::Down
            && matches!(phase, GestureClickPhase::Clicked | GestureClickPhase::Out)
        {
            self.draggable.end_drag(position, time_stamp);
            if self.has_scrolled {
                state_machine_instance.drag_end(position, time_stamp, pointer_id);
                self.has_scrolled = false;
                return ProcessEventResult::Scroll;
            }
        } else if previous_phase != GestureClickPhase::Down && phase == GestureClickPhase::Down {
            self.draggable.start_drag(position, time_stamp);
            self.has_scrolled = false;
        } else if hit_event == ListenerType::Move && phase == GestureClickPhase::Down {
            let has_dragged = self.draggable.drag(position, time_stamp);
            if has_dragged {
                if !self.has_scrolled {
                    state_machine_instance.drag_start(position, time_stamp, false, pointer_id);
                }
                self.has_scrolled = true;
                return ProcessEventResult::Scroll;
            }
        }
        ProcessEventResult::None
    }
}
