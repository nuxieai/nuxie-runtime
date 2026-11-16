//! The fallback-fonts half of upstream 8e1862e2 (text_language_test.cpp's
//! "new fallback fonts" sections), over pinned fixtures, plus the Nuxie
//! shape fallback sink (UNIV-5593).
#![cfg(feature = "testing")]
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    artboard_component_list::ArtboardComponentList,
    core::{CoreHandle, CoreType},
    generated::{
        core_registry::CoreRegistry,
        text::{text_input_base::TextInputBase, text_value_run_base::TextValueRunBase},
    },
    text::{
        font_hb::HbFont,
        text::{ShapeSource, Text},
        text_input::TextInput,
        text_value_run::TextValueRun,
    },
    text_engine::{self, FallbackProc, Font, FontRef},
};
use nuxie_runtime::{Artboard, File, RuntimeFactoryHandle, RuntimeFileHandle};
use std::{
    cell::RefCell,
    sync::{Arc, Mutex, MutexGuard},
};

// The fallback provider and its generations are process-wide.
static SERIAL: Mutex<()> = Mutex::new(());
fn serial() -> MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
fn asset(path: &str) -> Vec<u8> {
    let root =
        std::path::PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream"));
    std::fs::read(root.join("tests/unit_tests/assets").join(path)).unwrap()
}
fn import(path: &str) -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    File::import(&asset(path), factory, None, None, None).unwrap()
}
fn all<T: CoreType>(board: &CoreHandle) -> Vec<CoreHandle> {
    board
        .with_downcast::<Artboard, _>(|a| a.find_all_handles::<T>())
        .unwrap()
}
fn advance(board: &CoreHandle) {
    Artboard::advance_handle(
        board,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}
fn text_shapes(text: &CoreHandle) -> u64 {
    text.with_downcast::<Text, _>(|t| t.shape_count_for(ShapeSource::Update))
        .unwrap()
}
fn set_text(run: &CoreHandle, value: &str) {
    assert!(CoreRegistry::set_string_handle(
        run,
        TextValueRunBase::TEXT_PROPERTY_KEY.into(),
        value.into()
    ));
}

thread_local! { static FALLBACK: RefCell<Option<FontRef>> = const { RefCell::new(None) }; }
fn pick_fallback(_: u32, index: u32, _: &dyn Font) -> Option<FontRef> {
    if index == 0 {
        FALLBACK.with(|f| f.borrow().clone())
    } else {
        None
    }
}
struct RestoreFallback(Option<FallbackProc>);
impl Drop for RestoreFallback {
    fn drop(&mut self) {
        text_engine::set_fallback_proc(self.0);
        text_engine::set_fallback_proc_enabled(true);
        FALLBACK.with(|f| *f.borrow_mut() = None);
    }
}
fn use_fallback(font: Option<FontRef>) -> RestoreFallback {
    let old = text_engine::fallback_proc();
    FALLBACK.with(|f| *f.borrow_mut() = font);
    text_engine::set_fallback_proc(Some(pick_fallback));
    RestoreFallback(old)
}
fn arabic() -> FontRef {
    HbFont::decode(&asset("IBMPlexSansArabic-Regular.ttf")).unwrap()
}
fn draws_with(text: &CoreHandle, font: &FontRef) -> bool {
    text.with_downcast::<Text, _>(|t| {
        t.shape().iter().any(|p| {
            p.runs
                .iter()
                .any(|r| r.font.as_ref().is_some_and(|f| Arc::ptr_eq(f, font)))
        })
    })
    .unwrap()
}

/// A layout fixture's Text, set to Arabic its own font lacks.
fn arabic_text() -> (RuntimeFileHandle, CoreHandle, CoreHandle, CoreHandle) {
    let file = import("layout/layout_direction.riv");
    let board = file.with_file(|file| file.artboard()).unwrap();
    let text = board
        .with_downcast::<Artboard, _>(|a| a.find_handle::<Text>("SampleText"))
        .flatten()
        .unwrap();
    let run = all::<TextValueRun>(&board)[0].clone();
    set_text(&run, "مرحبا بالعالم");
    (file, board, text, run)
}

#[test]
fn new_fallback_fonts_reshape_text_on_the_next_advance() {
    let _serial = serial();
    let first = arabic();
    let _restore = use_fallback(Some(first.clone()));
    let (_file, board, text, _run) = arabic_text();
    advance(&board);
    assert!(draws_with(&text, &first));
    let shapes = text_shapes(&text);
    advance(&board);
    assert_eq!(
        text_shapes(&text),
        shapes,
        "nothing changed, nothing reshapes"
    );

    // A host fetched another face and says so: the text reshapes with no
    // change of its own.
    let second = arabic();
    FALLBACK.with(|f| *f.borrow_mut() = Some(second.clone()));
    text_engine::fallback_fonts_changed();
    advance(&board);
    assert!(text_shapes(&text) > shapes);
    assert!(draws_with(&text, &second));
}

#[test]
fn turning_fallbacks_off_and_on_does_not_reshape() {
    let _serial = serial();
    let _restore = use_fallback(Some(arabic()));
    let (_file, board, text, _run) = arabic_text();
    advance(&board);
    let shapes = text_shapes(&text);
    text_engine::set_fallback_proc_enabled(false);
    advance(&board);
    text_engine::set_fallback_proc_enabled(true);
    advance(&board);
    assert_eq!(text_shapes(&text), shapes);
}

#[test]
fn a_text_field_reshapes_for_new_fallback_fonts_on_the_next_advance() {
    let _serial = serial();
    let _restore = use_fallback(Some(arabic()));
    let file = import("text_input.riv");
    let artboard = file
        .with_file(|file| file.artboard_named("Text Input - Multiline"))
        .unwrap();
    let board = artboard.core_handle();
    let input = all::<TextInput>(&board)[0].clone();
    assert!(CoreRegistry::set_string_handle(
        &input,
        TextInputBase::TEXT_PROPERTY_KEY.into(),
        "مرحبا".into()
    ));
    let reports = record(|| advance(&board));
    assert!(
        reports.contains(&(input.identity_key(), true)),
        "{reports:?}"
    );
    let reports = record(|| advance(&board));
    assert!(
        !reports.iter().any(|(key, _)| *key == input.identity_key()),
        "an unchanged field does not shape again: {reports:?}"
    );
    text_engine::fallback_fonts_changed();
    let reports = record(|| {
        advance(&board);
        advance(&board);
    });
    assert!(
        reports.contains(&(input.identity_key(), true)),
        "{reports:?}"
    );
}

thread_local! {
    static REPORTS: RefCell<Vec<((usize, usize, u64), bool)>> = const { RefCell::new(Vec::new()) };
}
fn note(owner: &CoreHandle, used_fallback: bool) {
    REPORTS.with(|r| r.borrow_mut().push((owner.identity_key(), used_fallback)));
}
fn record(work: impl FnOnce()) -> Vec<((usize, usize, u64), bool)> {
    REPORTS.with(|r| r.borrow_mut().clear());
    text_engine::with_shape_fallback_sink(note, work);
    REPORTS.with(|r| std::mem::take(&mut *r.borrow_mut()))
}

#[test]
fn the_sink_hears_which_text_drew_with_the_fallback_fonts() {
    let _serial = serial();
    let _restore = use_fallback(Some(arabic()));
    let (_file, board, text, run) = arabic_text();
    let key = text.identity_key();
    assert_eq!(record(|| advance(&board)), vec![(key, true)]);
    assert!(
        record(|| advance(&board)).is_empty(),
        "a shape is reported once, when it is made"
    );

    set_text(&run, "Hello");
    assert_eq!(record(|| advance(&board)), vec![(key, false)]);

    // Outside the sink's scope nothing is told, and nothing is kept.
    set_text(&run, "مرحبا");
    advance(&board);
    assert!(record(|| advance(&board)).is_empty());
}

#[test]
fn the_sink_hears_text_whose_fallback_had_nothing_for_it() {
    // Asking the chain is what makes a text's look depend on it, whether or
    // not the fonts it holds now have the characters.
    let _serial = serial();
    let _restore = use_fallback(None);
    let (_file, board, text, _run) = arabic_text();
    assert_eq!(
        record(|| advance(&board)),
        vec![(text.identity_key(), true)]
    );
}

#[test]
fn quiet_list_rows_reshape_their_text_for_new_fallback_fonts() {
    let _serial = serial();
    let _restore = use_fallback(Some(arabic()));
    let file = import("artboard_list_overrides.riv");
    let artboard = file.with_file(|file| file.artboard_named("Main")).unwrap();
    let machine = artboard.state_machine_at(0).unwrap();
    let view_model_id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
    let instance = file.with_file(|file| {
        if view_model_id == u32::MAX {
            file.create_view_model_instance_for_artboard(artboard.core_handle())
        } else {
            file.create_view_model_instance_at(view_model_id as usize, 0)
        }
    });
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    for _ in 0..90 {
        machine.advance_and_apply(1.0 / 60.0);
    }
    let list = all::<ArtboardComponentList>(&artboard.core_handle())[0].clone();
    let row = list
        .with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(0))
        .flatten()
        .unwrap();
    assert_eq!(
        row.with_artboard(|row| row.quiet_host_row()),
        0,
        "the row went quiet"
    );
    let text = all::<Text>(&row.core_handle())
        .into_iter()
        .next()
        .expect("a row with text");
    let shapes = text_shapes(&text);
    assert!(shapes > 0);

    text_engine::fallback_fonts_changed();
    machine.advance_and_apply(1.0 / 60.0);
    machine.advance_and_apply(1.0 / 60.0);
    assert!(text_shapes(&text) > shapes);
}
