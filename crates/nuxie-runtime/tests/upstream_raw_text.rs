//! All three raw_text_test.cpp cases at db31ea87714cf6a4cba56dd5de030f941f7e35f8.
use nuxie_render_api::{NullFactory, PersistentFactory};
use nuxie_runtime::source::{
    factory::RuntimeFactoryHandle,
    text::{font_hb::HbFont, raw_text::RawText},
    text_engine::{FontRef, TextOrigin, TextOverflow, TextSizing, TextWordBreak, TextWrap},
};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

fn font() -> FontRef {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let bytes = std::fs::read(root.join("tests/unit_tests/assets/fonts/Inter_18pt-Regular.ttf"))
        .expect("read pinned Inter fixture");
    HbFont::decode(&bytes).expect("decode pinned Inter fixture")
}

fn factory() -> RuntimeFactoryHandle {
    let mut factory = PersistentFactory::new(NullFactory);
    RuntimeFactoryHandle::from_factory(&mut factory).expect("retained no-op factory")
}

#[test]
fn raw_text_keeps_one_style_per_paint_and_foreground_color() {
    let mut text = RawText::new(factory());
    let font = font();
    text.append("red", None, font.clone(), 16.0, -1.0, 0.0, 0xffff0000);
    text.append("blue", None, font.clone(), 16.0, -1.0, 0.0, 0xff0000ff);
    text.append("red again", None, font, 16.0, -1.0, 0.0, 0xffff0000);
    assert_eq!(text.style_count(), 2);
    assert_eq!(text.style_foreground_color(0), 0xffff0000);
    assert_eq!(text.style_foreground_color(1), 0xff0000ff);
}

#[test]
fn raw_text_clear_drops_the_styles_with_the_runs() {
    let factory = factory();
    let mut text = RawText::new(factory.clone());
    let font = font();
    for _ in 0..1000 {
        text.clear();
        let paint = factory.with_factory_mut(|factory| factory.make_render_paint());
        text.append(
            "frame",
            Some(Rc::new(RefCell::new(paint))),
            font.clone(),
            16.0,
            -1.0,
            0.0,
            0xff000000,
        );
        text.bounds();
    }
    assert_eq!(text.style_count(), 1);
}

#[test]
fn raw_text_ignores_layout_ordinals_out_of_range() {
    let mut text = RawText::new(factory());
    text.append("kept", None, font(), 16.0, -1.0, 0.0, 0xff000000);
    text.set_sizing_index(2);
    text.set_overflow_index(3);
    text.set_align_index(4);
    text.set_direction_index(2);
    text.set_max_width(40.0);
    text.set_max_height(10.0);
    let before = text.bounds();
    text.set_sizing_index(5);
    text.set_overflow_index(9);
    text.set_align_index(7);
    text.set_direction_index(-1);
    text.set_wrap_index(3);
    text.set_word_break_index(-2);
    text.set_origin_index(2);
    assert_eq!(text.sizing(), TextSizing::Fixed);
    assert_eq!(text.overflow(), TextOverflow::Ellipsis);
    assert_eq!(text.align_index(), 4);
    assert_eq!(text.direction_index(), 2);
    assert_eq!(text.wrap(), TextWrap::Wrap);
    assert_eq!(text.word_break(), TextWordBreak::BreakWord);
    assert_eq!(text.origin(), TextOrigin::Top);
    let after = text.bounds();
    assert_eq!(after.min_x, before.min_x);
    assert_eq!(after.max_y, before.max_y);
}
