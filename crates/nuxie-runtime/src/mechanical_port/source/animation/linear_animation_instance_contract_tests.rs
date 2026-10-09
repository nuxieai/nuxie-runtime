use super::*;
use crate::mechanical_port::source::{
    animation::{
        keyed_object::KeyedObject, keyed_property::KeyedProperty,
        keyframe_callback::KeyFrameCallback, listener_fire_event::ListenerFireEvent,
        listener_invocation::ListenerInvocation, state_machine::StateMachine,
        state_machine_instance::StateMachineInstance,
        state_machine_listener_single::StateMachineListenerSingle,
    },
    artboard::{ArtboardInstance, RuntimeArtboardInstanceHandle},
    core::{
        Core, CoreArena, CoreObject, PropertySetterCompletion, binary_reader::BinaryReader,
        field_types::core_callback_type::CallbackData,
    },
    event::Event,
    generated::{
        animation::{
            linear_animation_base::LinearAnimationBase, listener_action_base::ListenerActionBase,
        },
        core_registry::{CoreCapabilities, CoreField, CoreRegistry, CoreRegistryObject},
        event_base::EventBase,
    },
    listener_type::ListenerType,
    nested_artboard::NestedArtboard,
};
use std::{any::Any, cell::Cell};

struct Reporter<F>(F);
impl<F: FnMut(u32, u32, f32)> KeyedCallbackReporter for Reporter<F> {
    fn report_keyed_callback(&mut self, object: u32, property: u32, delay: f32) {
        (self.0)(object, property, delay);
    }
}

fn callback_animation(arena: &CoreArena) -> LinearAnimation {
    let mut frame = KeyFrameCallback::default();
    frame.base.base.compute_seconds(60);
    let mut property = KeyedProperty::default();
    property
        .base
        .set_property_key_value(EventBase::TRIGGER_PROPERTY_KEY.into());
    property.add_key_frame(arena.insert(frame));
    let mut object = KeyedObject::default();
    object.base.set_object_id_value(1);
    object.add_keyed_property(arena.insert(property));
    let mut animation = LinearAnimation::default();
    animation.base.set_enable_work_area_value(true);
    animation.base.set_work_start_value(0);
    animation.base.set_work_end_value(60);
    animation.add_keyed_object(arena.insert(object));
    animation
}

fn change_bounds(animation: &mut LinearAnimation) {
    animation.base.set_fps_value(120);
    animation.base.set_work_end_value(90);
}

fn assert_fresh_bounds(instance: &LinearAnimationInstance) {
    assert_eq!(instance.time(), 0.75);
    assert_eq!(instance.direction(), 1.0);
    assert!(instance.did_loop());
    assert!((instance.spilled_time() - 0.35).abs() < 0.000_001);
    assert_eq!(instance.total_time(), 1.1);
}

#[test]
fn invalid_loop_value_leaves_time_and_direction_unwrapped() {
    let mut instance = LinearAnimationInstance::new_runtime(
        Rc::new(RefCell::new(LinearAnimation::default())),
        RuntimeArtboardInstanceWeakHandle::default(),
        1.0,
    );
    instance.set_loop_value(99);
    assert_eq!(instance.loop_().value(), 99);
    assert!(instance.advance(1.1, None));
    assert_eq!(instance.time(), 1.1);
    assert_eq!(instance.direction(), 1.0);
    assert!(!instance.did_loop());
    assert_eq!(instance.spilled_time(), 0.0);
}

#[test]
fn loop_accessors_preserve_unnamed_wire_values_and_signed_override_bits() {
    for raw in [0, 1, 2, 99, u32::MAX] {
        let mut animation = LinearAnimation::default();
        animation.base.set_loop_value_value(raw);
        assert_eq!(animation.loop_kind().value(), raw);
        let mut instance = LinearAnimationInstance::new_runtime(
            Rc::new(RefCell::new(animation)),
            RuntimeArtboardInstanceWeakHandle::default(),
            1.0,
        );
        assert_eq!(instance.loop_().value(), raw);
        instance.set_loop_value(-2);
        assert_eq!(instance.loop_().value(), (-2_i32) as u32);
        instance.set_loop_value(-1);
        assert_eq!(instance.loop_().value(), raw);
    }
}

#[cfg(debug_assertions)]
#[test]
fn global_to_local_seconds_preserves_source_positive_range_assertion() {
    for raw in [1, 2] {
        let mut animation = LinearAnimation::default();
        animation.base.set_duration_value(0);
        animation.base.set_loop_value_value(raw);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            animation.global_to_local_seconds(1.0)
        }))
        .is_err());
    }
}

#[test]
fn runtime_reporter_can_change_animation_bounds_before_frame_arithmetic() {
    let arena = CoreArena::default();
    let animation = Rc::new(RefCell::new(callback_animation(&arena)));
    let mut instance = LinearAnimationInstance::new_runtime(
        animation.clone(),
        RuntimeArtboardInstanceWeakHandle::default(),
        1.0,
    );
    let mut calls = 0;
    let mut reporter = Reporter(|_: u32, _: u32, _: f32| {
        calls += 1;
        change_bounds(&mut animation.borrow_mut());
    });
    assert!(!instance.advance(1.1, Some(&mut reporter)));
    assert_eq!(calls, 1);
    assert_fresh_bounds(&instance);
}

#[test]
fn authored_reporter_can_change_animation_bounds_before_frame_arithmetic() {
    let arena = CoreArena::default();
    let animation = arena.insert(callback_animation(&arena));
    let mut instance = LinearAnimationInstance::new(
        animation.clone(),
        RuntimeArtboardInstanceWeakHandle::default(),
        1.0,
    );
    let mut calls = 0;
    let mut reporter = Reporter(|_: u32, _: u32, _: f32| {
        calls += 1;
        assert!(CoreRegistry::set_uint_handle(
            &animation,
            LinearAnimationBase::FPS_PROPERTY_KEY.into(),
            120
        ));
        assert!(CoreRegistry::set_uint_handle(
            &animation,
            LinearAnimationBase::WORK_END_PROPERTY_KEY.into(),
            90
        ));
    });
    assert!(!instance.advance(1.1, Some(&mut reporter)));
    assert_eq!(calls, 1);
    assert_fresh_bounds(&instance);
}

#[test]
fn copy_default_constructs_nested_event_notifier() {
    let arena = CoreArena::default();
    let nested_artboard = arena.insert(NestedArtboard::default());
    let artboard = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    let machine =
        StateMachineInstance::new(arena.insert(StateMachine::default()), artboard.downgrade());
    let mut instance = LinearAnimationInstance::new_runtime(
        Rc::new(RefCell::new(LinearAnimation::default())),
        artboard.downgrade(),
        1.0,
    );
    instance.set_nested_artboard(nested_artboard.clone());
    instance.add_nested_event_listener(machine.downgrade());
    instance.set_time(0.25);
    instance.set_direction(-1);
    instance.set_loop_value(1);
    let copy = instance.clone();
    assert_eq!(copy.time(), 0.25);
    assert_eq!(copy.direction(), -1.0);
    assert_eq!(copy.loop_value(), 1);
    assert_eq!(
        instance.nested_event_notifier.nested_artboard(),
        Some(nested_artboard)
    );
    assert_eq!(
        instance
            .nested_event_notifier
            .nested_event_listeners()
            .len(),
        1
    );
    assert!(copy.nested_event_notifier.nested_artboard().is_none());
    assert!(
        copy.nested_event_notifier
            .nested_event_listeners()
            .is_empty()
    );
}

// A real listener-action virtual override changes the authored animation during
// the nested Event dispatch. Registry storage delegates to a native action;
// only perform() is the source-supported custom behavior being observed.
struct ChangeAnimationBounds {
    base: ListenerFireEvent,
    animation: Rc<RefCell<LinearAnimation>>,
    calls: Rc<Cell<usize>>,
}
impl ChangeAnimationBounds {
    fn subtype(key: u16) -> bool {
        key == 65533 || ListenerActionBase::is_type_of(key)
    }
}
impl CoreCapabilities for ChangeAnimationBounds {
    fn listener_action_perform(
        &mut self,
        _: &mut StateMachineInstance,
        _: &ListenerInvocation,
    ) -> bool {
        self.calls.set(self.calls.get() + 1);
        change_bounds(&mut self.animation.borrow_mut());
        true
    }
}
impl CoreObject for ChangeAnimationBounds {
    fn core(&self) -> &Core {
        self.base.core()
    }
    fn core_mut(&mut self) -> &mut Core {
        self.base.core_mut()
    }
    fn core_type(&self) -> u16 {
        65533
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::subtype
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        self.base.deserialize(key, reader)
    }
}
impl CoreRegistryObject for ChangeAnimationBounds {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn set_uint_with_completion(&mut self, f: CoreField, v: u32, c: &mut PropertySetterCompletion) {
        self.base.set_uint_with_completion(f, v, c);
    }
    fn set_string_with_completion(
        &mut self,
        f: CoreField,
        v: String,
        c: &mut PropertySetterCompletion,
    ) {
        self.base.set_string_with_completion(f, v, c);
    }
    fn set_color_with_completion(
        &mut self,
        f: CoreField,
        v: i32,
        c: &mut PropertySetterCompletion,
    ) {
        self.base.set_color_with_completion(f, v, c);
    }
    fn set_bool_with_completion(
        &mut self,
        f: CoreField,
        v: bool,
        c: &mut PropertySetterCompletion,
    ) {
        self.base.set_bool_with_completion(f, v, c);
    }
    fn set_double_with_completion(
        &mut self,
        f: CoreField,
        v: f32,
        c: &mut PropertySetterCompletion,
    ) {
        self.base.set_double_with_completion(f, v, c);
    }
    fn set_callback_with_completion(
        &mut self,
        f: CoreField,
        v: CallbackData<'_>,
        c: &mut PropertySetterCompletion,
    ) {
        self.base.set_callback_with_completion(f, v, c);
    }
    fn set_int_with_completion(&mut self, f: CoreField, v: i32, c: &mut PropertySetterCompletion) {
        self.base.set_int_with_completion(f, v, c);
    }
    fn get_uint(&mut self, f: CoreField) -> u32 {
        self.base.get_uint(f)
    }
    fn get_string(&mut self, f: CoreField) -> String {
        self.base.get_string(f)
    }
    fn get_color(&mut self, f: CoreField) -> i32 {
        self.base.get_color(f)
    }
    fn get_bool(&mut self, f: CoreField) -> bool {
        self.base.get_bool(f)
    }
    fn get_double(&mut self, f: CoreField) -> f32 {
        self.base.get_double(f)
    }
    fn get_int(&mut self, f: CoreField) -> i32 {
        self.base.get_int(f)
    }
}

#[test]
fn self_reporter_dispatches_nested_event_before_frame_arithmetic() {
    let arena = CoreArena::default();
    let animation = Rc::new(RefCell::new(callback_animation(&arena)));
    let artboard = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    let event = arena.insert(Event::default());
    artboard.with_artboard_mut(|artboard| artboard.base.add_object(Some(event)));
    let calls = Rc::new(Cell::new(0));
    let action = arena.insert(ChangeAnimationBounds {
        base: ListenerFireEvent::default(),
        animation: animation.clone(),
        calls: calls.clone(),
    });
    let mut listener = StateMachineListenerSingle::default();
    listener
        .base
        .set_listener_type_value_value(ListenerType::Event as u32);
    listener.base.set_event_id_value(1);
    listener.base.base.base.set_target_id_value(0);
    listener.base.base.add_action(action);
    let mut definition = StateMachine::default();
    definition.add_listener(arena.insert(listener));
    let machine = StateMachineInstance::new(arena.insert(definition), artboard.downgrade());
    let mut instance = LinearAnimationInstance::new_runtime(animation, artboard.downgrade(), 1.0);
    instance.add_nested_event_listener(machine.downgrade());

    assert!(!instance.advance_and_report_to_self(1.1));
    assert_eq!(calls.get(), 1);
    assert_fresh_bounds(&instance);
}

struct ApplyHookContext {
    arena: CoreArena,
    target: CoreHandle,
    property: CoreHandle,
    last_frame: CoreHandle,
    change_property: bool,
    parents: Option<(LinearAnimationOwner, CoreHandle)>,
    calls: Cell<usize>,
}
impl crate::mechanical_port::source::core_context::CoreContext for ApplyHookContext {
    fn core_arena(&self) -> &CoreArena {
        &self.arena
    }
    fn resolve_handle(&self, _: u32) -> Option<CoreHandle> {
        Some(self.target.clone())
    }
}
impl crate::mechanical_port::source::animation::keyed_object::KeyedObjectContext
    for ApplyHookContext
{
    fn resolves_object(&self, _: u32) -> bool {
        true
    }
    fn resolve_object(&mut self, _: u32) -> Option<CoreHandle> {
        Some(self.target.clone())
    }
    fn object_supports_property(&self, _: u32, _: u32) -> bool {
        true
    }
    fn overrides_keyed_interpolation(&self, _: &CoreHandle, key: u32) -> bool {
        use crate::mechanical_port::source::{
            animation::keyframe_double::KeyFrameDouble, generated::node_base::NodeBase,
        };
        assert_eq!(key, u32::from(NodeBase::X_PROPERTY_KEY));
        self.calls.set(self.calls.get() + 1);
        self.last_frame
            .with_downcast_mut::<KeyFrameDouble, _>(|frame| {
                frame.base.base.base.base.set_frame_value(120);
                frame.base.base.base.compute_seconds(60);
            })
            .unwrap();
        if self.change_property {
            self.property
                .with_downcast_mut::<KeyedProperty, _>(|property| {
                    property
                        .base
                        .set_property_key_value(NodeBase::Y_PROPERTY_KEY.into());
                })
                .unwrap();
        }
        if let Some((animation, keyed_object)) = &self.parents {
            animation.with_mut(|animation| animation.base.set_fps_value(120));
            keyed_object
                .with_downcast_mut::<KeyedObject, _>(|object| {
                    object.base.set_object_id_value(99);
                })
                .unwrap();
        }
        true
    }
}
fn apply_hook_case(change_property: bool, owned_application: Option<bool>) {
    use crate::mechanical_port::source::{
        animation::keyframe_double::KeyFrameDouble, generated::node_base::NodeBase, node::Node,
    };
    let arena = CoreArena::default();
    let target = arena.insert(Node::default());
    let mut first = KeyFrameDouble::default();
    first.base.base.base.compute_seconds(60);
    let mut last = KeyFrameDouble::default();
    last.base.base.base.base.set_frame_value(60);
    last.base.base.base.compute_seconds(60);
    last.base.set_value_value(10.0);
    let last = arena.insert(last);
    let mut property = KeyedProperty::default();
    property
        .base
        .set_property_key_value(NodeBase::X_PROPERTY_KEY.into());
    property.add_key_frame(arena.insert(first));
    property.add_key_frame(last.clone());
    let property = arena.insert(property);
    let mut object = KeyedObject::default();
    object.add_keyed_property(property.clone());
    let parents = owned_application.map(|runtime| {
        let keyed_object = arena.insert(std::mem::take(&mut object));
        let mut animation = LinearAnimation::default();
        animation.add_keyed_object(keyed_object.clone());
        let owner = if runtime {
            LinearAnimationOwner::Runtime(Rc::new(RefCell::new(animation)))
        } else {
            LinearAnimationOwner::Authored(arena.insert(animation))
        };
        (owner, keyed_object)
    });
    let mut context = ApplyHookContext {
        arena,
        target: target.clone(),
        property,
        last_frame: last,
        change_property,
        parents: parents.clone(),
        calls: Cell::new(0),
    };
    if let Some((animation, keyed_object)) = &parents {
        animation.apply(&mut context, 1.5, 0.25, None);
        assert_eq!(animation.with(|animation| animation.base.fps()), 120);
        assert_eq!(
            keyed_object.with_downcast::<KeyedObject, _>(|object| object.base.object_id()),
            Some(99)
        );
    } else {
        object.apply(&mut context, 1.5, 0.25, None);
    }
    assert_eq!(context.calls.get(), 1);
    let result_key = if change_property {
        NodeBase::Y_PROPERTY_KEY
    } else {
        NodeBase::X_PROPERTY_KEY
    };
    assert_eq!(
        CoreRegistry::get_double_handle(&target, result_key.into()),
        Some(10.0)
    );
    if change_property {
        assert_eq!(
            CoreRegistry::get_double_handle(&target, NodeBase::X_PROPERTY_KEY.into()),
            Some(0.0)
        );
    }
}
#[test]
fn apply_computes_frame_index_before_the_override_hook() {
    apply_hook_case(false, None);
}
#[test]
fn apply_reads_property_key_after_the_override_hook_without_retaining_its_loan() {
    apply_hook_case(true, None);
}

impl crate::mechanical_port::source::animation::linear_animation::LinearAnimationArtboard
    for ApplyHookContext
{
    fn apply_keyed_object(
        &mut self,
        object: CoreHandle,
        time: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        KeyedObject::apply_occurrence(&object, self, time, mix, context);
    }
}
#[test]
fn runtime_application_hook_can_edit_both_parent_definitions() {
    apply_hook_case(true, Some(true));
}
#[test]
fn authored_application_hook_can_edit_both_parent_definitions() {
    apply_hook_case(true, Some(false));
}
