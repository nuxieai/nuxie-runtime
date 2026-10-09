use super::*;
use crate::mechanical_port::source::{
    core::{Core, CoreArena, CoreHandle, CoreObject},
    generated::core_registry::CoreCapabilities,
    shapes::{
        cubic_detached_vertex::CubicDetachedVertex, cubic_vertex::CubicVertex,
        straight_vertex::StraightVertex, vertex::Vertex,
    },
};
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};

type Observations = Rc<RefCell<Vec<(u32, bool, f32)>>>;

struct DeformProbe {
    base: CubicDetachedVertex,
    id: u32,
    cubic: bool,
    observations: Observations,
    change_next: Option<CoreHandle>,
    skin: CoreHandle,
}
impl DeformProbe {
    fn observe(&mut self, cubic_hook: bool) {
        // Skin is still shared for the full source call; another shared read
        // must remain supported while a virtual vertex operation runs.
        assert_eq!(
            self.skin
                .with_downcast::<Skin, _>(|skin| *skin.bind_transform()),
            Some(Mat2D::identity())
        );
        self.observations
            .borrow_mut()
            .push((self.id, cubic_hook, self.base.x()));
        if let Some(next) = self.change_next.as_ref() {
            next.with_mut(|object| {
                object
                    .as_vertex_behavior_mut()
                    .unwrap()
                    .vertex_mut()
                    .base
                    .set_x_value(7.0);
            })
            .unwrap();
        }
    }
}
impl VertexBehavior for DeformProbe {
    fn vertex(&self) -> &Vertex {
        self.base.vertex()
    }
    fn vertex_mut(&mut self) -> &mut Vertex {
        self.base.vertex_mut()
    }
    fn mark_geometry_dirty(&mut self) {}
    fn deform(&mut self, _: &Mat2D, _: &[f32]) {
        self.observe(false);
    }
}
impl CubicVertexBehavior for DeformProbe {
    fn cubic_vertex(&self) -> &CubicVertex {
        self.base.cubic_vertex()
    }
    fn cubic_vertex_mut(&mut self) -> &mut CubicVertex {
        self.base.cubic_vertex_mut()
    }
    fn compute_in(&mut self) {
        panic!("the custom cubic deform hook must be retained");
    }
    fn compute_out(&mut self) {
        panic!("the custom cubic deform hook must be retained");
    }
    fn deform(&mut self, _: &Mat2D, _: &[f32]) {
        self.observe(true);
    }
}
impl CoreCapabilities for DeformProbe {
    fn as_vertex_behavior_mut(&mut self) -> Option<&mut dyn VertexBehavior> {
        Some(self)
    }
    fn as_cubic_vertex_behavior_mut(&mut self) -> Option<&mut dyn CubicVertexBehavior> {
        if self.cubic { Some(self) } else { None }
    }
}
impl CoreObject for DeformProbe {
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
        key == 65533
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        |key| key == 65533
    }
    fn deserialize(
        &mut self,
        key: u16,
        reader: &mut crate::mechanical_port::source::core::binary_reader::BinaryReader<'_>,
    ) -> bool {
        self.base.deserialize(key, reader)
    }
}

fn initialized_skin(arena: &CoreArena) -> CoreHandle {
    let skin = arena.insert(Skin::default());
    skin.with_downcast_mut::<Skin, _>(|owner| owner.build_dependencies(skin.clone()))
        .unwrap();
    skin
}

#[test]
fn borrowed_deform_preserves_authored_order_helpers_and_virtual_hooks() {
    let arena = CoreArena::default();
    let skin = initialized_skin(&arena);
    let observations = Observations::default();
    let second = arena.insert(DeformProbe {
        base: Default::default(),
        id: 2,
        cubic: true,
        observations: observations.clone(),
        change_next: None,
        skin: skin.clone(),
    });
    let first = arena.insert(DeformProbe {
        base: Default::default(),
        id: 1,
        cubic: false,
        observations: observations.clone(),
        change_next: Some(second.clone()),
        skin: skin.clone(),
    });
    let straight = Rc::new(RefCell::new(StraightVertex::default()));
    let cubic = Rc::new(RefCell::new(CubicDetachedVertex::default()));
    let mut path = PointsPath::default();
    path.set_skin(skin);
    path.base.add_vertex(first.clone());
    path.base.add_runtime_straight_vertex(straight.clone());
    path.base.add_vertex(second);
    path.base.add_runtime_cubic_vertex(cubic.clone());
    path.base.add_vertex(first);
    // Both helper resources remain borrowed exclusively. The source PointsPath
    // deform adapter must filter them without trying to inspect their contents.
    let _straight_loan = straight.borrow_mut();
    let _cubic_loan = cubic.borrow_mut();
    path.update_before_path_super(ComponentDirt::PATH);
    assert_eq!(
        &*observations.borrow(),
        &[(1, false, 0.0), (2, true, 7.0), (1, false, 0.0)]
    );
    observations.borrow_mut().clear();
    path.update_before_path_super(ComponentDirt::WORLD_TRANSFORM);
    assert!(observations.borrow().is_empty());
}

#[test]
fn borrowed_deform_empty_and_helper_only_inputs_still_require_initialized_skin() {
    let arena = CoreArena::default();
    let skin = arena.insert(Skin::default());
    for helpers in [false, true] {
        let mut path = PointsPath::default();
        path.set_skin(skin.clone());
        if helpers {
            path.base
                .add_runtime_straight_vertex(Rc::new(RefCell::new(StraightVertex::default())));
        }
        assert!(
            catch_unwind(AssertUnwindSafe(
                || path.update_before_path_super(ComponentDirt::PATH)
            ))
            .is_err()
        );
    }
    skin.with_downcast_mut::<Skin, _>(|owner| owner.build_dependencies(skin.clone()))
        .unwrap();
    for helpers in [false, true] {
        let mut path = PointsPath::default();
        path.set_skin(skin.clone());
        if helpers {
            path.base
                .add_runtime_cubic_vertex(Rc::new(RefCell::new(CubicDetachedVertex::default())));
        }
        path.update_before_path_super(ComponentDirt::PATH);
    }
    // The existing public borrowed-slice API keeps the same eager check.
    assert!(catch_unwind(AssertUnwindSafe(|| Skin::default().deform(&[]))).is_err());
}

#[test]
fn borrowed_deform_keeps_stale_vertex_generation_failure() {
    let arena = CoreArena::default();
    let skin = initialized_skin(&arena);
    let observations = Observations::default();
    let stale = arena.insert(DeformProbe {
        base: Default::default(),
        id: 1,
        cubic: false,
        observations: observations.clone(),
        change_next: None,
        skin: skin.clone(),
    });
    let mut path = PointsPath::default();
    path.set_skin(skin.clone());
    path.base.add_vertex(stale.clone());
    drop(arena.remove(&stale));
    let replacement = arena.insert(DeformProbe {
        base: Default::default(),
        id: 2,
        cubic: false,
        observations: observations.clone(),
        change_next: None,
        skin,
    });
    assert_eq!(stale.identity_key().1, replacement.identity_key().1);
    assert_ne!(stale.identity_key().2, replacement.identity_key().2);
    assert!(
        catch_unwind(AssertUnwindSafe(
            || path.update_before_path_super(ComponentDirt::PATH)
        ))
        .is_err()
    );
    assert!(observations.borrow().is_empty());
}

impl crate::mechanical_port::source::generated::core_registry::CoreRegistryObject for DeformProbe {
    fn as_registry_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        crate::mechanical_port::source::core::CoreObject::is_type_of(self, key)
    }
    fn set_uint_with_completion(
        &mut self,
        field: crate::mechanical_port::source::generated::core_registry::CoreField,
        value: u32,
        completion: &mut crate::mechanical_port::source::core::PropertySetterCompletion,
    ) {
        self.base.set_uint_with_completion(field, value, completion);
    }
    fn set_string_with_completion(
        &mut self,
        field: crate::mechanical_port::source::generated::core_registry::CoreField,
        value: String,
        completion: &mut crate::mechanical_port::source::core::PropertySetterCompletion,
    ) {
        self.base
            .set_string_with_completion(field, value, completion);
    }
    fn set_color_with_completion(
        &mut self,
        field: crate::mechanical_port::source::generated::core_registry::CoreField,
        value: i32,
        completion: &mut crate::mechanical_port::source::core::PropertySetterCompletion,
    ) {
        self.base
            .set_color_with_completion(field, value, completion);
    }
    fn set_bool_with_completion(
        &mut self,
        field: crate::mechanical_port::source::generated::core_registry::CoreField,
        value: bool,
        completion: &mut crate::mechanical_port::source::core::PropertySetterCompletion,
    ) {
        self.base.set_bool_with_completion(field, value, completion);
    }
    fn set_double_with_completion(
        &mut self,
        field: crate::mechanical_port::source::generated::core_registry::CoreField,
        value: f32,
        completion: &mut crate::mechanical_port::source::core::PropertySetterCompletion,
    ) {
        self.base
            .set_double_with_completion(field, value, completion);
    }
    fn set_callback_with_completion(
        &mut self,
        field: crate::mechanical_port::source::generated::core_registry::CoreField,
        value: crate::mechanical_port::source::core::field_types::core_callback_type::CallbackData<
            '_,
        >,
        completion: &mut crate::mechanical_port::source::core::PropertySetterCompletion,
    ) {
        self.base
            .set_callback_with_completion(field, value, completion);
    }
    fn set_int_with_completion(
        &mut self,
        field: crate::mechanical_port::source::generated::core_registry::CoreField,
        value: i32,
        completion: &mut crate::mechanical_port::source::core::PropertySetterCompletion,
    ) {
        self.base.set_int_with_completion(field, value, completion);
    }
    fn get_uint(
        &mut self,
        field: crate::mechanical_port::source::generated::core_registry::CoreField,
    ) -> u32 {
        self.base.get_uint(field)
    }
    fn get_string(
        &mut self,
        field: crate::mechanical_port::source::generated::core_registry::CoreField,
    ) -> String {
        self.base.get_string(field)
    }
    fn get_color(
        &mut self,
        field: crate::mechanical_port::source::generated::core_registry::CoreField,
    ) -> i32 {
        self.base.get_color(field)
    }
    fn get_bool(
        &mut self,
        field: crate::mechanical_port::source::generated::core_registry::CoreField,
    ) -> bool {
        self.base.get_bool(field)
    }
    fn get_double(
        &mut self,
        field: crate::mechanical_port::source::generated::core_registry::CoreField,
    ) -> f32 {
        self.base.get_double(field)
    }
    fn get_int(
        &mut self,
        field: crate::mechanical_port::source::generated::core_registry::CoreField,
    ) -> i32 {
        self.base.get_int(field)
    }
}
