//! Literal public text_shape_memo_test.cpp and text_test.cpp additions at 95219cad.
//! Relative geometry assertions preserve the CSS/Taffy boundary.
#![cfg(feature = "testing")]
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    core::{CoreHandle, CoreType},
    generated::{
        core_registry::CoreRegistry,
        layout::{
            layout_component_style_base::LayoutComponentStyleBase,
            layout_sizing_style_base::LayoutSizingStyleBase,
        },
        text::{text_base::TextBase, text_value_run_base::TextValueRunBase},
    },
    layout::{
        layout_component_style::LayoutComponentStyle, layout_measure_mode::LayoutMeasureMode,
    },
    layout_component::LayoutComponent,
    lua::scripting_vm::RuntimeScriptingVmHandle,
    math::{aabb::Aabb, vec2d::Vec2D},
    text::{
        font_hb::HbFont,
        text::{ShapeSource, Text},
        text_value_run::TextValueRun,
    },
    text_engine::{self, FallbackProc, Font, FontRef, TextAlign},
};
use nuxie_runtime::{Artboard, File, RuntimeFactoryHandle, RuntimeFileHandle};
use nuxie_scripting::vm::{ScriptExecutionLimits, ScriptVm};
use std::{
    cell::RefCell,
    sync::{Arc, Mutex, MutexGuard},
};
// Upstream's fallback globals and aggregate shape counter are serial. Keep this
// fixture group isolated while translating its global-provider assertions.
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
fn fixture(path: &str) -> (RuntimeFileHandle, CoreHandle) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    // ReadRiveFile registers scripts; retain the same setup for every fixture.
    let vm = RuntimeScriptingVmHandle::new(Box::new(
        ScriptVm::new_with_execution_limits(ScriptExecutionLimits::default()).unwrap(),
    ));
    let file = File::import(&asset(path), factory, None, None, Some(vm)).unwrap();
    let artboard = file.with_file(|file| file.artboard()).unwrap();
    (file, artboard)
}
fn all<T: CoreType>(board: &CoreHandle) -> Vec<CoreHandle> {
    board
        .with_downcast::<Artboard, _>(|a| a.find_all_handles::<T>())
        .unwrap()
}
fn named<T: CoreType>(board: &CoreHandle, name: &str) -> CoreHandle {
    board
        .with_downcast::<Artboard, _>(|a| a.find_handle::<T>(name))
        .flatten()
        .unwrap()
}
fn advance(board: &CoreHandle, seconds: f32) {
    Artboard::advance_handle(
        board,
        seconds,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}
fn read<R>(text: &CoreHandle, f: impl FnOnce(&Text) -> R) -> R {
    text.with_downcast::<Text, _>(f).unwrap()
}
fn edit<R>(text: &CoreHandle, f: impl FnOnce(&mut Text) -> R) -> R {
    text.with_downcast_mut::<Text, _>(f).unwrap()
}
fn uint(owner: &CoreHandle, key: u16, value: u32) {
    assert!(CoreRegistry::set_uint_handle(owner, key.into(), value));
}
fn scalar(owner: &CoreHandle, key: u16, value: f32) {
    assert!(CoreRegistry::set_double_handle(owner, key.into(), value));
}
fn set_text(run: &CoreHandle, value: String) {
    assert!(CoreRegistry::set_string_handle(
        run,
        TextValueRunBase::TEXT_PROPERTY_KEY.into(),
        value
    ));
}
fn append(run: &CoreHandle, suffix: &str) {
    let value = run
        .with_downcast::<TextValueRun, _>(|r| format!("{}{suffix}", r.base.text()))
        .unwrap();
    set_text(run, value);
}
fn parent(text: &CoreHandle) -> CoreHandle {
    text.with(|o| o.as_component().unwrap().parent_handle())
        .flatten()
        .unwrap()
}
fn style(layout: &CoreHandle) -> CoreHandle {
    layout
        .with_downcast::<LayoutComponent, _>(LayoutComponent::style_handle)
        .flatten()
        .unwrap()
}
fn slot(text: &CoreHandle, size: Option<Vec2D>) {
    let layout = parent(text);
    let (width, height, direction) = layout
        .with_downcast::<LayoutComponent, _>(|l| {
            (l.layout_width(), l.layout_height(), l.actual_direction())
        })
        .unwrap();
    let (w, h) = style(&layout)
        .with_downcast::<LayoutComponentStyle, _>(|s| (s.width_scale_type(), s.height_scale_type()))
        .unwrap();
    edit(text, |t| {
        t.control_size(size.unwrap_or(Vec2D::new(width, height)), w, h, direction)
    });
}
fn lines(text: &CoreHandle) -> Vec<(f32, Vec<u16>)> {
    read(text, |t| {
        t.ordered_lines()
            .iter()
            .map(|line| {
                (
                    line.y(),
                    line.into_iter()
                        .map(|(run, i)| run.glyphs[i as usize])
                        .collect(),
                )
            })
            .collect()
    })
}
fn bounds(text: &CoreHandle) -> Aabb {
    read(text, Text::local_bounds)
}
fn shapes(text: &CoreHandle) -> u64 {
    read(text, Text::shape_count)
}
fn first_hug_text(board: &CoreHandle) -> CoreHandle {
    all::<Text>(board)
        .into_iter()
        .find(|t| {
            let p = parent(t);
            p.is_type_of(LayoutComponent::TYPE_KEY) && !p.is_type_of(Artboard::TYPE_KEY)
        })
        .unwrap()
}
#[test]
fn zero_slot_then_original_slot_shapes_and_breaks_nothing() {
    let _serial = serial();
    let (_file, board) = fixture("layout/layout_direction.riv");
    let text = named::<Text>(&board, "SampleText");
    advance(&board, 0.0);
    advance(&board, 0.0);
    let before = (
        lines(&text),
        bounds(&text),
        text_engine::shape_count(),
        read(&text, Text::line_break_count),
    );
    slot(&text, Some(Vec2D::default()));
    slot(&text, None);
    advance(&board, 0.0);
    assert_eq!(text_engine::shape_count(), before.2);
    assert_eq!(read(&text, Text::line_break_count), before.3);
    assert_eq!(lines(&text), before.0);
    assert_eq!(bounds(&text), before.1);
}
#[test]
fn direction_rebreaks_without_reshaping() {
    let _serial = serial();
    let (_file, board) = fixture("layout/layout_direction.riv");
    let text = named::<Text>(&board, "SampleText");
    advance(&board, 0.0);
    assert_eq!(read(&text, Text::align), TextAlign::Right);
    let count = shapes(&text);
    uint(
        &style(&named::<LayoutComponent>(&board, "TextContainer")),
        LayoutComponentStyleBase::DIRECTION_VALUE_PROPERTY_KEY,
        1,
    );
    advance(&board, 0.0);
    assert_eq!(read(&text, Text::align), TextAlign::Left);
    assert_eq!(shapes(&text), count);
    let (_fresh_file, fresh_board) = fixture("layout/layout_direction.riv");
    uint(
        &style(&named::<LayoutComponent>(&fresh_board, "TextContainer")),
        LayoutComponentStyleBase::DIRECTION_VALUE_PROPERTY_KEY,
        1,
    );
    let fresh = named::<Text>(&fresh_board, "SampleText");
    advance(&fresh_board, 0.0);
    assert_eq!(lines(&text), lines(&fresh));
    assert_eq!(bounds(&text), bounds(&fresh));
}
#[test]
fn unchanged_dirt_shapes_and_breaks_nothing() {
    let _serial = serial();
    let (_file, board) = fixture("ellipsis.riv");
    let text = all::<Text>(&board)[0].clone();
    advance(&board, 0.0);
    let before = (
        lines(&text),
        shapes(&text),
        read(&text, Text::line_break_count),
    );
    edit(&text, Text::mark_shape_dirty);
    advance(&board, 0.0);
    assert_eq!(shapes(&text), before.1);
    assert_eq!(read(&text, Text::line_break_count), before.2);
    assert_eq!(lines(&text), before.0);
    append(&all::<TextValueRun>(&board)[0], " more");
    advance(&board, 0.0);
    assert_eq!(shapes(&text), before.1 + 1);
}
#[test]
fn reshowing_hug_text_measures_without_shaping() {
    let _serial = serial();
    let (_file, board) = fixture("layout/text_layout_7_3.riv");
    let text = first_hug_text(&board);
    let style = style(&parent(&text));
    advance(&board, 0.0);
    advance(&board, 0.0);
    let before = (bounds(&text), text_engine::shape_count());
    uint(&style, LayoutSizingStyleBase::DISPLAY_VALUE_PROPERTY_KEY, 1);
    advance(&board, 0.0);
    advance(&board, 0.0);
    uint(&style, LayoutSizingStyleBase::DISPLAY_VALUE_PROPERTY_KEY, 0);
    advance(&board, 0.0);
    advance(&board, 0.0);
    assert_eq!(text_engine::shape_count(), before.1);
    assert_eq!(bounds(&text), before.0);
}
#[test]
fn fitted_text_uses_the_winning_shape() {
    let _serial = serial();
    let (_file, board) = fixture("ellipsis.riv");
    let text = all::<Text>(&board)[0].clone();
    uint(&text, TextBase::OVERFLOW_VALUE_PROPERTY_KEY, 0);
    advance(&board, 0.0);
    let (width, height, authored) = read(&text, |t| {
        (t.width(), t.height(), t.shape()[0].runs[0].size)
    });
    let updates = read(&text, |t| t.shape_count_for(ShapeSource::Update));
    let fits = read(&text, |t| t.shape_count_for(ShapeSource::Fit));
    uint(&text, TextBase::OVERFLOW_VALUE_PROPERTY_KEY, 5);
    advance(&board, 0.0);
    assert!(read(&text, |t| t.shape()[0].runs[0].size) < authored);
    assert!(read(&text, |t| t.shape_count_for(ShapeSource::Fit)) > fits);
    assert_eq!(
        read(&text, |t| t.shape_count_for(ShapeSource::Update)),
        updates
    );
    scalar(&text, TextBase::WIDTH_PROPERTY_KEY, width * 4.0);
    scalar(&text, TextBase::HEIGHT_PROPERTY_KEY, height * 4.0);
    let fits = read(&text, |t| t.shape_count_for(ShapeSource::Fit));
    advance(&board, 0.0);
    assert_eq!(read(&text, |t| t.shape()[0].runs[0].size), authored);
    assert_eq!(
        read(&text, |t| t.shape_count_for(ShapeSource::Fit)),
        fits + 1
    );
    assert_eq!(
        read(&text, |t| t.shape_count_for(ShapeSource::Update)),
        updates
    );
    let count = shapes(&text);
    scalar(&text, TextBase::WIDTH_PROPERTY_KEY, width * 5.0);
    advance(&board, 0.0);
    assert_eq!(shapes(&text), count);
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
        FALLBACK.with(|f| *f.borrow_mut() = None);
    }
}
fn reset_fallback() -> RestoreFallback {
    let old = text_engine::fallback_proc();
    text_engine::set_fallback_proc(None);
    RestoreFallback(old)
}
fn arabic() -> FontRef {
    HbFont::decode(&asset("IBMPlexSansArabic-Regular.ttf")).unwrap()
}
fn draws_with(text: &CoreHandle, font: &FontRef) -> bool {
    read(text, |t| {
        t.shape().iter().any(|p| {
            p.runs
                .iter()
                .any(|r| r.font.as_ref().is_some_and(|f| Arc::ptr_eq(f, font)))
        })
    })
}
#[test]
fn fallback_provider_and_font_changes_reshape() {
    let _serial = serial();
    let _restore = reset_fallback();
    let (_file, board) = fixture("layout/layout_direction.riv");
    let text = named::<Text>(&board, "SampleText");
    set_text(&all::<TextValueRun>(&board)[0], "مرحبا بالعالم".into());
    advance(&board, 0.0);
    let font = arabic();
    assert!(!draws_with(&text, &font));
    FALLBACK.with(|f| *f.borrow_mut() = Some(font.clone()));
    text_engine::set_fallback_proc(Some(pick_fallback));
    let count = shapes(&text);
    edit(&text, Text::mark_shape_dirty);
    advance(&board, 0.0);
    assert_eq!(shapes(&text), count + 1);
    assert!(draws_with(&text, &font));
    let other = arabic();
    FALLBACK.with(|f| *f.borrow_mut() = Some(other.clone()));
    text_engine::fallback_fonts_changed();
    let count = shapes(&text);
    edit(&text, Text::mark_shape_dirty);
    advance(&board, 0.0);
    assert_eq!(shapes(&text), count + 1);
    assert!(draws_with(&text, &other));
    // Preserve Nuxie's conservative host boundary even without notification.
    let unreported = arabic();
    FALLBACK.with(|f| *f.borrow_mut() = Some(unreported.clone()));
    edit(&text, Text::mark_shape_dirty);
    advance(&board, 0.0);
    assert!(draws_with(&text, &unreported));
}
#[test]
fn size_change_after_fallback_change_reshapes() {
    let _serial = serial();
    let _restore = reset_fallback();
    let (_file, board) = fixture("layout/layout_direction.riv");
    let text = named::<Text>(&board, "SampleText");
    set_text(&all::<TextValueRun>(&board)[0], "مرحبا بالعالم".into());
    advance(&board, 0.0);
    let font = arabic();
    assert!(!draws_with(&text, &font));
    FALLBACK.with(|f| *f.borrow_mut() = Some(font.clone()));
    text_engine::set_fallback_proc(Some(pick_fallback));
    let count = shapes(&text);
    let size = parent(&text)
        .with_downcast::<LayoutComponent, _>(|l| {
            Vec2D::new(l.layout_width() / 2.0, l.layout_height())
        })
        .unwrap();
    slot(&text, Some(size));
    advance(&board, 0.0);
    assert_eq!(shapes(&text), count + 1);
    assert!(draws_with(&text, &font));
}
#[test]
fn width_change_rebreaks_without_reshaping() {
    let _serial = serial();
    let (_file, board) = fixture("ellipsis.riv");
    let text = all::<Text>(&board)[0].clone();
    uint(&text, TextBase::OVERFLOW_VALUE_PROPERTY_KEY, 0);
    advance(&board, 0.0);
    assert_eq!(read(&text, |t| t.ordered_lines().len()), 2);
    let count = shapes(&text);
    let width = read(&text, |t| t.width());
    scalar(&text, TextBase::WIDTH_PROPERTY_KEY, width * 3.0);
    advance(&board, 0.0);
    assert_eq!(shapes(&text), count);
    assert_eq!(read(&text, |t| t.ordered_lines().len()), 1);
    let (_fresh_file, fresh_board) = fixture("ellipsis.riv");
    let fresh = all::<Text>(&fresh_board)[0].clone();
    uint(&fresh, TextBase::OVERFLOW_VALUE_PROPERTY_KEY, 0);
    scalar(&fresh, TextBase::WIDTH_PROPERTY_KEY, width * 3.0);
    advance(&fresh_board, 0.0);
    assert_eq!(lines(&text), lines(&fresh));
    assert_eq!(bounds(&text), bounds(&fresh));
    scalar(&text, TextBase::WIDTH_PROPERTY_KEY, width);
    advance(&board, 0.0);
    assert_eq!(shapes(&text), count);
    assert_eq!(read(&text, |t| t.ordered_lines().len()), 2);
}
#[test]
fn hug_layout_text_change_shapes_once() {
    let _serial = serial();
    let (_file, board) = fixture("layout/text_layout_7_3.riv");
    advance(&board, 0.0);
    let run = all::<TextValueRun>(&board)
        .into_iter()
        .find(|r| {
            let t = r
                .with_downcast::<TextValueRun, _>(TextValueRun::text_component)
                .flatten()
                .unwrap();
            let p = parent(&t);
            p.is_type_of(LayoutComponent::TYPE_KEY) && !p.is_type_of(Artboard::TYPE_KEY)
        })
        .unwrap();
    let text = run
        .with_downcast::<TextValueRun, _>(TextValueRun::text_component)
        .flatten()
        .unwrap();
    advance(&board, 0.0);
    let count = shapes(&text);
    append(&run, " and then some more words");
    advance(&board, 0.016);
    assert_eq!(shapes(&text), count + 1);
    let count = shapes(&text);
    edit(&text, |t| {
        t.measure_layout(
            200.0,
            LayoutMeasureMode::AtMost,
            200.0,
            LayoutMeasureMode::AtMost,
        )
    });
    assert_eq!(shapes(&text), count);
}
