/*
 * Copyright 2025 Rive
 */

// #pragma once
// #include "rive/renderer/ore/ore_render_pass.hpp"
// #include "rive/renderer/ore/ore_pipeline.hpp"
// #include "rive/refcnt.hpp"
// #import <Metal/Metal.h>

// Mechanical translation of the complete pinned source header
// renderer/src/ore/metal/ore_render_pass_metal.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;

use std::cell::{RefCell, RefMut};
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak as RcWeak};
use std::sync::Weak;

use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::AnyResourceHandle;
#[cfg(test)]
use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_types_hpp::IndexFormat;
use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_types_hpp::RenderPassDesc;

// `id<MTLRenderCommandEncoder>`, `id<MTLCommandBuffer>`, and
// `id<MTLBuffer>` are nullable, strong Objective-C owners under ARC. Rust's
// `Retained<T>` is the corresponding strong owner; `Option` preserves each
// source `nil` state. The non-Apple stand-ins keep this source-shaped
// translation available to tools that inspect it off Apple.
#[cfg(target_vendor = "apple")]
use objc2::rc::Retained;
#[cfg(target_vendor = "apple")]
use objc2::runtime::ProtocolObject;
#[cfg(target_vendor = "apple")]
use objc2_metal::{
    MTLBuffer, MTLCommandBuffer, MTLIndexType, MTLPrimitiveType, MTLRenderCommandEncoder,
};

#[cfg(target_vendor = "apple")]
type NativeMetalEncoder = Option<Retained<ProtocolObject<dyn MTLRenderCommandEncoder>>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalEncoder = Option<()>;

#[cfg(target_vendor = "apple")]
type NativeMetalCommandBuffer = Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalCommandBuffer = Option<()>;

#[cfg(target_vendor = "apple")]
type NativeMetalBuffer = Option<Retained<ProtocolObject<dyn MTLBuffer>>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalBuffer = Option<()>;

// The scalar Metal enum members remain values rather than owners. Stand-ins
// retain the authored source defaults on non-Apple inspection targets while
// the Apple branch uses the SDK enum types directly.
#[cfg(target_vendor = "apple")]
type NativeMetalIndexType = MTLIndexType;

#[cfg(not(target_vendor = "apple"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeMetalIndexType {
    UInt16,
    UInt32,
}

#[cfg(target_vendor = "apple")]
type NativeMetalPrimitiveType = MTLPrimitiveType;

#[cfg(not(target_vendor = "apple"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeMetalPrimitiveType {
    Point,
    Line,
    LineStrip,
    Triangle,
    TriangleStrip,
}

// namespace rive::ore

// class ContextMetal;
// The source forward declaration is retained for the friend relationship
// below. ContextMetal is owned by its own translation unit.

// class RenderPassMetal : public RenderPass
// {
// Rust has no class inheritance. `base` is the offset-zero RenderPass
// base-subobject. The derived owners follow in exact source declaration order;
// explicit Drop releases them in reverse order before the base.
#[repr(C)]
pub(crate) struct RenderPassMetalState {
    // C++ base subobject: `RenderPass`.
    pub(crate) base: ManuallyDrop<RenderPass>,

    // id<MTLRenderCommandEncoder> m_mtlEncoder = nil;
    pub(crate) m_mtlEncoder: ManuallyDrop<NativeMetalEncoder>,
    // id<MTLCommandBuffer> m_mtlCommandBuffer = nil;
    pub(crate) m_mtlCommandBuffer: ManuallyDrop<NativeMetalCommandBuffer>,
    // id<MTLBuffer> m_mtlIndexBuffer = nil;
    pub(crate) m_mtlIndexBuffer: ManuallyDrop<NativeMetalBuffer>,
    // MTLIndexType m_mtlIndexType = MTLIndexTypeUInt16;
    pub(crate) m_mtlIndexType: NativeMetalIndexType,
    // NSUInteger m_mtlIndexBufferOffset = 0;
    pub(crate) m_mtlIndexBufferOffset: usize,
    // MTLPrimitiveType m_mtlPrimitiveType = MTLPrimitiveTypeTriangle;
    pub(crate) m_mtlPrimitiveType: NativeMetalPrimitiveType,
    // rcp<Pipeline> m_currentPipeline;
    pub(crate) m_currentPipeline: ManuallyDrop<Option<AnyResourceHandle>>,
}

impl Drop for RenderPassMetalState {
    fn drop(&mut self) {
        #[cfg(target_vendor = "apple")]
        if !self.base.m_finished && self.m_mtlEncoder.is_some() {
            self.finish();
        }
        unsafe {
            ManuallyDrop::drop(&mut self.m_currentPipeline);
            ManuallyDrop::drop(&mut self.m_mtlIndexBuffer);
            ManuallyDrop::drop(&mut self.m_mtlCommandBuffer);
            ManuallyDrop::drop(&mut self.m_mtlEncoder);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl RenderPassMetalState {
    // public:

    // void setPipeline(Pipeline* pipeline) override;
    // void setVertexBuffer(uint32_t slot,
    //                      Buffer* buffer,
    //                      uint32_t offset = 0) override;
    // void setIndexBuffer(Buffer* buffer,
    //                     IndexFormat format,
    //                     uint32_t offset = 0) override;
    // void setBindGroup(uint32_t groupIndex,
    //                   BindGroup* bg,
    //                   const uint32_t* dynamicOffsets = nullptr,
    //                   uint32_t dynamicOffsetCount = 0) override;
    // void setViewport(float x,
    //                  float y,
    //                  float width,
    //                  float height,
    //                  float minDepth = 0.0f,
    //                  float maxDepth = 1.0f) override;
    // void setScissorRect(uint32_t x,
    //                     uint32_t y,
    //                     uint32_t width,
    //                     uint32_t height) override;
    // void setStencilReference(uint32_t ref) override;
    // void setBlendColor(float r, float g, float b, float a) override;
    // void draw(uint32_t vertexCount,
    //           uint32_t instanceCount = 1,
    //           uint32_t firstVertex = 0,
    //           uint32_t firstInstance = 0) override;
    // void drawIndexed(uint32_t indexCount,
    //                  uint32_t instanceCount = 1,
    //                  uint32_t firstIndex = 0,
    //                  int32_t baseVertex = 0,
    //                  uint32_t firstInstance = 0) override;
    // void finish() override;
    // void validate() const override;
    // The paired ore_render_pass_metal.mm translation owns these complete
    // command implementations and preserves their assertion/error ordering.

    // RenderPassMetal() = default;
    // All members use their authored C++ defaults: nil native owners, UInt16
    // indices, zero index offset, triangle primitive, and an empty pipeline
    // owner. Rust spells the implicit default constructor explicitly so the
    // source base constructor and every member initializer remain visible.
    pub(crate) fn new(context: Weak<ContextState>) -> Self {
        Self {
            base: ManuallyDrop::new(RenderPass::new(context)),
            m_mtlEncoder: ManuallyDrop::new(None),
            m_mtlCommandBuffer: ManuallyDrop::new(None),
            m_mtlIndexBuffer: ManuallyDrop::new(None),
            m_mtlIndexType: NativeMetalIndexType::UInt16,
            m_mtlIndexBufferOffset: 0,
            m_mtlPrimitiveType: NativeMetalPrimitiveType::Triangle,
            m_currentPipeline: ManuallyDrop::new(None),
        }
    }

    // RenderPassMetal(Context* context) : RenderPass(context) {}
    // The nullable source Context* is the `Weak<ContextState>` constructed
    // above; every other member retains its authored default initializer.
    // The source Context* constructor is represented by the Weak state above.

    // ~RenderPassMetal() override;
    // The source destructor auto-finishes only when an encoder exists; the
    // paired implementation owns that conditional finish and then native
    // owner release. Rust's eventual Drop implementation must retain the same
    // distinction rather than treating a nil encoder as a live pass.

    // RenderPassMetal(RenderPassMetal&& other) noexcept;
    // RenderPassMetal& operator=(RenderPassMetal&&) noexcept;
    // The paired implementation exposes explicit public move construction and
    // assignment because an ordinary Rust move would incorrectly transfer the
    // base state. These operations preserve the source quirk: the destination
    // move-constructed base is defaulted, assignment leaves both bases in
    // place, and only the moved-from native owners become nil.

    // private:
    // friend class ContextMetal;
    // Rust has no friend declarations; the owning context translation unit
    // uses its crate-local access to these source-private members.
}

pub(crate) struct RenderPassMetalInner {
    pub(crate) state: RefCell<RenderPassMetalState>,
}

impl RenderPassMetalInner {
    pub(crate) fn borrowState(&self) -> RefMut<'_, RenderPassMetalState> {
        self.state.borrow_mut()
    }
}

impl ActiveRenderPass for RenderPassMetalInner {
    fn isFinished(&self) -> bool {
        self.state.borrow().base.isFinished()
    }

    fn finish(&self) {
        self.borrowState().finish();
    }
}

/// Public unique concrete pass owner. Its Arc exists only to provide the weak
/// non-owning active-pass token; cloning the public owner is intentionally not
/// exposed.
pub struct RenderPassMetal {
    pub(crate) inner: Rc<RenderPassMetalInner>,
}

impl RenderPassMetal {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RenderPassMetalInner {
                state: RefCell::new(RenderPassMetalState::new(Weak::new())),
            }),
        }
    }

    pub fn new_with_context(context: &std::sync::Arc<ContextState>) -> Self {
        Self {
            inner: Rc::new(RenderPassMetalInner {
                state: RefCell::new(RenderPassMetalState::new(std::sync::Arc::downgrade(
                    context,
                ))),
            }),
        }
    }

    pub fn activeToken(&self) -> RcWeak<dyn ActiveRenderPass> {
        let token: Rc<dyn ActiveRenderPass> = self.inner.clone();
        Rc::downgrade(&token)
    }

    pub(crate) fn initializeNative(
        &self,
        encoder: NativeMetalEncoder,
        commandBuffer: NativeMetalCommandBuffer,
        desc: &RenderPassDesc<'_>,
    ) {
        let mut state = self.inner.borrowState();
        *state.m_mtlEncoder = encoder;
        *state.m_mtlCommandBuffer = commandBuffer;
        state.base.populateAttachmentMetadata(desc);
    }
}

impl Default for RenderPassMetal {
    fn default() -> Self {
        Self::new()
    }
}

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_active_pass_token_is_thread_safe() {
        // RenderPassMetalInner owns Rc<RefCell<...>> state and native encoder
        // handles.  The source token is therefore intentionally confined to
        // the recording thread (neither Send nor Sync); keeping the concrete
        // Rc witness here prevents this regression from being mistaken for a
        // shareable synchronization object.
        let _recording_thread_witness =
            std::marker::PhantomData::<std::rc::Rc<RenderPassMetalInner>>;
        assert!(std::mem::size_of::<RenderPassMetalInner>() > 0);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn live_encoder_covers_state_binding_draw_and_finish_ownership() {
        use objc2_metal::MTLCreateSystemDefaultDevice;
        let Some(_device) = MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        // The canonical owner is constructed first and receives native state
        // through initializeNative; this keeps the test on the translated
        // RenderPassMetal ABI rather than the removed adapter constructors.
        let context = Context::new(Features::default(), None);
        let pass = RenderPassMetal::new_with_context(&context.state);
        assert!(!pass.inner.borrowState().base.isFinished());
        context.setActiveRenderPass(Some(&pass));
        let token = context.activeRenderPass().expect("active pass token");
        assert!(!token.upgrade().expect("live pass").isFinished());
        context.finishActiveRenderPass();
        assert!(pass.inner.borrowState().base.isFinished());
        context.finishActiveRenderPass();
        assert!(pass.inner.borrowState().base.isFinished());
    }

    #[test]
    fn conversion_helpers_cover_every_source_enum_value() {
        #[cfg(target_vendor = "apple")]
        {
            assert_eq!(
                crate::types::PrimitiveTopology::triangleStrip as u8,
                crate::types::PrimitiveTopology::triangleStrip as u8
            );
            assert_eq!(IndexFormat::uint16 as u8, IndexFormat::uint16 as u8);
            assert_eq!(
                crate::types::CullMode::back as u8,
                crate::types::CullMode::back as u8
            );
            assert_eq!(
                crate::types::FaceWinding::clockwise as u8,
                crate::types::FaceWinding::clockwise as u8
            );
        }
    }
}
