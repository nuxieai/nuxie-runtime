use super::*;
use crate::mechanical_port::source::{
    animation::{
        entry_state::EntryState, state_transition::StateTransition,
        system_state_instance::SystemStateInstance,
    },
    artboard::{ArtboardInstance, RuntimeArtboardInstanceHandle},
    component::Component,
    core::{
        binary_reader::BinaryReader, field_types::core_callback_type::CallbackData, Core,
        CoreArena, CoreObject, PropertySetterCompletion,
    },
    drawable::Drawable,
    generated::core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
    listener_group::ListenerGroupBehavior,
    node::Node,
    text::{text::Text, text_value_run::TextValueRun},
};
use std::any::Any;

macro_rules! forward_owner {
    ($ty:ty, $key:expr) => {
        impl CoreObject for $ty {
            fn core(&self) -> &Core {
                self.inner.core()
            }
            fn core_mut(&mut self) -> &mut Core {
                self.inner.core_mut()
            }
            fn core_type(&self) -> u16 {
                $key
            }
            fn is_type_of(&self, key: u16) -> bool {
                key == $key || CoreObject::is_type_of(&self.inner, key)
            }
            fn type_predicate(&self) -> fn(u16) -> bool {
                self.inner.type_predicate()
            }
            fn deserialize(&mut self, key: u16, r: &mut BinaryReader<'_>) -> bool {
                self.inner.deserialize(key, r)
            }
            fn clone_boxed(&self) -> Option<Box<dyn CoreObject>> {
                self.inner.clone_boxed()
            }
        }
        impl CoreRegistryObject for $ty {
            fn as_registry_any(&self) -> &dyn Any {
                self
            }
            fn as_registry_any_mut(&mut self) -> &mut dyn Any {
                self
            }
            fn is_type_of(&self, key: u16) -> bool {
                CoreObject::is_type_of(self, key)
            }
            fn set_uint_with_completion(
                &mut self,
                f: CoreField,
                v: u32,
                c: &mut PropertySetterCompletion,
            ) {
                self.inner.set_uint_with_completion(f, v, c)
            }
            fn set_string_with_completion(
                &mut self,
                f: CoreField,
                v: String,
                c: &mut PropertySetterCompletion,
            ) {
                self.inner.set_string_with_completion(f, v, c)
            }
            fn set_color_with_completion(
                &mut self,
                f: CoreField,
                v: i32,
                c: &mut PropertySetterCompletion,
            ) {
                self.inner.set_color_with_completion(f, v, c)
            }
            fn set_bool_with_completion(
                &mut self,
                f: CoreField,
                v: bool,
                c: &mut PropertySetterCompletion,
            ) {
                self.inner.set_bool_with_completion(f, v, c)
            }
            fn set_double_with_completion(
                &mut self,
                f: CoreField,
                v: f32,
                c: &mut PropertySetterCompletion,
            ) {
                self.inner.set_double_with_completion(f, v, c)
            }
            fn set_callback_with_completion(
                &mut self,
                f: CoreField,
                v: CallbackData<'_>,
                c: &mut PropertySetterCompletion,
            ) {
                self.inner.set_callback_with_completion(f, v, c)
            }
            fn set_int_with_completion(
                &mut self,
                f: CoreField,
                v: i32,
                c: &mut PropertySetterCompletion,
            ) {
                self.inner.set_int_with_completion(f, v, c)
            }
            fn get_uint(&mut self, f: CoreField) -> u32 {
                self.inner.get_uint(f)
            }
            fn get_string(&mut self, f: CoreField) -> String {
                self.inner.get_string(f)
            }
            fn get_color(&mut self, f: CoreField) -> i32 {
                self.inner.get_color(f)
            }
            fn get_bool(&mut self, f: CoreField) -> bool {
                self.inner.get_bool(f)
            }
            fn get_double(&mut self, f: CoreField) -> f32 {
                self.inner.get_double(f)
            }
            fn get_int(&mut self, f: CoreField) -> i32 {
                self.inner.get_int(f)
            }
        }
    };
}

struct AppendTransition {
    inner: StateTransition,
    from: CoreHandle,
    appended: CoreHandle,
    calls: Rc<Cell<usize>>,
}
forward_owner!(AppendTransition, 65530);
impl CoreCapabilities for AppendTransition {
    fn as_state_transition(&self) -> Option<&StateTransition> {
        Some(&self.inner)
    }
    fn as_state_transition_mut(&mut self) -> Option<&mut StateTransition> {
        Some(&mut self.inner)
    }
    fn state_transition_allowed(
        &self,
        _from: &RuntimeStateInstanceHandle,
        _machine: &mut StateMachineInstance,
        _layer: RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> Option<AllowTransition> {
        self.calls.set(self.calls.get() + 1);
        self.from
            .with_mut(|state| state.layer_state_add_transition(self.appended.clone()))
            .unwrap();
        Some(AllowTransition::Yes)
    }
}
struct ClearRandom;
impl Drop for ClearRandom {
    fn drop(&mut self) {
        RandomProvider::clear_testing_mode();
    }
}

#[test]
fn random_second_pass_uses_live_transition_count_after_allowed_callback() {
    let arena = CoreArena::default();
    let from = arena.insert(EntryState::default());
    let to = arena.insert(EntryState::default());
    let mut second = StateTransition::default();
    second.set_state_to(Some(to.clone()));
    let second = arena.insert(second);
    let calls = Rc::new(Cell::new(0));
    let mut first = StateTransition::default();
    first.set_state_to(Some(to));
    first.base.set_random_weight_value(1);
    let first = arena.insert(AppendTransition {
        inner: first,
        from: from.clone(),
        appended: second.clone(),
        calls: calls.clone(),
    });
    from.with_mut(|state| state.layer_state_add_transition(first))
        .unwrap();
    let definition = arena.insert(StateMachine::default());
    let artboard = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    let machine = StateMachineInstance::new(definition, artboard.downgrade());
    RandomProvider::clear_randoms();
    let _clear = ClearRandom;
    RandomProvider::add_random_value(1.0);
    let instance = RuntimeStateInstanceHandle::new(
        from.clone(),
        Box::new(SystemStateInstance::new(from.clone())),
    );
    let selected = machine.with_instance_mut(|machine| {
        StateMachineLayerInstance::default().find_random_transition(machine, instance)
    });
    assert_eq!(calls.get(), 1, "first pass retains its original count");
    assert_eq!(
        from.with(|state| state.layer_state_transition_count())
            .flatten(),
        Some(2)
    );
    assert_eq!(
        selected,
        Some(second),
        "source second pass sees the appended transition's constructor-default evaluated weight"
    );
}

struct RetargetBinding {
    inner: DataBind,
    replacement: CoreHandle,
}
forward_owner!(RetargetBinding, 65531);
impl CoreCapabilities for RetargetBinding {
    fn as_data_bind(&self) -> Option<&DataBind> {
        Some(&self.inner)
    }
    fn as_data_bind_mut(&mut self) -> Option<&mut DataBind> {
        Some(&mut self.inner)
    }
    fn clone_completion_handler(&self) -> Option<fn(&CoreHandle, &CoreHandle) -> bool> {
        Some(|source, _clone| {
            source
                .with_downcast_mut::<RetargetBinding, _>(|source| {
                    source.inner.set_target(Some(source.replacement.clone()))
                })
                .unwrap();
            true
        })
    }
}
#[test]
fn machine_binding_target_is_read_after_clone_callback() {
    let arena = CoreArena::default();
    let first = arena.insert(Node::default());
    let replacement = arena.insert(Node::default());
    let mut binding = DataBind::default();
    binding.set_target(Some(first));
    let binding = arena.insert(RetargetBinding {
        inner: binding,
        replacement: replacement.clone(),
    });
    let mut definition = StateMachine::default();
    definition.add_data_bind(binding.clone());
    let definition = arena.insert(definition);
    let artboard = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    let machine = StateMachineInstance::new(definition, artboard.downgrade());
    assert_eq!(
        binding
            .with(|o| o.as_data_bind().unwrap().target())
            .flatten(),
        Some(replacement.clone())
    );
    let cloned = machine.with_instance(|m| m.data_bind_container.data_binds()[0].clone());
    assert_eq!(
        cloned
            .with(|o| o.as_data_bind().unwrap().target())
            .flatten(),
        Some(replacement),
        "native constructor rereads source target after the virtual clone finishes"
    );
}

struct QueryGroup {
    calls: Rc<RefCell<Vec<&'static str>>>,
    early: bool,
}
impl ListenerGroupBehavior for QueryGroup {
    fn tracked_pointer_ids(&self) -> Vec<i32> {
        vec![]
    }
    fn cancel_pointer(&self, _: i32, _: Vec2D, _: f32) -> bool {
        false
    }
    fn reset(&self, _: i32) {}
    fn release_event(&self, _: i32) {}
    fn hover(&self, _: i32) {}
    fn enable(&self, _: i32) {}
    fn disable(&self, _: i32) {}
    fn is_consumed(&self) -> bool {
        false
    }
    fn can_early_out(&self, _: &Component) -> bool {
        self.calls.borrow_mut().push("early");
        self.early
    }
    fn needs_down_listener(&self, _: &Component) -> bool {
        self.calls.borrow_mut().push("down");
        true
    }
    fn needs_up_listener(&self, _: &Component) -> bool {
        self.calls.borrow_mut().push("up");
        true
    }
    fn process_event(
        &self,
        _: &RuntimeDrawableOccurrence,
        _: Vec2D,
        _: i32,
        _: ListenerType,
        _: PointerButton,
        _: bool,
        _: f32,
        _: &mut StateMachineInstance,
    ) -> ProcessEventResult {
        ProcessEventResult::None
    }
}
#[test]
fn hit_add_listener_short_circuits_unneeded_virtual_queries() {
    let arena = CoreArena::default();
    let shape = arena.insert(crate::mechanical_port::source::shapes::shape::Shape::default());
    let occurrence = RuntimeDrawableOccurrence::Authored(shape);
    let hit = HitDrawable::new(occurrence.clone(), occurrence, false, true, true);
    let calls = Rc::new(RefCell::new(vec![]));
    hit.add_listener(RuntimeListenerGroupHandle::new(Box::new(QueryGroup {
        calls: calls.clone(),
        early: false,
    })));
    assert_eq!(&*calls.borrow(), &["early"]);
    assert!(!hit.can_early_out.get());
    assert!(!hit.has_down_listener.get());
    assert!(!hit.has_up_listener.get());
}

struct TextDirtyProbe {
    inner: Text,
    run: CoreHandle,
    seen: Rc<RefCell<Vec<bool>>>,
    replacement: Option<CoreHandle>,
}
forward_owner!(TextDirtyProbe, 65532);
impl CoreCapabilities for TextDirtyProbe {
    fn as_component(&self) -> Option<&Component> {
        self.inner.as_component()
    }
    fn as_component_mut(&mut self) -> Option<&mut Component> {
        self.inner.as_component_mut()
    }
    fn as_drawable(&self) -> Option<&Drawable> {
        self.inner.as_drawable()
    }
    fn as_drawable_mut(&mut self) -> Option<&mut Drawable> {
        self.inner.as_drawable_mut()
    }
    fn component_on_dirty(&mut self, dirt: ComponentDirt) -> bool {
        self.seen.borrow_mut().push(
            self.run
                .with_downcast::<TextValueRun, _>(TextValueRun::is_hit_target)
                .unwrap(),
        );
        if let Some(replacement) = self.replacement.take() {
            self.run
                .with_downcast_mut::<TextValueRun, _>(|run| run.set_text_component(replacement))
                .unwrap();
        }
        self.inner.component_on_dirty(dirt)
    }
}
#[test]
fn text_run_hit_target_is_published_after_text_dirt_callback() {
    let arena = CoreArena::default();
    let run = arena.insert(TextValueRun::default());
    let seen = Rc::new(RefCell::new(vec![]));
    let mut text = Text::default();
    text.as_component_mut()
        .unwrap()
        .set_dirt(ComponentDirt::NONE);
    let text = arena.insert(TextDirtyProbe {
        inner: text,
        run: run.clone(),
        seen: seen.clone(),
        replacement: None,
    });
    run.with_downcast_mut::<TextValueRun, _>(|run| run.set_text_component(text));
    let definition = arena.insert(StateMachine::default());
    let artboard = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    let machine = StateMachineInstance::new(definition, artboard.downgrade());
    machine.with_instance_mut(|machine| {
        machine.add_to_hit_lookup(
            RuntimeDrawableOccurrence::Authored(run.clone()),
            false,
            &mut HashMap::new(),
            RuntimeListenerGroupHandle::new(Box::new(QueryGroup {
                calls: Rc::new(RefCell::new(vec![])),
                early: false,
            })),
            false,
        )
    });
    assert_eq!(
        &*seen.borrow(),
        &[false],
        "the C++ Text dirt callback precedes HitTextRun construction"
    );
    assert_eq!(
        run.with_downcast::<TextValueRun, _>(TextValueRun::is_hit_target),
        Some(true)
    );
}

struct EndFire {
    inner:
        crate::mechanical_port::source::animation::state_machine_fire_event::StateMachineFireEvent,
    hits: Rc<Cell<usize>>,
}
forward_owner!(EndFire, 65528);
impl CoreCapabilities for EndFire {
    fn state_machine_fire_action_occurs(&self)->Option<crate::mechanical_port::source::animation::state_machine_fire_action::StateMachineFireOccurance>{
        Some(crate::mechanical_port::source::animation::state_machine_fire_action::StateMachineFireOccurance(1))
    }
    fn state_machine_fire_action_perform(&mut self, _: &mut StateMachineInstance) -> bool {
        self.hits.set(self.hits.get() + 1);
        true
    }
}
struct AppendEndFire {
    inner: crate::mechanical_port::source::animation::listener_number_change::ListenerNumberChange,
    transition: CoreHandle,
    event: CoreHandle,
}
forward_owner!(AppendEndFire, 65529);
impl CoreCapabilities for AppendEndFire {
    fn listener_action_matches(
        &self,
        o:crate::mechanical_port::source::animation::state_machine_fire_action::StateMachineFireOccurance,
    ) -> Option<bool> {
        Some(o.0 == 0)
    }
    fn listener_action_perform(
        &mut self,
        _: &mut StateMachineInstance,
        _: &ListenerInvocation,
    ) -> bool {
        assert_eq!(
            self.transition
                .with_mut(|t| t.state_machine_layer_component_add_event(self.event.clone())),
            Some(true)
        );
        true
    }
}
#[test]
fn completed_transition_rereads_end_events_after_start_listener_actions() {
    let arena = CoreArena::default();
    let from = arena.insert(EntryState::default());
    let to = arena.insert(EntryState::default());
    let mut transition = StateTransition::default();
    transition.set_state_to(Some(to));
    transition.base.set_duration_value(0);
    let transition = arena.insert(transition);
    let hits = Rc::new(Cell::new(0));
    let event = arena.insert(EndFire {
        inner: Default::default(),
        hits: hits.clone(),
    });
    let action = arena.insert(AppendEndFire {
        inner: Default::default(),
        transition: transition.clone(),
        event,
    });
    assert_eq!(
        transition.with_mut(|t| t.state_machine_layer_component_add_listener_action(action)),
        Some(true)
    );
    assert_eq!(
        from.with_mut(|s| s.layer_state_add_transition(transition)),
        Some(true)
    );
    let artboard = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    let machine =
        StateMachineInstance::new(arena.insert(StateMachine::default()), artboard.downgrade());
    let instance =
        RuntimeStateInstanceHandle::new(from.clone(), Box::new(SystemStateInstance::new(from)));
    assert!(machine.with_instance_mut(
        |m| StateMachineLayerInstance::default().try_change_state_from(m, Some(instance))
    ));
    assert_eq!(
        hits.get(),
        1,
        "event vector changes between separate source loops are visible at the end phase"
    );
}

struct AppendEventListener {
    inner: crate::mechanical_port::source::animation::listener_number_change::ListenerNumberChange,
    machine: CoreHandle,
    next: Option<CoreHandle>,
    hits: Rc<Cell<usize>>,
}
forward_owner!(AppendEventListener, 65527);
impl CoreCapabilities for AppendEventListener {
    fn listener_action_perform(
        &mut self,
        _: &mut StateMachineInstance,
        _: &ListenerInvocation,
    ) -> bool {
        self.hits.set(self.hits.get() + 1);
        if let Some(next) = self.next.take() {
            self.machine
                .with_downcast_mut::<StateMachine, _>(|machine| machine.add_listener(next))
                .unwrap();
        }
        true
    }
}
#[test]
fn event_listener_dispatch_observes_listener_added_by_an_earlier_action() {
    let arena = CoreArena::default();
    let definition = arena.insert(StateMachine::default());
    let artboard = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    let machine = StateMachineInstance::new(definition.clone(), artboard.downgrade());
    let event = arena.insert(crate::mechanical_port::source::event::Event::default());
    let event_id = artboard.with_artboard_mut(|artboard| {
        artboard.base.add_object(Some(event.clone()));
        artboard.base.id_of(&event)
    });
    let hits = Rc::new(Cell::new(0));
    let make_listener = |action: CoreHandle| {
        let mut listener = StateMachineListenerSingle::default();
        listener
            .base
            .set_listener_type_value_value(ListenerType::Event as u32);
        listener.base.set_event_id_value(event_id);
        listener.base.base.base.set_target_id_value(event_id);
        listener.base.base.add_action(action);
        arena.insert(listener)
    };
    let second_action = arena.insert(AppendEventListener {
        inner: Default::default(),
        machine: definition.clone(),
        next: None,
        hits: hits.clone(),
    });
    let second = make_listener(second_action);
    let first_action = arena.insert(AppendEventListener {
        inner: Default::default(),
        machine: definition.clone(),
        next: Some(second),
        hits: hits.clone(),
    });
    let first = make_listener(first_action);
    definition
        .with_downcast_mut::<StateMachine, _>(|machine| machine.add_listener(first))
        .unwrap();
    machine.with_instance_mut(|machine| {
        machine.notify_event_listeners(
            &[EventReport {
                event: Some(event),
                seconds_delay: 0.0,
                ..EventReport::default()
            }],
            None,
        )
    });
    assert_eq!(
        hits.get(),
        2,
        "source indexed listener traversal rereads its bound after the first listener action"
    );
}

#[test]
fn text_hit_drawable_rereads_text_after_dirt_callback() {
    let arena = CoreArena::default();
    let run = arena.insert(TextValueRun::default());
    let replacement = arena.insert(Text::default());
    let mut text = Text::default();
    text.as_component_mut()
        .unwrap()
        .set_dirt(ComponentDirt::NONE);
    let text = arena.insert(TextDirtyProbe {
        inner: text,
        run: run.clone(),
        seen: Rc::new(RefCell::new(vec![])),
        replacement: Some(replacement.clone()),
    });
    run.with_downcast_mut::<TextValueRun, _>(|run| run.set_text_component(text));
    let definition = arena.insert(StateMachine::default());
    let artboard = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    let machine = StateMachineInstance::new(definition, artboard.downgrade());
    machine.with_instance_mut(|machine| {
        machine.add_to_hit_lookup(
            RuntimeDrawableOccurrence::Authored(run),
            false,
            &mut HashMap::new(),
            RuntimeListenerGroupHandle::new(Box::new(QueryGroup {
                calls: Rc::new(RefCell::new(vec![])),
                early: false,
            })),
            false,
        )
    });
    let selected = machine.with_instance(|machine| {
        machine.hit_components[0]
            .as_hit_drawable()
            .unwrap()
            .drawable
            .authored_handle()
    });
    assert_eq!(
        selected,
        Some(replacement),
        "HitTextRun receives the second source textComponent read after dirt callback"
    );
}

struct RetireRandomSource {
    inner: StateTransition,
    from: CoreHandle,
}
forward_owner!(RetireRandomSource, 65526);
impl CoreCapabilities for RetireRandomSource {
    fn as_state_transition(&self) -> Option<&StateTransition> {
        Some(&self.inner)
    }
    fn as_state_transition_mut(&mut self) -> Option<&mut StateTransition> {
        Some(&mut self.inner)
    }
    fn state_transition_allowed(
        &self,
        _: &RuntimeStateInstanceHandle,
        _: &mut StateMachineInstance,
        _: RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> Option<AllowTransition> {
        assert!(self.from.remove_occurrence());
        Some(AllowTransition::Yes)
    }
}
#[test]
fn random_live_bound_keeps_existing_retired_source_fallback() {
    let arena = CoreArena::default();
    let from = arena.insert(EntryState::default());
    let to = arena.insert(EntryState::default());
    let mut transition = StateTransition::default();
    transition.set_state_to(Some(to));
    transition.base.set_random_weight_value(1);
    let transition = arena.insert(RetireRandomSource {
        inner: transition,
        from: from.clone(),
    });
    from.with_mut(|state| state.layer_state_add_transition(transition))
        .unwrap();
    let artboard = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    let machine =
        StateMachineInstance::new(arena.insert(StateMachine::default()), artboard.downgrade());
    let instance = RuntimeStateInstanceHandle::new(
        from.clone(),
        Box::new(SystemStateInstance::new(from.clone())),
    );
    let selected = machine.with_instance_mut(|machine| {
        StateMachineLayerInstance::default().find_random_transition(machine, instance)
    });
    assert_eq!(
        selected, None,
        "safe Rust retirement keeps the accepted missing-source fallback"
    );
    assert!(from.with(|_| ()).is_none());
}


#[test]
fn event_listener_rejects_a_different_artboard_with_colliding_event_id() {
    let arena = CoreArena::default();
    let definition = arena.insert(StateMachine::default());
    let artboard = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    let machine = StateMachineInstance::new(definition.clone(), artboard.downgrade());
    let event = arena.insert(crate::mechanical_port::source::event::Event::default());
    // This independently owned authored Artboard is the old-file target the
    // source guard excludes even when its event index collides with ours.
    let other_artboard = arena.insert(crate::mechanical_port::source::artboard::Artboard::default());
    let (event_id, other_id) = artboard.with_artboard_mut(|artboard| {
        artboard.base.add_object(Some(event.clone()));
        artboard.base.add_object(Some(other_artboard.clone()));
        (artboard.base.id_of(&event), artboard.base.id_of(&other_artboard))
    });
    let hits = Rc::new(Cell::new(0));
    let action = arena.insert(AppendEventListener {
        inner: Default::default(), machine: definition.clone(), next: None, hits: hits.clone(),
    });
    let mut listener = StateMachineListenerSingle::default();
    listener.base.set_listener_type_value_value(ListenerType::Event as u32);
    listener.base.set_event_id_value(event_id);
    listener.base.base.base.set_target_id_value(other_id);
    listener.base.base.add_action(action);
    let listener = arena.insert(listener);
    definition.with_downcast_mut::<StateMachine, _>(|definition| definition.add_listener(listener.clone())).unwrap();
    let dispatch = || machine.with_instance_mut(|machine| machine.notify_event_listeners(
        &[EventReport { event: Some(event.clone()), seconds_delay: 0.0, ..EventReport::default() }], None));
    dispatch();
    assert_eq!(hits.get(), 0, "a different Artboard is not the current Artboard even when the event index resolves here");
    // Current Artboard and an Event remain the source's two permitted live
    // targets; a missing target also falls through the original null guard.
    for target_id in [0, event_id, u32::MAX] {
        listener.with_downcast_mut::<StateMachineListenerSingle, _>(|listener| listener.base.base.base.set_target_id_value(target_id)).unwrap();
        dispatch();
    }
    assert_eq!(hits.get(), 3);
}
