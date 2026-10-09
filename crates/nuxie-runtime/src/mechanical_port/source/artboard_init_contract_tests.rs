//! Initialization phase reads compared with pinned C++ Artboard::initialize.
use super::*;
use crate::mechanical_port::source::{
    core::{
        Core, CoreObject, PropertySetterCompletion, binary_reader::BinaryReader,
        field_types::core_callback_type::CallbackData,
    },
    generated::core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
    shapes::shape::Shape,
};
use std::any::Any;

struct AppendAnimation {
    shape: Shape,
    root: CoreHandle,
    animation: CoreHandle,
}
impl AppendAnimation {
    fn subtype(key: u16) -> bool {
        key == 65534
    }
}
impl CoreCapabilities for AppendAnimation {}
impl CoreObject for AppendAnimation {
    fn core(&self) -> &Core {
        self.shape.core()
    }
    fn core_mut(&mut self) -> &mut Core {
        self.shape.core_mut()
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
        self.shape.deserialize(key, reader)
    }
    fn on_added_dirty(&mut self, _: &mut dyn CoreContext) -> StatusCode {
        self.root
            .with_downcast_mut::<Artboard, _>(|root| root.add_animation(self.animation.clone()))
            .unwrap();
        StatusCode::Ok
    }
    fn on_added_clean(&mut self, _: &mut dyn CoreContext) -> StatusCode {
        StatusCode::InvalidObject
    }
}
impl CoreRegistryObject for AppendAnimation {
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
        self.shape.set_uint_with_completion(f, v, c);
    }
    fn set_string_with_completion(
        &mut self,
        f: CoreField,
        v: String,
        c: &mut PropertySetterCompletion,
    ) {
        self.shape.set_string_with_completion(f, v, c);
    }
    fn set_color_with_completion(
        &mut self,
        f: CoreField,
        v: i32,
        c: &mut PropertySetterCompletion,
    ) {
        self.shape.set_color_with_completion(f, v, c);
    }
    fn set_bool_with_completion(
        &mut self,
        f: CoreField,
        v: bool,
        c: &mut PropertySetterCompletion,
    ) {
        self.shape.set_bool_with_completion(f, v, c);
    }
    fn set_double_with_completion(
        &mut self,
        f: CoreField,
        v: f32,
        c: &mut PropertySetterCompletion,
    ) {
        self.shape.set_double_with_completion(f, v, c);
    }
    fn set_callback_with_completion(
        &mut self,
        f: CoreField,
        v: CallbackData<'_>,
        c: &mut PropertySetterCompletion,
    ) {
        self.shape.set_callback_with_completion(f, v, c);
    }
    fn set_int_with_completion(&mut self, f: CoreField, v: i32, c: &mut PropertySetterCompletion) {
        self.shape.set_int_with_completion(f, v, c);
    }
    fn get_uint(&mut self, f: CoreField) -> u32 {
        self.shape.get_uint(f)
    }
    fn get_string(&mut self, f: CoreField) -> String {
        self.shape.get_string(f)
    }
    fn get_color(&mut self, f: CoreField) -> i32 {
        self.shape.get_color(f)
    }
    fn get_bool(&mut self, f: CoreField) -> bool {
        self.shape.get_bool(f)
    }
    fn get_double(&mut self, f: CoreField) -> f32 {
        self.shape.get_double(f)
    }
    fn get_int(&mut self, f: CoreField) -> i32 {
        self.shape.get_int(f)
    }
}

#[test]
fn animation_added_during_object_dirty_is_initialized_before_object_clean() {
    let arena = CoreArena::default();
    let root = arena.insert(Artboard::default());
    let mut keyed = KeyedObject::default();
    keyed.base.set_object_id_value(999);
    let mut animation = LinearAnimation::default();
    animation.add_keyed_object(arena.insert(keyed));
    let animation = arena.insert(animation);
    let appender = arena.insert(AppendAnimation {
        shape: Shape::default(),
        root: root.clone(),
        animation: animation.clone(),
    });
    root.with_downcast_mut::<Artboard, _>(|root_owner| {
        root_owner.add_object(Some(root.clone()));
        root_owner.add_object(Some(appender));
    })
    .unwrap();
    // The probe deliberately stops the clean pass. This observes the real
    // preceding animation-dirty pass without rendering or layout updates.
    assert_eq!(
        Artboard::initialize_handle(&root),
        StatusCode::InvalidObject
    );
    assert_eq!(
        animation.with_downcast::<LinearAnimation, _>(LinearAnimation::num_keyed_objects),
        Some(0),
        "the newly added animation must prune its unresolved keyed object in onAddedDirty"
    );
    assert_eq!(
        root.with_downcast::<Artboard, _>(Artboard::state_machine_count),
        Some(0),
        "a real animation prevents insertion of the default state machine"
    );
}
