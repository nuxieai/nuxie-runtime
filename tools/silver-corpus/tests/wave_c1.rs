//! Exact pinned grid/stack Silver action streams for Wave C1.

use silver_corpus::{
    Action, ActionTarget, Actions, Case, Execution, Lane, Status, compare_sriv, parse_sriv,
    read_manifest, resolve_expected,
};
use std::path::{Path, PathBuf};

const PROVENANCE: &str = "tests/unit_tests/runtime/layout_grid_stack_silver_test.cpp";

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn runtime_root() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

fn exact_grid_stack_actions() -> Vec<Action> {
    let mut actions = vec![
        Action::Advance {
            target: ActionTarget::StateMachine,
            seconds: 0.0,
        },
        Action::Draw,
    ];
    for _ in 0..120 {
        actions.extend([
            Action::Frame,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ]);
    }
    assert_eq!(actions.len(), 362);
    actions
}

fn replay(id: &str, participant_actions: bool) {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read silver corpus manifest");
    let manifest_case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .expect("Wave C1 corpus case");
    assert_eq!(manifest_case.provenance_file, PROVENANCE);

    let local_case;
    let case = if participant_actions {
        assert!(
            manifest_case
                .actions
                .executable()
                .is_some_and(<[Action]>::is_empty),
            "the local action stream exists only until the shared manifest owner adopts it",
        );
        local_case = Case {
            id: manifest_case.id.clone(),
            expected: manifest_case.expected.clone(),
            source: manifest_case.source.clone(),
            dependencies: manifest_case.dependencies.clone(),
            artboard: manifest_case.artboard.clone(),
            animation: manifest_case.animation.clone(),
            state_machine: manifest_case.state_machine.clone(),
            lane: Lane::Runtime,
            deterministic: manifest_case.deterministic.clone(),
            random: manifest_case.random.clone(),
            view_model: manifest_case.view_model.clone(),
            sample_times: vec![0.0, 0.016],
            actions: Actions::Executable(exact_grid_stack_actions()),
            verification: manifest_case.verification.clone(),
            status: Status::UnsupportedFeature,
            producer_class: manifest_case.producer_class.clone(),
            provenance_file: manifest_case.provenance_file.clone(),
            provenance_test: manifest_case.provenance_test.clone(),
            producer_line: manifest_case.producer_line,
            note: manifest_case.note.clone(),
        };
        &local_case
    } else {
        assert_eq!(
            manifest_case.actions.executable().map(<[Action]>::len),
            Some(362),
            "the manifest must retain the complete 120-frame helper stream",
        );
        manifest_case
    };

    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual).unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

#[test]
#[ignore = "expected-red: exact grid-with-layouts SRIV diverges at frame 1 operation 228"]
fn wave_c1_layout_grid_stack_001_grid_with_layouts() {
    replay("layout_grid_stack_grid_with_layouts", false);
}

#[test]
#[ignore = "expected-red: exact stack-with-layouts SRIV diverges at frame 1 operation 228"]
fn wave_c1_layout_grid_stack_002_stack_with_layouts() {
    replay("layout_grid_stack_stack_with_layouts", false);
}

#[test]
fn wave_c1_layout_grid_stack_003_grid_with_layouts_size_changing() {
    replay("layout_grid_stack_grid_with_layouts_size_changing", false);
}

#[test]
#[ignore = "expected-red: exact grid-with-layouts-span SRIV diverges at frame 34 operation 1116"]
fn wave_c1_layout_grid_stack_004_grid_with_layouts_span() {
    replay("layout_grid_stack_grid_with_layouts_span", false);
}

#[test]
#[ignore = "expected-red: exact size-span-changing SRIV diverges at frame 32 operation 1592"]
fn wave_c1_layout_grid_stack_005_grid_with_layouts_size_span_changing() {
    replay(
        "layout_grid_stack_grid_with_layouts_size_span_changing",
        false,
    );
}

#[test]
#[ignore = "expected-red: exact layout-participant draw fails because Text local 57 retains no render styles"]
fn wave_c1_layout_grid_stack_006_grid_with_layout_participants() {
    replay("layout_grid_stack_grid_with_layout_participants", true);
}
