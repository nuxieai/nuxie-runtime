//! tests/unit_tests/renderer/modulate_opacity_test.cpp at a4dbc3ff: the case
//! the cache-as-bitmap port added. The file's earlier cases have no port yet.
use super::render_context_null::*;
use super::*;

// The Renderer-level query a draw re-issued through a different renderer uses
// to carry this state across (Artboard::draw_cached_as_bitmap compositing
// through a fresh renderer, say).
#[test]
fn modulate_opacity_reported_through_the_renderer_api() {
    let (factory, _, _) = observing_factory(16, 16);
    let mut renderer = factory
        .borrow()
        .begin_frame(0, crate::RenderMode::RasterOrdering)
        .unwrap();

    assert_eq!(renderer.current_modulated_opacity(), Some(1.0));

    renderer.save();
    renderer.modulate_opacity(0.25);
    assert_eq!(renderer.current_modulated_opacity(), Some(0.25));
    renderer.restore();

    assert_eq!(renderer.current_modulated_opacity(), Some(1.0));

    // A renderer that does not track one reports nothing, so the caller can
    // tell "no opacity to carry" from "opacity 1".
    let untracked = NullRenderer;
    assert_eq!(untracked.current_modulated_opacity(), None);

    renderer.finish_without_readback().unwrap();
}

// Same query for the transform, which the cache reads to size its raster.
#[test]
fn current_transform_reported_through_the_renderer_api() {
    let (factory, _, _) = observing_factory(16, 16);
    let mut renderer = factory
        .borrow()
        .begin_frame(0, crate::RenderMode::RasterOrdering)
        .unwrap();
    assert_eq!(renderer.current_transform(), Some(Mat2D::IDENTITY));
    renderer.save();
    renderer.transform(Mat2D([2.0, 0.0, 0.0, 3.0, 5.0, 7.0]));
    assert_eq!(
        renderer.current_transform(),
        Some(Mat2D([2.0, 0.0, 0.0, 3.0, 5.0, 7.0]))
    );
    renderer.restore();
    assert_eq!(renderer.current_transform(), Some(Mat2D::IDENTITY));
    assert_eq!(NullRenderer.current_transform(), None);
    renderer.finish_without_readback().unwrap();
}
