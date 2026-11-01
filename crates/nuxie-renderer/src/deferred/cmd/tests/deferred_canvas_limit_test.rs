//! A canvas's real limit is the render context's own, the number replay
//! allocates against. Upstream's `kMaxDim` is a rule of its cached layers and
//! masks, so nothing here reads it.
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
        // The context's own limit, read as replay reads it when it allocates.
        {
            let core = factory.borrow();
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
