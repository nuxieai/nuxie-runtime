//! Distinct pinned Silver replays for callable Wave C9 state-machine cases.

use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::{Path, PathBuf};

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
        .expect("Wave C9 corpus case");
    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual).unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

#[test]
fn wave_c9_event_010_targetting_an_event_object_triggers_correctly() {
    replay("target_event");
}

#[test]
fn wave_c9_state_machine_009_transition_with_list_index_compares_to_number() {
    replay("transition_index_condition");
}

#[test]
fn wave_c9_state_machine_010_listeners_are_sorted_in_the_right_order() {
    replay("sorted_listeners");
}

#[test]
fn wave_c9_state_machine_011_listeners_with_multiple_event_types() {
    replay("multi_listeners");
}

#[test]
fn wave_c9_state_machine_013_nested_transition_duration_is_bindable() {
    replay("transition_duration_bind_nested");
}

#[test]
fn wave_c9_state_machine_014_list_transition_duration_is_bindable() {
    replay("transition_duration_bind_list");
}

#[test]
fn wave_c9_state_machine_016_component_based_transition_conditions() {
    replay("component_based_conditions");
}

#[test]
fn wave_c9_state_machine_017_component_conditions_with_other_props() {
    replay("component_based_conditions-Artboard2");
}

#[test]
fn wave_c9_state_machine_018_transitions_and_layers_trigger_actions() {
    replay("transition_actions");
}

#[test]
fn wave_c9_state_machine_019_paused_machine_updates_opacity_and_layout() {
    replay("paused_nested_artboard_opacity");
}
