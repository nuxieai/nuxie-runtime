//! Direct ports of all five cases in pinned
//! `tests/unit_tests/runtime/data_binding_keyframes.cpp`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_render_api::SerializingFactory;
use nuxie_runtime::{
    ArtboardInstance, RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance,
    StateMachineInstance,
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
    artboard: ArtboardInstance,
    state_machine: StateMachineInstance,
    view_model: RuntimeOwnedViewModelHandle,
}

fn fixture() -> Fixture {
    let file = read_runtime_file(&pinned_fixture("data_bind_keyframes_test.riv"))
        .expect("data_bind_keyframes_test.riv imports");
    let graphs =
        GraphFile::from_runtime_file(&file).expect("data_bind_keyframes_test.riv graph builds");
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let state_machine = artboard.state_machine_instance(0).expect("state machine 0");
    let view_model_index = usize::try_from(
        file.artboard(0)
            .and_then(|artboard| artboard.uint_property("viewModelId"))
            .expect("default artboard viewModelId"),
    )
    .expect("viewModelId fits usize");
    let instance_index = file
        .view_model_default_instance(view_model_index)
        .expect("authored default view-model instance")
        .instance_index;
    let view_model = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(&file, view_model_index, instance_index)
            .expect("default view-model instance builds"),
    );
    Fixture {
        file,
        graphs,
        artboard,
        state_machine,
        view_model,
    }
}

fn graph(fixture: &Fixture) -> &ArtboardGraph {
    fixture
        .graphs
        .artboards
        .first()
        .expect("default artboard graph")
}

fn set_start_text(view_model: &RuntimeOwnedViewModelHandle, text: &str) {
    assert!(
        view_model
            .borrow_mut()
            .set_string_by_property_name_path("keyfTextStart", text.as_bytes())
    );
}

fn set_start_x(view_model: &RuntimeOwnedViewModelHandle, x: f32) {
    assert!(
        view_model
            .borrow_mut()
            .set_number_by_property_name_path("startX", x)
    );
}

fn bind_and_advance(fixture: &mut Fixture, seconds: f32) {
    fixture
        .state_machine
        .bind_owned_view_model_handle(&fixture.view_model);
    fixture
        .state_machine
        .advance_and_apply(&mut fixture.artboard, seconds)
        .expect("state machine advances");
}

fn first_text_run(fixture: &Fixture) -> Option<Vec<u8>> {
    let local = graph(fixture)
        .local_objects
        .iter()
        .find(|object| object.type_name == Some("TextValueRun"))?
        .local_id;
    fixture
        .artboard
        .debug_string_property(local, property_key("TextValueRun", "text"))
        .map(<[u8]>::to_vec)
}

fn any_node_has_x(fixture: &Fixture, expected: f32) -> bool {
    let x_key = property_key("Node", "x");
    graph(fixture).local_objects.iter().any(|object| {
        object
            .type_name
            .and_then(nuxie_schema::definition_by_name)
            .is_some_and(|definition| definition.is_a("Node"))
            && fixture
                .artboard
                .double_property(object.local_id, x_key)
                .is_some_and(|actual| (actual - expected).abs() <= 0.0001)
    })
}

fn draw(fixture: &mut Fixture, factory: &mut SerializingFactory) {
    let mut renderer = factory.make_renderer();
    fixture
        .artboard
        .draw_artboard(
            &fixture.file,
            &fixture.graphs.artboards[0],
            &fixture.graphs.artboards,
            factory,
            &mut renderer,
            &BTreeMap::new(),
            None,
            true,
        )
        .expect("keyframe fixture draws");
}

fn missing_silver_match(_: &str, _: &[u8]) -> bool {
    panic!("nuxie-runtime tests do not own the pinned SRIV comparator")
}

#[test]
#[ignore = "expected-red: pinned data_bind_keyframes_test silver comparator is not wired here"]
fn data_binding_keyframes() {
    let mut fixture = fixture();
    let mut silver = SerializingFactory::new();
    let (width, height) = fixture.artboard.artboard_dimensions();
    silver.frame_size(width as u32, height as u32);

    bind_and_advance(&mut fixture, 0.016);
    draw(&mut fixture, &mut silver);

    let frames = (1.0_f32 / 0.2_f32) as usize;
    for _ in 0..frames {
        silver.add_frame();
        fixture
            .state_machine
            .advance_and_apply(&mut fixture.artboard, 0.2)
            .expect("first-half state machine advances");
        draw(&mut fixture, &mut silver);
    }

    set_start_text(&fixture.view_model, "updated--text");
    assert!(
        fixture
            .view_model
            .borrow_mut()
            .set_color_by_property_name_path("colorStart", 0xffff_ff00)
    );
    set_start_x(&fixture.view_model, 100.0);

    for _ in 0..frames {
        silver.add_frame();
        fixture
            .state_machine
            .advance_and_apply(&mut fixture.artboard, 0.2)
            .expect("second-half state machine advances");
        draw(&mut fixture, &mut silver);
    }

    assert!(missing_silver_match(
        "data_bind_keyframes_test",
        &silver.bytes()
    ));
}

#[test]
fn keyframe_value_binds_resolve_view_model_values_on_the_first_frame() {
    let mut fixture = fixture();
    set_start_text(&fixture.view_model, "SENTINEL_START");
    set_start_x(&fixture.view_model, 424_242.0);

    bind_and_advance(&mut fixture, 0.0);

    assert_eq!(
        first_text_run(&fixture).as_deref(),
        Some(&b"SENTINEL_START"[..])
    );
    assert!(any_node_has_x(&fixture, 424_242.0));
}

#[test]
fn keyframe_value_binds_update_when_the_source_view_model_changes() {
    let mut fixture = fixture();
    set_start_text(&fixture.view_model, "first");
    set_start_x(&fixture.view_model, 10.0);
    bind_and_advance(&mut fixture, 0.0);

    assert_eq!(first_text_run(&fixture).as_deref(), Some(&b"first"[..]));
    assert!(any_node_has_x(&fixture, 10.0));

    set_start_text(&fixture.view_model, "second");
    set_start_x(&fixture.view_model, 987.0);
    fixture
        .state_machine
        .advance_and_apply(&mut fixture.artboard, 0.0)
        .expect("updated state machine advances");

    assert_eq!(first_text_run(&fixture).as_deref(), Some(&b"second"[..]));
    assert!(any_node_has_x(&fixture, 987.0));
}

#[test]
fn keyframe_interpolation_reads_the_data_bound_start_value() {
    let mut fixture = fixture();
    let bound_start = 100_000.0;
    set_start_x(&fixture.view_model, bound_start);
    bind_and_advance(&mut fixture, 0.0);
    assert!(any_node_has_x(&fixture, bound_start));

    fixture
        .state_machine
        .advance_and_apply(&mut fixture.artboard, 0.5)
        .expect("tween state machine advances");
    let x_key = property_key("Node", "x");
    let in_tween = graph(&fixture).local_objects.iter().any(|object| {
        object
            .type_name
            .and_then(nuxie_schema::definition_by_name)
            .is_some_and(|definition| definition.is_a("Node"))
            && fixture
                .artboard
                .double_property(object.local_id, x_key)
                .is_some_and(|x| x > 50_000.0 && x < bound_start)
    });
    assert!(in_tween);
}

#[test]
fn standalone_animation_instance_ignores_keyframe_value_binds() {
    let mut fixture = fixture();
    assert!(!fixture.artboard.linear_animations().is_empty());
    set_start_text(&fixture.view_model, "SHOULD_NOT_BIND");
    set_start_x(&fixture.view_model, 424_242.0);
    fixture
        .artboard
        .bind_owned_view_model_artboard_handle(&fixture.file, &fixture.view_model);

    let mut animation = fixture
        .artboard
        .linear_animation_instance(0)
        .expect("animation 0");
    fixture
        .artboard
        .advance_linear_animation_instance(&mut animation, 0.0);
    fixture
        .artboard
        .apply_linear_animation_instance(&animation, 1.0);

    assert_ne!(
        first_text_run(&fixture).as_deref(),
        Some(&b"SHOULD_NOT_BIND"[..])
    );
    assert!(!any_node_has_x(&fixture, 424_242.0));
}
