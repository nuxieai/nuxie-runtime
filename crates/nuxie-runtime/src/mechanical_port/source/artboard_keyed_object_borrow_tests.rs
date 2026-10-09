//! Borrowed animation entries keep the existing owned custom hook and released
//! definition boundaries. No test relies on changing the pinned object table.
use super::*;
use crate::source::{
    animation::{
        interpolating_keyframe::KeyFrameValueContext, keyed_property::KeyedProperty,
        keyframe_double::KeyFrameDouble, linear_animation::LinearAnimationOwner,
    },
    generated::node_base::NodeBase,
    node::Node,
};
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
};

struct Values<F>(F);
impl<F: Fn(&CoreHandle) -> Option<f32>> KeyFrameValueContext for Values<F> {
    fn bool_value(&self, _: &CoreHandle) -> Option<bool> {
        None
    }
    fn string_value(&self, _: &CoreHandle) -> Option<String> {
        None
    }
    fn color_value(&self, _: &CoreHandle) -> Option<i32> {
        None
    }
    fn number_value(&self, keyframe: &CoreHandle) -> Option<f32> {
        (self.0)(keyframe)
    }
    fn stateful_interpolator_transform_value(
        &self,
        _: &CoreHandle,
        _: &CoreHandle,
        _: f32,
        _: f32,
        _: f32,
    ) -> Option<f32> {
        None
    }
    fn stateful_interpolator_transform(
        &self,
        _: &CoreHandle,
        _: &CoreHandle,
        _: f32,
    ) -> Option<f32> {
        None
    }
}
fn property(arena: &CoreArena, key: u16, value: f32) -> CoreHandle {
    let mut frame = KeyFrameDouble::default();
    frame.base.set_value_value(value);
    let mut property = KeyedProperty::default();
    property.base.set_property_key_value(key.into());
    property.add_key_frame(arena.insert(frame));
    arena.insert(property)
}
fn keyed(arena: &CoreArena, properties: &[CoreHandle]) -> CoreHandle {
    let mut keyed = KeyedObject::default();
    keyed.base.set_object_id_value(1);
    for property in properties {
        keyed.add_keyed_property(property.clone());
    }
    arena.insert(keyed)
}
fn owner(arena: &CoreArena, objects: &[CoreHandle], runtime: bool) -> LinearAnimationOwner {
    let mut animation = LinearAnimation::default();
    for object in objects {
        animation.add_keyed_object(object.clone());
    }
    if runtime {
        LinearAnimationOwner::Runtime(Rc::new(RefCell::new(animation)))
    } else {
        LinearAnimationOwner::Authored(arena.insert(animation))
    }
}

struct OwnedHook {
    animation: Option<LinearAnimationOwner>,
    replacement: CoreHandle,
    received: Vec<CoreHandle>,
}
impl LinearAnimationArtboard for OwnedHook {
    // Deliberately implements only the established owned entry.
    fn apply_keyed_object(
        &mut self,
        object: CoreHandle,
        _: f32,
        _: f32,
        _: Option<&dyn KeyFrameValueContext>,
    ) {
        self.received.push(object);
        if self.received.len() == 1 {
            if let Some(animation) = &self.animation {
                animation.with_mut(|animation| {
                    *animation = LinearAnimation::default();
                    animation.add_keyed_object(self.replacement.clone());
                });
            }
        }
    }
}
#[test]
fn owned_custom_hook_keeps_duplicates_and_retains_handles_after_snapshot_edit() {
    for runtime in [false, true] {
        let arena = CoreArena::default();
        let first = keyed(&arena, &[]);
        let second = keyed(&arena, &[]);
        let replacement = keyed(&arena, &[]);
        let animation = owner(
            &arena,
            &[first.clone(), first.clone(), second.clone()],
            runtime,
        );
        let mut hook = OwnedHook {
            animation: Some(animation.clone()),
            replacement: replacement.clone(),
            received: vec![],
        };
        animation.apply(&mut hook, 0.0, 1.0, None);
        assert_eq!(hook.received, [first.clone(), first, second]);
        animation.apply(&mut hook, 0.0, 1.0, None);
        assert_eq!(hook.received.last(), Some(&replacement));
        assert_eq!(hook.received.len(), 4);
    }
}
#[test]
fn direct_animation_custom_hook_still_receives_an_owned_identity() {
    let arena = CoreArena::default();
    let object = keyed(&arena, &[]);
    let mut animation = LinearAnimation::default();
    animation.add_keyed_object(object.clone());
    let mut hook = OwnedHook {
        animation: None,
        replacement: object.clone(),
        received: vec![],
    };
    animation.apply(&mut hook, 0.0, 1.0, None);
    drop(animation);
    assert_eq!(hook.received, [object]);
    assert!(hook.received[0].is_alive());
}

fn native_case(
    target_context: &mut dyn LinearAnimationArtboard,
    arena: &CoreArena,
    target: &CoreHandle,
) {
    let x = property(arena, NodeBase::X_PROPERTY_KEY, 3.0);
    let y = property(arena, NodeBase::Y_PROPERTY_KEY, 7.0);
    let keyed = keyed(arena, &[x]);
    let animation = owner(arena, &[keyed.clone(), keyed.clone()], true);
    let calls = Cell::new(0);
    let values = Values(|_: &CoreHandle| {
        calls.set(calls.get() + 1);
        // Neither the animation nor keyed owner may remain borrowed here.
        animation.with_mut(|animation| animation.base.set_fps_value(120));
        keyed
            .with_downcast_mut::<KeyedObject, _>(|keyed| {
                keyed.base.set_object_id_value(99);
                keyed.add_keyed_property(y.clone());
            })
            .unwrap();
        Some(18.0)
    });
    animation.apply(target_context, 0.0, 1.0, Some(&values));
    // The first property's snapshot predates the append. The second keyed
    // occurrence sees the fresh missing target ID and does not apply it.
    assert_eq!(calls.get(), 1);
    assert_eq!(
        CoreRegistry::get_double_handle(target, NodeBase::X_PROPERTY_KEY.into()),
        Some(18.0)
    );
    assert_eq!(
        CoreRegistry::get_double_handle(target, NodeBase::Y_PROPERTY_KEY.into()),
        Some(0.0)
    );
}
#[test]
fn all_four_native_entries_release_definition_loans_and_read_each_current_object_id() {
    let arena = CoreArena::default();
    let target = arena.insert(Node::default());
    native_case(
        &mut ArtboardObjectContext {
            arena: arena.clone(),
            objects: vec![None, Some(target.clone())],
        },
        &arena,
        &target,
    );
    let mut artboard = Artboard::default();
    artboard.objects = vec![None, Some(target.clone())];
    native_case(&mut artboard, &arena, &target);
    let mut instance = ArtboardInstance::default();
    instance.base.objects = vec![None, Some(target.clone())];
    native_case(&mut instance, &arena, &target);
    let root = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    root.with_artboard_mut(|root| root.base.objects.push(Some(target.clone())));
    native_case(
        &mut RuntimeArtboardObjectContext {
            arena: arena.clone(),
            root: root.core_handle(),
        },
        &arena,
        &target,
    );
}

struct ResolveEdit {
    arena: CoreArena,
    target: CoreHandle,
    keyed: CoreHandle,
    added_property: CoreHandle,
    ids: Vec<u32>,
}
impl CoreContext for ResolveEdit {
    fn core_arena(&self) -> &CoreArena {
        &self.arena
    }
    fn resolve_handle(&self, _: u32) -> Option<CoreHandle> {
        Some(self.target.clone())
    }
}
impl KeyedObjectContext for ResolveEdit {
    fn resolves_object(&self, _: u32) -> bool {
        true
    }
    fn resolve_object(&mut self, id: u32) -> Option<CoreHandle> {
        self.ids.push(id);
        self.keyed
            .with_downcast_mut::<KeyedObject, _>(|keyed| {
                keyed.base.set_object_id_value(99);
                keyed.add_keyed_property(self.added_property.clone());
            })
            .unwrap();
        Some(self.target.clone())
    }
    fn object_supports_property(&self, _: u32, _: u32) -> bool {
        true
    }
    fn overrides_keyed_interpolation(&self, _: &CoreHandle, _: u32) -> bool {
        false
    }
}
impl LinearAnimationArtboard for ResolveEdit {
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
fn open_target_resolution_precedes_the_fresh_property_snapshot() {
    let arena = CoreArena::default();
    let target = arena.insert(Node::default());
    let keyed = keyed(&arena, &[]);
    let animation = owner(&arena, &[keyed.clone()], true);
    let mut context = ResolveEdit {
        arena: arena.clone(),
        target: target.clone(),
        keyed,
        added_property: property(&arena, NodeBase::X_PROPERTY_KEY, 9.0),
        ids: vec![],
    };
    animation.apply(&mut context, 0.0, 1.0, None);
    assert_eq!(context.ids, [1]);
    assert_eq!(
        CoreRegistry::get_double_handle(&target, NodeBase::X_PROPERTY_KEY.into()),
        Some(9.0)
    );
}

fn retire_definitions_case(panic_after_retirement: bool) {
    let arena = CoreArena::default();
    let target = arena.insert(Node::default());
    let keyed = keyed(&arena, &[property(&arena, NodeBase::X_PROPERTY_KEY, 0.0)]);
    let animation = owner(&arena, &[keyed.clone()], false);
    let LinearAnimationOwner::Authored(animation_handle) = &animation else {
        unreachable!()
    };
    let values = Values(|_: &CoreHandle| {
        drop(arena.remove(animation_handle).unwrap());
        drop(arena.remove(&keyed).unwrap());
        assert!(!animation_handle.is_alive());
        assert!(!keyed.is_alive());
        let replacement = arena.insert(KeyedObject::default());
        assert_eq!(replacement.identity_key().1, keyed.identity_key().1);
        assert_ne!(replacement.identity_key().2, keyed.identity_key().2);
        assert!(!keyed.is_alive());
        assert!(
            !panic_after_retirement,
            "deliberate callback unwind after retirement"
        );
        Some(12.0)
    });
    let mut context = ArtboardObjectContext {
        arena: arena.clone(),
        objects: vec![None, Some(target.clone())],
    };
    let result = catch_unwind(AssertUnwindSafe(|| {
        animation.apply(&mut context, 0.0, 1.0, Some(&values))
    }));
    assert_eq!(result.is_err(), panic_after_retirement);
    assert!(!animation_handle.is_alive());
    assert!(!keyed.is_alive());
    assert_eq!(
        CoreRegistry::get_double_handle(&target, NodeBase::X_PROPERTY_KEY.into()),
        Some(if panic_after_retirement { 0.0 } else { 12.0 })
    );
}
#[test]
fn native_callback_can_retire_and_reuse_definition_slots_before_return() {
    retire_definitions_case(false);
}
#[test]
fn native_callback_unwind_keeps_definition_retirement_and_reuse() {
    retire_definitions_case(true);
}
