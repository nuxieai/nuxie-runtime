//! Experimental native Metal renderer adapter.
//!
//! The Apple product does not select this adapter until UNIV-2092. The module
//! begins as the UNIV-2086 tracer and grows by mechanically porting the pinned
//! upstream Metal implementation behind the existing renderer seam.

#[cfg(test)]
#[allow(dead_code)]
mod background_shader_compiler;
#[cfg(test)]
mod buffer;
// These mixed modules retain source-shaped configuration and test seams that
// the mechanical Metal owner consumes only in selected source branches.
// Preserve the complete shapes until the parity campaign is closed.
#[cfg(test)]
#[allow(dead_code)]
mod buffer_ring_coordinator;
mod capabilities;
mod command_submission;
#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod context;
mod context_options;
#[cfg(test)]
#[allow(dead_code)]
mod draw_combinations;
#[cfg(test)]
#[allow(dead_code)]
mod draw_pass;
#[cfg(test)]
#[allow(dead_code)]
mod draw_pipeline;
#[cfg(test)]
#[allow(dead_code)]
mod draw_shader;
mod drawable;
#[cfg(test)]
#[allow(dead_code)]
mod feather_atlas_pipeline;
#[cfg(test)]
#[allow(dead_code)]
mod feather_atlas_resource;
#[cfg(test)]
#[allow(dead_code)]
mod gradient_resource;
#[cfg(test)]
#[allow(dead_code)]
mod image_texture;
mod mechanical_render_context;
#[allow(dead_code)]
mod objc2_execution;
#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod pipeline_cache;
#[cfg(test)]
#[allow(dead_code)]
mod pipeline_names;
#[cfg(feature = "native-ore-metal-experimental")]
#[allow(dead_code)]
mod render_canvas;
#[cfg(test)]
#[allow(dead_code)]
mod render_target;
mod source_capabilities;
#[cfg(test)]
#[allow(dead_code)]
mod samplers;
#[cfg(test)]
#[allow(dead_code)]
mod shader_compile_plan;
#[cfg(test)]
#[allow(dead_code)]
mod tessellation_resource;
#[cfg(test)]
#[allow(dead_code)]
mod upload_buffer_ring;

#[cfg(any())]
use super::gpu;
use super::{BackendWorkMetrics, RenderMode, RendererError};
#[cfg(test)]
use super::{LogicalPaint, LogicalPath};
use crate::mechanical_port::source::include::rive::renderer_hpp::RendererContract;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_buffer_hpp::RenderResourceDomain;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_buffer_hpp::RiveRenderBufferHandle;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImageHandle;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_renderer_hpp::RiveRenderer;
use crate::mechanical_port::source::renderer::src::rive_render_paint_hpp::RiveRenderPaintHandle;
use crate::mechanical_port::source::renderer::src::rive_render_path_hpp::RiveRenderPathHandle;
use source_capabilities::MetalCapabilitySelection;
#[cfg(test)]
use source_capabilities::AtomicBarrierType;
#[cfg(test)]
use capabilities::{select_capabilities, ApplePlatform, MetalDeviceCapabilities};
#[cfg(any())]
use context::NativeMetalContext;
use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageSampler, Mat2D, RawPath,
    RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint, RenderPath,
    RenderShader, Renderer,
};
#[cfg(test)]
use nuxie_render_api::{PathVerb, RenderPaintStyle, Vec2D};
use objc2::runtime::{AnyObject, ProtocolObject};
#[cfg(test)]
use objc2::msg_send;
use objc2::rc::Retained;
use objc2_execution::ActualMetalExecutionInventory;
#[cfg(test)]
use objc2_foundation::NSError;
use objc2_metal::{
    MTLBuffer, MTLCreateSystemDefaultDevice, MTLDevice, MTLOrigin,
    MTLPixelFormat, MTLRegion, MTLResource,
    MTLSize, MTLStorageMode, MTLTexture, MTLTextureDescriptor, MTLTextureUsage,
};
#[cfg(test)]
use objc2_metal::MTLGPUFamily;
#[cfg(test)]
use objc2_metal::{MTLRenderPipelineDescriptor, MTLRenderPipelineState};
#[cfg(test)]
use objc2_metal::MTLLibrary;
#[cfg(feature = "native-ore-metal-experimental")]
pub use render_canvas::NativeMetalRenderCanvas;
use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;
#[cfg(test)]
use std::sync::Arc;

pub use drawable::NativeMetalDrawableFrame;
pub use context_options::{
    NativeMetalContextOptions, NativeMetalSynthesizedFailureType, ShaderCompilationMode,
};
#[cfg(test)]
const INLINE_VERTEX_BYTE_LIMIT: usize = 4_096;

pub(super) struct MechanicalMetalHost;

impl objc2_execution::NativeMetalHostCallbacks for MechanicalMetalHost {
    fn log(&mut self, message: String) {
        eprintln!("{message}");
    }

    fn generate_patch_buffer_data(
        &mut self,
        vertex_buffer: &ProtocolObject<dyn objc2_metal::MTLBuffer>,
        index_buffer: &ProtocolObject<dyn objc2_metal::MTLBuffer>,
    ) {
        unsafe {
            crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::GeneratePatchBufferData(
                vertex_buffer.contents().as_ptr().cast(),
                index_buffer.contents().as_ptr().cast(),
            );
        }
    }

    fn make_ore_context(
        &mut self,
        device: &ProtocolObject<dyn MTLDevice>,
        queue: Option<&ProtocolObject<dyn objc2_metal::MTLCommandQueue>>,
    ) -> Option<Box<dyn std::any::Any>> {
        #[cfg(feature = "native-ore-metal-experimental")]
        {
            let queue = queue?;
            let device = unsafe { Retained::retain(core::ptr::from_ref(device).cast_mut()) }?;
            let queue = unsafe { Retained::retain(core::ptr::from_ref(queue).cast_mut()) }?;
            let context = nuxie_ore_metal::metal::context::ContextMetal::MakeChecked(
                Some(device),
                Some(queue),
            )?;
            return Some(context as Box<dyn std::any::Any>);
        }
        #[cfg(not(feature = "native-ore-metal-experimental"))]
        {
            let _ = (device, queue);
            None
        }
    }

}

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
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    queue: RefCell<Option<Retained<ProtocolObject<dyn objc2_metal::MTLCommandQueue>>>>,
    // Immutable snapshot copied from the canonical RenderContextMetal source
    // owner during bootstrap. Product admission/reporting must consume this
    // snapshot; no parallel device-family probe is authoritative.
    capabilities: MetalCapabilitySelection,
    target_texture: Rc<RefCell<Retained<ProtocolObject<dyn MTLTexture>>>>,
    target_width: u32,
    target_height: u32,
    /// The canonical source owner is rooted exactly once during bootstrap.
    /// It is never lazily reconstructed: doing so would create a second
    /// capability/selector trace and split native ownership across two
    /// RenderContextMetal instances.
    mechanical: Rc<RefCell<mechanical_render_context::MechanicalRenderContext>>,
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
    /// Physical atomic content draw selectors successfully submitted by the
    /// adapter. This is not an authored/logical Rive draw count.
    pub atomic_draws: usize,
    /// Physical instances submitted by those atomic draw selectors.
    pub atomic_draw_instances: usize,
    /// Physical image-rectangle draw selectors successfully submitted by the
    /// translated source execution.
    pub image_rect_draw_calls: usize,
    /// Physical image-mesh draw selectors successfully submitted by the
    /// translated source execution.
    pub image_mesh_draw_calls: usize,
    /// Successful source image-texture bindings at IMAGE_TEXTURE_IDX.
    pub image_texture_binds: usize,
    pub atomic_draw_groups: usize,
    /// Semantic PLS barriers: initial, group transitions, and pre-resolve.
    pub atomic_barriers: usize,
    pub atomic_memory_barriers: usize,
    pub atomic_render_pass_breaks: usize,
    pub atomic_raster_order_group_barriers: usize,
}

impl NativeMetalExecutionInventory {
    fn from_execution(mode: RenderMode, metrics: ActualMetalExecutionInventory) -> Self {
        Self {
            mode,
            color_ramp_pipeline: metrics.color_ramp_draw_calls > 0,
            gradient_texture: metrics.gradient_texture_binds > 0,
            atomic_color_plane: metrics.atomic_color_plane_draw_calls > 0,
            advanced_blend_pipeline: metrics.advanced_blend_draw_calls > 0,
            hsl_blend_pipeline: metrics.hsl_blend_draw_calls > 0,
            fixed_function_color_output: metrics.fixed_function_draw_calls > 0,
            atomic_clip_plane: metrics.clip_atomic_buffer_binds > 0,
            atomic_coverage_plane: metrics.coverage_atomic_buffer_binds > 0,
            render_pass_initialize_pipeline: metrics.render_pass_initialize_draw_calls > 0,
            midpoint_fan_pipeline: metrics.midpoint_fan_draw_calls > 0,
            render_pass_resolve_pipeline: metrics.render_pass_resolve_draw_calls > 0,
            clipped_path_pipeline_set: metrics.clip_feature_draw_calls > 0,
            clip_rect_pipeline: metrics.clip_rect_feature_draw_calls > 0,
            outer_curve_pipeline: metrics.outer_curve_draw_calls > 0,
            interior_triangulation_pipeline: metrics.interior_triangulation_draw_calls > 0,
            atomic_draws: metrics.atomic_draw_calls,
            atomic_draw_instances: metrics.atomic_draw_instances,
            image_rect_draw_calls: metrics.image_rect_draw_calls,
            image_mesh_draw_calls: metrics.image_mesh_draw_calls,
            image_texture_binds: metrics.image_texture_binds,
            atomic_draw_groups: metrics.draw_groups,
            atomic_barriers: metrics.semantic_atomic_barriers,
            atomic_memory_barriers: metrics.memory_barriers,
            atomic_render_pass_breaks: metrics.render_pass_breaks,
            atomic_raster_order_group_barriers: metrics.raster_order_group_barriers,
        }
    }
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

    /// Source-shaped device-injected `MakeContext` adaptation. Queue and target
    /// resources retain their nullable Metal states; the mechanical frame
    /// owner is created lazily once both are nonnil.
    pub fn new_with_device_and_context_options(
        width: u32,
        height: u32,
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        options: NativeMetalContextOptions,
    ) -> Result<Self, RendererError> {
        Self::new_with_device_impl(width, height, None, device, options)
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
        Self::new_with_device_impl(width, height, requested_mode, device, options)
    }

    fn new_with_device_impl(
        width: u32,
        height: u32,
        requested_mode: Option<RenderMode>,
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        options: NativeMetalContextOptions,
    ) -> Result<Self, RendererError> {
        // RenderContextMetal is the sole capability authority. It is rooted
        // before any queue/target allocation and publishes the immutable
        // PlatformFeatures/MetalFeatures snapshot consumed below.
        let provisional_mode = requested_mode.unwrap_or(RenderMode::RasterOrdering);
        let mut mechanical = mechanical_render_context::MechanicalRenderContext::new_source(
            device.clone(),
            width,
            height,
            provisional_mode,
            options,
        )?;
        let capabilities = mechanical.source_capabilities();
        validate_extent(width, height, capabilities.max_texture_size)?;
        let mode = select_native_metal_mode(capabilities, requested_mode)?;
        let queue = device.newCommandQueue().ok_or_else(|| {
            RendererError::NativeMetal("failed to create native Metal command queue".into())
        })?;
        let target_texture = make_native_target_texture(&device, width, height)?;
        mechanical.set_mode(mode);
        mechanical.install_queue_and_target(
            queue.clone(),
            target_texture.clone(),
            width,
            height,
        )?;
        let mechanical = Rc::new(RefCell::new(mechanical));
        Ok(Self {
            device,
            queue: RefCell::new(Some(queue)),
            capabilities,
            target_texture: Rc::new(RefCell::new(target_texture)),
            target_width: width,
            target_height: height,
            mechanical,
            mode,
        })
    }

    fn mechanical_context(
        &self,
    ) -> Result<Rc<RefCell<mechanical_render_context::MechanicalRenderContext>>, RendererError>
    {
        Ok(Rc::clone(&self.mechanical))
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.target_width, self.target_height)
    }

    pub fn adapter_name(&self) -> String {
        self.device.name().to_string()
    }

    pub fn render_mode(&self) -> RenderMode {
        self.mode
    }

    /// Immutable capability view copied from the canonical source owner at
    /// construction. No adapter-side device-family query is performed here.
    pub(crate) fn source_capabilities(&self) -> MetalCapabilitySelection {
        self.capabilities
    }

    /// Replaces the dimensions used by subsequently created frames. Resize is
    /// rejected while the persistent mechanical context has an active frame;
    /// completed generations are retired only after their source submission.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        validate_extent(width, height, self.source_capabilities().max_texture_size)?;
        // Construct every size-dependent Metal owner before replacement. If
        // any allocation fails, the current generation remains intact.
        let replacement = make_native_target_texture(&self.device, width, height)?;
        self.mechanical
            .borrow_mut()
            .replace_target(replacement.clone(), width, height)?;
        *self.target_texture.borrow_mut() = replacement;
        self.target_width = width;
        self.target_height = height;
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
        let mechanical = self.mechanical_context()?;
        let (renderer, frame_number, resource_domain) = {
            let mut mechanical_context = mechanical.borrow_mut();
            let target_texture = self.target_texture.borrow().clone();
            if !mechanical_context.target_matches(
                &target_texture,
                self.target_width,
                self.target_height,
            ) {
                mechanical_context.replace_target(
                    target_texture,
                    self.target_width,
                    self.target_height,
                )?;
            }
            mechanical_context.begin_frame(clear_color)?;
            let context =
                unsafe { Pin::get_unchecked_mut(mechanical_context.render_context_mut()) };
            (
                unsafe { RiveRenderer::new_from_context(context) },
                mechanical_context.current_frame_number(),
                mechanical_context.resource_domain(),
            )
        };
        Ok(NativeMetalFrame {
            mechanical,
            renderer,
            resource_domain,
            collect_work_metrics,
            frame_number,
        })
    }

    /// Copies the selected device so the platform caller can configure its
    /// presentation owner without transferring that policy to the renderer.
    pub fn retained_metal_device(&self) -> Retained<ProtocolObject<dyn MTLDevice>> {
        self.device.clone()
    }

    /// Copies the renderer's ordered command queue for same-context Metal
    /// adapters. Work submitted by ORE/canvas integration must use this queue
    /// rather than selecting a second device or creating an unrelated queue.
    pub fn retained_metal_queue(
        &self,
    ) -> Option<Retained<ProtocolObject<dyn objc2_metal::MTLCommandQueue>>> {
        self.queue.borrow().clone()
    }

    /// Exact nullable `setCommandQueue` ownership transition.
    pub fn set_metal_command_queue(
        &self,
        queue: Option<Retained<ProtocolObject<dyn objc2_metal::MTLCommandQueue>>>,
    ) {
        *self.queue.borrow_mut() = queue.clone();
        self.mechanical.borrow_mut().set_command_queue(queue);
    }

    /// Wraps a caller-created Metal texture as a retained renderer image
    /// without allocating storage or uploading bytes.
    pub fn adopt_metal_image_texture(
        &self,
        texture: Retained<ProtocolObject<dyn MTLTexture>>,
        width: u32,
        height: u32,
    ) -> Option<Box<dyn RenderImage>> {
        if width == 0
            || height == 0
            || texture.width() as u32 != width
            || texture.height() as u32 != height
            || texture.textureType() != objc2_metal::MTLTextureType::Type2D
            || texture.depth() != 1
            || texture.arrayLength() != 1
            || texture.sampleCount() != 1
            || !texture.usage().contains(MTLTextureUsage::ShaderRead)
            || Retained::as_ptr(&texture.device()) != Retained::as_ptr(&self.device)
        {
            return None;
        }
        let mechanical = self.mechanical_context().ok()?;
        let domain = mechanical.borrow().resource_domain();
        let image = mechanical
            .borrow_mut()
            .adopt_image_handle(texture, width, height);
        image
            .map(|image| image.with_execution_domain(domain, Rc::clone(&mechanical) as Rc<dyn Any>))
            .map(|image| Box::new(image) as Box<dyn RenderImage>)
    }

    /// Creates a private texture shared by a render-target owner and a
    /// sampleable-image owner, matching the pinned Metal RenderCanvas factory.
    #[cfg(feature = "native-ore-metal-experimental")]
    pub fn make_metal_render_canvas(
        &self,
        width: u32,
        height: u32,
    ) -> Result<NativeMetalRenderCanvas, RendererError> {
        validate_extent(width, height, self.source_capabilities().max_texture_size)?;
        let mechanical = self.mechanical_context()?;
        let canvas = {
            let mut mechanical_context = mechanical.borrow_mut();
            let context =
                unsafe { Pin::get_unchecked_mut(mechanical_context.render_context_mut()) };
            context.makeRenderCanvasExecutable(width, height)
        };
        let resource_domain = mechanical.borrow().resource_domain();
        NativeMetalRenderCanvas::from_source(canvas, Rc::clone(&mechanical), resource_domain)
            .ok_or_else(|| {
                RendererError::NativeMetal("mechanical RenderCanvas creation failed".into())
            })
    }

    /// Constructs ORE from the exact retained device and queue owned by this
    /// renderer context. The opt-in feature corresponds to upstream's
    /// `RIVE_CANVAS` build and cannot select a second Metal service.
    #[cfg(feature = "native-ore-metal-experimental")]
    /// Runs a scoped operation against the cached source ORE singleton.
    /// Ownership remains in RenderContext exactly as in the pinned source;
    /// repeated calls observe the same object and it cannot outlive `self`.
    pub fn with_ore_context<R>(
        &self,
        callback: impl FnOnce(&mut nuxie_ore_metal::metal::context::ContextMetal) -> R,
    ) -> Option<R> {
        let mechanical = self.mechanical_context().ok()?;
        let result = mechanical.borrow_mut().with_ore_context(callback);
        result
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub(crate) fn begin_drawable_frame_parts<'a>(
        &self,
        drawable: &'a ProtocolObject<dyn objc2_metal::MTLDrawable>,
        texture: Retained<ProtocolObject<dyn MTLTexture>>,
        clear_color: u32,
    ) -> Result<NativeMetalDrawableFrame<'a>, RendererError> {
        let (expected_width, expected_height) = self.dimensions();
        if texture.pixelFormat() != MTLPixelFormat::BGRA8Unorm {
            return Err(RendererError::NativeMetal(
                "drawable texture is not BGRA8Unorm".into(),
            ));
        }
        let texture_device = texture.device();
        if Retained::as_ptr(&texture_device) != Retained::as_ptr(&self.device) {
            return Err(RendererError::NativeMetal(
                "drawable texture belongs to a different MTLDevice".into(),
            ));
        }
        if texture.width() as u32 != expected_width || texture.height() as u32 != expected_height {
            return Err(RendererError::NativeMetal(format!(
                "drawable texture is {}x{}, expected {}x{}",
                texture.width(),
                texture.height(),
                expected_width,
                expected_height,
            )));
        }
        let mechanical = self.mechanical_context()?;
        let restore_texture = self.target_texture.borrow().clone();
        NativeMetalDrawableFrame::new(
            mechanical,
            drawable,
            texture,
            restore_texture,
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
        let mechanical = self
            .mechanical_context()
            .unwrap_or_else(|error| panic!("mechanical render-buffer factory failed: {error}"));
        let source_buffer = mechanical.borrow_mut().make_render_buffer_handle(
            source_buffer_type(buffer_type),
            source_buffer_flags(flags),
            size_in_bytes,
        );
        let domain = mechanical.borrow().resource_domain();
        let source_buffer = source_buffer.map(|buffer| {
            buffer.with_execution_domain(domain, Rc::clone(&mechanical) as Rc<dyn Any>)
        });
        Box::new(
            source_buffer.unwrap_or_else(|| panic!("mechanical render-buffer owner was not valid")),
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
        let mechanical = self
            .mechanical_context()
            .unwrap_or_else(|error| panic!("mechanical linear-gradient factory failed: {error}"));
        let source = mechanical
            .borrow_mut()
            .make_linear_gradient_handle(sx, sy, ex, ey, colors, stops);
        Box::new(source.unwrap_or_else(|| panic!("mechanical linear gradient owner was not valid")))
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        let mechanical = self
            .mechanical_context()
            .unwrap_or_else(|error| panic!("mechanical radial-gradient factory failed: {error}"));
        let source = mechanical
            .borrow_mut()
            .make_radial_gradient_handle(cx, cy, radius, colors, stops);
        Box::new(source.unwrap_or_else(|| panic!("mechanical radial gradient owner was not valid")))
    }

    fn make_render_path(
        &mut self,
        mut raw_path: RawPath,
        fill_rule: FillRule,
    ) -> Box<dyn RenderPath> {
        let mechanical = self
            .mechanical_context()
            .unwrap_or_else(|error| panic!("mechanical render-path factory failed: {error}"));
        let source = mechanical
            .borrow_mut()
            .make_render_path_handle(&mut raw_path, fill_rule);
        Box::new(source.unwrap_or_else(|| panic!("mechanical render-path owner was not valid")))
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        let mechanical = self
            .mechanical_context()
            .unwrap_or_else(|error| panic!("mechanical empty-path factory failed: {error}"));
        let source = mechanical
            .borrow_mut()
            .make_empty_render_path_handle()
            .unwrap_or_else(|| panic!("mechanical empty-path owner was not valid"));
        Box::new(source)
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        let mechanical = self
            .mechanical_context()
            .unwrap_or_else(|error| panic!("mechanical paint factory failed: {error}"));
        let source = mechanical
            .borrow_mut()
            .make_render_paint_handle()
            .unwrap_or_else(|| panic!("mechanical paint owner was not valid"));
        Box::new(source)
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        let mechanical = self.mechanical_context().map_err(|_| ImageDecodeError)?;
        let domain = mechanical.borrow().resource_domain();
        let image = mechanical.borrow_mut().decode_image_handle(data);
        image
            .map(|image| image.with_execution_domain(domain, Rc::clone(&mechanical) as Rc<dyn Any>))
            .map(|image| Box::new(image) as Box<dyn RenderImage>)
            .ok_or(ImageDecodeError)
    }
}

fn source_buffer_type(
    value: RenderBufferType,
) -> crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferType {
    match value {
        RenderBufferType::Index => {
            crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferType::index
        }
        RenderBufferType::Vertex => {
            crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferType::vertex
        }
    }
}

fn source_buffer_flags(
    value: RenderBufferFlags,
) -> crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferFlags {
    match value {
        RenderBufferFlags::None => crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferFlags::none,
        RenderBufferFlags::MappedOnceAtInitialization => crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferFlags::mappedOnceAtInitialization,
    }
}

/// One native Metal frame retained until submission and deterministic readback.
pub struct NativeMetalFrame {
    renderer: RiveRenderer,
    mechanical: Rc<RefCell<mechanical_render_context::MechanicalRenderContext>>,
    resource_domain: RenderResourceDomain,
    collect_work_metrics: bool,
    frame_number: u64,
}

#[cfg(any())]
#[derive(Clone, Copy)]
struct NativeMetalRenderState {
    transform: Mat2D,
    opacity: f32,
}

#[cfg(any())]
impl Default for NativeMetalRenderState {
    fn default() -> Self {
        Self {
            transform: Mat2D::IDENTITY,
            opacity: 1.0,
        }
    }
}

#[cfg(any())]
struct SolidTracerDraw {
    vertices: Vec<[f32; 2]>,
    premultiplied_color: [f32; 4],
}

#[cfg(any())]
struct AtomicPathInput {
    path: LogicalPath,
    paint: LogicalPaint,
    state: super::DrawState,
}

#[cfg(any())]
struct GradientDraw {
    gradient_batch: super::logical_frame::GradientBatch,
    gradient: super::logical_frame::PreparedGradient,
    tessellation: super::draw::FillTessellation,
}

#[cfg(any())]
struct AtlasRequest {
    path: LogicalPath,
    paint: LogicalPaint,
    state: super::DrawState,
}

#[cfg(any())]
struct GradientUploadData {
    flush_uniforms: gpu::FlushUniforms,
    paths: [gpu::PathData; 2],
    paints: [gpu::PaintData; 2],
    paint_aux: [gpu::PaintAuxData; 2],
}

#[cfg(any())]
struct AtlasUploadData {
    flush_uniforms: gpu::FlushUniforms,
    paths: Vec<gpu::PathData>,
    paints: Vec<gpu::PaintData>,
    paint_aux: Vec<gpu::PaintAuxData>,
}

#[cfg(any())]
struct AtomicPathUploadData {
    flush_uniforms: gpu::FlushUniforms,
    paths: Vec<gpu::PathData>,
    paints: Vec<gpu::PaintData>,
    paint_aux: Vec<gpu::PaintAuxData>,
}

#[cfg(any())]
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

#[cfg(any())]
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

#[cfg(any())]
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
        <RiveRenderer as RendererContract>::save(&mut self.renderer);
    }

    fn restore(&mut self) {
        <RiveRenderer as RendererContract>::restore(&mut self.renderer);
    }

    fn transform(&mut self, transform: Mat2D) {
        <RiveRenderer as RendererContract>::transform(&mut self.renderer, &transform);
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        let Some(path) = path.as_any().downcast_ref::<RiveRenderPathHandle>() else {
            return;
        };
        let Some(paint) = paint.as_any().downcast_ref::<RiveRenderPaintHandle>() else {
            return;
        };
        unsafe {
            <RiveRenderer as RendererContract>::drawPath(
                &mut self.renderer,
                path.source_base() as *const _ as *mut _,
                paint.source_base() as *const _ as *mut _,
            );
        }
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        let Some(path) = path.as_any().downcast_ref::<RiveRenderPathHandle>() else {
            return;
        };
        unsafe {
            <RiveRenderer as RendererContract>::clipPath(
                &mut self.renderer,
                path.source_base() as *const _ as *mut _,
            );
        }
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        let Some(image) =
            image.and_then(|image| image.as_any().downcast_ref::<RiveRenderImageHandle>())
        else {
            return;
        };
        let Some(image_base) = image.source_base_for(&self.resource_domain) else {
            return;
        };
        unsafe {
            <RiveRenderer as RendererContract>::drawImage(
                &mut self.renderer,
                image_base as *const _,
                source_image_sampler(sampler),
                blend_mode,
                opacity,
            );
        }
    }

    fn draw_image_mesh(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv_coords: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        let Some(image) =
            image.and_then(|image| image.as_any().downcast_ref::<RiveRenderImageHandle>())
        else {
            return;
        };
        let Some(vertices) =
            vertices.and_then(|buffer| buffer.as_any().downcast_ref::<RiveRenderBufferHandle>())
        else {
            return;
        };
        let Some(uv_coords) =
            uv_coords.and_then(|buffer| buffer.as_any().downcast_ref::<RiveRenderBufferHandle>())
        else {
            return;
        };
        let Some(indices) =
            indices.and_then(|buffer| buffer.as_any().downcast_ref::<RiveRenderBufferHandle>())
        else {
            return;
        };
        let Some(image_base) = image.source_base_for(&self.resource_domain) else {
            return;
        };
        if !vertices.belongs_to(&self.resource_domain)
            || !uv_coords.belongs_to(&self.resource_domain)
            || !indices.belongs_to(&self.resource_domain)
        {
            return;
        }
        unsafe {
            <RiveRenderer as RendererContract>::drawImageMesh(
                &mut self.renderer,
                image_base as *const _,
                source_image_sampler(sampler),
                vertices.source_owner_unchecked(),
                uv_coords.source_owner_unchecked(),
                indices.source_owner_unchecked(),
                vertex_count,
                index_count,
                blend_mode,
                opacity,
            );
        }
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        <RiveRenderer as RendererContract>::modulateOpacity(&mut self.renderer, opacity);
    }
}

#[cfg(any())]
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
        let upload_completion = self.transfer_upload_ownership()?;
        let completion = NativeMetalContext::commit_with_upload_completion(
            &self.command_buffer,
            upload_completion,
        );
        if self.collect_work_metrics {
            self.backend_work.queue_submissions = 1;
        }
        completion.wait()?;

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
        let mut target = self.target.borrow_mut();
        let width = target.width();
        let height = target.height();
        let texture = target.retained_target_texture().ok_or_else(|| {
            RendererError::NativeMetal("native Metal target has no readback texture".into())
        })?;
        if let Some(flush) = atlas_flush.as_ref() {
            let tessellation_height = super::draw::tessellation_texture_height(&flush.spans);
            let upload_data = AtlasUploadData::new(width, height, flush);
            let lease = self.context.prepare_resources(
                0,
                tessellation_height as usize,
                Some(flush.physical_extent.map(|value| value as usize)),
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
        let encoder = if let Some(flush) = atlas_flush.as_ref() {
            draw_pass::make_render_pass_for_draws(
                &self.context,
                &self.command_buffer,
                &pass,
                &mut target,
                self.resource_lease
                    .as_ref()
                    .expect("atlas resource passes prepared a lease"),
                draw_pass::DrawPassDescriptor {
                    width,
                    height,
                    binding_plan: draw_pass::DrawPassPlanInput {
                        interlock_mode: shader_compile_plan::InterlockMode::RasterOrdering,
                        baseline_shader_misc_flags:
                            shader_compile_plan::FIXED_FUNCTION_COLOR_OUTPUT,
                        path_count: flush.draws.len(),
                        contour_count: flush.contours.len(),
                    },
                    wireframe: false,
                },
            )?
        } else if let Some(draw) = self.gradient_draws.first() {
            draw_pass::make_render_pass_for_draws(
                &self.context,
                &self.command_buffer,
                &pass,
                &mut target,
                self.resource_lease
                    .as_ref()
                    .expect("gradient resource passes prepared a lease"),
                draw_pass::DrawPassDescriptor {
                    width,
                    height,
                    binding_plan: draw_pass::DrawPassPlanInput {
                        interlock_mode: shader_compile_plan::InterlockMode::RasterOrdering,
                        baseline_shader_misc_flags:
                            shader_compile_plan::FIXED_FUNCTION_COLOR_OUTPUT,
                        path_count: 1,
                        contour_count: draw.tessellation.contours.len(),
                    },
                    wireframe: false,
                },
            )?
        } else {
            self.command_buffer
                .renderCommandEncoderWithDescriptor(&pass)
                .ok_or_else(|| {
                    RendererError::NativeMetal("failed to create render command encoder".into())
                })?
        };
        if let Some(flush) = atlas_flush.as_ref() {
            encode_atlas_final_draw(
                &self.context,
                &encoder,
                target.pixel_format(),
                flush,
                self.resource_lease
                    .as_ref()
                    .expect("atlas resource passes prepared a lease"),
            )?;
        } else if let Some(draw) = self.gradient_draws.first() {
            encode_gradient_final_draw(&self.context, &encoder, target.pixel_format(), draw)?;
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

impl NativeMetalFrame {
    pub fn finish(self) -> Result<Vec<u8>, RendererError> {
        Ok(self.finish_for_benchmark()?.pixels)
    }

    pub fn finish_for_benchmark(self) -> Result<NativeMetalFrameOutput, RendererError> {
        let source_mode = self.mechanical.borrow().mode();
        let completion = self
            .mechanical
            .borrow_mut()
            .finish(self.frame_number, self.frame_number)?;
        completion.wait()?;
        let source_metrics = self.mechanical.borrow().execution_inventory();
        let (width, height, texture) = {
            let mechanical = self.mechanical.borrow();
            let (width, height) = mechanical.dimensions();
            let texture = mechanical.retained_target_texture().ok_or_else(|| {
                RendererError::NativeMetal("mechanical target has no readback texture".into())
            })?;
            (width, height, texture)
        };
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
        let mut backend_work = BackendWorkMetrics::default();
        if self.collect_work_metrics {
            backend_work.queue_submissions = 1;
        }
        Ok(NativeMetalFrameOutput {
            pixels,
            backend_work,
            execution_inventory: NativeMetalExecutionInventory::from_execution(
                source_mode,
                source_metrics,
            ),
        })
    }

    pub(super) fn finish_present(
        &mut self,
        drawable: &ProtocolObject<dyn objc2_metal::MTLDrawable>,
    ) -> Result<(), RendererError> {
        let completion = self.mechanical.borrow_mut().finish_present(
            self.frame_number,
            self.frame_number,
            drawable,
        )?;
        completion.wait()?;
        Ok(())
    }
}

impl Drop for NativeMetalFrame {
    fn drop(&mut self) {
        if self.mechanical.borrow().is_active_frame() {
            self.mechanical.borrow_mut().abandon_frame();
        }
    }
}

#[cfg(any())]
fn encode_atomic_tessellation_pass(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    flush: &PreparedAtomicPathFlush,
    lease: &context::PreparedResourceLease,
) -> Result<(), RendererError> {
    let Some(tessellate_pipeline) = context.tessellate_pipeline() else {
        return Ok(());
    };
    let (Some(gaussian_integral_texture), Some(tess_span_index_buffer)) = (
        context.gaussian_integral_texture(),
        context.tess_span_index_buffer(),
    ) else {
        return Ok(());
    };
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
    encoder.setRenderPipelineState(tessellate_pipeline);
    // SAFETY: slot 9 is the generated `gaussianIntegralTexture` vertex binding
    // and the context retains the texture through command-buffer completion.
    unsafe {
        encoder.setVertexTexture_atIndex(Some(gaussian_integral_texture), 9);
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
                tess_span_index_buffer,
                0,
                flush.spans.len(),
            );
    }
    encoder.endEncoding();
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
struct AtomicBarrierInventory {
    semantic: usize,
    memory: usize,
    render_pass_breaks: usize,
}

#[cfg(test)]
fn atomic_barrier_inventory(
    draw_group_count: usize,
    barrier_type: AtomicBarrierType,
) -> AtomicBarrierInventory {
    let semantic = draw_group_count.saturating_add(1);
    AtomicBarrierInventory {
        semantic,
        memory: usize::from(barrier_type == AtomicBarrierType::memoryBarrier) * semantic,
        render_pass_breaks: usize::from(barrier_type == AtomicBarrierType::renderPassBreak)
            * semantic,
    }
}

#[cfg(any())]
fn make_atomic_main_encoder(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    pass: &MTLRenderPassDescriptor,
    target: &mut RenderTargetMetal,
    width: u32,
    height: u32,
    lease: &context::PreparedResourceLease,
    flush: &PreparedAtomicPathFlush,
) -> Result<Retained<ProtocolObject<dyn MTLRenderCommandEncoder>>, RendererError> {
    let encoder = draw_pass::make_render_pass_for_draws(
        context,
        command_buffer,
        pass,
        target,
        lease,
        draw_pass::DrawPassDescriptor {
            width,
            height,
            binding_plan: draw_pass::DrawPassPlanInput {
                interlock_mode: shader_compile_plan::InterlockMode::Atomics,
                baseline_shader_misc_flags: if flush.uses_advanced_blend {
                    0
                } else {
                    shader_compile_plan::FIXED_FUNCTION_COLOR_OUTPUT
                },
                path_count: flush.paths.len(),
                contour_count: flush.contours.len(),
            },
            wireframe: false,
        },
    )?;
    // Upstream binds the selected image sampler per draw batch after creating
    // the common pass. Current atomic batches carry no images, so every batch
    // selects the same linear-clamp fallback at the generated image slot.
    // SAFETY: slot 11 is the generated image-sampler ABI, and the context
    // retains the sampler through command completion.
    unsafe {
        encoder
            .setFragmentSamplerState_atIndex(context.image_sampler(ImageSampler::LINEAR_CLAMP), 11);
    }
    Ok(encoder)
}

#[cfg(any())]
fn apply_atomic_barrier(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    pass: &MTLRenderPassDescriptor,
    target: &mut RenderTargetMetal,
    width: u32,
    height: u32,
    lease: &context::PreparedResourceLease,
    flush: &PreparedAtomicPathFlush,
    encoder: Retained<ProtocolObject<dyn MTLRenderCommandEncoder>>,
    render_passes: &mut u64,
) -> Result<Retained<ProtocolObject<dyn MTLRenderCommandEncoder>>, RendererError> {
    match context.capabilities().atomic_barrier_type {
        AtomicBarrierType::rasterOrderGroup => Ok(encoder),
        AtomicBarrierType::memoryBarrier => {
            encoder.memoryBarrierWithScope_afterStages_beforeStages(
                MTLBarrierScope::Buffers | MTLBarrierScope::RenderTargets,
                MTLRenderStages::Fragment,
                MTLRenderStages::Fragment,
            );
            Ok(encoder)
        }
        AtomicBarrierType::renderPassBreak => {
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
                flush,
            )
        }
    }
}

#[cfg(any())]
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
    let (Some(path_patch_vertex_buffer), Some(path_patch_index_buffer)) = (
        context.path_patch_vertex_buffer(),
        context.path_patch_index_buffer(),
    ) else {
        return Ok(0);
    };
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
        flush,
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
        flush,
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
                flush,
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
            bind_vertex_buffer(&encoder, path_patch_vertex_buffer, 0);
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
                    path_patch_index_buffer,
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
        flush,
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

#[cfg(any())]
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

#[cfg(any())]
fn configure_raster_order_attachments(
    pass: &MTLRenderPassDescriptor,
    target: &RenderTargetMetal,
) -> Result<(), RendererError> {
    let textures = [
        target.clip_memoryless_texture(),
        target.scratch_color_memoryless_texture(),
        target.coverage_memoryless_texture(),
    ];
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

#[cfg(any())]
fn encode_gradient_resource_passes(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    draw: &GradientDraw,
    lease: &context::PreparedResourceLease,
) -> Result<(), RendererError> {
    let (Some(gaussian_integral_texture), Some(tess_span_index_buffer)) = (
        context.gaussian_integral_texture(),
        context.tess_span_index_buffer(),
    ) else {
        return Ok(());
    };
    let tessellation_height = super::draw::tessellation_texture_height(&draw.tessellation.spans);

    encode_color_ramp_pass(context, command_buffer, &draw.gradient_batch, lease)?;

    let Some(tessellate_pipeline) = context.tessellate_pipeline() else {
        return Ok(());
    };

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
    tessellation_encoder.setRenderPipelineState(tessellate_pipeline);
    // SAFETY: texture slot 9 is the pinned tessellation-shader Gaussian-table
    // ABI, and the context-retained texture outlives command-buffer completion.
    unsafe {
        tessellation_encoder.setVertexTexture_atIndex(Some(gaussian_integral_texture), 9);
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
                tess_span_index_buffer,
                0,
                draw.tessellation.spans.len(),
            );
    }
    tessellation_encoder.endEncoding();
    Ok(())
}

#[cfg(any())]
fn encode_color_ramp_pass(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    gradient_batch: &super::logical_frame::GradientBatch,
    lease: &context::PreparedResourceLease,
) -> Result<(), RendererError> {
    let Some(color_ramp_pipeline) = context.color_ramp_pipeline() else {
        return Ok(());
    };
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
    gradient_encoder.setRenderPipelineState(color_ramp_pipeline);
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

#[cfg(any())]
fn encode_atlas_resource_passes(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    flush: &super::logical_frame::PreparedRasterOrderingAtlasFlush,
    lease: &context::PreparedResourceLease,
) -> Result<(), RendererError> {
    let (
        Some(gaussian_integral_texture),
        Some(tess_span_index_buffer),
        Some(path_patch_vertex_buffer),
        Some(path_patch_index_buffer),
    ) = (
        context.gaussian_integral_texture(),
        context.tess_span_index_buffer(),
        context.path_patch_vertex_buffer(),
        context.path_patch_index_buffer(),
    )
    else {
        return Ok(());
    };
    let Some(tessellate_pipeline) = context.tessellate_pipeline() else {
        return Ok(());
    };
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
    tessellation_encoder.setRenderPipelineState(tessellate_pipeline);
    // SAFETY: texture slot 9 is the exact generated tessellation ABI and the
    // context retains the Gaussian table through command-buffer completion.
    unsafe {
        tessellation_encoder.setVertexTexture_atIndex(Some(gaussian_integral_texture), 9);
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
                tess_span_index_buffer,
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
    let atlas_pipelines = lease.feather_atlas_pipelines.as_ref().ok_or_else(|| {
        RendererError::NativeMetal("feather atlas pipeline pair is absent".to_owned())
    })?;
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
        atlas_encoder.setVertexTexture_atIndex(Some(gaussian_integral_texture), 9);
        atlas_encoder.setFragmentTexture_atIndex(lease.gradient.as_deref(), 8);
        atlas_encoder.setFragmentTexture_atIndex(Some(gaussian_integral_texture), 9);
        atlas_encoder.setVertexBuffer_offset_atIndex(Some(path_patch_vertex_buffer), 0, 0);
    }
    atlas_encoder.setCullMode(MTLCullMode::None);
    atlas_encoder.setRenderPipelineState(&atlas_pipelines.fill);
    for batch in &flush.fill_batches {
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
                    gpu::MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT,
                    MTLIndexType::UInt16,
                    path_patch_index_buffer,
                    gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT * std::mem::size_of::<u16>(),
                    batch.patch_count as usize,
                );
        }
    }
    atlas_encoder.setCullMode(MTLCullMode::Back);
    atlas_encoder.setRenderPipelineState(&atlas_pipelines.stroke);
    for batch in &flush.stroke_batches {
        let [left, top, right, bottom] = batch.scissor;
        atlas_encoder.setScissorRect(MTLScissorRect {
            x: left as usize,
            y: top as usize,
            width: usize::from(right - left),
            height: usize::from(bottom - top),
        });
        set_vertex_bytes(&atlas_encoder, &batch.base_patch, 4)?;
        // SAFETY: the static patch index buffer contains the exact stroke
        // border range, and the canonical batch supplies a validated base
        // patch and patch count into the flush-wide tessellation texture.
        unsafe {
            atlas_encoder.drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount(
                    MTLPrimitiveType::Triangle,
                    gpu::MIDPOINT_FAN_PATCH_BORDER_INDEX_COUNT,
                    MTLIndexType::UInt16,
                    path_patch_index_buffer,
                    0,
                    batch.patch_count as usize,
                );
        }
    }
    atlas_encoder.endEncoding();
    Ok(())
}

#[cfg(any())]
fn encode_atlas_final_draw(
    context: &NativeMetalContext,
    encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
    pixel_format: MTLPixelFormat,
    flush: &super::logical_frame::PreparedRasterOrderingAtlasFlush,
    lease: &context::PreparedResourceLease,
) -> Result<(), RendererError> {
    let atlas_blit_pipeline = context.atlas_blit_pipeline(pixel_format)?;
    encoder.setRenderPipelineState(&atlas_blit_pipeline);
    let triangles = lease.triangles.as_deref().ok_or_else(|| {
        RendererError::NativeMetal("atlas blit triangle upload is absent".to_owned())
    })?;
    bind_vertex_buffer(encoder, triangles, 0);
    lease.feather_atlas.as_deref().ok_or_else(|| {
        RendererError::NativeMetal("feather atlas texture is absent for final blit".to_owned())
    })?;
    // SAFETY: slot 11 is the generated image-sampler ABI. Common textures and
    // buffers were bound once by `make_render_pass_for_draws`, and the context
    // retains this per-batch default sampler through command completion.
    unsafe {
        encoder
            .setFragmentSamplerState_atIndex(context.image_sampler(ImageSampler::LINEAR_CLAMP), 11);
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

#[cfg(any())]
fn encode_gradient_final_draw(
    context: &NativeMetalContext,
    encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
    pixel_format: MTLPixelFormat,
    draw: &GradientDraw,
) -> Result<(), RendererError> {
    let (Some(path_patch_vertex_buffer), Some(path_patch_index_buffer)) = (
        context.path_patch_vertex_buffer(),
        context.path_patch_index_buffer(),
    ) else {
        return Ok(());
    };
    let midpoint_pipeline = context.midpoint_draw_pipeline(pixel_format)?;
    encoder.setRenderPipelineState(&midpoint_pipeline);
    // SAFETY: buffer slot zero is the pinned path-patch vertex ABI, offset zero
    // is aligned, and the context retains the complete buffer through completion.
    unsafe {
        encoder.setVertexBuffer_offset_atIndex(Some(path_patch_vertex_buffer), 0, 0);
    }
    // The first eight tessellation texels are a padding patch. Match
    // RenderContextMetalImpl's PATH_BASE_INSTANCE_UNIFORM_BUFFER_IDX binding
    // so instance zero starts at `base_instance == 1`, rather than joining the
    // zero-filled padding texel at the origin to the authored contour.
    set_vertex_bytes(encoder, &draw.tessellation.base_instance, 4)?;
    // SAFETY: slot 11 is the generated image-sampler ABI. Common textures and
    // buffers were bound once by `make_render_pass_for_draws`, and the context
    // retains this per-batch default sampler through command completion.
    unsafe {
        encoder
            .setFragmentSamplerState_atIndex(context.image_sampler(ImageSampler::LINEAR_CLAMP), 11);
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
                path_patch_index_buffer,
                0,
                draw.tessellation.instance_count as usize,
            );
    }
    Ok(())
}

#[cfg(any())]
fn set_vertex_bytes<T: Pod>(
    encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
    value: &T,
    index: usize,
) -> Result<(), RendererError> {
    set_vertex_slice(encoder, std::slice::from_ref(value), index)
}

#[cfg(any())]
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

#[cfg(any())]
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

#[cfg(any())]
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

/// Creates the exact target texture handed to the persistent mechanical
/// RenderContext. Target attachment and retirement are owned by that source
/// context; the factory retains only this generation handle for resize and
/// drawable restoration.
fn make_native_target_texture(
    device: &ProtocolObject<dyn MTLDevice>,
    width: u32,
    height: u32,
) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, RendererError> {
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
    descriptor.setUsage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
    device.newTextureWithDescriptor(&descriptor)
        .ok_or_else(|| {
            RendererError::NativeMetal("failed to allocate native target texture".into())
        })
}

fn source_image_sampler(
    value: ImageSampler,
) -> crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler {
    use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::{
        ImageFilter, ImageSampler as SourceImageSampler, ImageWrap,
    };
    SourceImageSampler {
        wrapX: match value.wrap_x {
            nuxie_render_api::ImageWrap::Clamp => ImageWrap::clamp,
            nuxie_render_api::ImageWrap::Repeat => ImageWrap::repeat,
            nuxie_render_api::ImageWrap::Mirror => ImageWrap::mirror,
        },
        wrapY: match value.wrap_y {
            nuxie_render_api::ImageWrap::Clamp => ImageWrap::clamp,
            nuxie_render_api::ImageWrap::Repeat => ImageWrap::repeat,
            nuxie_render_api::ImageWrap::Mirror => ImageWrap::mirror,
        },
        filter: match value.filter {
            nuxie_render_api::ImageFilter::Bilinear => ImageFilter::bilinear,
            nuxie_render_api::ImageFilter::Nearest => ImageFilter::nearest,
        },
    }
}

#[cfg(any())]
fn clear_color(color: u32) -> objc2_metal::MTLClearColor {
    let [alpha, red, green, blue] = color.to_be_bytes();
    let premultiply = |channel: u8| f64::from(u16::from(channel) * u16::from(alpha) / 255) / 255.0;
    objc2_metal::MTLClearColor {
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
fn inline_triangle_fan_vertex_count(point_count: usize) -> Option<usize> {
    let vertex_count = point_count.checked_sub(2)?.checked_mul(3)?;
    let vertex_bytes = vertex_count.checked_mul(std::mem::size_of::<[f32; 2]>())?;
    (point_count >= 3 && vertex_bytes <= INLINE_VERTEX_BYTE_LIMIT).then_some(vertex_count)
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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
    #[cfg(all(target_os = "visionos", target_abi = "sim"))]
    let platform = ApplePlatform::XrOsSimulator;
    #[cfg(all(target_os = "visionos", not(target_abi = "sim")))]
    let platform = ApplePlatform::XrOsDevice {
        is_apple_silicon: device.supportsFamily(MTLGPUFamily::Apple4),
    };
    #[cfg(all(target_os = "tvos", target_abi = "sim"))]
    let platform = ApplePlatform::AppleTvOsSimulator;
    #[cfg(all(target_os = "tvos", not(target_abi = "sim")))]
    let platform = ApplePlatform::AppleTvOsDevice {
        is_apple_silicon: device.supportsFamily(MTLGPUFamily::Apple4),
    };
    #[cfg(any(
        target_os = "macos",
        all(target_os = "ios", target_abi = "sim"),
        all(target_os = "visionos", target_abi = "sim"),
        all(target_os = "tvos", target_abi = "sim")
    ))]
    let _ = device;
    platform
}

#[cfg(test)]
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

#[cfg(any())]
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

#[cfg(test)]
pub(crate) struct MetalObjectCreation<T> {
    pub(crate) object: Option<T>,
    pub(crate) error: Option<String>,
}

#[cfg(test)]
pub(crate) fn new_render_pipeline_state(
    device: &ProtocolObject<dyn MTLDevice>,
    descriptor: &MTLRenderPipelineDescriptor,
) -> MetalObjectCreation<Retained<ProtocolObject<dyn MTLRenderPipelineState>>> {
    let mut error: Option<Retained<NSError>> = None;
    let object: Option<Retained<ProtocolObject<dyn MTLRenderPipelineState>>> = unsafe {
        msg_send![device,
            newRenderPipelineStateWithDescriptor: descriptor,
            error: &mut error
        ]
    };
    MetalObjectCreation {
        object,
        error: error.map(|error| error.localizedDescription().to_string()),
    }
}

#[cfg(test)]
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
    let mut error: Option<Retained<NSError>> = None;
    let library: Option<Retained<ProtocolObject<dyn MTLLibrary>>> =
        unsafe { msg_send![device, newLibraryWithData: data, error: &mut error] };
    unsafe { dispatch_release(data) };
    let error = error.map(|error| error.localizedDescription().to_string());
    if error.is_some() || library.is_none() {
        return Err(RendererError::NativeMetal(format!(
            "load tracer metallib: {}",
            error.as_deref().unwrap_or("<nil>")
        )));
    }
    Ok(library.expect("library checked nonnil"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_render_context_drops_members_in_reverse_then_base() {
        use crate::mechanical_port::source::renderer::src::render_context_cpp::takeRenderContextDropTrace;

        let _ = takeRenderContextDropTrace();
        let factory = NativeMetalFactory::new(2, 2).expect("create native Metal factory");
        drop(factory);

        let mut expected = vec![
            "logicalFlushes",
            "parametricAllocator",
            "polarAllocator",
            "tangentAllocator",
            "chopAllocator",
            "numChopsAllocator",
            "perFrameAllocator",
            "imageDrawData",
            "triangleData",
            "tessData",
            "gradientData",
            "contourData",
            "paintAuxData",
            "paintData",
            "pathData",
            "flushUniformData",
            "scissorLookup",
            "intersectionBoard",
            "indirectDrawList",
        ];
        #[cfg(feature = "native-ore-metal-experimental")]
        expected.push("oreContext");
        expected.extend(["implementation", "base"]);
        assert_eq!(takeRenderContextDropTrace(), expected);
    }

    #[cfg(feature = "native-ore-metal-experimental")]
    #[test]
    fn public_source_canvas_uses_one_shared_texture_and_releases_it_with_the_canvas() {
        use objc2::rc::Weak;

        let factory = NativeMetalFactory::new(4, 4).expect("create native Metal factory");
        let weak_texture = objc2::rc::autoreleasepool(|_| {
            let canvas = factory
                .make_metal_render_canvas(4, 4)
                .expect("construct the complete source RenderCanvas product path");
            assert_eq!((canvas.width(), canvas.height()), (4, 4));
            assert!(canvas.render_target_and_image_share_texture());
            let texture = canvas
                .retained_metal_texture()
                .expect("source image and target share a live native texture");
            assert_eq!((texture.width(), texture.height()), (4, 4));
            let weak = Weak::new(&*texture);
            drop(texture);
            assert!(weak.load().is_some());
            drop(canvas);
            weak
        });
        assert!(
            weak_texture.load().is_none(),
            "the public canvas path must not leak a third texture owner"
        );
    }

    #[cfg(any())]
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn factory_exposes_one_retained_ordered_metal_queue() {
        let factory = NativeMetalFactory::new(2, 2).expect("create native Metal factory");
        let first = factory.retained_metal_queue().expect("queue is set");
        let second = factory.retained_metal_queue().expect("queue is set");

        assert_eq!(Retained::as_ptr(&first), Retained::as_ptr(&second));
        assert_eq!(
            Retained::as_ptr(&first.device()),
            Retained::as_ptr(&factory.retained_metal_device())
        );
    }

    #[cfg(any())]
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn factory_retains_the_exact_injected_metal_device() {
        let device = MTLCreateSystemDefaultDevice().expect("system Metal device");
        let identity = Retained::as_ptr(&device);
        let factory = NativeMetalFactory::new_with_device_and_context_options(
            2,
            2,
            device,
            NativeMetalContextOptions::default(),
        )
        .expect("create factory from the injected Metal device");

        assert_eq!(Retained::as_ptr(&factory.retained_metal_device()), identity);
        assert_eq!(
            Retained::as_ptr(
                &factory
                    .retained_metal_queue()
                    .expect("queue is set")
                    .device()
            ),
            identity
        );
    }

    #[cfg(any())]
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
    #[cfg(any())]
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
            atomic_barrier_type: AtomicBarrierType::rasterOrderGroup,
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
            atomic_barrier_inventory(4, AtomicBarrierType::rasterOrderGroup),
            AtomicBarrierInventory {
                semantic: 5,
                memory: 0,
                render_pass_breaks: 0,
            }
        );
        assert_eq!(
            atomic_barrier_inventory(4, AtomicBarrierType::memoryBarrier),
            AtomicBarrierInventory {
                semantic: 5,
                memory: 5,
                render_pass_breaks: 0,
            }
        );
        assert_eq!(
            atomic_barrier_inventory(4, AtomicBarrierType::renderPassBreak),
            AtomicBarrierInventory {
                semantic: 5,
                memory: 0,
                render_pass_breaks: 5,
            }
        );
        assert_eq!(
            1 + atomic_barrier_inventory(4, AtomicBarrierType::renderPassBreak).render_pass_breaks,
            6,
            "initial main pass plus one replacement pass per semantic barrier"
        );
    }
}
