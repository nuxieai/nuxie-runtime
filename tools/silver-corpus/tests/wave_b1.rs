//! Live SRIV replays for every Wave B1 case already represented by the pinned corpus.

use silver_corpus::{
    Difference, Execution, Status, compare_sriv, parse_sriv, read_manifest, resolve_expected,
};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")))
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
    let (_, result) = replay_classified(id);
    result.unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

/// A stored `--no_ffp_contract` silver that the contracted production lane
/// reproduces only up to its recorded first difference. The strict-fp lane
/// mirrors the producer's arithmetic and must replay it exactly.
fn replay_ffp_contract_divergence(id: &str, expected: &str) {
    let (status, result) = replay_classified(id);
    assert_eq!(
        status,
        Status::Diverges,
        "{id} should be classified diverges"
    );
    if cfg!(feature = "strict-fp") {
        result.unwrap_or_else(|difference| panic!("{id} under strict-fp: {difference}"));
    } else {
        let difference = result.expect_err("contracted divergence should remain present");
        assert_eq!(difference.to_string(), expected);
    }
}

fn replay_classified(id: &str) -> (Status, Result<(), Difference>) {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read silver corpus manifest");
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .expect("Wave B1 corpus case");
    assert!(
        case.provenance_file
            .starts_with("tests/unit_tests/runtime/data_binding")
    );
    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    (case.status, compare_sriv(&expected, &actual))
}

#[test]
fn wave_b1_artboard_width_test() {
    replay("artboard_width_test");
}

#[test]
fn wave_b1_bidirectional_precedence_source_first() {
    replay("bidirectional_precedence-source_first");
}

#[test]
fn wave_b1_bidirectional_precedence_target_first() {
    replay("bidirectional_precedence-target_first");
}

#[test]
fn wave_b1_bidirectional_stateful_property() {
    replay("bidirectional_stateful_property");
}

#[test]
fn wave_b1_computed_root_transform_list() {
    replay("computed_root_transform-list");
}

#[test]
fn wave_b1_computed_root_transform_nested_artboard() {
    replay("computed_root_transform-nested_artboard");
}

#[test]
fn wave_b1_custom_property_enum() {
    replay("custom_property_enum");
}

#[test]
fn wave_b1_custom_property_trigger_bind() {
    replay("custom_property_trigger_bind");
}

#[test]
fn wave_b1_data_bind_font_test() {
    replay("data_bind_font_test");
}

#[test]
fn wave_b1_data_bind_keyframes_test() {
    replay("data_bind_keyframes_test");
}

#[test]
fn wave_b1_data_bind_solo_solos_to_values() {
    replay("data_bind_solo-solos-to-values");
}

#[test]
fn wave_b1_data_bind_solo_values_to_solos() {
    replay("data_bind_solo-values-to-solos");
}

#[test]
fn wave_b1_data_converter_interpolator_reset() {
    replay("data_converter_interpolator_reset");
}

#[test]
fn wave_b1_data_converter_to_number() {
    replay_ffp_contract_divergence(
        "data_converter_to_number",
        "frame 41, op 2120 (addRawPath): expected 1850 fields, got 1443",
    );
}

#[test]
fn wave_b1_databind_artboard() {
    replay("databind_artboard");
}

#[test]
fn wave_b1_format_number_with_commas() {
    replay("format_number_with_commas");
}

#[test]
fn wave_b1_image_fit_alignment() {
    replay("image_fit_alignment");
}

#[test]
fn wave_b1_image_fit_alignment_2() {
    replay("image_fit_alignment_2");
}

#[test]
fn wave_b1_image_fit_alignment_3() {
    replay("image_fit_alignment_3");
}

#[test]
fn wave_b1_image_fit_alignment_updated_test() {
    replay("image_fit_alignment_updated_test");
}

#[test]
fn wave_b1_interpolation_zero_duration() {
    replay("interpolation_zero_duration");
}

#[test]
fn wave_b1_list_to_length_test() {
    replay("list_to_length_test");
}

#[test]
fn wave_b1_list_to_path() {
    replay("list_to_path");
}

#[test]
fn wave_b1_listener_view_model() {
    replay("listener_view_model");
}

#[test]
fn wave_b1_relative_data_bind_path() {
    replay("relative_data_bind_path");
}

#[test]
fn wave_b1_relative_data_bind_path_fire_trigger() {
    replay("relative_data_bind_path-fire-trigger");
}

#[test]
fn wave_b1_relative_data_bind_path_listener() {
    replay("relative_data_bind_path-listener");
}

#[test]
fn wave_b1_relative_data_bind_path_scripted_input() {
    replay("relative_data_bind_path-scripted-input");
}

#[test]
fn wave_b1_relative_data_binding() {
    replay("relative_data_binding");
}

#[test]
fn wave_b1_state_transition_fire_trigger() {
    replay("state_transition_fire_trigger");
}

#[test]
fn wave_b1_time_based_interpolation() {
    replay("time_based_interpolation");
}

#[test]
fn wave_b1_trigger_based_listeners() {
    replay("trigger_based_listeners");
}

#[test]
fn wave_b1_trigger_fires_single_change() {
    replay("trigger_fires_single_change");
}

#[test]
fn wave_b1_unbound_stateful_component() {
    replay("unbound_stateful_component");
}

#[test]
fn wave_b1_viewmodel_based_condition() {
    replay("viewmodel_based_condition");
}

#[test]
fn wave_b1_viewmodel_image_reset() {
    replay("viewmodel_image_reset");
}
