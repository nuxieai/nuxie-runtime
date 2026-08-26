//! Exact pinned Silver action streams for Wave C2 layout cases.

use silver_corpus::{
    compare_sriv, parse_sriv, read_manifest, resolve_expected, Action, ActionTarget, Actions, Case,
    Execution, Lane, Status,
};
use std::path::{Path, PathBuf};

const PROVENANCE: &str = "tests/unit_tests/runtime/layout_test.cpp";

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

fn replay(id: &str) {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read Silver corpus manifest");
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .unwrap_or_else(|| panic!("missing Silver case {id}"));
    assert_eq!(case.provenance_file, PROVENANCE);
    assert!(
        case.actions
            .executable()
            .is_some_and(|actions| !actions.is_empty()),
        "the shared manifest retains the complete pinned action stream"
    );
    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual).unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

#[test]
#[ignore = "expected-red: complete collapsing_elements SRIV diverges from the pinned renderer stream"]
fn wave_c2_layout_014_collapsing_and_soloing() {
    replay("collapsing_elements");
}

#[test]
#[ignore = "expected-red: complete layout_display SRIV diverges from the pinned renderer stream"]
fn wave_c2_layout_015_animating_layout_display() {
    replay("layout_display");
}

#[test]
#[ignore = "expected-red: complete layout_paint SRIV diverges from the pinned renderer stream"]
fn wave_c2_layout_016_background_and_foreground_paints() {
    replay("layout_paint");
}

#[test]
#[ignore = "expected-red: complete layout_anim_bound SRIV diverges from the pinned renderer stream"]
fn wave_c2_layout_017_animation_time_databound() {
    replay("layout_anim_bound");
}

#[test]
#[ignore = "expected-red: complete layout_anim_nested SRIV diverges from the pinned renderer stream"]
fn wave_c2_layout_018_animation_nested_artboards() {
    replay("layout_anim_nested");
}

#[test]
#[ignore = "expected-red: complete layout_anim_component_list SRIV diverges from the pinned renderer stream"]
fn wave_c2_layout_019_animation_component_list() {
    replay("layout_anim_component_list");
}

#[test]
#[ignore = "expected-red: complete layout_aspect_ratio SRIV diverges from the pinned renderer stream"]
fn wave_c2_layout_020_aspect_ratio() {
    replay("layout_aspect_ratio");
}

#[test]
#[ignore = "expected-red: complete layout_fixed_fill SRIV diverges from the pinned renderer stream"]
fn wave_c2_layout_021_fixed_fill_round_trip() {
    replay("layout_fixed_fill");
}

#[test]
#[ignore = "expected-red: the exact hug-artboard stream reaches the missing computed frame-size owner before SRIV parity"]
fn wave_c2_layout_022_top_level_hug_artboard() {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read Silver corpus manifest");
    let manifest_case = manifest
        .cases
        .iter()
        .find(|case| case.id == "layout_hug_artboard")
        .expect("hug-artboard case");
    assert_eq!(manifest_case.provenance_file, PROVENANCE);
    assert!(manifest_case
        .actions
        .executable()
        .is_some_and(<[Action]>::is_empty));
    let local_case = Case {
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
        actions: Actions::Executable(vec![
            Action::Advance {
                target: ActionTarget::Artboard,
                seconds: 0.0,
            },
            Action::Draw,
            Action::Frame,
            Action::Advance {
                target: ActionTarget::StateMachine,
                seconds: 0.016,
            },
            Action::Draw,
        ]),
        verification: manifest_case.verification.clone(),
        status: Status::UnsupportedFeature,
        producer_class: manifest_case.producer_class.clone(),
        provenance_file: manifest_case.provenance_file.clone(),
        provenance_test: manifest_case.provenance_test.clone(),
        producer_line: manifest_case.producer_line,
        note: manifest_case.note.clone(),
    };
    let actual = Execution::run(&local_case, &runtime)
        .expect("execute the exact advance/draw/frame/advance/draw owner stream");
    let expected = parse_sriv(
        &std::fs::read(resolve_expected(&runtime, &local_case)).expect("read pinned SRIV"),
    )
    .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("layout_hug_artboard: {difference}"));
}
