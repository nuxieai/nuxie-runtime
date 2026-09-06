//! tests/unit_tests/renderer/deferred_path_geometry_test.cpp at 52662585.
//!
//! A host RawPath re-enters the recorder through the per-verb builders, so they
//! must drop the same zero-length segments as RiveRenderPath. No GPU.

use super::super::deferred_session::DeferredSession;
use nuxie_render_api::{Factory, PathVerb, RawPath};

#[test]
fn recorded_per_verb_geometry_drops_empty_segments() {
    let mut session = DeferredSession::with_caps(Default::default());
    let paint = session.make_render_paint();
    let mut path = session.make_empty_render_path();

    // A closed contour of coincident points: what a zero-sized shape hands us.
    let mut degenerate = RawPath::new();
    degenerate.move_to(304.0, 160.0);
    degenerate.line_to(304.0, 160.0);
    degenerate.line_to(304.0, 160.0);
    degenerate.line_to(304.0, 160.0);
    degenerate.close();
    // RawPath::addTo fans this fixture out to the per-verb builders directly.
    let mut points = degenerate.points().iter();
    for verb in degenerate.verbs() {
        match verb {
            PathVerb::Move => {
                let point = points.next().unwrap();
                path.move_to(point.x, point.y);
            }
            PathVerb::Line => {
                let point = points.next().unwrap();
                path.line_to(point.x, point.y);
            }
            PathVerb::Close => path.close(),
            _ => unreachable!("this upstream fixture contains only move, line, and close"),
        }
    }

    // Draws flush the pending per-verb geometry into the stream.
    session
        .screen_renderer(0)
        .borrow_mut()
        .draw_path(path.as_ref(), paint.as_ref());

    // This path's verbs are the only blob recorded, and lead it: move then
    // close, with the three empty lines gone.
    let commands = session.command_buffer();
    let commands = commands.lock().unwrap();
    let blobs = commands.blob_bytes();
    assert!(blobs.len() >= 2);
    assert_eq!(blobs[0], PathVerb::Move as u8);
    assert_eq!(blobs[1], PathVerb::Close as u8);
}
