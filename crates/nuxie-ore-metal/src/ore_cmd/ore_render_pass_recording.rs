//! renderer/ore/cmd/ore_render_pass_recording.hpp at 966499ff.
#![allow(non_snake_case)]
use super::{
    ore_command_buffer::SharedOreCommandBuffer,
    ore_commands::*,
    ore_deferred_resource::{
        DeferredBindGroup, DeferredBuffer, DeferredPipeline, DeferredTextureView,
    },
    ore_make_recording::encodePods,
};
use crate::{
    context::{ActiveRenderPass, Context},
    gpu_resource::AnyResourceHandle,
    render_pass::{RenderPass, RenderPassApi},
    types::*,
};
use std::{
    any::Any,
    cell::RefCell,
    rc::{Rc, Weak},
};

struct RecordingState {
    base: RefCell<RenderPass>,
    cmd: SharedOreCommandBuffer,
}
impl ActiveRenderPass for RecordingState {
    fn isFinished(&self) -> bool {
        self.base.borrow().isFinished()
    }
    fn finish(&self) {
        let mut base = self.base.borrow_mut();
        if base.m_finished {
            return;
        }
        self.cmd.borrow_mut().appendOpcode(CommandType::finish);
        base.m_finished = true;
        for group in &mut base.m_boundGroups {
            *group = None;
        }
    }
}
pub struct RenderPassRecording {
    state: Rc<RecordingState>,
}
impl RenderPassRecording {
    pub fn new(
        context: Option<&Context>,
        cmd: SharedOreCommandBuffer,
        desc: &RenderPassDesc<'_>,
    ) -> Self {
        let mut base = match context {
            Some(c) => crate::new_render_pass_backend_base(c),
            None => crate::new_render_pass_backend_base_without_context(),
        };
        base.populateAttachmentMetadata(desc);
        let out = Self {
            state: Rc::new(RecordingState {
                base: RefCell::new(base),
                cmd,
            }),
        };
        let mut begin = BeginRenderPassCmd {
            colorCount: desc.colorCount,
            ..Default::default()
        };
        for i in 0..desc.colorCount.min(4) as usize {
            let src = &desc.colorAttachments[i];
            begin.colors[i] = ColorAttachmentPOD {
                view: out.idOf(src.view),
                resolveTarget: out.idOf(src.resolveTarget),
                clearR: src.clearColor.r,
                clearG: src.clearColor.g,
                clearB: src.clearColor.b,
                clearA: src.clearColor.a,
                loadOp: src.loadOp,
                storeOp: src.storeOp,
                pad: [0; 2],
            };
        }
        let ds = &desc.depthStencil;
        begin.depthStencil = DepthStencilAttachmentPOD {
            view: out.idOf(ds.view),
            depthClearValue: ds.depthClearValue,
            stencilClearValue: ds.stencilClearValue,
            depthLoadOp: ds.depthLoadOp,
            depthStoreOp: ds.depthStoreOp,
            stencilLoadOp: ds.stencilLoadOp,
            stencilStoreOp: ds.stencilStoreOp,
        };
        out.state
            .cmd
            .borrow_mut()
            .append(CommandType::beginRenderPass, &begin);
        out
    }
    pub fn isFinished(&self) -> bool {
        self.state.isFinished()
    }
    fn idOf(&self, r: Option<&AnyResourceHandle>) -> u32 {
        if let Some(r) = r {
            macro_rules! cast {($($t:ty),*)=>{$(if let Some(d)=r.downcast_ref::<$t>() {return d.clientHandle();})*};}
            cast!(
                DeferredBuffer,
                DeferredPipeline,
                DeferredTextureView,
                DeferredBindGroup
            );
        }
        self.state.cmd.borrow_mut().capture(r)
    }
}
impl RenderPassApi for RenderPassRecording {
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
        let token: Rc<dyn ActiveRenderPass> = self.state.clone();
        Rc::downgrade(&token)
    }
    fn setPipeline(&mut self, pipeline: Option<&AnyResourceHandle>) {
        if !self
            .state
            .base
            .borrow()
            .checkPipelineCompat(pipeline.and_then(AnyResourceHandle::pipelineBase))
        {
            return;
        }
        let pipeline = self.idOf(pipeline);
        self.state
            .cmd
            .borrow_mut()
            .append(CommandType::setPipeline, &SetPipelineCmd { pipeline });
    }
    fn setVertexBuffer(&mut self, slot: u32, buffer: Option<&AnyResourceHandle>, offset: u32) {
        let buffer = self.idOf(buffer);
        self.state.cmd.borrow_mut().append(
            CommandType::setVertexBuffer,
            &SetVertexBufferCmd {
                slot,
                buffer,
                offset,
            },
        );
    }
    fn setIndexBuffer(
        &mut self,
        buffer: Option<&AnyResourceHandle>,
        format: IndexFormat,
        offset: u32,
    ) {
        let buffer = self.idOf(buffer);
        self.state.cmd.borrow_mut().append(
            CommandType::setIndexBuffer,
            &SetIndexBufferCmd {
                buffer,
                offset,
                format,
                pad: [0; 3],
            },
        );
    }
    fn setBindGroup(
        &mut self,
        groupIndex: u32,
        bindGroup: Option<&AnyResourceHandle>,
        dynamicOffsets: Option<&[u32]>,
        dynamicOffsetCount: u32,
    ) {
        if groupIndex < kMaxBindGroups {
            self.state.base.borrow_mut().m_boundGroups[groupIndex as usize] = bindGroup.cloned();
        }
        let bindGroup = self.idOf(bindGroup);
        let mut cmd = self.state.cmd.borrow_mut();
        let dynamicOffsetStart = if dynamicOffsetCount > 0 {
            cmd.append_blob(&encodePods(
                &dynamicOffsets.expect("nonempty dynamic offsets")[..dynamicOffsetCount as usize],
            ))
        } else {
            0
        };
        cmd.append(
            CommandType::setBindGroup,
            &SetBindGroupCmd {
                groupIndex,
                bindGroup,
                dynamicOffsetStart,
                dynamicOffsetCount,
                pad: 0,
            },
        );
    }
    fn setViewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        minDepth: f32,
        maxDepth: f32,
    ) {
        self.state.cmd.borrow_mut().append(
            CommandType::setViewport,
            &SetViewportCmd {
                x,
                y,
                width,
                height,
                minDepth,
                maxDepth,
            },
        );
    }
    fn setScissorRect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.state.cmd.borrow_mut().append(
            CommandType::setScissorRect,
            &SetScissorRectCmd {
                x,
                y,
                width,
                height,
            },
        );
    }
    fn setStencilReference(&mut self, reference: u32) {
        self.state.cmd.borrow_mut().append(
            CommandType::setStencilReference,
            &SetStencilReferenceCmd { reference },
        );
    }
    fn setBlendColor(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.state
            .cmd
            .borrow_mut()
            .append(CommandType::setBlendColor, &SetBlendColorCmd { r, g, b, a });
    }
    fn draw(&mut self, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) {
        self.state.cmd.borrow_mut().append(
            CommandType::draw,
            &DrawCmd {
                vertexCount,
                instanceCount,
                firstVertex,
                firstInstance,
            },
        );
    }
    fn drawIndexed(
        &mut self,
        indexCount: u32,
        instanceCount: u32,
        firstIndex: u32,
        baseVertex: i32,
        firstInstance: u32,
    ) {
        self.state.cmd.borrow_mut().append(
            CommandType::drawIndexed,
            &DrawIndexedCmd {
                indexCount,
                instanceCount,
                firstIndex,
                baseVertex,
                firstInstance,
            },
        );
    }
    fn finish(&mut self) {
        self.state.finish();
    }
    fn validate(&self) {
        self.state.base.borrow().validate();
    }
}
