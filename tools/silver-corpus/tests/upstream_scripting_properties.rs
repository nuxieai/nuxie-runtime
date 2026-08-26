//! One-for-one scripted silver ports of pinned
//! `tests/unit_tests/runtime/scripting/scripting_properties_test.cpp#9-#16`.
//!
//! Each test spells out its own upstream action sequence. `Execution::run`
//! is the shared high-level owner: it imports the scripting profile, registers
//! the File VM, instantiates the selected Artboard and StateMachine, attaches
//! scripted objects, binds the real ViewModel, draws through
//! `PersistentFactory<SerializingFactory>`, and returns the resulting SRIV.

use std::path::{Path, PathBuf};

use silver_corpus::{
    Action, ActionTarget, Actions, Case, Execution, Lane, PointerCoordinate, Status, compare_sriv,
    parse_sriv,
};

const UPSTREAM_SOURCE: &str = "tests/unit_tests/runtime/scripting/scripting_properties_test.cpp";

fn runtime_root() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

fn literal(value: f32) -> PointerCoordinate {
    PointerCoordinate::Literal(value)
}

fn artboard_width_over_two() -> PointerCoordinate {
    PointerCoordinate::Expression("artboard-width/2".to_owned())
}

fn run_exact_scripted_silver(
    runtime: &Path,
    ordinal: usize,
    id: &str,
    fixture: &str,
    sample_times: Vec<f32>,
    actions: Vec<Action>,
) -> anyhow::Result<()> {
    let case = Case {
        id: id.to_owned(),
        expected: format!("tests/unit_tests/silvers/{id}.sriv"),
        source: fixture.to_owned(),
        dependencies: Vec::new(),
        artboard: "default".to_owned(),
        animation: "none".to_owned(),
        state_machine: "default".to_owned(),
        lane: Lane::Scripted,
        deterministic: "cpp-test-defined".to_owned(),
        random: "cpp-test-defined".to_owned(),
        view_model: "cpp-test-defined".to_owned(),
        sample_times,
        actions: Actions::Executable(actions),
        verification: "sriv-v1-epsilon".to_owned(),
        status: Status::Diverges,
        producer_class: "runtime-literal".to_owned(),
        provenance_file: UPSTREAM_SOURCE.to_owned(),
        provenance_test: format!("scripting_properties_test.cpp#{ordinal}"),
        producer_line: 0,
        note: "literal test-local scripted silver action stream".to_owned(),
    };
    let actual = Execution::run(&case, runtime)?;
    let expected_path = runtime
        .join("tests/unit_tests/silvers")
        .join(format!("{id}.sriv"));
    let expected = parse_sriv(&std::fs::read(&expected_path)?)?;
    let actual = parse_sriv(actual.bytes())?;
    compare_sriv(&expected, &actual).map_err(|difference| anyhow::anyhow!("{id}: {difference}"))
}

#[test]
#[ignore = "expected-red: viewmodel_access frame 0 op 32 expects transform, live scripted SRIV emits save"]
fn wave_c12_silver_009_access_view_model_properties_and_enum_properties() {
    run_exact_scripted_silver(
        &runtime_root(),
        9,
        "viewmodel_access",
        "viewmodel_access.riv",
        vec![0.016],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "expected-red: viewmodel_from_instance frame 0 op 8 expects makeRenderPaint, live scripted SRIV emits frameSize"]
fn wave_c12_silver_010_creates_view_models_from_specified_named_instances() {
    run_exact_scripted_silver(
        &runtime_root(),
        10,
        "viewmodel_from_instance",
        "viewmodel_from_instance.riv",
        vec![0.016],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "expected-red: replace_view_model frame 0 op 42 transform tx expects 0, live scripted SRIV emits 250"]
fn wave_c12_silver_011_replace_a_view_model_property_value_from_a_script() {
    run_exact_scripted_silver(
        &runtime_root(),
        11,
        "replace_view_model",
        "replace_view_model.riv",
        vec![0.016, 0.016],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
            Action::Frame,
            Action::PointerDown {
                x: artboard_width_over_two(),
                y: literal(480.0),
                pointer_id: 0,
            },
            Action::PointerUp {
                x: artboard_width_over_two(),
                y: literal(480.0),
                pointer_id: 0,
            },
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "expected-red: remove_from_list frame 0 op 165 expects save, live scripted SRIV emits restore"]
fn wave_c12_silver_012_scripts_can_remove_items_from_lists() {
    let mut actions = vec![
        Action::BindDefaultViewModel,
        Action::Advance {
            target: ActionTarget::StateMachine,
            seconds: 0.1,
        },
        Action::Draw,
    ];
    for _ in 0..10 {
        actions.extend([
            Action::Frame,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ]);
    }
    run_exact_scripted_silver(
        &runtime_root(),
        12,
        "remove_from_list",
        "remove_from_list.riv",
        std::iter::once(0.1)
            .chain(std::iter::repeat_n(0.016, 10))
            .collect(),
        actions,
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "expected-red: list_index_script_access frame 0 op 80 addRawPath expects 33 fields, live scripted SRIV emits 808"]
fn wave_c12_silver_013_expose_list_index_to_scripts_and_ensure_type_is_correct() {
    run_exact_scripted_silver(
        &runtime_root(),
        13,
        "list_index_script_access",
        "list_index_script_access.riv",
        vec![0.1],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.1,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "expected-red: scripted_property_image frame 0 op 18 expects save, live scripted SRIV emits restore"]
fn wave_c12_silver_014_scripted_image_properties() {
    run_exact_scripted_silver(
        &runtime_root(),
        14,
        "scripted_property_image",
        "scripted_property_image.riv",
        vec![0.016],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "expected-red: image_scripting_property_value frame 0 op 23 transform tx expects -702, live scripted SRIV emits -139"]
fn wave_c12_silver_015_image_read_from_property_value() {
    run_exact_scripted_silver(
        &runtime_root(),
        15,
        "image_scripting_property_value",
        "image_scripting_property_value.riv",
        vec![0.0, 0.25],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.0,
            },
            Action::Draw,
            Action::Frame,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.25,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
#[ignore = "expected-red: reset_shared_viewmodel_instance_test frame 0 op 10 expects makeRenderPaint, live scripted SRIV emits frameSize"]
fn wave_c12_silver_016_reset_detached_view_model_instances_at_end_of_frame() {
    run_exact_scripted_silver(
        &runtime_root(),
        16,
        "reset_shared_viewmodel_instance_test",
        "reset_shared_viewmodel_instance_test.riv",
        vec![0.0, 0.016, 0.016, 0.016, 0.016, 0.016],
        vec![
            Action::BindDefaultViewModel,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.0,
            },
            Action::Draw,
            Action::Frame,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
            Action::Frame,
            Action::FireViewModelTrigger {
                property: "tri1".to_owned(),
            },
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
            Action::Frame,
            Action::FireViewModelTrigger {
                property: "tri1".to_owned(),
            },
            Action::PointerDown {
                x: literal(45.0),
                y: literal(165.0),
                pointer_id: 0,
            },
            Action::PointerUp {
                x: literal(45.0),
                y: literal(165.0),
                pointer_id: 0,
            },
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
            Action::Frame,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
            Action::Frame,
            Action::PointerDown {
                x: literal(45.0),
                y: literal(165.0),
                pointer_id: 0,
            },
            Action::PointerUp {
                x: literal(45.0),
                y: literal(165.0),
                pointer_id: 0,
            },
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#}"));
}
