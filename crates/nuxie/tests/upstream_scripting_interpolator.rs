//! ac0c03ae: tests/unit_tests/runtime/scripting/scripting_interpolator_test.cpp.
//! Exact upstream fixtures: their curves are linear before init and quadratic
//! afterward; the converter-input fixture captures its exponent during init.
#![cfg(feature = "scripting")]

use nuxie::{
    FileImportLimits, PersistentFactory, RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle, RuntimeViewModelInstanceHandle, ScriptExecutionLimits,
    ScriptedFile, ViewModelInstanceRuntime, import_unsigned_scripted,
};
use nuxie_render_api::RecordingFactory;
use nuxie_runtime::source::{
    animation::{
        state_machine::StateMachine, state_machine_layer::StateMachineLayer,
        state_transition::StateTransition,
    },
    core::{CoreHandle, CoreType},
    layout_component::LayoutComponent,
    nested_artboard_layout::NestedArtboardLayout,
    node::Node,
    scripted::scripted_interpolator::ScriptedInterpolator,
};

const QUAD_STEPS: [f32; 5] = [0.0, 6.25, 25.0, 56.25, 100.0];

fn read_file(name: &str) -> ScriptedFile {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream checkout");
    let path = std::path::PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    import_unsigned_scripted(
        &bytes,
        &mut factory,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("upstream scripted fixture imports")
}

fn artboard(file: &ScriptedFile, name: &str) -> RuntimeArtboardInstanceHandle {
    file.native_file()
        .with_file(|file| file.artboard_named(name))
        .expect("named artboard")
}

fn find<T: CoreType>(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
    artboard
        .with_artboard(|artboard| artboard.base.find_handle::<T>(name))
        .expect(name)
}

fn inner(host: &RuntimeArtboardInstanceHandle) -> RuntimeArtboardInstanceHandle {
    find::<NestedArtboardLayout>(host, "Inner")
        .with_downcast::<NestedArtboardLayout, _>(|nested| {
            nested.base.base.artboard_instance_handle(0)
        })
        .flatten()
        .expect("nested Inner artboard instance")
}

fn interpolator(artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    LayoutComponent::interpolator_occurrence(&artboard.core_handle()).expect("layout interpolator")
}

fn layout_interpolator(layout: &CoreHandle) -> Option<CoreHandle> {
    LayoutComponent::interpolator_occurrence(layout)
}

fn layout_width(layout: &CoreHandle) -> f32 {
    layout
        .with(|object| object.as_layout_component().unwrap().layout_width())
        .unwrap()
}

fn x(node: &CoreHandle) -> f32 {
    node.with(|object| object.as_node().unwrap().base.x())
        .unwrap()
}

fn approx(actual: f32, expected: f32) {
    // Catch Approx's default epsilon is 100 * FLT_EPSILON; its scale is zero.
    let actual = f64::from(actual);
    let expected = f64::from(expected);
    assert!(
        (actual - expected).abs() <= f64::from(f32::EPSILON) * 100.0 * expected.abs(),
        "{actual} != Approx({expected})"
    );
}

#[test]
fn layouts_that_inherit_share_the_definers_scripted_interpolator() {
    let file = read_file("scripted_interpolator_layout_inherit.riv");
    let host = artboard(&file, "Host");
    host.advance_default(0.0);
    let child = find::<LayoutComponent>(&host, "Child");
    let inner = inner(&host);
    let inner_child = find::<LayoutComponent>(&inner, "InnerChild");
    let curve = interpolator(&host);
    approx(
        curve
            .with_downcast::<ScriptedInterpolator, _>(|curve| curve.transform(0.5))
            .expect("ScriptedInterpolator"),
        0.25,
    );
    assert_eq!(layout_interpolator(&child), Some(curve.clone()));
    assert_eq!(interpolator(&inner), curve);
    assert_eq!(layout_interpolator(&inner_child), Some(curve));
    approx(layout_width(&child), 300.0);
    let height = host.with_artboard(|host| host.base.height());
    host.set_size(600.0, height);
    host.advance_default(0.25);
    for expected in QUAD_STEPS {
        host.advance_default(0.25);
        approx(layout_width(&child), 300.0 + expected * 2.0);
    }
}

#[test]
fn each_artboard_runs_its_own_layout_interpolator_instance() {
    let file = read_file("scripted_interpolator_layout_inherit.riv");
    let host_a = artboard(&file, "Host");
    let host_b = artboard(&file, "Host");
    host_a.advance_default(0.0);
    host_b.advance_default(0.0);
    assert_ne!(interpolator(&host_a), interpolator(&host_b));
    let inner_a = inner(&host_a);
    let inner_b = inner(&host_b);
    assert_eq!(interpolator(&inner_a), interpolator(&host_a));
    assert_eq!(interpolator(&inner_b), interpolator(&host_b));
}

fn transition_interpolator(machine: &CoreHandle) -> CoreHandle {
    let layer = machine
        .with_downcast::<StateMachine, _>(|machine| machine.layer(0))
        .flatten()
        .expect("layer 0");
    let states = layer
        .with_downcast::<StateMachineLayer, _>(|layer| layer.states().to_vec())
        .unwrap();
    for state in states {
        let transitions = state
            .with(|state| {
                (0..state.layer_state_transition_count().unwrap())
                    .map(|i| state.layer_state_transition(i).unwrap())
                    .collect::<Vec<_>>()
            })
            .unwrap();
        for transition in transitions {
            let curve = transition
                .with_downcast::<StateTransition, _>(|transition| transition.interpolator())
                .flatten();
            if let Some(curve) = curve {
                if curve
                    .with_downcast::<ScriptedInterpolator, _>(|_| ())
                    .is_some()
                {
                    return curve;
                }
            }
        }
    }
    panic!("scripted transition interpolator");
}

#[test]
fn scripted_transition_runs_its_state_machine_instances_clone() {
    let file = read_file("scripted_interpolator_transition_converter.riv");
    let artboard_a = artboard(&file, "Main");
    let artboard_b = artboard(&file, "Main");
    let machine_a = artboard_a.state_machine_at(0).unwrap();
    let machine_b = artboard_b.state_machine_at(0).unwrap();
    let shared =
        transition_interpolator(&machine_a.with_instance(|machine| machine.state_machine()));
    let clone_a = machine_a
        .with_instance(|machine| machine.scripted_object(&shared))
        .unwrap();
    let clone_b = machine_b
        .with_instance(|machine| machine.scripted_object(&shared))
        .unwrap();
    assert_ne!(clone_a, shared);
    assert_ne!(clone_a, clone_b);
    let mover = find::<Node>(&artboard_a, "Mover");
    for expected in QUAD_STEPS {
        machine_a.advance_and_apply(0.25);
        approx(x(&mover), expected);
    }
}

fn bind(machine: &RuntimeStateMachineInstanceHandle, vmi: &RuntimeViewModelInstanceHandle) {
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(vmi.instance()));
}

fn view_model_at(
    file: &ScriptedFile,
    artboard: &RuntimeArtboardInstanceHandle,
    index: usize,
) -> RuntimeViewModelInstanceHandle {
    let id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
    let instance = file
        .native_file()
        .with_file(|file| file.create_view_model_instance_at(id as usize, index))
        .expect("view model instance");
    ViewModelInstanceRuntime::new(instance).into_handle()
}

fn ease_value_to(
    machine: &RuntimeStateMachineInstanceHandle,
    vmi: &RuntimeViewModelInstanceHandle,
    node: &CoreHandle,
    target: f32,
    expected: &[f32],
) {
    vmi.property_number("value").unwrap().set_value(target);
    for expected in expected {
        machine.advance_and_apply(0.25);
        approx(x(node), *expected);
    }
}

#[test]
fn scripted_converter_interpolator_runs_per_bound_converter() {
    let file = read_file("scripted_interpolator_transition_converter.riv");
    let artboard = artboard(&file, "Main");
    let vmi = file
        .native_file()
        .with_file(|file| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .unwrap();
    let machine = artboard.state_machine_at(0).unwrap();
    bind(&machine, &vmi);
    machine.advance_and_apply(0.25);
    machine.advance_and_apply(0.25);
    let bound = find::<Node>(&artboard, "Bound");
    approx(x(&bound), 0.0);
    ease_value_to(&machine, &vmi, &bound, 100.0, &QUAD_STEPS);
}

#[test]
fn converter_interpolator_inits_with_bound_inputs_again_on_rebind() {
    let file = read_file("scripted_interpolator_converter_inputs.riv");
    let artboard = artboard(&file, "Main");
    let machine = artboard.state_machine_at(0).unwrap();
    let cubic = view_model_at(&file, &artboard, 0);
    let quad = view_model_at(&file, &artboard, 1);
    let eased = find::<Node>(&artboard, "Eased");
    bind(&machine, &cubic);
    machine.advance_and_apply(0.25);
    machine.advance_and_apply(0.25);
    ease_value_to(
        &machine,
        &cubic,
        &eased,
        100.0,
        &[0.0, 1.5625, 12.5, 42.1875, 100.0],
    );
    bind(&machine, &quad);
    machine.advance_and_apply(0.25);
    machine.advance_and_apply(0.25);
    ease_value_to(&machine, &quad, &eased, 100.0, &QUAD_STEPS);
}

#[test]
fn scripted_range_mapper_runs_its_own_initialized_clone() {
    let file = read_file("scripted_interpolator_converter_inputs.riv");
    let artboard = artboard(&file, "Main");
    let machine = artboard.state_machine_at(0).unwrap();
    let vmi = view_model_at(&file, &artboard, 0);
    bind(&machine, &vmi);
    machine.advance_and_apply(0.0);
    let mapped = find::<Node>(&artboard, "Mapped");
    let value = vmi.property_number("value").unwrap();
    for input in [25.0, 50.0, 75.0] {
        value.set_value(input);
        machine.advance_and_apply(0.0);
        approx(x(&mapped), input * input / 100.0);
    }
}
