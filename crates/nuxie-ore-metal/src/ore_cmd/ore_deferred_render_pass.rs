//! renderer/ore/cmd/ore_deferred_render_pass.hpp at e949498e.
#![allow(non_snake_case)]
use super::{
    ore_command_buffer::{OreCommandBuffer, SharedOreCommandBuffer},
    ore_render_pass_recording::RenderPassRecording,
    ore_replay::replayCommandBuffer,
};
use crate::{
    context::{ActiveRenderPass, ContextApi},
    gpu_resource::AnyResourceHandle,
    render_pass::RenderPassApi,
    types::*,
};
use std::{
    any::Any,
    cell::RefCell,
    rc::{Rc, Weak},
};

struct InlineState {
    recording: RefCell<RenderPassRecording>,
    buffer: SharedOreCommandBuffer,
    context: Weak<RefCell<dyn ContextApi>>,
}
impl ActiveRenderPass for InlineState {
    fn isFinished(&self) -> bool {
        self.recording.borrow().isFinished()
    }
    fn finish(&self) {
        if self.isFinished() {
            return;
        }
        // Latch before replay, which may reenter finishActiveRenderPass.
        self.recording.borrow_mut().finish();
        let context = self
            .context
            .upgrade()
            .expect("inline pass context outlives pass");
        replayCommandBuffer(&mut *context.borrow_mut(), &self.buffer.borrow(), None);
    }
}
pub struct InlineDeferredRenderPass {
    state: Rc<InlineState>,
}
impl InlineDeferredRenderPass {
    pub fn new(context: Rc<RefCell<dyn ContextApi>>, desc: &RenderPassDesc<'_>) -> Self {
        let buffer = Rc::new(RefCell::new(OreCommandBuffer::default()));
        let recording =
            RenderPassRecording::new(Some(context.borrow().contextBase()), buffer.clone(), desc);
        Self {
            state: Rc::new(InlineState {
                recording: RefCell::new(recording),
                buffer,
                context: Rc::downgrade(&context),
            }),
        }
    }
}
impl RenderPassApi for InlineDeferredRenderPass {
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
        let state: Rc<dyn ActiveRenderPass> = self.state.clone();
        Rc::downgrade(&state)
    }
    fn setPipeline(&mut self, p: Option<&AnyResourceHandle>) {
        self.state.recording.borrow_mut().setPipeline(p);
    }
    fn setVertexBuffer(&mut self, slot: u32, b: Option<&AnyResourceHandle>, offset: u32) {
        self.state
            .recording
            .borrow_mut()
            .setVertexBuffer(slot, b, offset);
    }
    fn setIndexBuffer(&mut self, b: Option<&AnyResourceHandle>, format: IndexFormat, offset: u32) {
        self.state
            .recording
            .borrow_mut()
            .setIndexBuffer(b, format, offset);
    }
    fn setBindGroup(
        &mut self,
        g: u32,
        b: Option<&AnyResourceHandle>,
        offsets: Option<&[u32]>,
        count: u32,
    ) {
        self.state
            .recording
            .borrow_mut()
            .setBindGroup(g, b, offsets, count);
    }
    fn setViewport(&mut self, x: f32, y: f32, w: f32, h: f32, min: f32, max: f32) {
        self.state
            .recording
            .borrow_mut()
            .setViewport(x, y, w, h, min, max);
    }
    fn setScissorRect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        self.state.recording.borrow_mut().setScissorRect(x, y, w, h);
    }
    fn setStencilReference(&mut self, r: u32) {
        self.state.recording.borrow_mut().setStencilReference(r);
    }
    fn setBlendColor(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.state.recording.borrow_mut().setBlendColor(r, g, b, a);
    }
    fn draw(&mut self, v: u32, i: u32, fv: u32, fi: u32) {
        self.state.recording.borrow_mut().draw(v, i, fv, fi);
    }
    fn drawIndexed(&mut self, i: u32, instances: u32, first: u32, base: i32, firstInstance: u32) {
        self.state
            .recording
            .borrow_mut()
            .drawIndexed(i, instances, first, base, firstInstance);
    }
    fn finish(&mut self) {
        self.state.finish();
    }
    fn validate(&self) {
        self.state.recording.borrow().validate();
    }
}
pub fn beginRenderPassRecordingOrImmediate(
    ctx: Rc<RefCell<dyn ContextApi>>,
    desc: &RenderPassDesc<'_>,
    outError: Option<&mut String>,
) -> Option<Box<dyn RenderPassApi>> {
    if ctx.borrow().deferredRecording() {
        if ctx.borrow().usesDeferredFrameReplay() {
            let context = ctx.borrow();
            return Some(Box::new(RenderPassRecording::new(
                Some(context.contextBase()),
                context.pendingFrame(),
                desc,
            )));
        }
        return Some(Box::new(InlineDeferredRenderPass::new(ctx, desc)));
    }
    ctx.borrow_mut().beginRenderPass(desc, outError)
}
