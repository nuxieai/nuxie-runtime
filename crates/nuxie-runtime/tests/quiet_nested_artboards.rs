//! Quiet nested artboards: a NestedArtboard whose mounted instance and nested
//! state machines are at rest skips its advance and data-bind passes until
//! its instance wakes it, as a quiet component list row does. A run that
//! really skips must draw every frame exactly as a run that never quiets.
#![cfg(feature = "testing")]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    RuntimeFactoryHandle,
    source::{
        artboard_component_list::ArtboardComponentList,
        file::{File, RuntimeFileHandle},
    },
};

const FRAMES: usize = 150;

fn import(asset: &str, factory: &mut PersistentFactory<RecordingFactory>) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR");
    let bytes = std::fs::read(
        std::path::PathBuf::from(root)
            .join("tests/unit_tests/assets")
            .join(asset),
    )
    .expect("upstream asset");
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("upstream asset imports")
}

/// Each frame's draw fingerprint, and the quiet skips counted on the way.
fn draws(asset: &str, quiet: bool, verify: bool) -> (Vec<u64>, u64) {
    ArtboardComponentList::set_quiet_rows_enabled(quiet);
    ArtboardComponentList::set_verify_quiet_rows(verify);
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = import(asset, &mut factory);
    let artboard = file.with_file(File::artboard_default).unwrap();
    let machine = artboard
        .default_state_machine_handle()
        .expect("a default state machine");
    let view_model = file.with_file(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    });
    if let Some(view_model) = view_model {
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model));
    }
    let skips = ArtboardComponentList::quiet_row_skips();
    let mut frames = Vec::with_capacity(FRAMES);
    for _ in 0..FRAMES {
        machine.advance_and_apply(1.0 / 60.0);
        factory.borrow_mut().clear();
        let mut renderer = factory.borrow().make_renderer();
        artboard.draw(&mut renderer);
        frames.push(factory.borrow().canonical_recording().fnv1a64());
    }
    let skips = ArtboardComponentList::quiet_row_skips() - skips;
    ArtboardComponentList::set_quiet_rows_enabled(true);
    ArtboardComponentList::set_verify_quiet_rows(true);
    (frames, skips)
}

#[test]
fn quiet_nested_artboards_draw_what_awake_ones_draw() {
    let mut skips = 0;
    for asset in [
        "script_create_text_runs.riv",
        "data_viz_demo.riv",
        "collapsing_elements.riv",
    ] {
        let (awake, awake_skips) = draws(asset, false, true);
        assert_eq!(awake_skips, 0, "{asset}: quieting is off");
        // Verified: every skipped pass runs anyway and must do no work.
        let (verified, verified_skips) = draws(asset, true, true);
        assert_eq!(verified, awake, "{asset}: verified run");
        // Skipping: the passes really are skipped.
        let (skipped, _) = draws(asset, true, false);
        assert_eq!(skipped, awake, "{asset}: skipping run");
        eprintln!("{asset}: {verified_skips} quiet skips");
        skips += verified_skips;
    }
    assert!(skips > 0, "nothing went quiet");
}
