//! Literal text_ellipsis_direction_test.cpp at upstream 0a9f1641.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    core::CoreHandle,
    generated::core_registry::CoreRegistry,
    layout_component::LayoutComponent,
    text::{
        cursor::{Cursor, CursorPosition},
        font_hb::HbFont,
        raw_text::RawText,
        raw_text_input::RawTextInput,
        text::Text,
        text_value_run::TextValueRun,
    },
    text_engine::{
        Font, FontRef, GlyphRun, OrderedLine, Paragraph, TextAlign, TextDirection, TextOverflow,
        TextRun, TextSizing, TextWordBreak, TextWrap, with_host_fallback_proc,
    },
};
use nuxie_runtime::{Artboard, File, RuntimeFactoryHandle, RuntimeFileHandle};
use std::{cell::RefCell, path::PathBuf};
fn asset(name: &str) -> Vec<u8> {
    let root = PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream"));
    std::fs::read(root.join("tests/unit_tests/assets").join(name)).unwrap()
}
fn font() -> FontRef {
    HbFont::decode(&asset("IBMPlexSansArabic-Regular.ttf")).unwrap()
}
fn factory() -> RuntimeFactoryHandle {
    RuntimeFactoryHandle::from_factory(&mut PersistentFactory::new(RecordingFactory::new()))
        .unwrap()
}
fn approx(a: f32, b: f32) {
    assert!(
        (f64::from(a) - f64::from(b)).abs() <= 100.0 * f64::from(f32::EPSILON) * f64::from(b).abs(),
        "{a} != {b}"
    );
}
fn ellipsize(text: &str, align: TextAlign) -> (Vec<Paragraph>, GlyphRun, OrderedLine) {
    let font = font();
    let chars: Vec<u32> = text.chars().map(u32::from).collect();
    let runs = [TextRun {
        font: Some(font.clone()),
        size: 20.0,
        line_height: -1.0,
        letter_spacing: 0.0,
        unichar_count: chars.len() as u32,
        script: 0,
        style_id: 0,
        level: 0,
    }];
    let shape = font.shape_text(&chars, &runs, -1);
    let lines = Text::break_lines(
        &shape,
        120.0,
        align,
        TextWrap::NoWrap,
        TextWordBreak::BreakWord,
    );
    assert_eq!(shape.len(), 1);
    assert_eq!(lines[0].len(), 1);
    let mut ellipsis = GlyphRun::default();
    let line = OrderedLine::new(
        &shape[0],
        &lines[0][0],
        120.0,
        align,
        true,
        true,
        &mut ellipsis,
        0.0,
    );
    (shape, ellipsis, line)
}
fn span(line: &OrderedLine) -> (f32, f32) {
    let start = line.start_x();
    let mut end = start;
    for (run, index) in line {
        end += run.advances[index as usize];
    }
    (start, end)
}
// OrderedLine owns clones in Rust. Compare the complete ellipsis glyph sequence
// instead of the C++ test's pointer into its separately stored ellipsis run.
fn same_run(a: &GlyphRun, b: &GlyphRun) {
    assert_eq!(a.glyphs, b.glyphs);
    assert_eq!(a.text_indices, b.text_indices);
    assert_eq!(a.advances, b.advances);
    assert_eq!(a.level, b.level);
}
#[test]
fn ellipsis_sits_at_the_end_edge_of_a_right_to_left_line() {
    let (shape, ellipsis, line) = ellipsize("مرحبا بالعالم هذا نص طويل جدا", TextAlign::Left);
    assert_eq!(shape[0].base_direction(), TextDirection::Rtl);
    assert_eq!(line.runs().len(), 2);
    same_run(line.runs().next().unwrap(), &ellipsis);
    assert_eq!(ellipsis.dir(), TextDirection::Rtl);
    let (run, index) = line.into_iter().last().unwrap();
    assert_eq!(run.text_indices[index as usize], 0);
}
#[test]
fn ellipsis_sits_at_the_end_edge_of_a_left_to_right_line() {
    let (_, ellipsis, line) = ellipsize("Hello world this is long", TextAlign::Left);
    same_run(line.runs().last().unwrap(), &ellipsis);
    assert_eq!(ellipsis.dir(), TextDirection::Ltr);
}
#[test]
fn ellipsis_follows_the_paragraph_not_the_run_it_cuts() {
    for (text, align, dir) in [
        (
            "مرحبا Hello World Again",
            TextAlign::Right,
            TextDirection::Rtl,
        ),
        (
            "Hello مرحبا بالعالم هذا",
            TextAlign::Left,
            TextDirection::Ltr,
        ),
    ] {
        let (shape, ellipsis, line) = ellipsize(text, align);
        assert_eq!(shape[0].base_direction(), dir);
        same_run(
            if dir == TextDirection::Rtl {
                line.runs().next().unwrap()
            } else {
                line.runs().last().unwrap()
            },
            &ellipsis,
        );
    }
}
#[test]
fn an_ellipsized_line_is_aligned_by_its_ellipsized_width() {
    for align in [TextAlign::Right, TextAlign::Center, TextAlign::Left] {
        let (_, _, line) = ellipsize("مرحبا بالعالم هذا نص طويل جدا", align);
        let (start, end) = span(&line);
        assert!(start >= 0.0);
        match align {
            TextAlign::Right => {
                assert!(line.glyph_line().start_x < 0.0);
                approx(end, 120.0)
            }
            TextAlign::Center => approx(start, 120.0 - end),
            _ => {
                assert_eq!(start, 0.0);
                assert!(end <= 120.0)
            }
        }
    }
}
thread_local! { static FALLBACK: RefCell<Option<FontRef>> = const { RefCell::new(None) }; }
fn with_arabic<R>(work: impl FnOnce() -> R) -> R {
    fn pick(_: u32, index: u32, _: &dyn Font) -> Option<FontRef> {
        if index == 0 {
            FALLBACK.with(|f| f.borrow().clone())
        } else {
            None
        }
    }
    struct Restore(Option<FontRef>);
    impl Drop for Restore {
        fn drop(&mut self) {
            FALLBACK.with(|f| *f.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(FALLBACK.with(|f| f.replace(Some(font()))));
    with_host_fallback_proc(pick, work)
}
fn fixture(name: &str) -> (RuntimeFileHandle, CoreHandle, CoreHandle) {
    let file = File::import(&asset(name), factory(), None, None, None).unwrap();
    let board = file.with_file(File::artboard).unwrap();
    let text = board
        .with_downcast::<Artboard, _>(|b| b.find_all_handles::<Text>()[0].clone())
        .unwrap();
    (file, board, text)
}
fn integer(h: &CoreHandle, key: i32, value: u32) {
    assert!(CoreRegistry::set_uint_handle(h, key, value));
}
fn content(board: &CoreHandle, value: &str) {
    let run = board
        .with_downcast::<Artboard, _>(|b| b.find_all_handles::<TextValueRun>()[0].clone())
        .unwrap();
    assert!(CoreRegistry::set_string_handle(&run, 268, value.to_owned()));
}
fn advance(board: &CoreHandle) {
    Artboard::advance_handle(
        board,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}
fn check_ellipsis_edge(text: &Text, rtl: bool) {
    let line = &text.ordered_lines()[0];
    let run = if rtl {
        line.runs().next().unwrap()
    } else {
        line.runs().last().unwrap()
    };
    let font = run.font.as_ref().unwrap();
    let dots = font.shape_text(
        &[46, 46, 46],
        &[TextRun {
            font: Some(font.clone()),
            size: run.size,
            line_height: run.line_height,
            letter_spacing: run.letter_spacing,
            unichar_count: 3,
            script: 0,
            style_id: run.style_id,
            level: 0,
        }],
        i32::from(rtl),
    );
    same_run(run, &dots[0].runs[0]);
}
#[test]
fn an_ellipsized_text_stays_inside_its_box() {
    with_arabic(|| {
        for (rtl, align, wrap) in [
            (false, 0, 1),
            (false, 1, 1),
            (false, 2, 1),
            (true, 0, 1),
            (true, 1, 1),
            (true, 1, 0),
        ] {
            let (_file, board, text) = fixture("ellipsis.riv");
            content(
                &board,
                if rtl {
                    "مرحبا بالعالم هذا نص طويل جدا جدا"
                } else {
                    "one two three four five six seven"
                },
            );
            integer(&text, 281, align);
            integer(&text, 683, wrap);
            advance(&board);
            text.with_downcast::<Text, _>(|t| {
                assert_eq!(t.ordered_lines().len(), 1);
                let (start, end) = span(&t.ordered_lines()[0]);
                assert!(start >= 0.0);
                assert!(end <= t.width() + 0.001);
                if align == 1 {
                    approx(end, t.width());
                }
                check_ellipsis_edge(t, rtl);
            })
            .unwrap();
        }
    });
}
#[test]
fn an_ellipsized_text_in_a_layout_aligns_to_the_layouts_direction() {
    with_arabic(|| {
        for rtl_layout in [false, true] {
            for rtl in [false, true] {
                let (_file, board, _) = fixture("layout/layout_direction.riv");
                let (container, text) = board
                    .with_downcast::<Artboard, _>(|b| {
                        (
                            b.find_handle::<LayoutComponent>("TextContainer").unwrap(),
                            b.find_handle::<Text>("SampleText").unwrap(),
                        )
                    })
                    .unwrap();
                let style = container
                    .with_downcast::<LayoutComponent, _>(LayoutComponent::style_handle)
                    .unwrap()
                    .unwrap();
                integer(&style, 599, if rtl_layout { 2 } else { 1 });
                content(
                    &board,
                    if rtl {
                        "مرحبا بالعالم هذا نص طويل جدا جدا جدا جدا جدا"
                    } else {
                        "one two three four five six seven eight nine ten"
                    },
                );
                integer(&text, 287, 3);
                integer(&text, 683, 1);
                advance(&board);
                text.with_downcast::<Text, _>(|t| {
                    assert_eq!(
                        t.align(),
                        if rtl_layout {
                            TextAlign::Right
                        } else {
                            TextAlign::Left
                        }
                    );
                    assert_eq!(t.ordered_lines().len(), 1);
                    check_ellipsis_edge(t, rtl);
                    let (start, end) = span(&t.ordered_lines()[0]);
                    let width = t.effective_width();
                    assert!(start >= 0.0);
                    assert!(end <= width + 0.001);
                    if rtl_layout {
                        approx(end, width)
                    } else {
                        assert_eq!(start, 0.0)
                    }
                })
                .unwrap();
            }
        }
    });
}
#[test]
fn an_ellipsized_line_reports_where_it_draws() {
    let content = "one two three four five six seven eight nine ten";
    let factory = factory();
    let mut text = RawText::new(factory.clone());
    text.append(content, None, font(), 20.0, -1.0, 0.0, 0xff000000);
    text.set_sizing(TextSizing::Fixed);
    text.set_overflow(TextOverflow::Ellipsis);
    text.set_wrap(TextWrap::NoWrap);
    text.set_align(TextAlign::Right);
    text.set_max_width(120.0);
    text.set_max_height(100.0);
    assert_eq!(text.line_count(), 1);
    let (mut first, mut start, mut end) = (true, 0.0, 0.0);
    text.for_each_glyph(0, |run, index, at| {
        let index = index as usize;
        let pen = at.x - run.offsets[index].x;
        if first {
            start = pen;
            first = false;
        }
        end = pen + run.advances[index];
        true
    });
    approx(start, text.ordered_lines()[0].start_x());
    assert!(start >= 0.0);
    approx(end, 120.0);
    let mut input = RawTextInput::new();
    input.set_font(Some(font()));
    input.set_font_size(20.0);
    input.set_sizing(TextSizing::Fixed);
    input.set_overflow(TextOverflow::Ellipsis);
    input.set_align(TextAlign::Right);
    input.set_max_width(120.0);
    input.set_max_height(30.0);
    input.set_text(content.to_owned());
    input.update(&factory);
    let line = &input.shape().ordered_lines()[0];
    let start = line.start_x();
    assert!(
        (start - line.glyph_line().start_x).abs()
            > 100.0 * f32::EPSILON * line.glyph_line().start_x.abs()
    );
    approx(
        input
            .cursor_visual_position_at(CursorPosition::new(0, 0))
            .x(),
        start,
    );
    let mut rects = Vec::new();
    Cursor::new(CursorPosition::new(0, 0), CursorPosition::new(0, 3))
        .selection_rects(&mut rects, &input.shape().layout_view());
    assert!(!rects.is_empty());
    approx(rects[0].min_x, start);
}

// font_test.cpp additions at 689e214a: changing either field reshapes ellipsis.
fn ellipsis_for_runs(
    first: FontRef,
    first_size: f32,
    second: FontRef,
    second_size: f32,
) -> GlyphRun {
    let chars: Vec<u32> = "ab cdefghijklmnopqrstuvwxyz"
        .chars()
        .map(u32::from)
        .collect();
    let run = |font, size, count| TextRun {
        font: Some(font),
        size,
        line_height: -1.0,
        letter_spacing: 0.0,
        unichar_count: count,
        script: 0,
        style_id: 0,
        level: 0,
    };
    let shape = first.shape_text(
        &chars,
        &[
            run(first.clone(), first_size, 3),
            run(second, second_size, (chars.len() - 3) as u32),
        ],
        -1,
    );
    assert_eq!(shape.len(), 1);
    let lines = nuxie_runtime::source::text_engine::GlyphLine::break_lines(&shape[0].runs, 10000.0);
    assert_eq!(lines.len(), 1);
    let mut ellipsis = GlyphRun::default();
    let _line = OrderedLine::new(
        &shape[0],
        &lines[0],
        120.0,
        TextAlign::Left,
        true,
        true,
        &mut ellipsis,
        0.0,
    );
    assert!(ellipsis.font.is_some());
    ellipsis
}
#[test]
fn ellipsis_follows_a_run_that_changes_only_its_size() {
    let font = HbFont::decode(&asset("RobotoFlex.ttf")).unwrap();
    let ellipsis = ellipsis_for_runs(font.clone(), 10.0, font, 30.0);
    assert_eq!(ellipsis.size, 30.0);
}
#[test]
fn ellipsis_follows_a_run_that_changes_only_its_font() {
    let roboto = HbFont::decode(&asset("RobotoFlex.ttf")).unwrap();
    let inter = HbFont::decode(&asset("fonts/Inter_18pt-Regular.ttf")).unwrap();
    let ellipsis = ellipsis_for_runs(roboto, 20.0, inter.clone(), 20.0);
    assert!(std::sync::Arc::ptr_eq(
        ellipsis.font.as_ref().unwrap(),
        &inter
    ));
}
