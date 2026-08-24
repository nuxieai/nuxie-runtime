//! Direct ports of both cases in pinned `tests/unit_tests/runtime/ik_test.cpp`.

use std::path::PathBuf;

use nuxie::{ArtboardInstance, File, Mat2D};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name).expect("schema definition");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            nuxie_schema::definition_by_name(ancestor)
                .expect("ancestor definition")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .unwrap_or_else(|| panic!("property {type_name}.{property_name}"))
        .key
        .int
}

fn is_a(type_name: &str, expected: &str) -> bool {
    nuxie_schema::definition_by_name(type_name).is_some_and(|definition| {
        definition.name == expected || definition.ancestors.contains(&expected)
    })
}

#[derive(Clone, Copy)]
struct IkLocals {
    bone_a: usize,
    bone_b: usize,
    target: usize,
    animation: usize,
}

fn setup(file: &File) -> IkLocals {
    let artboard = file.default_artboard().expect("default artboard");
    let graph = artboard.graph();
    let circle_a = graph.component_named("circle a").expect("circle a");
    assert!(is_a(circle_a.type_name, "Shape"));
    let circle_b = graph.component_named("circle b").expect("circle b");
    assert!(is_a(circle_b.type_name, "Shape"));
    let bone_a = graph.component_named("a").expect("bone a");
    assert!(is_a(bone_a.type_name, "Bone"));
    let bone_b = graph.component_named("b").expect("bone b");
    assert!(is_a(bone_b.type_name, "Bone"));
    let target = graph.component_named("target").expect("target");
    assert!(is_a(target.type_name, "Node"));
    let animation = artboard
        .animation_index_named("Animation 1")
        .expect("Animation 1");

    assert!(bone_b.dependent_locals.contains(&circle_a.local_id));
    assert!(bone_b.dependent_locals.contains(&circle_b.local_id));

    IkLocals {
        bone_a: bone_a.local_id,
        bone_b: bone_b.local_id,
        target: target.local_id,
        animation,
    }
}

fn about_equal(actual: Mat2D, expected: [f32; 6]) -> bool {
    actual
        .0
        .into_iter()
        .zip(expected)
        .all(|(actual, expected)| (actual - expected).abs() <= 0.0001)
}

fn assert_pose(
    artboard: &mut ArtboardInstance<'_>,
    locals: IkLocals,
    seconds: f32,
    expected_target: (f32, f32),
    expected_bone_a: [f32; 6],
    expected_bone_b: [f32; 6],
) {
    let _ = artboard
        .raw_mut()
        .apply_linear_animation(locals.animation, seconds, 1.0);
    artboard.advance(0.0);
    assert_eq!(
        artboard
            .raw()
            .double_property(locals.target, property_key("Node", "x")),
        Some(expected_target.0)
    );
    assert_eq!(
        artboard
            .raw()
            .double_property(locals.target, property_key("Node", "y")),
        Some(expected_target.1)
    );
    assert!(about_equal(
        artboard
            .world_transform(locals.bone_a)
            .expect("bone a transform"),
        expected_bone_a,
    ));
    assert!(about_equal(
        artboard
            .world_transform(locals.bone_b)
            .expect("bone b transform"),
        expected_bone_b,
    ));
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

fn assert_both_poses(artboard: &mut ArtboardInstance<'_>, locals: IkLocals) {
    assert_pose(
        artboard,
        locals,
        0.0,
        (296.0, 202.0),
        BONE_A_AT_ZERO,
        BONE_B_AT_ZERO,
    );
    assert_pose(
        artboard,
        locals,
        1.0,
        (450.0, 337.0),
        BONE_A_AT_ONE,
        BONE_B_AT_ONE,
    );
}

#[test]
fn two_bone_ik_places_bones_correctly() {
    let file = File::import(&pinned_fixture("two_bone_ik.riv")).expect("import fixture");
    let locals = setup(&file);
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("instantiate artboard");
    assert_both_poses(&mut artboard, locals);
}

#[test]
fn ik_keeps_working_after_a_lot_of_iterations() {
    let file = File::import(&pinned_fixture("two_bone_ik.riv")).expect("import fixture");
    let locals = setup(&file);
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("instantiate artboard");

    for _ in 0..1000 {
        assert_both_poses(&mut artboard, locals);
    }
}
