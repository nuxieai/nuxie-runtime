//! Direct port of pinned
//! `tests/unit_tests/runtime/nested_text_run_test.cpp`.

use std::path::PathBuf;

use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_runtime::ArtboardInstance;

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn nested_text_run(_artboard: &mut ArtboardInstance, _run: &str, _path: &str) -> Vec<u8> {
    panic!("Rust has no named nested TextValueRun path lookup owner")
}

fn set_nested_text_run(_artboard: &mut ArtboardInstance, _run: &str, _path: &str, _value: &[u8]) {
    panic!("Rust has no named nested TextValueRun path mutation owner")
}

#[test]
#[ignore = "expected-red: Rust has no getTextRun(run, nested_path) equivalent"]
fn validate_nested_text_get_set() {
    let file = read_runtime_file(&pinned_fixture("runtime_nested_text_runs.riv"))
        .expect("runtime_nested_text_runs.riv imports");
    let graphs =
        GraphFile::from_runtime_file(&file).expect("runtime_nested_text_runs.riv graph builds");
    let graph = graphs
        .artboards
        .iter()
        .find(|artboard| artboard.name.as_deref() == Some("ArtboardA"))
        .expect("ArtboardA graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("ArtboardA instantiates");
    assert_eq!(artboard.state_machines().len(), 1);

    assert_eq!(
        nested_text_run(&mut artboard, "ArtboardBRun", "ArtboardB-1"),
        b"Artboard B Run"
    );
    assert_eq!(
        nested_text_run(&mut artboard, "ArtboardBRun", "ArtboardB-2"),
        b"Artboard B Run"
    );

    assert_eq!(
        nested_text_run(&mut artboard, "ArtboardCRun", "ArtboardB-1/ArtboardC-1"),
        b"Artboard C Run"
    );
    assert_eq!(
        nested_text_run(&mut artboard, "ArtboardCRun", "ArtboardB-1/ArtboardC-2"),
        b"Artboard C Run"
    );
    assert_eq!(
        nested_text_run(&mut artboard, "ArtboardCRun", "ArtboardB-2/ArtboardC-1"),
        b"Artboard C Run"
    );
    assert_eq!(
        nested_text_run(&mut artboard, "ArtboardCRun", "ArtboardB-2/ArtboardC-2"),
        b"Artboard C Run"
    );

    set_nested_text_run(
        &mut artboard,
        "ArtboardBRun",
        "ArtboardB-1",
        b"Artboard B1 Run Updated",
    );
    set_nested_text_run(
        &mut artboard,
        "ArtboardBRun",
        "ArtboardB-2",
        b"Artboard B2 Run Updated",
    );
    set_nested_text_run(
        &mut artboard,
        "ArtboardCRun",
        "ArtboardB-1/ArtboardC-1",
        b"Artboard B1C1 Run Updated",
    );
    set_nested_text_run(
        &mut artboard,
        "ArtboardCRun",
        "ArtboardB-1/ArtboardC-2",
        b"Artboard B1C2 Run Updated",
    );
    set_nested_text_run(
        &mut artboard,
        "ArtboardCRun",
        "ArtboardB-2/ArtboardC-1",
        b"Artboard B2C1 Run Updated",
    );
    set_nested_text_run(
        &mut artboard,
        "ArtboardCRun",
        "ArtboardB-2/ArtboardC-2",
        b"Artboard B2C2 Run Updated",
    );

    assert_eq!(
        nested_text_run(&mut artboard, "ArtboardBRun", "ArtboardB-1"),
        b"Artboard B1 Run Updated"
    );
    assert_eq!(
        nested_text_run(&mut artboard, "ArtboardBRun", "ArtboardB-2"),
        b"Artboard B2 Run Updated"
    );
    assert_eq!(
        nested_text_run(&mut artboard, "ArtboardCRun", "ArtboardB-1/ArtboardC-1"),
        b"Artboard B1C1 Run Updated"
    );
    assert_eq!(
        nested_text_run(&mut artboard, "ArtboardCRun", "ArtboardB-1/ArtboardC-2"),
        b"Artboard B1C2 Run Updated"
    );
    assert_eq!(
        nested_text_run(&mut artboard, "ArtboardCRun", "ArtboardB-2/ArtboardC-1"),
        b"Artboard B2C1 Run Updated"
    );
    assert_eq!(
        nested_text_run(&mut artboard, "ArtboardCRun", "ArtboardB-2/ArtboardC-2"),
        b"Artboard B2C2 Run Updated"
    );
}
