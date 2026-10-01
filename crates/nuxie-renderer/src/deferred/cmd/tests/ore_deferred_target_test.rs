//! tests/unit_tests/renderer/ore_deferred_target_test.cpp at 6a2e3ab7.
use super::*;
use crate::deferred::{
    cmd::{
        deferred_replayer::{DeferredReplayer, snapshot_frame},
        deferred_session::DeferredSession,
    },
    ore::ore_deferred_context::DeferredOreContext,
};
use nuxie_ore_metal::{
    context::{ContextApi, RenderTargetInfo, ReplayCaps, TargetDesc},
    gpu_resource::AnyResourceHandle,
    ore_cmd::{
        ore_command_buffer::OreCommandReader, ore_commands::*,
        ore_deferred_resource::DeferredTextureView,
    },
    types::*,
};

fn target(width: u32, height: u32) -> TargetDesc {
    TargetDesc::color8(width, height, false)
}
fn target_wraps(ctx: &DeferredOreContext) -> Vec<WrapCanvasViewPOD> {
    let stream = ctx.stream();
    let stream = stream.borrow();
    let mut reader = OreCommandReader::new(stream.command_bytes(), stream.blob_bytes());
    let mut wraps = Vec::new();
    while let Some(kind) = reader.next::<CommandType>() {
        match kind {
            CommandType::wrapCanvasView => {
                let pod: WrapCanvasViewPOD = reader.read();
                if pod.mode == WrapCanvasViewMode::targetView as u32 {
                    wraps.push(pod);
                }
            }
            CommandType::beginRenderPass => {
                let _: BeginRenderPassCmd = reader.read();
            }
            CommandType::finish => {}
            CommandType::destroyResource => {
                let _: DestroyResourcePOD = reader.read();
            }
            _ => panic!("unexpected command in a target only stream"),
        }
    }
    wraps
}
fn draw_into(ctx: &mut dyn ContextApi, view: &AnyResourceHandle) -> bool {
    let mut desc = RenderPassDesc::default();
    desc.colorAttachments[0].view = Some(view);
    desc.colorAttachments[0].loadOp = LoadOp::clear;
    desc.colorAttachments[0].storeOp = StoreOp::store;
    desc.colorCount = 1;
    let Some(mut pass) = ctx.beginRenderPass(&desc, None) else {
        return false;
    };
    pass.finish();
    true
}
fn id_of(view: &AnyResourceHandle) -> u32 {
    view.downcast_ref::<DeferredTextureView>()
        .unwrap()
        .clientHandle()
}
fn same(a: &AnyResourceHandle, b: &AnyResourceHandle) -> bool {
    a.allocation_identity() == b.allocation_identity()
}
fn base(view: &AnyResourceHandle) -> &nuxie_ore_metal::texture::TextureView {
    &view.downcast_ref::<DeferredTextureView>().unwrap().base
}

struct PreserveSink {
    inner: TestSink,
    preserved: Vec<bool>,
    has_ore: bool,
    screen_width: u32,
    ore: OreContextHandle,
}
impl Default for PreserveSink {
    fn default() -> Self {
        Self {
            inner: TestSink::default(),
            preserved: Vec::new(),
            has_ore: true,
            screen_width: 64,
            ore: Rc::new(RefCell::new(DeferredOreContext::new(ReplayCaps::default()))),
        }
    }
}
impl DeferredFrameSink for PreserveSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.inner.factory()
    }
    fn begin_screen_frame(&mut self, target: u64) -> Option<RendererOwner> {
        self.inner.begin_screen_frame(target)
    }
    fn set_target_preserved(&mut self, value: bool) {
        self.preserved.push(value);
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        self.has_ore.then(|| self.ore.clone())
    }
    fn target_render_target(&mut self) -> Option<RenderTargetInfo> {
        Some(RenderTargetInfo {
            target: std::ptr::null_mut(),
            width: self.screen_width,
            height: 64,
            owner: None,
        })
    }
}

#[test]
fn a_target_view_claims_its_replay_slot_when_it_is_made() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    ctx.setTarget(target(64, 32));
    let view = ctx.targetView().unwrap();
    assert_eq!(base(&view).width(), 64);
    assert_eq!(base(&view).height(), 32);
    let wraps = target_wraps(&ctx);
    assert_eq!(wraps.len(), 1);
    assert_eq!(wraps[0].id, id_of(&view));
    assert!(same(&ctx.targetView().unwrap(), &view));
    assert_eq!(target_wraps(&ctx).len(), 1);
}
#[test]
fn a_frame_wraps_the_target_once_however_many_passes_draw_into_it() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    ctx.setTarget(target(64, 64));
    let view = ctx.targetView().unwrap();
    for _ in 0..3 {
        assert!(draw_into(&mut ctx, &view));
    }
    assert_eq!(target_wraps(&ctx).len(), 1);
    assert!(ctx.targetDrawn());
    ctx.resetFrame();
    for _ in 0..3 {
        assert!(draw_into(&mut ctx, &view));
    }
    let wraps = target_wraps(&ctx);
    assert_eq!(wraps.len(), 1);
    assert_eq!(wraps[0].id, id_of(&view));
    assert!(ctx.targetDrawn());
    ctx.resetFrame();
    assert!(same(&ctx.targetView().unwrap(), &view));
    assert!(target_wraps(&ctx).is_empty());
    assert!(!ctx.targetDrawn());
}
#[test]
fn a_target_changing_size_or_format_mints_a_new_view() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    ctx.setTarget(target(64, 64));
    let first = ctx.targetView().unwrap();
    ctx.setTarget(target(64, 64));
    assert!(same(&ctx.targetView().unwrap(), &first));
    ctx.setTarget(target(128, 64));
    let resized = ctx.targetView().unwrap();
    assert!(!same(&resized, &first));
    assert_eq!(base(&resized).width(), 128);
    ctx.setTarget(TargetDesc {
        width: 128,
        height: 64,
        format: TextureFormat::bgra8unorm,
        sampleCount: 4,
    });
    let reformatted = ctx.targetView().unwrap();
    assert!(!same(&reformatted, &resized));
    assert_eq!(
        base(&reformatted).texture().format(),
        Some(TextureFormat::bgra8unorm)
    );
    assert_eq!(base(&reformatted).texture().sampleCount(), Some(4));
}
#[test]
fn with_no_target_declared_there_is_no_view_and_passes_drop() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    assert!(ctx.targetView().is_none());
    ctx.setTarget(target(64, 64));
    let stale = ctx.targetView().unwrap();
    ctx.resetFrame();
    ctx.setTarget(TargetDesc::default());
    assert!(ctx.targetView().is_none());
    assert!(!draw_into(&mut ctx, &stale));
    assert!(target_wraps(&ctx).is_empty());
    assert!(!ctx.targetDrawn());
    ctx.setTarget(target(64, 64));
    assert!(same(&ctx.targetView().unwrap(), &stale));
    assert!(draw_into(&mut ctx, &stale));
    ctx.resetFrame();
    ctx.setTarget(target(128, 64));
    assert!(ctx.targetView().is_some());
    assert!(!draw_into(&mut ctx, &stale));
    assert!(!ctx.lastError().is_empty());
}
#[test]
fn replay_keeps_the_target_only_in_frames_a_script_drew_into_it() {
    let mut session = DeferredSession::with_caps(ReplayCaps::default());
    session.ore_context.borrow_mut().setTarget(target(64, 64));
    let view = session.ore_context.borrow_mut().targetView().unwrap();
    let mut sink = PreserveSink::default();
    let mut replayer = DeferredReplayer::default();
    assert!(draw_into(&mut *session.ore_context.borrow_mut(), &view));
    replayer.replay_frame(&snapshot_frame(&mut session), &mut sink);
    session.reset_frame();
    replayer.replay_frame(&snapshot_frame(&mut session), &mut sink);
    session.reset_frame();
    assert!(draw_into(&mut *session.ore_context.borrow_mut(), &view));
    replayer.replay_frame(&snapshot_frame(&mut session), &mut sink);
    assert_eq!(sink.preserved, [true, false, true]);
}
#[test]
fn replay_clears_the_target_when_no_ore_context_can_draw_it() {
    let mut session = DeferredSession::with_caps(ReplayCaps::default());
    session.ore_context.borrow_mut().setTarget(target(64, 64));
    let view = session.ore_context.borrow_mut().targetView().unwrap();
    let mut sink = PreserveSink {
        has_ore: false,
        ..Default::default()
    };
    let mut replayer = DeferredReplayer::default();
    assert!(draw_into(&mut *session.ore_context.borrow_mut(), &view));
    replayer.replay_frame(&snapshot_frame(&mut session), &mut sink);
    assert_eq!(sink.preserved, [false]);
}
#[test]
fn replay_clears_a_target_resized_since_the_script_drew_into_it() {
    let mut session = DeferredSession::with_caps(ReplayCaps::default());
    session.ore_context.borrow_mut().setTarget(target(64, 64));
    let view = session.ore_context.borrow_mut().targetView().unwrap();
    assert!(draw_into(&mut *session.ore_context.borrow_mut(), &view));
    let frame = snapshot_frame(&mut session);
    let mut sink = PreserveSink {
        screen_width: 128,
        ..Default::default()
    };
    let mut replayer = DeferredReplayer::default();
    replayer.replay_frame(&frame, &mut sink);
    assert_eq!(sink.preserved, [false]);
}
#[test]
fn the_target_view_cannot_be_sampled() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    ctx.setTarget(target(64, 64));
    let view = ctx.targetView().unwrap();
    let textures = [TexEntry {
        view: Some(&view),
        ..Default::default()
    }];
    let desc = BindGroupDesc {
        textures: &textures,
        textureCount: 1,
        ..Default::default()
    };
    assert!(ctx.makeBindGroup(&desc).is_none());
    assert!(!ctx.lastError().is_empty());
}
