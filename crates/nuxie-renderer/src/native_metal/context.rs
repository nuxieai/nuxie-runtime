//! Concrete Metal context and command-buffer ownership.
//!
//! This is a direct ownership adaptation of the pinned upstream context:
//! - `renderer/include/rive/renderer/metal/render_context_metal_impl.h:89-280`
//! - `renderer/src/metal/render_context_metal_impl.mm:100-240,414-656,717-725`
//! - `renderer/src/metal/render_context_metal_impl.mm:1023-1079,1227-1509`
//! - `renderer/src/metal/render_context_metal_impl.mm:1898-1925,2016-2030`
//! - `renderer/src/render_context.cpp:2472-2817`
//!
//! Pinned upstream source: `rive-runtime` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

use super::background_shader_compiler::{
    BackgroundShaderCompileError, NativeBackgroundShaderCompiler,
};
use super::buffer_ring_coordinator::{
    BufferRingCompletion, BufferRingCoordinator, BufferRingLease,
};
use super::capabilities::{ApplePlatform, MetalCapabilitySelection};
use super::draw_pipeline::{DrawPipeline, MetalInterlockMode};
use super::draw_shader::DrawShaderLibrary;
use super::feather_atlas_pipeline::FeatherAtlasPipelines;
use super::feather_atlas_resource::FeatherAtlasResource;
use super::gradient_resource::{GradientResource, GRADIENT_TEXTURE_WIDTH};
use super::samplers::NativeMetalSamplers;
use super::tessellation_resource::{
    TessellationResource, K_TESS_SPAN_INDICES, TESSELLATION_TEXTURE_WIDTH,
};
use super::upload_buffer_ring::UploadBufferRing;
use super::{make_solid_pipeline, new_library_from_metallib_bytes};
use crate::gpu::{self, DrawType};
use crate::native_metal::shader_compile_plan::{
    BackgroundCompileJob, InterlockMode, MetalFeatures, ENABLE_DITHER, FIXED_FUNCTION_COLOR_OUTPUT,
};
use crate::RendererError;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::NSString;
use objc2_metal::{
    MTLBuffer, MTLCommandBuffer, MTLCommandBufferStatus, MTLCommandQueue, MTLDevice, MTLDrawable,
    MTLLibrary, MTLOrigin, MTLPixelFormat, MTLRegion, MTLRenderPipelineDescriptor,
    MTLRenderPipelineState, MTLResourceOptions, MTLSize, MTLTexture, MTLTextureDescriptor,
    MTLTextureType, MTLTextureUsage,
};
use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::Mutex;

const RESOURCE_METALLIB: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/native_metal_resources.metallib"));

const COLOR_RAMP_VERTEX_MAIN: &str = "EF";
const COLOR_RAMP_FRAGMENT_MAIN: &str = "FF";
const TESSELLATE_VERTEX_MAIN: &str = "WF";
const TESSELLATE_FRAGMENT_MAIN: &str = "XF";
const UBER_PATH_VERTEX_MAIN: &str = "p1111000000::GC";
const UBER_PATH_FRAGMENT_MAIN: &str = "p1111111100::JB";
const SPECIALIZED_DRAW_VERTEX_MAIN: &str = "GC";
const SPECIALIZED_DRAW_FRAGMENT_MAIN: &str = "JB";

/// Coherent retained owners published by one successful preparation.
///
/// Upstream mutates nullable Metal owners during its sizing pass. Rust stages
/// a cheap retained clone and swaps it only after every fallible allocation,
/// lazy pipeline realization, compilation, and upload succeeds so typed errors
/// cannot expose a partially updated generation.
#[derive(Clone)]
struct ResourceGeneration {
    gradient: GradientResource,
    tessellation: TessellationResource,
    feather_atlas: Option<FeatherAtlasResource>,
    feather_atlas_pipelines: Option<FeatherAtlasPipelines>,
    specialized_atlas_blit_pipeline: Option<DrawPipeline>,
    atomic_path_pipelines: Option<AtomicPathPipelines>,
}

#[derive(Clone)]
struct AtomicPathPipelines {
    initialize: DrawPipeline,
    midpoint: DrawPipeline,
    resolve: DrawPipeline,
}

pub(crate) struct AtomicPathPipelineStates {
    pub(crate) initialize: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    pub(crate) midpoint: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    pub(crate) resolve: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
}

struct ResourceState {
    generation: ResourceGeneration,
    uploads: UploadRings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResourcePreparationStage {
    GradientTexture,
    TessellationTexture,
    FeatherAtlasTexture,
    FeatherAtlasPipelines,
}

pub(crate) struct PreparedResourceLease {
    pub(crate) gradient: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
    pub(crate) tessellation: Retained<ProtocolObject<dyn MTLTexture>>,
    pub(crate) feather_atlas: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
    pub(crate) feather_atlas_pipeline: Option<Retained<ProtocolObject<dyn MTLRenderPipelineState>>>,
    pub(crate) atomic_path_pipelines: Option<AtomicPathPipelineStates>,
    pub(crate) flush_uniforms: Retained<ProtocolObject<dyn MTLBuffer>>,
    pub(crate) gradient_spans: Option<Retained<ProtocolObject<dyn MTLBuffer>>>,
    pub(crate) tessellation_spans: Retained<ProtocolObject<dyn MTLBuffer>>,
    pub(crate) paths: Retained<ProtocolObject<dyn MTLBuffer>>,
    pub(crate) paints: Retained<ProtocolObject<dyn MTLBuffer>>,
    pub(crate) paint_aux: Retained<ProtocolObject<dyn MTLBuffer>>,
    pub(crate) contours: Retained<ProtocolObject<dyn MTLBuffer>>,
    pub(crate) triangles: Option<Retained<ProtocolObject<dyn MTLBuffer>>>,
    pub(crate) upload_calls: u64,
    pub(crate) upload_bytes: u64,
    ownership: BufferRingLease,
}

impl PreparedResourceLease {
    pub(crate) fn transfer_to_completion(&mut self) -> Result<BufferRingCompletion, RendererError> {
        self.ownership.transfer_to_completion().map_err(|error| {
            RendererError::NativeMetal(format!(
                "transfer native Metal upload-ring ownership: {error:?}"
            ))
        })
    }
}

pub(crate) struct UploadBatch<'a> {
    pub(crate) flush_uniforms: &'a gpu::FlushUniforms,
    pub(crate) gradient_spans: &'a [gpu::GradientSpan],
    pub(crate) tessellation_spans: &'a [gpu::TessVertexSpan],
    pub(crate) paths: &'a [gpu::PathData],
    pub(crate) paints: &'a [gpu::PaintData],
    pub(crate) paint_aux: &'a [gpu::PaintAuxData],
    pub(crate) contours: &'a [gpu::ContourData],
    pub(crate) triangles: &'a [gpu::TriangleVertex],
}

#[derive(Default)]
struct UploadRings {
    flush_uniforms: Option<UploadBufferRing>,
    gradient_spans: Option<UploadBufferRing>,
    tessellation_spans: Option<UploadBufferRing>,
    paths: Option<UploadBufferRing>,
    paints: Option<UploadBufferRing>,
    paint_aux: Option<UploadBufferRing>,
    contours: Option<UploadBufferRing>,
    triangles: Option<UploadBufferRing>,
}

/// Long-lived resources shared by every frame from one native Metal factory.
///
/// The context is intentionally concrete. It is the Metal owner translated
/// from `RenderContextMetalImpl`, not a speculative cross-backend HAL.
pub(crate) struct NativeMetalContext {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    capabilities: MetalCapabilitySelection,
    platform: ApplePlatform,
    solid_rgba_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    solid_bgra_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    _draw_shader_library: DrawShaderLibrary,
    _resource_shader_library: Retained<ProtocolObject<dyn MTLLibrary>>,
    color_ramp_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    tessellate_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    midpoint_draw_pipeline: DrawPipeline,
    tess_span_index_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    path_patch_vertex_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    path_patch_index_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    // Pinned static owners for the later atomic ImageRect draw path.
    _image_rect_vertex_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    _image_rect_index_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    gaussian_integral_texture: Retained<ProtocolObject<dyn MTLTexture>>,
    upload_coordinator: BufferRingCoordinator,
    resources: Mutex<ResourceState>,
    samplers: NativeMetalSamplers,
}

impl NativeMetalContext {
    pub(crate) fn new(
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        capabilities: MetalCapabilitySelection,
        platform: ApplePlatform,
    ) -> Result<Self, RendererError> {
        let queue = device.newCommandQueue().ok_or_else(|| {
            RendererError::NativeMetal("MTLDevice returned no command queue".to_owned())
        })?;
        let draw_shader_library = DrawShaderLibrary::load(&device)
            .map_err(|error| RendererError::NativeMetal(error.to_string()))?;
        let resource_shader_library = new_library_from_metallib_bytes(&device, RESOURCE_METALLIB)?;
        let color_ramp_pipeline = make_resource_pipeline(
            &device,
            &resource_shader_library,
            COLOR_RAMP_VERTEX_MAIN,
            COLOR_RAMP_FRAGMENT_MAIN,
            MTLPixelFormat::RGBA8Unorm,
        )?;
        let tessellate_pipeline = make_resource_pipeline(
            &device,
            &resource_shader_library,
            TESSELLATE_VERTEX_MAIN,
            TESSELLATE_FRAGMENT_MAIN,
            MTLPixelFormat::RGBA32Uint,
        )?;
        let midpoint_draw_pipeline = DrawPipeline::new(
            &device,
            Some(draw_shader_library.library()),
            &NSString::from_str(UBER_PATH_VERTEX_MAIN),
            &NSString::from_str(UBER_PATH_FRAGMENT_MAIN),
            DrawType::MidpointFanPatches,
            MetalInterlockMode::RasterOrdering,
            0,
        )
        .map_err(|error| RendererError::NativeMetal(error.to_string()))?;
        let tess_span_index_buffer = make_buffer(&device, &K_TESS_SPAN_INDICES)?;
        let (patch_vertices, patch_indices) = gpu::generate_patch_buffer_data();
        let path_patch_vertex_buffer = make_buffer(&device, &patch_vertices)?;
        let path_patch_index_buffer = make_buffer(&device, &patch_indices)?;
        let image_rect_vertex_buffer = make_buffer(&device, &gpu::IMAGE_RECT_VERTICES)?;
        let image_rect_index_buffer = make_buffer(&device, &gpu::IMAGE_RECT_INDICES)?;
        let gaussian_integral_texture = make_gaussian_integral_texture(&device)?;
        let gradient = GradientResource::new(&device, GRADIENT_TEXTURE_WIDTH, 1)?
            .expect("the canonical gradient texture extent is nonzero");
        let tessellation = TessellationResource::new(&device, TESSELLATION_TEXTURE_WIDTH, 1)?
            .expect("the canonical tessellation texture extent is nonzero");
        let resources = Mutex::new(ResourceState {
            generation: ResourceGeneration {
                gradient,
                tessellation,
                feather_atlas: None,
                feather_atlas_pipelines: None,
                specialized_atlas_blit_pipeline: None,
                atomic_path_pipelines: None,
            },
            uploads: UploadRings::default(),
        });
        let samplers = NativeMetalSamplers::new(&device)?;
        let solid_rgba_pipeline =
            make_solid_pipeline(&device, objc2_metal::MTLPixelFormat::RGBA8Unorm)?;
        let solid_bgra_pipeline =
            make_solid_pipeline(&device, objc2_metal::MTLPixelFormat::BGRA8Unorm)?;
        Ok(Self {
            device,
            queue,
            capabilities,
            platform,
            solid_rgba_pipeline,
            solid_bgra_pipeline,
            _draw_shader_library: draw_shader_library,
            _resource_shader_library: resource_shader_library,
            color_ramp_pipeline,
            tessellate_pipeline,
            midpoint_draw_pipeline,
            tess_span_index_buffer,
            path_patch_vertex_buffer,
            path_patch_index_buffer,
            _image_rect_vertex_buffer: image_rect_vertex_buffer,
            _image_rect_index_buffer: image_rect_index_buffer,
            gaussian_integral_texture,
            upload_coordinator: BufferRingCoordinator::new(),
            resources,
            samplers,
        })
    }

    pub(crate) fn device(&self) -> &ProtocolObject<dyn MTLDevice> {
        &self.device
    }

    pub(crate) fn retained_device(&self) -> Retained<ProtocolObject<dyn MTLDevice>> {
        self.device.clone()
    }

    pub(crate) fn capabilities(&self) -> MetalCapabilitySelection {
        self.capabilities
    }

    #[cfg(test)]
    pub(crate) fn feather_atlas_resources_are_initialized(&self) -> bool {
        let state = self
            .resources
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.generation.feather_atlas.is_some()
            && state.generation.feather_atlas_pipelines.is_some()
            && state.generation.specialized_atlas_blit_pipeline.is_some()
    }

    pub(crate) fn solid_pipeline(
        &self,
        pixel_format: objc2_metal::MTLPixelFormat,
    ) -> Result<&ProtocolObject<dyn MTLRenderPipelineState>, RendererError> {
        if pixel_format == objc2_metal::MTLPixelFormat::RGBA8Unorm {
            Ok(&self.solid_rgba_pipeline)
        } else if pixel_format == objc2_metal::MTLPixelFormat::BGRA8Unorm {
            Ok(&self.solid_bgra_pipeline)
        } else {
            Err(RendererError::NativeMetal(format!(
                "native Metal tracer does not support target pixel format {pixel_format:?}"
            )))
        }
    }

    pub(crate) fn color_ramp_pipeline(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        &self.color_ramp_pipeline
    }

    pub(crate) fn tessellate_pipeline(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        &self.tessellate_pipeline
    }

    pub(crate) fn midpoint_draw_pipeline(
        &self,
        pixel_format: MTLPixelFormat,
    ) -> Result<&ProtocolObject<dyn MTLRenderPipelineState>, RendererError> {
        self.midpoint_draw_pipeline
            .pipeline_state(pixel_format)
            .map_err(|error| RendererError::NativeMetal(error.to_string()))
    }

    pub(crate) fn atlas_blit_pipeline(
        &self,
        pixel_format: MTLPixelFormat,
    ) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, RendererError> {
        let state = self.resources.lock().map_err(|_| {
            RendererError::NativeMetal("native Metal resource ring is poisoned".to_owned())
        })?;
        state
            .generation
            .specialized_atlas_blit_pipeline
            .as_ref()
            .ok_or_else(|| {
                RendererError::NativeMetal(
                    "specialized AtlasBlit pipeline was not prepared".to_owned(),
                )
            })?
            .retained_pipeline_state(pixel_format)
            .map_err(|error| RendererError::NativeMetal(error.to_string()))
    }

    pub(crate) fn tess_span_index_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        &self.tess_span_index_buffer
    }

    pub(crate) fn path_patch_vertex_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        &self.path_patch_vertex_buffer
    }

    pub(crate) fn path_patch_index_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        &self.path_patch_index_buffer
    }

    pub(crate) fn gaussian_integral_texture(&self) -> &ProtocolObject<dyn MTLTexture> {
        &self.gaussian_integral_texture
    }

    pub(crate) fn image_sampler(
        &self,
        sampler: nuxie_render_api::ImageSampler,
    ) -> &ProtocolObject<dyn objc2_metal::MTLSamplerState> {
        self.samplers.sampler(sampler)
    }

    pub(crate) fn prepare_resources(
        &self,
        gradient_height: usize,
        tessellation_height: usize,
        feather_atlas_extent: Option<[usize; 2]>,
        feather_atlas_is_stroke: Option<bool>,
        uploads: UploadBatch<'_>,
    ) -> Result<PreparedResourceLease, RendererError> {
        let mut allow_all = |_| Ok(());
        self.prepare_resources_with_control(
            gradient_height,
            tessellation_height,
            feather_atlas_extent,
            feather_atlas_is_stroke,
            uploads,
            specialized_atlas_blit_job(),
            &mut allow_all,
        )
    }

    pub(crate) fn prepare_atomic_path_resources(
        &self,
        gradient_height: usize,
        tessellation_height: usize,
        pixel_format: MTLPixelFormat,
        uploads: UploadBatch<'_>,
    ) -> Result<PreparedResourceLease, RendererError> {
        let mut allow_all = |_| Ok(());
        self.prepare_resources_core(
            gradient_height,
            tessellation_height,
            None,
            None,
            uploads,
            specialized_atlas_blit_job(),
            Some(pixel_format),
            &mut allow_all,
        )
    }

    fn prepare_resources_with_control(
        &self,
        gradient_height: usize,
        tessellation_height: usize,
        feather_atlas_extent: Option<[usize; 2]>,
        feather_atlas_is_stroke: Option<bool>,
        uploads: UploadBatch<'_>,
        specialized_job: BackgroundCompileJob,
        before: &mut impl FnMut(ResourcePreparationStage) -> Result<(), RendererError>,
    ) -> Result<PreparedResourceLease, RendererError> {
        self.prepare_resources_core(
            gradient_height,
            tessellation_height,
            feather_atlas_extent,
            feather_atlas_is_stroke,
            uploads,
            specialized_job,
            None,
            before,
        )
    }

    fn prepare_resources_core(
        &self,
        gradient_height: usize,
        tessellation_height: usize,
        feather_atlas_extent: Option<[usize; 2]>,
        feather_atlas_is_stroke: Option<bool>,
        uploads: UploadBatch<'_>,
        specialized_job: BackgroundCompileJob,
        atomic_path_pixel_format: Option<MTLPixelFormat>,
        before: &mut impl FnMut(ResourcePreparationStage) -> Result<(), RendererError>,
    ) -> Result<PreparedResourceLease, RendererError> {
        let ownership = self.upload_coordinator.prepare_to_flush();
        let mut state = self.resources.lock().map_err(|_| {
            RendererError::NativeMetal("native Metal resource ring is poisoned".to_owned())
        })?;
        let mut generation = state.generation.clone();
        before(ResourcePreparationStage::GradientTexture)?;
        generation
            .gradient
            .resize(self.device(), GRADIENT_TEXTURE_WIDTH, gradient_height)?;
        before(ResourcePreparationStage::TessellationTexture)?;
        generation.tessellation.resize(
            self.device(),
            TESSELLATION_TEXTURE_WIDTH,
            tessellation_height,
        )?;
        if atomic_path_pixel_format.is_some() && generation.atomic_path_pipelines.is_none() {
            generation.atomic_path_pipelines = Some(compile_atomic_path_pipelines(
                &self.device,
                self.capabilities,
                self.platform,
            )?);
        }
        if feather_atlas_extent.is_some() != feather_atlas_is_stroke.is_some() {
            return Err(RendererError::NativeMetal(
                "feather atlas extent and pipeline selection must be supplied together".to_owned(),
            ));
        }
        if let Some([width, height]) = feather_atlas_extent {
            before(ResourcePreparationStage::FeatherAtlasTexture)?;
            if generation.feather_atlas.is_none() {
                let replacement = FeatherAtlasResource::new(self.device(), width, height)?;
                before(ResourcePreparationStage::FeatherAtlasPipelines)?;
                let pipelines =
                    FeatherAtlasPipelines::new(self.device(), self._draw_shader_library.library())
                        .map_err(|error| RendererError::NativeMetal(error.to_string()))?;
                let atlas_blit_pipeline = compile_specialized_atlas_blit_pipeline(
                    &self.device,
                    self.capabilities,
                    self.platform,
                    specialized_job,
                )?;
                generation.feather_atlas = replacement;
                generation.feather_atlas_pipelines = Some(pipelines);
                generation.specialized_atlas_blit_pipeline = Some(atlas_blit_pipeline);
            } else if let Some(resource) = generation.feather_atlas.as_mut() {
                resource.ensure_capacity(self.device(), width, height)?;
            }
        }
        let sizes = UploadSizes::new(&uploads)?;
        let upload_bytes = sizes.total_bytes()?;
        state.uploads.ensure_capacities(self.device(), sizes)?;
        let uploaded = state.uploads.upload(uploads)?;
        let gradient = generation.gradient.retained_texture();
        let tessellation = generation.tessellation.retained_texture().ok_or_else(|| {
            RendererError::NativeMetal("tessellation resource texture is absent".to_owned())
        })?;
        let feather_atlas = generation
            .feather_atlas
            .as_ref()
            .and_then(FeatherAtlasResource::retained_texture);
        if feather_atlas_extent.is_some() && feather_atlas.is_none() {
            return Err(RendererError::NativeMetal(
                "feather atlas resource texture is absent".to_owned(),
            ));
        }
        let feather_atlas_pipeline = if let Some(is_stroke) = feather_atlas_is_stroke {
            Some(
                generation
                    .feather_atlas_pipelines
                    .as_ref()
                    .ok_or_else(|| {
                        RendererError::NativeMetal(
                            "feather atlas pipeline pair is absent".to_owned(),
                        )
                    })?
                    .retained(is_stroke),
            )
        } else {
            None
        };
        let atomic_path_pipelines = atomic_path_pixel_format
            .map(|pixel_format| {
                let pipelines = generation.atomic_path_pipelines.as_ref().ok_or_else(|| {
                    RendererError::NativeMetal("atomic path pipeline set is absent".to_owned())
                })?;
                Ok(AtomicPathPipelineStates {
                    initialize: pipelines
                        .initialize
                        .retained_pipeline_state(pixel_format)
                        .map_err(|error| RendererError::NativeMetal(error.to_string()))?,
                    midpoint: pipelines
                        .midpoint
                        .retained_pipeline_state(pixel_format)
                        .map_err(|error| RendererError::NativeMetal(error.to_string()))?,
                    resolve: pipelines
                        .resolve
                        .retained_pipeline_state(pixel_format)
                        .map_err(|error| RendererError::NativeMetal(error.to_string()))?,
                })
            })
            .transpose()?;
        // Publish every concrete texture and paired pipeline together only
        // after all fallible realization, upload preparation, and lease-owner
        // extraction has succeeded.
        state.generation = generation;

        Ok(PreparedResourceLease {
            gradient,
            tessellation,
            feather_atlas,
            feather_atlas_pipeline,
            atomic_path_pipelines,
            flush_uniforms: uploaded.flush_uniforms,
            gradient_spans: uploaded.gradient_spans,
            tessellation_spans: uploaded.tessellation_spans,
            paths: uploaded.paths,
            paints: uploaded.paints,
            paint_aux: uploaded.paint_aux,
            contours: uploaded.contours,
            triangles: uploaded.triangles,
            upload_calls: sizes.nonempty_count(),
            upload_bytes,
            ownership,
        })
    }

    /// Acquires the one command buffer that a frame owns until finish or drop.
    pub(crate) fn make_command_buffer(
        &self,
    ) -> Result<Retained<ProtocolObject<dyn MTLCommandBuffer>>, RendererError> {
        require_command_buffer(self.queue.commandBuffer())
    }

    /// Commits the frame-owned command buffer and propagates Metal completion.
    pub(crate) fn commit_and_wait(
        command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    ) -> Result<(), RendererError> {
        command_buffer.commit();
        command_buffer.waitUntilCompleted();
        command_buffer_completion_result(
            command_buffer.status(),
            command_buffer.error().map(|error| format!("{error:?}")),
        )
    }

    /// Commits renderer work, then schedules the product drawable on the next
    /// command buffer from the same queue. This preserves the pinned product
    /// boundary in `fiddle_context_metal.mm:114-121,186-190`.
    pub(crate) fn commit_and_present(
        &self,
        render_command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
        drawable: &ProtocolObject<dyn MTLDrawable>,
    ) -> Result<(), RendererError> {
        render_command_buffer.commit();
        let presentation_command_buffer = match self.make_command_buffer() {
            Ok(command_buffer) => command_buffer,
            Err(presentation_error) => {
                render_command_buffer.waitUntilCompleted();
                let render_result = command_buffer_completion_result(
                    render_command_buffer.status(),
                    render_command_buffer
                        .error()
                        .map(|error| format!("{error:?}")),
                );
                return render_result.and(Err(presentation_error));
            }
        };
        presentation_command_buffer.presentDrawable(drawable);
        presentation_command_buffer.commit();

        render_command_buffer.waitUntilCompleted();
        let render_result = command_buffer_completion_result(
            render_command_buffer.status(),
            render_command_buffer
                .error()
                .map(|error| format!("{error:?}")),
        );
        presentation_command_buffer.waitUntilCompleted();
        let presentation_result = command_buffer_completion_result(
            presentation_command_buffer.status(),
            presentation_command_buffer
                .error()
                .map(|error| format!("{error:?}")),
        );
        render_result.and(presentation_result)
    }
}

#[derive(Clone, Copy)]
struct UploadSizes {
    flush_uniforms: usize,
    gradient_spans: usize,
    tessellation_spans: usize,
    paths: usize,
    paints: usize,
    paint_aux: usize,
    contours: usize,
    triangles: usize,
}

impl UploadSizes {
    fn new(batch: &UploadBatch<'_>) -> Result<Self, RendererError> {
        Ok(Self {
            flush_uniforms: std::mem::size_of_val(batch.flush_uniforms),
            gradient_spans: optional_upload_byte_len(batch.gradient_spans)?,
            tessellation_spans: upload_byte_len(batch.tessellation_spans)?,
            paths: upload_byte_len(batch.paths)?,
            paints: upload_byte_len(batch.paints)?,
            paint_aux: upload_byte_len(batch.paint_aux)?,
            contours: upload_byte_len(batch.contours)?,
            triangles: optional_upload_byte_len(batch.triangles)?,
        })
    }

    fn total_bytes(self) -> Result<u64, RendererError> {
        [
            self.flush_uniforms,
            self.gradient_spans,
            self.tessellation_spans,
            self.paths,
            self.paints,
            self.paint_aux,
            self.contours,
            self.triangles,
        ]
        .into_iter()
        .try_fold(0_u64, |total, bytes| {
            let bytes = u64::try_from(bytes).map_err(|_| {
                RendererError::NativeMetal("native Metal upload byte count exceeds UInt64".into())
            })?;
            total.checked_add(bytes).ok_or_else(|| {
                RendererError::NativeMetal("native Metal upload byte count overflow".into())
            })
        })
    }

    fn nonempty_count(self) -> u64 {
        [
            self.flush_uniforms,
            self.gradient_spans,
            self.tessellation_spans,
            self.paths,
            self.paints,
            self.paint_aux,
            self.contours,
            self.triangles,
        ]
        .into_iter()
        .filter(|bytes| *bytes != 0)
        .count() as u64
    }
}

struct UploadedBuffers {
    flush_uniforms: Retained<ProtocolObject<dyn MTLBuffer>>,
    gradient_spans: Option<Retained<ProtocolObject<dyn MTLBuffer>>>,
    tessellation_spans: Retained<ProtocolObject<dyn MTLBuffer>>,
    paths: Retained<ProtocolObject<dyn MTLBuffer>>,
    paints: Retained<ProtocolObject<dyn MTLBuffer>>,
    paint_aux: Retained<ProtocolObject<dyn MTLBuffer>>,
    contours: Retained<ProtocolObject<dyn MTLBuffer>>,
    triangles: Option<Retained<ProtocolObject<dyn MTLBuffer>>>,
}

impl UploadRings {
    fn ensure_capacities(
        &mut self,
        device: &ProtocolObject<dyn MTLDevice>,
        sizes: UploadSizes,
    ) -> Result<(), RendererError> {
        ensure_upload_capacity(device, &mut self.flush_uniforms, sizes.flush_uniforms)?;
        if sizes.gradient_spans != 0 {
            ensure_upload_capacity(device, &mut self.gradient_spans, sizes.gradient_spans)?;
        }
        ensure_upload_capacity(
            device,
            &mut self.tessellation_spans,
            sizes.tessellation_spans,
        )?;
        ensure_upload_capacity(device, &mut self.paths, sizes.paths)?;
        ensure_upload_capacity(device, &mut self.paints, sizes.paints)?;
        ensure_upload_capacity(device, &mut self.paint_aux, sizes.paint_aux)?;
        ensure_upload_capacity(device, &mut self.contours, sizes.contours)?;
        if sizes.triangles != 0 {
            ensure_upload_capacity(device, &mut self.triangles, sizes.triangles)?;
        }
        Ok(())
    }

    fn upload(&mut self, batch: UploadBatch<'_>) -> Result<UploadedBuffers, RendererError> {
        Ok(UploadedBuffers {
            flush_uniforms: upload_value(
                required_upload_ring(&mut self.flush_uniforms, "flush uniforms")?,
                batch.flush_uniforms,
            )?,
            gradient_spans: (!batch.gradient_spans.is_empty())
                .then(|| {
                    upload_slice(
                        required_upload_ring(&mut self.gradient_spans, "gradient spans")?,
                        batch.gradient_spans,
                    )
                })
                .transpose()?,
            tessellation_spans: upload_slice(
                required_upload_ring(&mut self.tessellation_spans, "tessellation spans")?,
                batch.tessellation_spans,
            )?,
            paths: upload_slice(required_upload_ring(&mut self.paths, "paths")?, batch.paths)?,
            paints: upload_slice(
                required_upload_ring(&mut self.paints, "paints")?,
                batch.paints,
            )?,
            paint_aux: upload_slice(
                required_upload_ring(&mut self.paint_aux, "paint aux")?,
                batch.paint_aux,
            )?,
            contours: upload_slice(
                required_upload_ring(&mut self.contours, "contours")?,
                batch.contours,
            )?,
            triangles: (!batch.triangles.is_empty())
                .then(|| {
                    upload_slice(
                        required_upload_ring(&mut self.triangles, "triangles")?,
                        batch.triangles,
                    )
                })
                .transpose()?,
        })
    }
}

const fn specialized_atlas_blit_job() -> BackgroundCompileJob {
    BackgroundCompileJob::new(
        DrawType::AtlasBlit,
        ENABLE_DITHER,
        InterlockMode::RasterOrdering,
        0,
    )
}

const fn atomic_path_jobs() -> [BackgroundCompileJob; 3] {
    [
        BackgroundCompileJob::new(
            DrawType::RenderPassInitialize,
            ENABLE_DITHER,
            InterlockMode::Atomics,
            FIXED_FUNCTION_COLOR_OUTPUT,
        ),
        BackgroundCompileJob::new(
            DrawType::MidpointFanPatches,
            ENABLE_DITHER,
            InterlockMode::Atomics,
            FIXED_FUNCTION_COLOR_OUTPUT,
        ),
        BackgroundCompileJob::new(
            DrawType::RenderPassResolve,
            ENABLE_DITHER,
            InterlockMode::Atomics,
            FIXED_FUNCTION_COLOR_OUTPUT,
        ),
    ]
}

fn compile_atomic_path_pipelines(
    device: &Retained<ProtocolObject<dyn MTLDevice>>,
    capabilities: MetalCapabilitySelection,
    platform: ApplePlatform,
) -> Result<AtomicPathPipelines, RendererError> {
    let [initialize, midpoint, resolve] = atomic_path_jobs();
    Ok(AtomicPathPipelines {
        initialize: compile_specialized_draw_pipeline(device, capabilities, platform, initialize)?,
        midpoint: compile_specialized_draw_pipeline(device, capabilities, platform, midpoint)?,
        resolve: compile_specialized_draw_pipeline(device, capabilities, platform, resolve)?,
    })
}

fn compile_specialized_atlas_blit_pipeline(
    device: &Retained<ProtocolObject<dyn MTLDevice>>,
    capabilities: MetalCapabilitySelection,
    platform: ApplePlatform,
    job: BackgroundCompileJob,
) -> Result<DrawPipeline, RendererError> {
    compile_specialized_draw_pipeline(device, capabilities, platform, job)
}

fn compile_specialized_draw_pipeline(
    device: &Retained<ProtocolObject<dyn MTLDevice>>,
    capabilities: MetalCapabilitySelection,
    platform: ApplePlatform,
    job: BackgroundCompileJob,
) -> Result<DrawPipeline, RendererError> {
    let compiler = NativeBackgroundShaderCompiler::new_metal(
        device.clone(),
        MetalFeatures {
            atomic_barrier_type: capabilities.atomic_barrier_type,
        },
        platform,
    );
    compiler.push_job(job);
    let finished = compiler.pop_finished_job(true).ok_or_else(|| {
        RendererError::NativeMetal(
            "specialized Metal compiler returned no completed job".to_owned(),
        )
    })?;
    debug_assert_eq!(finished.job, job);
    let library = finished.result.map_err(|error| {
        let detail = match error {
            BackgroundShaderCompileError::MetalCompilation {
                localized_description,
                ..
            } => localized_description.unwrap_or_else(|| "Metal returned no diagnostic".to_owned()),
            other => format!("{other:?}"),
        };
        RendererError::NativeMetal(format!(
            "compile specialized {:?} shader synchronously: {detail}",
            job.draw_type
        ))
    })?;
    let interlock_mode = match job.interlock_mode {
        InterlockMode::RasterOrdering => MetalInterlockMode::RasterOrdering,
        InterlockMode::Atomics => MetalInterlockMode::Atomics,
        _ => {
            return Err(RendererError::NativeMetal(format!(
                "native Metal cannot realize {:?} interlock",
                job.interlock_mode
            )))
        }
    };
    DrawPipeline::new(
        device,
        Some(&library),
        &NSString::from_str(SPECIALIZED_DRAW_VERTEX_MAIN),
        &NSString::from_str(SPECIALIZED_DRAW_FRAGMENT_MAIN),
        job.draw_type,
        interlock_mode,
        job.shader_misc_flags,
    )
    .map_err(|error| RendererError::NativeMetal(error.to_string()))
}

fn upload_byte_len<T>(values: &[T]) -> Result<usize, RendererError> {
    values
        .len()
        .checked_mul(std::mem::size_of::<T>())
        .filter(|length| *length != 0)
        .ok_or_else(|| {
            RendererError::NativeMetal(
                "native Metal upload payload is empty or exceeds address space".to_owned(),
            )
        })
}

fn optional_upload_byte_len<T>(values: &[T]) -> Result<usize, RendererError> {
    values
        .len()
        .checked_mul(std::mem::size_of::<T>())
        .ok_or_else(|| {
            RendererError::NativeMetal(
                "native Metal optional upload payload exceeds address space".to_owned(),
            )
        })
}

fn ensure_upload_capacity(
    device: &ProtocolObject<dyn MTLDevice>,
    ring: &mut Option<UploadBufferRing>,
    required_bytes: usize,
) -> Result<(), RendererError> {
    if ring
        .as_ref()
        .is_some_and(|ring| ring.capacity() >= required_bytes)
    {
        return Ok(());
    }
    *ring = UploadBufferRing::new(device, required_bytes)?;
    if ring.is_none() {
        return Err(RendererError::NativeMetal(
            "native Metal upload ring requires nonzero capacity".to_owned(),
        ));
    }
    Ok(())
}

fn required_upload_ring<'a>(
    ring: &'a mut Option<UploadBufferRing>,
    label: &str,
) -> Result<&'a mut UploadBufferRing, RendererError> {
    ring.as_mut().ok_or_else(|| {
        RendererError::NativeMetal(format!("native Metal {label} upload ring is absent"))
    })
}

fn upload_value<T: bytemuck::Pod>(
    ring: &mut UploadBufferRing,
    value: &T,
) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, RendererError> {
    upload_slice(ring, std::slice::from_ref(value))
}

fn upload_slice<T: bytemuck::Pod>(
    ring: &mut UploadBufferRing,
    values: &[T],
) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, RendererError> {
    let bytes = bytemuck::cast_slice(values);
    ring.map(bytes.len())
        .map_err(|error| RendererError::NativeMetal(format!("map upload ring: {error}")))?
        .copy_from_slice(bytes);
    ring.unmap_submit()
        .map_err(|error| RendererError::NativeMetal(format!("submit upload ring: {error}")))?;
    ring.retained_submitted_buffer()
}

fn make_resource_pipeline(
    device: &ProtocolObject<dyn MTLDevice>,
    library: &ProtocolObject<dyn MTLLibrary>,
    vertex_name: &str,
    fragment_name: &str,
    pixel_format: MTLPixelFormat,
) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, RendererError> {
    let vertex = library
        .newFunctionWithName(&NSString::from_str(vertex_name))
        .ok_or_else(|| {
            RendererError::NativeMetal(format!("missing resource vertex {vertex_name}"))
        })?;
    let fragment = library
        .newFunctionWithName(&NSString::from_str(fragment_name))
        .ok_or_else(|| {
            RendererError::NativeMetal(format!("missing resource fragment {fragment_name}"))
        })?;
    let descriptor = MTLRenderPipelineDescriptor::new();
    descriptor.setVertexFunction(Some(&vertex));
    descriptor.setFragmentFunction(Some(&fragment));
    // SAFETY: Metal render-pipeline descriptors always expose eight color
    // attachment descriptor slots; upstream configures slot zero for both
    // resource pipelines, and `descriptor` retains the returned attachment.
    let attachment = unsafe { descriptor.colorAttachments().objectAtIndexedSubscript(0) };
    attachment.setPixelFormat(pixel_format);
    device
        .newRenderPipelineStateWithDescriptor_error(&descriptor)
        .map_err(|error| {
            RendererError::NativeMetal(format!(
                "create {vertex_name}/{fragment_name} resource pipeline: {error:?}"
            ))
        })
}

fn make_buffer<T: bytemuck::Pod>(
    device: &ProtocolObject<dyn MTLDevice>,
    values: &[T],
) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, RendererError> {
    let bytes: &[u8] = bytemuck::cast_slice(values);
    let pointer = NonNull::new(bytes.as_ptr().cast_mut().cast::<c_void>()).ok_or_else(|| {
        RendererError::NativeMetal("native Metal static buffer pointer is null".to_owned())
    })?;
    // SAFETY: `T: Pod` makes the complete slice initialized plain data, and
    // `newBufferWithBytes` copies all `bytes.len()` bytes before it returns.
    // The source slice therefore remains valid for the entire Objective-C call.
    unsafe {
        device.newBufferWithBytes_length_options(
            pointer,
            bytes.len(),
            MTLResourceOptions::StorageModeShared,
        )
    }
    .ok_or_else(|| {
        RendererError::NativeMetal("failed to allocate native Metal static buffer".to_owned())
    })
}

fn make_gaussian_integral_texture(
    device: &ProtocolObject<dyn MTLDevice>,
) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, RendererError> {
    let descriptor = MTLTextureDescriptor::new();
    descriptor.setPixelFormat(MTLPixelFormat::R16Float);
    descriptor.setTextureType(MTLTextureType::Type1DArray);
    descriptor.setUsage(MTLTextureUsage::ShaderRead);
    // SAFETY: these are the exact non-zero upstream Gaussian table dimensions;
    // all values fit the typed Metal `NSUInteger`/Rust `usize` parameters.
    unsafe {
        descriptor.setWidth(crate::feather_lut::TABLE_SIZE);
        descriptor.setMipmapLevelCount(1);
        descriptor.setArrayLength(2);
    }
    let texture = device
        .newTextureWithDescriptor(&descriptor)
        .ok_or_else(|| {
            RendererError::NativeMetal(
                "failed to allocate native Metal Gaussian-integral texture".to_owned(),
            )
        })?;
    let rows = crate::feather_lut::table_rows();
    let bytes_per_row = crate::feather_lut::TABLE_SIZE * std::mem::size_of::<u16>();
    let region = MTLRegion {
        origin: MTLOrigin { x: 0, y: 0, z: 0 },
        size: MTLSize {
            width: crate::feather_lut::TABLE_SIZE,
            height: 1,
            depth: 1,
        },
    };
    for (slice, row) in rows.iter().enumerate() {
        let pointer = NonNull::new(row.as_ptr().cast_mut().cast::<c_void>()).ok_or_else(|| {
            RendererError::NativeMetal(
                "native Metal Gaussian-integral table pointer is null".to_owned(),
            )
        })?;
        // SAFETY: each source row contains exactly `TABLE_SIZE` initialized
        // `u16` texels, `region` selects one matching R16Float row, and Metal
        // consumes the borrowed bytes synchronously during `replaceRegion`.
        unsafe {
            texture.replaceRegion_mipmapLevel_slice_withBytes_bytesPerRow_bytesPerImage(
                region,
                0,
                slice,
                pointer,
                bytes_per_row,
                bytes_per_row,
            );
        }
    }
    Ok(texture)
}

fn require_command_buffer(
    command_buffer: Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>,
) -> Result<Retained<ProtocolObject<dyn MTLCommandBuffer>>, RendererError> {
    command_buffer.ok_or_else(|| {
        RendererError::NativeMetal("MTLCommandQueue returned no command buffer".to_owned())
    })
}

fn command_buffer_completion_result(
    status: MTLCommandBufferStatus,
    error: Option<String>,
) -> Result<(), RendererError> {
    if status == MTLCommandBufferStatus::Completed {
        return Ok(());
    }
    let detail = error.unwrap_or_else(|| format!("status {status:?}"));
    Err(RendererError::NativeMetal(format!(
        "command buffer failed: {detail}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::Zeroable;
    use objc2::rc::{Retained, Weak};
    use objc2_metal::{MTLCreateSystemDefaultDevice, MTLTexture};

    struct UploadFixture {
        flush_uniforms: gpu::FlushUniforms,
        gradient_spans: [gpu::GradientSpan; 1],
        tessellation_spans: [gpu::TessVertexSpan; 1],
        paths: [gpu::PathData; 1],
        paints: [gpu::PaintData; 1],
        paint_aux: [gpu::PaintAuxData; 1],
        contours: [gpu::ContourData; 1],
        triangles: [gpu::TriangleVertex; 1],
    }

    impl UploadFixture {
        fn new() -> Self {
            Self {
                flush_uniforms: gpu::FlushUniforms::zeroed(),
                gradient_spans: [gpu::GradientSpan::zeroed()],
                tessellation_spans: [gpu::TessVertexSpan::zeroed()],
                paths: [gpu::PathData::zeroed()],
                paints: [gpu::PaintData::zeroed()],
                paint_aux: [gpu::PaintAuxData::zeroed()],
                contours: [gpu::ContourData::zeroed()],
                triangles: [gpu::TriangleVertex::zeroed()],
            }
        }

        fn batch(&self) -> UploadBatch<'_> {
            UploadBatch {
                flush_uniforms: &self.flush_uniforms,
                gradient_spans: &self.gradient_spans,
                tessellation_spans: &self.tessellation_spans,
                paths: &self.paths,
                paints: &self.paints,
                paint_aux: &self.paint_aux,
                contours: &self.contours,
                triangles: &self.triangles,
            }
        }
    }

    fn live_context() -> Option<NativeMetalContext> {
        let device = MTLCreateSystemDefaultDevice()?;
        let platform = super::super::select_apple_platform(&device);
        let capabilities = super::super::select_device_capabilities(&device, platform);
        Some(
            NativeMetalContext::new(device, capabilities, platform)
                .expect("create native Metal context"),
        )
    }

    fn texture_identity(texture: &Retained<ProtocolObject<dyn MTLTexture>>) -> *const () {
        Retained::as_ptr(texture) as *const ()
    }

    #[test]
    fn context_owns_exact_upstream_image_rect_buffers_until_drop() {
        use objc2_metal::MTLResource;

        let Some(context) = live_context() else {
            return;
        };
        let vertex_bytes: &[u8] = bytemuck::cast_slice(&gpu::IMAGE_RECT_VERTICES);
        let index_bytes: &[u8] = bytemuck::cast_slice(&gpu::IMAGE_RECT_INDICES);
        assert_eq!(
            context._image_rect_vertex_buffer.length(),
            vertex_bytes.len()
        );
        assert_eq!(context._image_rect_index_buffer.length(), index_bytes.len());
        assert_eq!(
            context._image_rect_vertex_buffer.storageMode(),
            objc2_metal::MTLStorageMode::Shared
        );
        assert_eq!(
            context._image_rect_index_buffer.storageMode(),
            objc2_metal::MTLStorageMode::Shared
        );
        // SAFETY: both shared buffers are retained by `context`, their
        // contents pointers cover exactly the lengths reported by Metal, and
        // no mutation occurs while these byte slices are borrowed.
        unsafe {
            assert_eq!(
                std::slice::from_raw_parts(
                    context
                        ._image_rect_vertex_buffer
                        .contents()
                        .as_ptr()
                        .cast::<u8>(),
                    vertex_bytes.len(),
                ),
                vertex_bytes
            );
            assert_eq!(
                std::slice::from_raw_parts(
                    context
                        ._image_rect_index_buffer
                        .contents()
                        .as_ptr()
                        .cast::<u8>(),
                    index_bytes.len(),
                ),
                index_bytes
            );
        }
        let vertex_owner = Weak::new(&*context._image_rect_vertex_buffer);
        let index_owner = Weak::new(&*context._image_rect_index_buffer);
        drop(context);
        assert!(vertex_owner.load().is_none());
        assert!(index_owner.load().is_none());
    }

    fn current_texture_identities(context: &NativeMetalContext) -> [Option<*const ()>; 3] {
        let state = context.resources.lock().unwrap();
        [
            state
                .generation
                .gradient
                .retained_texture()
                .as_ref()
                .map(texture_identity),
            state
                .generation
                .tessellation
                .retained_texture()
                .as_ref()
                .map(texture_identity),
            state
                .generation
                .feather_atlas
                .as_ref()
                .and_then(FeatherAtlasResource::retained_texture)
                .as_ref()
                .map(texture_identity),
        ]
    }

    #[test]
    fn overlapping_resource_leases_retain_distinct_complete_generations() {
        let Some(context) = live_context() else {
            return;
        };
        let fixture = UploadFixture::new();
        let first = context
            .prepare_resources(2, 2, Some([16, 12]), Some(true), fixture.batch())
            .expect("prepare first concrete generation");
        let first_gradient = first.gradient.as_ref().unwrap();
        let first_tessellation = &first.tessellation;
        let first_atlas = first.feather_atlas.as_ref().unwrap();
        let first_ids = [
            texture_identity(first_gradient),
            texture_identity(first_tessellation),
            texture_identity(first_atlas),
        ];
        let first_weak = [
            Weak::new(&**first_gradient),
            Weak::new(&**first_tessellation),
            Weak::new(&**first_atlas),
        ];

        let second = context
            .prepare_resources(3, 4, Some([24, 20]), Some(true), fixture.batch())
            .expect("prepare replacement concrete generation");
        let second_ids = [
            texture_identity(second.gradient.as_ref().unwrap()),
            texture_identity(&second.tessellation),
            texture_identity(second.feather_atlas.as_ref().unwrap()),
        ];
        assert_ne!(first_ids, second_ids);
        assert_eq!(current_texture_identities(&context), second_ids.map(Some));
        assert!(first_weak.iter().all(|owner| owner.load().is_some()));
        assert_eq!((first_gradient.width(), first_gradient.height()), (512, 2));
        assert_eq!(
            (first_tessellation.width(), first_tessellation.height()),
            (2048, 2)
        );
        assert_eq!((first_atlas.width(), first_atlas.height()), (16, 12));

        drop(first);
        assert!(first_weak.iter().all(|owner| owner.load().is_none()));
        drop(second);
        assert_eq!(current_texture_identities(&context), second_ids.map(Some));
    }

    #[test]
    fn later_tessellation_failure_does_not_publish_staged_gradient() {
        let Some(context) = live_context() else {
            return;
        };
        let fixture = UploadFixture::new();
        drop(
            context
                .prepare_resources(2, 2, None, None, fixture.batch())
                .expect("prepare baseline generation"),
        );
        let baseline = current_texture_identities(&context);
        let mut fail_tessellation = |stage| {
            if stage == ResourcePreparationStage::TessellationTexture {
                Err(RendererError::NativeMetal(
                    "injected tessellation allocation failure".to_owned(),
                ))
            } else {
                Ok(())
            }
        };
        assert!(context
            .prepare_resources_with_control(
                3,
                4,
                None,
                None,
                fixture.batch(),
                specialized_atlas_blit_job(),
                &mut fail_tessellation,
            )
            .is_err());
        assert_eq!(current_texture_identities(&context), baseline);

        let replacement = context
            .prepare_resources(3, 4, None, None, fixture.batch())
            .expect("retry after staged tessellation failure");
        assert_ne!(current_texture_identities(&context), baseline);
        drop(replacement);
    }

    #[test]
    fn absent_required_tessellation_owner_does_not_publish_candidate_generation() {
        let Some(context) = live_context() else {
            return;
        };
        let fixture = UploadFixture::new();
        drop(
            context
                .prepare_resources(2, 2, None, None, fixture.batch())
                .expect("prepare baseline generation"),
        );
        let baseline = current_texture_identities(&context);

        assert!(matches!(
            context.prepare_resources(3, 0, None, None, fixture.batch()),
            Err(RendererError::NativeMetal(message))
                if message == "tessellation resource texture is absent"
        ));
        assert_eq!(current_texture_identities(&context), baseline);

        drop(
            context
                .prepare_resources(3, 4, None, None, fixture.batch())
                .expect("retry after absent candidate owner"),
        );
        assert_ne!(current_texture_identities(&context), baseline);
    }

    #[test]
    fn atlas_growth_failure_preserves_prior_complete_generation() {
        let Some(context) = live_context() else {
            return;
        };
        let fixture = UploadFixture::new();
        drop(
            context
                .prepare_resources(2, 2, Some([16, 12]), Some(false), fixture.batch())
                .expect("prepare baseline atlas generation"),
        );
        let baseline = current_texture_identities(&context);
        let baseline_pipeline = {
            let state = context.resources.lock().unwrap();
            let retained = state
                .generation
                .feather_atlas_pipelines
                .as_ref()
                .unwrap()
                .retained(false);
            Retained::as_ptr(&retained) as *const ()
        };
        let mut fail_atlas = |stage| {
            if stage == ResourcePreparationStage::FeatherAtlasTexture {
                Err(RendererError::NativeMetal(
                    "injected atlas allocation failure".to_owned(),
                ))
            } else {
                Ok(())
            }
        };
        assert!(context
            .prepare_resources_with_control(
                3,
                4,
                Some([24, 20]),
                Some(false),
                fixture.batch(),
                specialized_atlas_blit_job(),
                &mut fail_atlas,
            )
            .is_err());
        assert_eq!(current_texture_identities(&context), baseline);
        let state = context.resources.lock().unwrap();
        let retained = state
            .generation
            .feather_atlas_pipelines
            .as_ref()
            .unwrap()
            .retained(false);
        assert_eq!(Retained::as_ptr(&retained) as *const (), baseline_pipeline);
        drop(state);

        drop(
            context
                .prepare_resources(3, 4, Some([24, 20]), Some(false), fixture.batch())
                .expect("retry atlas growth"),
        );
        assert_ne!(current_texture_identities(&context), baseline);
    }

    #[test]
    fn specialized_compile_failure_publishes_nothing_and_releases_reservation() {
        use crate::native_metal::shader_compile_plan::SynthesizedFailureType;

        let Some(context) = live_context() else {
            return;
        };
        let fixture = UploadFixture::new();
        drop(
            context
                .prepare_resources(2, 2, None, None, fixture.batch())
                .expect("prepare baseline generation"),
        );
        let baseline = current_texture_identities(&context);
        let failure_job = specialized_atlas_blit_job()
            .with_synthesized_failure(SynthesizedFailureType::ShaderCompilation);

        for _ in 0..4 {
            let mut allow = |_| Ok(());
            let error = match context.prepare_resources_with_control(
                3,
                4,
                Some([16, 12]),
                Some(true),
                fixture.batch(),
                failure_job,
                &mut allow,
            ) {
                Err(error) => error,
                Ok(_) => panic!("injected specialized compilation must fail"),
            };
            assert!(matches!(
                error,
                RendererError::NativeMetal(message)
                    if message.contains("SynthesizedShaderCompilation")
            ));
            assert_eq!(current_texture_identities(&context), baseline);
            let state = context.resources.lock().unwrap();
            assert!(state.generation.feather_atlas.is_none());
            assert!(state.generation.feather_atlas_pipelines.is_none());
            assert!(state.generation.specialized_atlas_blit_pipeline.is_none());
        }

        let replacement = context
            .prepare_resources(3, 4, Some([16, 12]), Some(true), fixture.batch())
            .expect("retry after repeated specialized compilation failures");
        assert_ne!(current_texture_identities(&context), baseline);
        assert!(replacement.feather_atlas.is_some());
        drop(replacement);
    }

    #[test]
    fn missing_command_buffer_acquisition_fails_closed() {
        assert!(matches!(
            require_command_buffer(None),
            Err(RendererError::NativeMetal(message))
                if message == "MTLCommandQueue returned no command buffer"
        ));
    }

    #[test]
    fn completion_status_propagates_success_and_failure_details() {
        assert!(command_buffer_completion_result(MTLCommandBufferStatus::Completed, None).is_ok());
        assert!(matches!(
            command_buffer_completion_result(
                MTLCommandBufferStatus::Error,
                Some("synthetic Metal failure".to_owned()),
            ),
            Err(RendererError::NativeMetal(message))
                if message == "command buffer failed: synthetic Metal failure"
        ));
        assert!(matches!(
            command_buffer_completion_result(MTLCommandBufferStatus::Committed, None),
            Err(RendererError::NativeMetal(message))
                if message.starts_with("command buffer failed: status ")
        ));
    }

    #[test]
    fn upload_capacity_is_verbatim_reused_and_grows_by_replacement() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            return;
        };
        let mut ring = None;
        ensure_upload_capacity(&device, &mut ring, 4).unwrap();
        let initial = ring.as_ref().unwrap().buffer_identity(0);
        assert_eq!(ring.as_ref().unwrap().capacity(), 4);

        ensure_upload_capacity(&device, &mut ring, 4).unwrap();
        assert_eq!(ring.as_ref().unwrap().buffer_identity(0), initial);

        ensure_upload_capacity(&device, &mut ring, 9).unwrap();
        assert_eq!(ring.as_ref().unwrap().capacity(), 9);
        assert_ne!(ring.as_ref().unwrap().buffer_identity(0), initial);
    }

    #[test]
    fn empty_upload_payload_is_rejected_before_metal_mapping() {
        let empty: [gpu::GradientSpan; 0] = [];
        assert!(matches!(
            upload_byte_len(&empty),
            Err(RendererError::NativeMetal(message))
                if message == "native Metal upload payload is empty or exceeds address space"
        ));
    }

    #[test]
    fn later_upload_failure_leaves_earlier_rings_submitted_not_mapped() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            return;
        };
        let uniforms = gpu::FlushUniforms::zeroed();
        let gradient_spans = [gpu::GradientSpan::zeroed()];
        let tessellation_spans = [gpu::TessVertexSpan::zeroed()];
        let paths = [gpu::PathData::zeroed()];
        let paints = [gpu::PaintData::zeroed()];
        let paint_aux = [gpu::PaintAuxData::zeroed()];
        let contours = [gpu::ContourData::zeroed()];
        let batch = UploadBatch {
            flush_uniforms: &uniforms,
            gradient_spans: &gradient_spans,
            tessellation_spans: &tessellation_spans,
            paths: &paths,
            paints: &paints,
            paint_aux: &paint_aux,
            contours: &contours,
            triangles: &[],
        };
        let mut rings = UploadRings {
            flush_uniforms: UploadBufferRing::new(&device, std::mem::size_of_val(&uniforms))
                .unwrap(),
            gradient_spans: UploadBufferRing::new(&device, std::mem::size_of_val(&gradient_spans))
                .unwrap(),
            ..UploadRings::default()
        };

        assert!(matches!(
            rings.upload(batch),
            Err(RendererError::NativeMetal(message))
                if message == "native Metal tessellation spans upload ring is absent"
        ));
        assert!(rings
            .flush_uniforms
            .as_ref()
            .unwrap()
            .retained_submitted_buffer()
            .is_ok());
        assert!(rings
            .gradient_spans
            .as_ref()
            .unwrap()
            .retained_submitted_buffer()
            .is_ok());
    }
}
