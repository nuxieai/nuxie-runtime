use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("silver-corpus workspace root")
        .to_path_buf()
}

fn runtime_root(test: &str) -> Option<PathBuf> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").map(PathBuf::from);
    if root.is_none() {
        eprintln!(
            "skipping {test}; RIVE_RUNTIME_DIR is unset; point it at a pinned rive-runtime checkout"
        );
    }
    root
}

fn compare_case(id: &str, runtime: &Path) -> anyhow::Result<()> {
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))?;
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .ok_or_else(|| anyhow::anyhow!("missing silver case {id}"))?;
    let actual = Execution::run(case, runtime)?;
    let expected_path = resolve_expected(runtime, case);
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
    let Some(runtime) = runtime_root("upstream FL-B/FL-C exact silver assertions") else {
        return;
    };
    for id in [
        "focus_test",
        "multitouch",
        "multitouch_enter",
        "transition_index_condition",
    ] {
        compare_case(id, &runtime).unwrap_or_else(|error| panic!("{error:#}"));
    }
}

#[test]
fn upstream_fl_bc_resolved_silver_assertions() {
    let Some(runtime) = runtime_root("upstream FL-B/FL-C resolved silver assertions") else {
        return;
    };
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
        if let Err(error) = compare_case(id, &runtime) {
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
    let Some(runtime) = runtime_root("advanced-pin S4 silver divergence replay") else {
        return;
    };
    // bidirectional_stateful_property went exact when the nested-VMI binding
    // order was fixed (V20/V21/V34/V38): the stateful VMI now binds to the
    // mounted child before its state machine consumes the DataContext, so the
    // frame-3 transform no longer lags a pass behind.
    compare_case("bidirectional_stateful_property", &runtime)
        .unwrap_or_else(|error| panic!("promoted fixture regressed: {error:#}"));
    for (id, expected) in [
        (
            "paused_nested_artboard_opacity",
            "paused_nested_artboard_opacity: frame 1, op 103 (rewind): expected rewind, got drawPath",
        ),
        (
            "layout_text_match",
            "layout_text_match: frame 0, op 61 (save): expected save, got frame",
        ),
    ] {
        let error =
            compare_case(id, &runtime).expect_err("advanced-pin fixture unexpectedly became exact");
        assert_eq!(format!("{error:#}"), expected);
    }
}

#[test]
#[ignore = "coverage finding: docs/upstream-test-findings.md#finding-silver-scripted-listener-harness-gap — Execution::run constructs raw nuxie_runtime ArtboardInstance/StateMachineInstance values and never creates a nuxie_scripting ScriptingVm or attaches the fixture's ScriptAsset occurrences, so ScriptedListenerAction::performAction remains inert"]
fn upstream_fl_bc_multi_listener_scripted_action_assertion() {
    let Some(runtime) = runtime_root("upstream FL-B/FL-C multi-listener scripted assertion") else {
        return;
    };
    compare_case("multi_listeners", &runtime).unwrap_or_else(|error| panic!("{error:#}"));
}
