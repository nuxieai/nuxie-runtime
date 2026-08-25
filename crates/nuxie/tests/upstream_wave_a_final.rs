//! Final executable Wave A ports whose first review found empty or narrower evidence.

use std::path::PathBuf;

use nuxie::File;
use nuxie_render_api::SerializingFactory;

fn pinned_bytes(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    std::fs::read(root.join("tests/unit_tests/assets").join(name))
        .unwrap_or_else(|error| panic!("read pinned fixture {name}: {error}"))
}

fn close(actual: f32, expected: f32, margin: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= margin,
        "{label}: expected {expected} +/- {margin}, got {actual}"
    );
}

fn number(view_model: &nuxie::ViewModelInstance, name: &str) -> f32 {
    view_model
        .raw()
        .number_value_by_property_name_path(name)
        .unwrap_or_else(|| panic!("computed number property {name}"))
}

#[test]
#[ignore = "expected-red: live Image computedWidth/Height remains 0 instead of pinned initial 150"]
fn image_computed_width_height_tracks_layout_resize_complete_port() {
    let file = File::import(&pinned_bytes("image_computed_transform_bind.riv"))
        .expect("import image computed transform fixture");
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("instantiate default artboard");
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = artboard
        .instantiate_default_view_model_instance()
        .expect("default view-model instance");
    let _ = artboard.bind_view_model(&view_model);
    let _ = machine.bind_owned_view_model_handle(view_model.handle());

    let mut silver = SerializingFactory::new();
    let (width, height) = artboard.artboard_dimensions();
    silver.frame_size(width as u32, height as u32);
    let mut renderer = silver.make_renderer();
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(&mut machine),
        0.0,
        &mut view_model,
    );
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(&mut machine),
        0.016,
        &mut view_model,
    );
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("draw initial computed values frame");

    close(number(&view_model, "img1Width"), 150.0, 5.0, "initial img1Width");
    close(number(&view_model, "img1Height"), 150.0, 5.0, "initial img1Height");
    close(number(&view_model, "img2Width"), 150.0, 5.0, "initial img2Width");
    close(number(&view_model, "img2Height"), 150.0, 5.0, "initial img2Height");

    for _ in 0..(2.0_f32 / 0.032_f32) as usize {
        silver.add_frame();
        artboard.advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut machine),
            0.032,
            &mut view_model,
        );
        artboard
            .draw(&mut silver, &mut renderer)
            .expect("draw computed values animation frame");
    }

    close(number(&view_model, "img1Width"), 200.0, 0.01, "settled img1Width");
    close(number(&view_model, "img1Height"), 200.0, 0.01, "settled img1Height");
    close(number(&view_model, "img2Width"), 250.0, 0.01, "settled img2Width");
    close(number(&view_model, "img2Height"), 250.0, 0.01, "settled img2Height");
    assert!(silver.bytes().len() > 16, "translated silver stream is non-empty");
}

#[test]
#[ignore = "expected-red: pinned SRIV comparator is not wired into nuxie integration tests"]
fn data_bind_blobs_internal_external_complete_action_flow() {
    let file = File::import(&pinned_bytes("data_bind_blob_test.riv"))
        .expect("import data-bind blob fixture");
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("instantiate default artboard");
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = artboard
        .instantiate_default_view_model_instance()
        .expect("default view-model instance");
    let _ = artboard.bind_view_model(&view_model);
    let _ = machine.bind_owned_view_model_handle(view_model.handle());

    let mut silver = SerializingFactory::new();
    let (width, height) = artboard.artboard_dimensions();
    silver.frame_size(width as u32, height as u32);
    let mut renderer = silver.make_renderer();
    for elapsed in [0.1, 0.1, 0.5, 0.5, 0.5, 0.5] {
        if silver.bytes().len() > 16 {
            silver.add_frame();
        }
        artboard.advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut machine),
            elapsed,
            &mut view_model,
        );
        artboard
            .draw(&mut silver, &mut renderer)
            .expect("draw blob data-bind frame");
    }

    let external = pinned_bytes("data_enum_roundtrip.rml");
    let asset = std::sync::Arc::new(nuxie_runtime::RuntimeBlobAsset::new(
        "data_enum_roundtrip.rml",
        std::sync::Arc::from(external.clone()),
    ));
    assert!(
        view_model
            .raw_mut()
            .set_live_blob_asset_by_property_name_path("xml", Some(asset.clone()))
    );
    let applied = view_model
        .raw()
        .blob_asset_value_by_property_name_path("xml")
        .expect("xml blob property after external assignment");
    assert!(
        applied
            .live_blob_asset()
            .is_some_and(|current| std::sync::Arc::ptr_eq(current, &asset))
    );
    assert_eq!(applied.live_blob_bytes().map(<[u8]>::len), Some(external.len()));
    silver.add_frame();
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(&mut machine),
        0.5,
        &mut view_model,
    );
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("draw external blob frame");
    assert!(silver.bytes().len() > 16, "complete live action stream executed");
    panic!("expected-red: pinned data_bind_blob_test SRIV comparator is unavailable");
}
