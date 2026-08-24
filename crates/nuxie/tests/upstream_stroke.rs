//! Direct port of pinned `tests/unit_tests/runtime/stroke_test.cpp`.

use std::path::PathBuf;

use nuxie::File;

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

#[test]
fn stroke_can_be_looked_up_at_runtime() {
    let file = File::import(&pinned_fixture("stroke_name_test.riv")).expect("import fixture");
    let graph = file.default_artboard().expect("default artboard").graph();
    let stroke = graph.component_named("white_stroke").expect("named stroke");
    assert_eq!(stroke.type_name, "Stroke");
    let paint = graph
        .components
        .iter()
        .find(|component| {
            stroke.children.contains(&component.local_id) && component.type_name == "SolidColor"
        })
        .expect("stroke SolidColor paint");
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("instantiate artboard");

    assert!(artboard.raw_mut().set_color_property(
        paint.local_id,
        property_key("SolidColor", "colorValue"),
        0xff00_ffff,
    ));
}
