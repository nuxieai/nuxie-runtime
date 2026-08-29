//! Final executable Wave A ports whose first review found empty or narrower evidence.

use std::{path::PathBuf, sync::Arc};

use nuxie::{
    File, PersistentFactory, RuntimeFactoryHandle, RuntimeViewModelInstanceHandle,
    ViewModelInstanceRuntime,
};
use nuxie_render_api::SerializingFactory;
use nuxie_runtime::RuntimeBlobAsset;
use silver_corpus::{compare_sriv, parse_sriv};

fn pinned(relative: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let path = root.join("tests/unit_tests").join(relative);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn compare_silver(name: &str, actual: &[u8]) {
    let expected =
        parse_sriv(&pinned(&format!("silvers/{name}.sriv"))).expect("pinned SRIV parses");
    let actual = parse_sriv(actual).expect("Rust SRIV parses");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("{name} differs: {difference}"));
}

fn close(actual: f32, expected: f32, margin: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= margin,
        "{label}: expected {expected} +/- {margin}, got {actual}"
    );
}

fn number(view_model: &RuntimeViewModelInstanceHandle, name: &str) -> f32 {
    view_model
        .property_number(name)
        .unwrap_or_else(|| panic!("computed number property {name}"))
        .value()
}

#[test]
fn image_computed_width_height_tracks_layout_resize_complete_port() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut silver).expect("retained factory");
    let file = File::import(
        &pinned("assets/image_computed_transform_bind.riv"),
        factory,
        None,
        None,
        None,
    )
    .expect("import image computed transform fixture");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("default view-model instance");
    machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(view_model.instance());
    });

    let mut renderer = silver.borrow().make_renderer();
    machine.advance_and_apply(0.0);
    machine.advance_and_apply(0.016);
    artboard.draw(&mut renderer);

    close(
        number(&view_model, "img1Width"),
        150.0,
        5.0,
        "initial img1Width",
    );
    close(
        number(&view_model, "img1Height"),
        150.0,
        5.0,
        "initial img1Height",
    );
    close(
        number(&view_model, "img2Width"),
        150.0,
        5.0,
        "initial img2Width",
    );
    close(
        number(&view_model, "img2Height"),
        150.0,
        5.0,
        "initial img2Height",
    );

    for _ in 0..(2.0_f32 / 0.032_f32) as usize {
        silver.borrow_mut().add_frame();
        machine.advance_and_apply(0.032);
        artboard.draw(&mut renderer);
    }

    close(
        number(&view_model, "img1Width"),
        200.0,
        0.01,
        "settled img1Width",
    );
    close(
        number(&view_model, "img1Height"),
        200.0,
        0.01,
        "settled img1Height",
    );
    close(
        number(&view_model, "img2Width"),
        250.0,
        0.01,
        "settled img2Width",
    );
    close(
        number(&view_model, "img2Height"),
        250.0,
        0.01,
        "settled img2Height",
    );
    compare_silver("image_computed_transform_bind", &silver.borrow().bytes());
}

#[test]
#[ignore = "expected-red: data_bind_blob_test frame 0 op 24 expected makeRenderPaint, got save"]
fn data_bind_blobs_internal_external_complete_action_flow() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut silver).expect("retained factory");
    let file = File::import(
        &pinned("assets/data_bind_blob_test.riv"),
        factory,
        None,
        None,
        None,
    )
    .expect("import data-bind blob fixture");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut renderer = silver.borrow().make_renderer();
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let model_id = artboard.with_artboard(|artboard| artboard.view_model_id());
    assert_ne!(model_id, u32::MAX);
    let view_model = file
        .with_file(|file| file.create_view_model_instance_at(model_id as usize, 0))
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("authored view-model instance 0");
    let blob = view_model.property_blob("xml").expect("xml blob property");
    machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(view_model.instance());
    });

    machine.advance_and_apply(0.1);
    artboard.draw(&mut renderer);
    silver.borrow_mut().add_frame();
    machine.advance_and_apply(0.1);
    artboard.draw(&mut renderer);
    for _ in 0..(2.0_f32 / 0.5_f32) as usize {
        silver.borrow_mut().add_frame();
        machine.advance_and_apply(0.5);
        artboard.draw(&mut renderer);
    }

    let external = pinned("assets/data_enum_roundtrip.rml");
    let asset = Arc::new(RuntimeBlobAsset::new(
        "data_enum_roundtrip.rml",
        Arc::from(external.clone().into_boxed_slice()),
    ));
    blob.set_value(Some(asset.clone()));
    let applied = blob
        .testing_value()
        .expect("xml blob after external assignment");
    assert!(Arc::ptr_eq(&applied, &asset));
    assert_eq!(applied.bytes().len(), external.len());
    silver.borrow_mut().add_frame();
    machine.advance_and_apply(0.5);
    artboard.draw(&mut renderer);

    compare_silver("data_bind_blob_test", &silver.borrow().bytes());
}
