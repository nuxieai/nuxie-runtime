//! Complete skinned_winding_test.cpp at upstream 8ff564a3.
use nuxie_render_api::{FillRule, PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    artboard::Artboard,
    bones::{
        bone::Bone, root_bone::RootBone, skin::Skin, skinnable::SkinnableBehavior, tendon::Tendon,
        weight::Weight,
    },
    core::{CoreArena, CoreHandle, CoreType},
    generated::{
        bones::{root_bone_base::RootBoneBase, tendon_base::TendonBase, weight_base::WeightBase},
        component_base::ComponentBase,
        core_registry::CoreRegistry,
        shapes::{
            paint::fill_base::FillBase, path_base::PathBase,
            points_common_path_base::PointsCommonPathBase, vertex_base::VertexBase,
        },
        transform_component_base::TransformComponentBase,
    },
    shapes::{
        paint::{fill::Fill, shape_paint::ShapePaintPathKind, solid_color::SolidColor},
        points_path::PointsPath,
        shape::Shape,
        shape_path_flags::ShapePathFlags,
        straight_vertex::StraightVertex,
    },
    status_code::StatusCode,
};
use nuxie_runtime::{File, RuntimeFactoryHandle};

fn uint(owner: &CoreHandle, key: u16, value: u32) {
    assert!(CoreRegistry::set_uint_handle(owner, key.into(), value));
}
fn number(owner: &CoreHandle, key: u16, value: f32) {
    assert!(CoreRegistry::set_double_handle(owner, key.into(), value));
}
fn get_number(owner: &CoreHandle, key: u16) -> f32 {
    CoreRegistry::get_double_handle(owner, key.into()).unwrap()
}
fn scale(bone: &CoreHandle, x: f32, y: f32) {
    number(bone, TransformComponentBase::SCALE_X_PROPERTY_KEY, x);
    number(bone, TransformComponentBase::SCALE_Y_PROPERTY_KEY, y);
}
fn advance(root: &CoreHandle) {
    Artboard::advance_handle(
        root,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}
fn composed_area(shape: &CoreHandle) -> f32 {
    shape
        .with_downcast::<Shape, _>(|s| {
            s.with_path_mut(ShapePaintPathKind::LocalClockwise, |p| {
                p.raw_path().compute_coarse_area()
            })
        })
        .unwrap()
}
fn winding(skin: &CoreHandle) -> i32 {
    skin.with_downcast::<Skin, _>(Skin::winding_sign).unwrap()
}

struct QuadRig {
    _arena: CoreArena,
    artboard: CoreHandle,
    top: CoreHandle,
    bottom: CoreHandle,
    shape: CoreHandle,
    path: CoreHandle,
    skin: CoreHandle,
    vertices: Vec<CoreHandle>,
}
impl QuadRig {
    fn new(clockwise: bool) -> Self {
        Self::with_top_start_y(clockwise, 0.0)
    }
    fn with_top_start_y(clockwise: bool, top_start_y: f32) -> Self {
        let arena = CoreArena::default();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
        let artboard = arena.insert(Artboard::with_factory(factory));
        artboard
            .with_downcast_mut::<Artboard, _>(|a| a.set_core_arena(arena.clone()))
            .unwrap();
        artboard
            .with_downcast_mut::<Artboard, _>(|a| a.add_object(Some(artboard.clone())))
            .unwrap();
        let add = |component: &CoreHandle, parent: &CoreHandle| {
            let parent_id = artboard
                .with_downcast::<Artboard, _>(|a| a.id_of(parent))
                .unwrap();
            artboard
                .with_downcast_mut::<Artboard, _>(|a| a.add_object(Some(component.clone())))
                .unwrap();
            uint(component, ComponentBase::PARENT_ID_PROPERTY_KEY, parent_id);
        };
        let top = arena.insert(RootBone::default());
        let bottom = arena.insert(RootBone::default());
        let shape = arena.insert(Shape::default());
        let path = arena.insert(PointsPath::default());
        let skin = arena.insert(Skin::default());
        add(&top, &artboard);
        number(&bottom, RootBoneBase::Y_PROPERTY_KEY, 100.0);
        add(&bottom, &artboard);
        add(&shape, &artboard);
        let fill = arena.insert(Fill::default());
        uint(
            &fill,
            FillBase::FILL_RULE_PROPERTY_KEY,
            FillRule::Clockwise as u32,
        );
        add(&fill, &shape);
        add(&arena.insert(SolidColor::default()), &fill);
        assert!(CoreRegistry::set_bool_handle(
            &path,
            PointsCommonPathBase::IS_CLOSED_PROPERTY_KEY.into(),
            true
        ));
        if !clockwise {
            uint(
                &path,
                PathBase::PATH_FLAGS_PROPERTY_KEY,
                ShapePathFlags::IsCounterClockwise as u32,
            );
        }
        add(&path, &shape);
        let corners = [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
        let mut vertices = Vec::new();
        for i in 0..4 {
            let (x, y) = corners[if clockwise { i } else { 3 - i }];
            let vertex = arena.insert(StraightVertex::default());
            number(&vertex, VertexBase::X_PROPERTY_KEY, x);
            number(&vertex, VertexBase::Y_PROPERTY_KEY, y);
            add(&vertex, &path);
            let weight = arena.insert(Weight::default());
            uint(&weight, WeightBase::VALUES_PROPERTY_KEY, 255);
            uint(
                &weight,
                WeightBase::INDICES_PROPERTY_KEY,
                if y == 0.0 { 1 } else { 2 },
            );
            add(&weight, &vertex);
            vertices.push(vertex);
        }
        add(&skin, &path);
        for bone in [&top, &bottom] {
            let tendon = arena.insert(Tendon::default());
            uint(
                &tendon,
                TendonBase::BONE_ID_PROPERTY_KEY,
                artboard
                    .with_downcast::<Artboard, _>(|a| a.id_of(bone))
                    .unwrap(),
            );
            number(
                &tendon,
                TendonBase::TY_PROPERTY_KEY,
                get_number(bone, RootBoneBase::Y_PROPERTY_KEY),
            );
            add(&tendon, &skin);
        }
        number(&top, RootBoneBase::Y_PROPERTY_KEY, top_start_y);
        assert_eq!(Artboard::initialize_handle(&artboard), StatusCode::Ok);
        Self {
            _arena: arena,
            artboard,
            top,
            bottom,
            shape,
            path,
            skin,
            vertices,
        }
    }
    fn composed_area(&self) -> f32 {
        advance(&self.artboard);
        composed_area(&self.shape)
    }
    fn deformed_area(&self) -> f32 {
        self.path
            .with_downcast::<PointsPath, _>(|p| {
                let path = p.raw_path();
                path.compute_coarse_area_with_origin(path.bounds().center())
            })
            .unwrap()
    }
}

#[test]
fn unskinned_and_resting_skinned_quads_compose_clockwise() {
    for clockwise in [true, false] {
        let rig = QuadRig::new(clockwise);
        assert!(rig.composed_area() > 0.0);
        assert_eq!(winding(&rig.skin), 1);
        assert_eq!(rig.deformed_area() > 0.0, clockwise);
    }
}
#[test]
fn collapsed_first_frame_is_not_cached_as_winding() {
    let rig = QuadRig::new(false);
    scale(&rig.top, 0.0, 0.0);
    scale(&rig.bottom, 0.0, 0.0);
    advance(&rig.artboard);
    assert_eq!(winding(&rig.skin), 0);
    assert_eq!(rig.deformed_area(), 0.0);
    scale(&rig.top, 1.0, 1.0);
    scale(&rig.bottom, 1.0, 1.0);
    assert!(rig.composed_area() > 0.0);
    assert!(rig.deformed_area() < 0.0);
}
#[test]
fn collapsed_bone_does_not_vote_on_mirroring() {
    let rig = QuadRig::new(true);
    assert!(rig.composed_area() > 0.0);
    scale(&rig.top, -1.0, 1.0);
    scale(&rig.bottom, 0.0, 0.0);
    assert!(rig.composed_area() > 0.0);
    assert_eq!(winding(&rig.skin), -1);
    assert!(rig.deformed_area() < 0.0);
    scale(&rig.bottom, -1.0, 1.0);
    assert!(rig.composed_area() > 0.0);
    assert_eq!(winding(&rig.skin), -1);
}
#[test]
fn mixed_first_frame_leaves_nothing_wrong_cached() {
    let rig = QuadRig::new(false);
    scale(&rig.top, -1.5, 1.0);
    assert!(rig.composed_area() > 0.0);
    assert_eq!(winding(&rig.skin), 0);
    assert!(rig.deformed_area() > 0.0);
    scale(&rig.top, 1.0, 1.0);
    assert!(rig.composed_area() > 0.0);
    assert_eq!(winding(&rig.skin), 1);
    assert!(rig.deformed_area() < 0.0);
    scale(&rig.top, -1.0, 1.0);
    scale(&rig.bottom, -1.0, 1.0);
    assert!(rig.composed_area() > 0.0);
    assert_eq!(winding(&rig.skin), -1);
    assert!(rig.deformed_area() > 0.0);
}
#[test]
fn rig_far_from_origin_still_measures_winding() {
    let rig = QuadRig::new(false);
    for bone in [&rig.top, &rig.bottom] {
        number(bone, RootBoneBase::X_PROPERTY_KEY, 1e5);
        number(
            bone,
            RootBoneBase::Y_PROPERTY_KEY,
            get_number(bone, RootBoneBase::Y_PROPERTY_KEY) + 1e5,
        );
    }
    assert!(rig.composed_area() > 0.0);
    assert!((rig.deformed_area() + 10000.0).abs() <= 10000.0 * f32::EPSILON * 100.0);
    scale(&rig.top, -1.0, 1.0);
    scale(&rig.bottom, -1.0, 1.0);
    assert!(rig.composed_area() > 0.0);
    assert!((rig.deformed_area() - 10000.0).abs() <= 10000.0 * f32::EPSILON * 100.0);
}
#[test]
fn moving_vertex_measures_winding_again() {
    let rig = QuadRig::new(true);
    assert!(rig.composed_area() > 0.0);
    for vertex in &rig.vertices {
        number(
            vertex,
            VertexBase::X_PROPERTY_KEY,
            -get_number(vertex, VertexBase::X_PROPERTY_KEY),
        );
    }
    assert!(rig.composed_area() > 0.0);
    assert!(rig.deformed_area() < 0.0);
}
#[test]
fn fold_without_mirroring_keeps_measured_winding() {
    let rig = QuadRig::new(true);
    assert!(rig.composed_area() > 0.0);
    number(&rig.top, RootBoneBase::Y_PROPERTY_KEY, 200.0);
    let composed = rig.composed_area();
    assert_eq!(winding(&rig.skin), 1);
    assert!(rig.deformed_area() < 0.0);
    assert!(composed < 0.0);
}

#[test]
fn folded_first_frame_is_not_cached_as_winding() {
    for clockwise in [true, false] {
        let rig = QuadRig::with_top_start_y(clockwise, 200.0);
        let folded = rig.composed_area();
        assert_eq!(winding(&rig.skin), 1);
        assert_eq!(rig.deformed_area() < 0.0, clockwise);
        assert!(folded < 0.0);
        number(&rig.top, RootBoneBase::Y_PROPERTY_KEY, 0.0);
        assert!(rig.composed_area() > 0.0);
        assert_eq!(rig.deformed_area() > 0.0, clockwise);
    }
}

fn parent(node: &CoreHandle) -> Option<CoreHandle> {
    node.with(|o| o.as_component().and_then(|c| c.parent_handle()))
        .flatten()
}
fn is_ancestor(ancestor: &CoreHandle, of: &CoreHandle) -> bool {
    let mut current = parent(of);
    while let Some(node) = current {
        if &node == ancestor {
            return true;
        }
        current = parent(&node);
    }
    false
}
fn tendons(skin: &CoreHandle) -> Vec<CoreHandle> {
    skin.with_downcast::<Skin, _>(|s| s.tendons().to_vec())
        .unwrap()
}
fn tendon_bone(tendon: &CoreHandle) -> CoreHandle {
    tendon
        .with_downcast::<Tendon, _>(Tendon::bone)
        .flatten()
        .unwrap()
}
fn common_bone(skin: &CoreHandle) -> Option<CoreHandle> {
    let tendons = tendons(skin);
    let mut current = Some(tendon_bone(&tendons[0]));
    while let Some(node) = current {
        if node.is_type_of(Bone::TYPE_KEY)
            && tendons.iter().all(|t| {
                let b = tendon_bone(t);
                b == node || is_ancestor(&node, &b)
            })
        {
            return Some(node);
        }
        current = parent(&node);
    }
    None
}
fn first_fill(shape: &CoreHandle) -> Option<CoreHandle> {
    shape
        .with_downcast::<Shape, _>(|s| {
            s.paint_container
                .shape_paints()
                .iter()
                .find(|p| p.is_type_of(Fill::TYPE_KEY))
                .cloned()
        })
        .flatten()
}
fn raw_area(path: &CoreHandle) -> f32 {
    path.with_downcast::<PointsPath, _>(|p| p.raw_path().compute_coarse_area())
        .unwrap()
}

#[test]
fn real_rig_stays_clockwise_when_bones_mirror() {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        std::path::PathBuf::from(root).join("tests/unit_tests/assets/zombie_skins.riv"),
    )
    .unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let file = File::import(&bytes, factory, None, None, None).unwrap();
    let source = file.with_file(File::artboard).unwrap();
    let objects = source
        .with_downcast::<Artboard, _>(|a| a.objects().to_vec())
        .unwrap();
    let mut found = None;
    for core in objects.iter().flatten() {
        let Some((skin, shape)) =
            core.with_downcast::<PointsPath, _>(|p| (p.skin(), p.shape_handle()))
        else {
            continue;
        };
        let (Some(skin), Some(shape)) = (skin, shape) else {
            continue;
        };
        if shape
            .with_downcast::<Shape, _>(|s| s.paths().len())
            .unwrap()
            != 1
            || first_fill(&shape).is_none()
            || tendons(&skin).len() < 2
        {
            continue;
        }
        if let Some(bone) = common_bone(&skin) {
            found = Some((shape, bone));
            break;
        }
    }
    let (source_shape, source_bone) = found.expect("filled single path with common bone");
    uint(&source_shape, ComponentBase::PARENT_ID_PROPERTY_KEY, 0);
    let (shape_id, bone_id) = source
        .with_downcast::<Artboard, _>(|a| (a.id_of(&source_shape), a.id_of(&source_bone)))
        .unwrap();
    let artboard = Artboard::instance_from_handle(&source).unwrap();
    let (shape, bone) = artboard.with_artboard(|a| {
        (
            a.objects()[shape_id as usize].clone().unwrap(),
            a.objects()[bone_id as usize].clone().unwrap(),
        )
    });
    assert_eq!(parent(&shape), Some(artboard.core_handle()));
    assert!(!is_ancestor(&bone, &shape));
    let path = shape
        .with_downcast::<Shape, _>(|s| s.paths()[0].clone())
        .unwrap();
    let skin = path
        .with_downcast::<PointsPath, _>(|p| p.skin())
        .flatten()
        .unwrap();
    uint(
        &first_fill(&shape).unwrap(),
        FillBase::FILL_RULE_PROPERTY_KEY,
        FillRule::Clockwise as u32,
    );
    shape
        .with_downcast_mut::<Shape, _>(Shape::path_changed)
        .unwrap();
    artboard.advance_default(0.0);
    let rest_area = raw_area(&path);
    assert!(composed_area(&shape) > 0.0);
    assert_eq!(winding(&skin), 1);
    number(&bone, TransformComponentBase::SCALE_X_PROPERTY_KEY, -1.0);
    artboard.advance_default(0.0);
    assert_eq!(winding(&skin), -1);
    assert!(raw_area(&path) * rest_area < 0.0);
    assert!(composed_area(&shape) > 0.0);
    number(&bone, TransformComponentBase::SCALE_X_PROPERTY_KEY, 1.0);
    let tendons = tendons(&skin);
    let leaf = tendons
        .iter()
        .map(tendon_bone)
        .find(|candidate| {
            candidate != &bone
                && !tendons
                    .iter()
                    .any(|other| is_ancestor(candidate, &tendon_bone(other)))
        })
        .expect("leaf tendon bone");
    number(&leaf, TransformComponentBase::SCALE_X_PROPERTY_KEY, -1.0);
    artboard.advance_default(0.0);
    assert_eq!(winding(&skin), 0);
    let folded_area = raw_area(&path);
    assert!((folded_area - rest_area).abs() > 0.01 * rest_area.abs());
    assert!(composed_area(&shape) * folded_area >= 0.0);
}
