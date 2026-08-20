//! Experimental native Metal renderer adapter.
//!
//! The Apple product does not select this adapter until UNIV-2092. The module
//! begins as the UNIV-2086 tracer and grows by mechanically porting the pinned
//! upstream Metal implementation behind the existing renderer seam.

#[allow(dead_code)]
mod background_shader_compiler;
mod buffer;
mod buffer_ring_coordinator;
mod capabilities;
mod context;
#[allow(dead_code)]
mod draw_combinations;
#[allow(dead_code)]
mod draw_pipeline;
#[allow(dead_code)]
mod draw_shader;
mod drawable;
mod feather_atlas_pipeline;
mod feather_atlas_resource;
mod gradient_resource;
#[allow(dead_code)]
mod image_texture;
mod pipeline_cache;
#[allow(dead_code)]
mod pipeline_names;
mod render_canvas;
#[allow(dead_code)]
mod render_target;
#[allow(dead_code)]
mod samplers;
#[allow(dead_code)]
mod shader_compile_plan;
mod tessellation_resource;
mod upload_buffer_ring;

use super::gpu;
use super::{
    logical_frame::{
        prepare_atomic_clipped_path_flush, prepare_atomic_path_flush,
        prepare_single_gradient_batch, AtomicPathFlushInput, LogicalDrawState,
        PreparedAtomicPatchKind, PreparedAtomicPathFlush,
    },
    BackendWorkMetrics, LogicalFrameConfig, LogicalPaint, LogicalPath, LogicalShader, RenderMode,
    RendererError,
};
use buffer::NativeMetalBuffer;
use bytemuck::{Pod, Zeroable};
use capabilities::{
    select_capabilities, ApplePlatform, AtomicBarrierType, MetalCapabilitySelection,
    MetalDeviceCapabilities,
};
use context::NativeMetalContext;
use image_texture::NativeMetalTextureFormat;
use nuxie_image_codec::{decode_image_rgba, preflight_encoded_image};
use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageSampler, Mat2D, PathVerb,
    RawPath, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint,
    RenderPaintStyle, RenderPath, RenderShader, Renderer, Vec2D,
};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{msg_send, rc::Retained};
use objc2_foundation::{NSError, NSString};
use objc2_metal::{
    MTLBarrierScope, MTLBlendFactor, MTLClearColor, MTLCommandBuffer, MTLCommandEncoder,
    MTLCreateSystemDefaultDevice, MTLCullMode, MTLDevice, MTLGPUFamily, MTLIndexType, MTLLibrary,
    MTLLoadAction, MTLOrigin, MTLPixelFormat, MTLPrimitiveType, MTLRegion, MTLRenderCommandEncoder,
    MTLRenderPassDescriptor, MTLRenderPipelineDescriptor, MTLRenderPipelineState, MTLRenderStages,
    MTLScissorRect, MTLSize, MTLStorageMode, MTLStoreAction, MTLTexture, MTLTextureDescriptor,
    MTLTextureUsage, MTLViewport,
};
pub use render_canvas::NativeMetalRenderCanvas;
use render_target::RenderTargetMetal;
use std::cell::RefCell;
use std::ffi::c_void;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::Arc;

pub use drawable::NativeMetalDrawableFrame;
pub use pipeline_cache::{NativeMetalContextOptions, ShaderCompilationMode};

const TRACER_METALLIB: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/native_metal_tracer.metallib"));
const INLINE_VERTEX_BYTE_LIMIT: usize = 4_096;

#[link(name = "System")]
extern "C" {
    fn dispatch_data_create(
        buffer: NonNull<c_void>,
        size: usize,
        queue: Option<NonNull<c_void>>,
        destructor: *mut c_void,
    ) -> *mut AnyObject;
    fn dispatch_release(object: *mut AnyObject);
}

/// Explicitly selected native Metal renderer domain.
pub struct NativeMetalFactory {
    context: Arc<NativeMetalContext>,
    target: Rc<RefCell<RenderTargetMetal>>,
    mode: RenderMode,
}

/// Concrete native-Metal frame result used by parity and performance oracles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeMetalFrameOutput {
    pub pixels: Vec<u8>,
    pub backend_work: BackendWorkMetrics,
    pub execution_inventory: NativeMetalExecutionInventory,
}

/// Concrete native execution selected and realized by one completed frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeMetalExecutionInventory {
    pub mode: RenderMode,
    pub color_ramp_pipeline: bool,
    pub gradient_texture: bool,
    /// Fixed-function atomic SrcOver does not use the offscreen color plane.
    pub atomic_color_plane: bool,
    pub advanced_blend_pipeline: bool,
    pub hsl_blend_pipeline: bool,
    pub fixed_function_color_output: bool,
    pub atomic_clip_plane: bool,
    pub atomic_coverage_plane: bool,
    pub render_pass_initialize_pipeline: bool,
    pub midpoint_fan_pipeline: bool,
    pub render_pass_resolve_pipeline: bool,
    /// Flush-wide clipping selected specialized initialize/midpoint/resolve.
    pub clipped_path_pipeline_set: bool,
    /// At least one path used the fixed-function atomic clip-rectangle shader.
    pub clip_rect_pipeline: bool,
    pub outer_curve_pipeline: bool,
    pub interior_triangulation_pipeline: bool,
    pub atomic_draws: usize,
    pub atomic_draw_groups: usize,
    /// Semantic PLS barriers: initial, group transitions, and pre-resolve.
    pub atomic_barriers: usize,
    pub atomic_memory_barriers: usize,
    pub atomic_render_pass_breaks: usize,
}

impl NativeMetalFactory {
    pub fn new(width: u32, height: u32) -> Result<Self, RendererError> {
        Self::new_impl(width, height, None, NativeMetalContextOptions::default())
    }

    pub fn new_with_mode(width: u32, height: u32, mode: RenderMode) -> Result<Self, RendererError> {
        Self::new_impl(
            width,
            height,
            Some(mode),
            NativeMetalContextOptions::default(),
        )
    }

    pub fn new_with_context_options(
        width: u32,
        height: u32,
        options: NativeMetalContextOptions,
    ) -> Result<Self, RendererError> {
        Self::new_impl(width, height, None, options)
    }

    pub fn new_with_mode_and_context_options(
        width: u32,
        height: u32,
        mode: RenderMode,
        options: NativeMetalContextOptions,
    ) -> Result<Self, RendererError> {
        Self::new_impl(width, height, Some(mode), options)
    }

    fn new_impl(
        width: u32,
        height: u32,
        requested_mode: Option<RenderMode>,
        options: NativeMetalContextOptions,
    ) -> Result<Self, RendererError> {
        let device = MTLCreateSystemDefaultDevice()
            .ok_or_else(|| RendererError::NativeMetal("no system Metal device".to_owned()))?;
        let platform = select_apple_platform(&device);
        let capabilities =
            select_device_capabilities(&device, platform, options.disable_framebuffer_reads);
        // Preserve the established failure order: invalid target dimensions
        // are rejected immediately after the capability probe, before queue,
        // library, sampler, pipeline, or target allocation can mask them.
        validate_extent(width, height, capabilities.max_texture_size)?;
        let mode = select_native_metal_mode(capabilities, requested_mode)?;
        let queue = device.newCommandQueue().ok_or_else(|| {
            RendererError::NativeMetal("MTLDevice returned no command queue".to_owned())
        })?;
        let context = Arc::new(NativeMetalContext::new_with_queue_and_options(
            device,
            queue,
            capabilities,
            platform,
            options,
        )?);
        let target = Rc::new(RefCell::new(make_tracer_target(&context, width, height)?));
        Ok(Self {
            context,
            target,
            mode,
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        let target = self.target.borrow();
        (target.width(), target.height())
    }

    pub fn adapter_name(&self) -> String {
        self.context.device().name().to_string()
    }

    pub fn render_mode(&self) -> RenderMode {
        self.mode
    }

    /// Replaces the dimensions used by subsequently created frames. Frames
    /// already handed to a caller retain their original size, matching the
    /// upstream target-owner boundary during an in-flight resize.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        validate_extent(width, height, self.context.capabilities().max_texture_size)?;
        // Construct every size-dependent Metal owner before replacement. If
        // any allocation fails, the current generation remains intact.
        let replacement = Rc::new(RefCell::new(make_tracer_target(
            &self.context,
            width,
            height,
        )?));
        self.target = replacement;
        Ok(())
    }

    pub fn begin_frame(&self, clear_color: u32) -> Result<NativeMetalFrame, RendererError> {
        self.begin_frame_for_benchmark(clear_color, false)
    }

    /// Starts one frame with optional concrete backend-work accounting.
    ///
    /// These counters describe Rust's Metal command topology. In the shared
    /// metric vocabulary, one Metal command buffer is the top-level
    /// `command_encoders` unit and its render encoders are `render_passes`.
    /// The pinned C++ renderer supplies the structural oracle but exposes no
    /// numerical counter API, so callers must not interpret them as C++
    /// numeric equality.
    pub fn begin_frame_for_benchmark(
        &self,
        clear_color: u32,
        collect_work_metrics: bool,
    ) -> Result<NativeMetalFrame, RendererError> {
        // Acquisition happens here, once. The resulting concrete Metal owner
        // moves into the frame and is either committed by `finish` or released
        // uncommitted when the frame is abandoned.
        let command_buffer = self.context.make_command_buffer()?;
        Ok(NativeMetalFrame {
            context: Arc::clone(&self.context),
            target: Rc::clone(&self.target),
            mode: self.mode,
            command_buffer,
            clear_color,
            state: NativeMetalRenderState::default(),
            state_stack: Vec::new(),
            atomic_logical_state: LogicalDrawState::default(),
            solid_draws: Vec::new(),
            atomic_path_inputs: Vec::new(),
            gradient_draws: Vec::new(),
            atlas_requests: Vec::new(),
            resource_lease: None,
            collect_work_metrics,
            backend_work: BackendWorkMetrics {
                command_encoders: u64::from(collect_work_metrics),
                ..BackendWorkMetrics::default()
            },
            atomic_draw_count: 0,
            atomic_draw_group_count: 0,
            atomic_barrier_count: 0,
            atomic_memory_barrier_count: 0,
            atomic_render_pass_break_count: 0,
            atomic_uses_clipping: false,
            atomic_uses_clip_rects: false,
            atomic_uses_advanced_blend: false,
            atomic_uses_hsl_blend_modes: false,
            unsupported: None,
        })
    }

    /// Copies the selected device so the platform caller can configure its
    /// presentation owner without transferring that policy to the renderer.
    pub fn retained_metal_device(&self) -> Retained<ProtocolObject<dyn MTLDevice>> {
        self.context.retained_device()
    }

    /// Copies the renderer's ordered command queue for same-context Metal
    /// adapters. Work submitted by ORE/canvas integration must use this queue
    /// rather than selecting a second device or creating an unrelated queue.
    pub fn retained_metal_queue(
        &self,
    ) -> Retained<ProtocolObject<dyn objc2_metal::MTLCommandQueue>> {
        self.context.retained_queue()
    }

    /// Wraps a caller-created Metal texture as a retained renderer image
    /// without allocating storage or uploading bytes.
    pub fn adopt_metal_image_texture(
        &self,
        texture: Retained<ProtocolObject<dyn MTLTexture>>,
        width: u32,
        height: u32,
    ) -> Option<Box<dyn RenderImage>> {
        self.context
            .adopt_image_texture(texture, width, height)
            .map(|image| Box::new(image) as Box<dyn RenderImage>)
    }

    /// Creates a private texture shared by a render-target owner and a
    /// sampleable-image owner, matching the pinned Metal RenderCanvas factory.
    pub fn make_metal_render_canvas(
        &self,
        width: u32,
        height: u32,
    ) -> Result<NativeMetalRenderCanvas, RendererError> {
        validate_extent(width, height, self.context.capabilities().max_texture_size)?;
        self.context.make_render_canvas(width, height)
    }

    /// Constructs ORE from the exact retained device and queue owned by this
    /// renderer context. The opt-in feature corresponds to upstream's
    /// `RIVE_CANVAS` build and cannot select a second Metal service.
    #[cfg(feature = "native-ore-metal-experimental")]
    pub fn make_ore_context(&self) -> nuxie_ore_metal::metal::context::ContextMetal {
        nuxie_ore_metal::metal::context::ContextMetal::make(
            self.context.retained_device(),
            self.context.retained_queue(),
        )
    }

    pub(crate) fn begin_drawable_frame_parts<'a>(
        &self,
        drawable: &'a ProtocolObject<dyn objc2_metal::MTLDrawable>,
        texture: Retained<ProtocolObject<dyn MTLTexture>>,
        clear_color: u32,
    ) -> Result<NativeMetalDrawableFrame<'a>, RendererError> {
        let (expected_width, expected_height) = self.dimensions();
        NativeMetalDrawableFrame::new(
            Arc::clone(&self.context),
            self.mode,
            drawable,
            texture,
            expected_width,
            expected_height,
            clear_color,
        )
    }
}

impl Factory for NativeMetalFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        // The established Factory seam is infallible, while Metal allocation
        // can fail. Terminate at this explicit backend boundary: substituting a
        // CPU buffer or the wgpu backend would conceal the selected renderer
        // and violate the native Metal fail-closed contract.
        Box::new(
            NativeMetalBuffer::new(self.context.device(), buffer_type, flags, size_in_bytes)
                .unwrap_or_else(|error| {
                    panic!("native Metal render-buffer allocation failed: {error}")
                }),
        )
    }

    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        Box::new(LogicalShader::Linear {
            start: (sx, sy),
            end: (ex, ey),
            colors: colors.to_vec(),
            stops: stops.to_vec(),
        })
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        Box::new(LogicalShader::Radial {
            center: (cx, cy),
            radius,
            colors: colors.to_vec(),
            stops: stops.to_vec(),
        })
    }

    fn make_render_path(
        &mut self,
        mut raw_path: RawPath,
        fill_rule: FillRule,
    ) -> Box<dyn RenderPath> {
        raw_path.renew_mutation_id();
        Box::new(LogicalPath {
            raw_path: Arc::new(raw_path),
            fill_rule,
            valid: true,
        })
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        Box::new(LogicalPath {
            raw_path: Arc::new(RawPath::new()),
            fill_rule: FillRule::NonZero,
            valid: true,
        })
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        Box::new(LogicalPaint::default())
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        let dimensions = preflight_encoded_image(data).ok_or(ImageDecodeError)?;
        let max_texture_size = self.context.capabilities().max_texture_size;
        if dimensions.width == 0
            || dimensions.height == 0
            || dimensions.width > max_texture_size
            || dimensions.height > max_texture_size
        {
            return Err(ImageDecodeError);
        }
        let decoded = decode_image_rgba(data).ok_or(ImageDecodeError)?;
        if (decoded.width, decoded.height) != (dimensions.width, dimensions.height) {
            return Err(ImageDecodeError);
        }
        let mip_level_count = u32::BITS - (decoded.width | decoded.height).leading_zeros();
        let texture = self
            .context
            .make_image_texture(
                decoded.width,
                decoded.height,
                mip_level_count,
                NativeMetalTextureFormat::Rgba32,
                &decoded.pixels,
                1,
                1,
                false,
                true,
            )
            .map_err(|_| ImageDecodeError)?;
        Ok(Box::new(texture))
    }
}

/// One native Metal frame retained until submission and deterministic readback.
pub struct NativeMetalFrame {
    context: Arc<NativeMetalContext>,
    target: Rc<RefCell<RenderTargetMetal>>,
    mode: RenderMode,
    command_buffer: Retained<ProtocolObject<dyn MTLCommandBuffer>>,
    clear_color: u32,
    state: NativeMetalRenderState,
    state_stack: Vec<NativeMetalRenderState>,
    atomic_logical_state: LogicalDrawState,
    solid_draws: Vec<SolidTracerDraw>,
    atomic_path_inputs: Vec<AtomicPathInput>,
    gradient_draws: Vec<GradientDraw>,
    atlas_requests: Vec<AtlasRequest>,
    resource_lease: Option<context::PreparedResourceLease>,
    collect_work_metrics: bool,
    backend_work: BackendWorkMetrics,
    atomic_draw_count: usize,
    atomic_draw_group_count: usize,
    atomic_barrier_count: usize,
    atomic_memory_barrier_count: usize,
    atomic_render_pass_break_count: usize,
    atomic_uses_clipping: bool,
    atomic_uses_clip_rects: bool,
    atomic_uses_advanced_blend: bool,
    atomic_uses_hsl_blend_modes: bool,
    unsupported: Option<&'static str>,
}

#[derive(Clone, Copy)]
struct NativeMetalRenderState {
    transform: Mat2D,
    opacity: f32,
}

impl Default for NativeMetalRenderState {
    fn default() -> Self {
        Self {
            transform: Mat2D::IDENTITY,
            opacity: 1.0,
        }
    }
}

struct SolidTracerDraw {
    vertices: Vec<[f32; 2]>,
    premultiplied_color: [f32; 4],
}

struct AtomicPathInput {
    path: LogicalPath,
    paint: LogicalPaint,
    state: super::DrawState,
}

struct GradientDraw {
    gradient_batch: super::logical_frame::GradientBatch,
    gradient: super::logical_frame::PreparedGradient,
    tessellation: super::draw::FillTessellation,
}

struct AtlasRequest {
    path: LogicalPath,
    paint: LogicalPaint,
    state: super::DrawState,
}

struct GradientUploadData {
    flush_uniforms: gpu::FlushUniforms,
    paths: [gpu::PathData; 2],
    paints: [gpu::PaintData; 2],
    paint_aux: [gpu::PaintAuxData; 2],
}

struct AtlasUploadData {
    flush_uniforms: gpu::FlushUniforms,
    paths: Vec<gpu::PathData>,
    paints: Vec<gpu::PaintData>,
    paint_aux: Vec<gpu::PaintAuxData>,
}

struct AtomicPathUploadData {
    flush_uniforms: gpu::FlushUniforms,
    paths: Vec<gpu::PathData>,
    paints: Vec<gpu::PaintData>,
    paint_aux: Vec<gpu::PaintAuxData>,
}

impl AtomicPathUploadData {
    fn new(width: u32, height: u32, clear_color: u32, flush: &PreparedAtomicPathFlush) -> Self {
        let tessellation_height = super::draw::tessellation_texture_height(&flush.spans);
        let mut flush_uniforms = super::analytic_uniforms(width, height, tessellation_height);
        flush_uniforms.color_clear_value = gpu::swizzle_rive_color_to_rgba_premul(clear_color);
        flush_uniforms.coverage_clear_value = 0;
        flush_uniforms.max_path_id = u32::try_from(flush.paths.len()).unwrap_or(u32::MAX);
        flush_uniforms.render_target_update_bounds = [0, 0, width as i32, height as i32];
        if let Some(gradient_batch) = flush.gradient_batch.as_ref() {
            flush_uniforms.inverse_viewports[0] = -2.0 / gradient_batch.height.max(1) as f32;
        }
        let mut paths = Vec::with_capacity(flush.paths.len() + 1);
        paths.push(gpu::PathData::zeroed());
        paths.extend_from_slice(&flush.paths);
        let mut paints = Vec::with_capacity(flush.paints.len() + 1);
        paints.push(gpu::PaintData::solid(
            0,
            FillRule::NonZero,
            BlendMode::SrcOver,
        ));
        paints.extend_from_slice(&flush.paints);
        let mut paint_aux = Vec::with_capacity(flush.paint_aux.len() + 1);
        paint_aux.push(gpu::PaintAuxData::zeroed());
        paint_aux.extend_from_slice(&flush.paint_aux);
        Self {
            flush_uniforms,
            paths,
            paints,
            paint_aux,
        }
    }

    fn batch<'a>(&'a self, flush: &'a PreparedAtomicPathFlush) -> context::UploadBatch<'a> {
        context::UploadBatch {
            flush_uniforms: &self.flush_uniforms,
            gradient_spans: flush
                .gradient_batch
                .as_ref()
                .map_or(&[], |batch| batch.spans.as_slice()),
            tessellation_spans: &flush.spans,
            paths: &self.paths,
            paints: &self.paints,
            paint_aux: &self.paint_aux,
            contours: &flush.contours,
            triangles: &flush.triangles,
        }
    }
}

impl AtlasUploadData {
    fn new(
        width: u32,
        height: u32,
        flush: &super::logical_frame::PreparedRasterOrderingAtlasFlush,
    ) -> Self {
        let tessellation_height = super::draw::tessellation_texture_height(&flush.spans);
        let mut flush_uniforms = super::analytic_uniforms(width, height, tessellation_height);
        flush_uniforms.max_path_id = u32::try_from(flush.paths.len()).unwrap_or(u32::MAX);
        flush_uniforms.render_target_update_bounds = [0, 0, width as i32, height as i32];
        flush_uniforms.atlas_texture_inverse_size = [
            1.0 / flush.physical_extent[0] as f32,
            1.0 / flush.physical_extent[1] as f32,
        ];
        flush_uniforms.atlas_content_inverse_viewport = [
            2.0 / flush.content_extent[0] as f32,
            -2.0 / flush.content_extent[1] as f32,
        ];
        let mut paths = Vec::with_capacity(flush.paths.len() + 1);
        paths.push(gpu::PathData::zeroed());
        paths.extend_from_slice(&flush.paths);
        let mut paints = Vec::with_capacity(flush.paints.len() + 1);
        paints.push(gpu::PaintData::solid(
            0,
            FillRule::NonZero,
            BlendMode::SrcOver,
        ));
        paints.extend_from_slice(&flush.paints);
        let mut paint_aux = Vec::with_capacity(flush.paint_aux.len() + 1);
        paint_aux.push(gpu::PaintAuxData::zeroed());
        paint_aux.extend_from_slice(&flush.paint_aux);
        Self {
            flush_uniforms,
            paths,
            paints,
            paint_aux,
        }
    }

    fn batch<'a>(
        &'a self,
        flush: &'a super::logical_frame::PreparedRasterOrderingAtlasFlush,
    ) -> context::UploadBatch<'a> {
        context::UploadBatch {
            flush_uniforms: &self.flush_uniforms,
            gradient_spans: &[],
            tessellation_spans: &flush.spans,
            paths: &self.paths,
            paints: &self.paints,
            paint_aux: &self.paint_aux,
            contours: &flush.contours,
            triangles: &flush.triangles,
        }
    }
}

impl GradientUploadData {
    fn new(width: u32, height: u32, draw: &GradientDraw) -> Self {
        let tessellation_height =
            super::draw::tessellation_texture_height(&draw.tessellation.spans);
        let mut flush_uniforms = super::analytic_uniforms(width, height, tessellation_height);
        flush_uniforms.inverse_viewports[0] = -2.0 / draw.gradient_batch.height.max(1) as f32;
        flush_uniforms.max_path_id = 1;
        flush_uniforms.render_target_update_bounds = [0, 0, width as i32, height as i32];
        Self {
            flush_uniforms,
            paths: [gpu::PathData::zeroed(), draw.tessellation.path],
            paints: [
                gpu::PaintData::solid(0, FillRule::NonZero, BlendMode::SrcOver),
                gpu::PaintData::gradient(
                    draw.gradient.paint_type,
                    draw.gradient.texture_y,
                    FillRule::NonZero,
                    BlendMode::SrcOver,
                ),
            ],
            paint_aux: [
                gpu::PaintAuxData::zeroed(),
                super::gradient_paint_aux(None, draw.gradient),
            ],
        }
    }

    fn batch<'a>(&'a self, draw: &'a GradientDraw) -> context::UploadBatch<'a> {
        context::UploadBatch {
            flush_uniforms: &self.flush_uniforms,
            gradient_spans: &draw.gradient_batch.spans,
            tessellation_spans: &draw.tessellation.spans,
            paths: &self.paths,
            paints: &self.paints,
            paint_aux: &self.paint_aux,
            contours: &draw.tessellation.contours,
            triangles: &[],
        }
    }
}

impl Renderer for NativeMetalFrame {
    fn save(&mut self) {
        if self.reject_post_clipped_content_state_mutation() {
            return;
        }
        self.state_stack.push(self.state);
        if self.mode == RenderMode::ClockwiseAtomic {
            self.atomic_logical_state.save();
        }
    }

    fn restore(&mut self) {
        if self.reject_post_clipped_content_state_mutation() {
            return;
        }
        if let Some(state) = self.state_stack.pop() {
            self.state = state;
        }
        if self.mode == RenderMode::ClockwiseAtomic {
            self.atomic_logical_state.restore();
        }
    }

    fn transform(&mut self, transform: Mat2D) {
        if self.reject_post_clipped_content_state_mutation() {
            return;
        }
        self.state.transform = super::multiply(self.state.transform, transform);
        if self.mode == RenderMode::ClockwiseAtomic {
            self.atomic_logical_state.transform(transform);
        }
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        let Some(path) = path.as_any().downcast_ref::<LogicalPath>() else {
            self.unsupported
                .get_or_insert("path from another renderer backend");
            return;
        };
        let Some(paint) = paint.as_any().downcast_ref::<LogicalPaint>() else {
            self.unsupported
                .get_or_insert("paint from another renderer backend");
            return;
        };
        if paint.feather != 0.0 {
            if !path.valid
                || paint.invalid_shader
                || paint.shader.is_some()
                || paint.blend_mode != BlendMode::SrcOver
                || self.state.opacity != 1.0
                || !self.solid_draws.is_empty()
                || !self.gradient_draws.is_empty()
                || !super::draw::feather_requires_atlas(paint.feather, self.state.transform, false)
                || self
                    .atlas_requests
                    .first()
                    .is_some_and(|request| request.paint.style != paint.style)
            {
                self.unsupported.get_or_insert(
                    "native Metal feather atlas requires same-style solid SrcOver draws",
                );
                return;
            }
            let state = super::DrawState {
                transform: self.state.transform,
                opacity: self.state.opacity,
                ..Default::default()
            };
            self.atlas_requests.push(AtlasRequest {
                path: path.clone(),
                paint: paint.clone(),
                state,
            });
            return;
        }
        if !path.valid
            || paint.style != RenderPaintStyle::Fill
            || paint.invalid_shader
            || (self.mode != RenderMode::ClockwiseAtomic && paint.blend_mode != BlendMode::SrcOver)
            || self.state.opacity != 1.0
        {
            self.unsupported
                .get_or_insert("native Metal tracer only supports opaque solid SrcOver fills");
            return;
        }

        if self.mode == RenderMode::ClockwiseAtomic {
            if !self.solid_draws.is_empty()
                || !self.gradient_draws.is_empty()
                || !self.atlas_requests.is_empty()
            {
                self.unsupported
                    .get_or_insert("native Metal atomic tracer supports one path draw");
                return;
            }
            if self.atomic_logical_state.state.clip_stack_height != 0
                && !self.atomic_path_inputs.is_empty()
            {
                self.unsupported.get_or_insert(
                    "native Metal atomic clip tracer supports one clipped content draw",
                );
                return;
            }
            let state = self.atomic_logical_state.state;
            self.atomic_path_inputs.push(AtomicPathInput {
                path: path.clone(),
                paint: paint.clone(),
                state,
            });
            return;
        }

        if let Some(shader @ LogicalShader::Linear { .. }) = paint.shader.as_ref() {
            if !self.solid_draws.is_empty()
                || !self.gradient_draws.is_empty()
                || !is_single_closed_cubic_contour(path)
            {
                self.unsupported.get_or_insert(
                    "native Metal gradient tracer requires one closed cubic fill and no other draws",
                );
                return;
            }
            let Some(gradient_batch) =
                prepare_single_gradient_batch(shader, self.state.opacity, self.state.transform)
            else {
                self.unsupported
                    .get_or_insert("native Metal gradient parameters are invalid");
                return;
            };
            let Some(gradient) = gradient_batch.draw(0) else {
                self.unsupported
                    .get_or_insert("native Metal gradient parameters are invalid");
                return;
            };
            let Some(mut tessellation) =
                super::draw::build_fill_tessellation(&path.raw_path, self.state.transform)
            else {
                self.unsupported
                    .get_or_insert("native Metal gradient path has no tessellatable geometry");
                return;
            };
            for contour in &mut tessellation.contours {
                contour.path_id = 1;
            }
            debug_assert!(matches!(shader, LogicalShader::Linear { .. }));
            self.gradient_draws.push(GradientDraw {
                gradient_batch,
                gradient,
                tessellation,
            });
            return;
        }

        if paint.shader.is_some() {
            self.unsupported
                .get_or_insert("native Metal tracer does not support this shader yet");
            return;
        }
        if paint.color >> 24 != 0xff || !self.gradient_draws.is_empty() {
            self.unsupported
                .get_or_insert("native Metal tracer only supports opaque solid SrcOver fills");
            return;
        }
        let Some(vertices) = solid_triangle_fan(path, self.state.transform) else {
            self.unsupported.get_or_insert(
                "native Metal tracer requires one pixel-aligned axis-aligned rectangle",
            );
            return;
        };
        let Some(vertex_bytes) = vertices.len().checked_mul(std::mem::size_of::<[f32; 2]>()) else {
            self.unsupported
                .get_or_insert("native Metal tracer path exceeds inline vertex limit");
            return;
        };
        if vertex_bytes > INLINE_VERTEX_BYTE_LIMIT {
            self.unsupported
                .get_or_insert("native Metal tracer path exceeds inline vertex limit");
            return;
        }
        self.solid_draws.push(SolidTracerDraw {
            vertices,
            premultiplied_color: premultiplied_color(paint.color, self.state.opacity),
        });
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        if self.mode != RenderMode::ClockwiseAtomic {
            self.unsupported
                .get_or_insert("native Metal tracer does not support clipping yet");
            return;
        }
        let Some(path) = path.as_any().downcast_ref::<LogicalPath>() else {
            self.unsupported
                .get_or_insert("clip path from another renderer backend");
            return;
        };
        let result = if self.atomic_path_inputs.is_empty() {
            self.atomic_logical_state
                .clip_path(self.logical_frame_config(), path)
        } else {
            self.atomic_logical_state
                .clip_rect_after_atomic_content(path)
        };
        if let Err(error) = result {
            self.unsupported.get_or_insert(error);
        }
    }

    fn draw_image(
        &mut self,
        _image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        _blend_mode: BlendMode,
        _opacity: f32,
    ) {
        self.unsupported
            .get_or_insert("native Metal tracer does not support images yet");
    }

    fn draw_image_mesh(
        &mut self,
        _image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        _vertices: Option<&dyn RenderBuffer>,
        _uv_coords: Option<&dyn RenderBuffer>,
        _indices: Option<&dyn RenderBuffer>,
        _vertex_count: u32,
        _index_count: u32,
        _blend_mode: BlendMode,
        _opacity: f32,
    ) {
        self.unsupported
            .get_or_insert("native Metal tracer does not support image meshes yet");
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        if self.reject_post_clipped_content_state_mutation() {
            return;
        }
        self.state.opacity *= opacity;
        if self.mode == RenderMode::ClockwiseAtomic {
            self.atomic_logical_state.state.opacity *= opacity;
        }
    }
}

impl NativeMetalFrame {
    fn reject_post_clipped_content_state_mutation(&mut self) -> bool {
        let must_reject = self.mode == RenderMode::ClockwiseAtomic
            && !self.atomic_path_inputs.is_empty()
            && self.atomic_logical_state.state.clip_stack_height != 0;
        if must_reject {
            self.unsupported.get_or_insert(
                "native Metal atomic clip tracer does not support state mutation after content",
            );
        }
        must_reject
    }

    fn logical_frame_config(&self) -> LogicalFrameConfig {
        let target = self.target.borrow();
        LogicalFrameConfig {
            width: target.width(),
            height: target.height(),
            mode: self.mode,
            max_texture_dimension_2d: self.context.capabilities().max_texture_size,
            msaa_atlas_supports_clip_rect: false,
        }
    }

    pub fn finish(self) -> Result<Vec<u8>, RendererError> {
        Ok(self.finish_for_benchmark()?.pixels)
    }

    pub fn finish_for_benchmark(mut self) -> Result<NativeMetalFrameOutput, RendererError> {
        let encoded = self.encode();
        let (width, height, texture) = match encoded {
            Ok(encoded) => encoded,
            Err(error) => return Err(error),
        };
        let atomic_pipelines = self
            .resource_lease
            .as_ref()
            .and_then(|lease| lease.atomic_path_pipelines.as_ref());
        // Execution inventory is frame-local. A reusable target may retain a
        // color owner allocated by an earlier advanced flush, but a later
        // fixed-function flush does not bind or execute against that plane.
        let atomic_color_plane = atomic_pipelines.is_some() && self.atomic_uses_advanced_blend;
        let atomic_clip_plane = atomic_pipelines.is_some();
        let atomic_coverage_plane = atomic_pipelines.is_some();
        let gradient_texture = self
            .resource_lease
            .as_ref()
            .is_some_and(|lease| lease.gradient.is_some());
        let color_ramp_pipeline = gradient_texture
            && (!self.gradient_draws.is_empty()
                || self
                    .atomic_path_inputs
                    .iter()
                    .any(|draw| draw.paint.shader.is_some()));
        let clipped_path_pipeline_set = self.atomic_uses_clipping
            && atomic_pipelines.is_some_and(|pipelines| {
                pipelines.outer_curve.is_some() && pipelines.interior.is_some()
            });
        let execution_inventory = NativeMetalExecutionInventory {
            mode: self.mode,
            color_ramp_pipeline,
            gradient_texture,
            atomic_color_plane,
            advanced_blend_pipeline: self.atomic_uses_advanced_blend,
            hsl_blend_pipeline: self.atomic_uses_hsl_blend_modes,
            fixed_function_color_output: atomic_pipelines.is_some()
                && !self.atomic_uses_advanced_blend,
            atomic_clip_plane,
            atomic_coverage_plane,
            render_pass_initialize_pipeline: atomic_pipelines.is_some(),
            midpoint_fan_pipeline: atomic_pipelines.is_some(),
            render_pass_resolve_pipeline: atomic_pipelines.is_some(),
            clipped_path_pipeline_set,
            clip_rect_pipeline: self.atomic_uses_clip_rects,
            outer_curve_pipeline: atomic_pipelines
                .is_some_and(|pipelines| pipelines.outer_curve.is_some()),
            interior_triangulation_pipeline: atomic_pipelines
                .is_some_and(|pipelines| pipelines.interior.is_some()),
            atomic_draws: self.atomic_draw_count,
            atomic_draw_groups: self.atomic_draw_group_count,
            atomic_barriers: self.atomic_barrier_count,
            atomic_memory_barriers: self.atomic_memory_barrier_count,
            atomic_render_pass_breaks: self.atomic_render_pass_break_count,
        };
        let mut upload_completion = self.transfer_upload_ownership()?;
        let completion = NativeMetalContext::commit_and_wait(&self.command_buffer);
        if self.collect_work_metrics {
            self.backend_work.queue_submissions = 1;
        }
        let release = upload_completion
            .as_mut()
            .map(|completion| {
                completion.complete().map_err(|error| {
                    RendererError::NativeMetal(format!(
                        "complete native Metal upload-ring ownership: {error:?}"
                    ))
                })
            })
            .transpose();
        completion?;
        release?;

        let row_bytes = usize::try_from(width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or_else(|| RendererError::NativeMetal("readback row size overflow".into()))?;
        let byte_len = row_bytes
            .checked_mul(height as usize)
            .ok_or_else(|| RendererError::NativeMetal("readback size overflow".into()))?;
        let mut pixels = vec![0; byte_len];
        let pointer = NonNull::new(pixels.as_mut_ptr().cast::<c_void>())
            .ok_or_else(|| RendererError::NativeMetal("readback buffer is null".into()))?;
        let region = MTLRegion {
            origin: MTLOrigin { x: 0, y: 0, z: 0 },
            size: MTLSize {
                width: width as usize,
                height: height as usize,
                depth: 1,
            },
        };
        unsafe {
            texture.getBytes_bytesPerRow_fromRegion_mipmapLevel(pointer, row_bytes, region, 0)
        };
        if texture.pixelFormat() == MTLPixelFormat::BGRA8Unorm {
            for pixel in pixels.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }
        }
        Ok(NativeMetalFrameOutput {
            pixels,
            backend_work: self.backend_work,
            execution_inventory,
        })
    }

    fn encode(
        &mut self,
    ) -> Result<(u32, u32, Retained<ProtocolObject<dyn MTLTexture>>), RendererError> {
        if let Some(reason) = self.unsupported {
            return Err(RendererError::Unsupported(reason));
        }
        if !self.atomic_path_inputs.is_empty() {
            let (width, height, pixel_format, texture) = {
                let target = self.target.borrow();
                let texture = target.retained_target_texture().ok_or_else(|| {
                    RendererError::NativeMetal(
                        "native Metal target has no readback texture".to_owned(),
                    )
                })?;
                (
                    target.width(),
                    target.height(),
                    target.pixel_format(),
                    texture,
                )
            };
            let inputs = self
                .atomic_path_inputs
                .iter()
                .map(|input| AtomicPathFlushInput {
                    path: &input.path,
                    paint: &input.paint,
                    state: input.state,
                })
                .collect::<Vec<_>>();
            let config = self.logical_frame_config();
            let flush = if self.atomic_logical_state.state.clip_stack_height == 0 {
                prepare_atomic_path_flush(config, &inputs)
            } else if inputs.len() == 1 {
                prepare_atomic_clipped_path_flush(config, &mut self.atomic_logical_state, inputs[0])
            } else {
                Err("native Metal atomic clip tracer requires one content draw")
            }
            .map_err(RendererError::Unsupported)?;
            if let Some(flush) = flush {
                if flush
                    .draws
                    .iter()
                    .any(|draw| draw.instance_count == 0 && draw.triangle_range.is_empty())
                    || flush.spans.is_empty()
                    || flush.contours.is_empty()
                {
                    return Err(RendererError::Unsupported(
                        "native Metal atomic tracer only supports midpoint-fan path resources",
                    ));
                }
                let tessellation_height = super::draw::tessellation_texture_height(&flush.spans);
                let upload_data =
                    AtomicPathUploadData::new(width, height, self.clear_color, &flush);
                let uses_interior_geometry = flush.draws.iter().any(|draw| {
                    draw.patch_kind == PreparedAtomicPatchKind::OuterCurve
                        || !draw.triangle_range.is_empty()
                });
                let lease = self.context.prepare_atomic_path_resources(
                    flush
                        .gradient_batch
                        .as_ref()
                        .map_or(0, |batch| batch.height as usize),
                    tessellation_height as usize,
                    pixel_format,
                    flush.uses_clipping,
                    flush.uses_clip_rects,
                    uses_interior_geometry,
                    flush.uses_advanced_blend,
                    flush.uses_hsl_blend_modes,
                    upload_data.batch(&flush),
                )?;
                if self.collect_work_metrics {
                    self.backend_work.buffer_upload_calls = lease.upload_calls;
                    self.backend_work.buffer_upload_bytes = lease.upload_bytes;
                }
                self.resource_lease = Some(lease);
                if let Some(gradient_batch) = flush.gradient_batch.as_ref() {
                    encode_color_ramp_pass(
                        &self.context,
                        &self.command_buffer,
                        gradient_batch,
                        self.resource_lease
                            .as_ref()
                            .expect("atomic gradient reservation retained by the frame"),
                    )?;
                }
                encode_atomic_tessellation_pass(
                    &self.context,
                    &self.command_buffer,
                    &flush,
                    self.resource_lease
                        .as_ref()
                        .expect("atomic path resource reservation retained by the frame"),
                )?;
                let atomic_render_passes = {
                    let mut target = self.target.borrow_mut();
                    encode_atomic_main_pass(
                        &self.context,
                        &self.command_buffer,
                        &mut target,
                        width,
                        height,
                        self.clear_color,
                        &flush,
                        self.resource_lease
                            .as_ref()
                            .expect("atomic path resource reservation retained by the frame"),
                    )?
                };
                self.atomic_draw_count = flush.authored_draw_count;
                self.atomic_draw_group_count = flush.draw_group_starts.len();
                let barriers = atomic_barrier_inventory(
                    self.atomic_draw_group_count,
                    self.context.capabilities().atomic_barrier_type,
                );
                self.atomic_barrier_count = barriers.semantic;
                self.atomic_memory_barrier_count = barriers.memory;
                self.atomic_render_pass_break_count = barriers.render_pass_breaks;
                self.atomic_uses_clipping = flush.uses_clipping;
                self.atomic_uses_clip_rects = flush.uses_clip_rects;
                self.atomic_uses_advanced_blend = flush.uses_advanced_blend;
                self.atomic_uses_hsl_blend_modes = flush.uses_hsl_blend_modes;
                if self.collect_work_metrics {
                    let gradient_passes = u64::from(flush.gradient_batch.is_some());
                    self.backend_work.render_passes = 1 + atomic_render_passes + gradient_passes;
                    self.backend_work.gpu_draw_calls =
                        u64::try_from(flush.draws.len()).unwrap_or(u64::MAX) + 3 + gradient_passes;
                    let path_instances = flush
                        .draws
                        .iter()
                        .map(|draw| draw.instance_count as usize)
                        .sum::<usize>();
                    let submitted_path_instances = flush
                        .draws
                        .iter()
                        // Interior triangulation is a non-instanced draw, but
                        // the work metric counts it as one submitted instance.
                        .map(|draw| (draw.instance_count as usize).max(1))
                        .sum::<usize>();
                    self.backend_work.gpu_draw_instances = u64::try_from(
                        flush
                            .gradient_batch
                            .as_ref()
                            .map_or(0, |batch| batch.spans.len())
                            + flush.spans.len()
                            + submitted_path_instances
                            + 2,
                    )
                    .unwrap_or(u64::MAX);
                    self.backend_work.tessellation_spans =
                        u64::try_from(flush.spans.len()).unwrap_or(u64::MAX);
                    self.backend_work.path_patches =
                        u64::try_from(path_instances).unwrap_or(u64::MAX);
                }
                return Ok((width, height, texture));
            }
        }
        let atlas_flush = if self.atlas_requests.is_empty() {
            None
        } else {
            let config = self.logical_frame_config();
            let inputs = self
                .atlas_requests
                .iter()
                .map(|request| super::logical_frame::RasterOrderingAtlasInput {
                    path: &request.path,
                    paint: &request.paint,
                    state: request.state,
                })
                .collect::<Vec<_>>();
            super::logical_frame::prepare_raster_ordering_atlas_flush(config, &inputs)
                .map_err(RendererError::Unsupported)?
        };
        let target = self.target.borrow();
        let width = target.width();
        let height = target.height();
        let texture = target.retained_target_texture().ok_or_else(|| {
            RendererError::NativeMetal("native Metal target has no readback texture".into())
        })?;
        if let Some(flush) = atlas_flush.as_ref() {
            let tessellation_height = super::draw::tessellation_texture_height(&flush.spans);
            let upload_data = AtlasUploadData::new(width, height, flush);
            let is_stroke = flush
                .draws
                .first()
                .map(|draw| draw.is_stroke)
                .ok_or_else(|| RendererError::NativeMetal("atlas flush has no draws".to_owned()))?;
            if flush.draws.iter().any(|draw| draw.is_stroke != is_stroke) {
                return Err(RendererError::Unsupported(
                    "native Metal atlas checkpoint does not mix fill and stroke masks",
                ));
            }
            let lease = self.context.prepare_resources(
                0,
                tessellation_height as usize,
                Some(flush.physical_extent.map(|value| value as usize)),
                Some(is_stroke),
                upload_data.batch(flush),
            )?;
            if self.collect_work_metrics {
                self.backend_work.buffer_upload_calls = lease.upload_calls;
                self.backend_work.buffer_upload_bytes = lease.upload_bytes;
            }
            self.resource_lease = Some(lease);
            encode_atlas_resource_passes(
                &self.context,
                &self.command_buffer,
                flush,
                self.resource_lease
                    .as_ref()
                    .expect("atlas resource reservation retained by the frame"),
            )?;
        } else if let Some(draw) = self.gradient_draws.first() {
            let tessellation_height =
                super::draw::tessellation_texture_height(&draw.tessellation.spans);
            let upload_data = GradientUploadData::new(width, height, draw);
            let lease = self.context.prepare_resources(
                draw.gradient_batch.height as usize,
                tessellation_height as usize,
                None,
                None,
                upload_data.batch(draw),
            )?;
            if self.collect_work_metrics {
                self.backend_work.buffer_upload_calls = lease.upload_calls;
                self.backend_work.buffer_upload_bytes = lease.upload_bytes;
            }
            self.resource_lease = Some(lease);
            encode_gradient_resource_passes(
                &self.context,
                &self.command_buffer,
                draw,
                self.resource_lease
                    .as_ref()
                    .expect("gradient resource reservation retained by the frame"),
            )?;
        }
        let pass = MTLRenderPassDescriptor::renderPassDescriptor();
        pass.setRenderTargetWidth(width as usize);
        pass.setRenderTargetHeight(height as usize);
        let attachment = unsafe { pass.colorAttachments().objectAtIndexedSubscript(0) };
        attachment.setTexture(Some(&texture));
        attachment.setLoadAction(MTLLoadAction::Clear);
        attachment.setStoreAction(MTLStoreAction::Store);
        attachment.setClearColor(clear_color(self.clear_color));
        if !self.gradient_draws.is_empty() || atlas_flush.is_some() {
            configure_raster_order_attachments(&pass, &target)?;
        }
        let encoder = self
            .command_buffer
            .renderCommandEncoderWithDescriptor(&pass)
            .ok_or_else(|| {
                RendererError::NativeMetal("failed to create render command encoder".into())
            })?;
        if let Some(flush) = atlas_flush.as_ref() {
            encode_atlas_final_draw(
                &self.context,
                &encoder,
                target.pixel_format(),
                width,
                height,
                flush,
                self.resource_lease
                    .as_ref()
                    .expect("atlas resource passes prepared a lease"),
            )?;
        } else if let Some(draw) = self.gradient_draws.first() {
            encode_gradient_final_draw(
                &self.context,
                &encoder,
                target.pixel_format(),
                width,
                height,
                draw,
                self.resource_lease
                    .as_ref()
                    .expect("gradient resource passes prepared a lease"),
            )?;
        } else if !self.solid_draws.is_empty() {
            encoder.setRenderPipelineState(self.context.solid_pipeline(target.pixel_format())?);
            let viewport = [width as f32, height as f32];
            let viewport_pointer = NonNull::from(&viewport).cast::<c_void>();
            unsafe {
                encoder.setVertexBytes_length_atIndex(
                    viewport_pointer,
                    std::mem::size_of_val(&viewport),
                    1,
                );
            }
            for draw in &self.solid_draws {
                let vertex_pointer =
                    NonNull::new(draw.vertices.as_ptr().cast_mut().cast::<c_void>())
                        .expect("solid draw has triangle vertices");
                let color_pointer = NonNull::from(&draw.premultiplied_color).cast::<c_void>();
                unsafe {
                    encoder.setVertexBytes_length_atIndex(
                        vertex_pointer,
                        std::mem::size_of_val(draw.vertices.as_slice()),
                        0,
                    );
                    encoder.setFragmentBytes_length_atIndex(
                        color_pointer,
                        std::mem::size_of_val(&draw.premultiplied_color),
                        0,
                    );
                    encoder.drawPrimitives_vertexStart_vertexCount(
                        MTLPrimitiveType::Triangle,
                        0,
                        draw.vertices.len(),
                    );
                }
            }
        }
        encoder.endEncoding();
        if self.collect_work_metrics {
            self.backend_work.render_passes =
                if self.gradient_draws.is_empty() && atlas_flush.is_none() {
                    1
                } else {
                    3
                };
            if let Some(flush) = atlas_flush.as_ref() {
                self.backend_work.gpu_draw_calls = u64::try_from(
                    flush
                        .fill_batches
                        .len()
                        .saturating_add(flush.stroke_batches.len())
                        .saturating_add(2),
                )
                .unwrap_or(u64::MAX);
                self.backend_work.gpu_draw_instances = u64::try_from(
                    // `gpu_draw_instances` counts instances submitted per
                    // draw, not vertices within an instance. The atlas blit
                    // is one non-instanced draw of six vertices.
                    flush.spans.len()
                        + flush
                            .fill_batches
                            .iter()
                            .chain(&flush.stroke_batches)
                            .map(|batch| batch.patch_count as usize)
                            .sum::<usize>()
                        + 1,
                )
                .unwrap_or(u64::MAX);
                self.backend_work.tessellation_spans =
                    u64::try_from(flush.spans.len()).unwrap_or(u64::MAX);
                self.backend_work.path_patches = flush
                    .fill_batches
                    .iter()
                    .chain(&flush.stroke_batches)
                    .map(|batch| u64::from(batch.patch_count))
                    .sum();
            } else if let Some(draw) = self.gradient_draws.first() {
                self.backend_work.gpu_draw_calls = 3;
                self.backend_work.gpu_draw_instances = u64::try_from(
                    draw.gradient_batch.spans.len()
                        + draw.tessellation.spans.len()
                        + draw.tessellation.instance_count as usize,
                )
                .unwrap_or(u64::MAX);
                self.backend_work.tessellation_spans =
                    u64::try_from(draw.tessellation.spans.len()).unwrap_or(u64::MAX);
                self.backend_work.path_patches = u64::from(draw.tessellation.instance_count);
            } else {
                self.backend_work.gpu_draw_calls =
                    u64::try_from(self.solid_draws.len()).unwrap_or(u64::MAX);
                self.backend_work.gpu_draw_instances = self.backend_work.gpu_draw_calls;
            }
        }
        Ok((width, height, texture))
    }

    fn transfer_upload_ownership(
        &mut self,
    ) -> Result<Option<buffer_ring_coordinator::BufferRingCompletion>, RendererError> {
        self.resource_lease
            .as_mut()
            .map(context::PreparedResourceLease::transfer_to_completion)
            .transpose()
    }
}

fn encode_atomic_tessellation_pass(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    flush: &PreparedAtomicPathFlush,
    lease: &context::PreparedResourceLease,
) -> Result<(), RendererError> {
    let tessellation_height = super::draw::tessellation_texture_height(&flush.spans);
    let pass = MTLRenderPassDescriptor::renderPassDescriptor();
    pass.setRenderTargetWidth(tessellation_resource::TESSELLATION_TEXTURE_WIDTH);
    pass.setRenderTargetHeight(tessellation_height as usize);
    // SAFETY: Metal render-pass descriptors always expose color attachment
    // slot zero; this pass declares exactly that one tessellation target.
    let attachment = unsafe { pass.colorAttachments().objectAtIndexedSubscript(0) };
    attachment.setTexture(Some(&lease.tessellation));
    attachment.setLoadAction(MTLLoadAction::DontCare);
    attachment.setStoreAction(MTLStoreAction::Store);
    let encoder = command_buffer
        .renderCommandEncoderWithDescriptor(&pass)
        .ok_or_else(|| {
            RendererError::NativeMetal(
                "failed to create atomic path tessellation encoder".to_owned(),
            )
        })?;
    encoder.setViewport(MTLViewport {
        originX: 0.0,
        originY: 0.0,
        width: tessellation_resource::TESSELLATION_TEXTURE_WIDTH as f64,
        height: tessellation_height as f64,
        znear: 0.0,
        zfar: 1.0,
    });
    encoder.setRenderPipelineState(context.tessellate_pipeline());
    // SAFETY: slot 9 is the generated `gaussianIntegralTexture` vertex binding
    // and the context retains the texture through command-buffer completion.
    unsafe {
        encoder.setVertexTexture_atIndex(Some(context.gaussian_integral_texture()), 9);
    }
    bind_vertex_buffer(&encoder, &lease.flush_uniforms, 3);
    bind_vertex_buffer(&encoder, &lease.tessellation_spans, 0);
    bind_vertex_buffer(&encoder, &lease.paths, 5);
    bind_vertex_buffer(&encoder, &lease.contours, 8);
    encoder.setCullMode(MTLCullMode::Back);
    // SAFETY: the shared index buffer contains the complete generated
    // K_TESS_SPAN_INDICES table, and the instance count is the number of
    // initialized canonical tessellation-span records retained by the lease.
    unsafe {
        encoder
            .drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount(
                MTLPrimitiveType::Triangle,
                tessellation_resource::K_TESS_SPAN_INDICES.len(),
                MTLIndexType::UInt16,
                context.tess_span_index_buffer(),
                0,
                flush.spans.len(),
            );
    }
    encoder.endEncoding();
    Ok(())
}

const ATOMIC_COLOR_BUFFER_INDEX: usize = 16;
const ATOMIC_CLIP_BUFFER_INDEX: usize = 17;
const ATOMIC_COVERAGE_BUFFER_INDEX: usize = 19;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AtomicBarrierInventory {
    semantic: usize,
    memory: usize,
    render_pass_breaks: usize,
}

fn atomic_barrier_inventory(
    draw_group_count: usize,
    barrier_type: AtomicBarrierType,
) -> AtomicBarrierInventory {
    let semantic = draw_group_count.saturating_add(1);
    AtomicBarrierInventory {
        semantic,
        memory: usize::from(barrier_type == AtomicBarrierType::MemoryBarrier) * semantic,
        render_pass_breaks: usize::from(barrier_type == AtomicBarrierType::RenderPassBreak)
            * semantic,
    }
}

fn make_atomic_main_encoder(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    pass: &MTLRenderPassDescriptor,
    target: &mut RenderTargetMetal,
    width: u32,
    height: u32,
    lease: &context::PreparedResourceLease,
    uses_color_plane: bool,
) -> Result<Retained<ProtocolObject<dyn MTLRenderCommandEncoder>>, RendererError> {
    let encoder = command_buffer
        .renderCommandEncoderWithDescriptor(pass)
        .ok_or_else(|| {
            RendererError::NativeMetal("failed to create atomic path encoder".to_owned())
        })?;
    encoder.setViewport(MTLViewport {
        originX: 0.0,
        originY: 0.0,
        width: width as f64,
        height: height as f64,
        znear: 0.0,
        zfar: 1.0,
    });
    bind_vertex_buffer(&encoder, &lease.flush_uniforms, 3);
    bind_fragment_buffer(&encoder, &lease.flush_uniforms, 3);
    bind_vertex_buffer(&encoder, &lease.paths, 5);
    bind_fragment_buffer(&encoder, &lease.paints, 6);
    bind_fragment_buffer(&encoder, &lease.paint_aux, 7);
    bind_vertex_buffer(&encoder, &lease.contours, 8);
    // SAFETY: 7, 8, 9, 11, 16, 17, and 19 are generated Metal binding indices
    // for tessellation, gradient ramp, Gaussian LUT, linear-clamp sampler,
    // color atomics, clip atomics, and coverage atomics. Every object/buffer is
    // retained by the context, resource lease, or render target until
    // command-buffer completion.
    unsafe {
        encoder.setVertexTexture_atIndex(Some(&lease.tessellation), 7);
        if let Some(gradient) = lease.gradient.as_deref() {
            encoder.setFragmentTexture_atIndex(Some(gradient), 8);
        }
        encoder.setVertexTexture_atIndex(Some(context.gaussian_integral_texture()), 9);
        encoder.setFragmentTexture_atIndex(Some(context.gaussian_integral_texture()), 9);
        encoder.setFragmentSamplerState_atIndex(
            Some(context.image_sampler(ImageSampler::LINEAR_CLAMP)),
            11,
        );
        if uses_color_plane {
            encoder.setFragmentBuffer_offset_atIndex(
                Some(target.color_atomic_buffer()?),
                0,
                ATOMIC_COLOR_BUFFER_INDEX,
            );
        }
        encoder.setFragmentBuffer_offset_atIndex(
            Some(target.clip_atomic_buffer()?),
            0,
            ATOMIC_CLIP_BUFFER_INDEX,
        );
        encoder.setFragmentBuffer_offset_atIndex(
            Some(target.coverage_atomic_buffer()?),
            0,
            ATOMIC_COVERAGE_BUFFER_INDEX,
        );
    }
    Ok(encoder)
}

fn apply_atomic_barrier(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    pass: &MTLRenderPassDescriptor,
    target: &mut RenderTargetMetal,
    width: u32,
    height: u32,
    lease: &context::PreparedResourceLease,
    uses_color_plane: bool,
    encoder: Retained<ProtocolObject<dyn MTLRenderCommandEncoder>>,
    render_passes: &mut u64,
) -> Result<Retained<ProtocolObject<dyn MTLRenderCommandEncoder>>, RendererError> {
    match context.capabilities().atomic_barrier_type {
        AtomicBarrierType::RasterOrderGroup => Ok(encoder),
        AtomicBarrierType::MemoryBarrier => {
            encoder.memoryBarrierWithScope_afterStages_beforeStages(
                MTLBarrierScope::Buffers | MTLBarrierScope::RenderTargets,
                MTLRenderStages::Fragment,
                MTLRenderStages::Fragment,
            );
            Ok(encoder)
        }
        AtomicBarrierType::RenderPassBreak => {
            encoder.endEncoding();
            // SAFETY: the atomic main pass declares color attachment slot zero;
            // only its load action changes before the replacement encoder.
            let attachment = unsafe { pass.colorAttachments().objectAtIndexedSubscript(0) };
            attachment.setLoadAction(MTLLoadAction::Load);
            *render_passes = render_passes.saturating_add(1);
            make_atomic_main_encoder(
                context,
                command_buffer,
                pass,
                target,
                width,
                height,
                lease,
                uses_color_plane,
            )
        }
    }
}

fn encode_atomic_main_pass(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    target: &mut RenderTargetMetal,
    width: u32,
    height: u32,
    clear: u32,
    flush: &PreparedAtomicPathFlush,
    lease: &context::PreparedResourceLease,
) -> Result<u64, RendererError> {
    target.prepare_atomic_planes(flush.uses_advanced_blend)?;
    let pipelines = lease
        .atomic_path_pipelines
        .as_ref()
        .ok_or_else(|| RendererError::NativeMetal("atomic path pipelines are absent".to_owned()))?;
    let texture = target.target_texture().ok_or_else(|| {
        RendererError::NativeMetal("native Metal target has no attached texture".to_owned())
    })?;
    let pass = MTLRenderPassDescriptor::renderPassDescriptor();
    pass.setRenderTargetWidth(width as usize);
    pass.setRenderTargetHeight(height as usize);
    // SAFETY: Metal render-pass descriptors always expose color attachment
    // slot zero; this pass declares exactly the retained target texture there.
    let attachment = unsafe { pass.colorAttachments().objectAtIndexedSubscript(0) };
    attachment.setTexture(Some(texture));
    attachment.setLoadAction(MTLLoadAction::Clear);
    attachment.setStoreAction(MTLStoreAction::Store);
    attachment.setClearColor(clear_color(clear));

    let mut render_passes = 1;
    let mut encoder = make_atomic_main_encoder(
        context,
        command_buffer,
        &pass,
        target,
        width,
        height,
        lease,
        flush.uses_advanced_blend,
    )?;
    let full_scissor = MTLScissorRect {
        x: 0,
        y: 0,
        width: width as usize,
        height: height as usize,
    };
    encoder.setScissorRect(full_scissor);
    encoder.setRenderPipelineState(&pipelines.initialize);
    // SAFETY: the generated initialize shader consumes the four implicit
    // triangle-strip vertices and requires no caller-provided vertex buffer.
    unsafe {
        encoder.drawPrimitives_vertexStart_vertexCount(MTLPrimitiveType::TriangleStrip, 0, 4);
    }
    encoder = apply_atomic_barrier(
        context,
        command_buffer,
        &pass,
        target,
        width,
        height,
        lease,
        flush.uses_advanced_blend,
        encoder,
        &mut render_passes,
    )?;

    let mut next_group_start = flush.draw_group_starts.iter().copied().skip(1).peekable();
    for (draw_index, draw) in flush.draws.iter().enumerate() {
        if next_group_start.peek().copied() == Some(draw_index) {
            encoder = apply_atomic_barrier(
                context,
                command_buffer,
                &pass,
                target,
                width,
                height,
                lease,
                flush.uses_advanced_blend,
                encoder,
                &mut render_passes,
            )?;
            next_group_start.next();
        }
        // A semantic barrier may replace the render encoder. Reapply the
        // desired scissor even when it matches the preceding logical draw.
        if let Some([left, top, right, bottom]) = draw.scissor {
            encoder.setScissorRect(MTLScissorRect {
                x: left as usize,
                y: top as usize,
                width: usize::from(right - left),
                height: usize::from(bottom - top),
            });
        } else {
            encoder.setScissorRect(full_scissor);
        }
        if draw.instance_count != 0 {
            let (pipeline, index_count, index_offset) = match draw.patch_kind {
                PreparedAtomicPatchKind::Midpoint => {
                    (&pipelines.midpoint, gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT, 0)
                }
                PreparedAtomicPatchKind::OuterCurve => (
                    pipelines.outer_curve.as_ref().ok_or_else(|| {
                        RendererError::NativeMetal(
                            "atomic outer-curve pipeline is absent".to_owned(),
                        )
                    })?,
                    gpu::OUTER_CURVE_PATCH_INDEX_COUNT,
                    (gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT
                        + gpu::MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT)
                        * std::mem::size_of::<u16>(),
                ),
            };
            encoder.setRenderPipelineState(pipeline);
            bind_vertex_buffer(&encoder, context.path_patch_vertex_buffer(), 0);
            encoder.setCullMode(MTLCullMode::Back);
            set_vertex_bytes(&encoder, &draw.base_instance, 4)?;
            // SAFETY: the retained patch index/vertex buffers contain the
            // generated midpoint and outer-curve ranges; this command names
            // only its initialized relocated instances.
            unsafe {
                encoder.drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount(
                    MTLPrimitiveType::Triangle,
                    index_count,
                    MTLIndexType::UInt16,
                    context.path_patch_index_buffer(),
                    index_offset,
                    draw.instance_count as usize,
                );
            }
        } else {
            let pipeline = pipelines.interior.as_ref().ok_or_else(|| {
                RendererError::NativeMetal("atomic interior pipeline is absent".to_owned())
            })?;
            let triangles = lease.triangles.as_deref().ok_or_else(|| {
                RendererError::NativeMetal("atomic interior triangle upload is absent".to_owned())
            })?;
            encoder.setRenderPipelineState(pipeline);
            bind_vertex_buffer(&encoder, triangles, 0);
            encoder.setCullMode(MTLCullMode::None);
            // SAFETY: the typed logical writer initialized every retained
            // triangle in this command's exact range.
            unsafe {
                encoder.drawPrimitives_vertexStart_vertexCount(
                    MTLPrimitiveType::Triangle,
                    draw.triangle_range.start,
                    draw.triangle_range.len(),
                );
            }
        }
    }
    // C++ attaches `plsAtomicPreResolve` to the resolve batch. Apply it once
    // after the final group, independent of the number of authored draws in
    // that group.
    encoder = apply_atomic_barrier(
        context,
        command_buffer,
        &pass,
        target,
        width,
        height,
        lease,
        flush.uses_advanced_blend,
        encoder,
        &mut render_passes,
    )?;

    encoder.setScissorRect(full_scissor);

    encoder.setRenderPipelineState(&pipelines.resolve);
    // SAFETY: the generated resolve shader consumes the four implicit
    // triangle-strip vertices and requires no caller-provided vertex buffer.
    unsafe {
        encoder.drawPrimitives_vertexStart_vertexCount(MTLPrimitiveType::TriangleStrip, 0, 4);
    }
    encoder.endEncoding();
    Ok(render_passes)
}

fn is_single_closed_cubic_contour(path: &LogicalPath) -> bool {
    let verbs = path.raw_path.verbs();
    verbs.first() == Some(&PathVerb::Move)
        && verbs.last() == Some(&PathVerb::Close)
        && verbs.iter().filter(|verb| **verb == PathVerb::Move).count() == 1
        && verbs.iter().any(|verb| *verb == PathVerb::Cubic)
        && verbs.iter().all(|verb| {
            matches!(
                verb,
                PathVerb::Move | PathVerb::Line | PathVerb::Cubic | PathVerb::Close
            )
        })
}

fn configure_raster_order_attachments(
    pass: &MTLRenderPassDescriptor,
    target: &RenderTargetMetal,
) -> Result<(), RendererError> {
    let textures = [
        target.clip_memoryless_texture(),
        target.scratch_color_memoryless_texture(),
        target.coverage_memoryless_texture(),
    ];
    if textures.iter().any(|texture| texture.is_none()) {
        return Err(RendererError::Unsupported(
            "native Metal gradient tracer requires raster-order attachments",
        ));
    }
    for (index, texture) in textures.into_iter().enumerate() {
        // SAFETY: `index + 1` is limited to Metal color attachments 1...3,
        // matching upstream's clip, scratch-color, and coverage planes; the
        // render-pass descriptor retains the selected attachment descriptor.
        let attachment = unsafe { pass.colorAttachments().objectAtIndexedSubscript(index + 1) };
        attachment.setTexture(texture);
        attachment.setLoadAction(if index == 1 {
            MTLLoadAction::DontCare
        } else {
            MTLLoadAction::Clear
        });
        attachment.setStoreAction(MTLStoreAction::DontCare);
        if index != 1 {
            attachment.setClearColor(MTLClearColor {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 0.0,
            });
        }
    }
    Ok(())
}

fn encode_gradient_resource_passes(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    draw: &GradientDraw,
    lease: &context::PreparedResourceLease,
) -> Result<(), RendererError> {
    let tessellation_height = super::draw::tessellation_texture_height(&draw.tessellation.spans);

    encode_color_ramp_pass(context, command_buffer, &draw.gradient_batch, lease)?;

    let tessellation_pass = MTLRenderPassDescriptor::renderPassDescriptor();
    tessellation_pass.setRenderTargetWidth(tessellation_resource::TESSELLATION_TEXTURE_WIDTH);
    tessellation_pass.setRenderTargetHeight(tessellation_height as usize);
    // SAFETY: every Metal render-pass descriptor exposes color attachment zero,
    // which upstream uses for the RGBA32Uint tessellation texture.
    let tessellation_attachment = unsafe {
        tessellation_pass
            .colorAttachments()
            .objectAtIndexedSubscript(0)
    };
    tessellation_attachment.setTexture(Some(&lease.tessellation));
    tessellation_attachment.setLoadAction(MTLLoadAction::DontCare);
    tessellation_attachment.setStoreAction(MTLStoreAction::Store);
    let tessellation_encoder = command_buffer
        .renderCommandEncoderWithDescriptor(&tessellation_pass)
        .ok_or_else(|| {
            RendererError::NativeMetal("failed to create tessellation encoder".to_owned())
        })?;
    tessellation_encoder.setViewport(MTLViewport {
        originX: 0.0,
        originY: 0.0,
        width: tessellation_resource::TESSELLATION_TEXTURE_WIDTH as f64,
        height: tessellation_height as f64,
        znear: 0.0,
        zfar: 1.0,
    });
    tessellation_encoder.setRenderPipelineState(context.tessellate_pipeline());
    // SAFETY: texture slot 9 is the pinned tessellation-shader Gaussian-table
    // ABI, and the context-retained texture outlives command-buffer completion.
    unsafe {
        tessellation_encoder.setVertexTexture_atIndex(Some(context.gaussian_integral_texture()), 9);
    }
    bind_vertex_buffer(&tessellation_encoder, &lease.flush_uniforms, 3);
    bind_vertex_buffer(&tessellation_encoder, &lease.tessellation_spans, 0);
    bind_vertex_buffer(&tessellation_encoder, &lease.paths, 5);
    bind_vertex_buffer(&tessellation_encoder, &lease.contours, 8);
    tessellation_encoder.setCullMode(MTLCullMode::Back);
    // SAFETY: the retained index buffer contains exactly
    // `K_TESS_SPAN_INDICES.len()` UInt16 indices from offset zero; all bound Pod
    // inputs remain retained through completion and the instance count equals
    // the span count.
    unsafe {
        tessellation_encoder
            .drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount(
                MTLPrimitiveType::Triangle,
                tessellation_resource::K_TESS_SPAN_INDICES.len(),
                MTLIndexType::UInt16,
                context.tess_span_index_buffer(),
                0,
                draw.tessellation.spans.len(),
            );
    }
    tessellation_encoder.endEncoding();
    Ok(())
}

fn encode_color_ramp_pass(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    gradient_batch: &super::logical_frame::GradientBatch,
    lease: &context::PreparedResourceLease,
) -> Result<(), RendererError> {
    let gradient_pass = MTLRenderPassDescriptor::renderPassDescriptor();
    gradient_pass.setRenderTargetWidth(gradient_resource::GRADIENT_TEXTURE_WIDTH);
    gradient_pass.setRenderTargetHeight(gradient_batch.height as usize);
    // SAFETY: every Metal render-pass descriptor exposes color attachment zero,
    // which upstream uses for the RGBA8 color-ramp texture.
    let gradient_attachment =
        unsafe { gradient_pass.colorAttachments().objectAtIndexedSubscript(0) };
    let gradient_texture = lease.gradient.as_deref().ok_or_else(|| {
        RendererError::NativeMetal("gradient resource texture is absent".to_owned())
    })?;
    gradient_attachment.setTexture(Some(gradient_texture));
    gradient_attachment.setLoadAction(MTLLoadAction::DontCare);
    gradient_attachment.setStoreAction(MTLStoreAction::Store);
    let gradient_encoder = command_buffer
        .renderCommandEncoderWithDescriptor(&gradient_pass)
        .ok_or_else(|| {
            RendererError::NativeMetal("failed to create color-ramp encoder".to_owned())
        })?;
    gradient_encoder.setViewport(MTLViewport {
        originX: 0.0,
        originY: 0.0,
        width: gradient_resource::GRADIENT_TEXTURE_WIDTH as f64,
        height: gradient_batch.height as f64,
        znear: 0.0,
        zfar: 1.0,
    });
    gradient_encoder.setRenderPipelineState(context.color_ramp_pipeline());
    bind_vertex_buffer(&gradient_encoder, &lease.flush_uniforms, 3);
    bind_vertex_buffer(
        &gradient_encoder,
        lease.gradient_spans.as_deref().ok_or_else(|| {
            RendererError::NativeMetal("gradient span upload is absent".to_owned())
        })?,
        0,
    );
    gradient_encoder.setCullMode(MTLCullMode::Back);
    // SAFETY: the compiled color-ramp ABI consumes exactly eight vertices per
    // span; the retained shared upload buffer contains every initialized Pod
    // span, and the instance count is the same slice length.
    unsafe {
        gradient_encoder.drawPrimitives_vertexStart_vertexCount_instanceCount(
            MTLPrimitiveType::TriangleStrip,
            0,
            gpu::GRAD_SPAN_TRI_STRIP_VERTEX_COUNT,
            gradient_batch.spans.len(),
        );
    }
    gradient_encoder.endEncoding();
    Ok(())
}

fn encode_atlas_resource_passes(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    flush: &super::logical_frame::PreparedRasterOrderingAtlasFlush,
    lease: &context::PreparedResourceLease,
) -> Result<(), RendererError> {
    let tessellation_height = super::draw::tessellation_texture_height(&flush.spans);
    let tessellation_pass = MTLRenderPassDescriptor::renderPassDescriptor();
    tessellation_pass.setRenderTargetWidth(tessellation_resource::TESSELLATION_TEXTURE_WIDTH);
    tessellation_pass.setRenderTargetHeight(tessellation_height as usize);
    // SAFETY: every Metal render-pass descriptor owns attachment zero; this is
    // the pinned RGBA32Uint tessellation target and the lease retains it.
    let tessellation_attachment = unsafe {
        tessellation_pass
            .colorAttachments()
            .objectAtIndexedSubscript(0)
    };
    tessellation_attachment.setTexture(Some(&lease.tessellation));
    tessellation_attachment.setLoadAction(MTLLoadAction::DontCare);
    tessellation_attachment.setStoreAction(MTLStoreAction::Store);
    let tessellation_encoder = command_buffer
        .renderCommandEncoderWithDescriptor(&tessellation_pass)
        .ok_or_else(|| {
            RendererError::NativeMetal("failed to create atlas tessellation encoder".to_owned())
        })?;
    tessellation_encoder.setViewport(MTLViewport {
        originX: 0.0,
        originY: 0.0,
        width: tessellation_resource::TESSELLATION_TEXTURE_WIDTH as f64,
        height: tessellation_height as f64,
        znear: 0.0,
        zfar: 1.0,
    });
    tessellation_encoder.setRenderPipelineState(context.tessellate_pipeline());
    // SAFETY: texture slot 9 is the exact generated tessellation ABI and the
    // context retains the Gaussian table through command-buffer completion.
    unsafe {
        tessellation_encoder.setVertexTexture_atIndex(Some(context.gaussian_integral_texture()), 9);
    }
    bind_vertex_buffer(&tessellation_encoder, &lease.flush_uniforms, 3);
    bind_vertex_buffer(&tessellation_encoder, &lease.tessellation_spans, 0);
    bind_vertex_buffer(&tessellation_encoder, &lease.paths, 5);
    bind_vertex_buffer(&tessellation_encoder, &lease.contours, 8);
    tessellation_encoder.setCullMode(MTLCullMode::Back);
    // SAFETY: the retained UInt16 index buffer contains the complete static
    // tessellation index table; every per-frame input is held by the lease.
    unsafe {
        tessellation_encoder
            .drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount(
                MTLPrimitiveType::Triangle,
                tessellation_resource::K_TESS_SPAN_INDICES.len(),
                MTLIndexType::UInt16,
                context.tess_span_index_buffer(),
                0,
                flush.spans.len(),
            );
    }
    tessellation_encoder.endEncoding();

    let atlas_texture = lease
        .feather_atlas
        .as_deref()
        .ok_or_else(|| RendererError::NativeMetal("feather atlas texture is absent".to_owned()))?;
    let atlas_pass = MTLRenderPassDescriptor::renderPassDescriptor();
    atlas_pass.setRenderTargetWidth(flush.content_extent[0] as usize);
    atlas_pass.setRenderTargetHeight(flush.content_extent[1] as usize);
    // SAFETY: every Metal render-pass descriptor owns attachment zero; this is
    // the pinned private R16Float feather-atlas target retained by the lease.
    let atlas_attachment = unsafe { atlas_pass.colorAttachments().objectAtIndexedSubscript(0) };
    atlas_attachment.setTexture(Some(atlas_texture));
    atlas_attachment.setLoadAction(MTLLoadAction::Clear);
    atlas_attachment.setStoreAction(MTLStoreAction::Store);
    atlas_attachment.setClearColor(MTLClearColor {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
        alpha: 0.0,
    });
    let atlas_encoder = command_buffer
        .renderCommandEncoderWithDescriptor(&atlas_pass)
        .ok_or_else(|| {
            RendererError::NativeMetal("failed to create feather atlas encoder".to_owned())
        })?;
    atlas_encoder.setViewport(MTLViewport {
        originX: 0.0,
        originY: 0.0,
        width: flush.content_extent[0] as f64,
        height: flush.content_extent[1] as f64,
        znear: 0.0,
        zfar: 1.0,
    });
    atlas_encoder.setRenderPipelineState(lease.feather_atlas_pipeline.as_deref().ok_or_else(
        || RendererError::NativeMetal("feather atlas pipeline is absent".to_owned()),
    )?);
    bind_vertex_buffer(&atlas_encoder, &lease.flush_uniforms, 3);
    bind_fragment_buffer(&atlas_encoder, &lease.flush_uniforms, 3);
    bind_vertex_buffer(&atlas_encoder, &lease.paths, 5);
    bind_vertex_buffer(&atlas_encoder, &lease.paints, 6);
    bind_vertex_buffer(&atlas_encoder, &lease.paint_aux, 7);
    bind_vertex_buffer(&atlas_encoder, &lease.contours, 8);
    // SAFETY: the generated atlas ABI fixes tessellation/Gaussian/gradient at
    // slots 7/9/8/9. A solid atlas paint intentionally binds no gradient
    // texture, matching upstream's nullable zero-height gradient resource.
    unsafe {
        atlas_encoder.setVertexTexture_atIndex(Some(&lease.tessellation), 7);
        atlas_encoder.setVertexTexture_atIndex(Some(context.gaussian_integral_texture()), 9);
        atlas_encoder.setFragmentTexture_atIndex(lease.gradient.as_deref(), 8);
        atlas_encoder.setFragmentTexture_atIndex(Some(context.gaussian_integral_texture()), 9);
        atlas_encoder.setVertexBuffer_offset_atIndex(
            Some(context.path_patch_vertex_buffer()),
            0,
            0,
        );
    }
    let is_stroke = flush
        .draws
        .first()
        .map(|draw| draw.is_stroke)
        .ok_or_else(|| RendererError::NativeMetal("atlas flush has no draws".to_owned()))?;
    atlas_encoder.setCullMode(if is_stroke {
        MTLCullMode::Back
    } else {
        MTLCullMode::None
    });
    let (index_count, index_offset) = if is_stroke {
        (gpu::MIDPOINT_FAN_PATCH_BORDER_INDEX_COUNT, 0)
    } else {
        (
            gpu::MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT,
            gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT * std::mem::size_of::<u16>(),
        )
    };
    let batches = if is_stroke {
        &flush.stroke_batches
    } else {
        &flush.fill_batches
    };
    for batch in batches {
        let [left, top, right, bottom] = batch.scissor;
        atlas_encoder.setScissorRect(MTLScissorRect {
            x: left as usize,
            y: top as usize,
            width: usize::from(right - left),
            height: usize::from(bottom - top),
        });
        set_vertex_bytes(&atlas_encoder, &batch.base_patch, 4)?;
        // SAFETY: the static patch index buffer contains both exact index
        // ranges; every canonical batch supplies a validated base patch and
        // patch count into the flush-wide tessellation texture.
        unsafe {
            atlas_encoder.drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount(
                    MTLPrimitiveType::Triangle,
                    index_count,
                    MTLIndexType::UInt16,
                    context.path_patch_index_buffer(),
                    index_offset,
                    batch.patch_count as usize,
                );
        }
    }
    atlas_encoder.endEncoding();
    Ok(())
}

fn encode_atlas_final_draw(
    context: &NativeMetalContext,
    encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
    pixel_format: MTLPixelFormat,
    width: u32,
    height: u32,
    flush: &super::logical_frame::PreparedRasterOrderingAtlasFlush,
    lease: &context::PreparedResourceLease,
) -> Result<(), RendererError> {
    encoder.setViewport(MTLViewport {
        originX: 0.0,
        originY: 0.0,
        width: width as f64,
        height: height as f64,
        znear: 0.0,
        zfar: 1.0,
    });
    let atlas_blit_pipeline = context.atlas_blit_pipeline(pixel_format)?;
    encoder.setRenderPipelineState(&atlas_blit_pipeline);
    let triangles = lease.triangles.as_deref().ok_or_else(|| {
        RendererError::NativeMetal("atlas blit triangle upload is absent".to_owned())
    })?;
    bind_vertex_buffer(encoder, triangles, 0);
    bind_vertex_buffer(encoder, &lease.flush_uniforms, 3);
    bind_fragment_buffer(encoder, &lease.flush_uniforms, 3);
    bind_vertex_buffer(encoder, &lease.paths, 5);
    bind_vertex_buffer(encoder, &lease.paints, 6);
    bind_vertex_buffer(encoder, &lease.paint_aux, 7);
    bind_vertex_buffer(encoder, &lease.contours, 8);
    let atlas_texture = lease.feather_atlas.as_deref().ok_or_else(|| {
        RendererError::NativeMetal("feather atlas texture is absent for final blit".to_owned())
    })?;
    // SAFETY: slots 7-11 are the pinned compatible AtlasBlit ubershader ABI;
    // all objects are retained by the context or resource lease until the
    // synchronous command-buffer completion point.
    unsafe {
        encoder.setVertexTexture_atIndex(Some(&lease.tessellation), 7);
        encoder.setVertexTexture_atIndex(Some(context.gaussian_integral_texture()), 9);
        encoder.setFragmentTexture_atIndex(lease.gradient.as_deref(), 8);
        encoder.setFragmentTexture_atIndex(Some(context.gaussian_integral_texture()), 9);
        encoder.setFragmentTexture_atIndex(Some(atlas_texture), 10);
        encoder.setFragmentSamplerState_atIndex(
            Some(context.image_sampler(ImageSampler::LINEAR_CLAMP)),
            11,
        );
    }
    encoder.setCullMode(MTLCullMode::Back);
    // SAFETY: the canonical typed writer initialized every TriangleVertex in
    // `flush.triangles`, and the resource lease retains that upload buffer.
    unsafe {
        encoder.drawPrimitives_vertexStart_vertexCount(
            MTLPrimitiveType::Triangle,
            0,
            flush.triangles.len(),
        );
    }
    Ok(())
}

fn encode_gradient_final_draw(
    context: &NativeMetalContext,
    encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
    pixel_format: MTLPixelFormat,
    width: u32,
    height: u32,
    draw: &GradientDraw,
    lease: &context::PreparedResourceLease,
) -> Result<(), RendererError> {
    encoder.setViewport(MTLViewport {
        originX: 0.0,
        originY: 0.0,
        width: width as f64,
        height: height as f64,
        znear: 0.0,
        zfar: 1.0,
    });
    let midpoint_pipeline = context.midpoint_draw_pipeline(pixel_format)?;
    encoder.setRenderPipelineState(&midpoint_pipeline);
    // SAFETY: buffer slot zero is the pinned path-patch vertex ABI, offset zero
    // is aligned, and the context retains the complete buffer through completion.
    unsafe {
        encoder.setVertexBuffer_offset_atIndex(Some(context.path_patch_vertex_buffer()), 0, 0);
    }
    // The first eight tessellation texels are a padding patch. Match
    // RenderContextMetalImpl's PATH_BASE_INSTANCE_UNIFORM_BUFFER_IDX binding
    // so instance zero starts at `base_instance == 1`, rather than joining the
    // zero-filled padding texel at the origin to the authored contour.
    set_vertex_bytes(encoder, &draw.tessellation.base_instance, 4)?;
    bind_vertex_buffer(encoder, &lease.flush_uniforms, 3);
    bind_fragment_buffer(encoder, &lease.flush_uniforms, 3);
    bind_vertex_buffer(encoder, &lease.paths, 5);
    bind_vertex_buffer(encoder, &lease.paints, 6);
    bind_vertex_buffer(encoder, &lease.paint_aux, 7);
    bind_vertex_buffer(encoder, &lease.contours, 8);
    // SAFETY: slots 7, 8, 9, and 11 are the exact pinned ubershader ABI. The
    // lease and context retain every texture and sampler until synchronous
    // command-buffer completion, so no bound Objective-C object can dangle.
    unsafe {
        encoder.setVertexTexture_atIndex(Some(&lease.tessellation), 7);
        encoder.setVertexTexture_atIndex(Some(context.gaussian_integral_texture()), 9);
        encoder.setFragmentTexture_atIndex(lease.gradient.as_deref(), 8);
        encoder.setFragmentTexture_atIndex(Some(context.gaussian_integral_texture()), 9);
        encoder.setFragmentSamplerState_atIndex(
            Some(context.image_sampler(ImageSampler::LINEAR_CLAMP)),
            11,
        );
    }
    encoder.setCullMode(MTLCullMode::Back);
    // SAFETY: the retained patch index buffer contains at least
    // `MIDPOINT_FAN_PATCH_INDEX_COUNT` UInt16 indices from offset zero, and the
    // instance count comes from the validated tessellation batch.
    unsafe {
        encoder
            .drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount(
                MTLPrimitiveType::Triangle,
                gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT,
                MTLIndexType::UInt16,
                context.path_patch_index_buffer(),
                0,
                draw.tessellation.instance_count as usize,
            );
    }
    Ok(())
}

fn set_vertex_bytes<T: Pod>(
    encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
    value: &T,
    index: usize,
) -> Result<(), RendererError> {
    set_vertex_slice(encoder, std::slice::from_ref(value), index)
}

fn bind_vertex_buffer(
    encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
    buffer: &ProtocolObject<dyn objc2_metal::MTLBuffer>,
    index: usize,
) {
    // SAFETY: every buffer is retained by the frame's prepared-resource lease
    // through synchronous command-buffer completion, offset zero is aligned,
    // and each index is the pinned shader ABI selected by the caller.
    unsafe { encoder.setVertexBuffer_offset_atIndex(Some(buffer), 0, index) };
}

fn bind_fragment_buffer(
    encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
    buffer: &ProtocolObject<dyn objc2_metal::MTLBuffer>,
    index: usize,
) {
    // SAFETY: the retained shared buffer remains alive through synchronous
    // completion, offset zero is aligned, and the caller supplies the pinned
    // fragment-buffer ABI index.
    unsafe { encoder.setFragmentBuffer_offset_atIndex(Some(buffer), 0, index) };
}

fn set_vertex_slice<T: Pod>(
    encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
    values: &[T],
    index: usize,
) -> Result<(), RendererError> {
    let bytes: &[u8] = bytemuck::cast_slice(values);
    let pointer = NonNull::new(bytes.as_ptr().cast_mut().cast::<c_void>()).ok_or_else(|| {
        RendererError::NativeMetal("native Metal inline vertex data is empty".to_owned())
    })?;
    // SAFETY: `T: Pod` guarantees initialized bytes, the non-null check rejects
    // an empty slice, and Metal copies `bytes` during `setVertexBytes` before
    // the borrowed Rust slice can expire.
    unsafe {
        encoder.setVertexBytes_length_atIndex(pointer, bytes.len(), index);
    }
    Ok(())
}

/// Creates one complete size-dependent target generation for the diagnostic
/// adapter. Upstream constructs `RenderTargetMetal` as one owner and attaches
/// the product-supplied texture later; this headless tracer creates and
/// attaches its shared readback texture at the same boundary.
fn make_tracer_target(
    context: &NativeMetalContext,
    width: u32,
    height: u32,
) -> Result<RenderTargetMetal, RendererError> {
    let mut target = RenderTargetMetal::new(
        context.retained_device(),
        MTLPixelFormat::BGRA8Unorm,
        width,
        height,
        context.capabilities(),
    )?;
    // SAFETY: `NativeMetalFactory::{new,resize}` validate both dimensions
    // against the selected device limit before this helper is called, and
    // `RenderTargetMetal::new` above independently rejects zero dimensions.
    // Both values widen losslessly from `u32` to `usize`; Metal receives no
    // borrowed Rust storage from this convenience constructor.
    let descriptor = unsafe {
        MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
            MTLPixelFormat::BGRA8Unorm,
            width as usize,
            height as usize,
            false,
        )
    };
    descriptor.setStorageMode(MTLStorageMode::Shared);
    descriptor.setUsage(MTLTextureUsage::RenderTarget);
    let texture = context
        .device()
        .newTextureWithDescriptor(&descriptor)
        .ok_or_else(|| RendererError::NativeMetal("failed to allocate render target".into()))?;
    target.set_target_texture(Some(texture))?;
    Ok(target)
}

fn clear_color(color: u32) -> MTLClearColor {
    let [alpha, red, green, blue] = color.to_be_bytes();
    let premultiply = |channel: u8| f64::from(u16::from(channel) * u16::from(alpha) / 255) / 255.0;
    MTLClearColor {
        red: premultiply(red),
        green: premultiply(green),
        blue: premultiply(blue),
        alpha: f64::from(alpha) / 255.0,
    }
}

fn validate_extent(width: u32, height: u32, max_texture_size: u32) -> Result<(), RendererError> {
    if width == 0 || height == 0 || width > max_texture_size || height > max_texture_size {
        return Err(RendererError::InvalidTextureExtent {
            label: "render target",
            width,
            height,
            max_dimension: max_texture_size,
        });
    }
    Ok(())
}

fn select_native_metal_mode(
    capabilities: MetalCapabilitySelection,
    requested: Option<RenderMode>,
) -> Result<RenderMode, RendererError> {
    match requested {
        Some(RenderMode::Msaa) => Err(RendererError::Unsupported(
            "native Metal does not implement WebGPU MSAA",
        )),
        Some(RenderMode::RasterOrdering) if !capabilities.supports_raster_ordering => Err(
            RendererError::Unsupported("native Metal device does not support raster ordering"),
        ),
        Some(RenderMode::ClockwiseAtomic) if !capabilities.supports_atomic_mode => Err(
            RendererError::Unsupported("native Metal device does not support atomic mode"),
        ),
        Some(mode) => Ok(mode),
        None if capabilities.supports_raster_ordering => Ok(RenderMode::RasterOrdering),
        None if capabilities.supports_atomic_mode => Ok(RenderMode::ClockwiseAtomic),
        None => Err(RendererError::Unsupported(
            "native Metal device exposes neither raster-order nor atomic execution",
        )),
    }
}

fn solid_triangle_fan(path: &LogicalPath, transform: Mat2D) -> Option<Vec<[f32; 2]>> {
    let verbs = path.raw_path.verbs();
    let line_count = verbs.iter().filter(|verb| **verb == PathVerb::Line).count();
    let point_count = path.raw_path.points().len();
    if verbs.first() != Some(&PathVerb::Move)
        || verbs.last() != Some(&PathVerb::Close)
        || line_count + 2 != verbs.len()
        || point_count != line_count + 1
    {
        return None;
    }
    let vertex_count = inline_triangle_fan_vertex_count(point_count)?;
    if !fill_rule_accepts_source_winding(path.fill_rule, path.raw_path.points()) {
        return None;
    }
    let points: Vec<[f32; 2]> = path
        .raw_path
        .points()
        .iter()
        .map(|point| {
            let point = transform.transform_point(*point);
            [point.x, point.y]
        })
        .collect();
    if !is_convex_finite_polygon(&points) || !is_pixel_aligned_axis_aligned_rectangle(&points) {
        return None;
    }
    let mut vertices = Vec::with_capacity(vertex_count);
    for index in 1..points.len() - 1 {
        vertices.extend([points[0], points[index], points[index + 1]]);
    }
    Some(vertices)
}

fn fill_rule_accepts_source_winding(fill_rule: FillRule, points: &[Vec2D]) -> bool {
    if fill_rule != FillRule::Clockwise {
        return true;
    }
    let Some(origin) = points.first() else {
        return false;
    };
    let doubled_area = points[1..]
        .windows(2)
        .map(|edge| {
            let ax = f64::from(edge[0].x) - f64::from(origin.x);
            let ay = f64::from(edge[0].y) - f64::from(origin.y);
            let bx = f64::from(edge[1].x) - f64::from(origin.x);
            let by = f64::from(edge[1].y) - f64::from(origin.y);
            ax * by - bx * ay
        })
        .sum::<f64>();
    doubled_area > 0.0
}

fn is_pixel_aligned_axis_aligned_rectangle(points: &[[f32; 2]]) -> bool {
    if points.len() != 4
        || points
            .iter()
            .flatten()
            .any(|coordinate| coordinate.fract() != 0.0)
    {
        return false;
    }
    for index in 0..points.len() {
        let a = points[index];
        let b = points[(index + 1) % points.len()];
        if (a[0] == b[0]) == (a[1] == b[1]) {
            return false;
        }
    }
    points
        .iter()
        .filter(|point| point[0] == points[0][0])
        .count()
        == 2
        && points
            .iter()
            .filter(|point| point[1] == points[0][1])
            .count()
            == 2
}

fn inline_triangle_fan_vertex_count(point_count: usize) -> Option<usize> {
    let vertex_count = point_count.checked_sub(2)?.checked_mul(3)?;
    let vertex_bytes = vertex_count.checked_mul(std::mem::size_of::<[f32; 2]>())?;
    (point_count >= 3 && vertex_bytes <= INLINE_VERTEX_BYTE_LIMIT).then_some(vertex_count)
}

fn is_convex_finite_polygon(points: &[[f32; 2]]) -> bool {
    if points.len() < 3
        || points
            .iter()
            .flatten()
            .any(|coordinate| !coordinate.is_finite())
    {
        return false;
    }

    for first in 0..points.len() {
        let first_next = (first + 1) % points.len();
        for second in first + 1..points.len() {
            let second_next = (second + 1) % points.len();
            if first == second_next || first_next == second {
                continue;
            }
            if line_segments_intersect(
                points[first],
                points[first_next],
                points[second],
                points[second_next],
            ) {
                return false;
            }
        }
    }

    let mut winding = 0.0_f32;
    for index in 0..points.len() {
        let a = points[index];
        let b = points[(index + 1) % points.len()];
        let c = points[(index + 2) % points.len()];
        let cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
        if cross.abs() <= f32::EPSILON {
            continue;
        }
        if winding != 0.0 && cross.signum() != winding.signum() {
            return false;
        }
        winding = cross;
    }
    winding != 0.0
}

fn line_segments_intersect(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> bool {
    fn orientation(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
        (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
    }

    fn on_segment(a: [f32; 2], b: [f32; 2], point: [f32; 2]) -> bool {
        point[0] >= a[0].min(b[0])
            && point[0] <= a[0].max(b[0])
            && point[1] >= a[1].min(b[1])
            && point[1] <= a[1].max(b[1])
    }

    let ac = orientation(a, b, c);
    let ad = orientation(a, b, d);
    let ca = orientation(c, d, a);
    let cb = orientation(c, d, b);
    if ac * ad < 0.0 && ca * cb < 0.0 {
        return true;
    }
    (ac == 0.0 && on_segment(a, b, c))
        || (ad == 0.0 && on_segment(a, b, d))
        || (ca == 0.0 && on_segment(c, d, a))
        || (cb == 0.0 && on_segment(c, d, b))
}

fn select_apple_platform(device: &ProtocolObject<dyn MTLDevice>) -> ApplePlatform {
    #[cfg(target_os = "macos")]
    let platform = ApplePlatform::MacOs;
    #[cfg(all(target_os = "ios", target_abi = "sim"))]
    let platform = ApplePlatform::IosSimulator {
        host_is_arm64: cfg!(target_arch = "aarch64"),
    };
    #[cfg(all(target_os = "ios", not(target_abi = "sim")))]
    let platform = ApplePlatform::IosDevice {
        is_apple_silicon: device.supportsFamily(MTLGPUFamily::Apple4),
    };
    #[cfg(any(target_os = "macos", all(target_os = "ios", target_abi = "sim")))]
    let _ = device;
    platform
}

fn select_device_capabilities(
    device: &ProtocolObject<dyn MTLDevice>,
    platform: ApplePlatform,
    disable_framebuffer_reads: bool,
) -> MetalCapabilitySelection {
    let device_capabilities = MetalDeviceCapabilities {
        supports_apple1: device.supportsFamily(MTLGPUFamily::Apple1),
        supports_apple2: device.supportsFamily(MTLGPUFamily::Apple2),
        supports_apple3: device.supportsFamily(MTLGPUFamily::Apple3),
        supports_common2: device.supportsFamily(MTLGPUFamily::Common2),
        supports_mac2: device.supportsFamily(MTLGPUFamily::Mac2),
        raster_order_groups: device.areRasterOrderGroupsSupported(),
    };

    select_capabilities(platform, device_capabilities, disable_framebuffer_reads)
}

fn premultiplied_color(color: ColorInt, opacity: f32) -> [f32; 4] {
    let [alpha, red, green, blue] = color.to_be_bytes();
    let alpha = f32::from(alpha) / 255.0 * opacity.clamp(0.0, 1.0);
    [
        f32::from(red) / 255.0 * alpha,
        f32::from(green) / 255.0 * alpha,
        f32::from(blue) / 255.0 * alpha,
        alpha,
    ]
}

fn make_solid_pipeline(
    device: &ProtocolObject<dyn MTLDevice>,
    pixel_format: MTLPixelFormat,
) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, RendererError> {
    let library = new_library_from_metallib_bytes(device, TRACER_METALLIB)?;
    let vertex = library
        .newFunctionWithName(&NSString::from_str("nuxie_tracer_solid_vertex"))
        .ok_or_else(|| RendererError::NativeMetal("tracer vertex function is absent".into()))?;
    let fragment = library
        .newFunctionWithName(&NSString::from_str("nuxie_tracer_solid_fragment"))
        .ok_or_else(|| RendererError::NativeMetal("tracer fragment function is absent".into()))?;
    let descriptor = MTLRenderPipelineDescriptor::new();
    descriptor.setVertexFunction(Some(&vertex));
    descriptor.setFragmentFunction(Some(&fragment));
    let attachment = unsafe { descriptor.colorAttachments().objectAtIndexedSubscript(0) };
    attachment.setPixelFormat(pixel_format);
    attachment.setBlendingEnabled(true);
    attachment.setSourceRGBBlendFactor(MTLBlendFactor::One);
    attachment.setDestinationRGBBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
    attachment.setSourceAlphaBlendFactor(MTLBlendFactor::One);
    attachment.setDestinationAlphaBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
    device
        .newRenderPipelineStateWithDescriptor_error(&descriptor)
        .map_err(|error| RendererError::NativeMetal(format!("create solid pipeline: {error:?}")))
}

fn new_library_from_metallib_bytes(
    device: &ProtocolObject<dyn MTLDevice>,
    bytes: &[u8],
) -> Result<Retained<ProtocolObject<dyn MTLLibrary>>, RendererError> {
    let buffer = NonNull::new(bytes.as_ptr().cast_mut().cast::<c_void>())
        .ok_or_else(|| RendererError::NativeMetal("embedded metallib is empty".into()))?;
    let data = unsafe { dispatch_data_create(buffer, bytes.len(), None, std::ptr::null_mut()) };
    if data.is_null() {
        return Err(RendererError::NativeMetal(
            "failed to create dispatch data for metallib".into(),
        ));
    }
    let result: Result<Retained<ProtocolObject<dyn MTLLibrary>>, Retained<NSError>> =
        unsafe { msg_send![device, newLibraryWithData: data, error: _] };
    unsafe { dispatch_release(data) };
    result.map_err(|error| RendererError::NativeMetal(format!("load tracer metallib: {error:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_metal::MTLCommandQueue;

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn factory_exposes_one_retained_ordered_metal_queue() {
        let factory = NativeMetalFactory::new(2, 2).expect("create native Metal factory");
        let first = factory.retained_metal_queue();
        let second = factory.retained_metal_queue();

        assert_eq!(Retained::as_ptr(&first), Retained::as_ptr(&second));
        assert_eq!(
            Retained::as_ptr(&first.device()),
            Retained::as_ptr(&factory.retained_metal_device())
        );
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn begin_frame_owns_one_uncommitted_buffer_and_one_target_generation() {
        let mut factory = NativeMetalFactory::new(2, 2).expect("create native Metal factory");
        assert!(
            !factory.context.feather_atlas_resources_are_initialized(),
            "non-atlas factory creation must not realize the lazy atlas texture or pipelines"
        );
        let first_generation = Rc::clone(&factory.target);
        let frame = factory
            .begin_frame(0)
            .expect("acquire frame-owned command buffer");

        assert_eq!(
            frame.command_buffer.status(),
            objc2_metal::MTLCommandBufferStatus::NotEnqueued
        );
        assert!(Rc::ptr_eq(&frame.target, &first_generation));

        // Upstream mutates a reference-counted target after construction: the
        // product attaches its drawable texture and the atomic path lazily
        // realizes storage. Prove the shared Rust generation preserves both
        // mutations even while a frame retains it.
        let retained_texture = first_generation
            .borrow()
            .retained_target_texture()
            .expect("tracer target owns its attached texture");
        {
            let mut target = first_generation.borrow_mut();
            let first_atomic = NonNull::from(
                target
                    .color_atomic_buffer()
                    .expect("allocate shared generation atomic buffer"),
            );
            let second_atomic = NonNull::from(
                target
                    .color_atomic_buffer()
                    .expect("reuse shared generation atomic buffer"),
            );
            assert_eq!(first_atomic, second_atomic);

            target
                .set_target_texture(None)
                .expect("detach product texture from shared generation");
            assert!(target.target_texture().is_none());
            target
                .set_target_texture(Some(retained_texture))
                .expect("reattach product texture to shared generation");
        }

        factory.resize(3, 1).expect("replace target generation");
        assert!(!Rc::ptr_eq(&factory.target, &first_generation));
        let old_target = frame.target.borrow();
        assert_eq!((old_target.width(), old_target.height()), (2, 2));
        assert_eq!(factory.dimensions(), (3, 1));
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn abandoned_frame_releases_its_native_command_buffer_and_target_generation() {
        use objc2::rc::Weak;
        use objc2_metal::{MTLBuffer, MTLTexture};

        let mut factory = NativeMetalFactory::new(2, 2).expect("create native Metal factory");
        let (command_buffer, texture_owners, atomic_buffer_owners) =
            objc2::rc::autoreleasepool(|_| {
                let frame = factory
                    .begin_frame(0)
                    .expect("acquire frame-owned command buffer");
                let command_buffer = Weak::new(&*frame.command_buffer);

                let texture_owners: Vec<Weak<ProtocolObject<dyn MTLTexture>>> = {
                    let target = frame.target.borrow();
                    [
                        target.target_texture(),
                        target.coverage_memoryless_texture(),
                        target.clip_memoryless_texture(),
                        target.scratch_color_memoryless_texture(),
                    ]
                    .into_iter()
                    .flatten()
                    .map(Weak::new)
                    .collect()
                };
                let atomic_buffer_owners: Vec<Weak<ProtocolObject<dyn MTLBuffer>>> = {
                    let mut target = frame.target.borrow_mut();
                    vec![
                        Weak::new(
                            target
                                .color_atomic_buffer()
                                .expect("allocate color atomic buffer"),
                        ),
                        Weak::new(
                            target
                                .coverage_atomic_buffer()
                                .expect("allocate coverage atomic buffer"),
                        ),
                        Weak::new(
                            target
                                .clip_atomic_buffer()
                                .expect("allocate clip atomic buffer"),
                        ),
                    ]
                };

                factory.resize(3, 1).expect("replace target generation");
                assert!(command_buffer.load().is_some());
                assert!(texture_owners.iter().all(|owner| owner.load().is_some()));
                assert!(atomic_buffer_owners
                    .iter()
                    .all(|owner| owner.load().is_some()));

                drop(frame);
                (command_buffer, texture_owners, atomic_buffer_owners)
            });
        assert!(command_buffer.load().is_none());
        assert!(texture_owners.iter().all(|owner| owner.load().is_none()));
        assert!(atomic_buffer_owners
            .iter()
            .all(|owner| owner.load().is_none()));
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn factory_rejects_invalid_extent_before_context_resource_creation() {
        assert!(matches!(
            NativeMetalFactory::new(0, 1),
            Err(RendererError::InvalidTextureExtent {
                label: "render target",
                width: 0,
                height: 1,
                ..
            })
        ));
        assert!(matches!(
            NativeMetalFactory::new_with_mode(0, 1, RenderMode::Msaa),
            Err(RendererError::InvalidTextureExtent {
                label: "render target",
                width: 0,
                height: 1,
                ..
            })
        ));
    }

    #[test]
    fn mode_selection_prefers_raster_and_maps_the_explicit_atomic_request() {
        let both = MetalCapabilitySelection {
            max_texture_size: 16_384,
            supports_raster_ordering: true,
            supports_atomic_mode: true,
            path_id_granularity: 1,
            supports_texture_compression_etc2: false,
            supports_texture_compression_astc: true,
            supports_texture_compression_bc: true,
            atomic_barrier_type: AtomicBarrierType::RasterOrderGroup,
        };
        assert_eq!(
            select_native_metal_mode(both, None).unwrap(),
            RenderMode::RasterOrdering
        );
        assert_eq!(
            select_native_metal_mode(both, Some(RenderMode::ClockwiseAtomic)).unwrap(),
            RenderMode::ClockwiseAtomic
        );
        assert!(matches!(
            select_native_metal_mode(both, Some(RenderMode::Msaa)),
            Err(RendererError::Unsupported(_))
        ));

        let atomic_only = MetalCapabilitySelection {
            supports_raster_ordering: false,
            ..both
        };
        assert_eq!(
            select_native_metal_mode(atomic_only, None).unwrap(),
            RenderMode::ClockwiseAtomic
        );
        assert!(matches!(
            select_native_metal_mode(atomic_only, Some(RenderMode::RasterOrdering)),
            Err(RendererError::Unsupported(_))
        ));
    }

    #[test]
    fn polygon_filter_accepts_both_convex_windings() {
        assert!(is_convex_finite_polygon(&[
            [0.0, 0.0],
            [2.0, 0.0],
            [2.0, 1.0],
            [0.0, 1.0],
        ]));
        assert!(is_convex_finite_polygon(&[
            [0.0, 1.0],
            [2.0, 1.0],
            [2.0, 0.0],
            [0.0, 0.0],
        ]));
    }

    #[test]
    fn polygon_filter_rejects_geometry_the_triangle_fan_cannot_render_exactly() {
        assert!(!is_convex_finite_polygon(&[
            [0.0, 0.0],
            [2.0, 0.0],
            [1.0, 0.5],
            [2.0, 1.0],
            [0.0, 1.0],
        ]));
        assert!(!is_convex_finite_polygon(&[
            [0.0, 0.0],
            [f32::NAN, 0.0],
            [0.0, 1.0],
        ]));
        assert!(!is_convex_finite_polygon(&[
            [1.0, 0.0],
            [-0.809_017, 0.587_785],
            [0.309_017, -0.951_057],
            [0.309_017, 0.951_057],
            [-0.809_017, -0.587_785],
        ]));
    }

    #[test]
    fn diagnostic_pipeline_accepts_only_pixel_aligned_axis_aligned_rectangles() {
        assert!(is_pixel_aligned_axis_aligned_rectangle(&[
            [8.0, 8.0],
            [56.0, 8.0],
            [56.0, 56.0],
            [8.0, 56.0],
        ]));
        assert!(!is_pixel_aligned_axis_aligned_rectangle(&[
            [4.0, 4.0],
            [60.0, 4.0],
            [32.0, 60.0],
        ]));
        assert!(!is_pixel_aligned_axis_aligned_rectangle(&[
            [8.5, 8.0],
            [56.0, 8.0],
            [56.0, 56.0],
            [8.5, 56.0],
        ]));
    }

    #[test]
    fn clockwise_fill_checks_source_winding_but_accepts_reflected_draws() {
        let make_path = |points: &[[f32; 2]], fill_rule| {
            let mut raw_path = RawPath::new();
            raw_path.move_to(points[0][0], points[0][1]);
            for point in &points[1..] {
                raw_path.line_to(point[0], point[1]);
            }
            raw_path.close();
            LogicalPath {
                raw_path: Arc::new(raw_path),
                fill_rule,
                valid: true,
            }
        };
        let clockwise = make_path(
            &[[8.0, 8.0], [56.0, 8.0], [56.0, 56.0], [8.0, 56.0]],
            FillRule::Clockwise,
        );
        let counterclockwise = make_path(
            &[[8.0, 56.0], [56.0, 56.0], [56.0, 8.0], [8.0, 8.0]],
            FillRule::Clockwise,
        );
        let translated_unit = make_path(
            &[
                [4096.0, 4096.0],
                [4097.0, 4096.0],
                [4097.0, 4097.0],
                [4096.0, 4097.0],
            ],
            FillRule::Clockwise,
        );
        let reflected = Mat2D([-1.0, 0.0, 0.0, 1.0, 64.0, 0.0]);

        assert!(solid_triangle_fan(&clockwise, Mat2D::IDENTITY).is_some());
        assert!(solid_triangle_fan(&clockwise, reflected).is_some());
        assert!(solid_triangle_fan(&translated_unit, Mat2D::IDENTITY).is_some());
        assert!(solid_triangle_fan(&counterclockwise, Mat2D::IDENTITY).is_none());
    }

    #[test]
    fn extent_validation_reports_the_selected_device_limit() {
        assert!(validate_extent(8_192, 8_192, 8_192).is_ok());
        assert!(matches!(
            validate_extent(8_193, 1, 8_192),
            Err(RendererError::InvalidTextureExtent {
                max_dimension: 8_192,
                ..
            })
        ));
    }

    #[test]
    fn admitted_atlas_fixture_has_pinned_cpp_extent_and_threshold() {
        let mut path = RawPath::new();
        path.move_to(16.0, 16.0);
        path.line_to(48.0, 16.0);
        path.line_to(48.0, 48.0);
        path.line_to(16.0, 48.0);
        path.close();
        let paint = LogicalPaint {
            style: RenderPaintStyle::Stroke,
            thickness: 8.0,
            feather: 24.0,
            ..LogicalPaint::default()
        };

        assert!(crate::draw::feather_requires_atlas(
            paint.feather,
            Mat2D::IDENTITY,
            false
        ));
        let placement = crate::feather_atlas_placement(
            &path,
            Mat2D::IDENTITY,
            paint.feather,
            paint.effective_stroke(),
            64,
            64,
        )
        .expect("atlas-selected fixture has a visible placement");
        assert_eq!(placement.bounds, [0.0, 0.0, 64.0, 64.0]);
        assert_eq!([placement.width, placement.height], [33, 33]);
        assert_eq!(
            crate::cpp_webgpu_atlas_physical_size([33, 33], [64, 64], 16_384),
            [41, 41]
        );
    }

    #[test]
    fn triangle_fan_size_is_bounded_before_allocating() {
        assert_eq!(inline_triangle_fan_vertex_count(172), Some(510));
        assert_eq!(inline_triangle_fan_vertex_count(173), None);
        assert_eq!(inline_triangle_fan_vertex_count(usize::MAX), None);
    }

    #[test]
    fn four_atomic_draw_groups_apply_exact_upstream_barrier_policy() {
        assert_eq!(
            atomic_barrier_inventory(4, AtomicBarrierType::RasterOrderGroup),
            AtomicBarrierInventory {
                semantic: 5,
                memory: 0,
                render_pass_breaks: 0,
            }
        );
        assert_eq!(
            atomic_barrier_inventory(4, AtomicBarrierType::MemoryBarrier),
            AtomicBarrierInventory {
                semantic: 5,
                memory: 5,
                render_pass_breaks: 0,
            }
        );
        assert_eq!(
            atomic_barrier_inventory(4, AtomicBarrierType::RenderPassBreak),
            AtomicBarrierInventory {
                semantic: 5,
                memory: 0,
                render_pass_breaks: 5,
            }
        );
        assert_eq!(
            1 + atomic_barrier_inventory(4, AtomicBarrierType::RenderPassBreak).render_pass_breaks,
            6,
            "initial main pass plus one replacement pass per semantic barrier"
        );
    }
}
