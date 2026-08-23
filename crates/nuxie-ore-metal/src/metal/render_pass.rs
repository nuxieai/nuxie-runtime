// Mechanical translation of:
//   renderer/src/ore/metal/ore_render_pass_metal.hpp
//   renderer/src/ore/metal/ore_render_pass_metal.mm
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

#![allow(non_snake_case)]

use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::context::{ActiveRenderPass, ContextState};
use crate::gpu_resource::AnyResourceHandle;
use crate::metal::bind_group::BindGroupMetal;
use crate::metal::buffer::BufferMetal;
use crate::metal::pipeline::PipelineMetal;
use crate::render_pass::{RenderPass, RenderPassError};
use crate::types::{IndexFormat, RenderPassDesc};

#[cfg(target_vendor = "apple")]
use objc2::rc::Retained;
#[cfg(target_vendor = "apple")]
use objc2::runtime::ProtocolObject;
#[cfg(target_vendor = "apple")]
use objc2_metal::{
    MTLBuffer, MTLCommandBuffer, MTLCommandEncoder, MTLCullMode, MTLIndexType, MTLPrimitiveType,
    MTLRenderCommandEncoder, MTLScissorRect, MTLViewport, MTLWinding,
};

#[cfg(target_vendor = "apple")]
struct RetainedMetalEncoder(Retained<ProtocolObject<dyn MTLRenderCommandEncoder>>);

// SAFETY: all access to this stateful encoder is serialized by
// `RenderPassMetalInner::state`; no encoder method can run concurrently.
#[cfg(target_vendor = "apple")]
unsafe impl Send for RetainedMetalEncoder {}
// SAFETY: shared access never escapes the serializing state mutex.
#[cfg(target_vendor = "apple")]
unsafe impl Sync for RetainedMetalEncoder {}

#[cfg(target_vendor = "apple")]
#[expect(
    dead_code,
    reason = "the command buffer is retained for the pass lifetime, not messaged by the pass"
)]
struct RetainedMetalCommandBuffer(Retained<ProtocolObject<dyn MTLCommandBuffer>>);

// SAFETY: the command buffer is retained for lifetime only; this module does
// not commit it or invoke stateful command-buffer operations through the field.
#[cfg(target_vendor = "apple")]
unsafe impl Send for RetainedMetalCommandBuffer {}
// SAFETY: same retain-only invariant as `Send` above.
#[cfg(target_vendor = "apple")]
unsafe impl Sync for RetainedMetalCommandBuffer {}

#[cfg(target_vendor = "apple")]
struct RetainedMetalIndexBuffer(Retained<ProtocolObject<dyn objc2_metal::MTLBuffer>>);

// SAFETY: the native buffer is retained as immutable draw state and all use is
// serialized by the render-pass state mutex.
#[cfg(target_vendor = "apple")]
unsafe impl Send for RetainedMetalIndexBuffer {}
// SAFETY: same serialized immutable-handle invariant as `Send` above.
#[cfg(target_vendor = "apple")]
unsafe impl Sync for RetainedMetalIndexBuffer {}

/// Concrete Metal render pass.
///
/// The public value is intentionally not `Clone`: an ordinary Rust move is
/// the C++ move operation and transfers the sole caller-facing owner without
/// touching native state. The context receives only a weak active-pass token.
/// Encoding is recording-thread-bound, so the public pass is neither `Send`
/// nor `Sync`.
///
/// ```compile_fail
/// fn require_send_sync<T: Send + Sync>() {}
/// require_send_sync::<nuxie_ore_metal::metal::render_pass::RenderPassMetal>();
/// ```
pub struct RenderPassMetal {
    inner: Arc<RenderPassMetalInner>,
    // Encoding follows the source's caller-serialized recording-thread
    // contract. Only the private active-pass token crosses threads.
    _recording_thread: PhantomData<Rc<()>>,
}

struct RenderPassMetalInner {
    state: Mutex<RenderPassMetalState>,
}

struct RenderPassMetalState {
    // Rust drops fields in declaration order. This order mirrors C++ derived
    // destruction: current pipeline, index buffer, command buffer, encoder,
    // then the portable base and its bound groups.
    m_currentPipeline: Option<AnyResourceHandle>,
    #[cfg(target_vendor = "apple")]
    m_mtlIndexBuffer: Option<RetainedMetalIndexBuffer>,
    #[cfg(target_vendor = "apple")]
    #[expect(
        dead_code,
        reason = "exact source lifetime owner; ContextMetal commits its separate command-buffer owner"
    )]
    m_mtlCommandBuffer: RetainedMetalCommandBuffer,
    #[cfg(target_vendor = "apple")]
    m_mtlEncoder: Option<RetainedMetalEncoder>,
    #[cfg(target_vendor = "apple")]
    m_mtlIndexType: MTLIndexType,
    #[cfg(target_vendor = "apple")]
    m_mtlIndexBufferOffset: usize,
    #[cfg(target_vendor = "apple")]
    m_mtlPrimitiveType: MTLPrimitiveType,
    base: RenderPass,
}

impl RenderPassMetal {
    /// Publish a pass only after ContextMetal has created a live encoder.
    #[cfg(target_vendor = "apple")]
    pub(crate) fn with_native_encoder(
        context: &Arc<ContextState>,
        desc: &RenderPassDesc<'_>,
        encoder: Retained<ProtocolObject<dyn MTLRenderCommandEncoder>>,
        command_buffer: Retained<ProtocolObject<dyn MTLCommandBuffer>>,
    ) -> Self {
        Self {
            inner: Arc::new(RenderPassMetalInner {
                state: Mutex::new(RenderPassMetalState {
                    m_currentPipeline: None,
                    m_mtlIndexBuffer: None,
                    m_mtlCommandBuffer: RetainedMetalCommandBuffer(command_buffer),
                    m_mtlEncoder: Some(RetainedMetalEncoder(encoder)),
                    m_mtlIndexType: MTLIndexType::UInt16,
                    m_mtlIndexBufferOffset: 0,
                    m_mtlPrimitiveType: MTLPrimitiveType::Triangle,
                    base: RenderPass::new(Arc::downgrade(context), desc),
                }),
            }),
            _recording_thread: PhantomData,
        }
    }

    pub(crate) fn active_token(&self) -> Arc<dyn ActiveRenderPass> {
        self.inner.clone()
    }

    pub fn is_finished(&self) -> bool {
        self.inner.lock_state().base.is_finished()
    }

    pub fn finish(&self) {
        self.inner.finish_inner();
    }

    #[cfg(target_vendor = "apple")]
    pub fn setPipeline(&self, pipeline: &AnyResourceHandle) -> Result<(), RenderPassError> {
        let mut state = self.inner.lock_state();
        state.base.validate()?;
        let Some(pipeline_metal) = pipeline.downcast_ref::<PipelineMetal>() else {
            return state
                .base
                .fail(RenderPassError::WrongBackendResource("pipeline"));
        };
        state.base.check_pipeline_compat(pipeline_metal.base())?;

        let desc = pipeline_metal.base().desc();
        {
            let encoder = state.encoder()?;
            encoder.setRenderPipelineState(pipeline_metal.mtl_pipeline());
            encoder.setDepthStencilState(pipeline_metal.mtl_depth_stencil());
            encoder.setCullMode(cull_mode_to_mtl(desc.cullMode));
            encoder.setFrontFacingWinding(winding_to_mtl(desc.winding));
        }
        state.m_mtlPrimitiveType = primitive_topology_to_mtl(desc.topology);
        state.m_currentPipeline = Some(pipeline.clone());

        if desc.depthStencil.depthBias != 0 || desc.depthStencil.depthBiasSlopeScale != 0.0 {
            state.encoder()?.setDepthBias_slopeScale_clamp(
                desc.depthStencil.depthBias as f32,
                desc.depthStencil.depthBiasSlopeScale,
                desc.depthStencil.depthBiasClamp,
            );
        }
        Ok(())
    }

    #[cfg(target_vendor = "apple")]
    pub fn setVertexBuffer(
        &self,
        slot: u32,
        buffer: &AnyResourceHandle,
        offset: u32,
    ) -> Result<(), RenderPassError> {
        const METAL_VERTEX_BUFFER_BASE: u32 = 16;

        let state = self.inner.lock_state();
        state.base.validate()?;
        let Some(buffer) = buffer.downcast_ref::<BufferMetal>() else {
            return state
                .base
                .fail(RenderPassError::WrongBackendResource("buffer"));
        };
        if offset > buffer.base().size() {
            return state.base.fail(RenderPassError::BufferOffsetOutOfRange {
                offset,
                size: buffer.base().size(),
            });
        }
        if slot >= 15 {
            return state
                .base
                .fail(RenderPassError::VertexBufferSlotOutOfRange { slot });
        }
        let Some(native_slot) = slot.checked_add(METAL_VERTEX_BUFFER_BASE) else {
            return state
                .base
                .fail(RenderPassError::VertexBufferSlotOutOfRange { slot });
        };
        let current = buffer.mark_bound_and_current_buffer();
        // SAFETY: `current` remains retained through this call and Metal's
        // command encoder records its own resource reference. ContextMetal
        // validates shader-generated slots; the vertex base keeps them clear
        // of low uniform-buffer indices.
        unsafe {
            state.encoder()?.setVertexBuffer_offset_atIndex(
                Some(&current),
                offset as usize,
                native_slot as usize,
            );
        }
        Ok(())
    }

    #[cfg(target_vendor = "apple")]
    pub fn setIndexBuffer(
        &self,
        buffer: &AnyResourceHandle,
        format: IndexFormat,
        offset: u32,
    ) -> Result<(), RenderPassError> {
        let mut state = self.inner.lock_state();
        state.base.validate()?;
        let Some(buffer) = buffer.downcast_ref::<BufferMetal>() else {
            return state
                .base
                .fail(RenderPassError::WrongBackendResource("buffer"));
        };
        let index_type = index_format_to_mtl(format)
            .ok_or(RenderPassError::InvalidIndexFormat)
            .or_else(|error| state.base.fail(error))?;
        if offset > buffer.base().size() {
            return state.base.fail(RenderPassError::BufferOffsetOutOfRange {
                offset,
                size: buffer.base().size(),
            });
        }

        state.m_mtlIndexBuffer = Some(RetainedMetalIndexBuffer(
            buffer.mark_bound_and_current_buffer(),
        ));
        state.m_mtlIndexType = index_type;
        state.m_mtlIndexBufferOffset = offset as usize;
        Ok(())
    }

    #[cfg(target_vendor = "apple")]
    pub fn setBindGroup(
        &self,
        group_index: u32,
        group: &AnyResourceHandle,
        dynamic_offsets: &[u32],
    ) -> Result<(), RenderPassError> {
        let mut state = self.inner.lock_state();
        state.base.validate()?;
        let Some(group_metal) = group.downcast_ref::<BindGroupMetal>() else {
            return state
                .base
                .fail(RenderPassError::WrongBackendResource("bind group"));
        };

        for binding in group_metal.buffers() {
            validate_native_binding_slot(&state.base, "buffer", binding.vs_slot(), 31)?;
            validate_native_binding_slot(&state.base, "buffer", binding.fs_slot(), 31)?;
        }
        for binding in group_metal.textures() {
            validate_native_binding_slot(&state.base, "texture", binding.vs_slot(), 128)?;
            validate_native_binding_slot(&state.base, "texture", binding.fs_slot(), 128)?;
        }
        for binding in group_metal.samplers() {
            validate_native_binding_slot(&state.base, "sampler", binding.vs_slot(), 16)?;
            validate_native_binding_slot(&state.base, "sampler", binding.fs_slot(), 16)?;
        }

        // Validate the complete buffer plan before changing owner or encoder
        // state. ContextMetal normally guarantees each source index, but this
        // checked boundary keeps a malformed internal payload fail-closed.
        let mut dynamic_index = 0;
        for binding in group_metal.buffers() {
            let Some(source) = binding.source(group_metal) else {
                return state
                    .base
                    .fail(RenderPassError::WrongBackendResource("bind-group buffer"));
            };
            let mut offset = binding.offset();
            if binding.has_dynamic_offset()
                && let Some(dynamic_offset) = dynamic_offsets.get(dynamic_index)
            {
                offset = offset
                    .checked_add(*dynamic_offset)
                    .ok_or(RenderPassError::DynamicOffsetOverflow)
                    .or_else(|error| state.base.fail(error))?;
                dynamic_index = dynamic_index.saturating_add(1);
            }
            if offset > source.base().size() {
                return state.base.fail(RenderPassError::BufferOffsetOutOfRange {
                    offset,
                    size: source.base().size(),
                });
            }
        }

        // C++ takes the strong owner before it emits any native bindings.
        state.base.retain_bound_group(group_index, group)?;
        let encoder = state.encoder()?;
        dynamic_index = 0;
        for binding in group_metal.buffers() {
            let source = binding
                .source(group_metal)
                .expect("buffer plan was validated above");
            let mut offset = binding.offset();
            if binding.has_dynamic_offset()
                && let Some(dynamic_offset) = dynamic_offsets.get(dynamic_index)
            {
                offset = offset
                    .checked_add(*dynamic_offset)
                    .expect("dynamic offsets were validated before encoder mutation");
                dynamic_index = dynamic_index.saturating_add(1);
            }
            let buffer = source.mark_bound_and_current_buffer();
            // SAFETY: the current backing is retained through both calls and
            // marked bound before publication; resolved stage slots come from
            // ContextMetal's shader binding map.
            unsafe {
                if binding.vs_slot() != crate::binding_map::BindingMap::kAbsent {
                    encoder.setVertexBuffer_offset_atIndex(
                        Some(&buffer),
                        offset as usize,
                        binding.vs_slot() as usize,
                    );
                }
                if binding.fs_slot() != crate::binding_map::BindingMap::kAbsent {
                    encoder.setFragmentBuffer_offset_atIndex(
                        Some(&buffer),
                        offset as usize,
                        binding.fs_slot() as usize,
                    );
                }
            }
        }

        for binding in group_metal.textures() {
            // SAFETY: native handles are retained by the bind group until
            // finish releases its logical owner; resolved slots come from the
            // validated binding map.
            unsafe {
                if binding.vs_slot() != crate::binding_map::BindingMap::kAbsent {
                    encoder.setVertexTexture_atIndex(binding.texture(), binding.vs_slot() as usize);
                }
                if binding.fs_slot() != crate::binding_map::BindingMap::kAbsent {
                    encoder
                        .setFragmentTexture_atIndex(binding.texture(), binding.fs_slot() as usize);
                }
            }
        }

        for binding in group_metal.samplers() {
            // SAFETY: sampler states are immutable and retained by the bound
            // group; resolved slots come from the validated binding map.
            unsafe {
                if binding.vs_slot() != crate::binding_map::BindingMap::kAbsent {
                    encoder.setVertexSamplerState_atIndex(
                        binding.sampler(),
                        binding.vs_slot() as usize,
                    );
                }
                if binding.fs_slot() != crate::binding_map::BindingMap::kAbsent {
                    encoder.setFragmentSamplerState_atIndex(
                        binding.sampler(),
                        binding.fs_slot() as usize,
                    );
                }
            }
        }
        Ok(())
    }

    #[cfg(target_vendor = "apple")]
    pub fn setViewport(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) -> Result<(), RenderPassError> {
        let state = self.inner.lock_state();
        state.base.validate()?;
        state.encoder()?.setViewport(MTLViewport {
            originX: x as f64,
            originY: y as f64,
            width: width as f64,
            height: height as f64,
            znear: min_depth as f64,
            zfar: max_depth as f64,
        });
        Ok(())
    }

    #[cfg(target_vendor = "apple")]
    pub fn setScissorRect(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), RenderPassError> {
        let state = self.inner.lock_state();
        state.base.validate()?;
        state.encoder()?.setScissorRect(MTLScissorRect {
            x: x as usize,
            y: y as usize,
            width: width as usize,
            height: height as usize,
        });
        Ok(())
    }

    #[cfg(target_vendor = "apple")]
    pub fn setStencilReference(&self, reference: u32) -> Result<(), RenderPassError> {
        let state = self.inner.lock_state();
        state.base.validate()?;
        state.encoder()?.setStencilReferenceValue(reference);
        Ok(())
    }

    #[cfg(target_vendor = "apple")]
    pub fn setBlendColor(
        &self,
        red: f32,
        green: f32,
        blue: f32,
        alpha: f32,
    ) -> Result<(), RenderPassError> {
        let state = self.inner.lock_state();
        state.base.validate()?;
        state
            .encoder()?
            .setBlendColorRed_green_blue_alpha(red, green, blue, alpha);
        Ok(())
    }

    #[cfg(target_vendor = "apple")]
    pub fn draw(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<(), RenderPassError> {
        let state = self.inner.lock_state();
        state.base.validate()?;
        // SAFETY: all scalar values widen losslessly to NSUInteger; pipeline
        // compatibility and vertex bindings are established by prior calls.
        unsafe {
            state
                .encoder()?
                .drawPrimitives_vertexStart_vertexCount_instanceCount_baseInstance(
                    state.m_mtlPrimitiveType,
                    first_vertex as usize,
                    vertex_count as usize,
                    instance_count as usize,
                    first_instance as usize,
                );
        }
        Ok(())
    }

    #[cfg(target_vendor = "apple")]
    pub fn drawIndexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    ) -> Result<(), RenderPassError> {
        let state = self.inner.lock_state();
        state.base.validate()?;
        let Some(index_buffer) = state.m_mtlIndexBuffer.as_ref() else {
            return state.base.fail(RenderPassError::MissingIndexBuffer);
        };
        let index_size = match state.m_mtlIndexType {
            MTLIndexType::UInt32 => std::mem::size_of::<u32>(),
            _ => std::mem::size_of::<u16>(),
        };
        let offset = (first_index as usize)
            .checked_mul(index_size)
            .and_then(|first| state.m_mtlIndexBufferOffset.checked_add(first))
            .ok_or(RenderPassError::IndexOffsetOverflow)
            .or_else(|error| state.base.fail(error))?;
        let end = (index_count as usize)
            .checked_mul(index_size)
            .and_then(|span| offset.checked_add(span))
            .ok_or(RenderPassError::IndexOffsetOverflow)
            .or_else(|error| state.base.fail(error))?;
        if end > index_buffer.0.length() {
            return state.base.fail(RenderPassError::IndexBufferOutOfRange);
        }

        // SAFETY: the index backing remains retained in pass state, the byte
        // offset calculation is checked, and all unsigned arguments widen
        // losslessly to NSUInteger.
        unsafe {
            state
                .encoder()?
                .drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount_baseVertex_baseInstance(
                    state.m_mtlPrimitiveType,
                    index_count as usize,
                    state.m_mtlIndexType,
                    &index_buffer.0,
                    offset,
                    instance_count as usize,
                    base_vertex as isize,
                    first_instance as usize,
                );
        }
        Ok(())
    }
}

fn validate_native_binding_slot(
    pass: &RenderPass,
    kind: &'static str,
    slot: u16,
    limit: u16,
) -> Result<(), RenderPassError> {
    if slot == crate::binding_map::BindingMap::kAbsent || slot < limit {
        return Ok(());
    }
    pass.fail(RenderPassError::NativeBindingSlotOutOfRange { kind, slot, limit })
}

impl RenderPassMetalInner {
    fn lock_state(&self) -> MutexGuard<'_, RenderPassMetalState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn finish_inner(&self) {
        self.lock_state().finish();
    }
}

impl ActiveRenderPass for RenderPassMetalInner {
    fn is_finished(&self) -> bool {
        self.lock_state().base.is_finished()
    }

    fn finish(&self) {
        self.finish_inner();
    }
}

impl Drop for RenderPassMetalInner {
    fn drop(&mut self) {
        let state = self
            .state
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.finish();
    }
}

impl RenderPassMetalState {
    #[cfg(target_vendor = "apple")]
    fn encoder(&self) -> Result<&ProtocolObject<dyn MTLRenderCommandEncoder>, RenderPassError> {
        self.m_mtlEncoder
            .as_ref()
            .map(|encoder| encoder.0.as_ref())
            .ok_or(RenderPassError::Finished)
    }

    fn finish(&mut self) {
        if self.base.is_finished() {
            return;
        }
        #[cfg(target_vendor = "apple")]
        if let Some(encoder) = self.m_mtlEncoder.take() {
            encoder.0.endEncoding();
        }
        self.base.finish();
        self.m_currentPipeline = None;
    }
}

#[cfg(target_vendor = "apple")]
fn primitive_topology_to_mtl(topology: crate::types::PrimitiveTopology) -> MTLPrimitiveType {
    match topology {
        crate::types::PrimitiveTopology::pointList => MTLPrimitiveType::Point,
        crate::types::PrimitiveTopology::lineList => MTLPrimitiveType::Line,
        crate::types::PrimitiveTopology::lineStrip => MTLPrimitiveType::LineStrip,
        crate::types::PrimitiveTopology::triangleList => MTLPrimitiveType::Triangle,
        crate::types::PrimitiveTopology::triangleStrip => MTLPrimitiveType::TriangleStrip,
    }
}

#[cfg(target_vendor = "apple")]
fn index_format_to_mtl(format: IndexFormat) -> Option<MTLIndexType> {
    match format {
        IndexFormat::uint16 => Some(MTLIndexType::UInt16),
        IndexFormat::uint32 => Some(MTLIndexType::UInt32),
        IndexFormat::none => None,
    }
}

#[cfg(target_vendor = "apple")]
fn cull_mode_to_mtl(mode: crate::types::CullMode) -> MTLCullMode {
    match mode {
        crate::types::CullMode::none => MTLCullMode::None,
        crate::types::CullMode::front => MTLCullMode::Front,
        crate::types::CullMode::back => MTLCullMode::Back,
    }
}

#[cfg(target_vendor = "apple")]
fn winding_to_mtl(winding: crate::types::FaceWinding) -> MTLWinding {
    match winding {
        crate::types::FaceWinding::clockwise => MTLWinding::Clockwise,
        crate::types::FaceWinding::counterClockwise => MTLWinding::CounterClockwise,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_vendor = "apple")]
    use crate::gpu_resource::ResourceHandle;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn internal_active_pass_token_is_thread_safe() {
        assert_send_sync::<RenderPassMetalInner>();
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn live_encoder_covers_state_binding_draw_and_finish_ownership() {
        use crate::bind_group::BindGroup;
        use crate::metal::bind_group::{BindGroupMetal, MTLBufferBinding};
        use crate::metal::buffer::BufferMetalContextState;
        use crate::metal::texture::{TextureMetal, TextureViewMetal};
        use crate::types::{
            BufferUsage, ColorAttachment, Features, PipelineDesc, RenderPassDesc, TextureAspect,
            TextureDesc, TextureFormat, TextureViewDesc, TextureViewDimension,
        };
        use objc2_foundation::NSString;
        use objc2_metal::{
            MTLCommandBuffer, MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice, MTLLibrary,
            MTLDepthStencilDescriptor, MTLPixelFormat, MTLRenderPassDescriptor,
            MTLRenderPipelineDescriptor, MTLResourceOptions, MTLTextureDescriptor,
            MTLTextureUsage,
        };

        let Some(device) = MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let queue = device.newCommandQueue().expect("create command queue");
        let command_buffer = queue.commandBuffer().expect("create command buffer");
        let texture_desc = MTLTextureDescriptor::new();
        texture_desc.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        texture_desc.setUsage(MTLTextureUsage::RenderTarget);
        // SAFETY: the test uses non-zero, representable two-dimensional
        // extents and attachment zero, which always exists in Metal's fixed
        // render-pass color attachment table.
        unsafe {
            texture_desc.setWidth(4);
            texture_desc.setHeight(4);
        }
        let texture = device
            .newTextureWithDescriptor(&texture_desc)
            .expect("create render target");
        let portable_texture_desc = TextureDesc {
            width: 4,
            height: 4,
            format: TextureFormat::bgra8unorm,
            renderTarget: true,
            ..TextureDesc::default()
        };
        let portable_texture =
            TextureMetal::with_native_texture(&portable_texture_desc, texture.clone())
                .into_resource(None)
                .erase();
        let portable_view = TextureViewMetal::new(&TextureViewDesc {
            texture: &portable_texture,
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        })
        .into_resource(None)
        .erase();
        let pass_desc = RenderPassDesc {
            colorAttachments: [
                ColorAttachment {
                    view: Some(&portable_view),
                    ..ColorAttachment::default()
                },
                ColorAttachment::default(),
                ColorAttachment::default(),
                ColorAttachment::default(),
            ],
            ..RenderPassDesc::default()
        };
        let descriptor = MTLRenderPassDescriptor::new();
        // SAFETY: attachment index zero is within Metal's fixed color table.
        unsafe {
            descriptor
                .colorAttachments()
                .objectAtIndexedSubscript(0)
                .setTexture(Some(&texture));
        }
        let encoder = command_buffer
            .renderCommandEncoderWithDescriptor(&descriptor)
            .expect("create render encoder");
        let context = ContextState::new(Features::default(), None);
        let pass =
            RenderPassMetal::with_native_encoder(&context, &pass_desc, encoder, command_buffer);
        context.set_active_render_pass(pass.active_token());

        assert_eq!(
            pass.setPipeline(&portable_view),
            Err(RenderPassError::WrongBackendResource("pipeline"))
        );

        let source = NSString::from_str(
            "#include <metal_stdlib>\nusing namespace metal;\nvertex float4 vertex_main(uint vertex_id [[vertex_id]]) { float2 p[3] = {float2(-1, -1), float2(3, -1), float2(-1, 3)}; return float4(p[vertex_id % 3], 0, 1); }\nfragment float4 fragment_main() { return float4(1, 0, 0, 1); }",
        );
        let library = device
            .newLibraryWithSource_options_error(&source, None)
            .expect("compile render-pass test library");
        let vertex = library
            .newFunctionWithName(&NSString::from_str("vertex_main"))
            .expect("load vertex function");
        let fragment = library
            .newFunctionWithName(&NSString::from_str("fragment_main"))
            .expect("load fragment function");
        let native_pipeline_desc = MTLRenderPipelineDescriptor::new();
        native_pipeline_desc.setVertexFunction(Some(&vertex));
        native_pipeline_desc.setFragmentFunction(Some(&fragment));
        // SAFETY: color target zero is in Metal's fixed attachment table.
        unsafe {
            native_pipeline_desc
                .colorAttachments()
                .objectAtIndexedSubscript(0)
                .setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        }
        let native_pipeline = device
            .newRenderPipelineStateWithDescriptor_error(&native_pipeline_desc)
            .expect("create render pipeline");
        // Pinned ContextMetal::mtlMakePipeline always creates a depth/stencil
        // state, even for the default descriptor. Construct the same complete
        // native owner here; a nil test-only state violates the source owner
        // invariant and Metal validation correctly rejects binding it.
        let native_depth_stencil = device
            .newDepthStencilStateWithDescriptor(&MTLDepthStencilDescriptor::new())
            .expect("create default depth/stencil state");
        let pipeline = PipelineMetal::with_native_states(
            &PipelineDesc::default(),
            native_pipeline,
            Some(native_depth_stencil),
        )
        .into_resource(None)
        .erase();
        pass.setPipeline(&pipeline)
            .expect("bind compatible pipeline");
        assert_eq!(pipeline.debugging_ref_count(), 2);

        let native_buffer = device
            .newBufferWithLength_options(16, MTLResourceOptions::StorageModeShared)
            .expect("create draw buffer");
        let buffer = BufferMetal::with_native_buffer(
            16,
            BufferUsage::uniform,
            device.clone(),
            native_buffer,
            BufferMetalContextState::new(),
            None,
        )
        .into_resource(None);
        let erased_buffer = buffer.clone().erase();
        let rejected_group = BindGroupMetal::from_parts(
            BindGroup::from_parts(0, None, vec![erased_buffer.clone()], vec![], vec![]),
            vec![MTLBufferBinding::new(
                0,
                0,
                0,
                false,
                31,
                crate::binding_map::BindingMap::kAbsent,
            )],
            vec![],
            vec![],
        );
        let rejected_group = ResourceHandle::new(None, rejected_group).erase();
        let before_rejected_group = buffer.current_buffer();
        assert_eq!(
            pass.setBindGroup(0, &rejected_group, &[]),
            Err(RenderPassError::NativeBindingSlotOutOfRange {
                kind: "buffer",
                slot: 31,
                limit: 31,
            })
        );
        assert_eq!(rejected_group.debugging_ref_count(), 1);
        crate::types::Buffer::update(&*buffer, &[2], 0)
            .expect("a rejected group must not mark the buffer bound");
        assert_eq!(
            Retained::as_ptr(&before_rejected_group),
            Retained::as_ptr(&buffer.current_buffer()),
            "group slot validation must precede owner and buffer-state mutation"
        );
        let before_rejected_slot = buffer.current_buffer();
        assert_eq!(
            pass.setVertexBuffer(15, &erased_buffer, 0),
            Err(RenderPassError::VertexBufferSlotOutOfRange { slot: 15 })
        );
        crate::types::Buffer::update(&*buffer, &[1], 0)
            .expect("a rejected slot must not mark the buffer bound");
        assert_eq!(
            Retained::as_ptr(&before_rejected_slot),
            Retained::as_ptr(&buffer.current_buffer()),
            "slot validation must happen before buffer-state mutation"
        );
        pass.setVertexBuffer(0, &erased_buffer, 0)
            .expect("bind live vertex buffer");
        pass.setIndexBuffer(&erased_buffer, IndexFormat::uint16, 0)
            .expect("bind live index buffer");

        let group = BindGroupMetal::from_parts(
            BindGroup::from_parts(1, None, vec![erased_buffer.clone()], vec![], vec![]),
            vec![MTLBufferBinding::new(
                0,
                0,
                0,
                true,
                0,
                crate::binding_map::BindingMap::kAbsent,
            )],
            vec![],
            vec![],
        );
        let group = ResourceHandle::new(None, group).erase();
        pass.setBindGroup(0, &group, &[4])
            .expect("bind dynamic uniform group");
        pass.setViewport(0.0, 0.0, 4.0, 4.0, 0.0, 1.0)
            .expect("set viewport");
        pass.setScissorRect(0, 0, 4, 4).expect("set scissor");
        pass.setStencilReference(3).expect("set stencil reference");
        pass.setBlendColor(0.25, 0.5, 0.75, 1.0)
            .expect("set blend color");
        pass.draw(3, 1, 0, 0).expect("draw triangle");
        pass.drawIndexed(3, 1, 0, 0, 0)
            .expect("draw indexed triangle");
        assert_eq!(group.debugging_ref_count(), 2);

        context.finish_active_render_pass();
        assert!(pass.is_finished());
        assert_eq!(group.debugging_ref_count(), 1);
        assert_eq!(pipeline.debugging_ref_count(), 1);
        pass.finish();
        assert_eq!(group.debugging_ref_count(), 1);
        assert_eq!(
            pass.setViewport(0.0, 0.0, 4.0, 4.0, 0.0, 1.0),
            Err(RenderPassError::Finished)
        );
    }

    #[test]
    fn conversion_helpers_cover_every_source_enum_value() {
        #[cfg(target_vendor = "apple")]
        {
            assert_eq!(
                primitive_topology_to_mtl(crate::types::PrimitiveTopology::triangleStrip),
                MTLPrimitiveType::TriangleStrip
            );
            assert_eq!(index_format_to_mtl(IndexFormat::none), None);
            assert_eq!(
                cull_mode_to_mtl(crate::types::CullMode::back),
                MTLCullMode::Back
            );
            assert_eq!(
                winding_to_mtl(crate::types::FaceWinding::clockwise),
                MTLWinding::Clockwise
            );
        }
    }
}
