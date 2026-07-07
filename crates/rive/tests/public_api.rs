use rive::{File, RecordingFactory};
use std::path::PathBuf;

#[test]
fn public_api_imports_lists_instantiates_and_draws() {
    let fixture = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets/shapetest.riv");
    let bytes = std::fs::read(&fixture).expect("read fixture");
    let file = File::import(&bytes).expect("import file");

    assert!(file.artboard_count() >= 1);
    let names = file
        .artboards()
        .map(|artboard| artboard.name().unwrap_or("<unnamed>").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(names.len(), file.artboard_count());

    let artboard = file.default_artboard().expect("default artboard");
    let mut instance = artboard.instantiate().expect("instantiate artboard");
    instance.advance(0.0);

    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    instance
        .draw(&mut factory, &mut renderer)
        .expect("draw artboard");

    let stream = factory.stream();
    assert!(stream.contains("rive-golden-stream-v1"));
    assert!(stream.contains("drawPath"));
}
