//! Direct ports of all five cases in pinned
//! tests/unit_tests/runtime/data_binding_keyframes.cpp.
use nuxie_render_api::{PersistentFactory, SerializingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    node::Node,
    text::text_value_run::TextValueRun,
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_color::ViewModelInstanceColor,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_string::ViewModelInstanceString,
    },
};
use nuxie_runtime::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};
use std::path::PathBuf;

use nuxie_sriv as sriv;

fn pinned_path(relative: &str) -> PathBuf {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    PathBuf::from(root).join("tests/unit_tests").join(relative)
}

struct Fixture {
    // Retain the defining file and the same factory used at import.
    _file: RuntimeFileHandle,
    silver: PersistentFactory<SerializingFactory>,
    artboard: RuntimeArtboardInstanceHandle,
    state_machine: RuntimeStateMachineInstanceHandle,
    view_model: CoreHandle,
}

fn fixture() -> Fixture {
    let bytes = std::fs::read(pinned_path("assets/data_bind_keyframes_test.riv"))
        .expect("pinned keyframe fixture");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut silver).expect("retained factory");
    let file = File::import(&bytes, retained, None, None, None).expect("native import");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let state_machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine 0");
    let view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .expect("default view-model instance");
    Fixture {
        _file: file,
        silver,
        artboard,
        state_machine,
        view_model,
    }
}

fn property(view_model: &CoreHandle, name: &str) -> CoreHandle {
    view_model
        .with_downcast::<ViewModelInstance, _>(|instance| instance.property_value_named(name))
        .flatten()
        .unwrap_or_else(|| panic!("view-model property {name}"))
}

fn set_start_text(view_model: &CoreHandle, text: &str) {
    property(view_model, "keyfTextStart")
        .with_downcast_mut::<ViewModelInstanceString, _>(|value| {
            value.set_value(text);
        })
        .expect("keyfTextStart string");
}

fn set_start_x(view_model: &CoreHandle, x: f32) {
    property(view_model, "startX")
        .with_downcast_mut::<ViewModelInstanceNumber, _>(|value| {
            value.set_value(x);
        })
        .expect("startX number");
}

fn bind_and_advance(fixture: &Fixture, seconds: f32) {
    fixture.state_machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(fixture.view_model.clone());
    });
    fixture.state_machine.advance_and_apply(seconds);
}

fn first_text_run(fixture: &Fixture) -> Option<Vec<u8>> {
    let run = fixture
        .artboard
        .with_artboard(|artboard| artboard.find_all_handles::<TextValueRun>().first().cloned())?;
    run.with_downcast::<TextValueRun, _>(|run| run.base.text().as_bytes().to_vec())
}

fn catch_approx_eq(actual: f32, expected: f32) -> bool {
    let actual = f64::from(actual);
    let expected = f64::from(expected);
    let scale = f64::from(f32::EPSILON) * 100.0 * expected.abs();
    let difference = (actual - expected).abs();
    difference <= scale
}

#[test]
fn catch_approx_widens_float_operands_before_comparing() {
    let expected = f32::from_bits(0x0072_abfc);
    let actual = f32::from_bits(expected.to_bits() + 90);
    assert!(!catch_approx_eq(actual, expected));
}

fn any_node(fixture: &Fixture, predicate: impl Fn(f32) -> bool) -> bool {
    let nodes = fixture
        .artboard
        .with_artboard(|artboard| artboard.find_all_handles::<Node>());
    nodes.iter().any(|node| {
        node.with(|object| predicate(object.as_node().expect("Node or derived owner").base.x()))
            .expect("live Node")
    })
}

fn any_node_has_x(fixture: &Fixture, expected: f32) -> bool {
    any_node(fixture, |actual| catch_approx_eq(actual, expected))
}

#[test]
fn data_binding_keyframes() {
    let fixture = fixture();
    let (width, height) = fixture
        .artboard
        .with_artboard(|artboard| (artboard.width(), artboard.height()));
    fixture
        .silver
        .borrow_mut()
        .frame_size(width as u32, height as u32);
    let mut renderer = fixture.silver.borrow().make_renderer();
    bind_and_advance(&fixture, 0.016);
    fixture.artboard.draw(&mut renderer);

    let frames = (1.0_f32 / 0.2_f32) as usize;
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.state_machine.advance_and_apply(0.2);
        fixture.artboard.draw(&mut renderer);
    }

    set_start_text(&fixture.view_model, "updated--text");
    property(&fixture.view_model, "colorStart")
        .with_downcast_mut::<ViewModelInstanceColor, _>(|value| {
            value.set_value(0xffff_ff00u32 as i32)
        })
        .expect("colorStart color");
    set_start_x(&fixture.view_model, 100.0);

    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.state_machine.advance_and_apply(0.2);
        fixture.artboard.draw(&mut renderer);
    }

    let expected = std::fs::read(pinned_path("silvers/data_bind_keyframes_test.sriv"))
        .expect("pinned keyframe silver");
    let actual = fixture.silver.borrow().bytes().to_vec();
    // Upstream matches checks the exact byte count before comparing typed values.
    assert_eq!(actual.len(), expected.len(), "pinned SRIV byte length");
    let expected = sriv::parse_sriv(&expected).expect("valid pinned SRIV");
    let actual = sriv::parse_sriv(&actual).expect("valid native SRIV");
    sriv::compare_sriv(&expected, &actual).expect("pinned keyframe binding silver");
}

#[test]
fn keyframe_value_binds_resolve_view_model_values_on_the_first_frame() {
    let fixture = fixture();
    set_start_text(&fixture.view_model, "SENTINEL_START");
    set_start_x(&fixture.view_model, 424_242.0);
    bind_and_advance(&fixture, 0.0);
    assert_eq!(
        first_text_run(&fixture).as_deref(),
        Some(&b"SENTINEL_START"[..])
    );
    assert!(any_node_has_x(&fixture, 424_242.0));
}

#[test]
fn keyframe_value_binds_update_when_the_source_view_model_changes() {
    let fixture = fixture();
    set_start_text(&fixture.view_model, "first");
    set_start_x(&fixture.view_model, 10.0);
    bind_and_advance(&fixture, 0.0);
    assert_eq!(first_text_run(&fixture).as_deref(), Some(&b"first"[..]));
    assert!(any_node_has_x(&fixture, 10.0));

    set_start_text(&fixture.view_model, "second");
    set_start_x(&fixture.view_model, 987.0);
    fixture.state_machine.advance_and_apply(0.0);
    assert_eq!(first_text_run(&fixture).as_deref(), Some(&b"second"[..]));
    assert!(any_node_has_x(&fixture, 987.0));
}

#[test]
fn keyframe_interpolation_reads_the_data_bound_start_value() {
    let fixture = fixture();
    let bound_start = 100_000.0;
    set_start_x(&fixture.view_model, bound_start);
    bind_and_advance(&fixture, 0.0);
    assert!(any_node_has_x(&fixture, bound_start));
    fixture.state_machine.advance_and_apply(0.5);
    let in_tween = any_node(&fixture, |x| x > 50_000.0 && x < bound_start);
    assert!(in_tween);
}

#[test]
fn standalone_animation_instance_ignores_keyframe_value_binds() {
    // The pinned standalone case creates no StateMachineInstance: creating one
    // would itself build the keyframe bindings this case must exclude.
    let bytes = std::fs::read(pinned_path("assets/data_bind_keyframes_test.riv"))
        .expect("pinned keyframe fixture");
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, retained, None, None, None).expect("native import");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    assert!(artboard.with_artboard(|artboard| artboard.animation_count()) > 0);
    let view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .expect("default view-model instance");
    set_start_text(&view_model, "SHOULD_NOT_BIND");
    set_start_x(&view_model, 424_242.0);
    artboard.bind_view_model_instance(Some(view_model));
    let mut animation = artboard.animation_at(0).expect("animation 0");
    animation.advance_and_apply(0.0);
    let run = artboard
        .with_artboard(|artboard| artboard.find_all_handles::<TextValueRun>().first().cloned())
        .expect("first text run");
    assert_ne!(
        run.with_downcast::<TextValueRun, _>(|run| run.base.text().to_owned())
            .expect("TextValueRun"),
        "SHOULD_NOT_BIND"
    );
    let nodes = artboard.with_artboard(|artboard| artboard.find_all_handles::<Node>());
    assert!(!nodes.iter().any(|node| {
        node.with(|node| catch_approx_eq(node.as_node().expect("Node").base.x(), 424_242.0))
            .expect("live Node")
    }));
}
