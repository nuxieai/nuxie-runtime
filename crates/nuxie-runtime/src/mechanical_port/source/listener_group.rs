use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
    },
    component::Component,
    core::{Core, CoreHandle},
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
    listener: Option<CoreHandle>,
    pointers: HashMap<i32, Box<PointerData>>,
    pointers_pool: Vec<Box<PointerData>>,
}

impl ListenerGroup {
    pub fn new(listener: CoreHandle) -> Self {
        Self::new_optional(Some(listener))
    }

    pub fn new_optional(listener: Option<CoreHandle>) -> Self {
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

    fn has_listener(&self, kind: ListenerType) -> bool {
        self.listener
            .as_ref()
            .and_then(|listener| {
                listener
                    .with(|listener| listener.state_machine_listener_has(kind))
                    .flatten()
            })
            .unwrap_or(false)
    }

    pub fn can_early_out(&self, _drawable: &Component) -> bool {
        !(self.has_listener(ListenerType::Enter)
            || self.has_listener(ListenerType::Exit)
            || self.has_listener(ListenerType::Move)
            || self.has_listener(ListenerType::Drag))
    }

    pub fn needs_down_listener(&self, _drawable: &Component) -> bool {
        self.has_listener(ListenerType::Down)
            || self.has_listener(ListenerType::Click)
            || self.has_listener(ListenerType::Drag)
    }

    pub fn needs_up_listener(&self, _drawable: &Component) -> bool {
        self.has_listener(ListenerType::Up)
            || self.has_listener(ListenerType::Click)
            || self.has_listener(ListenerType::Drag)
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
        let mut pointer = self.pointers.remove(&pointer_id).unwrap_or_else(|| {
            self.pointers_pool
                .pop()
                .unwrap_or_else(|| Box::new(PointerData::default()))
        });
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

        let listener = self
            .listener
            .clone()
            .expect("base listener dispatch requires an authored listener");
        let mut should_perform_changes = false;
        let mut listener_type_matched = hit_event;
        if hover_change {
            if is_group_hovered
                && state_machine_instance.listener_has(&listener, ListenerType::Enter)
            {
                should_perform_changes = true;
                listener_type_matched = ListenerType::Enter;
            } else if !is_group_hovered
                && state_machine_instance.listener_has(&listener, ListenerType::Exit)
            {
                should_perform_changes = true;
                listener_type_matched = ListenerType::Exit;
            }
        }
        if pointer.phase == GestureClickPhase::Clicked
            && state_machine_instance.listener_has(&listener, ListenerType::Click)
        {
            should_perform_changes = true;
            listener_type_matched = ListenerType::Click;
        } else if is_group_hovered && state_machine_instance.listener_has(&listener, hit_event) {
            should_perform_changes = true;
        }
        if pointer.phase == GestureClickPhase::Down
            && state_machine_instance.listener_has(&listener, ListenerType::Drag)
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
            state_machine_instance.perform_listener_changes(
                &listener,
                ListenerInvocation::pointer(
                    position,
                    pointer.previous_position,
                    pointer_id,
                    listener_type_matched as u32,
                    time_stamp,
                ),
            );
            state_machine_instance.mark_needs_advance();
            self.consume();
        }
        pointer.previous_position = position;
        self.pointers.insert(pointer_id, pointer);
        ProcessEventResult::Pointer
    }

    pub fn listener(&self) -> CoreHandle {
        self.listener
            .clone()
            .expect("an authored listener group retains its listener")
    }
}

pub struct HitTarget {
    component: CoreHandle,
    is_opaque: bool,
}

impl HitTarget {
    pub fn new(component: CoreHandle, is_opaque: bool) -> Self {
        Self {
            component,
            is_opaque,
        }
    }
    pub fn component(&self) -> CoreHandle {
        self.component.clone()
    }
    pub fn is_opaque(&self) -> bool {
        self.is_opaque
    }
}

pub trait ListenerGroupBehavior {
    fn reset(&mut self, pointer_id: i32);
    fn release_event(&mut self, pointer_id: i32);
    fn hover(&mut self, pointer_id: i32);
    fn enable(&mut self, pointer_id: i32);
    fn disable(&mut self, pointer_id: i32);
    fn is_consumed(&self) -> bool;
    fn can_early_out(&self, drawable: &Component) -> bool;
    fn needs_down_listener(&self, drawable: &Component) -> bool;
    fn needs_up_listener(&self, drawable: &Component) -> bool;
    #[allow(clippy::too_many_arguments)]
    fn process_event(
        &mut self,
        component: &mut Component,
        position: Vec2D,
        pointer_id: i32,
        hit_event: ListenerType,
        can_hit: bool,
        time_stamp: f32,
        state_machine_instance: &mut StateMachineInstance,
    ) -> ProcessEventResult;
}

#[derive(Clone)]
pub struct RuntimeListenerGroupHandle(Rc<RefCell<Box<dyn ListenerGroupBehavior>>>);

impl RuntimeListenerGroupHandle {
    pub fn new(group: Box<dyn ListenerGroupBehavior>) -> Self {
        Self(Rc::new(RefCell::new(group)))
    }

    pub fn with_group<R>(&self, use_group: impl FnOnce(&dyn ListenerGroupBehavior) -> R) -> R {
        use_group(self.0.borrow().as_ref())
    }

    pub fn with_group_mut<R>(
        &self,
        use_group: impl FnOnce(&mut dyn ListenerGroupBehavior) -> R,
    ) -> R {
        use_group(self.0.borrow_mut().as_mut())
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl ListenerGroupBehavior for ListenerGroup {
    fn reset(&mut self, pointer_id: i32) {
        ListenerGroup::reset(self, pointer_id);
    }
    fn release_event(&mut self, pointer_id: i32) {
        ListenerGroup::release_event(self, pointer_id);
    }
    fn hover(&mut self, pointer_id: i32) {
        ListenerGroup::hover(self, pointer_id);
    }
    fn enable(&mut self, pointer_id: i32) {
        ListenerGroup::enable(self, pointer_id);
    }
    fn disable(&mut self, pointer_id: i32) {
        ListenerGroup::disable(self, pointer_id);
    }
    fn is_consumed(&self) -> bool {
        ListenerGroup::is_consumed(self)
    }
    fn can_early_out(&self, drawable: &Component) -> bool {
        ListenerGroup::can_early_out(self, drawable)
    }
    fn needs_down_listener(&self, drawable: &Component) -> bool {
        ListenerGroup::needs_down_listener(self, drawable)
    }
    fn needs_up_listener(&self, drawable: &Component) -> bool {
        ListenerGroup::needs_up_listener(self, drawable)
    }
    fn process_event(
        &mut self,
        component: &mut Component,
        position: Vec2D,
        pointer_id: i32,
        hit_event: ListenerType,
        can_hit: bool,
        time_stamp: f32,
        state_machine_instance: &mut StateMachineInstance,
    ) -> ProcessEventResult {
        ListenerGroup::process_event(
            self,
            component,
            position,
            pointer_id,
            hit_event,
            can_hit,
            time_stamp,
            state_machine_instance,
        )
    }
}

impl ListenerGroupBehavior
    for crate::mechanical_port::source::constraints::draggable_constraint::DraggableConstraintListenerGroup
{
    fn reset(&mut self, pointer_id: i32) {
        Self::reset(self, pointer_id);
    }
    fn release_event(&mut self, pointer_id: i32) {
        Self::release_event(self, pointer_id);
    }
    fn hover(&mut self, pointer_id: i32) {
        Self::hover(self, pointer_id);
    }
    fn enable(&mut self, pointer_id: i32) {
        Self::enable(self, pointer_id);
    }
    fn disable(&mut self, pointer_id: i32) {
        Self::disable(self, pointer_id);
    }
    fn is_consumed(&self) -> bool {
        Self::is_consumed(self)
    }
    fn can_early_out(&self, drawable: &Component) -> bool {
        Self::can_early_out(self, drawable)
    }
    fn needs_down_listener(&self, drawable: &Component) -> bool {
        Self::needs_down_listener(self, drawable)
    }
    fn needs_up_listener(&self, drawable: &Component) -> bool {
        Self::needs_up_listener(self, drawable)
    }
    fn process_event(
        &mut self,
        component: &mut Component,
        position: Vec2D,
        pointer_id: i32,
        hit_event: ListenerType,
        can_hit: bool,
        time_stamp: f32,
        state_machine_instance: &mut StateMachineInstance,
    ) -> ProcessEventResult {
        Self::process_event(
            self,
            component,
            position,
            pointer_id,
            hit_event,
            can_hit,
            time_stamp,
            state_machine_instance,
        )
    }
}

pub struct ListenerGroupWithTargets {
    group: Box<dyn ListenerGroupBehavior>,
    targets: Vec<HitTarget>,
}

impl ListenerGroupWithTargets {
    pub fn new(group: Box<dyn ListenerGroupBehavior>, targets: Vec<Box<HitTarget>>) -> Self {
        Self {
            group,
            targets: targets.into_iter().map(|target| *target).collect(),
        }
    }
    pub fn group(&mut self) -> &mut dyn ListenerGroupBehavior {
        self.group.as_mut()
    }
    pub fn targets(&self) -> &[HitTarget] {
        &self.targets
    }

    pub fn into_parts(self) -> (RuntimeListenerGroupHandle, Vec<HitTarget>) {
        (RuntimeListenerGroupHandle::new(self.group), self.targets)
    }
}

pub enum ListenerGroupProvider {
    ScrollConstraint(CoreHandle),
    ScrollBarConstraint(CoreHandle),
    ScriptedDrawable(CoreHandle),
}

impl ListenerGroupProvider {
    pub fn from(component: &CoreHandle) -> Option<Self> {
        match component.with(|component| component.core_type())? {
            crate::mechanical_port::source::generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase::TYPE_KEY => Some(Self::ScrollConstraint(component.clone())),
            crate::mechanical_port::source::generated::constraints::scrolling::scroll_bar_constraint_base::ScrollBarConstraintBase::TYPE_KEY => Some(Self::ScrollBarConstraint(component.clone())),
            crate::mechanical_port::source::generated::scripted::scripted_layout_base::ScriptedLayoutBase::TYPE_KEY |
            crate::mechanical_port::source::generated::scripted::scripted_drawable_base::ScriptedDrawableBase::TYPE_KEY => Some(Self::ScriptedDrawable(component.clone())),
            _ => None,
        }
    }

    pub fn listener_groups(&self) -> Vec<ListenerGroupWithTargets> {
        use crate::mechanical_port::source::constraints::{
            draggable_constraint::DraggableConstraint,
            scrolling::{
                scroll_bar_constraint::ScrollBarConstraint, scroll_constraint::ScrollConstraint,
            },
        };
        let (owner, draggables) = match self {
            Self::ScrollConstraint(owner) => (
                owner,
                owner
                    .with_downcast_mut::<ScrollConstraint, _>(ScrollConstraint::draggables)
                    .expect("a scroll listener provider remains alive"),
            ),
            Self::ScrollBarConstraint(owner) => (
                owner,
                owner
                    .with_downcast_mut::<ScrollBarConstraint, _>(ScrollBarConstraint::draggables)
                    .expect("a scroll-bar listener provider remains alive"),
            ),
            Self::ScriptedDrawable(_) => return Vec::new(),
        };
        DraggableConstraint::listener_groups(owner.clone(), draggables)
    }

    pub fn hit_components(
        &self,
    ) -> Vec<Box<dyn crate::mechanical_port::source::animation::state_machine_instance::HitComponent>>
    {
        let Self::ScriptedDrawable(owner) = self else {
            return Vec::new();
        };
        let listens = owner
            .with(|owner| {
                owner.as_scripted_object().map(|scripted| {
                    scripted.wants_pointer_down()
                        || scripted.wants_pointer_up()
                        || scripted.wants_pointer_move()
                        || scripted.wants_pointer_exit()
                })
            })
            .flatten()
            .expect("a scripted listener provider retains ScriptedObject");
        if listens {
            vec![Box::new(crate::mechanical_port::source::scripted::scripted_drawable::HitScriptedDrawable::new(owner.clone()))]
        } else {
            Vec::new()
        }
    }
}
