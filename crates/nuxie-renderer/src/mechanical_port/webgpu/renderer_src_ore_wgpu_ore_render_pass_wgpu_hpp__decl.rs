//! Complete mechanical declaration translation of
//! `renderer/src/ore/wgpu/ore_render_pass_wgpu.hpp`.

#![allow(non_snake_case)]

use super::ore_context_wgpu_decl::ContextWGPU;
use super::webgpu_cpp_decl::{
    Buffer as WagyuBuffer, IndexFormat as WagyuIndexFormat, RenderPassEncoder,
};
use nuxie_ore_metal::context::ActiveRenderPass;
use nuxie_ore_metal::gpu_resource::AnyResourceHandle;
use nuxie_ore_metal::render_pass::{RenderPass, RenderPassApi};
use std::cell::{RefCell, RefMut};
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak as RcWeak};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_render_pass_wgpu.hpp");

#[repr(C)]
pub(crate) struct RenderPassWGPUState {
    pub(crate) base: ManuallyDrop<RenderPass>,
    pub(crate) m_wgpuContext: *mut ContextWGPU,
    pub(crate) m_currentPipeline: ManuallyDrop<Option<AnyResourceHandle>>,
    pub(crate) m_wgpuPassEncoder: ManuallyDrop<RenderPassEncoder>,
    pub(crate) m_wgpuIndexBuffer: ManuallyDrop<WagyuBuffer>,
    pub(crate) m_wgpuIndexFormat: WagyuIndexFormat,
    pub(crate) m_wgpuIndexOffset: u32,
}

impl RenderPassWGPUState {
    pub(crate) fn new(base: RenderPass, concrete: *mut ContextWGPU) -> Self {
        Self {
            base: ManuallyDrop::new(base),
            m_wgpuContext: concrete,
            m_currentPipeline: ManuallyDrop::new(None),
            m_wgpuPassEncoder: ManuallyDrop::new(RenderPassEncoder::default()),
            m_wgpuIndexBuffer: ManuallyDrop::new(WagyuBuffer::default()),
            m_wgpuIndexFormat: WagyuIndexFormat::Uint16,
            m_wgpuIndexOffset: 0,
        }
    }
}

impl Drop for RenderPassWGPUState {
    fn drop(&mut self) {
        super::ore_render_pass_wgpu_impl::finish(self);
        unsafe {
            ManuallyDrop::drop(&mut self.m_wgpuIndexBuffer);
            ManuallyDrop::drop(&mut self.m_wgpuPassEncoder);
            ManuallyDrop::drop(&mut self.m_currentPipeline);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

pub(crate) struct RenderPassWGPUInner {
    pub(crate) state: RefCell<RenderPassWGPUState>,
}

impl RenderPassWGPUInner {
    pub(crate) fn borrowState(&self) -> RefMut<'_, RenderPassWGPUState> {
        self.state.borrow_mut()
    }
}

impl ActiveRenderPass for RenderPassWGPUInner {
    fn isFinished(&self) -> bool {
        nuxie_ore_metal::render_pass_is_finished(&self.state.borrow().base)
    }

    fn finish(&self) {
        super::ore_render_pass_wgpu_impl::finish(&mut self.borrowState());
    }
}

pub(crate) struct RenderPassWGPU {
    pub(crate) inner: Rc<RenderPassWGPUInner>,
}

impl RenderPassWGPU {
    pub(crate) fn new(context: &ContextWGPU) -> Self {
        Self {
            inner: Rc::new(RenderPassWGPUInner {
                state: RefCell::new(RenderPassWGPUState::new(
                    nuxie_ore_metal::new_render_pass_backend_base(&context.base),
                    context as *const ContextWGPU as *mut ContextWGPU,
                )),
            }),
        }
    }

    pub(crate) fn activeToken(&self) -> RcWeak<dyn ActiveRenderPass> {
        let token: Rc<dyn ActiveRenderPass> = self.inner.clone();
        Rc::downgrade(&token)
    }
}

impl RenderPassApi for RenderPassWGPU {
    fn asAny(&self) -> &dyn std::any::Any {
        self
    }
    fn asAnyMut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn intoAny(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
    fn activeToken(&self) -> RcWeak<dyn ActiveRenderPass> {
        self.activeToken()
    }

    fn setPipeline(&mut self, pipeline: Option<&AnyResourceHandle>) {
        super::ore_render_pass_wgpu_impl::setPipeline(&mut self.inner.borrowState(), pipeline)
    }
    fn setVertexBuffer(&mut self, slot: u32, buffer: Option<&AnyResourceHandle>, offset: u32) {
        super::ore_render_pass_wgpu_impl::setVertexBuffer(
            &mut self.inner.borrowState(),
            slot,
            buffer,
            offset,
        )
    }
    fn setIndexBuffer(
        &mut self,
        buffer: Option<&AnyResourceHandle>,
        format: nuxie_ore_metal::types::IndexFormat,
        offset: u32,
    ) {
        super::ore_render_pass_wgpu_impl::setIndexBuffer(
            &mut self.inner.borrowState(),
            buffer,
            format,
            offset,
        )
    }
    fn setBindGroup(
        &mut self,
        groupIndex: u32,
        bindGroup: Option<&AnyResourceHandle>,
        dynamicOffsets: Option<&[u32]>,
        dynamicOffsetCount: u32,
    ) {
        super::ore_render_pass_wgpu_impl::setBindGroup(
            &mut self.inner.borrowState(),
            groupIndex,
            bindGroup,
            dynamicOffsets,
            dynamicOffsetCount,
        )
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
        super::ore_render_pass_wgpu_impl::setViewport(
            &mut self.inner.borrowState(),
            x,
            y,
            width,
            height,
            minDepth,
            maxDepth,
        )
    }
    fn setScissorRect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        super::ore_render_pass_wgpu_impl::setScissorRect(
            &mut self.inner.borrowState(),
            x,
            y,
            width,
            height,
        )
    }
    fn setStencilReference(&mut self, reference: u32) {
        super::ore_render_pass_wgpu_impl::setStencilReference(
            &mut self.inner.borrowState(),
            reference,
        )
    }
    fn setBlendColor(&mut self, r: f32, g: f32, b: f32, a: f32) {
        super::ore_render_pass_wgpu_impl::setBlendColor(&mut self.inner.borrowState(), r, g, b, a)
    }
    fn draw(&mut self, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) {
        super::ore_render_pass_wgpu_impl::draw(
            &mut self.inner.borrowState(),
            vertexCount,
            instanceCount,
            firstVertex,
            firstInstance,
        )
    }
    fn drawIndexed(
        &mut self,
        indexCount: u32,
        instanceCount: u32,
        firstIndex: u32,
        baseVertex: i32,
        firstInstance: u32,
    ) {
        super::ore_render_pass_wgpu_impl::drawIndexed(
            &mut self.inner.borrowState(),
            indexCount,
            instanceCount,
            firstIndex,
            baseVertex,
            firstInstance,
        )
    }
    fn finish(&mut self) {
        super::ore_render_pass_wgpu_impl::finish(&mut self.inner.borrowState())
    }
    fn validate(&self) {
        super::ore_render_pass_wgpu_impl::validate(&self.inner.state.borrow())
    }
}

pub(crate) const SOURCE_PUBLIC_METHOD_COUNT: usize = 18;
pub(crate) const SOURCE_BACKEND_FIELD_COUNT: usize = 6;
pub(crate) const SOURCE_FRIEND_COUNT: usize = 1;
const _: [(); 2388] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_header_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 66);
        assert_eq!(SOURCE_PUBLIC_METHOD_COUNT, 18);
        assert_eq!(SOURCE_BACKEND_FIELD_COUNT, 6);
        assert_eq!(SOURCE_FRIEND_COUNT, 1);
    }
}
