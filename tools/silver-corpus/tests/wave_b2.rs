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

#[test]
fn wave_b2_deterministic_mode_replays_the_pinned_action_stream() {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read silver corpus manifest");
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == "deterministic_mode")
        .expect("deterministic_mode case");
    assert_eq!(
        case.provenance_file,
        "tests/unit_tests/runtime/file_test.cpp"
    );
    assert_eq!(
        case.provenance_test,
        "Test deterministic mode for randomization and elastic scroll physics"
    );

    let actual =
        Execution::run(case, &runtime).expect("execute pinned deterministic action stream");
    let expected_path = resolve_expected(&runtime, case);
    let expected_bytes = std::fs::read(&expected_path).expect("read pinned deterministic silver");
    let expected = parse_sriv(&expected_bytes).expect("parse pinned deterministic silver");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust deterministic stream");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("deterministic_mode: {difference}"));
}
