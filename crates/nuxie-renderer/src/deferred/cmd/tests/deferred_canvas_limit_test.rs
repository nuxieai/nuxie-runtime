//! A canvas's real limit is the render context's own. Replay's allocation reads
//! no limit, so a canvas's only guard is what asks for this number. Upstream's
//! `kMaxDim` is a rule of its cached layers and masks, so nothing here reads it.
use super::super::deferred_session::DeferredSession;
use super::render_context_null::*;
use super::*;

#[test]
fn a_session_reports_its_contexts_canvas_limit() {
    // Past the 2048 default and either side of WebGPU's 8192, so neither a
    // fixed number nor the default can pass.
    for limit in [4096, 8192, 16384] {
        let (factory, _, _) = observing_factory(8, 8);
        factory
            .borrow()
            .with_backend_mut_for_test(|backend| backend.set_max_texture_size(limit));
        // The context's own limit, the same number images and video frames meet.
        {
            let core = factory.borrow();
            assert_eq!(core.max_texture_size(), limit);
            assert!(core.check_texture_extent("canvas", limit, limit).is_ok());
            assert!(core
                .check_texture_extent("canvas", limit + 1, limit)
                .is_err());
        }

        let mut session = DeferredSession::with_caps(Default::default());
        session.bind_render_context(factory.persistent_context());
        assert_eq!(DeferredCanvasHost::max_canvas_dim(&session), limit);
        // What a picture reads: the host a factory hands out.
        let host = session.canvas_content_host().unwrap();
        assert_eq!(host.borrow().max_canvas_dim(), limit);
    }
}

// An unbound session makes no canvas, so the default stands.
#[test]
fn an_unbound_session_reports_the_default_canvas_limit() {
    let session = DeferredSession::with_caps(Default::default());
    assert_eq!(DeferredCanvasHost::max_canvas_dim(&session), 2048);
}
