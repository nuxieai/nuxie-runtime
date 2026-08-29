use std::{cell::RefCell, path::PathBuf, rc::Rc};

use nuxie::{PersistentFactory, RecordingFactory, RenderPaint, RenderPaintStyle};
use nuxie_runtime::source::{
    factory::RuntimeFactoryHandle,
    text::{font_hb::HbFont, raw_text::RawText},
    text_engine::{Font, FontRef, TextAlign, TextOverflow, TextSizing, with_host_fallback_proc},
};

type PaintHandle = Rc<RefCell<Box<dyn RenderPaint>>>;

fn upstream_asset(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    std::fs::read(root.join("tests/unit_tests/assets").join(name))
        .unwrap_or_else(|error| panic!("read upstream RawText asset {name}: {error}"))
}

fn font(name: &str) -> FontRef {
    HbFont::decode(&upstream_asset(name)).unwrap_or_else(|| panic!("{name} decodes"))
}

fn roboto() -> FontRef {
    font("RobotoFlex.ttf")
}

fn raw_text(factory: &mut PersistentFactory<RecordingFactory>) -> (RawText, RuntimeFactoryHandle) {
    let factory = RuntimeFactoryHandle::from_factory(factory).expect("retained recording factory");
    (RawText::new(factory.clone()), factory)
}

fn paint(factory: &RuntimeFactoryHandle) -> PaintHandle {
    Rc::new(RefCell::new(
        factory.with_factory_mut(|factory| factory.make_render_paint()),
    ))
}

fn append_default(raw: &mut RawText, text: &str, paint: Option<PaintHandle>, font: &FontRef) {
    raw.append(text, paint, font.clone(), 16.0, -1.0, 0.0, 0xff00_0000);
}

fn with_fallback<R>(font: FontRef, work: impl FnOnce() -> R) -> R {
    thread_local! {
        static FALLBACK: RefCell<Option<FontRef>> = const { RefCell::new(None) };
    }
    fn pick(_: u32, index: u32, _: &dyn Font) -> Option<FontRef> {
        if index > 0 {
            return None;
        }
        FALLBACK.with(|font| font.borrow().clone())
    }
    struct Restore(Option<FontRef>);
    impl Drop for Restore {
        fn drop(&mut self) {
            FALLBACK.with(|font| *font.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(FALLBACK.with(|slot| slot.replace(Some(font))));
    with_host_fallback_proc(pick, work)
}

#[test]
fn d_rt_api_defaults_empty_run_and_lazy_equality_noops() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let (mut raw, _) = raw_text(&mut factory);
    assert!(raw.empty());
    assert_eq!(raw.sizing(), TextSizing::AutoWidth);
    assert_eq!(raw.overflow(), TextOverflow::Visible);
    assert_eq!(raw.align(), TextAlign::Left);
    assert_eq!(raw.max_width(), 0.0);
    assert_eq!(raw.max_height(), 0.0);
    assert_eq!(raw.paragraph_spacing(), 0.0);
    assert!(!raw.debug_dirty());
    assert_eq!(raw.bounds(), nuxie::Aabb::new(0.0, 0.0, 0.0, 0.0));
    assert!(!raw.debug_dirty(), "initial RawText is clean");

    append_default(&mut raw, "", None, &roboto());
    assert!(!raw.empty(), "empty means no runs, not no characters");
    assert!(raw.debug_dirty());

    // The pinned source also asserts when shaping a zero-length run, so use
    // a populated RawText for the separate lazy-update/setter checks.
    drop(raw);
    let (mut raw, _) = raw_text(&mut factory);
    append_default(&mut raw, "A", None, &roboto());
    let _ = raw.bounds();
    assert!(!raw.debug_dirty());

    raw.set_sizing(TextSizing::AutoWidth);
    raw.set_overflow(TextOverflow::Visible);
    raw.set_align(TextAlign::Left);
    raw.set_max_width(0.0);
    raw.set_max_height(0.0);
    raw.set_paragraph_spacing(0.0);
    assert!(!raw.debug_dirty(), "equal setters do not dirty");

    raw.set_max_width(f32::NAN);
    assert!(raw.debug_dirty());
    let _ = raw.bounds();
    raw.set_max_width(f32::NAN);
    assert!(raw.debug_dirty(), "NaN compares unequal each time");
}

#[test]
fn d_rt_api_append_nul_style_identity_clear_and_stale_bounds() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let mut renderer = factory.borrow().make_renderer();
    let (mut raw, factory_handle) = raw_text(&mut factory);
    let font = roboto();
    let paint = paint(&factory_handle);
    paint.borrow_mut().style(RenderPaintStyle::Fill);
    paint.borrow_mut().color(0xff12_3456);
    // The pinned C++ literal constructs a std::string from a const char*, so
    // its first embedded NUL terminates the translated input.
    raw.append(
        "wide",
        Some(paint.clone()),
        font.clone(),
        24.0,
        -1.0,
        0.0,
        0xff00_0000,
    );
    raw.append("!", Some(paint), font, 12.0, 40.0, 3.0, 0xffff_0000);
    let before = raw.bounds();
    assert!(before.width() > 0.0 && before.height() > 0.0);
    raw.render(&mut renderer, None);

    raw.clear();
    assert!(raw.empty());
    assert_eq!(raw.bounds(), before, "C++ retains stale bounds after clear");
    raw.render(&mut renderer, None);
    drop(raw);
    assert_eq!(
        factory.borrow().stream().matches("drawPath").count(),
        1,
        "cleared draw commands stay empty"
    );
}

#[test]
fn d_rt_api_override_replaces_only_monochrome_paint_and_clipping_is_lazy() {
    let font = roboto();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let mut renderer = factory.borrow().make_renderer();
    let (mut raw, factory_handle) = raw_text(&mut factory);
    let authored = paint(&factory_handle);
    authored.borrow_mut().color(0xff11_2233);
    let override_paint = paint(&factory_handle);
    override_paint.borrow_mut().color(0xffaa_bbcc);
    append_default(&mut raw, "override", Some(authored), &font);
    raw.set_sizing(TextSizing::Fixed);
    raw.set_max_width(80.0);
    raw.set_max_height(30.0);
    raw.set_overflow(TextOverflow::Clipped);
    raw.render(&mut renderer, Some(override_paint));
    drop(raw);
    let clipped = factory.borrow().stream();
    assert!(clipped.contains("clipPath"));
    assert!(clipped.contains("ffaabbcc"));

    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let mut renderer = factory.borrow().make_renderer();
    let (mut raw, factory_handle) = raw_text(&mut factory);
    let authored = paint(&factory_handle);
    authored.borrow_mut().color(0xff11_2233);
    let override_paint = paint(&factory_handle);
    override_paint.borrow_mut().color(0xffaa_bbcc);
    append_default(&mut raw, "override", Some(authored), &font);
    raw.set_sizing(TextSizing::Fixed);
    raw.set_max_width(80.0);
    raw.set_max_height(30.0);
    raw.set_overflow(TextOverflow::Visible);
    raw.render(&mut renderer, Some(override_paint));
    drop(raw);
    let visible = factory.borrow().stream();
    assert!(!visible.contains("clipPath"));
}

#[test]
fn r_rt_owner_font_validation_and_color_cases_are_safe() {
    assert!(HbFont::decode(b"not a font").is_none());
    let emoji = font("TwemojiMozilla.subset.ttf");
    let regular = roboto();
    with_fallback(emoji.clone(), || {
        for (text, font, size, width) in [
            ("A", &emoji, 32.0, 200.0),
            ("❤❤❤", &emoji, 32.0, 400.0),
            ("Hello ❤ World", &regular, 32.0, 400.0),
            ("❤", &emoji, 1.0, 100.0),
            ("❤", &emoji, 200.0, 2000.0),
        ] {
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            let mut renderer = factory.borrow().make_renderer();
            let (mut raw, _) = raw_text(&mut factory);
            raw.set_max_width(width);
            raw.set_sizing(TextSizing::AutoHeight);
            raw.append(text, None, font.clone(), size, -1.0, 0.0, 0xff00_0000);
            raw.render(&mut renderer, None);
        }
    });
}
