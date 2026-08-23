//! Complete mechanical declaration translation of
//! `renderer/src/ore/gl/ore_render_pass_gl.hpp`.

#![allow(non_snake_case)]

use nuxie_ore_metal::context::{ActiveRenderPass, Context};
use nuxie_ore_metal::gpu_resource::AnyResourceHandle;
use nuxie_ore_metal::render_pass::{RenderPass, RenderPassApi};
use nuxie_ore_metal::types::IndexFormat;
use std::cell::{RefCell, RefMut};
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak as RcWeak};

use super::gles3_decl::GLExecutionStamp;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_gl_ore_render_pass_gl.hpp");

/// Exact source `RenderPassGL::GLResolveEntry` record.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GLResolveEntry {
    pub(crate) colorIndex: u32,
    pub(crate) resolveTarget: u32,
    pub(crate) resolveTex: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

/// Source-ordered state for the `RenderPass` base followed by every authored
/// `RenderPassGL` field. The previous-binding fields contain the actual
/// synchronous source `GLuint` values saved before this pass.
#[repr(C)]
pub(crate) struct RenderPassGLState {
    pub(crate) base: ManuallyDrop<RenderPass>,
    pub(crate) m_glFBO: u32,
    pub(crate) m_glVAO: u32,
    pub(crate) m_prevVAO: u32,
    pub(crate) m_prevFBO: u32,
    pub(crate) m_ownsFBO: bool,
    pub(crate) m_ownsVAO: bool,
    pub(crate) m_currentPipeline: ManuallyDrop<Option<AnyResourceHandle>>,
    pub(crate) m_viewportWidth: u32,
    pub(crate) m_viewportHeight: u32,
    pub(crate) m_maxSamplerSlot: u32,
    pub(crate) m_maxAttribSlot: u32,
    pub(crate) m_usedSamplers: bool,
    pub(crate) m_usedAttribs: bool,
    pub(crate) m_glIndexFormat: IndexFormat,
    pub(crate) m_glStencilRef: u32,
    pub(crate) m_glResolveCount: u32,
    pub(crate) m_glResolves: [GLResolveEntry; 4],
    /// Rust execution/lifetime sidecar after the complete source field prefix.
    /// The source default constructor is deliberately unstamped and inert;
    /// every context-created pass carries the concrete domain generation.
    pub(crate) rust_execution: Option<GLExecutionStamp>,
}

impl RenderPassGLState {
    fn fromBase(base: RenderPass, execution: Option<GLExecutionStamp>) -> Self {
        Self {
            base: ManuallyDrop::new(base),
            m_glFBO: 0,
            m_glVAO: 0,
            m_prevVAO: 0,
            m_prevFBO: 0,
            m_ownsFBO: false,
            m_ownsVAO: false,
            m_currentPipeline: ManuallyDrop::new(None),
            m_viewportWidth: 0,
            m_viewportHeight: 0,
            m_maxSamplerSlot: 0,
            m_maxAttribSlot: 0,
            m_usedSamplers: false,
            m_usedAttribs: false,
            m_glIndexFormat: IndexFormat::uint16,
            m_glStencilRef: 0,
            m_glResolveCount: 0,
            m_glResolves: [GLResolveEntry::default(); 4],
            rust_execution: execution,
        }
    }

    pub(crate) fn new(context: &Context, execution: GLExecutionStamp) -> Self {
        Self::fromBase(
            nuxie_ore_metal::new_render_pass_backend_base(context),
            Some(execution),
        )
    }

    #[cfg(test)]
    pub(crate) fn newUnstamped(context: &Context) -> Self {
        Self::fromBase(nuxie_ore_metal::new_render_pass_backend_base(context), None)
    }

    pub(crate) fn withoutContext() -> Self {
        Self::fromBase(
            nuxie_ore_metal::new_render_pass_backend_base_without_context(),
            None,
        )
    }

    pub(crate) fn executionStamp(&self) -> &GLExecutionStamp {
        self.rust_execution
            .as_ref()
            .expect("live RenderPassGL requires GL execution authority")
    }
}

impl Drop for RenderPassGLState {
    fn drop(&mut self) {
        // The C++ destructor calls finish only for a live, unfinished pass.
        if !nuxie_ore_metal::render_pass_is_finished(&self.base)
            && self.rust_execution.is_some()
        {
            let execution = self.executionStamp().clone();
            if execution
                .withDeleteCurrent(|| super::ore_render_pass_gl_impl::finish(self))
                .is_none()
            {
                // A lost generation cannot legally execute its numeric GL
                // names. Release the Rust/resource graph without GL calls.
                super::ore_render_pass_gl_impl::abandonAfterContextLoss(self);
            }
        }

        // C++ derived-field teardown precedes the inherited RenderPass base.
        unsafe {
            ManuallyDrop::drop(&mut self.m_currentPipeline);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

pub(crate) struct RenderPassGLInner {
    pub(crate) state: RefCell<RenderPassGLState>,
}

impl RenderPassGLInner {
    pub(crate) fn borrowState(&self) -> RefMut<'_, RenderPassGLState> {
        self.state.borrow_mut()
    }

    fn withCurrentState<R>(&self, callback: impl FnOnce(&mut RenderPassGLState) -> R) -> R {
        let execution = self.state.borrow().executionStamp().clone();
        execution.withCurrent(|| callback(&mut self.state.borrow_mut()))
    }
}

impl ActiveRenderPass for RenderPassGLInner {
    fn isFinished(&self) -> bool {
        nuxie_ore_metal::render_pass_is_finished(&self.state.borrow().base)
    }

    fn finish(&self) {
        self.withCurrentState(super::ore_render_pass_gl_impl::finish);
    }
}

/// Unique, thread-affine source render pass. The `Rc<RefCell<_>>` spelling is
/// deliberate: the source GL context is thread-affine and the context retains
/// only a non-owning active-pass token. This type is neither Send nor Sync.
pub(crate) struct RenderPassGL {
    pub(crate) inner: Rc<RenderPassGLInner>,
}

impl RenderPassGL {
    /// Source `RenderPassGL(Context*)` constructor.
    pub(crate) fn new(context: &Context, execution: GLExecutionStamp) -> Self {
        Self {
            inner: Rc::new(RenderPassGLInner {
                state: RefCell::new(RenderPassGLState::new(context, execution)),
            }),
        }
    }

    /// Source default constructor, whose inherited context pointer is null.
    pub(crate) fn withoutContext() -> Self {
        Self {
            inner: Rc::new(RenderPassGLInner {
                state: RefCell::new(RenderPassGLState::withoutContext()),
            }),
        }
    }

    pub(crate) fn activeToken(&self) -> RcWeak<dyn ActiveRenderPass> {
        let token: Rc<dyn ActiveRenderPass> = self.inner.clone();
        Rc::downgrade(&token)
    }
}

impl Default for RenderPassGL {
    fn default() -> Self {
        Self::withoutContext()
    }
}

impl RenderPassApi for RenderPassGL {
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
        self.inner
            .withCurrentState(|state| super::ore_render_pass_gl_impl::setPipeline(state, pipeline));
    }

    fn setVertexBuffer(&mut self, slot: u32, buffer: Option<&AnyResourceHandle>, offset: u32) {
        self.inner.withCurrentState(|state| {
            super::ore_render_pass_gl_impl::setVertexBuffer(state, slot, buffer, offset)
        });
    }

    fn setIndexBuffer(
        &mut self,
        buffer: Option<&AnyResourceHandle>,
        format: IndexFormat,
        offset: u32,
    ) {
        self.inner.withCurrentState(|state| {
            super::ore_render_pass_gl_impl::setIndexBuffer(state, buffer, format, offset)
        });
    }

    fn setBindGroup(
        &mut self,
        groupIndex: u32,
        bindGroup: Option<&AnyResourceHandle>,
        dynamicOffsets: Option<&[u32]>,
        dynamicOffsetCount: u32,
    ) {
        self.inner.withCurrentState(|state| {
            super::ore_render_pass_gl_impl::setBindGroup(
                state,
                groupIndex,
                bindGroup,
                dynamicOffsets,
                dynamicOffsetCount,
            )
        });
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
        self.inner.withCurrentState(|state| {
            super::ore_render_pass_gl_impl::setViewport(
                state, x, y, width, height, minDepth, maxDepth,
            )
        });
    }

    fn setScissorRect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.inner.withCurrentState(|state| {
            super::ore_render_pass_gl_impl::setScissorRect(state, x, y, width, height)
        });
    }

    fn setStencilReference(&mut self, reference: u32) {
        self.inner.withCurrentState(|state| {
            super::ore_render_pass_gl_impl::setStencilReference(state, reference)
        });
    }

    fn setBlendColor(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.inner.withCurrentState(|state| {
            super::ore_render_pass_gl_impl::setBlendColor(state, r, g, b, a)
        });
    }

    fn draw(&mut self, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) {
        self.inner.withCurrentState(|state| {
            super::ore_render_pass_gl_impl::draw(
                state,
                vertexCount,
                instanceCount,
                firstVertex,
                firstInstance,
            )
        });
    }

    fn drawIndexed(
        &mut self,
        indexCount: u32,
        instanceCount: u32,
        firstIndex: u32,
        baseVertex: i32,
        firstInstance: u32,
    ) {
        self.inner.withCurrentState(|state| {
            super::ore_render_pass_gl_impl::drawIndexed(
                state,
                indexCount,
                instanceCount,
                firstIndex,
                baseVertex,
                firstInstance,
            )
        });
    }

    fn finish(&mut self) {
        self.inner
            .withCurrentState(super::ore_render_pass_gl_impl::finish);
    }

    fn validate(&self) {
        self.inner
            .withCurrentState(|state| super::ore_render_pass_gl_impl::validate(state));
    }
}

pub(crate) const SOURCE_PUBLIC_CALLABLE_COUNT: usize = 17;
pub(crate) const SOURCE_BACKEND_FIELD_COUNT: usize = 17;
pub(crate) const SOURCE_RESOLVE_FIELD_COUNT: usize = 5;
pub(crate) const SOURCE_FRIEND_COUNT: usize = 1;
const _: [(); 3006] = [(); PINNED_SOURCE.len()];
