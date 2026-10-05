//! All sixteen cases from d7d5fdd8 tests/unit_tests/runtime/text_selection_test.cpp.
use nuxie_render_api::{
    BlendMode, ImageSampler, Mat2D, PersistentFactory, RecordingFactory, RenderBuffer, RenderImage,
    RenderPaint, RenderPath, Renderer,
};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    core::{CoreArena, CoreHandle},
    drawable_flag::DrawableFlag,
    generated::{
        core_registry::CoreRegistry,
        drawable_base::DrawableBase,
        node_base::NodeBase,
        text::{
            text_base::TextBase, text_style_base::TextStyleBase,
            text_value_run_base::TextValueRunBase,
        },
        transform_component_base::TransformComponentBase,
        world_transform_component_base::WorldTransformComponentBase,
    },
    input::focusable::{Key, KeyModifiers},
    math::vec2d::Vec2D,
    selection_style::SelectionStyle,
    text::{
        cursor::CursorPosition,
        glyph_lookup::GlyphLookup,
        text::{Text, TextValueRunHandle},
        text_layout_view::TextLayoutView,
        text_selection_controller::TextSelectionController,
        text_value_run::TextValueRun,
    },
};
use nuxie_runtime::{
    Artboard, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};

fn import(name: &str) -> RuntimeFileHandle {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream checkout"),
    );
    let bytes = std::fs::read(root.join("tests/unit_tests/assets").join(name)).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    File::import(&bytes, factory, None, None, None).unwrap()
}
fn scalar(owner: &CoreHandle, key: u16, value: f32) {
    assert!(CoreRegistry::set_double_handle(
        owner,
        i32::from(key),
        value
    ));
}
fn integer(owner: &CoreHandle, key: u16, value: u32) {
    assert!(CoreRegistry::set_uint_handle(owner, i32::from(key), value));
}
fn run(text: &CoreHandle, index: usize) -> CoreHandle {
    text.with_downcast::<Text, _>(|text| match &text.runs()[index] {
        TextValueRunHandle::Core(run) => run.clone(),
        TextValueRunHandle::Runtime(_) => {
            panic!("imported fixture run must have an arena identity")
        }
    })
    .unwrap()
}
fn set_run(run: &CoreHandle, value: &str) {
    assert!(CoreRegistry::set_string_handle(
        run,
        i32::from(TextValueRunBase::TEXT_PROPERTY_KEY),
        value.to_owned()
    ));
}
fn set_text(text: &CoreHandle, value: &str) {
    let count = text
        .with_downcast::<Text, _>(|text| text.runs().len())
        .unwrap();
    assert!(count > 0);
    set_run(&run(text, 0), value);
    for index in 1..count {
        set_run(&run(text, index), "");
    }
}
fn point_at(text: &CoreHandle, offset: u32) -> Vec2D {
    // Independent lookup on actual drawn lines, not controller hit testing.
    let (point, artboard) = text
        .with_downcast::<Text, _>(|text| {
            let mut lookup = GlyphLookup::default();
            lookup.compute(text.unichars(), text.shape());
            let view = TextLayoutView::new(
                text.shape(),
                &[],
                text.ordered_lines(),
                &lookup,
                text.unichars().len() as u32,
            );
            let mut position = CursorPosition::unresolved(offset);
            position.resolve_line(&view);
            let visual = position.clamped(&view).visual_position(&view);
            assert!(visual.found());
            (
                text.shape_world_transform()
                    * Vec2D::new(visual.x(), (visual.top() + visual.bottom()) / 2.0),
                text.artboard_handle().unwrap(),
            )
        })
        .unwrap();
    artboard
        .with_downcast_mut::<Artboard, _>(|artboard| artboard.root_transform(point))
        .unwrap()
}
fn advance(artboard: &CoreHandle) {
    Artboard::advance_handle(
        artboard,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}
struct Scene {
    _file: RuntimeFileHandle,
    artboard: Option<RuntimeArtboardInstanceHandle>,
    texts: Vec<CoreHandle>,
    selection: TextSelectionController,
}
impl Scene {
    fn new() -> Self {
        let file = import("new_text.riv");
        let source = file.with_file(|file| file.artboard()).unwrap();
        let artboard = Artboard::instance_from_handle(&source).unwrap();
        let texts = artboard.with_artboard(|artboard| artboard.find_all_handles::<Text>());
        assert!(texts.len() >= 4);
        for (i, value) in [
            "Introduction",
            "Rive supports interactive graphics.",
            "Learn more",
            "Outside the selected range",
        ]
        .iter()
        .enumerate()
        {
            let text = &texts[i];
            set_text(text, value);
            scalar(text, NodeBase::X_PROPERTY_KEY, 0.0);
            scalar(text, NodeBase::Y_PROPERTY_KEY, i as f32 * 200.0);
            scalar(text, TextBase::ORIGIN_X_PROPERTY_KEY, 0.0);
            scalar(text, TextBase::ORIGIN_Y_PROPERTY_KEY, 0.0);
            integer(text, TextBase::SIZING_VALUE_PROPERTY_KEY, 0);
            integer(text, TextBase::OVERFLOW_VALUE_PROPERTY_KEY, 0);
            let styles = text
                .with_downcast::<Text, _>(|text| text.text_style_paints().to_vec())
                .unwrap();
            for style in styles {
                scalar(&style, TextStyleBase::FONT_SIZE_PROPERTY_KEY, 24.0);
            }
        }
        artboard.advance_default(0.0);
        let mut selection = TextSelectionController::new();
        for text in &texts[..4] {
            assert!(selection.add(text, "\n"));
        }
        Self {
            _file: file,
            artboard: Some(artboard),
            texts,
            selection,
        }
    }
    fn advance(&self) {
        self.artboard.as_ref().unwrap().advance_default(0.0);
    }
}
#[derive(Default)]
struct CountingRenderer {
    paths: usize,
}
impl Renderer for CountingRenderer {
    fn save(&mut self) {}
    fn restore(&mut self) {}
    fn transform(&mut self, _: Mat2D) {}
    fn draw_path(&mut self, _: &dyn RenderPath, _: &dyn RenderPaint) {
        self.paths += 1;
    }
    fn clip_path(&mut self, _: &dyn RenderPath) {}
    fn draw_image(&mut self, _: Option<&dyn RenderImage>, _: ImageSampler, _: BlendMode, _: f32) {}
    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: u32,
        _: u32,
        _: BlendMode,
        _: f32,
    ) {
    }
    fn modulate_opacity(&mut self, _: f32) {}
}

#[test]
fn regular_text_drag_spans_objects_and_gaps_in_reading_order() {
    let mut scene = Scene::new();
    let text = &scene.texts;
    let start = point_at(&text[0], 5);
    let end = point_at(&text[2], 5);
    let s = &mut scene.selection;
    assert!(s.pointer_down(Some(&text[0]), start, 7, false));
    assert!(s.selected_text().is_empty());
    assert!(s.pointer_move(Vec2D::new(end.x + 20.0, (start.y + end.y) / 2.0), 7));
    assert!(s.pointer_up(end, 7));
    assert!(!s.is_dragging());
    assert_eq!(
        s.selected_text(),
        "duction\nRive supports interactive graphics.\nLearn"
    );
    for t in &text[..3] {
        assert!(!s.selection_rects(t).is_empty());
    }
    assert!(s.selection_rects(&text[3]).is_empty());
    assert!(s.pointer_down(Some(&text[2]), end, 0, false));
    assert!(s.pointer_up(start, 0));
    assert_eq!(
        s.selected_text(),
        "duction\nRive supports interactive graphics.\nLearn"
    );
}
#[test]
fn regular_text_selection_builds_its_lookup_only_when_needed() {
    let mut scene = Scene::new();
    let a = scene.texts[0].clone();
    let b = scene.texts[1].clone();
    let mut renderer = CountingRenderer::default();
    for text in &scene.texts {
        assert_eq!(scene.selection.lookup_build_count(text), 0);
    }
    scene.selection.set_range(&a, 0, 3);
    scene.selection.set_range(&b, 0, 0);
    b.with_downcast_mut::<Text, _>(|text| text.draw(&mut renderer))
        .unwrap();
    assert!(scene.selection.selection_rects(&b).is_empty());
    assert_eq!(scene.selection.lookup_build_count(&b), 0);
    assert!(!scene.selection.selection_rects(&a).is_empty());
    assert_eq!(scene.selection.lookup_build_count(&a), 1);
    let arena = CoreArena::default();
    let mut style = SelectionStyle::default();
    style.set_highlight_color(0x80ff8800u32 as i32);
    style.set_corner_radius(6.0);
    let style = arena.insert(style);
    scene.selection.set_style(Some(style));
    a.with_downcast_mut::<Text, _>(Text::mark_paint_dirty)
        .unwrap();
    scene.advance();
    scene.selection.set_range(&a, 0, 3);
    assert!(!scene.selection.selection_rects(&a).is_empty());
    assert_eq!(scene.selection.lookup_build_count(&a), 1);
    let style = a
        .with_downcast::<Text, _>(|text| text.text_style_paints()[0].clone())
        .unwrap();
    scalar(&style, TextStyleBase::FONT_SIZE_PROPERTY_KEY, 32.0);
    scene.advance();
    assert_eq!(scene.selection.lookup_build_count(&a), 1);
    assert!(!scene.selection.selection_rects(&a).is_empty());
    assert_eq!(scene.selection.lookup_build_count(&a), 2);
    set_text(&a, "Changed source");
    scene.advance();
    a.with_downcast_mut::<Text, _>(|text| text.draw(&mut renderer))
        .unwrap();
    assert!(scene.selection.selected_text().is_empty());
    assert_eq!(scene.selection.lookup_build_count(&a), 2);
}
#[test]
fn regular_text_selection_includes_last_character_and_object_separators() {
    let mut scene = Scene::new();
    let a = &scene.texts[0];
    let b = &scene.texts[1];
    let s = &mut scene.selection;
    assert!(s.select(a, 0, a, u32::MAX));
    assert_eq!(s.selected_text(), "Introduction");
    assert_eq!(s.selection(a).last().code_point_index(), 12);
    assert!(s.select(a, 12, b, 0));
    assert_eq!(s.selected_text(), "\n");
    assert!(s.select(b, 0, a, 12));
    assert_eq!(s.selected_text(), "\n");
    assert!(s.select(a, 12, a, 12));
    assert!(!s.has_selection());
    assert!(s.selected_text().is_empty());
    assert!(s.selection_rects(a).is_empty());
}
#[test]
fn regular_text_selection_copies_runs_exactly_without_visual_wrap_breaks() {
    let mut scene = Scene::new();
    let text = scene
        .texts
        .iter()
        .find(|text| {
            text.with_downcast::<Text, _>(|text| text.runs().len() >= 2)
                .unwrap()
        })
        .unwrap()
        .clone();
    set_text(&text, "Hello ");
    set_run(&run(&text, 1), "styled world\nNext paragraph");
    integer(&text, TextBase::SIZING_VALUE_PROPERTY_KEY, 1);
    scalar(&text, TextBase::WIDTH_PROPERTY_KEY, 100.0);
    scene.advance();
    let mut separate = TextSelectionController::new();
    scene.selection.remove(&text);
    assert!(separate.add(&text, "\n"));
    assert!(
        text.with_downcast::<Text, _>(|text| text.ordered_lines().len() > 2)
            .unwrap()
    );
    separate.select_all();
    assert_eq!(
        separate.selected_text(),
        "Hello styled world\nNext paragraph"
    );
    assert!(!separate.selection_rects(&text).is_empty());
}
#[test]
fn regular_text_highlights_draw_within_existing_text_render_pass() {
    let mut scene = Scene::new();
    let artboard = scene.artboard.as_ref().unwrap();
    let mut before = CountingRenderer::default();
    artboard.draw(&mut before);
    assert!(
        scene
            .selection
            .select(&scene.texts[0], 3, &scene.texts[2], 5)
    );
    let mut selected = CountingRenderer::default();
    artboard.draw(&mut selected);
    assert_eq!(selected.paths, before.paths + 3);
    scene.selection.clear();
    let mut after = CountingRenderer::default();
    artboard.draw(&mut after);
    assert_eq!(after.paths, before.paths);
}
#[test]
fn regular_text_selection_survives_relayout_and_whole_object_transforms() {
    let mut scene = Scene::new();
    let text = scene.texts[1].clone();
    assert!(scene.selection.select(&text, 0, &text, u32::MAX));
    let value = scene.selection.selected_text();
    integer(&text, TextBase::SIZING_VALUE_PROPERTY_KEY, 1);
    scalar(&text, TextBase::WIDTH_PROPERTY_KEY, 130.0);
    scalar(&text, TextBase::ORIGIN_X_PROPERTY_KEY, 0.5);
    scalar(&text, TransformComponentBase::ROTATION_PROPERTY_KEY, 0.35);
    scalar(&text, TransformComponentBase::SCALE_X_PROPERTY_KEY, 1.5);
    scene.advance();
    assert_eq!(scene.selection.selected_text(), value);
    assert!(
        text.with_downcast::<Text, _>(|text| text.ordered_lines().len() > 1)
            .unwrap()
    );
    assert!(
        scene
            .selection
            .pointer_down(Some(&text), point_at(&text, 5), 0, false)
    );
    assert!(scene.selection.pointer_up(point_at(&text, 13), 0));
    assert_eq!(scene.selection.selected_text(), "supports");
    assert!(!scene.selection.selection_rects(&text).is_empty());
}
#[test]
fn regular_text_source_changes_invalidate_shared_range_and_capture() {
    let mut scene = Scene::new();
    assert!(scene.selection.pointer_down(
        Some(&scene.texts[0]),
        point_at(&scene.texts[0], 1),
        0,
        false
    ));
    assert!(
        scene
            .selection
            .pointer_move(point_at(&scene.texts[2], 5), 0)
    );
    assert!(scene.selection.has_selection());
    set_text(&scene.texts[1], "Updated from data binding");
    scene.advance();
    assert!(!scene.selection.has_selection());
    assert!(!scene.selection.is_dragging());
    assert!(scene.selection.selected_text().is_empty());
    assert!(scene.selection.selection_rects(&scene.texts[0]).is_empty());
    assert!(
        !scene
            .selection
            .pointer_move(point_at(&scene.texts[2], 6), 0)
    );
}
#[test]
fn regular_text_selection_keeps_one_pointer_and_handles_cancellation() {
    let mut scene = Scene::new();
    let text = &scene.texts[0];
    let s = &mut scene.selection;
    assert!(s.pointer_down(Some(text), point_at(text, 1), 9, false));
    assert!(!s.pointer_down(Some(text), point_at(text, 8), 10, false));
    assert!(!s.pointer_move(point_at(text, 8), 10));
    assert!(s.pointer_move(point_at(text, 5), 9));
    assert_eq!(s.selected_text(), "ntro");
    assert!(!s.pointer_cancel(10));
    assert!(s.pointer_move(Vec2D::new(f32::NAN, 0.0), 9));
    assert_eq!(s.selected_text(), "ntro");
    assert!(s.pointer_cancel(9));
    assert!(!s.is_dragging());
    assert!(s.selected_text().is_empty());
    assert!(!s.pointer_down(None, point_at(text, 0), 0, false));
}
#[test]
fn regular_text_accepts_selection_commands_without_editing() {
    let mut scene = Scene::new();
    let text = &scene.texts[0];
    let s = &mut scene.selection;
    assert!(s.select(text, 1, text, 4));
    for (key, modifiers) in [
        (Key::BACKSPACE, KeyModifiers::NONE),
        (Key::DELETE, KeyModifiers::NONE),
        (Key::V, KeyModifiers::META),
        (Key::X, KeyModifiers::CTRL),
    ] {
        assert!(!s.key_input(key, modifiers, true));
    }
    assert_eq!(s.selected_text(), "ntr");
    assert!(s.key_input(Key::A, KeyModifiers::CTRL, true));
    assert_eq!(
        s.selected_text(),
        "Introduction\nRive supports interactive graphics.\nLearn more\nOutside the selected range"
    );
    assert!(s.key_input(Key::ESCAPE, KeyModifiers::NONE, true));
    assert!(s.selected_text().is_empty());
    assert_eq!(
        run(text, 0)
            .with_downcast::<TextValueRun, _>(|run| run.base.text().to_owned())
            .unwrap(),
        "Introduction"
    );
}
#[test]
fn regular_text_selection_ownership_survives_either_destruction_order() {
    let mut scene = Scene::new();
    let text = &scene.texts[0];
    assert!(!scene.selection.add(text, "\n"));
    let mut other = TextSelectionController::new();
    assert!(!other.add(text, "\n"));
    scene.selection.remove(text);
    assert!(other.add(text, "\n"));
    other.select_all();
    assert!(other.has_selection());
    drop(scene.artboard.take());
    assert!(!other.has_selection());
    assert!(other.selected_text().is_empty());
    assert!(scene.selection.selected_text().is_empty());
}
#[test]
fn regular_text_selection_supports_inline_object_separators() {
    let mut scene = Scene::new();
    let a = &scene.texts[0];
    let b = &scene.texts[1];
    scene.selection.remove(a);
    scene.selection.remove(b);
    let mut inline = TextSelectionController::new();
    assert!(inline.add(a, "\n"));
    assert!(inline.add(b, " "));
    assert!(inline.select(a, 5, b, 4));
    assert_eq!(inline.selected_text(), "duction Rive");
}
#[test]
fn regular_text_selection_copies_unicode_source_offsets() {
    let mut scene = Scene::new();
    let text = scene.texts[0].clone();
    set_text(&text, "Aé😀Z");
    scene.advance();
    assert!(scene.selection.select(&text, 1, &text, 3));
    assert_eq!(scene.selection.selected_text(), "é😀");
    assert!(scene.selection.select(&text, 0, &text, u32::MAX));
    assert_eq!(scene.selection.selected_text(), "Aé😀Z");
}
#[test]
fn empty_and_hidden_text_cannot_leave_stale_selection_highlights() {
    let mut scene = Scene::new();
    let text = scene.texts[0].clone();
    set_text(&text, "");
    scene.advance();
    assert!(!scene.selection.select(&text, 0, &text, 1));
    assert!(scene.selection.selection_rects(&text).is_empty());
    scene.selection.select_all();
    assert_eq!(
        scene.selection.selected_text(),
        "Rive supports interactive graphics.\nLearn more\nOutside the selected range"
    );
    scalar(
        &scene.texts[1],
        WorldTransformComponentBase::OPACITY_PROPERTY_KEY,
        0.0,
    );
    scene.advance();
    assert!(scene.selection.selected_text().is_empty());
    assert!(scene.selection.selection_rects(&scene.texts[2]).is_empty());
}
#[test]
fn destroying_selection_controller_unregisters_its_text_objects() {
    let file = import("hello_world.riv");
    let artboard = file.with_file(|file| file.artboard()).unwrap();
    advance(&artboard);
    let text = artboard
        .with_downcast::<Artboard, _>(|a| a.find_all_handles::<Text>()[0].clone())
        .unwrap();
    {
        let mut selection = TextSelectionController::new();
        assert!(selection.add(&text, "\n"));
        selection.select_all();
        assert_eq!(selection.selected_text(), "Hello World!");
    }
    let mut next = TextSelectionController::new();
    assert!(next.add(&text, "\n"));
    assert!(next.selected_text().is_empty());
    integer(&text, TextBase::OVERFLOW_VALUE_PROPERTY_KEY, 3);
    advance(&artboard);
    assert!(!next.select(&text, 0, &text, 5));
}
#[test]
fn regular_text_selection_spans_actual_nested_artboard_instances() {
    let file = import("runtime_nested_text_runs.riv");
    let source = file
        .with_file(|file| file.artboard_named_source("ArtboardA"))
        .unwrap();
    let root = Artboard::instance_from_handle(&source).unwrap();
    let first_run = root
        .with_artboard(|a| a.get_text_run("ArtboardBRun", "ArtboardB-1"))
        .unwrap();
    let middle_run = root
        .with_artboard(|a| a.get_text_run("ArtboardCRun", "ArtboardB-1/ArtboardC-1"))
        .unwrap();
    let last_run = root
        .with_artboard(|a| a.get_text_run("ArtboardBRun", "ArtboardB-2"))
        .unwrap();
    set_run(&first_run, "Introduction");
    set_run(&middle_run, "Nested middle");
    set_run(&last_run, "Learn more");
    root.advance_default(0.0);
    let text_of = |run: &CoreHandle| {
        run.with_downcast::<TextValueRun, _>(TextValueRun::text_component)
            .unwrap()
            .unwrap()
    };
    let first = text_of(&first_run);
    let middle = text_of(&middle_run);
    let last = text_of(&last_run);
    assert!(first != last);
    assert!(
        first
            .with_downcast::<Text, _>(|t| t.artboard_handle())
            .unwrap()
            != last
                .with_downcast::<Text, _>(|t| t.artboard_handle())
                .unwrap()
    );
    assert_eq!(
        first
            .with_downcast::<Text, _>(|t| t.name().to_owned())
            .unwrap(),
        last.with_downcast::<Text, _>(|t| t.name().to_owned())
            .unwrap()
    );
    let mut selection = TextSelectionController::new();
    for text in [&first, &middle, &last] {
        assert!(selection.add(text, "\n"));
    }
    assert!(selection.pointer_down(Some(&first), point_at(&first, 5), 0, false));
    assert!(selection.pointer_up(point_at(&last, 5), 0));
    assert_eq!(selection.selected_text(), "duction\nNested middle\nLearn");
    assert!(!selection.selection_rects(&middle).is_empty());
}
#[test]
fn selectable_discovery_exposes_stable_safe_host_tokens() {
    let mut scene = Scene::new();
    let root = scene.artboard.as_ref().unwrap().core_handle();
    let a = &scene.texts[0];
    let b = &scene.texts[1];
    let s = &mut scene.selection;
    s.synchronize(Some(&root));
    assert!(s.target_ids().is_empty());
    integer(
        a,
        DrawableBase::DRAWABLE_FLAGS_PROPERTY_KEY,
        u32::from(DrawableFlag::SELECTABLE.0),
    );
    integer(
        b,
        DrawableBase::DRAWABLE_FLAGS_PROPERTY_KEY,
        u32::from(DrawableFlag::SELECTABLE.0),
    );
    s.synchronize(Some(&root));
    let ids = s.target_ids();
    assert_eq!(ids.len(), 2);
    assert!(s.target(ids[0]).as_ref() == Some(a));
    s.synchronize(Some(&root));
    assert_eq!(s.target_ids(), ids);
    assert_eq!(s.source_text(a), "Introduction");
    let mut offset = 0;
    let mut distance = 0.0;
    assert!(s.hit_test(a, point_at(a, 5), false, &mut offset, &mut distance));
    assert_eq!(offset, 5);
    assert_eq!(distance, 0.0);
    assert!(!s.hit_test(
        a,
        Vec2D::new(10000.0, 10000.0),
        false,
        &mut offset,
        &mut distance
    ));
    assert!(s.hit_test(
        a,
        Vec2D::new(10000.0, 10000.0),
        true,
        &mut offset,
        &mut distance
    ));
    s.set_range(a, 5, 12);
    s.set_range(b, 0, 4);
    assert!(s.has_selection());
    assert_eq!(s.selected_text(), "duction\nRive");
    assert!(!s.selection_rects(a).is_empty());
    assert!(!s.selection_rects(b).is_empty());
    integer(a, DrawableBase::DRAWABLE_FLAGS_PROPERTY_KEY, 0);
    s.synchronize(Some(&root));
    assert!(s.target(ids[0]).is_none());
    assert!(!s.has_selection());
    integer(
        a,
        DrawableBase::DRAWABLE_FLAGS_PROPERTY_KEY,
        u32::from(DrawableFlag::SELECTABLE.0),
    );
    s.synchronize(Some(&root));
    assert!(s.target(ids[0]).is_none());
    assert_eq!(s.target_ids().len(), 2);
}

// Supplemental Rust ownership-boundary regression, not an upstream case:
// releasing the controller's retained File may synchronously destroy a source
// Text and unregister it from the same controller.
#[test]
fn replacing_style_file_releases_controller_borrow_before_source_text_destruction() {
    let file = import("hello_world.riv");
    let weak_file = file.downgrade();
    let source = file.with_file(|file| file.artboard()).unwrap();
    advance(&source);
    let text = source
        .with_downcast::<Artboard, _>(|artboard| artboard.find_all_handles::<Text>()[0].clone())
        .unwrap();
    let instance = Artboard::instance_from_handle(&source).unwrap();
    let mut selection = TextSelectionController::new();
    selection.synchronize(Some(&instance.core_handle()));
    assert!(selection.add(&text, "\n"));
    selection.select_all();
    assert_eq!(selection.selected_text(), "Hello World!");

    drop(file);
    drop(instance);
    assert!(weak_file.upgrade().is_some());
    assert!(text.is_alive());
    selection.synchronize(None);

    assert!(weak_file.upgrade().is_none());
    assert!(!text.is_alive());
    assert!(selection.target_ids().is_empty());
    assert!(!selection.has_selection());
    assert!(selection.selected_text().is_empty());
}

#[test]
fn regular_selection_keeps_upstream_cr_cell_and_skips_line_feed() {
    // d7d5fdd8 src/text/cursor.cpp:254-316 emits every overlapping glyph cell.
    let mut scene = Scene::new();
    let text = scene.texts[0].clone();
    set_text(&text, "a\r\nb");
    integer(&text, TextBase::WRAP_VALUE_PROPERTY_KEY, 1);
    scene.advance();
    scene.selection.set_range(&text, 1, 2);
    assert_eq!(scene.selection.selection_rects(&text).len(), 1);
    scene.selection.set_range(&text, 2, 3);
    assert!(scene.selection.selection_rects(&text).is_empty());
}

#[test]
fn regular_unicode_whitespace_does_not_create_a_third_soft_wrap_line() {
    let scene = Scene::new();
    let text = scene.texts[0].clone();
    set_text(&text, "a  \u{2003}a");
    integer(&text, TextBase::SIZING_VALUE_PROPERTY_KEY, 1);
    integer(&text, TextBase::WRAP_VALUE_PROPERTY_KEY, 0);
    scalar(&text, TextBase::WIDTH_PROPERTY_KEY, 8.0);
    scene.advance();
    text.with_downcast::<Text, _>(|text| {
        assert_eq!(text.ordered_lines().len(), 2);
    })
    .unwrap();
}
