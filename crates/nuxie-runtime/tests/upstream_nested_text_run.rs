//! Direct port of pinned
//! `tests/unit_tests/runtime/nested_text_run_test.cpp`.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    generated::{core_registry::CoreRegistry, text::text_value_run_base::TextValueRunBase},
    text::text_value_run::TextValueRun,
};
use nuxie_runtime::{
    Artboard, File, ImportResult, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn nested_text_run(artboard: &mut RuntimeArtboardInstanceHandle, run: &str, path: &str) -> Vec<u8> {
    let run = artboard
        .with_artboard(|artboard| artboard.get_text_run(run, path))
        .expect("nested TextValueRun");
    assert!(run.is_type_of(TextValueRunBase::TYPE_KEY));
    run.with_downcast::<TextValueRun, _>(|run| run.base.text().as_bytes().to_vec())
        .unwrap()
}

fn set_nested_text_run(
    artboard: &mut RuntimeArtboardInstanceHandle,
    run: &str,
    path: &str,
    value: &[u8],
) {
    let run = artboard
        .with_artboard(|artboard| artboard.get_text_run(run, path))
        .expect("nested TextValueRun");
    assert!(CoreRegistry::set_string_handle(
        &run,
        i32::from(TextValueRunBase::TEXT_PROPERTY_KEY),
        std::str::from_utf8(value).unwrap().to_owned()
    ));
}

#[test]
fn validate_nested_text_get_set() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(
        &pinned_fixture("runtime_nested_text_runs.riv"),
        retained,
        Some(&mut result),
        None,
        None,
    )
    .expect("runtime_nested_text_runs.riv imports");
    assert_eq!(result, ImportResult::Success);
    let source = file
        .with_file(|file| file.artboard_named_source("ArtboardA"))
        .expect("ArtboardA");
    let mut artboard = Artboard::instance_from_handle(&source).expect("ArtboardA instantiates");
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.state_machine_count()),
        1
    );

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
