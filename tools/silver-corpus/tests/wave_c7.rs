//! Exact pinned Silver replay for the callable Wave C7 TextInput render case.

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

fn replay_text_input_silver() {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read Silver corpus manifest");
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == "text_input")
        .expect("text_input corpus case");
    assert_eq!(
        case.provenance_file,
        "tests/unit_tests/runtime/text_input_test.cpp"
    );
    assert_eq!(
        case.provenance_test,
        "file with text input renders correctly"
    );

    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("text_input: {difference}"));
}

#[test]
fn wave_c7_text_input_002_file_with_text_input_renders_correctly() {
    replay_text_input_silver();
}
