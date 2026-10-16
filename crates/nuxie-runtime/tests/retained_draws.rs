//! Retained draws: a leaf artboard whose content stands since its last draw
//! replays the calls that draw recorded. A run that replays must draw every
//! frame exactly as a run that draws in full, and a write no component reports
//! to the artboard (a paint's visibility, read only when it draws) must not
//! replay stale calls. Testing builds also check every replay against a fresh
//! recording of the artboard's draw.
#![cfg(feature = "testing")]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    RuntimeFactoryHandle,
    source::{
        advance_flags::AdvanceFlags,
        artboard::{Artboard, RuntimeArtboardInstanceHandle},
        file::{File, RuntimeFileHandle},
        generated::{
            core_registry::CoreRegistry, shapes::paint::shape_paint_base::ShapePaintBase,
        },
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

fn draw(
    artboard: &RuntimeArtboardInstanceHandle,
    factory: &mut PersistentFactory<RecordingFactory>,
) -> u64 {
    factory.borrow_mut().clear();
    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);
    factory.borrow().canonical_recording().fnv1a64()
}

/// Each frame's draw fingerprint, and the replays counted on the way.
fn draws(asset: &str, retained: bool) -> (Vec<u64>, u64) {
    Artboard::set_retained_draws_enabled(retained);
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
    let replays = Artboard::retained_draw_replays();
    let mut frames = Vec::with_capacity(FRAMES);
    for _ in 0..FRAMES {
        machine.advance_and_apply(1.0 / 60.0);
        frames.push(draw(&artboard, &mut factory));
    }
    let replays = Artboard::retained_draw_replays() - replays;
    Artboard::set_retained_draws_enabled(true);
    (frames, replays)
}

#[test]
fn retained_draws_draw_what_full_draws_draw() {
    let mut replays = 0;
    for asset in [
        "component_list_1.riv",
        "data_viz_demo.riv",
        "script_create_text_runs.riv",
        "collapsing_elements.riv",
        "car_widgets_v01.riv",
    ] {
        let (full, full_replays) = draws(asset, false);
        assert_eq!(full_replays, 0, "{asset}: retaining is off");
        let (retained, asset_replays) = draws(asset, true);
        assert_eq!(retained, full, "{asset}");
        eprintln!("{asset}: {asset_replays} replays");
        replays += asset_replays;
    }
    assert!(replays > 0, "nothing replayed");
}

/// One instance of a file and its factory.
struct Run {
    factory: PersistentFactory<RecordingFactory>,
    artboard: RuntimeArtboardInstanceHandle,
    paints: Vec<nuxie_runtime::source::core::CoreHandle>,
    _file: RuntimeFileHandle,
}

impl Run {
    fn new(asset: &str) -> Self {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = import(asset, &mut factory);
        let artboard = file.with_file(File::artboard_default).unwrap();
        artboard.advance(0.0, AdvanceFlags::NONE);
        let paints = artboard.with_artboard(|artboard| {
            artboard
                .objects()
                .iter()
                .flatten()
                .filter(|object| object.is_type_of(ShapePaintBase::TYPE_KEY))
                .cloned()
                .collect()
        });
        Self {
            factory,
            artboard,
            paints,
            _file: file,
        }
    }

    fn set_visible(&self, index: usize, visible: bool) {
        assert!(CoreRegistry::set_bool_handle(
            &self.paints[index],
            ShapePaintBase::IS_VISIBLE_PROPERTY_KEY.into(),
            visible
        ));
    }
}

#[test]
fn hiding_a_paint_never_replays_its_draw() {
    let asset = "circle_clips.riv";
    let mut retained = Run::new(asset);
    let mut full = Run::new(asset);
    assert!(!retained.paints.is_empty(), "the artboard has paints");
    // Both draw every step; the retained run must draw what the full run does.
    let step = |retained: &mut Run, full: &mut Run, what: &str| {
        let drawn = draw(&retained.artboard, &mut retained.factory);
        Artboard::set_retained_draws_enabled(false);
        assert_eq!(drawn, draw(&full.artboard, &mut full.factory), "{what}");
        Artboard::set_retained_draws_enabled(true);
        drawn
    };
    let mut changed = 0;
    for index in 0..retained.paints.len() {
        // Seen, recorded, then replayed.
        let replays = Artboard::retained_draw_replays();
        let shown = (0..4).map(|_| step(&mut retained, &mut full, "shown")).last();
        assert!(Artboard::retained_draw_replays() > replays, "the draw replays");
        // Visibility reports nothing to the artboard; only its draw reads it.
        retained.set_visible(index, false);
        full.set_visible(index, false);
        let hidden = (0..3).map(|_| step(&mut retained, &mut full, "hidden")).last();
        changed += usize::from(hidden != shown);
        retained.set_visible(index, true);
        full.set_visible(index, true);
    }
    assert!(changed > 0, "hiding a paint changes the draw");
}
