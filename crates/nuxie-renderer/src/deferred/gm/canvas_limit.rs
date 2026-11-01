//! A session's canvas limit on a live device, so the read reaches a product
//! factory and not only the null context of `deferred_canvas_limit_test`.
use super::ore_gm_helper::*;
use crate::deferred::cmd::deferred_session::DeferredSession;

#[test]
fn a_session_on_metal_reports_the_devices_canvas_limit() {
    let host = GmHost::with_screen(0xff000000, false);
    let device_limit = host.factory.borrow().source_capabilities().max_texture_size;
    // The device's family table says one of these, each past the 2048 default.
    assert!(matches!(device_limit, 8192 | 16384));

    let mut session = DeferredSession::with_caps(Default::default());
    session.bind_render_context(host.factory.persistent_context());
    assert_eq!(DeferredCanvasHost::max_canvas_dim(&session), device_limit);
}
