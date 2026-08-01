use std::path::PathBuf;
use std::sync::Arc;

use nuxie::{
    RawText, RawTextFont, RecordingFactory, RenderPaintStyle, TextAlign, TextOverflow, TextSizing,
};

fn upstream_asset(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    std::fs::read(root.join("tests/unit_tests/assets").join(name))
        .unwrap_or_else(|error| panic!("read upstream RawText asset {name}: {error}"))
}

fn roboto() -> RawTextFont {
    RawTextFont::decode(Arc::<[u8]>::from(upstream_asset("RobotoFlex.ttf")))
        .expect("Roboto Flex decodes")
}

#[test]
fn d_rt_api_defaults_empty_run_and_lazy_equality_noops() {
    let mut factory = RecordingFactory::new();
    let mut raw = RawText::new(&mut factory);
    assert!(raw.empty());
    assert_eq!(raw.sizing(), TextSizing::AutoWidth);
    assert_eq!(raw.overflow(), TextOverflow::Visible);
    assert_eq!(raw.align(), TextAlign::Left);
    assert_eq!(raw.max_width(), 0.0);
    assert_eq!(raw.max_height(), 0.0);
    assert_eq!(raw.paragraph_spacing(), 0.0);
    assert_eq!(raw.debug_update_count(), 0);
    assert_eq!(raw.bounds(), nuxie::Aabb::new(0.0, 0.0, 0.0, 0.0));
    assert_eq!(raw.debug_update_count(), 0, "initial RawText is clean");

    raw.append_default("", None, &roboto());
    assert!(!raw.empty(), "empty means no runs, not no characters");
    let _ = raw.bounds();
    assert_eq!(raw.debug_update_count(), 1);

    raw.set_sizing(TextSizing::AutoWidth);
    raw.set_overflow(TextOverflow::Visible);
    raw.set_align(TextAlign::Left);
    raw.set_max_width(0.0);
    raw.set_max_height(0.0);
    raw.set_paragraph_spacing(0.0);
    let _ = raw.bounds();
    assert_eq!(raw.debug_update_count(), 1, "equal setters do not dirty");

    raw.set_max_width(f32::NAN);
    let _ = raw.bounds();
    raw.set_max_width(f32::NAN);
    let _ = raw.bounds();
    assert_eq!(
        raw.debug_update_count(),
        3,
        "NaN compares unequal each time"
    );
}

#[test]
fn d_rt_api_append_nul_style_identity_clear_and_stale_bounds() {
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    let mut raw = RawText::new(&mut factory);
    let font = roboto();
    let paint = raw.make_paint();
    paint.with_paint(|paint| {
        paint.style(RenderPaintStyle::Fill);
        paint.color(0xff12_3456);
    });
    raw.append(
        "wide\0ignored",
        Some(paint.clone()),
        &font,
        24.0,
        -1.0,
        0.0,
        0xff00_0000,
    );
    raw.append("!", Some(paint), &font, 12.0, 40.0, 3.0, 0xffff_0000);
    let before = raw.bounds();
    assert!(before.width() > 0.0 && before.height() > 0.0);
    raw.render(&mut renderer, None);

    raw.clear();
    assert!(raw.empty());
    assert_eq!(raw.bounds(), before, "C++ retains stale bounds after clear");
    raw.render(&mut renderer, None);
    drop(raw);
    assert_eq!(
        factory.stream().matches("drawPath").count(),
        1,
        "cleared draw commands stay empty"
    );
}

#[test]
fn d_rt_api_override_replaces_only_monochrome_paint_and_clipping_is_lazy() {
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    let mut raw = RawText::new(&mut factory);
    let font = roboto();
    let authored = raw.make_paint();
    authored.with_paint(|paint| paint.color(0xff11_2233));
    let override_paint = raw.make_paint();
    override_paint.with_paint(|paint| paint.color(0xffaa_bbcc));
    raw.append_default("override", Some(authored), &font);
    raw.set_sizing(TextSizing::Fixed);
    raw.set_max_width(80.0);
    raw.set_max_height(30.0);
    raw.set_overflow(TextOverflow::Clipped);
    raw.render(&mut renderer, Some(&override_paint));
    drop(raw);
    let clipped = factory.stream();
    assert!(clipped.contains("clipPath"));
    assert!(clipped.contains("ffaabbcc"));

    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    let mut raw = RawText::new(&mut factory);
    let authored = raw.make_paint();
    authored.with_paint(|paint| paint.color(0xff11_2233));
    let override_paint = raw.make_paint();
    override_paint.with_paint(|paint| paint.color(0xffaa_bbcc));
    raw.append_default("override", Some(authored), &font);
    raw.set_sizing(TextSizing::Fixed);
    raw.set_max_width(80.0);
    raw.set_max_height(30.0);
    raw.set_overflow(TextOverflow::Visible);
    raw.render(&mut renderer, Some(&override_paint));
    drop(raw);
    let visible = factory.stream();
    assert!(!visible.contains("clipPath"));
}

#[test]
fn r_rt_owner_font_validation_and_color_cases_are_safe() {
    assert!(RawTextFont::decode(Arc::<[u8]>::from(&b"not a font"[..])).is_err());
    let emoji = RawTextFont::decode(Arc::<[u8]>::from(upstream_asset(
        "TwemojiMozilla.subset.ttf",
    )))
    .expect("Twemoji decodes");
    let regular = roboto().with_fallbacks([emoji.clone()]);
    for (text, font, size, width) in [
        ("A", &emoji, 32.0, 200.0),
        ("❤❤❤", &emoji, 32.0, 400.0),
        ("Hello ❤ World", &regular, 32.0, 400.0),
        ("❤", &emoji, 1.0, 100.0),
        ("❤", &emoji, 200.0, 2000.0),
    ] {
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let mut raw = RawText::new(&mut factory);
        raw.set_max_width(width);
        raw.set_sizing(TextSizing::AutoHeight);
        raw.append(text, None, font, size, -1.0, 0.0, 0xff00_0000);
        raw.render(&mut renderer, None);
    }
}
