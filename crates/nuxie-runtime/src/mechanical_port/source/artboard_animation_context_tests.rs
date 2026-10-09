//! The runtime animation context resolves the current table and releases its root
//! before applying a keyframe. The authored compatibility route remains open.
use super::*;
use crate::source::{
    animation::{
        interpolating_keyframe::KeyFrameValueContext, keyed_property::KeyedProperty,
        keyframe_double::KeyFrameDouble,
    },
    core::{
        Core, CoreObject, PropertySetterCompletion, binary_reader::BinaryReader,
        field_types::core_callback_type::CallbackData,
    },
    generated::{
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
        node_base::NodeBase,
    },
    node::Node,
};
use std::{
    any::Any,
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
};

fn context(root: &RuntimeArtboardInstanceHandle) -> RuntimeArtboardObjectContext {
    let handle = root.core_handle();
    RuntimeArtboardObjectContext {
        arena: handle.retain_arena().unwrap(),
        root: handle,
    }
}

struct EditTableOnValue {
    root: RuntimeArtboardInstanceWeakHandle,
    replacement: CoreHandle,
    calls: Cell<usize>,
}
impl KeyFrameValueContext for EditTableOnValue {
    fn bool_value(&self, _: &CoreHandle) -> Option<bool> {
        None
    }
    fn string_value(&self, _: &CoreHandle) -> Option<String> {
        None
    }
    fn color_value(&self, _: &CoreHandle) -> Option<i32> {
        None
    }
    fn number_value(&self, _: &CoreHandle) -> Option<f32> {
        let call = self.calls.get();
        self.calls.set(call + 1);
        // This would panic if resolve retained the root through keyframe apply.
        self.root
            .with_artboard_mut(|root| {
                root.base.objects[1] = Some(self.replacement.clone());
            })
            .unwrap();
        Some(if call == 0 { 10.0 } else { 20.0 })
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

#[test]
fn each_keyed_object_resolves_after_the_previous_keyframe_callback() {
    let root = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    let arena = context(&root).arena;
    let first = arena.insert(Node::default());
    let second = arena.insert(Node::default());
    root.with_artboard_mut(|root| root.base.objects.push(Some(first.clone())));
    let mut animation = LinearAnimation::default();
    for _ in 0..2 {
        let mut property = KeyedProperty::default();
        property
            .base
            .set_property_key_value(NodeBase::X_PROPERTY_KEY.into());
        property.add_key_frame(arena.insert(KeyFrameDouble::default()));
        let mut keyed = KeyedObject::default();
        keyed.base.set_object_id_value(1);
        keyed.add_keyed_property(arena.insert(property));
        animation.add_keyed_object(arena.insert(keyed));
    }
    let values = EditTableOnValue {
        root: root.downgrade(),
        replacement: second.clone(),
        calls: Cell::new(0),
    };
    root.apply_linear_animation(&mut animation, 0.0, 1.0, Some(&values));
    assert_eq!(values.calls.get(), 2);
    assert_eq!(
        CoreRegistry::get_double_handle(&first, NodeBase::X_PROPERTY_KEY.into()),
        Some(10.0)
    );
    assert_eq!(
        CoreRegistry::get_double_handle(&second, NodeBase::X_PROPERTY_KEY.into()),
        Some(20.0)
    );
}

#[test]
fn runtime_resolution_preserves_shared_reads_and_mutable_loan_conflicts() {
    let root = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    let lookup = context(&root);
    root.with_artboard(|_| assert_eq!(lookup.resolve_handle(0), Some(lookup.root.clone())));
    assert!(
        catch_unwind(AssertUnwindSafe(
            || root.with_artboard_mut(|_| lookup.resolve_handle(0))
        ))
        .is_err()
    );
    assert_eq!(lookup.resolve_handle(0), Some(lookup.root.clone()));
    assert!(lookup.resolve_handle(u32::MAX).is_none());
    root.with_artboard_mut(|root| root.base.objects[0] = None);
    assert!(lookup.resolve_handle(0).is_none());
}

#[test]
fn context_does_not_keep_a_runtime_root_alive_or_resolve_a_reused_generation() {
    let mut instance = ArtboardInstance::default();
    // Imported instances retire their runtime root in Artboard::drop.
    instance.base.is_instance = true;
    let root = RuntimeArtboardInstanceHandle::new(instance);
    let lookup = context(&root);
    let weak = root.downgrade();
    let identity = lookup.root.identity_key();
    drop(root);
    assert!(weak.upgrade().is_none());
    assert!(lookup.resolve_handle(0).is_none());
    let replacement = lookup.arena.insert(Artboard::default());
    assert_eq!(replacement.identity_key().1, identity.1);
    replacement
        .with_downcast_mut::<Artboard, _>(|owner| owner.objects.push(Some(replacement.clone())))
        .unwrap();
    assert!(lookup.resolve_handle(0).is_none());
    assert_eq!(
        replacement
            .with_artboard(|owner| owner.resolve_handle(0))
            .flatten(),
        Some(replacement)
    );
}

struct ProjectedArtboard {
    inner: Artboard,
    calls: Rc<Cell<usize>>,
}
impl ProjectedArtboard {
    fn subtype(key: u16) -> bool {
        key == 65534
    }
}
impl CoreCapabilities for ProjectedArtboard {}
impl CoreObject for ProjectedArtboard {
    fn core(&self) -> &Core {
        CoreObject::core(&self.inner)
    }
    fn core_mut(&mut self) -> &mut Core {
        CoreObject::core_mut(&mut self.inner)
    }
    fn core_type(&self) -> u16 {
        65534
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::subtype
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        self.inner.deserialize(key, reader)
    }
    fn as_any(&self) -> &dyn Any {
        self.calls.set(self.calls.get() + 1);
        &self.inner
    }
}
impl CoreRegistryObject for ProjectedArtboard {
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
        self.inner.set_uint_with_completion(f, v, c);
    }
    fn set_string_with_completion(
        &mut self,
        f: CoreField,
        v: String,
        c: &mut PropertySetterCompletion,
    ) {
        self.inner.set_string_with_completion(f, v, c);
    }
    fn set_color_with_completion(
        &mut self,
        f: CoreField,
        v: i32,
        c: &mut PropertySetterCompletion,
    ) {
        self.inner.set_color_with_completion(f, v, c);
    }
    fn set_bool_with_completion(
        &mut self,
        f: CoreField,
        v: bool,
        c: &mut PropertySetterCompletion,
    ) {
        self.inner.set_bool_with_completion(f, v, c);
    }
    fn set_double_with_completion(
        &mut self,
        f: CoreField,
        v: f32,
        c: &mut PropertySetterCompletion,
    ) {
        self.inner.set_double_with_completion(f, v, c);
    }
    fn set_callback_with_completion(
        &mut self,
        f: CoreField,
        v: CallbackData<'_>,
        c: &mut PropertySetterCompletion,
    ) {
        self.inner.set_callback_with_completion(f, v, c);
    }
    fn set_int_with_completion(&mut self, f: CoreField, v: i32, c: &mut PropertySetterCompletion) {
        self.inner.set_int_with_completion(f, v, c);
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

#[test]
fn authored_projection_runs_once_per_lookup_even_with_non_artboard_type_key() {
    let arena = CoreArena::default();
    let first = arena.insert(Node::default());
    let second = arena.insert(Node::default());
    let calls = Rc::new(Cell::new(0));
    let mut inner = Artboard::default();
    inner.objects.push(Some(first.clone()));
    let root = arena.insert(ProjectedArtboard {
        inner,
        calls: calls.clone(),
    });
    let lookup = RuntimeArtboardObjectContext {
        arena,
        root: root.clone(),
    };
    calls.set(0);
    assert_eq!(lookup.resolve_handle(0), Some(first));
    assert_eq!(calls.get(), 1);
    root.with_mut(|object| {
        object
            .as_registry_any_mut()
            .downcast_mut::<ProjectedArtboard>()
            .unwrap()
            .inner
            .objects[0] = Some(second.clone())
    })
    .unwrap();
    assert_eq!(lookup.resolve_handle(0), Some(second));
    assert_eq!(calls.get(), 2);
    assert!(lookup.resolve_handle(99).is_none());
    assert_eq!(calls.get(), 3);
}
