//! Complete runtime/skinned_vertex_weight_test.cpp at upstream 3f67b2f8.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::RuntimeFactoryHandle;
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    artboard::Artboard,
    bones::{root_bone::RootBone, skin::Skin, tendon::Tendon, weight::Weight},
    core::{CoreArena, CoreHandle},
    generated::{
        bones::{root_bone_base::RootBoneBase, tendon_base::TendonBase, weight_base::WeightBase},
        component_base::ComponentBase,
        core_registry::CoreRegistry,
        shapes::{points_common_path_base::PointsCommonPathBase, vertex_base::VertexBase},
    },
    math::vec2d::Vec2D,
    shapes::{
        cubic_detached_vertex::CubicDetachedVertex,
        paint::{fill::Fill, solid_color::SolidColor},
        points_path::PointsPath,
        shape::Shape,
        straight_vertex::StraightVertex,
    },
    status_code::StatusCode,
};

fn uint(owner: &CoreHandle, key: u16, value: u32) {
    assert!(CoreRegistry::set_uint_handle(owner, key.into(), value));
}
fn number(owner: &CoreHandle, key: u16, value: f32) {
    assert!(CoreRegistry::set_double_handle(owner, key.into(), value));
}

// Two straight vertices weighted to one bone, and a cubic vertex supplied
// with either no weight or a base Weight instead of CubicWeight.
struct CubicRig {
    _arena: CoreArena,
    artboard: CoreHandle,
    bone: CoreHandle,
    weighted: [CoreHandle; 2],
    cubic: CoreHandle,
}

impl CubicRig {
    fn new(cubic_weight: Option<Weight>) -> Self {
        let arena = CoreArena::default();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
        let artboard = arena.insert(Artboard::with_factory(factory));
        artboard
            .with_downcast_mut::<Artboard, _>(|a| {
                a.set_core_arena(arena.clone());
                a.add_object(Some(artboard.clone()));
            })
            .unwrap();
        let add = |component: &CoreHandle, parent: &CoreHandle| {
            artboard
                .with_downcast_mut::<Artboard, _>(|a| {
                    a.add_object(Some(component.clone()));
                })
                .unwrap();
            let parent_id = artboard
                .with_downcast::<Artboard, _>(|a| a.id_of(parent))
                .unwrap();
            uint(component, ComponentBase::PARENT_ID_PROPERTY_KEY, parent_id);
        };
        let bone = arena.insert(RootBone::default());
        add(&bone, &artboard);
        let shape = arena.insert(Shape::default());
        add(&shape, &artboard);
        let fill = arena.insert(Fill::default());
        add(&fill, &shape);
        add(&arena.insert(SolidColor::default()), &fill);
        let path = arena.insert(PointsPath::default());
        assert!(CoreRegistry::set_bool_handle(
            &path,
            PointsCommonPathBase::IS_CLOSED_PROPERTY_KEY.into(),
            true,
        ));
        add(&path, &shape);
        let weighted = std::array::from_fn(|i| {
            let vertex = arena.insert(StraightVertex::default());
            number(&vertex, VertexBase::X_PROPERTY_KEY, i as f32 * 100.0);
            add(&vertex, &path);
            let weight = arena.insert(Weight::default());
            uint(&weight, WeightBase::VALUES_PROPERTY_KEY, 255);
            uint(&weight, WeightBase::INDICES_PROPERTY_KEY, 1);
            add(&weight, &vertex);
            vertex
        });
        let cubic = arena.insert(CubicDetachedVertex::default());
        number(&cubic, VertexBase::X_PROPERTY_KEY, 50.0);
        number(&cubic, VertexBase::Y_PROPERTY_KEY, 100.0);
        add(&cubic, &path);
        if let Some(weight) = cubic_weight {
            add(&arena.insert(weight), &cubic);
        }
        let skin = arena.insert(Skin::default());
        add(&skin, &path);
        let tendon = arena.insert(Tendon::default());
        uint(
            &tendon,
            TendonBase::BONE_ID_PROPERTY_KEY,
            artboard
                .with_downcast::<Artboard, _>(|a| a.id_of(&bone))
                .unwrap(),
        );
        add(&tendon, &skin);
        assert_eq!(Artboard::initialize_handle(&artboard), StatusCode::Ok);
        Self {
            _arena: arena,
            artboard,
            bone,
            weighted,
            cubic,
        }
    }

    fn check_deform(&self) {
        let advance = || {
            Artboard::advance_handle(
                &self.artboard,
                0.0,
                AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
            );
        };
        advance();
        number(&self.bone, RootBoneBase::X_PROPERTY_KEY, 10.0);
        advance();
        let translation = |vertex: &CoreHandle| {
            vertex
                .with(|v| v.as_vertex_behavior().unwrap().render_translation())
                .unwrap()
        };
        assert_eq!(translation(&self.weighted[0]), Vec2D::new(10.0, 0.0));
        assert_eq!(translation(&self.weighted[1]), Vec2D::new(110.0, 0.0));
        assert_eq!(translation(&self.cubic), Vec2D::new(50.0, 100.0));
    }

    fn cubic_has_weight(&self) -> bool {
        self.cubic
            .with(|v| v.as_vertex_behavior().unwrap().has_weight())
            .unwrap()
    }
}

#[test]
fn a_skinned_cubic_vertex_without_a_weight_does_not_crash_the_deform() {
    let rig = CubicRig::new(None);
    assert!(!rig.cubic_has_weight());
    rig.check_deform();
}

#[test]
fn a_base_weight_under_a_skinned_cubic_vertex_is_not_used() {
    let mut weight = Weight::default();
    weight.set_values(255);
    weight.set_indices(1);
    let rig = CubicRig::new(Some(weight));
    assert!(!rig.cubic_has_weight());
    rig.check_deform();
}
