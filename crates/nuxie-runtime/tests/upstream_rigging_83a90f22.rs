//! Focused owner-branch regressions for 83a90f22 (no upstream test was added).
use nuxie_runtime::source::{
    artboard::Artboard,
    bones::{bone::Bone, skin::Skin, tendon::Tendon},
    core::{CoreArena, CoreHandle},
    core_context::CoreContext,
    generated::{
        component_base::{ComponentBase, ComponentBaseCallbacks},
        core_registry::CoreRegistry,
        shapes::vertex_base::VertexBaseCallbacks,
    },
    math::{mat2d::Mat2D, vec2d::Vec2D},
    shapes::{
        contour_mesh_vertex::ContourMeshVertex,
        vertex::{Vertex, VertexBehavior},
    },
    status_code::StatusCode,
};

struct Context {
    arena: CoreArena,
    objects: Vec<CoreHandle>,
}
impl CoreContext for Context {
    fn core_arena(&self) -> &CoreArena {
        &self.arena
    }
    fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        self.objects.get(id as usize).cloned()
    }
}

#[test]
fn tendon_rejects_missing_and_non_bone_targets_but_retains_a_valid_bone() {
    for (bone_id, expected) in [
        (99, StatusCode::InvalidObject),
        (1, StatusCode::InvalidObject),
        (2, StatusCode::Ok),
    ] {
        let arena = CoreArena::default();
        let root = arena.insert(Artboard::default());
        let skin = arena.insert(Skin::default());
        let bone = arena.insert(Bone::default());
        let tendon = arena.insert(Tendon::default());
        let mut context = Context {
            arena,
            objects: vec![root, skin.clone(), bone.clone()],
        };
        CoreRegistry::set_uint_handle(&tendon, ComponentBase::PARENT_ID_PROPERTY_KEY as i32, 1);
        let result = tendon
            .with_downcast_mut::<Tendon, _>(|tendon| {
                tendon.set_bone_id(bone_id);
                // Explicitly establish that superclass setup succeeds with this
                // valid Skin parent before exercising the bone-resolution branch.
                assert_eq!(
                    tendon.base.base.on_added_dirty(&mut context),
                    StatusCode::Ok
                );
                tendon.on_added_dirty(&mut context)
            })
            .expect("Tendon");
        assert_eq!(result, expected);
        let resolved = tendon
            .with_downcast::<Tendon, _>(Tendon::bone)
            .expect("Tendon");
        assert_eq!(resolved, (expected == StatusCode::Ok).then_some(bone));
    }
}

struct Callbacks;
impl ComponentBaseCallbacks for Callbacks {
    fn notify_property_changed(&mut self, _: u16) {}
}
impl VertexBaseCallbacks for Callbacks {
    fn notify_property_changed(&mut self, _: u16) {}
}

fn check_weightless(vertex: &mut impl VertexBehavior) {
    vertex.vertex_mut().base.set_x(7.0, &mut Callbacks);
    vertex.vertex_mut().base.set_y(-3.0, &mut Callbacks);
    assert!(!vertex.has_weight());
    let bind = Vec2D::new(7.0, -3.0);
    assert_eq!(vertex.render_translation(), bind);
    vertex.deform(&Mat2D::new(2.0, 0.0, 0.0, 3.0, 40.0, 50.0), &[]);
    assert_eq!(vertex.render_translation(), bind);
}

#[test]
fn weightless_vertex_preserves_bind_position_under_nonidentity_deformation() {
    check_weightless(&mut Vertex::default());
}

#[test]
fn weightless_contour_mesh_vertex_preserves_bind_position_under_nonidentity_deformation() {
    check_weightless(&mut ContourMeshVertex::default());
}
