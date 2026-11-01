//! renderer/cmd/deferred_render_factory.hpp and deferred_session.hpp at
//! a4dbc3ff: the recorder's CTM shadow, which cache-as-bitmap reads at record
//! time to size its raster. The scope in that shadow is ours: a draw reads it
//! at record time to know a fade reaches it (docs/CSS_LAYOUT.md, 4.4).
use super::super::deferred_session::DeferredSession;
use super::*;

#[test]
fn a_recorder_reports_the_transform_the_replay_will_build() {
    let session = DeferredSession::with_caps(Default::default());
    let screen = session.screen_renderer(0);
    let mut screen = screen.borrow_mut();
    assert_eq!(screen.current_transform(), Some(Mat2D::IDENTITY));
    screen.save();
    screen.transform(Mat2D([2.0, 0.0, 0.0, 2.0, 10.0, 0.0]));
    screen.transform(Mat2D([1.0, 0.0, 0.0, 1.0, 5.0, 3.0]));
    // Concatenated in local space: the translate is scaled by the outer 2x.
    assert_eq!(
        screen.current_transform(),
        Some(Mat2D([2.0, 0.0, 0.0, 2.0, 20.0, 6.0]))
    );
    screen.restore();
    assert_eq!(screen.current_transform(), Some(Mat2D::IDENTITY));
    // An unbalanced restore leaves the bottom entry in place.
    screen.restore();
    assert_eq!(screen.current_transform(), Some(Mat2D::IDENTITY));
}

#[test]
fn a_retained_screen_recorder_drops_its_transform_at_the_frame_boundary() {
    let mut session = DeferredSession::with_caps(Default::default());
    let screen = session.screen_renderer(0);
    // A top level transform with no restore, as an FFI host may leave it.
    screen
        .borrow_mut()
        .transform(Mat2D([3.0, 0.0, 0.0, 3.0, 0.0, 0.0]));
    session.reset_frame();
    // The host still holds the same recorder across frames.
    assert!(Rc::ptr_eq(&screen, &session.screen_renderer(0)));
    assert_eq!(screen.borrow().current_transform(), Some(Mat2D::IDENTITY));
}

#[test]
fn a_canvas_content_renderer_reports_its_own_transform() {
    let mut session = DeferredSession::with_caps(Default::default());
    let canvas = fake_canvas();
    let mut content = session
        .begin_canvas_content(canvas.clone(), 0)
        .expect("content renderer");
    assert_eq!(content.current_transform(), Some(Mat2D::IDENTITY));
    content.transform(Mat2D([0.5, 0.0, 0.0, 0.5, 0.0, 0.0]));
    assert_eq!(
        content.current_transform(),
        Some(Mat2D([0.5, 0.0, 0.0, 0.5, 0.0, 0.0]))
    );
    drop(content);
    session.end_canvas_content(&canvas);
}

#[test]
fn a_recorder_reports_the_scope_the_replay_will_build() {
    let session = DeferredSession::with_caps(Default::default());
    let screen = session.screen_renderer(0);
    let mut screen = screen.borrow_mut();
    assert_eq!(screen.current_modulated_opacity(), Some(1.0));
    screen.save();
    screen.modulate_opacity(0.5);
    screen.save();
    screen.modulate_opacity(0.5);
    assert_eq!(screen.current_modulated_opacity(), Some(0.25));
    screen.restore();
    assert_eq!(screen.current_modulated_opacity(), Some(0.5));
    screen.restore();
    assert_eq!(screen.current_modulated_opacity(), Some(1.0));
    // RiveRenderer::modulateOpacity stops at 0, so replay reads the same.
    screen.modulate_opacity(-1.0);
    assert_eq!(screen.current_modulated_opacity(), Some(0.0));
}

#[test]
fn a_retained_screen_recorder_drops_its_scope_at_the_frame_boundary() {
    let mut session = DeferredSession::with_caps(Default::default());
    let screen = session.screen_renderer(0);
    // A top level scope with no restore, as an FFI host may leave it.
    screen.borrow_mut().modulate_opacity(0.5);
    session.reset_frame();
    assert_eq!(screen.borrow().current_modulated_opacity(), Some(1.0));
}

#[test]
fn a_canvas_content_renderer_starts_at_one_under_a_scope() {
    let mut session = DeferredSession::with_caps(Default::default());
    let screen = session.screen_renderer(0);
    screen.borrow_mut().modulate_opacity(0.5);
    let canvas = fake_canvas();
    let mut content = session
        .begin_canvas_content(canvas.clone(), 0)
        .expect("content renderer");
    assert_eq!(content.current_modulated_opacity(), Some(1.0));
    content.modulate_opacity(0.25);
    assert_eq!(content.current_modulated_opacity(), Some(0.25));
    assert_eq!(screen.borrow().current_modulated_opacity(), Some(0.5));
    drop(content);
    session.end_canvas_content(&canvas);

    // A canvas opened again in the frame reuses its recorder, and starts over.
    let content = session
        .begin_canvas_content(canvas.clone(), 0)
        .expect("content renderer");
    assert_eq!(content.current_modulated_opacity(), Some(1.0));
    drop(content);
    session.end_canvas_content(&canvas);
}
