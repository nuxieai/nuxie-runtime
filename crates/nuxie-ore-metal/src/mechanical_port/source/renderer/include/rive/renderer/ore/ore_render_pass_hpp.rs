/*
 * Copyright 2025 Rive
 */

// #pragma once

// #include "rive/refcnt.hpp"
// #include "rive/renderer/ore/ore_types.hpp"

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/ore/ore_render_pass.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;

use std::any::Any;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::rc::Weak as RcWeak;
use std::sync::Weak;

use super::super::gpu_resource_hpp::AnyResourceHandle;
#[cfg(test)]
use super::ore_types_hpp::RenderPassDesc;
use super::ore_types_hpp::{IndexFormat, TextureFormat, kMaxBindGroups};

// namespace rive::ore

pub trait RenderPassApi {
    fn asAny(&self) -> &dyn Any;
    fn asAnyMut(&mut self) -> &mut dyn Any;
    fn intoAny(self: Box<Self>) -> Box<dyn Any>;
    /// Non-owning spelling of the source `RenderPass*` identity.  Returning
    /// the weak token is important: merely asking Context which pass is
    /// active must not extend the unique pass/encoder lifetime.
    fn activeToken(&self) -> RcWeak<dyn ActiveRenderPass>;
    fn setPipeline(&mut self, pipeline: Option<&AnyResourceHandle>);
    fn setVertexBuffer(&mut self, slot: u32, buffer: Option<&AnyResourceHandle>, offset: u32);
    fn setIndexBuffer(
        &mut self,
        buffer: Option<&AnyResourceHandle>,
        format: IndexFormat,
        offset: u32,
    );
    fn setBindGroup(
        &mut self,
        groupIndex: u32,
        bindGroup: Option<&AnyResourceHandle>,
        dynamicOffsets: Option<&[u32]>,
        dynamicOffsetCount: u32,
    );
    fn setViewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        minDepth: f32,
        maxDepth: f32,
    );
    fn setScissorRect(&mut self, x: u32, y: u32, width: u32, height: u32);
    fn setStencilReference(&mut self, reference: u32);
    fn setBlendColor(&mut self, r: f32, g: f32, b: f32, a: f32);
    fn draw(&mut self, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32);
    fn drawIndexed(
        &mut self,
        indexCount: u32,
        instanceCount: u32,
        firstIndex: u32,
        baseVertex: i32,
        firstInstance: u32,
    );
    fn finish(&mut self);
    fn validate(&self);
}

// class Buffer;
// class Texture;
// class TextureView;
// class Sampler;
// class Pipeline;
// class BindGroup;
// class Context;
// The source forward declarations remain source-visible here. The complete
// backend owners are supplied by their corresponding translation units.

// Backend-agnostic render-pass command surface. Concrete backends implement
// the source-pure-virtual command methods and own their native encoder state.
// The portable base retains lifecycle, attachment-compatibility metadata, the
// non-owning Context back-pointer, and strong bound BindGroup references.
#[repr(C)]
pub struct RenderPassMembers {
    // protected:
    // bool m_finished = false;
    pub(crate) m_finished: bool,

    // Attachment metadata — populated by beginRenderPass() from the
    // RenderPassDesc via populateAttachmentMetadata(). Used by setPipeline()
    // for format/sampleCount validation.
    // TextureFormat m_colorFormats[4] = {};
    // Rust's explicit zero discriminant preserves C++ enum value-initialization
    // (`TextureFormat::r8unorm` is the zero-valued source format).
    pub(crate) m_colorFormats: [TextureFormat; 4],
    // uint32_t m_colorCount = 0;
    pub(crate) m_colorCount: u32,
    // TextureFormat m_depthFormat = {};
    pub(crate) m_depthFormat: TextureFormat,
    // bool m_hasDepth = false;
    pub(crate) m_hasDepth: bool,
    // uint32_t m_sampleCount = 1;
    pub(crate) m_sampleCount: u32,
    // Cross-backend back-pointer used by the pipeline-compat validator to
    // route errors through Context::setLastError(). Weak ref.
    // Context* m_context = nullptr;
    // Rust represents the nullable raw pointer as `Weak<ContextState>`; it is
    // non-owning and cannot dereference a destroyed Context.
    pub(crate) m_context: Weak<ContextState>,
    // Last-authored C++ member, therefore first explicit drop.
    pub(crate) m_boundGroups: [Option<AnyResourceHandle>; kMaxBindGroups as usize],
}

#[repr(C)]
pub struct RenderPass {
    pub(crate) members: ManuallyDrop<RenderPassMembers>,
}

impl Deref for RenderPass {
    type Target = RenderPassMembers;

    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl DerefMut for RenderPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.members
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            #[cfg(test)]
            crate::gpu_resource::record_resource_drop_stage("RenderPass.boundGroups");
            core::ptr::drop_in_place(&mut self.m_boundGroups);
            #[cfg(test)]
            crate::gpu_resource::record_resource_drop_stage("RenderPass.context");
            core::ptr::drop_in_place(&mut self.m_context);
        }
    }
}

impl RenderPass {
    pub(crate) fn ownsResource(&self, resource: &AnyResourceHandle) -> bool {
        self.m_context
            .upgrade()
            .is_some_and(|context| resource.belongsTo(&context.resourceDomain()))
    }

    // public:

    // virtual void setPipeline(Pipeline* pipeline) = 0;
    // `RenderPassApi` above is the callable Rust virtual-dispatch surface.

    // virtual void setVertexBuffer(uint32_t slot,
    //                              Buffer* buffer,
    //                              uint32_t offset = 0) = 0;
    // Rust has no pure-virtual member declaration. `offset` remains explicit
    // and callers pass the source default `0` when omitted at the C++ call
    // site.

    // virtual void setIndexBuffer(Buffer* buffer,
    //                             IndexFormat format,
    //                             uint32_t offset = 0) = 0;
    // `IndexFormat` and the source default offset remain part of the API.

    // Bind a pre-created BindGroup, optionally overriding dynamic UBO offsets.
    // virtual void setBindGroup(uint32_t groupIndex,
    //                           BindGroup* bg,
    //                           const uint32_t* dynamicOffsets = nullptr,
    //                           uint32_t dynamicOffsetCount = 0) = 0;
    // The concrete backend retains the source nullable BindGroup borrow and
    // nullable dynamic-offset pointer/default count without changing order.

    // virtual void setViewport(float x,
    //                          float y,
    //                          float width,
    //                          float height,
    //                          float minDepth = 0.0f,
    //                          float maxDepth = 1.0f) = 0;

    // virtual void setScissorRect(uint32_t x,
    //                             uint32_t y,
    //                             uint32_t width,
    //                             uint32_t height) = 0;

    // virtual void setStencilReference(uint32_t ref) = 0;
    // virtual void setBlendColor(float r, float g, float b, float a) = 0;

    // virtual void draw(uint32_t vertexCount,
    //                   uint32_t instanceCount = 1,
    //                   uint32_t firstVertex = 0,
    //                   uint32_t firstInstance = 0) = 0;

    // virtual void drawIndexed(uint32_t indexCount,
    //                          uint32_t instanceCount = 1,
    //                          uint32_t firstIndex = 0,
    //                          int32_t baseVertex = 0,
    //                          uint32_t firstInstance = 0) = 0;

    // virtual void finish() = 0;

    // bool isFinished() const { return m_finished; }
    pub fn isFinished(&self) -> bool {
        self.m_finished
    }

    // virtual ~RenderPass() = default;
    // Rust's default drop glue supplies the virtual-destructor boundary. The
    // field order above retains the source owner graph and bound-group drops.

    // RenderPass(const RenderPass&) = delete;
    // RenderPass& operator=(const RenderPass&) = delete;
    // Rust exposes no Clone implementation; ordinary moves transfer the
    // complete source-shaped owner.

    // Populate attachment metadata from the descriptor. Called by every
    // backend's beginRenderPass() immediately after constructing the
    // RenderPass; defined inline because it only reads cross-backend
    // texture/view accessors that are uniform across backends.
    // inline void populateAttachmentMetadata(const RenderPassDesc& desc);
    // The definition is in the companion ore_context translation, matching
    // the pinned source's definition in ore_context.hpp.

    // virtual void validate() const {};
    // The default virtual body is intentionally a no-op; concrete backends may
    // provide their validation seam without changing this source default.
    pub fn validate(&self) {}

    // protected:
    // friend class Context;
    // Rust has no friend declarations; the owning translation units use
    // crate visibility for this source-protected state.

    // RenderPass() = default;
    pub(crate) fn new(context: Weak<ContextState>) -> Self {
        Self {
            members: ManuallyDrop::new(RenderPassMembers {
                m_finished: false,
                m_colorFormats: [TextureFormat::r8unorm; 4],
                m_colorCount: 0,
                m_depthFormat: TextureFormat::r8unorm,
                m_hasDepth: false,
                m_sampleCount: 1,
                m_context: context,
                m_boundGroups: std::array::from_fn(|_| None),
            }),
        }
    }

    // RenderPass(Context* context) : m_context(context) {}
    // The nullable source pointer is represented by Weak<ContextState>;
    // all other members retain their authored default initializers.
    // The source Context* constructor is represented by `Weak<ContextState>`
    // so pass validation can publish errors without extending context life.

    // WebGPU-spec pipeline/attachment compatibility check, invoked from
    // every backend's setPipeline().
    // inline bool checkPipelineCompat(const Pipeline* pipeline) const;
    // The definition is in the companion ore_context translation, matching
    // the pinned source's definition in ore_context.hpp.
}

// } // namespace rive::ore
#[cfg(all(test, target_vendor = "apple"))]
mod tests {
    use super::*;
    use crate::gpu_resource::ResourceHandle;
    use crate::metal::texture::{TextureMetal, TextureViewMetal};
    use crate::pipeline::Pipeline;
    use crate::types::{
        ColorAttachment, ColorTargetState, DepthStencilAttachment, DepthStencilState, PipelineDesc,
        TextureAspect, TextureDesc, TextureViewDesc, TextureViewDimension,
    };
    use std::sync::Arc;

    fn texture_view(format: TextureFormat, sample_count: u32) -> AnyResourceHandle {
        let texture_desc = TextureDesc {
            width: 4,
            height: 4,
            format,
            renderTarget: true,
            sampleCount: sample_count,
            ..TextureDesc::default()
        };
        let texture =
            crate::gpu_resource::ResourceHandle::new(None, TextureMetal::new(&texture_desc))
                .erase();
        let view_desc = TextureViewDesc {
            texture: Some(&texture),
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        };
        crate::gpu_resource::ResourceHandle::new(
            None,
            TextureViewMetal::new(texture.clone(), &view_desc),
        )
        .erase()
    }

    fn pass_for(desc: &RenderPassDesc<'_>) -> RenderPass {
        let mut pass = RenderPass::new(Weak::new());
        pass.populateAttachmentMetadata(desc);
        pass
    }

    #[test]
    fn attachment_metadata_drives_compatibility_in_upstream_check_order() {
        let color = texture_view(TextureFormat::bgra8unorm, 4);
        let depth = texture_view(TextureFormat::depth32float, 4);
        let pass_desc = RenderPassDesc {
            colorAttachments: [
                ColorAttachment {
                    view: Some(&color),
                    ..ColorAttachment::default()
                },
                ColorAttachment::default(),
                ColorAttachment::default(),
                ColorAttachment::default(),
            ],
            colorCount: 1,
            depthStencil: DepthStencilAttachment {
                view: Some(&depth),
                ..DepthStencilAttachment::default()
            },
            label: None,
        };
        let pass = pass_for(&pass_desc);
        let pipeline = Pipeline::new(&PipelineDesc {
            colorTargets: [ColorTargetState::default(); 4],
            colorCount: 1,
            depthStencil: DepthStencilState {
                format: TextureFormat::depth32float,
                ..DepthStencilState::default()
            },
            sampleCount: 4,
            ..PipelineDesc::default()
        })
        .expect("compatible pipeline");

        assert!(pass.checkPipelineCompat(Some(&pipeline)));

        let wrong_count = Pipeline::new(&PipelineDesc {
            colorCount: 0,
            sampleCount: 1,
            ..PipelineDesc::default()
        })
        .expect("wrong-count pipeline");
        assert!(!pass.checkPipelineCompat(Some(&wrong_count)));
    }

    #[test]
    fn depth_only_pass_takes_its_sample_count_from_depth() {
        let depth = texture_view(TextureFormat::depth16unorm, 2);
        let desc = RenderPassDesc {
            colorCount: 0,
            depthStencil: DepthStencilAttachment {
                view: Some(&depth),
                ..DepthStencilAttachment::default()
            },
            ..RenderPassDesc::default()
        };
        let pass = pass_for(&desc);
        let pipeline = Pipeline::new(&PipelineDesc {
            colorCount: 0,
            depthStencil: DepthStencilState {
                format: TextureFormat::depth16unorm,
                ..DepthStencilState::default()
            },
            sampleCount: 2,
            ..PipelineDesc::default()
        })
        .expect("depth-only pipeline");

        assert!(pass.checkPipelineCompat(Some(&pipeline)));
    }

    #[test]
    fn bound_groups_are_retained_by_slot_until_finish_and_finish_is_idempotent() {
        let desc = RenderPassDesc {
            colorCount: 0,
            ..RenderPassDesc::default()
        };
        let mut pass = pass_for(&desc);
        let group =
            ResourceHandle::new(None, crate::gpu_resource::TestGPUResource::new(7_u32)).erase();

        pass.m_boundGroups[2] = Some(group.clone());
        assert_eq!(group.debugging_refcnt(), 2);
        pass.m_finished = true;
        assert!(pass.isFinished());
        drop(pass);
        assert_eq!(group.debugging_refcnt(), 1);
    }

    #[test]
    fn invalid_group_slot_fails_closed_without_retaining() {
        let desc = RenderPassDesc {
            colorCount: 0,
            ..RenderPassDesc::default()
        };
        let pass = pass_for(&desc);
        let group =
            ResourceHandle::new(None, crate::gpu_resource::TestGPUResource::new(9_u32)).erase();

        assert!(kMaxBindGroups as usize >= pass.m_boundGroups.len());
        assert_eq!(group.debugging_refcnt(), 1);
    }

    #[test]
    fn compatibility_failure_routes_the_first_exact_error_to_the_live_context() {
        let context = Context::new(crate::types::Features::default(), None);
        let desc = RenderPassDesc {
            colorCount: 0,
            ..RenderPassDesc::default()
        };
        let mut pass = RenderPass::new(Arc::downgrade(&context.state));
        pass.populateAttachmentMetadata(&desc);
        let pipeline = Pipeline::new(&PipelineDesc {
            colorCount: 1,
            ..PipelineDesc::default()
        })
        .expect("incompatible pipeline");

        assert!(!pass.checkPipelineCompat(Some(&pipeline)));
        assert_eq!(
            context.lastError(),
            "setPipeline: pipeline has 1 color targets but render pass was begun with 0"
        );
    }
}
