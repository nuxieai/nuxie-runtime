//! tests/gm/ore_gm_sink.hpp at 6a2e3ab7.
use super::ore_gm_helper::*;
use crate::deferred::cmd::{deferred_replayer::DeferredFrameSink, render_replay::RendererOwner};
use nuxie_ore_metal::context::RenderTargetInfo;

/// A GM is handed an already-open screen renderer; replay must not replace it.
pub(super) struct GMFrameSink<'a> {
    pub host: &'a mut GmHost,
    pub target: Option<RenderTargetInfo>,
}
impl DeferredFrameSink for GMFrameSink<'_> {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.host.factory.persistent_context().unwrap()
    }
    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.host.factory.persistent_context()
    }
    fn begin_screen_frame(&mut self, target: u64) -> Option<RendererOwner> {
        assert_eq!(target, 0);
        Some(self.host.screen())
    }
    fn begin_ore_frame(&mut self) {
        self.host.begin_ore();
    }
    fn end_ore_frame(&mut self) {
        self.host.end_ore();
    }
    fn target_render_target(&mut self) -> Option<RenderTargetInfo> {
        self.target.clone()
    }
}
