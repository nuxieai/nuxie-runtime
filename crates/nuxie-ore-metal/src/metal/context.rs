// Mechanical translation of:
//   renderer/include/rive/renderer/ore/ore_context_metal.hpp
//   renderer/src/ore/metal/ore_context_metal.mm
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

//! ORE Metal context, resource factories, and frame submission.
//!
//! Adaptations are deliberately narrow:
//! - Objective-C nullable factory results become `Option<AnyResourceHandle>`.
//!   Textures, buffers, shaders, and pipelines publish only after required
//!   native allocation succeeds; texture views and samplers preserve the
//!   source's live logical resource with a nullable native handle. Bind-group
//!   layout lookup failures preserve the source's skip-and-continue behavior
//!   and last-error-wins ordering across buffers, textures, then samplers.
//! - Source `assert`-only backend downcasts fail closed with a stable context
//!   error. This removes undefined behavior without introducing a HAL.
//! - `m_deferredBindGroups` is not recreated. The pinned tree declares and
//!   drains that vector but has no writer, so inventing a defer route would
//!   claim behavior absent from the source oracle.
//! - `beginFrame` intentionally ignores `FrameDescriptor` and does not clear
//!   `lastError`, matching the pinned Metal implementation despite the base
//!   header's contradictory error-reset comment.
//!
//! The host `RenderCanvas` and `gpu::Texture` C++ types are outside this
//! isolated crate. Their two wrappers converge here on `wrap_native_texture`,
//! a retained `MTLTexture` seam plus the width/height and render-target bit
//! those host objects supply. This preserves the native owner graph and exact
//! format fallback without importing the renderer implementation.

#![allow(non_snake_case)]

use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::bind_group::BindGroup;
use crate::bind_group_layout::{
    BindGroupLayout, validate_color_requires_fragment, validate_layouts_against_binding_map,
};
use crate::binding_map::BindingMap;
use crate::context::{ContextState, FrameDescriptor, ShaderTarget};
use crate::gpu_resource::{AnyResourceHandle, GpuResourceManager};
use crate::metal::bind_group::{
    BindGroupMetal, MTLBufferBinding, MTLSamplerBinding, MTLTextureBinding,
};
use crate::metal::buffer::{BufferErrorSink, BufferMetal, BufferMetalContextState};
use crate::metal::pipeline::PipelineMetal;
use crate::metal::render_pass::RenderPassMetal;
use crate::metal::sampler::SamplerMetal;
use crate::metal::shader_module::ShaderModuleMetal;
use crate::metal::texture::{TextureMetal, TextureViewMetal};
use crate::types::{
    BindGroupDesc, BindGroupLayoutDesc, BindGroupLayoutEntry, BindingKind, BlendFactor, BlendOp,
    BufferDesc, ColorWriteMask, CompareFunction, Features, Filter, LoadOp, PipelineDesc,
    RenderPassDesc, SamplerDesc, ShaderModuleDesc, StencilOp, StoreOp, TextureDesc, TextureFormat,
    TextureType, TextureViewDesc, TextureViewDimension, VertexFormat, VertexStepMode, WrapMode,
    kMaxBindGroups,
};

#[cfg(target_vendor = "apple")]
use objc2::rc::Retained;
#[cfg(target_vendor = "apple")]
use objc2::runtime::ProtocolObject;
#[cfg(target_vendor = "apple")]
use objc2_foundation::{NSRange, NSString};
#[cfg(target_vendor = "apple")]
use objc2_metal::{
    MTLBlendFactor, MTLBlendOperation, MTLClearColor, MTLColorWriteMask, MTLCommandBuffer,
    MTLCommandBufferStatus, MTLCommandEncoder, MTLCommandQueue, MTLCompareFunction,
    MTLDepthStencilDescriptor, MTLDevice, MTLLibrary, MTLLoadAction, MTLPixelFormat,
    MTLRenderPassDescriptor, MTLRenderPipelineDescriptor, MTLResource, MTLResourceOptions,
    MTLSamplerAddressMode, MTLSamplerDescriptor, MTLSamplerMinMagFilter, MTLSamplerMipFilter,
    MTLStencilDescriptor, MTLStencilOperation, MTLStorageMode, MTLStoreAction, MTLTexture,
    MTLTextureDescriptor, MTLTextureType, MTLTextureUsage, MTLVertexDescriptor, MTLVertexFormat,
    MTLVertexStepFunction,
};

const MAX_BINDINGS_PER_KIND: usize = 8;
const METAL_VERTEX_BUFFER_BASE: usize = 16;

#[cfg(target_vendor = "apple")]
struct RetainedMetalDevice(Retained<ProtocolObject<dyn MTLDevice>>);

#[cfg(target_vendor = "apple")]
struct RetainedMetalQueue(Retained<ProtocolObject<dyn MTLCommandQueue>>);

#[cfg(target_vendor = "apple")]
struct RetainedMetalCommandBuffer(Retained<ProtocolObject<dyn MTLCommandBuffer>>);

/// Product-facing completion token for one submitted Metal command buffer.
///
/// Pinned ORE publishes only a completed serial. The authenticated product
/// adapter additionally needs to distinguish successful completion from a
/// driver-reported command-buffer failure before publishing rendered pixels.
#[cfg(target_vendor = "apple")]
#[derive(Clone)]
pub struct MetalSubmissionCompletion {
    result: Arc<Mutex<Option<Result<(), String>>>>,
}

#[cfg(target_vendor = "apple")]
impl MetalSubmissionCompletion {
    pub fn result(&self) -> Option<Result<(), String>> {
        self.result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn complete(&self, result: Result<(), String>) {
        *self
            .result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(result);
    }
}

#[cfg(target_vendor = "apple")]
fn command_buffer_completion_result(
    status: MTLCommandBufferStatus,
    error: Option<String>,
) -> Result<(), String> {
    if status == MTLCommandBufferStatus::Completed {
        return Ok(());
    }
    Err(error.unwrap_or_else(|| format!("Metal command buffer completed with status {status:?}")))
}

/// Concrete Metal ORE context.
///
/// Recording is caller-serialized and the context is intentionally neither
/// `Send` nor `Sync`.
///
/// ```compile_fail
/// fn require_send_sync<T: Send + Sync>() {}
/// require_send_sync::<nuxie_ore_metal::metal::context::ContextMetal>();
/// ```
pub struct ContextMetal {
    // Rust fields drop in declaration order. Match ContextMetal's explicit
    // destructor release of command buffer, queue, device, followed by the
    // completion state and finally the portable Context base.
    #[cfg(target_vendor = "apple")]
    command_buffer: Mutex<Option<RetainedMetalCommandBuffer>>,
    #[cfg(target_vendor = "apple")]
    queue: RetainedMetalQueue,
    #[cfg(target_vendor = "apple")]
    device: RetainedMetalDevice,
    buffer_state: Arc<BufferMetalContextState>,
    state: Arc<ContextState>,
    // The source context is a caller-serialized recorder. Keep the public
    // object thread-bound while its narrow completion/error tokens stay
    // independently Send + Sync.
    _recording_thread: PhantomData<Rc<()>>,
}

impl ContextMetal {
    #[cfg(target_vendor = "apple")]
    pub fn make(
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    ) -> Self {
        Self::make_with_manager(device, queue, None)
    }

    #[cfg(target_vendor = "apple")]
    pub(crate) fn make_with_manager(
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
        manager: Option<GpuResourceManager>,
    ) -> Self {
        let features = populate_features(&device);
        let state = ContextState::new(features, manager);
        let error_sink: Arc<dyn BufferErrorSink> = state.clone();
        let buffer_state = BufferMetalContextState::with_error_sink(Arc::downgrade(&error_sink));
        Self {
            command_buffer: Mutex::new(None),
            queue: RetainedMetalQueue(queue),
            device: RetainedMetalDevice(device),
            buffer_state,
            state,
            _recording_thread: PhantomData,
        }
    }

    pub fn features(&self) -> &Features {
        self.state.features()
    }

    pub fn shader_target(&self) -> ShaderTarget {
        ShaderTarget::msl
    }

    pub fn shaderTarget(&self) -> ShaderTarget {
        self.shader_target()
    }

    pub fn current_serial(&self) -> u64 {
        self.buffer_state.current_serial()
    }

    pub fn currentSerial(&self) -> u64 {
        self.current_serial()
    }

    #[cfg(target_vendor = "apple")]
    pub fn completed_serial(&self) -> u64 {
        self.buffer_state.completed_serial()
    }

    #[cfg(target_vendor = "apple")]
    pub fn completedSerial(&self) -> u64 {
        self.completed_serial()
    }

    pub fn last_error(&self) -> String {
        self.state.last_error()
    }

    pub fn lastError(&self) -> String {
        self.last_error()
    }

    pub fn clear_last_error(&self) {
        self.state.clear_last_error();
    }

    pub fn clearLastError(&self) {
        self.clear_last_error();
    }

    #[cfg(target_vendor = "apple")]
    pub(crate) fn current_command_buffer(
        &self,
    ) -> Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>> {
        self.lock_command_buffer()
            .as_ref()
            .map(|command_buffer| command_buffer.0.clone())
    }

    /// Start recording a new queue-owned command buffer.
    ///
    /// The descriptor is intentionally ignored, exactly as in the pinned
    /// Objective-C++ implementation.
    #[cfg(target_vendor = "apple")]
    pub fn begin_frame(&self, _descriptor: &FrameDescriptor) {
        *self.lock_command_buffer() = self.queue.0.commandBuffer().map(RetainedMetalCommandBuffer);
        // Pinned source is ordinary unsigned `++m_currentSerial`; preserve
        // its defined uint64 rollover rather than introducing a failure.
        let serial = self.current_serial().wrapping_add(1);
        self.buffer_state.set_current_serial(serial);
    }

    #[cfg(target_vendor = "apple")]
    pub fn beginFrame(&self, descriptor: &FrameDescriptor) {
        self.begin_frame(descriptor);
    }

    #[cfg(target_vendor = "apple")]
    pub fn wait_for_gpu(&self) {
        if let Some(command_buffer) = self.lock_command_buffer().as_ref() {
            command_buffer.0.waitUntilCompleted();
        }
    }

    #[cfg(target_vendor = "apple")]
    pub fn waitForGPU(&self) {
        self.wait_for_gpu();
    }

    #[cfg(target_vendor = "apple")]
    pub fn end_frame(&self) {
        let _ = self.end_frame_with_completion();
    }

    /// Commit the current frame and return a token that reports native success.
    ///
    /// This is a narrow corrective product seam over pinned ORE's serial-only
    /// completion callback. It does not change `end_frame`'s submission order.
    #[cfg(target_vendor = "apple")]
    pub fn end_frame_with_completion(&self) -> Option<MetalSubmissionCompletion> {
        let Some(command_buffer) = self.lock_command_buffer().take() else {
            return None;
        };
        let finished_serial = self.current_serial();
        let completion_state = Arc::clone(&self.buffer_state);
        let submission = MetalSubmissionCompletion {
            result: Arc::new(Mutex::new(None)),
        };
        let submission_for_handler = submission.clone();
        let completion = block2::RcBlock::new(
            move |buffer: std::ptr::NonNull<ProtocolObject<dyn MTLCommandBuffer>>| {
                // SAFETY: Metal invokes the copied completion block with the
                // non-null command buffer retained for the callback duration.
                let buffer = unsafe { buffer.as_ref() };
                let result = command_buffer_completion_result(
                    buffer.status(),
                    buffer
                        .error()
                        .map(|error| format!("Metal command buffer failed: {error:?}")),
                );
                submission_for_handler.complete(result);
                completion_state.complete_serial(finished_serial);
            },
        );
        // SAFETY: Metal copies the heap block and supplies a non-null command
        // buffer while invoking it. Every captured value is owned and
        // thread-safe for the completion-handler lifetime.
        unsafe {
            command_buffer
                .0
                .addCompletedHandler(std::ptr::from_ref(&*completion).cast_mut());
        }
        command_buffer.0.commit();
        Some(submission)
    }

    #[cfg(target_vendor = "apple")]
    pub fn endFrame(&self) {
        self.end_frame();
    }

    #[cfg(target_vendor = "apple")]
    pub fn make_buffer(&self, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        let initial = if let Some(data) = desc.data() {
            let pointer = std::ptr::NonNull::new(data.as_ptr().cast_mut().cast())?;
            // SAFETY: `data` is valid for `desc.size()` bytes, and Metal copies
            // the complete slice before this synchronous call returns.
            unsafe {
                self.device.0.newBufferWithBytes_length_options(
                    pointer,
                    desc.size() as usize,
                    MTLResourceOptions::StorageModeShared,
                )
            }
        } else {
            self.device.0.newBufferWithLength_options(
                desc.size() as usize,
                MTLResourceOptions::StorageModeShared,
            )
        }?;
        if let Some(label) = desc.label {
            initial.setLabel(Some(&NSString::from_str(label)));
        }
        Some(
            BufferMetal::with_native_buffer(
                desc.size(),
                desc.usage,
                self.device.0.clone(),
                initial,
                Arc::clone(&self.buffer_state),
                desc.label,
            )
            .into_resource(self.state.manager())
            .erase(),
        )
    }

    #[cfg(target_vendor = "apple")]
    pub fn makeBuffer(&self, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        self.make_buffer(desc)
    }

    #[cfg(target_vendor = "apple")]
    pub fn make_texture(&self, desc: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        let is_msaa = desc.sampleCount > 1 && desc.r#type == TextureType::texture2D;
        let mipmap_level_count = if is_msaa { 1 } else { desc.numMipmaps };
        if desc.width == 0
            || desc.height == 0
            || desc.depthOrArrayLayers == 0
            || mipmap_level_count == 0
            || desc.sampleCount == 0
        {
            self.state.set_last_error(
                "makeTexture: dimensions, layers, mip levels, and sample count must be non-zero",
            );
            return None;
        }
        let descriptor = MTLTextureDescriptor::new();
        descriptor.setTextureType(if is_msaa {
            MTLTextureType::Type2DMultisample
        } else {
            texture_type_to_mtl(desc.r#type)
        });
        descriptor.setPixelFormat(format_to_mtl(desc.format));
        // SAFETY: the checks above establish objc2-metal's nonzero size/count
        // preconditions, and every u32-to-NSUInteger conversion is lossless.
        unsafe {
            descriptor.setWidth(desc.width as usize);
            descriptor.setHeight(desc.height as usize);
            descriptor.setMipmapLevelCount(mipmap_level_count as usize);
            descriptor.setSampleCount(desc.sampleCount as usize);
            match desc.r#type {
                TextureType::texture3D => {
                    descriptor.setDepth(desc.depthOrArrayLayers as usize);
                    descriptor.setArrayLength(1);
                }
                TextureType::array2D => {
                    descriptor.setDepth(1);
                    descriptor.setArrayLength(desc.depthOrArrayLayers as usize);
                }
                TextureType::texture2D | TextureType::cube => {
                    descriptor.setDepth(1);
                    descriptor.setArrayLength(1);
                }
            }
        }
        descriptor.setStorageMode(if desc.renderTarget {
            MTLStorageMode::Private
        } else {
            MTLStorageMode::Shared
        });
        let mut usage = MTLTextureUsage::ShaderRead;
        if desc.renderTarget {
            usage |= MTLTextureUsage::RenderTarget;
        }
        if matches!(
            desc.format,
            TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
        ) {
            usage |= MTLTextureUsage::PixelFormatView;
        }
        descriptor.setUsage(usage);
        let native = self.device.0.newTextureWithDescriptor(&descriptor)?;
        if let Some(label) = desc.label {
            native.setLabel(Some(&NSString::from_str(label)));
        }
        Some(
            TextureMetal::with_native_texture(desc, native)
                .into_resource(self.state.manager())
                .erase(),
        )
    }

    #[cfg(target_vendor = "apple")]
    pub fn makeTexture(&self, desc: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        self.make_texture(desc)
    }

    /// Shared native seam for `wrapCanvasTexture` and `wrapRiveTexture`.
    #[cfg(target_vendor = "apple")]
    pub fn wrap_native_texture(
        &self,
        native: Retained<ProtocolObject<dyn MTLTexture>>,
        width: u32,
        height: u32,
        render_target: bool,
    ) -> AnyResourceHandle {
        let texture_desc = TextureDesc {
            width,
            height,
            format: format_from_mtl(native.pixelFormat()),
            r#type: TextureType::texture2D,
            renderTarget: render_target,
            numMipmaps: 1,
            sampleCount: 1,
            ..TextureDesc::default()
        };
        let texture = TextureMetal::with_native_texture(&texture_desc, native.clone())
            .into_resource(self.state.manager())
            .erase();
        let view_desc = TextureViewDesc {
            texture: &texture,
            dimension: TextureViewDimension::texture2D,
            aspect: crate::types::TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        };
        TextureViewMetal::with_native_texture_view(&view_desc, native)
            .into_resource(self.state.manager())
            .erase()
    }

    #[cfg(target_vendor = "apple")]
    pub fn make_texture_view(&self, desc: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        let Some(texture) = desc.texture.downcast_ref::<TextureMetal>() else {
            self.state
                .set_last_error("makeTextureView: texture is not a Metal texture");
            return None;
        };
        let Some(source) = texture.mtlTexture() else {
            self.state
                .set_last_error("makeTextureView: source Metal texture is nil");
            return None;
        };
        let native = if source.textureType() == MTLTextureType::Type2DMultisample {
            None
        } else {
            let mip_range = NSRange {
                location: desc.baseMipLevel as usize,
                length: desc.mipCount as usize,
            };
            let slice_range = NSRange {
                location: desc.baseLayer as usize,
                length: desc.layerCount as usize,
            };
            // SAFETY: descriptor ranges are forwarded losslessly. Metal
            // returns nil when the view is incompatible or out of range.
            unsafe {
                source.newTextureViewWithPixelFormat_textureType_levels_slices(
                    source.pixelFormat(),
                    view_dimension_to_mtl(desc.dimension),
                    mip_range,
                    slice_range,
                )
            }
        };
        Some(self.publish_texture_view(desc, native))
    }

    #[cfg(target_vendor = "apple")]
    pub fn makeTextureView(&self, desc: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        self.make_texture_view(desc)
    }

    #[cfg(target_vendor = "apple")]
    pub fn make_sampler(&self, desc: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        let descriptor = MTLSamplerDescriptor::new();
        descriptor.setMinFilter(filter_to_mtl(desc.minFilter));
        descriptor.setMagFilter(filter_to_mtl(desc.magFilter));
        descriptor.setMipFilter(mip_filter_to_mtl(desc.mipmapFilter));
        descriptor.setSAddressMode(wrap_to_mtl(desc.wrapU));
        descriptor.setTAddressMode(wrap_to_mtl(desc.wrapV));
        descriptor.setRAddressMode(wrap_to_mtl(desc.wrapW));
        descriptor.setLodMinClamp(desc.minLod);
        descriptor.setLodMaxClamp(desc.maxLod);
        descriptor.setMaxAnisotropy(desc.maxAnisotropy as usize);
        if desc.compare != CompareFunction::none {
            descriptor.setCompareFunction(compare_to_mtl(desc.compare));
        }
        if let Some(label) = desc.label {
            descriptor.setLabel(Some(&NSString::from_str(label)));
        }
        let native = self.device.0.newSamplerStateWithDescriptor(&descriptor);
        Some(self.publish_sampler(native))
    }

    #[cfg(target_vendor = "apple")]
    pub fn makeSampler(&self, desc: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        self.make_sampler(desc)
    }

    #[cfg(target_vendor = "apple")]
    fn publish_texture_view(
        &self,
        desc: &TextureViewDesc<'_>,
        native: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
    ) -> AnyResourceHandle {
        match native {
            Some(native) => TextureViewMetal::with_native_texture_view(desc, native),
            None => TextureViewMetal::new(desc),
        }
        .into_resource(self.state.manager())
        .erase()
    }

    #[cfg(target_vendor = "apple")]
    fn publish_sampler(
        &self,
        native: Option<Retained<ProtocolObject<dyn objc2_metal::MTLSamplerState>>>,
    ) -> AnyResourceHandle {
        match native {
            Some(native) => SamplerMetal::with_native_sampler_state(native),
            None => SamplerMetal::new(),
        }
        .into_resource(self.state.manager())
        .erase()
    }

    #[cfg(target_vendor = "apple")]
    pub fn make_shader_module(&self, desc: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        let code = desc.code?;
        let source = std::str::from_utf8(code).ok()?;
        let native = match self
            .device
            .0
            .newLibraryWithSource_options_error(&NSString::from_str(source), None)
        {
            Ok(library) => library,
            Err(_error) => return None,
        };
        let mut module = ShaderModuleMetal::from_compiled_library(Some(native))
            .expect("a non-null retained Metal library must publish a shader module");
        module.apply_binding_map_from_desc(desc);
        Some(module.into_resource(self.state.manager()).erase())
    }

    #[cfg(target_vendor = "apple")]
    pub fn makeShaderModule(&self, desc: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        self.make_shader_module(desc)
    }

    #[cfg(target_vendor = "apple")]
    pub fn make_pipeline(
        &self,
        desc: &PipelineDesc<'_>,
        mut out_error: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        if desc.colorCount > desc.colorTargets.len() as u32 {
            report_pipeline_validation(
                &self.state,
                &mut out_error,
                format!(
                    "colorCount {} exceeds the {} color-target slots",
                    desc.colorCount,
                    desc.colorTargets.len()
                ),
            );
            return None;
        }
        if desc.bindGroupLayouts.is_some_and(|layouts| {
            layouts.len() > usize::try_from(kMaxBindGroups).expect("small constant")
        }) {
            report_pipeline_validation(
                &self.state,
                &mut out_error,
                format!("bindGroupLayouts exceeds kMaxBindGroups ({kMaxBindGroups})"),
            );
            return None;
        }

        let empty_binding_map = BindingMap::default();
        let binding_map = desc
            .vertexModule
            .or(desc.fragmentModule)
            .and_then(|module| module.downcast_ref::<ShaderModuleMetal>())
            .map_or(&empty_binding_map, |module| &module.m_bindingMap);
        let mut layouts = Vec::new();
        if let Some(desc_layouts) = desc.bindGroupLayouts {
            for (index, handle) in desc_layouts.iter().enumerate() {
                match handle {
                    Some(handle) => {
                        let Some(layout) = handle.downcast_ref::<BindGroupLayout>() else {
                            report_pipeline_validation(
                                &self.state,
                                &mut out_error,
                                format!(
                                    "PipelineDesc::bindGroupLayouts[{index}] is not a Metal BindGroupLayout"
                                ),
                            );
                            return None;
                        };
                        layouts.push(Some(layout));
                    }
                    None => layouts.push(None),
                }
            }
        }
        let mut validation_error = String::new();
        if !validate_layouts_against_binding_map(
            binding_map,
            Some(&layouts),
            layouts.len() as u32,
            Some(&mut validation_error),
        ) || !validate_color_requires_fragment(
            desc.colorCount,
            desc.fragmentModule.is_some(),
            Some(&mut validation_error),
        ) {
            report_pipeline_validation(&self.state, &mut out_error, validation_error);
            return None;
        }

        let Some(vertex_handle) = desc.vertexModule else {
            report_pipeline_validation(
                &self.state,
                &mut out_error,
                "vertex shader module is null".to_owned(),
            );
            return None;
        };
        let Some(vertex_module) = vertex_handle.downcast_ref::<ShaderModuleMetal>() else {
            report_pipeline_validation(
                &self.state,
                &mut out_error,
                "vertex shader module is not a Metal shader module".to_owned(),
            );
            return None;
        };

        let Some(vertex_library) = vertex_module.mtlLibrary() else {
            report_pipeline_native(&mut out_error, "vertex shader library is nil".to_owned());
            return None;
        };
        let Some(vertex_entry) = desc.vertexEntryPoint else {
            report_pipeline_native(&mut out_error, "vertex entry point is null".to_owned());
            return None;
        };
        let Some(vertex_function) =
            vertex_library.newFunctionWithName(&NSString::from_str(vertex_entry))
        else {
            report_pipeline_native(
                &mut out_error,
                format!("vertex entry point '{vertex_entry}' not found in shader library"),
            );
            return None;
        };

        let descriptor = MTLRenderPipelineDescriptor::new();
        descriptor.setVertexFunction(Some(&vertex_function));
        if let Some(fragment_handle) = desc.fragmentModule {
            let Some(fragment_module) = fragment_handle.downcast_ref::<ShaderModuleMetal>() else {
                report_pipeline_native(
                    &mut out_error,
                    "fragment shader module is not a Metal shader module".to_owned(),
                );
                return None;
            };
            let Some(fragment_library) = fragment_module.mtlLibrary() else {
                report_pipeline_native(&mut out_error, "fragment shader library is nil".to_owned());
                return None;
            };
            let Some(fragment_entry) = desc.fragmentEntryPoint else {
                report_pipeline_native(&mut out_error, "fragment entry point is null".to_owned());
                return None;
            };
            let Some(fragment_function) =
                fragment_library.newFunctionWithName(&NSString::from_str(fragment_entry))
            else {
                report_pipeline_native(
                    &mut out_error,
                    format!("fragment entry point '{fragment_entry}' not found in shader library"),
                );
                return None;
            };
            descriptor.setFragmentFunction(Some(&fragment_function));
        }

        if let Some(vertex_buffers) = desc.vertexBuffers.filter(|layouts| !layouts.is_empty()) {
            let vertex_descriptor = MTLVertexDescriptor::new();
            let native_layouts = vertex_descriptor.layouts();
            let native_attributes = vertex_descriptor.attributes();
            for (buffer_index, layout) in vertex_buffers.iter().enumerate() {
                let metal_index = buffer_index
                    .checked_add(METAL_VERTEX_BUFFER_BASE)
                    .expect("vertex-buffer index overflow");
                // SAFETY: Metal exposes 31 vertex-buffer layout slots and the
                // translated context reserves 0..16 for shader buffers.
                if metal_index >= 31 {
                    report_pipeline_native(
                        &mut out_error,
                        "vertex buffer count exceeds Metal's reserved slot range".to_owned(),
                    );
                    return None;
                }
                let native_layout = unsafe { native_layouts.objectAtIndexedSubscript(metal_index) };
                unsafe {
                    native_layout.setStride(layout.stride as usize);
                    native_layout.setStepRate(1);
                }
                native_layout.setStepFunction(match layout.stepMode {
                    VertexStepMode::vertex => MTLVertexStepFunction::PerVertex,
                    VertexStepMode::instance => MTLVertexStepFunction::PerInstance,
                });
                for attribute in layout.attributes {
                    if attribute.shaderSlot >= self.features().maxVertexAttributes {
                        report_pipeline_native(
                            &mut out_error,
                            format!(
                                "vertex attribute slot {} exceeds Metal limit {}",
                                attribute.shaderSlot,
                                self.features().maxVertexAttributes
                            ),
                        );
                        return None;
                    }
                    let native_attribute = unsafe {
                        native_attributes.objectAtIndexedSubscript(attribute.shaderSlot as usize)
                    };
                    native_attribute.setFormat(vertex_format_to_mtl(attribute.format));
                    unsafe {
                        native_attribute.setOffset(attribute.offset as usize);
                        native_attribute.setBufferIndex(metal_index);
                    }
                }
            }
            descriptor.setVertexDescriptor(Some(&vertex_descriptor));
        }

        let color_attachments = descriptor.colorAttachments();
        for index in 0..desc.colorCount as usize {
            // SAFETY: `colorCount` was range-checked against the fixed array.
            let attachment = unsafe { color_attachments.objectAtIndexedSubscript(index) };
            let target = desc
                .colorTargets
                .get(index)
                .expect("colorCount was checked against colorTargets");
            attachment.setPixelFormat(format_to_mtl(target.format));
            attachment.setWriteMask(color_write_mask_to_mtl(target.writeMask));
            if target.blendEnabled {
                attachment.setBlendingEnabled(true);
                attachment.setSourceRGBBlendFactor(blend_factor_to_mtl(target.blend.srcColor));
                attachment.setDestinationRGBBlendFactor(blend_factor_to_mtl(target.blend.dstColor));
                attachment.setRgbBlendOperation(blend_op_to_mtl(target.blend.colorOp));
                attachment.setSourceAlphaBlendFactor(blend_factor_to_mtl(target.blend.srcAlpha));
                attachment
                    .setDestinationAlphaBlendFactor(blend_factor_to_mtl(target.blend.dstAlpha));
                attachment.setAlphaBlendOperation(blend_op_to_mtl(target.blend.alphaOp));
            }
        }

        let has_depth_stencil = desc.depthStencil.format != TextureFormat::rgba8unorm;
        if has_depth_stencil {
            descriptor.setDepthAttachmentPixelFormat(format_to_mtl(desc.depthStencil.format));
            if matches!(
                desc.depthStencil.format,
                TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
            ) {
                descriptor.setStencilAttachmentPixelFormat(format_to_mtl(desc.depthStencil.format));
            }
        }
        descriptor.setRasterSampleCount(desc.sampleCount as usize);
        if let Some(label) = desc.label {
            descriptor.setLabel(Some(&NSString::from_str(label)));
        }

        let native_pipeline = match self
            .device
            .0
            .newRenderPipelineStateWithDescriptor_error(&descriptor)
        {
            Ok(pipeline) => pipeline,
            Err(error) => {
                report_pipeline_native(&mut out_error, error.localizedDescription().to_string());
                return None;
            }
        };

        let depth_descriptor = MTLDepthStencilDescriptor::new();
        depth_descriptor.setDepthCompareFunction(compare_to_mtl(desc.depthStencil.depthCompare));
        depth_descriptor.setDepthWriteEnabled(desc.depthStencil.depthWriteEnabled);
        let front = stencil_descriptor(
            desc.stencilFront.compare,
            desc.stencilFront.failOp,
            desc.stencilFront.depthFailOp,
            desc.stencilFront.passOp,
            desc.stencilReadMask,
            desc.stencilWriteMask,
        );
        depth_descriptor.setFrontFaceStencil(Some(&front));
        let back = stencil_descriptor(
            desc.stencilBack.compare,
            desc.stencilBack.failOp,
            desc.stencilBack.depthFailOp,
            desc.stencilBack.passOp,
            desc.stencilReadMask,
            desc.stencilWriteMask,
        );
        depth_descriptor.setBackFaceStencil(Some(&back));
        let depth_state = self
            .device
            .0
            .newDepthStencilStateWithDescriptor(&depth_descriptor);
        Some(
            PipelineMetal::with_native_states(desc, native_pipeline, depth_state)
                .into_resource(self.state.manager())
                .erase(),
        )
    }

    #[cfg(target_vendor = "apple")]
    pub fn makePipeline(
        &self,
        desc: &PipelineDesc<'_>,
        out_error: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        self.make_pipeline(desc, out_error)
    }

    #[cfg(target_vendor = "apple")]
    pub fn begin_render_pass(
        &self,
        desc: &RenderPassDesc<'_>,
        _out_error: Option<&mut String>,
    ) -> Option<RenderPassMetal> {
        self.state.finish_active_render_pass();
        if desc.colorCount > desc.colorAttachments.len() as u32 {
            self.state.set_last_error(format!(
                "beginRenderPass: colorCount {} exceeds the {} attachment slots",
                desc.colorCount,
                desc.colorAttachments.len()
            ));
            return None;
        }
        let Some(command_buffer) = self.current_command_buffer() else {
            self.state
                .set_last_error("beginRenderPass: beginFrame has not created a command buffer");
            return None;
        };
        let descriptor = MTLRenderPassDescriptor::new();
        let color_attachments = descriptor.colorAttachments();
        for (index, attachment) in desc
            .colorAttachments
            .iter()
            .take(desc.colorCount as usize)
            .enumerate()
        {
            let Some(view_handle) = attachment.view else {
                self.state.set_last_error(format!(
                    "beginRenderPass: color attachment {index} view is null"
                ));
                return None;
            };
            let Some(view) = view_handle.downcast_ref::<TextureViewMetal>() else {
                self.state.set_last_error(format!(
                    "beginRenderPass: color attachment {index} is not a Metal texture view"
                ));
                return None;
            };
            let Some(native) = view.mtlTexture() else {
                self.state.set_last_error(format!(
                    "beginRenderPass: color attachment {index} has no native texture"
                ));
                return None;
            };
            let Some(base_texture) = view
                .base()
                .texture()
                .downcast_ref::<TextureMetal>()
                .and_then(TextureMetal::mtlTexture)
            else {
                self.state.set_last_error(format!(
                    "beginRenderPass: color attachment {index} has no Metal base texture"
                ));
                return None;
            };
            // SAFETY: colorCount was bounded by Ore's fixed attachment array.
            let native_attachment = unsafe { color_attachments.objectAtIndexedSubscript(index) };
            native_attachment.setTexture(Some(native));
            let has_view = !std::ptr::eq(native, base_texture);
            native_attachment.setLevel(if has_view {
                0
            } else {
                view.base().baseMipLevel() as usize
            });
            native_attachment.setSlice(if has_view {
                0
            } else {
                view.base().baseLayer() as usize
            });
            native_attachment.setLoadAction(load_op_to_mtl(attachment.loadOp));
            native_attachment.setStoreAction(store_op_to_mtl(attachment.storeOp));
            native_attachment.setClearColor(MTLClearColor {
                red: f64::from(attachment.clearColor.r),
                green: f64::from(attachment.clearColor.g),
                blue: f64::from(attachment.clearColor.b),
                alpha: f64::from(attachment.clearColor.a),
            });

            if let Some(resolve_handle) = attachment.resolveTarget {
                let Some(resolve_view) = resolve_handle.downcast_ref::<TextureViewMetal>() else {
                    self.state.set_last_error(format!(
                        "beginRenderPass: resolve attachment {index} is not a Metal texture view"
                    ));
                    return None;
                };
                let Some(resolve_native) = resolve_view.mtlTexture() else {
                    self.state.set_last_error(format!(
                        "beginRenderPass: resolve attachment {index} has no native texture"
                    ));
                    return None;
                };
                let Some(resolve_base) = resolve_view
                    .base()
                    .texture()
                    .downcast_ref::<TextureMetal>()
                    .and_then(TextureMetal::mtlTexture)
                else {
                    self.state.set_last_error(format!(
                        "beginRenderPass: resolve attachment {index} has no Metal base texture"
                    ));
                    return None;
                };
                native_attachment.setResolveTexture(Some(resolve_native));
                let resolve_has_view = !std::ptr::eq(resolve_native, resolve_base);
                native_attachment.setResolveLevel(if resolve_has_view {
                    0
                } else {
                    resolve_view.base().baseMipLevel() as usize
                });
                native_attachment.setResolveSlice(if resolve_has_view {
                    0
                } else {
                    resolve_view.base().baseLayer() as usize
                });
                native_attachment.setStoreAction(if attachment.storeOp == StoreOp::store {
                    MTLStoreAction::StoreAndMultisampleResolve
                } else {
                    MTLStoreAction::MultisampleResolve
                });
            }
        }

        if let Some(depth_handle) = desc.depthStencil.view {
            let Some(view) = depth_handle.downcast_ref::<TextureViewMetal>() else {
                self.state.set_last_error(
                    "beginRenderPass: depth attachment is not a Metal texture view",
                );
                return None;
            };
            let Some(native) = view.mtlTexture() else {
                self.state
                    .set_last_error("beginRenderPass: depth attachment has no native texture");
                return None;
            };
            let Some(base_texture) = view.base().texture().downcast_ref::<TextureMetal>() else {
                self.state
                    .set_last_error("beginRenderPass: depth attachment has no Metal base texture");
                return None;
            };
            let Some(base_native) = base_texture.mtlTexture() else {
                self.state
                    .set_last_error("beginRenderPass: depth base texture has no native texture");
                return None;
            };
            let has_view = !std::ptr::eq(native, base_native);
            let depth = descriptor.depthAttachment();
            depth.setTexture(Some(native));
            depth.setLevel(if has_view {
                0
            } else {
                view.base().baseMipLevel() as usize
            });
            depth.setSlice(if has_view {
                0
            } else {
                view.base().baseLayer() as usize
            });
            depth.setLoadAction(load_op_to_mtl(desc.depthStencil.depthLoadOp));
            depth.setStoreAction(store_op_to_mtl(desc.depthStencil.depthStoreOp));
            depth.setClearDepth(f64::from(desc.depthStencil.depthClearValue));

            if matches!(
                base_texture.base().format(),
                TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
            ) {
                let stencil = descriptor.stencilAttachment();
                stencil.setTexture(Some(native));
                stencil.setLevel(if has_view {
                    0
                } else {
                    view.base().baseMipLevel() as usize
                });
                stencil.setSlice(if has_view {
                    0
                } else {
                    view.base().baseLayer() as usize
                });
                stencil.setLoadAction(load_op_to_mtl(desc.depthStencil.stencilLoadOp));
                stencil.setStoreAction(store_op_to_mtl(desc.depthStencil.stencilStoreOp));
                stencil.setClearStencil(desc.depthStencil.stencilClearValue);
            }
        }

        let Some(encoder) = command_buffer.renderCommandEncoderWithDescriptor(&descriptor) else {
            self.state
                .set_last_error("beginRenderPass: renderCommandEncoderWithDescriptor returned nil");
            return None;
        };
        if let Some(label) = desc.label {
            encoder.setLabel(Some(&NSString::from_str(label)));
        }
        let pass = RenderPassMetal::with_native_encoder(&self.state, desc, encoder, command_buffer);
        self.state.set_active_render_pass(pass.active_token());
        Some(pass)
    }

    #[cfg(target_vendor = "apple")]
    pub fn beginRenderPass(
        &self,
        desc: &RenderPassDesc<'_>,
        out_error: Option<&mut String>,
    ) -> Option<RenderPassMetal> {
        self.begin_render_pass(desc, out_error)
    }

    pub fn make_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDesc<'_>,
    ) -> Option<AnyResourceHandle> {
        if desc.groupIndex >= kMaxBindGroups {
            self.state.set_last_error(format!(
                "makeBindGroupLayout: groupIndex {} out of range [0, {})",
                desc.groupIndex, kMaxBindGroups
            ));
            return None;
        }
        for entry in desc.entries {
            for (stage, slot) in [
                ("nativeSlotVS", entry.nativeSlotVS),
                ("nativeSlotFS", entry.nativeSlotFS),
                ("nativeSlotCS", entry.nativeSlotCS),
            ] {
                if native_slot(entry.kind, slot).is_none() {
                    let limit = native_slot_limit(entry.kind);
                    self.state.set_last_error(format!(
                        "makeBindGroupLayout: binding {} {} {stage} {slot} out of range [0, {limit})",
                        entry.binding,
                        binding_kind_name(entry.kind),
                    ));
                    return None;
                }
            }
        }
        Some(
            BindGroupLayout::from_context_entries(desc.groupIndex, desc.entries)
                .into_resource(self.state.manager())
                .erase(),
        )
    }

    pub fn makeBindGroupLayout(&self, desc: &BindGroupLayoutDesc<'_>) -> Option<AnyResourceHandle> {
        self.make_bind_group_layout(desc)
    }

    #[cfg(target_vendor = "apple")]
    pub fn make_bind_group(&self, desc: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        let Some(layout_handle) = desc.layout else {
            self.state
                .set_last_error("makeBindGroup: BindGroupDesc::layout is null");
            return None;
        };
        let Some(layout) = layout_handle.downcast_ref::<BindGroupLayout>() else {
            self.state
                .set_last_error("makeBindGroup: layout is not a Metal BindGroupLayout");
            return None;
        };
        let group_index = layout.group_index();
        if group_index >= kMaxBindGroups {
            self.state.set_last_error(format!(
                "makeBindGroup: layout->groupIndex {group_index} out of range"
            ));
            return None;
        }

        let mut retained_buffers = Vec::new();
        let mut buffers = Vec::new();
        let mut dynamic_offset_count = 0_u32;
        for entry in desc.ubos.iter().take(MAX_BINDINGS_PER_KIND) {
            let Some(handle) = entry.buffer else {
                self.state.set_last_error(format!(
                    "makeBindGroup: (group={group_index}, binding={}) buffer is null",
                    entry.slot
                ));
                return None;
            };
            if handle.downcast_ref::<BufferMetal>().is_none() {
                self.state.set_last_error(format!(
                    "makeBindGroup: (group={group_index}, binding={}) buffer is not a Metal buffer",
                    entry.slot
                ));
                return None;
            }
            let Some((vs_slot, fs_slot)) =
                self.lookup_stages(layout, group_index, entry.slot, BindingKind::uniformBuffer)
            else {
                continue;
            };
            let dynamic = layout.has_dynamic_offset(entry.slot);
            dynamic_offset_count = dynamic_offset_count
                .checked_add(u32::from(dynamic))
                .expect("at most eight dynamic bindings fit in u32");
            let source_index = retained_buffers.len();
            retained_buffers.push(handle.clone());
            buffers.push(MTLBufferBinding::new(
                source_index,
                entry.offset,
                entry.slot,
                dynamic,
                vs_slot,
                fs_slot,
            ));
        }

        let mut retained_views = Vec::new();
        let mut textures = Vec::new();
        for entry in desc.textures.iter().take(MAX_BINDINGS_PER_KIND) {
            let Some(handle) = entry.view else {
                self.state.set_last_error(format!(
                    "makeBindGroup: (group={group_index}, binding={}) texture view is null",
                    entry.slot
                ));
                return None;
            };
            let Some(view) = handle.downcast_ref::<TextureViewMetal>() else {
                self.state.set_last_error(format!(
                    "makeBindGroup: (group={group_index}, binding={}) view is not a Metal texture view",
                    entry.slot
                ));
                return None;
            };
            let Some((vs_slot, fs_slot)) =
                self.lookup_stages(layout, group_index, entry.slot, BindingKind::sampledTexture)
            else {
                continue;
            };
            let native = view.mtlTexture().and_then(retain_texture);
            retained_views.push(handle.clone());
            textures.push(MTLTextureBinding::with_native(native, vs_slot, fs_slot));
        }

        let mut retained_samplers = Vec::new();
        let mut samplers = Vec::new();
        for entry in desc.samplers.iter().take(MAX_BINDINGS_PER_KIND) {
            let Some(handle) = entry.sampler else {
                self.state.set_last_error(format!(
                    "makeBindGroup: (group={group_index}, binding={}) sampler is null",
                    entry.slot
                ));
                return None;
            };
            let Some(sampler) = handle.downcast_ref::<SamplerMetal>() else {
                self.state.set_last_error(format!(
                    "makeBindGroup: (group={group_index}, binding={}) sampler is not a Metal sampler",
                    entry.slot
                ));
                return None;
            };
            let Some((vs_slot, fs_slot)) =
                self.lookup_stages(layout, group_index, entry.slot, BindingKind::sampler)
            else {
                continue;
            };
            let native = sampler.mtlSampler().and_then(retain_sampler);
            retained_samplers.push(handle.clone());
            samplers.push(MTLSamplerBinding::with_native(native, vs_slot, fs_slot));
        }

        let base = BindGroup::from_parts(
            dynamic_offset_count,
            Some(layout_handle.clone()),
            retained_buffers,
            retained_views,
            retained_samplers,
        );
        Some(
            BindGroupMetal::from_parts(base, buffers, textures, samplers)
                .into_resource(self.state.manager())
                .erase(),
        )
    }

    #[cfg(target_vendor = "apple")]
    pub fn makeBindGroup(&self, desc: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        self.make_bind_group(desc)
    }

    fn lookup_stages(
        &self,
        layout: &BindGroupLayout,
        group_index: u32,
        binding: u32,
        expected: BindingKind,
    ) -> Option<(u16, u16)> {
        let Some(entry) = layout.find_entry(binding) else {
            self.state.set_last_error(format!(
                "makeBindGroup: (group={group_index}, binding={binding}) not declared in BindGroupLayout"
            ));
            return None;
        };
        let samplers_match = matches!(
            (entry.kind, expected),
            (
                BindingKind::sampler | BindingKind::comparisonSampler,
                BindingKind::sampler | BindingKind::comparisonSampler
            )
        );
        if entry.kind != expected && !samplers_match {
            self.state.set_last_error(format!(
                "makeBindGroup: (group={group_index}, binding={binding}) layout kind mismatch"
            ));
            return None;
        }
        let Some(vs) = native_slot(entry.kind, entry.nativeSlotVS) else {
            self.state.set_last_error(format!(
                "makeBindGroup: (group={group_index}, binding={binding}) nativeSlotVS {} out of range [0, {})",
                entry.nativeSlotVS,
                native_slot_limit(entry.kind),
            ));
            return None;
        };
        let Some(fs) = native_slot(entry.kind, entry.nativeSlotFS) else {
            self.state.set_last_error(format!(
                "makeBindGroup: (group={group_index}, binding={binding}) nativeSlotFS {} out of range [0, {})",
                entry.nativeSlotFS,
                native_slot_limit(entry.kind),
            ));
            return None;
        };
        if vs == BindingMap::kAbsent && fs == BindingMap::kAbsent {
            self.state.set_last_error(format!(
                "makeBindGroup: (group={group_index}, binding={binding}) layout has no resolved native slot — call makeLayoutFromShader"
            ));
            return None;
        }
        Some((vs, fs))
    }

    #[cfg(target_vendor = "apple")]
    fn lock_command_buffer(&self) -> MutexGuard<'_, Option<RetainedMetalCommandBuffer>> {
        self.command_buffer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn native_slot(kind: BindingKind, slot: u32) -> Option<u16> {
    if slot == BindGroupLayoutEntry::kNativeSlotAbsent {
        return Some(BindingMap::kAbsent);
    }
    if slot >= native_slot_limit(kind) {
        return None;
    }
    u16::try_from(slot).ok()
}

fn native_slot_limit(kind: BindingKind) -> u32 {
    match kind {
        BindingKind::uniformBuffer
        | BindingKind::storageBufferRO
        | BindingKind::storageBufferRW => 31,
        BindingKind::sampledTexture | BindingKind::storageTexture => 128,
        BindingKind::sampler | BindingKind::comparisonSampler => 16,
    }
}

fn binding_kind_name(kind: BindingKind) -> &'static str {
    match kind {
        BindingKind::uniformBuffer => "uniformBuffer",
        BindingKind::storageBufferRO => "storageBufferRO",
        BindingKind::storageBufferRW => "storageBufferRW",
        BindingKind::sampledTexture => "sampledTexture",
        BindingKind::storageTexture => "storageTexture",
        BindingKind::sampler => "sampler",
        BindingKind::comparisonSampler => "comparisonSampler",
    }
}

#[cfg(target_vendor = "apple")]
fn retain_texture(
    value: &ProtocolObject<dyn MTLTexture>,
) -> Option<Retained<ProtocolObject<dyn MTLTexture>>> {
    // SAFETY: `value` is a live Objective-C object for this call; retain adds
    // the +1 owner transferred into the bind-group native record.
    unsafe { Retained::retain(std::ptr::from_ref(value).cast_mut()) }
}

#[cfg(target_vendor = "apple")]
fn retain_sampler(
    value: &ProtocolObject<dyn objc2_metal::MTLSamplerState>,
) -> Option<Retained<ProtocolObject<dyn objc2_metal::MTLSamplerState>>> {
    // SAFETY: same exact +1 ownership transfer as `retain_texture`.
    unsafe { Retained::retain(std::ptr::from_ref(value).cast_mut()) }
}

#[cfg(target_vendor = "apple")]
fn report_pipeline_validation(
    state: &ContextState,
    out_error: &mut Option<&mut String>,
    message: String,
) {
    if let Some(error) = out_error.as_deref_mut() {
        *error = message;
    } else {
        state.set_last_error(format!("makePipeline: {message}"));
    }
}

#[cfg(target_vendor = "apple")]
fn report_pipeline_native(out_error: &mut Option<&mut String>, message: String) {
    if let Some(error) = out_error.as_deref_mut() {
        *error = message;
    }
}

#[cfg(target_vendor = "apple")]
fn vertex_format_to_mtl(value: VertexFormat) -> MTLVertexFormat {
    match value {
        VertexFormat::float1 => MTLVertexFormat::Float,
        VertexFormat::float2 => MTLVertexFormat::Float2,
        VertexFormat::float3 => MTLVertexFormat::Float3,
        VertexFormat::float4 => MTLVertexFormat::Float4,
        VertexFormat::uint8x4 => MTLVertexFormat::UChar4,
        VertexFormat::sint8x4 => MTLVertexFormat::Char4,
        VertexFormat::unorm8x4 => MTLVertexFormat::UChar4Normalized,
        VertexFormat::snorm8x4 => MTLVertexFormat::Char4Normalized,
        VertexFormat::uint16x2 => MTLVertexFormat::UShort2,
        VertexFormat::sint16x2 => MTLVertexFormat::Short2,
        VertexFormat::unorm16x2 => MTLVertexFormat::UShort2Normalized,
        VertexFormat::snorm16x2 => MTLVertexFormat::Short2Normalized,
        VertexFormat::uint16x4 => MTLVertexFormat::UShort4,
        VertexFormat::sint16x4 => MTLVertexFormat::Short4,
        VertexFormat::float16x2 => MTLVertexFormat::Half2,
        VertexFormat::float16x4 => MTLVertexFormat::Half4,
        VertexFormat::uint32 => MTLVertexFormat::UInt,
    }
}

#[cfg(target_vendor = "apple")]
fn color_write_mask_to_mtl(value: ColorWriteMask) -> MTLColorWriteMask {
    let mut result = MTLColorWriteMask::None;
    if value & ColorWriteMask::red != ColorWriteMask::none {
        result |= MTLColorWriteMask::Red;
    }
    if value & ColorWriteMask::green != ColorWriteMask::none {
        result |= MTLColorWriteMask::Green;
    }
    if value & ColorWriteMask::blue != ColorWriteMask::none {
        result |= MTLColorWriteMask::Blue;
    }
    if value & ColorWriteMask::alpha != ColorWriteMask::none {
        result |= MTLColorWriteMask::Alpha;
    }
    result
}

#[cfg(target_vendor = "apple")]
fn blend_factor_to_mtl(value: BlendFactor) -> MTLBlendFactor {
    match value {
        BlendFactor::zero => MTLBlendFactor::Zero,
        BlendFactor::one => MTLBlendFactor::One,
        BlendFactor::srcColor => MTLBlendFactor::SourceColor,
        BlendFactor::oneMinusSrcColor => MTLBlendFactor::OneMinusSourceColor,
        BlendFactor::srcAlpha => MTLBlendFactor::SourceAlpha,
        BlendFactor::oneMinusSrcAlpha => MTLBlendFactor::OneMinusSourceAlpha,
        BlendFactor::dstColor => MTLBlendFactor::DestinationColor,
        BlendFactor::oneMinusDstColor => MTLBlendFactor::OneMinusDestinationColor,
        BlendFactor::dstAlpha => MTLBlendFactor::DestinationAlpha,
        BlendFactor::oneMinusDstAlpha => MTLBlendFactor::OneMinusDestinationAlpha,
        BlendFactor::srcAlphaSaturated => MTLBlendFactor::SourceAlphaSaturated,
        BlendFactor::blendColor => MTLBlendFactor::BlendColor,
        BlendFactor::oneMinusBlendColor => MTLBlendFactor::OneMinusBlendColor,
    }
}

#[cfg(target_vendor = "apple")]
fn blend_op_to_mtl(value: BlendOp) -> MTLBlendOperation {
    match value {
        BlendOp::add => MTLBlendOperation::Add,
        BlendOp::subtract => MTLBlendOperation::Subtract,
        BlendOp::reverseSubtract => MTLBlendOperation::ReverseSubtract,
        BlendOp::min => MTLBlendOperation::Min,
        BlendOp::max => MTLBlendOperation::Max,
    }
}

#[cfg(target_vendor = "apple")]
fn stencil_op_to_mtl(value: StencilOp) -> MTLStencilOperation {
    match value {
        StencilOp::keep => MTLStencilOperation::Keep,
        StencilOp::zero => MTLStencilOperation::Zero,
        StencilOp::replace => MTLStencilOperation::Replace,
        StencilOp::incrementClamp => MTLStencilOperation::IncrementClamp,
        StencilOp::decrementClamp => MTLStencilOperation::DecrementClamp,
        StencilOp::invert => MTLStencilOperation::Invert,
        StencilOp::incrementWrap => MTLStencilOperation::IncrementWrap,
        StencilOp::decrementWrap => MTLStencilOperation::DecrementWrap,
    }
}

#[cfg(target_vendor = "apple")]
fn stencil_descriptor(
    compare: CompareFunction,
    fail: StencilOp,
    depth_fail: StencilOp,
    pass: StencilOp,
    read_mask: u8,
    write_mask: u8,
) -> Retained<MTLStencilDescriptor> {
    let descriptor = MTLStencilDescriptor::new();
    descriptor.setStencilCompareFunction(compare_to_mtl(compare));
    descriptor.setStencilFailureOperation(stencil_op_to_mtl(fail));
    descriptor.setDepthFailureOperation(stencil_op_to_mtl(depth_fail));
    descriptor.setDepthStencilPassOperation(stencil_op_to_mtl(pass));
    descriptor.setReadMask(u32::from(read_mask));
    descriptor.setWriteMask(u32::from(write_mask));
    descriptor
}

#[cfg(target_vendor = "apple")]
fn load_op_to_mtl(value: LoadOp) -> MTLLoadAction {
    match value {
        LoadOp::clear => MTLLoadAction::Clear,
        LoadOp::load => MTLLoadAction::Load,
        LoadOp::dontCare => MTLLoadAction::DontCare,
    }
}

#[cfg(target_vendor = "apple")]
fn store_op_to_mtl(value: StoreOp) -> MTLStoreAction {
    match value {
        StoreOp::store => MTLStoreAction::Store,
        StoreOp::discard => MTLStoreAction::DontCare,
    }
}

#[cfg(target_vendor = "apple")]
fn populate_features(device: &ProtocolObject<dyn MTLDevice>) -> Features {
    let mut features = Features {
        colorBufferFloat: true,
        colorBufferHalfFloat: true,
        perTargetBlend: true,
        perTargetWriteMask: true,
        textureViewSampling: true,
        drawBaseInstance: true,
        depthBiasClamp: true,
        anisotropicFiltering: true,
        texture3D: true,
        textureArrays: true,
        computeShaders: true,
        storageBuffers: true,
        bc: cfg!(target_os = "macos"),
        etc2: cfg!(target_os = "ios"),
        astc: cfg!(target_os = "ios"),
        maxColorAttachments: 8,
        maxTextureSize2D: 16_384,
        maxTextureSizeCube: 16_384,
        maxTextureSize3D: 2_048,
        maxUniformBufferSize: 256 * 1_024,
        maxVertexAttributes: 31,
        maxSamplers: 16,
        maxSamples: 1,
    };
    for sample_count in [8, 4, 2] {
        if device.supportsTextureSampleCount(sample_count) {
            features.maxSamples = sample_count as u32;
            break;
        }
    }
    features
}

#[cfg(target_vendor = "apple")]
fn texture_type_to_mtl(value: TextureType) -> MTLTextureType {
    match value {
        TextureType::texture2D => MTLTextureType::Type2D,
        TextureType::cube => MTLTextureType::TypeCube,
        TextureType::texture3D => MTLTextureType::Type3D,
        TextureType::array2D => MTLTextureType::Type2DArray,
    }
}

#[cfg(target_vendor = "apple")]
fn view_dimension_to_mtl(value: TextureViewDimension) -> MTLTextureType {
    match value {
        TextureViewDimension::texture2D => MTLTextureType::Type2D,
        TextureViewDimension::cube => MTLTextureType::TypeCube,
        TextureViewDimension::texture3D => MTLTextureType::Type3D,
        TextureViewDimension::array2D => MTLTextureType::Type2DArray,
        TextureViewDimension::cubeArray => MTLTextureType::TypeCubeArray,
    }
}

#[cfg(target_vendor = "apple")]
fn filter_to_mtl(value: Filter) -> MTLSamplerMinMagFilter {
    match value {
        Filter::nearest => MTLSamplerMinMagFilter::Nearest,
        Filter::linear => MTLSamplerMinMagFilter::Linear,
    }
}

#[cfg(target_vendor = "apple")]
fn mip_filter_to_mtl(value: Filter) -> MTLSamplerMipFilter {
    match value {
        Filter::nearest => MTLSamplerMipFilter::Nearest,
        Filter::linear => MTLSamplerMipFilter::Linear,
    }
}

#[cfg(target_vendor = "apple")]
fn wrap_to_mtl(value: WrapMode) -> MTLSamplerAddressMode {
    match value {
        WrapMode::repeat => MTLSamplerAddressMode::Repeat,
        WrapMode::mirrorRepeat => MTLSamplerAddressMode::MirrorRepeat,
        WrapMode::clampToEdge => MTLSamplerAddressMode::ClampToEdge,
    }
}

#[cfg(target_vendor = "apple")]
fn compare_to_mtl(value: CompareFunction) -> MTLCompareFunction {
    match value {
        CompareFunction::none | CompareFunction::never => MTLCompareFunction::Never,
        CompareFunction::less => MTLCompareFunction::Less,
        CompareFunction::equal => MTLCompareFunction::Equal,
        CompareFunction::lessEqual => MTLCompareFunction::LessEqual,
        CompareFunction::greater => MTLCompareFunction::Greater,
        CompareFunction::notEqual => MTLCompareFunction::NotEqual,
        CompareFunction::greaterEqual => MTLCompareFunction::GreaterEqual,
        CompareFunction::always => MTLCompareFunction::Always,
    }
}

#[cfg(target_vendor = "apple")]
fn format_to_mtl(value: TextureFormat) -> MTLPixelFormat {
    match value {
        TextureFormat::r8unorm => MTLPixelFormat::R8Unorm,
        TextureFormat::rg8unorm => MTLPixelFormat::RG8Unorm,
        TextureFormat::rgba8unorm => MTLPixelFormat::RGBA8Unorm,
        TextureFormat::rgba8snorm => MTLPixelFormat::RGBA8Snorm,
        TextureFormat::bgra8unorm => MTLPixelFormat::BGRA8Unorm,
        TextureFormat::rgba16float => MTLPixelFormat::RGBA16Float,
        TextureFormat::rg16float => MTLPixelFormat::RG16Float,
        TextureFormat::r16float => MTLPixelFormat::R16Float,
        TextureFormat::rgba32float => MTLPixelFormat::RGBA32Float,
        TextureFormat::rg32float => MTLPixelFormat::RG32Float,
        TextureFormat::r32float => MTLPixelFormat::R32Float,
        TextureFormat::rgb10a2unorm => MTLPixelFormat::RGB10A2Unorm,
        TextureFormat::r11g11b10float => MTLPixelFormat::RG11B10Float,
        TextureFormat::depth16unorm => MTLPixelFormat::Depth16Unorm,
        TextureFormat::depth24plusStencil8 => {
            if cfg!(target_arch = "aarch64") || cfg!(target_os = "ios") {
                MTLPixelFormat::Depth32Float_Stencil8
            } else {
                MTLPixelFormat::Depth24Unorm_Stencil8
            }
        }
        TextureFormat::depth32float => MTLPixelFormat::Depth32Float,
        TextureFormat::depth32floatStencil8 => MTLPixelFormat::Depth32Float_Stencil8,
        TextureFormat::bc1unorm => MTLPixelFormat::BC1_RGBA,
        TextureFormat::bc3unorm => MTLPixelFormat::BC3_RGBA,
        TextureFormat::bc7unorm => MTLPixelFormat::BC7_RGBAUnorm,
        TextureFormat::etc2rgb8 => MTLPixelFormat::ETC2_RGB8,
        TextureFormat::etc2rgba8 => MTLPixelFormat::EAC_RGBA8,
        TextureFormat::astc4x4 => MTLPixelFormat::ASTC_4x4_LDR,
        TextureFormat::astc6x6 => MTLPixelFormat::ASTC_6x6_LDR,
        TextureFormat::astc8x8 => MTLPixelFormat::ASTC_8x8_LDR,
    }
}

#[cfg(target_vendor = "apple")]
fn format_from_mtl(value: MTLPixelFormat) -> TextureFormat {
    match value {
        MTLPixelFormat::RGBA8Unorm => TextureFormat::rgba8unorm,
        MTLPixelFormat::BGRA8Unorm => TextureFormat::bgra8unorm,
        MTLPixelFormat::RGBA16Float => TextureFormat::rgba16float,
        MTLPixelFormat::RGB10A2Unorm => TextureFormat::rgb10a2unorm,
        _ => TextureFormat::rgba8unorm,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_vendor = "apple")]
    use crate::types::{ColorAttachment, SampEntry, TexEntry, TextureAspect, UBOEntry};

    #[test]
    fn native_slot_validation_preserves_absent_and_rejects_every_limit() {
        assert_eq!(
            native_slot(
                BindingKind::uniformBuffer,
                BindGroupLayoutEntry::kNativeSlotAbsent
            ),
            Some(BindingMap::kAbsent)
        );
        assert_eq!(native_slot(BindingKind::uniformBuffer, 30), Some(30));
        assert_eq!(native_slot(BindingKind::uniformBuffer, 31), None);
        assert_eq!(native_slot(BindingKind::sampledTexture, 127), Some(127));
        assert_eq!(native_slot(BindingKind::sampledTexture, 128), None);
        assert_eq!(native_slot(BindingKind::sampler, 15), Some(15));
        assert_eq!(native_slot(BindingKind::sampler, 16), None);
        assert_eq!(native_slot(BindingKind::uniformBuffer, 65_536), None);
    }

    #[cfg(target_vendor = "apple")]
    fn live_context() -> Option<ContextMetal> {
        use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice};

        let Some(device) = MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device for ORE context");
            return None;
        };
        let Some(queue) = device.newCommandQueue() else {
            crate::live_metal_test_unavailable("Metal command queue for ORE context");
            return None;
        };
        Some(ContextMetal::make(device, queue))
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn frame_serial_completion_outlives_context_and_begin_preserves_error() {
        let Some(context) = live_context() else {
            return;
        };
        context.state.set_last_error("pinned error");
        context.begin_frame(&FrameDescriptor {
            safeFrameNumber: 99,
            currentFrameNumber: 101,
            ..FrameDescriptor::default()
        });
        assert_eq!(context.currentSerial(), 1);
        assert_eq!(context.lastError(), "pinned error");
        let command_buffer = context
            .current_command_buffer()
            .expect("beginFrame creates command buffer");
        let completion = Arc::clone(&context.buffer_state);
        let submission = context
            .end_frame_with_completion()
            .expect("current command buffer");
        drop(context);
        command_buffer.waitUntilCompleted();
        assert_eq!(completion.completed_serial(), 1);
        assert_eq!(submission.result(), Some(Ok(())));
        assert!(
            command_buffer_completion_result(
                MTLCommandBufferStatus::Error,
                Some("injected Metal failure".to_owned())
            )
            .is_err()
        );
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn factories_publish_only_complete_native_resources() {
        let Some(context) = live_context() else {
            return;
        };
        let buffer = context
            .make_buffer(
                &BufferDesc::initialized(crate::types::BufferUsage::uniform, &[1, 2, 3, 4], false)
                    .expect("small descriptor"),
            )
            .expect("buffer");
        assert!(buffer.downcast_ref::<BufferMetal>().is_some());

        let texture_desc = TextureDesc {
            width: 4,
            height: 4,
            ..TextureDesc::default()
        };
        for invalid in [
            TextureDesc {
                width: 0,
                ..texture_desc
            },
            TextureDesc {
                height: 0,
                ..texture_desc
            },
            TextureDesc {
                depthOrArrayLayers: 0,
                ..texture_desc
            },
            TextureDesc {
                numMipmaps: 0,
                ..texture_desc
            },
            TextureDesc {
                sampleCount: 0,
                ..texture_desc
            },
        ] {
            assert!(context.make_texture(&invalid).is_none());
            assert_eq!(
                context.last_error(),
                "makeTexture: dimensions, layers, mip levels, and sample count must be non-zero"
            );
        }
        let texture = context.make_texture(&texture_desc).expect("texture");
        let view = context
            .make_texture_view(&TextureViewDesc {
                texture: &texture,
                dimension: TextureViewDimension::texture2D,
                aspect: TextureAspect::all,
                baseMipLevel: 0,
                mipCount: 1,
                baseLayer: 0,
                layerCount: 1,
            })
            .expect("view");
        assert!(view.downcast_ref::<TextureViewMetal>().is_some());
        let sampler = context
            .make_sampler(&SamplerDesc::default())
            .expect("sampler");
        assert!(sampler.downcast_ref::<SamplerMetal>().is_some());

        let native = texture
            .downcast_ref::<TextureMetal>()
            .and_then(TextureMetal::mtlTexture)
            .and_then(retain_texture)
            .expect("retained native texture");
        let wrapped = context.wrap_native_texture(native, 4, 4, false);
        let wrapped_view = wrapped
            .downcast_ref::<TextureViewMetal>()
            .expect("wrapped view");
        let wrapped_texture = wrapped_view
            .base()
            .texture()
            .downcast_ref::<TextureMetal>()
            .expect("wrapped base texture");
        assert_eq!(wrapped_texture.base().width(), 4);
        assert_eq!(wrapped_texture.base().format(), TextureFormat::rgba8unorm);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn nil_native_texture_view_and_sampler_still_publish_logical_resources() {
        let Some(context) = live_context() else {
            return;
        };
        let texture = context
            .make_texture(&TextureDesc {
                width: 4,
                height: 4,
                ..TextureDesc::default()
            })
            .expect("texture");
        let source = texture
            .downcast_ref::<TextureMetal>()
            .and_then(TextureMetal::mtlTexture)
            .expect("native source texture");
        let view_desc = TextureViewDesc {
            texture: &texture,
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        };

        let view = context.publish_texture_view(&view_desc, None);
        let view = view
            .downcast_ref::<TextureViewMetal>()
            .expect("logical Metal view");
        assert!(std::ptr::eq(
            view.mtlTexture().expect("source-texture fallback"),
            source,
        ));

        let sampler = context.publish_sampler(None);
        let sampler = sampler
            .downcast_ref::<SamplerMetal>()
            .expect("logical Metal sampler");
        assert!(sampler.mtlSampler().is_none());
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn public_layout_factory_rejects_out_of_range_native_slots_without_publication() {
        let Some(context) = live_context() else {
            return;
        };
        let mut entry = BindGroupLayoutEntry {
            binding: 7,
            nativeSlotVS: 31,
            ..BindGroupLayoutEntry::default()
        };
        assert!(
            context
                .make_bind_group_layout(&BindGroupLayoutDesc {
                    entries: std::slice::from_ref(&entry),
                    ..BindGroupLayoutDesc::default()
                })
                .is_none()
        );
        assert_eq!(
            context.last_error(),
            "makeBindGroupLayout: binding 7 uniformBuffer nativeSlotVS 31 out of range [0, 31)"
        );

        entry.kind = BindingKind::sampledTexture;
        entry.nativeSlotVS = BindGroupLayoutEntry::kNativeSlotAbsent;
        entry.nativeSlotFS = 128;
        assert!(
            context
                .make_bind_group_layout(&BindGroupLayoutDesc {
                    entries: std::slice::from_ref(&entry),
                    ..BindGroupLayoutDesc::default()
                })
                .is_none()
        );
        assert_eq!(
            context.last_error(),
            "makeBindGroupLayout: binding 7 sampledTexture nativeSlotFS 128 out of range [0, 128)"
        );

        entry.kind = BindingKind::sampler;
        entry.nativeSlotFS = BindGroupLayoutEntry::kNativeSlotAbsent;
        entry.nativeSlotCS = 16;
        assert!(
            context
                .make_bind_group_layout(&BindGroupLayoutDesc {
                    entries: std::slice::from_ref(&entry),
                    ..BindGroupLayoutDesc::default()
                })
                .is_none()
        );
        assert_eq!(
            context.last_error(),
            "makeBindGroupLayout: binding 7 sampler nativeSlotCS 16 out of range [0, 16)"
        );
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn buffer_allocation_failure_immediately_replaces_and_clears_context_error() {
        let Some(context) = live_context() else {
            return;
        };
        let buffer = context
            .make_buffer(&BufferDesc::uninitialized(
                crate::types::BufferUsage::uniform,
                4,
            ))
            .expect("buffer");
        let buffer = buffer.downcast_ref::<BufferMetal>().expect("Metal buffer");
        buffer.mark_bound();
        buffer.fail_next_allocation_for_test();
        context.state.set_last_error("older context error");

        crate::types::Buffer::update(buffer, &[9], 0)
            .expect("degraded update still writes the current backing");
        assert_eq!(
            context.last_error(),
            "ore: Metal buffer backing allocation failed; reusing in flight backing for this update"
        );
        context.clear_last_error();
        assert_eq!(context.last_error(), "");
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn native_shader_compile_failure_preserves_context_error() {
        let Some(context) = live_context() else {
            return;
        };
        context.state.set_last_error("earlier context error");
        let empty_binding_map = [2, 1, 14, 0, 0, 0, 0, 0];
        assert!(
            context
                .make_shader_module(&ShaderModuleDesc {
                    code: Some(b"this is not valid Metal shading language"),
                    bindingMapBytes: Some(&empty_binding_map),
                    ..ShaderModuleDesc::default()
                })
                .is_none()
        );
        assert_eq!(context.last_error(), "earlier context error");

        assert!(
            context
                .make_shader_module(&ShaderModuleDesc::default())
                .is_none()
        );
        assert_eq!(context.last_error(), "earlier context error");

        assert!(
            context
                .make_shader_module(&ShaderModuleDesc {
                    code: Some(&[0xff]),
                    ..ShaderModuleDesc::default()
                })
                .is_none()
        );
        assert_eq!(context.last_error(), "earlier context error");
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn pipeline_validation_routes_to_out_error_without_overwriting_context_error() {
        let Some(context) = live_context() else {
            return;
        };
        context.state.set_last_error("earlier context error");
        let mut out_error = String::new();
        assert!(
            context
                .make_pipeline(&PipelineDesc::default(), Some(&mut out_error))
                .is_none()
        );
        assert_eq!(
            out_error,
            "pipeline declares color outputs but has no fragment shader; supply `fragment`, or omit `colorTargets` for a depth-only pipeline"
        );
        assert_eq!(context.last_error(), "earlier context error");

        out_error.clear();
        assert!(
            context
                .make_pipeline(
                    &PipelineDesc {
                        colorCount: 0,
                        ..PipelineDesc::default()
                    },
                    Some(&mut out_error),
                )
                .is_none()
        );
        assert_eq!(out_error, "vertex shader module is null");
        assert_eq!(context.last_error(), "earlier context error");
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn shader_and_pipeline_factories_publish_live_native_state_and_reject_bad_entry() {
        let Some(context) = live_context() else {
            return;
        };
        let source = br#"
#include <metal_stdlib>
using namespace metal;
vertex float4 vs_main(uint vertex_id [[vertex_id]]) {
    return float4(vertex_id == 0 ? -1.0 : 1.0, -1.0, 0.0, 1.0);
}
fragment float4 fs_main() { return float4(1.0); }
"#;
        let empty_binding_map = [2, 1, 14, 0, 0, 0, 0, 0];
        let module = context
            .make_shader_module(&ShaderModuleDesc {
                code: Some(source),
                bindingMapBytes: Some(&empty_binding_map),
                ..ShaderModuleDesc::default()
            })
            .expect("compile MSL through ContextMetal");
        let desc = PipelineDesc {
            vertexModule: Some(&module),
            fragmentModule: Some(&module),
            ..PipelineDesc::default()
        };
        let pipeline = context
            .make_pipeline(&desc, None)
            .expect("publish complete native pipeline");
        assert!(pipeline.downcast_ref::<PipelineMetal>().is_some());

        let bad_desc = PipelineDesc {
            vertexModule: Some(&module),
            vertexEntryPoint: Some("missing_vertex"),
            fragmentModule: Some(&module),
            ..PipelineDesc::default()
        };
        let mut error = String::new();
        assert!(context.make_pipeline(&bad_desc, Some(&mut error)).is_none());
        assert_eq!(
            error,
            "vertex entry point 'missing_vertex' not found in shader library"
        );
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn pipeline_layout_validation_preserves_vertex_reflection_precedence() {
        let Some(context) = live_context() else {
            return;
        };
        let source = br#"
#include <metal_stdlib>
using namespace metal;
vertex float4 vs_main(uint vertex_id [[vertex_id]]) {
    return float4(vertex_id == 0 ? -1.0 : 1.0, -1.0, 0.0, 1.0);
}
fragment float4 fs_main() { return float4(1.0); }
"#;
        let empty = [2, 1, 14, 0, 0, 0, 0, 0];
        let fragment_sampler = [
            2, 1, 14, 0, 1, 0, 0, 0, // header
            0, 0, 5, 2, 0, 0xff, 0xff, 0, 0, 0xff, 0xff, 0, 0, 0,
        ];
        let vertex = context
            .make_shader_module(&ShaderModuleDesc {
                code: Some(source),
                bindingMapBytes: Some(&empty),
                ..ShaderModuleDesc::default()
            })
            .expect("vertex module");
        let fragment = context
            .make_shader_module(&ShaderModuleDesc {
                code: Some(source),
                bindingMapBytes: Some(&fragment_sampler),
                ..ShaderModuleDesc::default()
            })
            .expect("fragment module");
        let pipeline = context.make_pipeline(
            &PipelineDesc {
                vertexModule: Some(&vertex),
                fragmentModule: Some(&fragment),
                ..PipelineDesc::default()
            },
            None,
        );
        assert!(
            pipeline.is_some(),
            "a non-null vertex module is the binding-map source even when its map is empty"
        );

        let mut error = String::new();
        assert!(
            context
                .make_pipeline(
                    &PipelineDesc {
                        vertexModule: None,
                        fragmentModule: Some(&fragment),
                        ..PipelineDesc::default()
                    },
                    Some(&mut error),
                )
                .is_none()
        );
        assert_eq!(
            error,
            "@group(0) @binding(0): shader declares sampler but PipelineDesc::bindGroupLayouts has no entry for group 0"
        );
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn bind_group_skips_unresolved_entries_and_last_error_follows_kind_order() {
        let Some(context) = live_context() else {
            return;
        };
        let layout = context
            .make_bind_group_layout(&BindGroupLayoutDesc::default())
            .expect("empty layout");
        let buffer = context
            .make_buffer(&BufferDesc::uninitialized(
                crate::types::BufferUsage::uniform,
                16,
            ))
            .expect("buffer");
        let texture = context
            .make_texture(&TextureDesc {
                width: 1,
                height: 1,
                ..TextureDesc::default()
            })
            .expect("texture");
        let view = context
            .make_texture_view(&TextureViewDesc {
                texture: &texture,
                dimension: TextureViewDimension::texture2D,
                aspect: TextureAspect::all,
                baseMipLevel: 0,
                mipCount: 1,
                baseLayer: 0,
                layerCount: 1,
            })
            .expect("view");
        let sampler = context
            .make_sampler(&SamplerDesc::default())
            .expect("sampler");
        let ubos = [UBOEntry {
            slot: 1,
            buffer: Some(&buffer),
            offset: 0,
            size: 16,
        }];
        let textures = [TexEntry {
            slot: 2,
            view: Some(&view),
        }];
        let samplers = [SampEntry {
            slot: 3,
            sampler: Some(&sampler),
        }];
        let group = context
            .make_bind_group(&BindGroupDesc {
                layout: Some(&layout),
                ubos: &ubos,
                textures: &textures,
                samplers: &samplers,
                label: None,
            })
            .expect("source publishes a group after skipping invalid entries");
        let group = group.downcast_ref::<BindGroupMetal>().expect("Metal group");
        assert!(group.buffers().is_empty());
        assert!(group.textures().is_empty());
        assert!(group.samplers().is_empty());
        assert_eq!(
            context.last_error(),
            "makeBindGroup: (group=0, binding=3) not declared in BindGroupLayout"
        );
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn beginning_a_second_pass_auto_finishes_the_first() {
        let Some(context) = live_context() else {
            return;
        };
        context.begin_frame(&FrameDescriptor::default());
        let texture = context
            .make_texture(&TextureDesc {
                width: 4,
                height: 4,
                renderTarget: true,
                ..TextureDesc::default()
            })
            .expect("render target");
        let view = context
            .make_texture_view(&TextureViewDesc {
                texture: &texture,
                dimension: TextureViewDimension::texture2D,
                aspect: TextureAspect::all,
                baseMipLevel: 0,
                mipCount: 1,
                baseLayer: 0,
                layerCount: 1,
            })
            .expect("render-target view");
        let desc = RenderPassDesc {
            colorAttachments: [
                ColorAttachment {
                    view: Some(&view),
                    ..ColorAttachment::default()
                },
                ColorAttachment::default(),
                ColorAttachment::default(),
                ColorAttachment::default(),
            ],
            ..RenderPassDesc::default()
        };
        let first = context.begin_render_pass(&desc, None).expect("first pass");
        let second = context.begin_render_pass(&desc, None).expect("second pass");
        assert!(first.is_finished());
        assert!(!second.is_finished());
        second.finish();
        context.end_frame();
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn render_pass_is_not_published_without_a_frame_command_buffer() {
        let Some(context) = live_context() else {
            return;
        };
        assert!(
            context
                .begin_render_pass(&RenderPassDesc::default(), None)
                .is_none()
        );
        assert_eq!(
            context.last_error(),
            "beginRenderPass: beginFrame has not created a command buffer"
        );
    }
}
