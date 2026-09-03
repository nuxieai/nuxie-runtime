//! Direct ports of both cases in pinned `tests/unit_tests/runtime/ik_test.cpp`.

use std::path::PathBuf;

use nuxie::{
    CoreHandle, File, Mat2D, PersistentFactory, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
};
use nuxie_render_api::SerializingFactory;
use nuxie_runtime::source::{
    animation::linear_animation::LinearAnimation, bones::bone::Bone, node::Node,
    shapes::shape::Shape,
};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

#[derive(Clone)]
struct IkOwners {
    bone_a: CoreHandle,
    bone_b: CoreHandle,
    target: CoreHandle,
    animation: CoreHandle,
}

fn setup(artboard: &RuntimeArtboardInstanceHandle) -> IkOwners {
    let circle_a = artboard
        .with_artboard(|artboard| artboard.base.find_handle::<Shape>("circle a"))
        .expect("circle a");
    let circle_b = artboard
        .with_artboard(|artboard| artboard.base.find_handle::<Shape>("circle b"))
        .expect("circle b");
    let bone_a = artboard
        .with_artboard(|artboard| artboard.base.find_handle::<Bone>("a"))
        .expect("bone a");
    let bone_b = artboard
        .with_artboard(|artboard| artboard.base.find_handle::<Bone>("b"))
        .expect("bone b");
    let target = artboard
        .with_artboard(|artboard| artboard.base.find_handle::<Node>("target"))
        .expect("target");
    let animation = artboard
        .with_artboard(|artboard| artboard.base.animation_named("Animation 1"))
        .expect("Animation 1");

    bone_b
        .with(|bone| {
            let dependents = bone.as_component().expect("Bone Component").dependents();
            assert!(
                dependents
                    .iter()
                    .any(|dependent| dependent.authored() == Some(&circle_a))
            );
            assert!(
                dependents
                    .iter()
                    .any(|dependent| dependent.authored() == Some(&circle_b))
            );
        })
        .expect("live bone b");

    IkOwners {
        bone_a,
        bone_b,
        target,
        animation,
    }
}

fn about_equal(actual: Mat2D, expected: [f32; 6]) -> bool {
    actual
        .values()
        .iter()
        .copied()
        .zip(expected)
        .all(|(actual, expected)| (actual - expected).abs() <= 0.0001)
}

fn world_transform(owner: &CoreHandle) -> Mat2D {
    owner
        .with(|owner| {
            owner
                .as_world_transform_component()
                .map(|owner| *owner.world_transform())
        })
        .flatten()
        .expect("WorldTransformComponent")
}

fn assert_pose(
    artboard: &RuntimeArtboardInstanceHandle,
    owners: &IkOwners,
    seconds: f32,
    expected_target: (f32, f32),
    expected_bone_a: [f32; 6],
    expected_bone_b: [f32; 6],
    cycle: usize,
) {
    owners
        .animation
        .with_downcast_mut::<LinearAnimation, _>(|animation| {
            artboard.apply_linear_animation(animation, seconds, 1.0, None)
        })
        .expect("LinearAnimation");
    artboard.advance_default(0.0);
    assert_eq!(
        owners
            .target
            .with_downcast::<Node, _>(|target| (target.base.x(), target.base.y())),
        Some(expected_target)
    );
    let actual_bone_a = world_transform(&owners.bone_a);
    assert!(
        about_equal(actual_bone_a, expected_bone_a),
        "cycle {cycle} bone a mismatch: actual={actual_bone_a:?}, expected={expected_bone_a:?}"
    );
    let actual_bone_b = world_transform(&owners.bone_b);
    assert!(
        about_equal(actual_bone_b, expected_bone_b),
        "cycle {cycle} bone b mismatch: actual={actual_bone_b:?}, expected={expected_bone_b:?}"
    );
}

const BONE_A_AT_ZERO: [f32; 6] = [
    0.1163221150636673,
    -0.993211567401886,
    0.993211567401886,
    0.1163221150636673,
    26.015254974365234,
    475.2149658203125,
];
const BONE_B_AT_ZERO: [f32; 6] = [
    0.9740715622901917,
    0.22624030709266663,
    -0.22624030709266663,
    0.9740715622901917,
    64.31568145751953,
    148.1883544921875,
];
const BONE_A_AT_ONE: [f32; 6] = [
    0.6502798199653625,
    -0.7596948146820068,
    0.7596948146820068,
    0.6502798199653625,
    26.015254974365234,
    475.2149658203125,
];
const BONE_B_AT_ONE: [f32; 6] = [
    0.8823678493499756,
    0.47056037187576294,
    -0.4705604314804077,
    0.8823679089546204,
    240.1275634765625,
    225.07647705078125,
];

fn assert_both_poses(artboard: &RuntimeArtboardInstanceHandle, owners: &IkOwners, cycle: usize) {
    assert_pose(
        artboard,
        owners,
        0.0,
        (296.0, 202.0),
        BONE_A_AT_ZERO,
        BONE_B_AT_ZERO,
        cycle,
    );
    assert_pose(
        artboard,
        owners,
        1.0,
        (450.0, 337.0),
        BONE_A_AT_ONE,
        BONE_B_AT_ONE,
        cycle,
    );
}

fn fixture() -> (
    PersistentFactory<SerializingFactory>,
    RuntimeArtboardInstanceHandle,
) {
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let file = File::import(
        &pinned_fixture("two_bone_ik.riv"),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("import fixture");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    (factory, artboard)
}

#[test]
fn two_bone_ik_places_bones_correctly() {
    let (_factory, artboard) = fixture();
    let owners = setup(&artboard);
    assert_both_poses(&artboard, &owners, 0);
}

#[test]
fn ik_keeps_working_after_a_lot_of_iterations() {
    let (_factory, artboard) = fixture();
    let owners = setup(&artboard);
    for cycle in 0..1000 {
        assert_both_poses(&artboard, &owners, cycle);
    }
}
