use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
    },
    component::Component,
    core::{Core, CoreHandle},
    drawable::RuntimeDrawableOccurrence,
    gesture_click_phase::GestureClickPhase,
    listener_type::ListenerType,
    math::vec2d::Vec2D,
    process_event_result::ProcessEventResult,
};

pub struct PointerData {
    pub is_hovered: Cell<bool>,
    pub is_prev_hovered: Cell<bool>,
    pub phase: Cell<GestureClickPhase>,
    previous_position: Cell<Vec2D>,
}

impl Default for PointerData {
    fn default() -> Self {
        Self {
            is_hovered: Cell::new(false),
            is_prev_hovered: Cell::new(false),
            phase: Cell::new(GestureClickPhase::Out),
            previous_position: Cell::new(Vec2D::new(0.0, 0.0)),
        }
    }
}

impl PointerData {
    pub fn previous_position(&self) -> Vec2D {
        self.previous_position.get()
    }
}

pub struct ListenerGroup {
    is_consumed: Cell<bool>,
    has_dragged: Cell<bool>,
    listener: Option<CoreHandle>,
    pointers: RefCell<HashMap<i32, Rc<PointerData>>>,
    pointers_pool: RefCell<Vec<Rc<PointerData>>>,
}

impl ListenerGroup {
    pub fn new(listener: CoreHandle) -> Self {
        Self::new_optional(Some(listener))
    }

    pub fn new_optional(listener: Option<CoreHandle>) -> Self {
        Self {
            is_consumed: Cell::new(false),
            has_dragged: Cell::new(false),
            listener,
            pointers: RefCell::new(HashMap::new()),
            pointers_pool: RefCell::new(Vec::new()),
        }
    }

    pub fn pointer_data(&self, id: i32) -> Rc<PointerData> {
        self.pointers
            .borrow_mut()
            .entry(id)
            .or_insert_with(|| {
                self.pointers_pool
                    .borrow_mut()
                    .pop()
                    .unwrap_or_else(|| Rc::new(PointerData::default()))
            })
            .clone()
    }

    pub fn consume(&self) {
        self.is_consumed.set(true);
    }

    pub fn hover(&self, id: i32) {
        self.pointer_data(id).is_hovered.set(true);
    }

    pub fn reset(&self, pointer_id: i32) {
        let pointer = self.pointer_data(pointer_id);
        if pointer.phase.get() != GestureClickPhase::Disabled {
            self.is_consumed.set(false);
            pointer.is_prev_hovered.set(pointer.is_hovered.get());
            pointer.is_hovered.set(false);
        }
        if pointer.phase.get() == GestureClickPhase::Clicked {
            pointer.phase.set(GestureClickPhase::Out);
        }
    }

    pub fn release_event(&self, pointer_id: i32) {
        if let Some(pointer) = self.pointers.borrow_mut().remove(&pointer_id) {
            pointer.is_hovered.set(false);
            pointer.is_prev_hovered.set(false);
            pointer.phase.set(GestureClickPhase::Out);
            pointer.previous_position.set(Vec2D::new(0.0, 0.0));
            self.pointers_pool.borrow_mut().push(pointer);
        }
    }

    pub fn enable(&self, pointer_id: i32) {
        self.pointer_data(pointer_id)
            .phase
            .set(GestureClickPhase::Out);
    }

    pub fn disable(&self, pointer_id: i32) {
        self.pointer_data(pointer_id)
            .phase
            .set(GestureClickPhase::Disabled);
        self.consume();
    }

    pub fn is_consumed(&self) -> bool {
        self.is_consumed.get()
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
        &self,
        _component: &RuntimeDrawableOccurrence,
        position: Vec2D,
        pointer_id: i32,
        hit_event: ListenerType,
        can_hit: bool,
        time_stamp: f32,
        state_machine_instance: &mut StateMachineInstance,
    ) -> ProcessEventResult {
        let pointer = self.pointer_data(pointer_id);
        let previous_phase = pointer.phase.get();
        if !can_hit && pointer.is_hovered.get() {
            pointer.is_hovered.set(false);
        }

        let is_group_hovered = can_hit && pointer.is_hovered.get();
        let hover_change = pointer.is_prev_hovered.get() != is_group_hovered;
        if hover_change && is_group_hovered {
            pointer.previous_position.set(position);
        }

        if is_group_hovered {
            if hit_event == ListenerType::Down {
                pointer.phase.set(GestureClickPhase::Down);
            } else if hit_event == ListenerType::Up
                && pointer.phase.get() == GestureClickPhase::Down
            {
                pointer.phase.set(GestureClickPhase::Clicked);
            }
        } else if hit_event == ListenerType::Down || hit_event == ListenerType::Up {
            pointer.phase.set(GestureClickPhase::Out);
        }

        if previous_phase == GestureClickPhase::Down
            && matches!(
                pointer.phase.get(),
                GestureClickPhase::Clicked | GestureClickPhase::Out
            )
            && self.has_dragged.get()
        {
            state_machine_instance.drag_end(position, time_stamp, pointer_id);
            self.has_dragged.set(false);
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
        if pointer.phase.get() == GestureClickPhase::Clicked
            && state_machine_instance.listener_has(&listener, ListenerType::Click)
        {
            should_perform_changes = true;
            listener_type_matched = ListenerType::Click;
        } else if is_group_hovered && state_machine_instance.listener_has(&listener, hit_event) {
            should_perform_changes = true;
        }
        if pointer.phase.get() == GestureClickPhase::Down
            && state_machine_instance.listener_has(&listener, ListenerType::Drag)
            && hit_event == ListenerType::Move
        {
            should_perform_changes = true;
            listener_type_matched = ListenerType::Drag;
            if !self.has_dragged.get() {
                state_machine_instance.drag_start(position, time_stamp, false, pointer_id);
                self.has_dragged.set(true);
            }
        }

        if should_perform_changes {
            state_machine_instance.perform_listener_changes(
                &listener,
                ListenerInvocation::pointer(
                    position,
                    pointer.previous_position.get(),
                    pointer_id,
                    listener_type_matched as u32,
                    time_stamp,
                ),
            );
            state_machine_instance.mark_needs_advance();
            self.consume();
            {
                use crate::mechanical_port::source::profiler::rive_profile;
                if rive_profile::global_listener_enabled() {
                    let artboard = state_machine_instance
                        .artboard()
                        .upgrade()
                        .expect("live listener Artboard");
                    let artboard_name =
                        artboard.with_artboard(|artboard| artboard.base.name().to_owned());
                    let listener_name = listener
                        .with(|object| {
                            object
                                .as_state_machine_listener()
                                .expect("StateMachineListener")
                                .base
                                .name()
                                .to_owned()
                        })
                        .expect("live listener");
                    rive_profile::record_global_listener_perform_change(
                        &artboard_name,
                        &state_machine_instance.name(),
                        &listener_name,
                        listener_type_matched as u32,
                        hit_event as u32,
                        pointer_id as u32,
                    );
                }
            }
        }
        pointer.previous_position.set(position);
        ProcessEventResult::Pointer
    }

    pub fn listener(&self) -> CoreHandle {
        self.listener
            .clone()
            .expect("an authored listener group retains its listener")
    }
}

pub struct HitTarget {
    component: RuntimeDrawableOccurrence,
    is_opaque: bool,
}

impl HitTarget {
    pub fn new(component: RuntimeDrawableOccurrence, is_opaque: bool) -> Self {
        Self {
            component,
            is_opaque,
        }
    }
    pub fn component(&self) -> RuntimeDrawableOccurrence {
        self.component.clone()
    }
    pub fn is_opaque(&self) -> bool {
        self.is_opaque
    }
}

pub trait ListenerGroupBehavior {
    fn reset(&self, pointer_id: i32);
    fn release_event(&self, pointer_id: i32);
    fn hover(&self, pointer_id: i32);
    fn enable(&self, pointer_id: i32);
    fn disable(&self, pointer_id: i32);
    fn is_consumed(&self) -> bool;
    fn can_early_out(&self, drawable: &Component) -> bool;
    fn needs_down_listener(&self, drawable: &Component) -> bool;
    fn needs_up_listener(&self, drawable: &Component) -> bool;
    #[allow(clippy::too_many_arguments)]
    fn process_event(
        &self,
        component: &RuntimeDrawableOccurrence,
        position: Vec2D,
        pointer_id: i32,
        hit_event: ListenerType,
        can_hit: bool,
        time_stamp: f32,
        state_machine_instance: &mut StateMachineInstance,
    ) -> ProcessEventResult;
}

#[derive(Clone)]
pub struct RuntimeListenerGroupHandle(Rc<dyn ListenerGroupBehavior>);

impl RuntimeListenerGroupHandle {
    pub fn new(group: Box<dyn ListenerGroupBehavior>) -> Self {
        Self(Rc::from(group))
    }

    pub fn with_group<R>(&self, use_group: impl FnOnce(&dyn ListenerGroupBehavior) -> R) -> R {
        use_group(self.0.as_ref())
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl ListenerGroupBehavior for ListenerGroup {
    fn reset(&self, pointer_id: i32) {
        ListenerGroup::reset(self, pointer_id);
    }
    fn release_event(&self, pointer_id: i32) {
        ListenerGroup::release_event(self, pointer_id);
    }
    fn hover(&self, pointer_id: i32) {
        ListenerGroup::hover(self, pointer_id);
    }
    fn enable(&self, pointer_id: i32) {
        ListenerGroup::enable(self, pointer_id);
    }
    fn disable(&self, pointer_id: i32) {
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
        &self,
        component: &RuntimeDrawableOccurrence,
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
    fn reset(&self, pointer_id: i32) {
        Self::reset(self, pointer_id);
    }
    fn release_event(&self, pointer_id: i32) {
        Self::release_event(self, pointer_id);
    }
    fn hover(&self, pointer_id: i32) {
        Self::hover(self, pointer_id);
    }
    fn enable(&self, pointer_id: i32) {
        Self::enable(self, pointer_id);
    }
    fn disable(&self, pointer_id: i32) {
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
        &self,
        component: &RuntimeDrawableOccurrence,
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
