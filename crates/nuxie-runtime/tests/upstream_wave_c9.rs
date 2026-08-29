//! Preserved state-machine, event, and input tests against the native owners.
//! Authority: pinned state_machine_test.cpp, state_machine_event_test.cpp,
//! and state_machine_input_test.cpp. No parsed graph substitutes for execution.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    File, ImportResult, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    RuntimeStateMachineInstanceHandle,
    source::{
        animation::{
            animation_state::AnimationState, blend_animation_1d::BlendAnimation1D,
            blend_state_1d::BlendState1D, blend_state_transition::BlendStateTransition,
            listener_fire_event::ListenerFireEvent, listener_input_change::ListenerInputChange,
            nested_bool::NestedBool, nested_number::NestedNumber,
            nested_state_machine::NestedStateMachine, nested_trigger::NestedTrigger,
            state_machine::StateMachine, state_machine_bool::StateMachineBool,
            state_machine_instance::StateMachineInstance, state_machine_layer::StateMachineLayer,
            state_machine_listener::StateMachineListener,
        },
        core::{CoreHandle, CoreType},
        event::Event,
        generated::core_registry::CoreRegistry,
        math::vec2d::Vec2D,
        nested_artboard::NestedArtboard,
        node::Node,
        shapes::shape::Shape,
        viewmodel::viewmodel_instance_trigger::ViewModelInstanceTrigger,
    },
};
use std::{any::Any, path::PathBuf};

fn read<T: Any, R>(owner: &CoreHandle, f: impl FnOnce(&T) -> R) -> R {
    owner
        .with_downcast(f)
        .expect("live native owner of expected type")
}
fn read_listener<R>(owner: &CoreHandle, f: impl FnOnce(&StateMachineListener) -> R) -> R {
    owner
        .with(|object| {
            f(object
                .as_state_machine_listener()
                .expect("native listener base"))
        })
        .expect("live listener owner")
}
fn typed<T: CoreType>(owner: &CoreHandle) {
    assert!(owner.is_type_of(T::TYPE_KEY));
}
fn key(owner: &str, property: &str) -> i32 {
    let def = nuxie_schema::definition_by_name(owner).unwrap();
    std::iter::once(def.name)
        .chain(def.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|def| def.properties)
        .find(|p| p.name == property)
        .unwrap()
        .key
        .int as i32
}
fn uint(owner: &CoreHandle, type_name: &str, property: &str) -> u32 {
    CoreRegistry::get_uint_handle(owner, key(type_name, property)).expect("live property")
}
fn name(owner: &CoreHandle) -> String {
    owner
        .with(|object| {
            object
                .as_component()
                .expect("named Component")
                .name()
                .to_owned()
        })
        .unwrap()
}
struct Fixture {
    machine: RuntimeStateMachineInstanceHandle,
    artboard: RuntimeArtboardInstanceHandle,
    file: RuntimeFileHandle,
    _factory: PersistentFactory<RecordingFactory>,
}
impl Fixture {
    fn new(asset: &str, artboard_name: Option<&str>, machine_name: Option<&str>) -> Self {
        let path = PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets")
        .join(asset);
        let bytes = std::fs::read(&path).expect("pinned fixture");
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let retained = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
        let mut result = ImportResult::Malformed;
        let file = File::import(&bytes, retained, Some(&mut result), None, None)
            .unwrap_or_else(|| panic!("{}: {result:?}", path.display()));
        assert_eq!(result, ImportResult::Success);
        let artboard = file
            .with_file(|file| match artboard_name {
                Some(name) => file.artboard_named(name),
                None => file.artboard_default(),
            })
            .expect("artboard instance");
        let machine = match machine_name {
            Some(name) => artboard.state_machine_named(name),
            None => artboard.state_machine_at(0),
        }
        .expect("state machine instance");
        Self {
            machine,
            artboard,
            file,
            _factory: factory,
        }
    }
    fn definition(&self) -> CoreHandle {
        self.machine.with_instance(|m| m.state_machine())
    }
    fn layer(&self) -> CoreHandle {
        read::<StateMachine, _>(&self.definition(), |m| m.layer(0)).unwrap()
    }
    fn assert_counts(&self, animations: usize, machines: usize) {
        self.artboard.with_artboard(|a| {
            assert_eq!(a.animation_count(), animations);
            assert_eq!(a.state_machine_count(), machines);
        });
    }
    fn advance(&self, seconds: f32) {
        // Pinned advance(), not advanceAndApply(): several pointer tests
        // intentionally retain design-time transforms after the state applies.
        self.machine
            .with_instance_mut(|m| m.advance_seconds(seconds));
    }
    fn initial_advance(&self) {
        self.artboard.advance_default(0.0);
        self.advance(0.0);
    }
    fn listener(&self, index: usize) -> CoreHandle {
        read::<StateMachine, _>(&self.definition(), |m| m.listener(index)).unwrap()
    }
    fn resolve(&self, id: u32) -> CoreHandle {
        self.artboard
            .with_artboard(|a| a.resolve_handle(id))
            .expect("resolved authored object")
    }
    fn events(&self) -> Vec<CoreHandle> {
        self.artboard
            .with_artboard(|a| a.find_all_handles::<Event>())
    }
    fn event_count(&self) -> usize {
        self.machine.with_instance(|m| m.reported_event_count())
    }
    fn event_name(&self, index: usize) -> String {
        name(
            &self
                .machine
                .with_instance(|m| m.reported_event_at(index).event.expect("reported event")),
        )
    }
    fn down(&self, x: f32, y: f32) {
        self.machine
            .with_instance_mut(|m| m.pointer_down(Vec2D::new(x, y), 0));
    }
    fn up(&self, x: f32, y: f32) {
        self.machine
            .with_instance_mut(|m| m.pointer_up(Vec2D::new(x, y), 0));
    }
}
fn layer_shape(layer: &CoreHandle, count: usize) {
    read::<StateMachineLayer, _>(layer, |layer| {
        assert_eq!(layer.state_count(), count);
        assert!(layer.any_state().is_some());
        assert!(layer.entry_state().is_some());
        assert!(layer.exit_state().is_some());
    });
}
fn transitions(state: &CoreHandle) -> Vec<CoreHandle> {
    state
        .with(|s| {
            (0..s.layer_state_transition_count().unwrap())
                .map(|i| s.layer_state_transition(i).unwrap())
                .collect()
        })
        .unwrap()
}
fn state_to(transition: &CoreHandle) -> CoreHandle {
    transition
        .with(|t| t.as_state_transition().unwrap().state_to())
        .flatten()
        .unwrap()
}
fn animation_name(state: &CoreHandle) -> String {
    let animation = read::<AnimationState, _>(state, |s| s.animation()).unwrap();
    animation
        .with_downcast::<nuxie_runtime::source::animation::linear_animation::LinearAnimation, _>(
            |a| a.name().to_owned(),
        )
        .unwrap()
}

#[test]
fn wave_c9_state_machine_001_file_with_state_machine_can_be_read() {
    let f = Fixture::new("rocket.riv", None, Some("Button"));
    f.assert_counts(3, 1);
    let machine = f.definition();
    read::<StateMachine, _>(&machine, |m| {
        assert_eq!(m.layer_count(), 1);
        assert_eq!(m.input_count(), 2);
        typed::<StateMachineBool>(&m.input_named("Hover").unwrap());
        typed::<StateMachineBool>(&m.input_named("Press").unwrap());
    });
    let layer = f.layer();
    layer_shape(&layer, 6);
    let states = read::<StateMachineLayer, _>(&layer, |l| l.states().to_vec());
    let mut animation_states = 0;
    for state in states {
        if state.is_type_of(AnimationState::TYPE_KEY) {
            animation_states += 1;
            assert!(read::<AnimationState, _>(&state, |s| s.animation()).is_some());
        }
    }
    assert_eq!(animation_states, 3);
    let entry = read::<StateMachineLayer, _>(&layer, |l| l.entry_state()).unwrap();
    let entry_transitions = transitions(&entry);
    assert_eq!(entry_transitions.len(), 1);
    let idle = state_to(&entry_transitions[0]);
    typed::<AnimationState>(&idle);
    assert_eq!(animation_name(&idle), "idle");
    let idle_transitions = transitions(&idle);
    assert_eq!(idle_transitions.len(), 2);
    for transition in idle_transitions {
        if animation_name(&state_to(&transition)) == "Roll_over" {
            assert_eq!(
                transition.with(|t| t.as_state_transition().unwrap().condition_count()),
                Some(1)
            );
        }
    }
    f.machine.with_instance_mut(|m| {
        assert_eq!(m.get_bool("Hover").map(|i| i.base.name()), Some("Hover"));
        assert_eq!(m.get_bool("Press").map(|i| i.base.name()), Some("Press"));
        assert!(m.get_bool("Hover").is_some());
        assert!(m.get_bool("Press").is_some());
        assert_eq!(m.state_changed_count(), 0);
        assert_eq!(m.current_animation_count(), 0);
    });
}

#[test]
fn wave_c9_state_machine_002_file_with_blend_states_loads_correctly() {
    let f = Fixture::new("blend_test.riv", None, Some("blend"));
    f.assert_counts(4, 2);
    assert_eq!(
        read::<StateMachine, _>(&f.definition(), |m| m.layer_count()),
        1
    );
    let layer = f.layer();
    layer_shape(&layer, 5);
    let a = read::<StateMachineLayer, _>(&layer, |l| l.state(1)).unwrap();
    let b = read::<StateMachineLayer, _>(&layer, |l| l.state(2)).unwrap();
    typed::<BlendState1D>(&a);
    typed::<BlendState1D>(&b);
    assert_eq!(
        a.with(|s| s.blend_state_animations().unwrap().len()),
        Some(3)
    );
    assert_eq!(
        b.with(|s| s.blend_state_animations().unwrap().len()),
        Some(3)
    );
    for (index, expected_name, expected_value) in [
        (0, "horizontal", 0.0),
        (1, "vertical", 100.0),
        (2, "rotate", 0.0),
    ] {
        let blend = a
            .with(|s| s.blend_state_animations().unwrap()[index].clone())
            .unwrap();
        typed::<BlendAnimation1D>(&blend);
        read::<BlendAnimation1D, _>(&blend, |blend| {
            let animation = blend.base.base.animation().unwrap();
            assert_eq!(
                animation
                    .with_downcast::<nuxie_runtime::source::animation::linear_animation::LinearAnimation,_>(|a| a.name().to_owned())
                    .unwrap(),
                expected_name
            );
            assert_eq!(blend.base.value(), expected_value);
        });
    }
    let transitions = transitions(&a);
    assert_eq!(transitions.len(), 1);
    typed::<BlendStateTransition>(&transitions[0]);
    assert!(
        read::<BlendStateTransition, _>(&transitions[0], |t| t.exit_blend_animation()).is_some()
    );
}

#[test]
fn wave_c9_state_machine_003_animation_state_without_animation_does_not_crash() {
    let f = Fixture::new("multiple_state_machines.riv", None, Some("two"));
    f.assert_counts(1, 4);
    assert_eq!(
        read::<StateMachine, _>(&f.definition(), |m| m.layer_count()),
        1
    );
    let layer = f.layer();
    layer_shape(&layer, 4);
    let state = read::<StateMachineLayer, _>(&layer, |l| l.state(3)).unwrap();
    typed::<AnimationState>(&state);
    assert!(read::<AnimationState, _>(&state, |s| s.animation()).is_none());
    f.advance(0.0);
}
#[test]
fn wave_c9_state_machine_004_oneshot_blend_keeps_going_after_animations_stop() {
    let f = Fixture::new("oneshotblend.riv", None, Some("State Machine 1"));
    for seconds in [0.0, 0.5, 1.0] {
        f.advance(seconds);
        assert!(f.machine.with_instance(StateMachineInstance::needs_advance));
    }
}

#[test]
fn wave_c9_event_001_file_with_state_machine_listeners_can_be_read() {
    let f = Fixture::new("bullet_man.riv", Some("Bullet Man"), None);
    assert_eq!(f.artboard.with_artboard(|a| a.state_machine_count()), 1);
    read::<StateMachine, _>(&f.definition(), |m| {
        assert_eq!(m.listener_count(), 3);
        assert_eq!(m.input_count(), 4);
    });
    for (index, expected_name) in ["HandWickHit", "HandCannonHit", "HandHelmetHit"]
        .into_iter()
        .enumerate()
    {
        let listener = f.listener(index);
        let (target, action) = read_listener(&listener, |l| {
            assert_eq!(l.action_count(), 1);
            (l.target_id(), l.action(0).unwrap())
        });
        let target = f.resolve(target);
        typed::<Node>(&target);
        assert_eq!(name(&target), expected_name);
        typed::<ListenerInputChange>(&action);
        assert_eq!(
            uint(&action, "ListenerInputChange", "inputId"),
            index as u32
        );
    }
}
#[test]
fn wave_c9_event_002_hit_testing_via_state_machine_works() {
    let f = Fixture::new("bullet_man.riv", Some("Bullet Man"), None);
    assert_eq!(f.artboard.with_artboard(|a| a.state_machine_count()), 1);
    f.initial_advance();
    assert!(
        f.machine
            .with_instance(|m| m.get_trigger("Light").is_some())
    );
    f.down(71.0, 263.0);
    assert!(
        f.machine
            .with_instance(|m| m.get_trigger("Light").unwrap().fired())
    );
}
#[test]
fn wave_c9_event_003_hit_toggle_boolean_listener() {
    let f = Fixture::new("light_switch.riv", None, None);
    assert_eq!(f.artboard.with_artboard(|a| a.state_machine_count()), 1);
    f.initial_advance();
    assert!(
        f.machine
            .with_instance(|m| m.get_bool("On").unwrap().value())
    );
    f.down(150.0, 258.0);
    f.up(150.0, 258.0);
    assert!(
        !f.machine
            .with_instance(|m| m.get_bool("On").unwrap().value())
    );
    f.down(150.0, 258.0);
    f.up(150.0, 258.0);
    assert!(
        f.machine
            .with_instance(|m| m.get_bool("On").unwrap().value())
    );
}
#[test]
fn wave_c9_event_004_can_query_all_rive_events() {
    let f = Fixture::new("event_on_listener.riv", None, None);
    assert_eq!(f.artboard.with_artboard(|a| a.count::<Event>()), 4);
}
#[test]
fn wave_c9_event_005_can_query_rive_event_at_index() {
    let f = Fixture::new("event_on_listener.riv", None, None);
    let event = f
        .artboard
        .with_artboard(|a| a.object_handle_at::<Event>(0))
        .unwrap();
    assert_eq!(name(&event), "Somewhere.com");
}
#[test]
fn wave_c9_event_006_events_load_on_listener() {
    let f = Fixture::new("event_on_listener.riv", None, None);
    assert_eq!(f.artboard.with_artboard(|a| a.state_machine_count()), 1);
    f.initial_advance();
    assert_eq!(f.events().len(), 4);
    assert_eq!(
        read::<StateMachine, _>(&f.definition(), |m| m.listener_count()),
        1
    );
    let (target, action) = read_listener(&f.listener(0), |l| {
        assert_eq!(l.action_count(), 2);
        (l.target_id(), l.action(0).unwrap())
    });
    typed::<Shape>(&f.resolve(target));
    typed::<ListenerFireEvent>(&action);
    let id = read::<ListenerFireEvent, _>(&action, |a| a.event_id());
    assert_ne!(id, 0);
    let event = f.resolve(id);
    typed::<Event>(&event);
    assert_eq!(name(&event), "Footstep");
    assert_eq!(f.event_count(), 0);
    f.down(343.0, 116.0);
    f.up(343.0, 116.0);
    assert_eq!(f.event_count(), 2);
    assert_eq!(f.event_name(0), "Footstep");
    assert_eq!(f.event_name(1), "Event 3");
    f.advance(0.0);
    assert_eq!(f.event_count(), 0);
}
#[test]
fn wave_c9_event_007_events_load_on_state_and_transition() {
    let f = Fixture::new("events_on_states.riv", None, None);
    assert_eq!(f.artboard.with_artboard(|a| a.state_machine_count()), 1);
    f.initial_advance();
    assert_eq!(
        read::<StateMachine, _>(&f.definition(), |m| m.layer_count()),
        1
    );
    let layer = f.layer();
    assert_eq!(read::<StateMachineLayer, _>(&layer, |l| l.state_count()), 5);
    let entry = read::<StateMachineLayer, _>(&layer, |l| l.entry_state()).unwrap();
    let entry_transitions = transitions(&entry);
    assert_eq!(entry_transitions.len(), 1);
    let transition = &entry_transitions[0];
    assert_eq!(
        transition.with(|t| t.as_state_transition().unwrap().events().len()),
        Some(0)
    );
    let first = state_to(transition);
    typed::<AnimationState>(&first);
    assert_eq!(
        first.with(|s| s.state_machine_layer_component_events().unwrap().len()),
        Some(2)
    );
    let next = transitions(&first);
    assert_eq!(next.len(), 1);
    assert_eq!(
        next[0].with(|t| t.as_state_transition().unwrap().events().len()),
        Some(2)
    );
    assert_eq!(f.event_count(), 1);
    assert_eq!(f.event_name(0), "First");
    f.advance(1.0);
    assert_eq!(f.event_count(), 0);
    f.advance(1.0);
    assert_eq!(f.event_count(), 2);
    assert_eq!(f.event_name(0), "Second");
    assert_eq!(f.event_name(1), "Third");
    f.advance(1.0);
    assert_eq!(f.event_count(), 1);
    assert_eq!(f.event_name(0), "Fourth");
}
#[test]
fn wave_c9_event_008_timeline_events_load_and_report() {
    let f = Fixture::new("timeline_event_test.riv", None, None);
    assert_eq!(f.artboard.with_artboard(|a| a.state_machine_count()), 1);
    f.initial_advance();
    assert_eq!(f.event_count(), 0);
    f.advance(0.4);
    assert_eq!(f.event_count(), 0);
    f.advance(0.2);
    assert_eq!(f.event_count(), 1);
    assert_eq!(f.event_name(0), "Half");
    let actual = f64::from(
        f.machine
            .with_instance(|m| m.reported_event_at(0).seconds_delay),
    );
    let expected = f64::from(0.1f32);
    let relative_margin = f64::from(f32::EPSILON) * 100.0 * expected.abs();
    assert!((actual - expected).abs() <= relative_margin);
}
#[test]
fn wave_c9_event_011_view_model_listener_event_is_host_visible() {
    let f = Fixture::new("vm_listener_fire_event.riv", None, None);
    read_listener(&f.listener(0), |l| {
        assert_eq!(l.listener_input_type_count(), 1);
        let input = l.listener_input_type(0).unwrap();
        assert_eq!(
            input.with(|i| i.listener_input_type_value()),
            Some(Some(11))
        );
    });
    let instance = f
        .file
        .with_file_mut(|file| file.create_view_model_instance_at(0, 0))
        .unwrap();
    f.machine
        .with_instance_mut(|m| m.bind_view_model_instance(instance.clone()));
    f.machine.advance_and_apply(0.0);
    assert_eq!(f.event_count(), 0);
    let go = instance
        .with(|i| {
            i.as_view_model_instance()
                .unwrap()
                .property_value_named("go")
        })
        .flatten()
        .unwrap();
    go.with_downcast_mut::<ViewModelInstanceTrigger, _>(ViewModelInstanceTrigger::trigger)
        .unwrap();
    f.machine.advance_and_apply(0.016);
    assert_eq!(f.event_count(), 1);
    assert_eq!(f.event_name(0), "ding");
    f.machine.advance_and_apply(0.016);
    assert_eq!(f.event_count(), 0);
}
#[test]
fn wave_c9_input_001_file_with_state_machine_inputs_loads() {
    let f = Fixture::new("smi_test.riv", None, None);
    let nested = f
        .artboard
        .with_artboard(|a| a.find_handle::<NestedArtboard>("artboard to nest component"))
        .unwrap();
    read::<NestedArtboard, _>(&nested, |n| {
        assert_eq!(n.base.x(), 100.0);
        assert_eq!(n.base.y(), 100.0);
        assert_eq!(n.base.name(), "artboard to nest component");
        assert_eq!(n.base.artboard_id(), 1);
    });
    let machine = f
        .artboard
        .with_artboard(|a| a.find_handle::<NestedStateMachine>(""))
        .unwrap();
    assert_eq!(name(&machine), "");
    assert_eq!(uint(&machine, "NestedStateMachine", "animationId"), 0);
    let trigger = f
        .artboard
        .with_artboard(|a| a.find_handle::<NestedTrigger>(""))
        .unwrap();
    assert_eq!(name(&trigger), "");
    assert_eq!(uint(&trigger, "NestedTrigger", "inputId"), 0);
    let boolean = f
        .artboard
        .with_artboard(|a| a.find_handle::<NestedBool>(""))
        .unwrap();
    assert_eq!(name(&boolean), "");
    assert_eq!(uint(&boolean, "NestedBool", "inputId"), 1);
    let number = f
        .artboard
        .with_artboard(|a| a.find_handle::<NestedNumber>(""))
        .unwrap();
    assert_eq!(name(&number), "");
    assert_eq!(uint(&number, "NestedNumber", "inputId"), 2);
}
