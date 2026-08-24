//! Direct ports of the single-case rotation, scale, transform, and translation
//! constraint tests from pinned `tests/unit_tests/runtime`.

use std::path::PathBuf;

use nuxie::{File, Mat2D};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn named_local(file: &File, name: &str) -> usize {
    file.default_artboard()
        .expect("default artboard")
        .graph()
        .component_named(name)
        .unwrap_or_else(|| panic!("component {name}"))
        .local_id
}

fn decompose(matrix: Mat2D) -> (f32, f32, f32, f32, f32) {
    let [m0, m1, m2, m3, x, y] = matrix.0;
    let rotation = m1.atan2(m0);
    let scale_x = (m0 * m0 + m1 * m1).sqrt();
    let scale_y = if scale_x == 0.0 {
        0.0
    } else {
        (m0 * m3 - m2 * m1) / scale_x
    };
    (x, y, scale_x, scale_y, rotation)
}

#[test]
fn rotation_constraint_updates_world_transform() {
    let file = File::import(&pinned_fixture("rotation_constraint.riv")).expect("import fixture");
    let target = named_local(&file, "target");
    let rectangle = named_local(&file, "rect");
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("instantiate artboard");

    artboard.advance(0.0);
    let (_, _, _, _, target_rotation) =
        decompose(artboard.world_transform(target).expect("target transform"));
    let (_, _, _, _, rectangle_rotation) = decompose(
        artboard
            .world_transform(rectangle)
            .expect("rectangle transform"),
    );

    assert_eq!(target_rotation, rectangle_rotation);
}

#[test]
fn scale_constraint_updates_world_transform() {
    let file = File::import(&pinned_fixture("scale_constraint.riv")).expect("import fixture");
    let target = named_local(&file, "target");
    let rectangle = named_local(&file, "rect");
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("instantiate artboard");

    artboard.advance(0.0);
    let (_, _, target_scale_x, target_scale_y, _) =
        decompose(artboard.world_transform(target).expect("target transform"));
    let (_, _, rectangle_scale_x, rectangle_scale_y, _) = decompose(
        artboard
            .world_transform(rectangle)
            .expect("rectangle transform"),
    );

    assert_eq!(target_scale_x, rectangle_scale_x);
    assert_eq!(target_scale_y, rectangle_scale_y);
}

#[test]
fn transform_constraint_updates_world_transform() {
    let file = File::import(&pinned_fixture("transform_constraint.riv")).expect("import fixture");
    let target = named_local(&file, "Target");
    let rectangle = named_local(&file, "Rectangle");
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("instantiate artboard");

    artboard.advance(0.0);
    let target_transform = artboard.world_transform(target).expect("target transform");
    let rectangle_transform = artboard
        .world_transform(rectangle)
        .expect("rectangle transform");

    assert!(
        target_transform
            .0
            .iter()
            .zip(rectangle_transform.0)
            .all(|(target, rectangle)| (target - rectangle).abs() <= 0.0001)
    );
}

#[test]
fn translation_constraint_updates_world_transform() {
    let file = File::import(&pinned_fixture("translation_constraint.riv")).expect("import fixture");
    let target = named_local(&file, "target");
    let rectangle = named_local(&file, "rect");
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("instantiate artboard");

    artboard.advance(0.0);
    let (target_x, target_y, _, _, _) =
        decompose(artboard.world_transform(target).expect("target transform"));
    let (rectangle_x, rectangle_y, _, _, _) = decompose(
        artboard
            .world_transform(rectangle)
            .expect("rectangle transform"),
    );

    assert_eq!(target_x, rectangle_x);
    assert_eq!(target_y, rectangle_y);
}

#[test]
fn ik_with_skinned_bones_orders_correctly() {
    let file = File::import(&pinned_fixture("complex_ik_dependency.riv")).expect("import fixture");
    let graph = file.default_artboard().expect("default artboard").graph();
    let one = graph.component_named("One").expect("bone One");
    assert_eq!(one.type_name, "Bone");
    let two = graph.component_named("Two").expect("bone Two");
    assert_eq!(two.type_name, "Bone");
    let skin = graph
        .components
        .iter()
        .find(|component| component.type_name == "Skin")
        .expect("skin");
    let first_constraint = two.constraint_locals.first().expect("Two constraint");
    let constraint = graph
        .components
        .iter()
        .find(|component| component.local_id == *first_constraint)
        .expect("Two constraint component");

    assert_eq!(constraint.type_name, "IKConstraint");
    assert!(
        skin.graph_order.expect("skin graph order") > one.graph_order.expect("One graph order")
    );
    assert!(
        skin.graph_order.expect("skin graph order") > two.graph_order.expect("Two graph order")
    );
}
