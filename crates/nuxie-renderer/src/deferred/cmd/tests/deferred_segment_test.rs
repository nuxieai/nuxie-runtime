//! tests/unit_tests/renderer/deferred_segment_test.cpp at e949498e.
use super::super::{deferred_render_factory::DeferredRenderer, deferred_session::*};
use super::*;
fn canvas_recorder(session: &DeferredSession, id: u64) -> DeferredRenderer {
    DeferredRenderer::new(
        session.command_buffer(),
        Some(session.canvases()),
        Some(session.routing.clone()),
        id,
    )
}
fn size(session: &DeferredSession) -> u32 {
    session
        .command_buffer()
        .lock()
        .unwrap()
        .command_bytes()
        .len() as u32
}
#[test]
fn screen_only_frame_is_one_segment() {
    let mut s = DeferredSession::new(None);
    let paint = s.make_render_paint();
    let path = s.make_empty_render_path();
    let after = size(&s);
    s.screen_renderer(0)
        .borrow_mut()
        .draw_path(path.as_ref(), paint.as_ref());
    s.close_open_range();
    let all = s.scheduler_segments();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].target, SegmentTarget::Screen);
    assert_eq!(all[0].target_id, 0);
    assert_eq!(all[0].begin, after);
    assert_eq!(all[0].end, size(&s));
}
#[test]
fn untargeted_bytes_claim_no_segment() {
    let mut s = DeferredSession::new(None);
    let _paint = s.make_render_paint();
    let _path = s.make_empty_render_path();
    assert!(size(&s) > 0);
    s.close_open_range();
    assert!(s.scheduler_segments().is_empty());
}
#[test]
fn canvas_carves_leading_and_trailing_segments() {
    let mut s = DeferredSession::new(None);
    let mut canvas = canvas_recorder(&s, 1);
    let paint = s.make_render_paint();
    let path = s.make_empty_render_path();
    let after = size(&s);
    s.screen_renderer(0)
        .borrow_mut()
        .draw_path(path.as_ref(), paint.as_ref());
    canvas.draw_path(path.as_ref(), paint.as_ref());
    s.screen_renderer(0)
        .borrow_mut()
        .draw_path(path.as_ref(), paint.as_ref());
    s.close_open_range();
    let all = s.scheduler_segments();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].target, SegmentTarget::Screen);
    assert_eq!(all[0].begin, after);
    assert_eq!(all[1].target, SegmentTarget::Canvas);
    assert_eq!(all[1].target_id, 1);
    assert_eq!(all[1].begin, all[0].end);
    assert_eq!(all[2].target, SegmentTarget::Screen);
    assert_eq!(all[2].begin, all[1].end);
    assert_eq!(all[2].end, size(&s));
}
#[test]
fn initial_canvas_has_no_leading_screen() {
    let mut s = DeferredSession::new(None);
    let mut canvas = canvas_recorder(&s, 1);
    let paint = s.make_render_paint();
    let path = s.make_empty_render_path();
    s.close_open_range();
    let start = size(&s);
    canvas.draw_path(path.as_ref(), paint.as_ref());
    s.screen_renderer(0)
        .borrow_mut()
        .draw_path(path.as_ref(), paint.as_ref());
    s.close_open_range();
    let after: Vec<_> = s
        .scheduler_segments()
        .into_iter()
        .filter(|s| s.begin >= start)
        .collect();
    assert_eq!(after.len(), 2);
    assert_eq!(after[0].target, SegmentTarget::Canvas);
    assert_eq!(after[0].begin, start);
    assert_eq!(after[1].target, SegmentTarget::Screen);
    assert_eq!(after[1].begin, after[0].end);
}
#[test]
fn screen_targets_get_own_segments() {
    let mut s = DeferredSession::new(None);
    let paint = s.make_render_paint();
    let path = s.make_empty_render_path();
    for id in [0, 7, 0] {
        s.screen_renderer(id)
            .borrow_mut()
            .draw_path(path.as_ref(), paint.as_ref());
    }
    s.close_open_range();
    let all = s.scheduler_segments();
    assert_eq!(all.len(), 3);
    assert_eq!(
        all.iter().map(|s| s.target_id).collect::<Vec<_>>(),
        [0, 7, 0]
    );
    assert!(all.iter().all(|s| s.target == SegmentTarget::Screen));
    for pair in all.windows(2) {
        assert_eq!(pair[1].begin, pair[0].end);
    }
}
#[test]
fn canvas_hands_back_to_interrupted_screen() {
    let mut s = DeferredSession::new(None);
    let mut canvas = canvas_recorder(&s, 1);
    let paint = s.make_render_paint();
    let path = s.make_empty_render_path();
    s.screen_renderer(7)
        .borrow_mut()
        .draw_path(path.as_ref(), paint.as_ref());
    canvas.draw_path(path.as_ref(), paint.as_ref());
    s.close_open_range();
    let _later = s.make_empty_render_path();
    s.close_open_range();
    let all = s.scheduler_segments();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].target, SegmentTarget::Screen);
    assert_eq!(all[0].target_id, 7);
    assert_eq!(all[1].target, SegmentTarget::Canvas);
    assert_eq!(all[2].target, SegmentTarget::Screen);
    assert_eq!(all[2].target_id, 7);
}
#[test]
fn frame_closes_when_last_target_finishes() {
    let mut s = DeferredSession::new(None);
    let a = s.acquire_screen_target();
    let b = s.acquire_screen_target();
    assert_eq!(a, 0);
    assert_eq!(b, 1);
    s.begin_target_frame(a);
    assert!(s.end_target_frame(a));
    s.begin_target_frame(b);
    assert!(s.end_target_frame(b));
    s.begin_target_frame(a);
    s.begin_target_frame(b);
    assert!(!s.end_target_frame(b));
    assert!(s.end_target_frame(a));
    s.release_screen_target(a);
    assert_eq!(s.acquire_screen_target(), a);
}
#[test]
fn screen_recorder_survives_reset() {
    let mut s = DeferredSession::new(None);
    let first = s.screen_renderer(3);
    s.reset_frame();
    assert!(Rc::ptr_eq(&first, &s.screen_renderer(3)));
}
