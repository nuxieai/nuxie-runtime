//! tests/unit_tests/runtime/serialized_scripts_test.cpp at d97f3547.

use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::PathBuf;
use std::sync::Mutex;

// Catch runs these cases serially. Execution changes process-wide test RNG state.
static REPLAY_LOCK: Mutex<()> = Mutex::new(());

fn replay_and_compare(id: &str) {
    let _guard = REPLAY_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let upstream = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR pinned fixture checkout"),
    );
    let manifest = read_manifest(
        &PathBuf::from(
            option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
        )
        .join("../../silver-corpus.toml"),
    )
    .unwrap();
    let case = manifest.cases.iter().find(|case| case.id == id).unwrap();
    let actual = Execution::run(case, &upstream)
        .unwrap_or_else(|error| panic!("{id} replay failed: {error:#}"));
    let expected = std::fs::read(resolve_expected(&upstream, case)).unwrap();
    compare_sriv(
        &parse_sriv(&expected).unwrap(),
        &parse_sriv(actual.bytes()).unwrap(),
    )
    .unwrap_or_else(|error| panic!("{id} Silver comparison: {error}"));
}

#[test]
#[ignore = "expected-red UNIV-3015: unadjudicated geometry at frame 14 op 4852, AddRawPath x 826.0928 -> 1375.4121"]
fn game_menu_ad_script_test() {
    replay_and_compare("game_menu_ad_police_files");
}

#[test]
#[ignore = "expected-red UNIV-3015: unadjudicated geometry at frame 0 op 3862, transform tx 842.5 -> 800"]
fn inventory_script_test() {
    replay_and_compare("inventory_demo_test_v2");
}

#[test]
#[ignore = "expected-red UNIV-3015: unadjudicated geometry at frame 0 op 34900, AddRawPath x 1194.5 -> 2105"]
fn layout_planets_script_test() {
    replay_and_compare("layoutstest_8-planets-grid");
}
