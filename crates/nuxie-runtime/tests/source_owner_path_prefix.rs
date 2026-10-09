//! Path prefix composition retains the source callback and identity boundaries.
use nuxie_runtime::source::{
    component::{Component, ComponentDirt, ComponentOccurrenceHandle},
    core::{
        Core, CoreArena, CoreHandle, CoreObject, PropertySetterCompletion,
        binary_reader::BinaryReader, field_types::core_callback_type::CallbackData,
    },
    generated::core_registry::{CoreCapabilities, CoreField, CoreRegistry, CoreRegistryObject},
    node::Node,
    shapes::{
        ellipse::Ellipse, list_path::ListPath, points_path::PointsPath, polygon::Polygon,
        rectangle::Rectangle, star::Star, triangle::Triangle,
    },
    transform_component::TransformComponent,
};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};
macro_rules! fixture_core { ($ty:ty, {$($extra:tt)*}) => { impl CoreObject for $ty {
 fn core(&self)->&Core { self.payload.core() }
 fn core_mut(&mut self)->&mut Core { self.payload.core_mut() }
 fn core_type(&self)->u16 { 65534 }
 fn is_type_of(&self,key:u16)->bool { key==65534 }
 fn type_predicate(&self)->fn(u16)->bool { |key| key==65534 }
 fn deserialize(&mut self,key:u16,reader:&mut BinaryReader<'_>)->bool { self.payload.deserialize(key,reader) }
 $($extra)*
} impl CoreRegistryObject for $ty {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        key == 65534
    }
    fn set_uint_with_completion(&mut self, f: CoreField, v: u32, c: &mut PropertySetterCompletion) {
        self.payload.set_uint_with_completion(f, v, c);
    }
    fn set_string_with_completion(
        &mut self,
        f: CoreField,
        v: String,
        c: &mut PropertySetterCompletion,
    ) {
        self.payload.set_string_with_completion(f, v, c);
    }
    fn set_color_with_completion(
        &mut self,
        f: CoreField,
        v: i32,
        c: &mut PropertySetterCompletion,
    ) {
        self.payload.set_color_with_completion(f, v, c);
    }
    fn set_bool_with_completion(
        &mut self,
        f: CoreField,
        v: bool,
        c: &mut PropertySetterCompletion,
    ) {
        self.payload.set_bool_with_completion(f, v, c);
    }
    fn set_double_with_completion(
        &mut self,
        f: CoreField,
        v: f32,
        c: &mut PropertySetterCompletion,
    ) {
        self.payload.set_double_with_completion(f, v, c);
    }
    fn set_callback_with_completion(
        &mut self,
        f: CoreField,
        v: CallbackData<'_>,
        c: &mut PropertySetterCompletion,
    ) {
        self.payload.set_callback_with_completion(f, v, c);
    }
    fn set_int_with_completion(&mut self, f: CoreField, v: i32, c: &mut PropertySetterCompletion) {
        self.payload.set_int_with_completion(f, v, c);
    }
    fn get_uint(&mut self, f: CoreField) -> u32 {
        self.payload.get_uint(f)
    }
    fn get_string(&mut self, f: CoreField) -> String {
        self.payload.get_string(f)
    }
    fn get_color(&mut self, f: CoreField) -> i32 {
        self.payload.get_color(f)
    }
    fn get_bool(&mut self, f: CoreField) -> bool {
        self.payload.get_bool(f)
    }
    fn get_double(&mut self, f: CoreField) -> f32 {
        self.payload.get_double(f)
    }
    fn get_int(&mut self, f: CoreField) -> i32 {
        self.payload.get_int(f)
    }
}
 }; }
struct ProbeConstraint {
    payload: Node,
    callback: Box<dyn FnMut(CoreHandle)>,
}
impl CoreCapabilities for ProbeConstraint {
    fn constraint_apply(&mut self, owner: CoreHandle) -> bool {
        (self.callback)(owner);
        true
    }
}
fixture_core!(ProbeConstraint, {});
struct ProjectedPath {
    payload: PointsPath,
    projections: Rc<RefCell<Vec<&'static str>>>,
}
impl CoreCapabilities for ProjectedPath {
    fn as_component(&self) -> Option<&Component> {
        self.payload.as_component()
    }
    fn as_component_mut(&mut self) -> Option<&mut Component> {
        self.payload.as_component_mut()
    }
    fn as_transform_component(&self) -> Option<&TransformComponent> {
        self.payload.as_transform_component()
    }
    fn as_transform_component_mut(&mut self) -> Option<&mut TransformComponent> {
        self.payload.as_transform_component_mut()
    }
    fn component_update_handler(&self) -> Option<fn(&CoreHandle, ComponentDirt) -> bool> {
        self.payload.component_update_handler()
    }
}
fixture_core!(ProjectedPath, {
    fn as_any(&self) -> &dyn Any {
        self.projections.borrow_mut().push("shared");
        &self.payload
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self.projections.borrow_mut().push("mut");
        &mut self.payload
    }
});
fn occurrence(owner: &CoreHandle) -> ComponentOccurrenceHandle {
    ComponentOccurrenceHandle::Authored(owner.clone())
}
fn add_constraint(
    arena: &CoreArena,
    owner: &CoreHandle,
    callback: impl FnMut(CoreHandle) + 'static,
) {
    let constraint = arena.insert(ProbeConstraint {
        payload: Node::default(),
        callback: Box::new(callback),
    });
    owner
        .with_mut(|object| {
            object
                .as_transform_component_mut()
                .unwrap()
                .add_constraint(constraint)
        })
        .unwrap();
}
#[test]
fn native_path_owners_release_composition_before_constraints_and_reread_tail() {
    let arena = CoreArena::default();
    let owners = [
        arena.insert(PointsPath::default()),
        arena.insert(Ellipse::default()),
        arena.insert(Rectangle::default()),
        arena.insert(Polygon::default()),
        arena.insert(Star::default()),
        arena.insert(Triangle::default()),
        arena.insert(ListPath::default()),
    ];
    for owner in owners {
        assert!(CoreRegistry::set_double_handle(
            &owner,
            nuxie_runtime::source::generated::node_base::NodeBase::X_PROPERTY_KEY.into(),
            7.0
        ));
        let calls = Rc::new(Cell::new(0));
        let callback_calls = calls.clone();
        add_constraint(&arena, &owner, move |owner| {
            owner
                .with_mut(|object| {
                    let transform = object.as_transform_component_mut().unwrap();
                    assert_eq!(
                        transform.world_transform()[4],
                        7.0,
                        "local composition precedes constraint"
                    );
                    transform.mutable_world_transform()[4] = 11.0;
                })
                .unwrap();
            assert!(CoreRegistry::set_double_handle(&owner, nuxie_runtime::source::generated::world_transform_component_base::WorldTransformComponentBase::OPACITY_PROPERTY_KEY.into(), 0.25));
            callback_calls.set(callback_calls.get() + 1);
        });
        occurrence(&owner).update(
            ComponentDirt::TRANSFORM
                | ComponentDirt::WORLD_TRANSFORM
                | ComponentDirt::RENDER_OPACITY
                | ComponentDirt::N_SLICER,
        );
        assert_eq!(calls.get(), 1);
        owner
            .with(|object| {
                assert_eq!(
                    object.as_transform_component().unwrap().world_transform()[4],
                    11.0
                );
                assert_eq!(
                    object.as_transform_component().unwrap().render_opacity(),
                    0.25
                );
                assert_eq!(object.as_path().unwrap().geometry_version(), 1);
            })
            .unwrap();
    }
}
#[test]
fn projected_path_keeps_each_original_projection_and_released_super_phase() {
    for (dirt, expected) in [
        (
            ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM,
            vec!["mut", "mut", "mut", "shared", "mut"],
        ),
        (
            ComponentDirt::WORLD_TRANSFORM,
            vec!["mut", "mut", "shared", "mut"],
        ),
        (ComponentDirt::NONE, vec!["mut", "mut"]),
    ] {
        let arena = CoreArena::default();
        let projections = Rc::new(RefCell::new(Vec::new()));
        let owner = arena.insert(ProjectedPath {
            payload: PointsPath::default(),
            projections: projections.clone(),
        });
        occurrence(&owner).update(dirt);
        assert_eq!(*projections.borrow(), expected);
    }
}
#[test]
fn path_constraint_retirement_does_not_apply_tail_to_reused_slot() {
    let arena = CoreArena::default();
    let owner = arena.insert(PointsPath::default());
    let callback_arena = arena.weak_handle();
    let replacement = Rc::new(RefCell::new(None));
    let callback_replacement = replacement.clone();
    add_constraint(&arena, &owner, move |owner| {
        assert!(callback_arena.remove(&owner).is_some());
        let next = callback_arena.insert(PointsPath::default());
        assert_ne!(owner, next);
        assert!(!owner.is_alive());
        *callback_replacement.borrow_mut() = Some(next);
    });
    occurrence(&owner).update(
        ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER | ComponentDirt::RENDER_OPACITY,
    );
    let next = replacement.borrow().clone().unwrap();
    assert_eq!(
        next.with(|object| object.as_path().unwrap().geometry_version()),
        Some(0)
    );
    occurrence(&owner).update(ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER);
    assert_eq!(
        next.with(|object| object.as_path().unwrap().geometry_version()),
        Some(0)
    );
}
#[test]
fn panic_in_path_constraint_releases_receiver_and_allows_retry() {
    let arena = CoreArena::default();
    let owner = arena.insert(PointsPath::default());
    let first = Rc::new(Cell::new(true));
    let callback_first = first.clone();
    add_constraint(&arena, &owner, move |owner| {
        owner
            .with_mut(|object| {
                object
                    .as_transform_component_mut()
                    .unwrap()
                    .mutable_world_transform()[4] = 19.0
            })
            .unwrap();
        if callback_first.replace(false) {
            panic!("intentional constraint panic");
        }
    });
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        occurrence(&owner).update(ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER)
    }));
    assert!(result.is_err());
    assert_eq!(
        owner.with(|object| object.as_path().unwrap().geometry_version()),
        Some(0)
    );
    occurrence(&owner).update(ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER);
    assert_eq!(
        owner.with(|object| object.as_path().unwrap().geometry_version()),
        Some(1)
    );
}
