use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("silver-corpus workspace root")
        .to_path_buf()
}

fn runtime_root() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

fn compare_case(id: &str) -> anyhow::Result<()> {
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))?;
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .ok_or_else(|| anyhow::anyhow!("missing silver case {id}"))?;
    let runtime = runtime_root();
    let actual = Execution::run(case, &runtime)?;
    let expected_path = resolve_expected(&runtime, case);
    let expected_bytes = std::fs::read(&expected_path)?;
    let expected = parse_sriv(&expected_bytes)?;
    let actual = parse_sriv(actual.bytes())?;
    compare_sriv(&expected, &actual).map_err(|difference| anyhow::anyhow!("{id}: {difference}"))
}

#[test]
fn upstream_fl_bc_exact_silver_assertions() {
    // Literal fixture/action streams from the corresponding upstream
    // TEST_CASEs. The final comparison is the original `silver.matches(...)`
    // assertion against the pinned .sriv file.
    for id in [
        "focus_test",
        "multitouch",
        "multitouch_enter",
        "transition_index_condition",
    ] {
        compare_case(id).unwrap_or_else(|error| panic!("{error:#}"));
    }
}

#[test]
fn upstream_fl_bc_resolved_silver_assertions() {
    let mut differences = Vec::new();
    for id in [
        "focus_traversal",
        "hittest_ab1",
        "hittest_ab1_grand_parent",
        "hittest_ab1_parent",
        "hittest_nested",
        "sorted_listeners",
        "transition_actions",
        "transition_duration_bind_list",
        "transition_duration_bind_nested",
    ] {
        if let Err(error) = compare_case(id) {
            differences.push(format!("{error:#}"));
        }
    }
    assert!(
        differences.is_empty(),
        "pinned FL-B/FL-C silver divergences:\n{}",
        differences.join("\n")
    );
}

#[test]
fn advanced_pin_s4_divergences_are_replayed_and_recorded() {
    for (id, expected) in [
        (
            "bidirectional_stateful_property",
            "bidirectional_stateful_property: frame 3, op 180 (transform), field tx: expected 150, got 100",
        ),
        (
            "paused_nested_artboard_opacity",
            "paused_nested_artboard_opacity: frame 1, op 103 (rewind): expected rewind, got drawPath",
        ),
        (
            "layout_text_match",
            "layout_text_match: frame 0, op 61 (save): expected save, got frame",
        ),
    ] {
        let error = compare_case(id).expect_err("advanced-pin fixture unexpectedly became exact");
        assert_eq!(format!("{error:#}"), expected);
    }
}

#[test]
#[ignore = "Post-FL-D runner gap: Execution::run constructs raw nuxie_runtime ArtboardInstance/StateMachineInstance values and never creates a nuxie_scripting ScriptingVm or attaches the fixture's ScriptAsset occurrence, so ScriptedListenerAction::performAction remains inert"]
fn upstream_fl_bc_multi_listener_scripted_action_assertion() {
    compare_case("multi_listeners").unwrap_or_else(|error| panic!("{error:#}"));
}
