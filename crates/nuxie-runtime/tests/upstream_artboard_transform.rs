//! Direct ports of all six cases in pinned
//! `tests/unit_tests/runtime/artboard_transform_test.cpp`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::GraphFile;
use nuxie_render_api::{RecordingFactory, SerializingFactory, Vec2D};
use nuxie_runtime::{
    ArtboardInstance, Mat2D, RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance,
};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
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

struct Fixture {
    file: RuntimeFile,
    graphs: GraphFile,
    graph_index: usize,
    artboard: ArtboardInstance,
}

fn fixture(asset: &str, artboard_name: Option<&str>) -> Fixture {
    let file = read_runtime_file(&pinned_fixture(asset))
        .unwrap_or_else(|error| panic!("{asset} imports: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&file)
        .unwrap_or_else(|error| panic!("{asset} graph builds: {error:#}"));
    let graph_index = artboard_name
        .map(|name| {
            graphs
                .artboards
                .iter()
                .position(|graph| graph.name.as_deref() == Some(name))
                .unwrap_or_else(|| panic!("{asset} has artboard {name}"))
        })
        .unwrap_or(0);
    let artboard = ArtboardInstance::from_graph_with_artboards(
        &file,
        &graphs.artboards[graph_index],
        &graphs.artboards,
    )
    .unwrap_or_else(|error| panic!("{asset} artboard instantiates: {error:#}"));
    Fixture {
        file,
        graphs,
        graph_index,
        artboard,
    }
}

fn draw_recording(fixture: &mut Fixture) -> String {
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    fixture
        .artboard
        .draw_artboard(
            &fixture.file,
            &fixture.graphs.artboards[fixture.graph_index],
            &fixture.graphs.artboards,
            &mut factory,
            &mut renderer,
            &BTreeMap::new(),
            None,
            true,
        )
        .expect("artboard draws");
    factory.stream()
}

fn draw_serializing(fixture: &mut Fixture, factory: &mut SerializingFactory) {
    let mut renderer = factory.make_renderer();
    fixture
        .artboard
        .draw_artboard(
            &fixture.file,
            &fixture.graphs.artboards[fixture.graph_index],
            &fixture.graphs.artboards,
            factory,
            &mut renderer,
            &BTreeMap::new(),
            None,
            true,
        )
        .expect("artboard draws");
}

fn parse_matrix(line: &str) -> Option<Mat2D> {
    let values = line
        .strip_prefix("transform matrix=[")?
        .strip_suffix(']')?
        .split(',')
        .map(str::parse::<f32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let values: [f32; 6] = values.try_into().ok()?;
    Some(Mat2D(values))
}

fn contains_matrix(stream: &str, expected: Mat2D) -> bool {
    stream.lines().filter_map(parse_matrix).any(|actual| {
        actual
            .0
            .into_iter()
            .zip(expected.0)
            .all(|(actual, expected)| (actual - expected).abs() <= 0.0001)
    })
}

fn clip_transform(stream: &str) -> Option<Mat2D> {
    let mut stack = vec![Mat2D::IDENTITY];
    for line in stream.lines() {
        if line == "save" {
            stack.push(*stack.last()?);
        } else if line == "restore" {
            if stack.len() > 1 {
                stack.pop();
            }
        } else if let Some(transform) = parse_matrix(line) {
            let current = stack.last_mut()?;
            *current = current.multiply(transform);
        } else if line.starts_with("clipPath path=") {
            return stack.last().copied();
        }
    }
    None
}

#[test]
fn artboard_bakes_its_own_rotation_and_scale_into_draw() {
    let mut fixture = fixture("nested_artboard_opacity.riv", None);
    fixture.artboard.advance(0.0).expect("initial advance");
    fixture
        .artboard
        .set_double_property(0, property_key("Artboard", "scaleX"), 2.0);
    fixture
        .artboard
        .set_double_property(0, property_key("Artboard", "scaleY"), 3.0);
    fixture.artboard.advance(0.0).expect("scaled advance");
    assert!(contains_matrix(
        &draw_recording(&mut fixture),
        Mat2D([2.0, 0.0, 0.0, 3.0, 0.0, 0.0])
    ));

    fixture
        .artboard
        .set_double_property(0, property_key("Artboard", "rotation"), 1.570_796_3);
    fixture.artboard.advance(0.0).expect("rotated advance");
    let mut expected = Mat2D::from_rotation(1.570_796_3);
    expected.scale_by_values(2.0, 3.0);
    assert!(contains_matrix(&draw_recording(&mut fixture), expected));
}

#[test]
fn artboard_transform_is_only_pushed_when_non_default() {
    let mut plain = fixture("nested_artboard_opacity.riv", None);
    plain.artboard.advance(0.0).expect("plain advance");
    let plain_count = draw_recording(&mut plain)
        .lines()
        .filter(|line| line.starts_with("transform matrix="))
        .count();

    let mut scaled = fixture("nested_artboard_opacity.riv", None);
    scaled
        .artboard
        .set_double_property(0, property_key("Artboard", "scaleX"), 2.0);
    scaled.artboard.advance(0.0).expect("scaled advance");
    let scaled_count = draw_recording(&mut scaled)
        .lines()
        .filter(|line| line.starts_with("transform matrix="))
        .count();
    assert_eq!(scaled_count, plain_count + 1);
}

#[test]
fn artboard_rotation_is_honored_in_state_machine_hit_testing() {
    let mut fixture = fixture("opaque_hit_test.riv", Some("main"));
    fixture
        .artboard
        .set_double_property(0, property_key("Artboard", "rotation"), 3.141_592_7);
    fixture.artboard.advance(0.0).expect("rotated advance");
    let machine_index = fixture.graphs.artboards[fixture.graph_index]
        .state_machines
        .iter()
        .position(|machine| machine.name.as_deref() == Some("main-state-machine"))
        .expect("main-state-machine");
    let mut machine = fixture
        .artboard
        .state_machine_instance(machine_index)
        .expect("state machine");
    machine
        .advance_and_apply(&mut fixture.artboard, 0.0)
        .expect("machine advance");
    fixture.artboard.advance(0.0).expect("artboard advance");
    machine
        .advance_and_apply(&mut fixture.artboard, 0.0)
        .expect("second machine advance");
    assert!(
        !machine
            .get_bool("toGreen")
            .and_then(|input| input.bool_value())
            .expect("toGreen")
    );

    let (min_x, min_y, _, _) = fixture.artboard.artboard_bounds();
    let frame = (-min_x, -min_y);
    let content = (100.0, 250.0);
    let rotation = Mat2D::from_rotation(3.141_592_7);
    let rotated = rotation.transform_point(content.0 - frame.0, content.1 - frame.1);
    let world = (frame.0 + rotated.0, frame.1 + rotated.1);
    assert!((world.0 - content.0).hypot(world.1 - content.1) > 1.0);
    machine.pointer_down(&mut fixture.artboard, content.0, content.1, 0);
    assert!(
        !machine
            .get_bool("toGreen")
            .and_then(|input| input.bool_value())
            .expect("toGreen")
    );
    machine.pointer_down(&mut fixture.artboard, world.0, world.1, 0);
    assert!(
        machine
            .get_bool("toGreen")
            .and_then(|input| input.bool_value())
            .expect("toGreen")
    );
}

#[test]
#[ignore = "expected-red: Rust does not expose exact ArtboardInstance::rootTransform(point)"]
fn nested_artboards_own_rotation_affects_root_transform() {
    let mut fixture = fixture("nested_artboard_opacity.riv", None);
    fixture.artboard.update_pass();
    let mut saw_nested = false;
    fixture
        .artboard
        .try_visit_nested_artboard_instances_mut(&mut |_depth, graph_id, child| -> Result<(), ()> {
            let graph = fixture
                .graphs
                .artboards
                .iter()
                .find(|graph| graph.global_id == graph_id)
                .expect("nested graph");
            if graph.name.as_deref() != Some("Nested artboard") {
                return Ok(());
            }
            child.update_pass();
            let before = nested_root_transform(child, Vec2D::new(10.0, 0.0));
            child.set_double_property(0, property_key("Artboard", "rotation"), 1.570_796_4);
            child.update_pass();
            let after = nested_root_transform(child, Vec2D::new(10.0, 0.0));
            assert!((after.x - before.x).hypot(after.y - before.y) > 1.0);
            saw_nested = true;
            Ok(())
        })
        .expect("nested tree visits");
    assert!(saw_nested);
}

fn nested_root_transform(_: &mut ArtboardInstance, _: Vec2D) -> Vec2D {
    panic!("Rust does not expose the exact C++ ArtboardInstance::rootTransform(point) owner")
}

fn default_view_model(fixture: &Fixture) -> RuntimeOwnedViewModelHandle {
    let view_model_index = usize::try_from(
        fixture
            .file
            .artboard(fixture.graph_index)
            .and_then(|artboard| artboard.uint_property("viewModelId"))
            .expect("viewModelId"),
    )
    .expect("viewModelId fits usize");
    let instance_index = fixture
        .file
        .view_model_default_instance(view_model_index)
        .expect("default view-model instance")
        .instance_index;
    RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(
            &fixture.file,
            view_model_index,
            instance_index,
        )
        .expect("view model builds"),
    )
}

fn missing_silver_match(_: &str, _: &[u8]) -> bool {
    panic!("nuxie-runtime tests do not own the pinned SRIV comparator")
}

#[test]
#[ignore = "expected-red: pinned artboard transform/opacity silver comparator is not wired here"]
fn artboard_transform_and_opacity() {
    let mut fixture = fixture("artboard_opacity_and_transform_test.riv", None);
    let mut silver = SerializingFactory::new();
    let (width, height) = fixture.artboard.artboard_dimensions();
    silver.frame_size(width as u32, height as u32);
    let mut machine = fixture
        .artboard
        .state_machine_instance(0)
        .expect("state machine 0");
    let view_model = default_view_model(&fixture);
    machine.bind_owned_view_model_handle(&view_model);
    machine
        .advance_and_apply(&mut fixture.artboard, 0.1)
        .expect("initial advance");
    draw_serializing(&mut fixture, &mut silver);
    for _ in 0..11 {
        silver.add_frame();
        machine
            .advance_and_apply(&mut fixture.artboard, 0.1)
            .expect("frame advance");
        let view_model = view_model.borrow();
        let x = view_model
            .number_value_by_property_name_path("xPos")
            .expect("xPos");
        let y = view_model
            .number_value_by_property_name_path("yPos")
            .expect("yPos");
        drop(view_model);
        machine.pointer_down(&mut fixture.artboard, x, y, 0);
        machine.pointer_up(&mut fixture.artboard, x, y, 0);
        draw_serializing(&mut fixture, &mut silver);
    }
    assert!(missing_silver_match(
        "artboard_opacity_and_transform_test",
        &silver.bytes()
    ));
}

#[test]
fn artboard_clip_is_transformed_by_its_own_rotation() {
    let mut fixture = fixture("nested_artboard_opacity.riv", None);
    fixture
        .artboard
        .set_bool_property(0, property_key("Artboard", "clip"), true);
    fixture
        .artboard
        .set_double_property(0, property_key("Artboard", "rotation"), 1.570_796_3);
    fixture.artboard.advance(0.0).expect("rotated clip advance");
    let transform = clip_transform(&draw_recording(&mut fixture)).expect("clipPath was called");
    assert!(transform.0[1].abs() > 0.0001 || transform.0[2].abs() > 0.0001);
}
