//! Nested cases from tests/unit_tests/renderer/ore_render_pass_recording_test.cpp at 65638e57.
#![allow(non_snake_case)]
use super::ore_deferred_context::DeferredOreContext;
use nuxie_ore_metal::{
    buffer::BufferApi,
    context::{ActiveRenderPass, Context, ContextApi, FrameDescriptor, ShaderTarget},
    gpu_resource::AnyResourceHandle,
    ore_cmd::{
        ore_command_buffer::{OreCommandBuffer, OreCommandReader},
        ore_commands::*,
        ore_deferred_render_pass::beginRecordedRenderPass,
        ore_deferred_resource::DeferredBuffer,
        ore_replay::replayCommandBuffer,
    },
    render_pass::RenderPassApi,
    types::*,
};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    ffi::c_void,
    rc::{Rc, Weak},
};

#[derive(Debug, PartialEq)]
struct Op(CommandType, u32);
fn begin() -> Op {
    Op(CommandType::beginRenderPass, 0)
}
fn draw(n: u32) -> Op {
    Op(CommandType::draw, n)
}
fn finish() -> Op {
    Op(CommandType::finish, 0)
}
fn ops_of(stream: &OreCommandBuffer) -> Vec<Op> {
    let mut reader = OreCommandReader::new(stream.command_bytes(), stream.blob_bytes());
    let mut ops = Vec::new();
    while let Some(kind) = reader.next::<CommandType>() {
        let value = match kind {
            CommandType::draw => reader.read::<DrawCmd>().vertexCount,
            CommandType::setViewport => reader.read::<SetViewportCmd>().x as u32,
            CommandType::bufferUpdate => reader.read::<BufferUpdatePOD>().offset,
            _ => {
                reader.skip(ore_payload_size_of(kind));
                0
            }
        };
        ops.push(Op(kind, value));
    }
    ops
}
fn recorded_ops(ctx: &DeferredOreContext) -> Vec<Op> {
    ops_of(&ctx.stream().borrow())
}
fn begin_deferred(ctx: &mut DeferredOreContext) -> Box<dyn RenderPassApi> {
    ctx.beginRenderPass(&RenderPassDesc::default(), None)
        .unwrap()
}
fn is_finished(pass: &dyn RenderPassApi) -> bool {
    pass.activeToken().upgrade().unwrap().isFinished()
}
fn update_buffer(buffer: &AnyResourceHandle) {
    buffer
        .downcast_ref::<DeferredBuffer>()
        .unwrap()
        .update(&[0; 4], 4, 8)
        .unwrap();
}

#[derive(Default)]
struct LiveLog {
    ops: Vec<Op>,
    open: usize,
    max_open: usize,
}
struct LivePassState {
    log: Rc<RefCell<LiveLog>>,
    finished: Cell<bool>,
}
impl ActiveRenderPass for LivePassState {
    fn isFinished(&self) -> bool {
        self.finished.get()
    }
    fn finish(&self) {
        if self.finished.replace(true) {
            return;
        }
        let mut log = self.log.borrow_mut();
        log.ops.push(finish());
        log.open -= 1;
    }
}
struct LivePass(Rc<LivePassState>);
impl Drop for LivePass {
    fn drop(&mut self) {
        self.0.finish();
    }
}
impl RenderPassApi for LivePass {
    fn asAny(&self) -> &dyn Any {
        self
    }
    fn asAnyMut(&mut self) -> &mut dyn Any {
        self
    }
    fn intoAny(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn activeToken(&self) -> Weak<dyn ActiveRenderPass> {
        let state: Rc<dyn ActiveRenderPass> = self.0.clone();
        Rc::downgrade(&state)
    }
    fn setPipeline(&mut self, _: Option<&AnyResourceHandle>) {}
    fn setVertexBuffer(&mut self, _: u32, _: Option<&AnyResourceHandle>, _: u32) {}
    fn setIndexBuffer(&mut self, _: Option<&AnyResourceHandle>, _: IndexFormat, _: u32) {}
    fn setBindGroup(&mut self, _: u32, _: Option<&AnyResourceHandle>, _: Option<&[u32]>, _: u32) {}
    fn setViewport(&mut self, x: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
        self.0
            .log
            .borrow_mut()
            .ops
            .push(Op(CommandType::setViewport, x as u32));
    }
    fn setScissorRect(&mut self, _: u32, _: u32, _: u32, _: u32) {}
    fn setStencilReference(&mut self, _: u32) {}
    fn setBlendColor(&mut self, _: f32, _: f32, _: f32, _: f32) {}
    fn draw(&mut self, n: u32, _: u32, _: u32, _: u32) {
        self.0.log.borrow_mut().ops.push(draw(n));
    }
    fn drawIndexed(&mut self, _: u32, _: u32, _: u32, _: i32, _: u32) {}
    fn finish(&mut self) {
        self.0.finish();
    }
    fn validate(&self) {}
}
struct LiveStubContext {
    base: Context,
    log: Rc<RefCell<LiveLog>>,
    frame_replay: bool,
}
impl LiveStubContext {
    fn new() -> Self {
        Self {
            base: nuxie_ore_metal::new_context_backend_base(Features::default(), None),
            log: Rc::new(RefCell::new(LiveLog::default())),
            frame_replay: false,
        }
    }
}
impl ContextApi for LiveStubContext {
    fn contextBase(&self) -> &Context {
        &self.base
    }
    fn features(&self) -> Features {
        self.base.features()
    }
    fn lastError(&self) -> String {
        self.base.lastError()
    }
    fn clearLastError(&self) {
        self.base.clearLastError();
    }
    fn setLastError(&self, message: &str) {
        self.base.setLastError(message);
    }
    fn usesDeferredFrameReplay(&self) -> bool {
        self.frame_replay
    }
    fn makeBuffer(&mut self, _: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeTexture(&mut self, _: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeTextureViewImpl(&mut self, _: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeSampler(&mut self, _: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeShaderModule(&mut self, _: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makeBindGroupLayout(&mut self, _: &BindGroupLayoutDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn makePipeline(
        &mut self,
        _: &PipelineDesc<'_>,
        _: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        None
    }
    fn makeBindGroup(&mut self, _: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        None
    }
    fn beginRenderPass(
        &mut self,
        _: &RenderPassDesc<'_>,
        _: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>> {
        {
            let mut log = self.log.borrow_mut();
            log.ops.push(begin());
            log.open += 1;
            log.max_open = log.max_open.max(log.open);
        }
        Some(Box::new(LivePass(Rc::new(LivePassState {
            log: self.log.clone(),
            finished: Cell::new(false),
        }))))
    }
    fn beginFrame(&mut self, _: &FrameDescriptor) {}
    fn endFrame(&mut self) {}
    fn waitForGPU(&mut self) {}
    unsafe fn wrapCanvasTexture(&mut self, _: *mut c_void) -> Option<AnyResourceHandle> {
        None
    }
    unsafe fn wrapRiveTexture(
        &mut self,
        _: *mut c_void,
        _: u32,
        _: u32,
    ) -> Option<AnyResourceHandle> {
        None
    }
    fn shaderTarget(&self) -> ShaderTarget {
        ShaderTarget::wgsl
    }
}

#[test]
fn a_pass_begun_inside_another_lands_ahead_of_it_whole() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let mut outer = begin_deferred(&mut ctx);
    outer.draw(1, 1, 0, 0);
    let mut inner = begin_deferred(&mut ctx);
    inner.draw(2, 1, 0, 0);
    inner.finish();
    outer.draw(3, 1, 0, 0);
    outer.finish();
    assert_eq!(
        recorded_ops(&ctx),
        [
            begin(),
            draw(2),
            finish(),
            begin(),
            draw(1),
            draw(3),
            finish()
        ]
    );
    assert!(!ctx.hasOpenRenderPasses());
}

#[test]
fn lifecycle_commands_between_the_two_begins_keep_their_place() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let mut outer = begin_deferred(&mut ctx);
    outer.draw(1, 1, 0, 0);
    let buffer = ctx
        .makeBuffer(&BufferDesc {
            usage: BufferUsage::vertex,
            size: 16,
            data: None,
            immutable: false,
            label: None,
        })
        .unwrap();
    update_buffer(&buffer);
    let mut inner = begin_deferred(&mut ctx);
    inner.draw(2, 1, 0, 0);
    inner.finish();
    outer.finish();
    assert_eq!(
        recorded_ops(&ctx),
        [
            Op(CommandType::makeBuffer, 0),
            Op(CommandType::bufferUpdate, 8),
            begin(),
            draw(2),
            finish(),
            begin(),
            draw(1),
            finish()
        ]
    );
}

#[test]
fn three_levels_of_nesting_settle_innermost_first() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let mut a = begin_deferred(&mut ctx);
    a.draw(1, 1, 0, 0);
    let mut b = begin_deferred(&mut ctx);
    b.draw(2, 1, 0, 0);
    let mut c = begin_deferred(&mut ctx);
    c.draw(3, 1, 0, 0);
    c.finish();
    b.draw(22, 1, 0, 0);
    b.finish();
    a.draw(11, 1, 0, 0);
    a.finish();
    assert_eq!(
        recorded_ops(&ctx),
        [
            begin(),
            draw(3),
            finish(),
            begin(),
            draw(2),
            draw(22),
            finish(),
            begin(),
            draw(1),
            draw(11),
            finish()
        ]
    );
}

#[test]
fn sibling_nested_passes_keep_their_order_ahead_of_the_enclosing_one() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let mut outer = begin_deferred(&mut ctx);
    outer.draw(1, 1, 0, 0);
    let mut first = begin_deferred(&mut ctx);
    first.draw(2, 1, 0, 0);
    first.finish();
    outer.draw(11, 1, 0, 0);
    let mut second = begin_deferred(&mut ctx);
    second.draw(3, 1, 0, 0);
    second.finish();
    outer.draw(111, 1, 0, 0);
    outer.finish();
    assert_eq!(
        recorded_ops(&ctx),
        [
            begin(),
            draw(2),
            finish(),
            begin(),
            draw(3),
            finish(),
            begin(),
            draw(1),
            draw(11),
            draw(111),
            finish()
        ]
    );
}

#[test]
fn finishing_the_enclosing_pass_finishes_the_nested_one_first() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let mut outer = begin_deferred(&mut ctx);
    outer.draw(1, 1, 0, 0);
    let mut inner = begin_deferred(&mut ctx);
    inner.draw(2, 1, 0, 0);
    outer.finish();
    assert!(is_finished(inner.as_ref()));
    assert!(!ctx.hasOpenRenderPasses());
    assert_eq!(
        recorded_ops(&ctx),
        [begin(), draw(2), finish(), begin(), draw(1), finish()]
    );
}

#[test]
fn a_destroyed_unfinished_pass_finishes_and_settles() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let mut outer = begin_deferred(&mut ctx);
    outer.draw(1, 1, 0, 0);
    {
        let mut inner = begin_deferred(&mut ctx);
        inner.draw(2, 1, 0, 0);
    }
    outer.finish();
    assert_eq!(
        recorded_ops(&ctx),
        [begin(), draw(2), finish(), begin(), draw(1), finish()]
    );
}

#[test]
fn a_write_during_the_enclosing_pass_reaches_its_earlier_draws() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let buffer = ctx
        .makeBuffer(&BufferDesc {
            usage: BufferUsage::vertex,
            size: 16,
            data: None,
            immutable: false,
            label: None,
        })
        .unwrap();
    let mut outer = begin_deferred(&mut ctx);
    outer.setVertexBuffer(0, Some(&buffer), 0);
    outer.draw(1, 1, 0, 0);
    update_buffer(&buffer);
    let mut inner = begin_deferred(&mut ctx);
    inner.draw(2, 1, 0, 0);
    inner.finish();
    outer.draw(3, 1, 0, 0);
    outer.finish();
    assert_eq!(
        recorded_ops(&ctx),
        [
            Op(CommandType::makeBuffer, 0),
            Op(CommandType::bufferUpdate, 8),
            begin(),
            draw(2),
            finish(),
            begin(),
            Op(CommandType::setVertexBuffer, 0),
            draw(1),
            draw(3),
            finish()
        ]
    );
}

#[test]
fn a_pass_outliving_its_context_finishes_quietly() {
    let (mut outer, inner) = {
        let mut ctx = DeferredOreContext::fromReal(None);
        let mut outer = begin_deferred(&mut ctx);
        outer.draw(1, 1, 0, 0);
        let mut inner = begin_deferred(&mut ctx);
        inner.draw(2, 1, 0, 0);
        (outer, inner)
    };
    assert!(is_finished(outer.as_ref()));
    assert!(is_finished(inner.as_ref()));
    outer.finish();
    drop(inner);
    drop(outer);
    let mut inline = {
        let ctx = Rc::new(RefCell::new(LiveStubContext::new()));
        let mut pass = beginRecordedRenderPass(ctx.clone(), &RenderPassDesc::default()).unwrap();
        pass.draw(1, 1, 0, 0);
        pass
    };
    assert!(is_finished(inline.as_ref()));
    inline.finish();
    drop(inline);
}

#[test]
fn reclaiming_from_a_token_finishes_only_the_passes_begun_after_it() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let outer = begin_deferred(&mut ctx);
    let token = ctx.nextRenderPassToken();
    let first = begin_deferred(&mut ctx);
    let second = begin_deferred(&mut ctx);
    assert_eq!(ctx.finishOpenRenderPassesFrom(token), 2);
    assert!(is_finished(first.as_ref()));
    assert!(is_finished(second.as_ref()));
    assert!(!is_finished(outer.as_ref()));
    assert!(ctx.hasOpenRenderPasses());
    assert_eq!(ctx.finishOpenRenderPassesFrom(token), 0);
    assert_eq!(ctx.finishOpenRenderPassesFrom(0), 1);
    assert!(is_finished(outer.as_ref()));
    assert!(!ctx.hasOpenRenderPasses());
}

#[test]
fn a_frame_reset_finishes_what_a_script_left_open() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let mut pass = begin_deferred(&mut ctx);
    pass.draw(1, 1, 0, 0);
    ctx.resetFrame();
    assert!(is_finished(pass.as_ref()));
    assert!(ctx.stream().borrow().empty());
    pass.finish();
    assert!(ctx.stream().borrow().empty());
}

#[test]
fn inline_passes_drain_nested_first_without_overlapping() {
    let ctx = Rc::new(RefCell::new(LiveStubContext::new()));
    let log = ctx.borrow().log.clone();
    let mut outer = beginRecordedRenderPass(ctx.clone(), &RenderPassDesc::default()).unwrap();
    outer.draw(1, 1, 0, 0);
    let mut inner = beginRecordedRenderPass(ctx.clone(), &RenderPassDesc::default()).unwrap();
    inner.draw(2, 1, 0, 0);
    inner.finish();
    assert_eq!(log.borrow().ops, [begin(), draw(2), finish()]);
    outer.draw(3, 1, 0, 0);
    outer.finish();
    assert_eq!(
        log.borrow().ops,
        [
            begin(),
            draw(2),
            finish(),
            begin(),
            draw(1),
            draw(3),
            finish()
        ]
    );
    assert_eq!(log.borrow().max_open, 1);
    assert!(!ctx.borrow().hasOpenRenderPasses());
}

#[test]
fn an_inline_pass_finished_by_its_enclosing_pass_still_drains() {
    let ctx = Rc::new(RefCell::new(LiveStubContext::new()));
    let log = ctx.borrow().log.clone();
    let mut outer = beginRecordedRenderPass(ctx.clone(), &RenderPassDesc::default()).unwrap();
    outer.draw(1, 1, 0, 0);
    let mut inner = beginRecordedRenderPass(ctx.clone(), &RenderPassDesc::default()).unwrap();
    inner.draw(2, 1, 0, 0);
    outer.finish();
    assert!(is_finished(inner.as_ref()));
    assert_eq!(
        log.borrow().ops,
        [begin(), draw(2), finish(), begin(), draw(1), finish()]
    );
    assert_eq!(log.borrow().max_open, 1);
}

#[test]
fn frame_replay_backends_settle_nested_passes_in_pending_frame() {
    let ctx = Rc::new(RefCell::new(LiveStubContext::new()));
    ctx.borrow_mut().frame_replay = true;
    ctx.borrow().setDeferredRecording(true);
    let log = ctx.borrow().log.clone();
    let mut outer = beginRecordedRenderPass(ctx.clone(), &RenderPassDesc::default()).unwrap();
    outer.draw(1, 1, 0, 0);
    let mut inner = beginRecordedRenderPass(ctx.clone(), &RenderPassDesc::default()).unwrap();
    inner.draw(2, 1, 0, 0);
    inner.finish();
    outer.draw(3, 1, 0, 0);
    outer.finish();
    assert!(log.borrow().ops.is_empty());
    let pending = ctx.borrow().pendingFrame();
    replayCommandBuffer(&mut *ctx.borrow_mut(), &pending.borrow(), None);
    assert_eq!(
        log.borrow().ops,
        [
            begin(),
            draw(2),
            finish(),
            begin(),
            draw(1),
            draw(3),
            finish()
        ]
    );
    assert_eq!(log.borrow().max_open, 1);
}
