//! Direct translation of `tests/unit_tests/runtime/text_style_background_test.cpp`
//! at upstream 1f04919af881fe51c929924dc773c835ca9071f0.

use nuxie_render_api::{FillRule, NullRenderer, PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    math::{aabb::Aabb, path_types::PathVerb},
    shapes::{paint::shape_paint_path::ShapePaintPath, shape_paint_container::ShapePaintContainer},
    text::{
        text::Text, text_selection_path::TextSelectionPath,
        text_style_background::TextStyleBackground, text_style_paint::TextStylePaint,
    },
};
use nuxie_runtime::{Artboard, File, RuntimeFactoryHandle};

// Upstream ShapePaintPath::numContours counts move verbs in the raw path.
fn num_contours(path: &ShapePaintPath) -> usize {
    path.raw_path()
        .verbs()
        .iter()
        .filter(|&&verb| verb == PathVerb::Move)
        .count()
}

fn touching_lines() -> [Aabb; 2] {
    [
        Aabb::new(0.0, 0.0, 100.0, 20.0),
        Aabb::new(0.0, 20.0, 60.0, 40.0),
    ]
}

#[test]
fn selection_path_joins_rects_from_multiple_lines() {
    let mut selection = TextSelectionPath::new(true, FillRule::EvenOdd);
    selection.update(&touching_lines(), 0.0);
    let path = &selection.path;
    assert_eq!(path.fill_rule(), FillRule::EvenOdd);
    assert!(!path.empty());
    assert_eq!(num_contours(path), 1);
    let bounds = path.raw_path().bounds();
    assert_eq!(bounds.left(), 0.0);
    assert_eq!(bounds.top(), 0.0);
    assert_eq!(bounds.right(), 100.0);
    assert_eq!(bounds.bottom(), 40.0);
    for verb in path.raw_path().verbs() {
        assert_ne!(*verb, PathVerb::Cubic);
    }
}

#[test]
fn selection_path_rounds_corners_with_clamped_radius() {
    let mut selection = TextSelectionPath::new(true, FillRule::EvenOdd);
    selection.update(&touching_lines(), 6.0);
    let path = &selection.path;
    assert!(!path.empty());
    let cubics = path
        .raw_path()
        .verbs()
        .iter()
        .filter(|&&verb| verb == PathVerb::Cubic)
        .count();
    assert_eq!(cubics, 6);
    let bounds = path.raw_path().bounds();
    assert!(bounds.left() >= 0.0);
    assert!(bounds.top() >= 0.0);
    assert!(bounds.right() <= 100.0);
    assert!(bounds.bottom() <= 40.0);
}

#[test]
fn selection_path_keeps_disjoint_lines_as_separate_contours() {
    let mut selection = TextSelectionPath::new(true, FillRule::EvenOdd);
    selection.update(
        &[
            Aabb::new(0.0, 0.0, 100.0, 20.0),
            Aabb::new(0.0, 30.0, 60.0, 50.0),
        ],
        4.0,
    );
    assert_eq!(num_contours(&selection.path), 2);
}

#[test]
fn selection_path_rewinds_between_updates() {
    let mut selection = TextSelectionPath::new(true, FillRule::EvenOdd);
    let rects = [Aabb::new(0.0, 0.0, 100.0, 20.0)];
    selection.update(&rects, 0.0);
    assert_eq!(num_contours(&selection.path), 1);
    selection.update(&rects, 0.0);
    assert_eq!(num_contours(&selection.path), 1);
    selection.update(&[], 0.0);
    assert!(selection.path.empty());
}

#[test]
fn editor_exported_text_style_background_renders_at_runtime() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/sync/text_style_background.riv");
    let bytes = std::fs::read(&fixture).unwrap_or_else(|error| {
        panic!(
            "read pinned fixture {} (run make fixtures): {error}",
            fixture.display()
        )
    });
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file =
        File::import(&bytes, retained, None, None, None).expect("background fixture imports");
    let artboard = file
        .with_file(|file| file.artboard())
        .expect("source artboard");
    let backgrounds = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_all_handles::<TextStyleBackground>())
        .unwrap();
    assert_eq!(backgrounds.len(), 1);
    let background = &backgrounds[0];
    let parent = background
        .with_downcast::<TextStyleBackground, _>(|background| {
            assert_eq!(background.base.corner_radius(), 8.0);
            assert_eq!(background.paints.shape_paints().len(), 2);
            background.base.parent_handle().expect("background parent")
        })
        .unwrap();
    let style_background = parent
        .with_downcast::<TextStylePaint, _>(TextStylePaint::background)
        .expect("background parent is TextStylePaint");
    assert_eq!(style_background.as_ref(), Some(background));

    Artboard::advance_handle(
        &artboard,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
    let path_bounds = background
        .with_downcast_mut::<TextStyleBackground, _>(|background| {
            let path = background.local_path();
            assert!(!path.empty());
            assert_eq!(num_contours(path), 1);
            assert!(path.raw_path().verbs().contains(&PathVerb::Cubic));
            path.raw_path().bounds()
        })
        .unwrap();
    let texts = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_all_handles::<Text>())
        .unwrap();
    assert_eq!(texts.len(), 1);
    let text_bounds = texts[0]
        .with_downcast::<Text, _>(Text::local_bounds)
        .unwrap();
    assert!(path_bounds.width() > 0.0);
    assert!(path_bounds.width() <= text_bounds.width() + 1.0);
    assert!(path_bounds.height() <= text_bounds.height() + 1.0);

    Artboard::draw_handle(&artboard, &mut NullRenderer::new());
}

#[test]
fn text_style_background_is_a_shape_paint_container() {
    let mut background = TextStyleBackground::default();
    assert!(std::ptr::eq(
        ShapePaintContainer::from_component(&background).expect("shape paint container"),
        &background.paints,
    ));
    // A Rust reference is non-null; preserve upstream's two-accessor identity assertion.
    let local_path = background.local_path() as *const ShapePaintPath;
    assert_eq!(
        background.local_clockwise_path() as *const ShapePaintPath,
        local_path
    );
    assert_eq!(background.local_path().fill_rule(), FillRule::EvenOdd);
    assert_eq!(background.base.corner_radius(), 0.0);
}
