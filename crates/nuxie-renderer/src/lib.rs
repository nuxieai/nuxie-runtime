//! Pure-Rust WebGPU and WebGL2 renderers behind the `nuxie-render-api` trait
//! boundary.

#[cfg(test)]
mod atlas_blit_oracle;
#[cfg(test)]
mod atlas_input_oracle;
#[cfg(test)]
mod atlas_mask_oracle;
mod atlas_pipeline;
#[cfg(test)]
mod atlas_placement_oracle;
mod atomic_pipeline;
mod clockwise_atomic_pipeline;
mod composite_pipeline;
#[cfg(test)]
mod direct_grid_oracle;
mod draw;
mod feather_lut;
mod gpu;
mod gr_triangulator;
mod gradient_pipeline;
// Kept standalone until a renderer path has a proven grouping integration.
#[cfg(target_arch = "wasm32")]
mod browser;
#[allow(dead_code)]
mod intersection_board;
mod logical_flush;
mod mipmap_pipeline;
mod msaa_atlas_pipeline;
mod msaa_image_mesh_pipeline;
mod msaa_stencil_pipeline;
mod path_pipeline;
mod skyline;
#[cfg(test)]
mod tess_span_oracle;
mod tessellator;
#[cfg(target_arch = "wasm32")]
mod webgl2;
mod work_metrics;

use bytemuck::{Pod, Zeroable};
use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageSampler, Mat2D, PathVerb,
    RawPath, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint,
    RenderPaintStyle, RenderPath, RenderShader, Renderer, StrokeCap, StrokeJoin, Vec2D,
};
use std::any::Any;
use std::cell::RefCell;
use std::error::Error;
#[cfg(target_os = "macos")]
use std::ffi::c_void;
use std::fmt;
use std::io::Cursor;
use std::sync::{Arc, Mutex, Weak};
use work_metrics::{CountedCommandEncoderExt, CountedDeviceExt, CountedQueueExt};

#[derive(Debug)]
pub enum RendererError {
    Adapter(String),
    AtlasPacking(&'static str),
    Device(String),
    InvalidTextureExtent {
        label: &'static str,
        width: u32,
        height: u32,
        max_dimension: u32,
    },
    Map(String),
    Unsupported(&'static str),
    WebGl2(String),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adapter(message) => write!(f, "wgpu adapter error: {message}"),
            Self::AtlasPacking(message) => write!(f, "atlas packing error: {message}"),
            Self::Device(message) => write!(f, "wgpu device error: {message}"),
            Self::InvalidTextureExtent {
                label,
                width,
                height,
                max_dimension,
            } => write!(
                f,
                "invalid {label} texture extent {width}x{height}; dimensions must be between 1 and {max_dimension}"
            ),
            Self::Map(message) => write!(f, "wgpu readback error: {message}"),
            Self::Unsupported(feature) => write!(f, "unsupported renderer feature: {feature}"),
            Self::WebGl2(message) => write!(f, "WebGL2 renderer error: {message}"),
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{BrowserBackend, BrowserBackendPreference, BrowserFactory, BrowserFrame};
#[cfg(target_arch = "wasm32")]
pub use webgl2::{WebGl2Factory, WebGl2Frame};

impl Error for RendererError {}

const MAX_ATOMIC_PATHS: usize = logical_flush::MAX_PATH_COUNT;
// RenderContextWebGPUImpl retains PlatformFeatures' default texture limit.
const CPP_WEBGPU_PLATFORM_MAX_TEXTURE_DIMENSION: u32 = 2048;
// RenderContext::atlasMaxSize applies an additional 4096 cap.
const CPP_LOGICAL_ATLAS_MAX_DIMENSION: u32 = 4096;
const FEATHER_ATLAS_PADDING: u32 = 2;
// A single Metal command buffer first fails at 2,044 direct MSAA draws with
// the current per-draw tessellation resources. Reuse that twofold safety
// fence while the shared C++ logical-flush resource layout is translated.
const MAX_DRAWS_PER_SUBMISSION: usize = 1_024;
const MAX_CACHED_FRAME_ATTACHMENTS: usize = 1;

fn texture_extent_supported(width: u32, height: u32, max_dimension: u32) -> bool {
    width != 0 && height != 0 && width <= max_dimension && height <= max_dimension
}

fn validate_texture_extent(
    label: &'static str,
    width: u32,
    height: u32,
    max_dimension: u32,
) -> Result<(), RendererError> {
    if texture_extent_supported(width, height, max_dimension) {
        Ok(())
    } else {
        Err(RendererError::InvalidTextureExtent {
            label,
            width,
            height,
            max_dimension,
        })
    }
}

fn validate_atomic_path_count(path_count: usize) -> Result<(), RendererError> {
    if path_count <= MAX_ATOMIC_PATHS {
        Ok(())
    } else {
        Err(RendererError::Unsupported(
            "atomic runs exceed the C++ logical-flush path budget",
        ))
    }
}

struct Context {
    device: wgpu::Device,
    queue: wgpu::Queue,
    frame_attachments: FrameAttachmentPool,
    adapter_info: WgpuAdapterInfo,
    non_zero_stencil_pipeline: wgpu::RenderPipeline,
    even_odd_stencil_pipeline: wgpu::RenderPipeline,
    cover_pipeline: wgpu::RenderPipeline,
    patch_vertex_buffer: wgpu::Buffer,
    patch_index_buffer: wgpu::Buffer,
    tessellator: tessellator::Tessellator,
    path_pipeline: path_pipeline::PathPipeline,
    atomic_pipeline: atomic_pipeline::AtomicPipeline,
    clockwise_atomic_pipeline: clockwise_atomic_pipeline::ClockwiseAtomicPipeline,
    atlas_pipeline: atlas_pipeline::AtlasPipeline,
    composite_pipeline: composite_pipeline::CompositePipeline,
    gradient_pipeline: gradient_pipeline::GradientPipeline,
    mipmap_pipeline: mipmap_pipeline::MipmapPipeline,
    msaa_atlas_pipeline: msaa_atlas_pipeline::MsaaAtlasPipeline,
    msaa_image_mesh_pipeline: msaa_image_mesh_pipeline::MsaaImageMeshPipeline,
    msaa_stencil_pipeline: msaa_stencil_pipeline::MsaaStencilPipeline,
    feather_lut: feather_lut::FeatherLut,
}

struct FrameAttachments {
    target_texture: wgpu::Texture,
    target_view: wgpu::TextureView,
    multisample_view: wgpu::TextureView,
    stencil_view: wgpu::TextureView,
}

impl FrameAttachments {
    fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let target_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-offscreen-target"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let target_view = target_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let multisample_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-multisample-target"),
            size,
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let multisample_view =
            multisample_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let stencil_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-stencil-target"),
            size,
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let stencil_view = stencil_texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            target_texture,
            target_view,
            multisample_view,
            stencil_view,
        }
    }
}

struct FrameAttachmentPool {
    width: u32,
    height: u32,
    available: Mutex<Vec<Arc<FrameAttachments>>>,
}

impl FrameAttachmentPool {
    fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            available: Mutex::new(vec![Arc::new(FrameAttachments::new(device, width, height))]),
        }
    }

    fn checkout(&self, device: &wgpu::Device) -> Arc<FrameAttachments> {
        self.available
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pop()
            .unwrap_or_else(|| Arc::new(FrameAttachments::new(device, self.width, self.height)))
    }

    fn recycle(&self, attachments: Arc<FrameAttachments>) {
        let mut available = self
            .available
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if available.len() < MAX_CACHED_FRAME_ATTACHMENTS {
            available.push(attachments);
        }
    }

    #[cfg(test)]
    fn cached(&self) -> Arc<FrameAttachments> {
        Arc::clone(
            self.available
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .last()
                .expect("frame attachment pool is empty outside frame execution"),
        )
    }

    #[cfg(test)]
    fn cached_len(&self) -> usize {
        self.available
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}

pub struct WgpuFactory {
    context: Arc<Context>,
    width: u32,
    height: u32,
    mode: RenderMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WgpuAdapterInfo {
    pub backend: String,
    pub name: String,
    pub vendor: u32,
    pub device: u32,
    pub driver: String,
    pub driver_info: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WgpuFrameMetrics {
    pub draw_calls: u64,
    pub logical_flushes: u64,
    pub atomic_strategy_partitions: u64,
    pub backend_work: BackendWorkMetrics,
}

pub use work_metrics::BackendWorkMetrics;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Msaa,
    ClockwiseAtomic,
}

impl WgpuFactory {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(width: u32, height: u32) -> Result<Self, RendererError> {
        Self::new_with_mode(width, height, RenderMode::Msaa)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_with_mode(width: u32, height: u32, mode: RenderMode) -> Result<Self, RendererError> {
        pollster::block_on(Self::new_async_with_mode(width, height, mode))
    }

    /// Asynchronously creates an MSAA renderer.
    ///
    /// This is the browser-safe constructor and does not block while requesting
    /// the WebGPU adapter or device.
    pub async fn new_async(width: u32, height: u32) -> Result<Self, RendererError> {
        Self::new_async_with_mode(width, height, RenderMode::Msaa).await
    }

    /// Asynchronously creates a renderer for the requested draw mode.
    pub async fn new_async_with_mode(
        width: u32,
        height: u32,
        mode: RenderMode,
    ) -> Result<Self, RendererError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .map_err(|error| RendererError::Adapter(error.to_string()))?;
        let adapter_info = adapter.get_info();
        let adapter_info = WgpuAdapterInfo {
            backend: format!("{:?}", adapter_info.backend).to_ascii_lowercase(),
            name: adapter_info.name,
            vendor: adapter_info.vendor,
            device: adapter_info.device,
            driver: adapter_info.driver,
            driver_info: adapter_info.driver_info,
        };
        let adapter_limits = adapter.limits();
        validate_texture_extent(
            "render target",
            width,
            height,
            adapter_limits.max_texture_dimension_2d,
        )?;
        let required_features = adapter.features() & wgpu::Features::CLIP_DISTANCES;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("nuxie-renderer-device"),
                required_features,
                required_limits: wgpu::Limits {
                    max_storage_buffers_per_shader_stage: 7,
                    max_texture_dimension_2d: adapter_limits.max_texture_dimension_2d,
                    ..wgpu::Limits::downlevel_defaults()
                },
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|error| RendererError::Device(error.to_string()))?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nuxie-solid-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("solid.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nuxie-solid-pipeline-layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let vertex_buffer_layouts = [Some(Vertex::layout())];
        let pipeline_descriptor = |label, fragment, depth_stencil| wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &vertex_buffer_layouts,
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil,
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment,
            multiview_mask: None,
            cache: None,
        };
        let stencil_face = |pass_op| wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Always,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op,
        };
        let stencil_state = |front, back| wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::Always),
            stencil: wgpu::StencilState {
                front,
                back,
                read_mask: 0xff,
                write_mask: 0xff,
            },
            bias: wgpu::DepthBiasState::default(),
        };
        let stencil_targets = [Some(wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Rgba8Unorm,
            blend: None,
            write_mask: wgpu::ColorWrites::empty(),
        })];
        let stencil_fragment = || wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fragment_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &stencil_targets,
        };
        let non_zero_stencil_pipeline = device.create_render_pipeline(&pipeline_descriptor(
            "nuxie-non-zero-stencil-pipeline",
            Some(stencil_fragment()),
            Some(stencil_state(
                stencil_face(wgpu::StencilOperation::IncrementWrap),
                stencil_face(wgpu::StencilOperation::DecrementWrap),
            )),
        ));
        let even_odd_stencil_pipeline = device.create_render_pipeline(&pipeline_descriptor(
            "nuxie-even-odd-stencil-pipeline",
            Some(stencil_fragment()),
            Some(stencil_state(
                stencil_face(wgpu::StencilOperation::Invert),
                stencil_face(wgpu::StencilOperation::Invert),
            )),
        ));
        let cover_stencil_face = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::NotEqual,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Zero,
        };
        let cover_pipeline = device.create_render_pipeline(&pipeline_descriptor(
            "nuxie-cover-pipeline",
            Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            Some(stencil_state(cover_stencil_face, cover_stencil_face)),
        ));
        let (patch_vertices, patch_indices) = gpu::generate_patch_buffer_data();
        let patch_vertex_buffer =
            device.create_counted_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("nuxie-patch-vertices"),
                contents: bytemuck::cast_slice(&patch_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let patch_index_buffer =
            device.create_counted_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("nuxie-patch-indices"),
                contents: bytemuck::cast_slice(&patch_indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let tessellator = tessellator::Tessellator::new(&device);
        let path_pipeline = path_pipeline::PathPipeline::new(&device);
        let atomic_pipeline = atomic_pipeline::AtomicPipeline::new(&device);
        let clockwise_atomic_pipeline =
            clockwise_atomic_pipeline::ClockwiseAtomicPipeline::new(&device);
        let atlas_pipeline = atlas_pipeline::AtlasPipeline::new(&device);
        let composite_pipeline = composite_pipeline::CompositePipeline::new(&device);
        let gradient_pipeline = gradient_pipeline::GradientPipeline::new(&device);
        let mipmap_pipeline = mipmap_pipeline::MipmapPipeline::new(&device);
        let msaa_atlas_pipeline = msaa_atlas_pipeline::MsaaAtlasPipeline::new(&device);
        let msaa_image_mesh_pipeline =
            msaa_image_mesh_pipeline::MsaaImageMeshPipeline::new(&device);
        let msaa_stencil_pipeline = msaa_stencil_pipeline::MsaaStencilPipeline::new(&device);
        let feather_lut = feather_lut::FeatherLut::new(&device, &queue);
        let frame_attachments = FrameAttachmentPool::new(&device, width, height);
        Ok(Self {
            context: Arc::new(Context {
                device,
                queue,
                frame_attachments,
                adapter_info,
                non_zero_stencil_pipeline,
                even_odd_stencil_pipeline,
                cover_pipeline,
                patch_vertex_buffer,
                patch_index_buffer,
                tessellator,
                path_pipeline,
                atomic_pipeline,
                clockwise_atomic_pipeline,
                atlas_pipeline,
                composite_pipeline,
                gradient_pipeline,
                mipmap_pipeline,
                msaa_atlas_pipeline,
                msaa_image_mesh_pipeline,
                msaa_stencil_pipeline,
                feather_lut,
            }),
            width,
            height,
            mode,
        })
    }

    pub fn begin_frame(&self, clear_color: ColorInt) -> WgpuFrame {
        self.begin_frame_for_benchmark(clear_color, false)
    }

    pub fn begin_frame_for_benchmark(
        &self,
        clear_color: ColorInt,
        collect_work_metrics: bool,
    ) -> WgpuFrame {
        WgpuFrame {
            context: Arc::clone(&self.context),
            width: self.width,
            height: self.height,
            clear_color,
            state: DrawState::default(),
            stack: Vec::new(),
            draws: Vec::new(),
            draw_calls: 0,
            logical_flush: logical_flush::LogicalFlush::default(),
            logical_flush_allocations: LogicalFlushAllocations::default(),
            logical_flush_starts: vec![0],
            clips: Vec::new(),
            next_clip_id: 1,
            msaa_path_clips: Vec::new(),
            msaa_path_clip_id: 0,
            unsupported: None,
            mode: self.mode,
            work_recorder: work_metrics::FrameWorkRecorder::new(collect_work_metrics),
        }
    }

    pub fn adapter_info(&self) -> &WgpuAdapterInfo {
        &self.context.adapter_info
    }
}

impl Factory for WgpuFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        Box::new(WgpuBuffer {
            context: Arc::clone(&self.context),
            buffer_type,
            flags,
            bytes: vec![0; size_in_bytes],
            submitted: None,
        })
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
        Box::new(WgpuShader::Linear {
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
        Box::new(WgpuShader::Radial {
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
        Box::new(WgpuPath {
            valid: true,
            raw_path,
            fill_rule,
        })
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        Box::new(WgpuPath {
            valid: true,
            raw_path: RawPath::new(),
            fill_rule: FillRule::NonZero,
        })
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        Box::new(WgpuPaint::default())
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        let Some((width, height, pixels)) = decode_image_rgba(data) else {
            return Err(ImageDecodeError);
        };
        if !texture_extent_supported(
            width,
            height,
            self.context.device.limits().max_texture_dimension_2d,
        ) {
            return Err(ImageDecodeError);
        }
        let mip_level_count = u32::BITS - (width | height).leading_zeros();
        let texture = self
            .context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("nuxie-image"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
        self.context.queue.write_counted_texture(
            texture.as_image_copy(),
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            texture.size(),
        );
        self.context.mipmap_pipeline.generate(
            &self.context.device,
            &self.context.queue,
            &texture,
            mip_level_count,
        );
        let view = texture.create_view(&Default::default());
        Ok(Box::new(WgpuImage {
            width,
            height,
            texture: Some(Arc::new(WgpuImageTexture { texture, view })),
            owner: Arc::downgrade(&self.context),
        }))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct WgpuPath {
    raw_path: RawPath,
    fill_rule: FillRule,
    valid: bool,
}

impl RenderPath for WgpuPath {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn rewind(&mut self) {
        self.raw_path.rewind();
    }

    fn reserve(&mut self, verbs: usize, points: usize) {
        self.raw_path.reserve(verbs, points);
    }

    fn fill_rule(&mut self, value: FillRule) {
        self.fill_rule = value;
    }

    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let Some(path) = wgpu_path(path) else {
            self.raw_path.rewind();
            self.valid = false;
            return;
        };
        if !path.valid {
            self.raw_path.rewind();
            self.valid = false;
            return;
        }
        self.raw_path.add_path(&path.raw_path, transform);
    }

    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let Some(path) = wgpu_path(path) else {
            self.raw_path.rewind();
            self.valid = false;
            return;
        };
        if !path.valid {
            self.raw_path.rewind();
            self.valid = false;
            return;
        }
        self.raw_path.add_path_backwards(&path.raw_path, transform);
    }

    fn add_raw_path(&mut self, path: &RawPath) {
        self.raw_path.add_path(path, Mat2D::IDENTITY);
    }

    fn move_to(&mut self, x: f32, y: f32) {
        self.raw_path.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.raw_path.line_to(x, y);
    }

    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.raw_path.cubic_to(ox, oy, ix, iy, x, y);
    }

    fn close(&mut self) {
        self.raw_path.close();
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum WgpuShader {
    Linear {
        start: (f32, f32),
        end: (f32, f32),
        colors: Vec<ColorInt>,
        stops: Vec<f32>,
    },
    Radial {
        center: (f32, f32),
        radius: f32,
        colors: Vec<ColorInt>,
        stops: Vec<f32>,
    },
}

#[derive(Clone)]
struct GradientDefinition {
    paint_type: gpu::PaintType,
    colors: Vec<ColorInt>,
    stops: Vec<f32>,
    coeffs: [f32; 3],
}

#[derive(Clone, Copy)]
struct PreparedGradient {
    paint_type: gpu::PaintType,
    texture_y: f32,
    matrix: Mat2D,
    texture_span: [f32; 2],
}

struct GradientBatch {
    spans: Vec<gpu::GradientSpan>,
    height: u32,
    draws: Vec<Option<PreparedGradient>>,
}

impl RenderShader for WgpuShader {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
struct WgpuPaint {
    style: RenderPaintStyle,
    color: ColorInt,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    feather: f32,
    blend_mode: BlendMode,
    shader: Option<WgpuShader>,
    invalid_shader: bool,
}

#[derive(Clone, Copy)]
struct AtlasPlacement {
    scale: f32,
    translate: [f32; 2],
    bounds: [f32; 4],
    origin: [u32; 2],
    width: u32,
    height: u32,
}

impl Default for WgpuPaint {
    fn default() -> Self {
        Self {
            style: RenderPaintStyle::Fill,
            color: 0xff000000,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 0.0,
            blend_mode: BlendMode::SrcOver,
            shader: None,
            invalid_shader: false,
        }
    }
}

impl WgpuPaint {
    fn effective_stroke(&self) -> Option<(f32, StrokeJoin, StrokeCap)> {
        (self.style == RenderPaintStyle::Stroke).then_some((
            self.thickness,
            if self.feather != 0.0 {
                StrokeJoin::Round
            } else {
                self.join
            },
            if self.feather != 0.0 {
                StrokeCap::Round
            } else {
                self.cap
            },
        ))
    }

    fn is_opaque(&self) -> bool {
        if self.feather != 0.0 || self.blend_mode != BlendMode::SrcOver {
            return false;
        }
        match &self.shader {
            None => self.color >> 24 == 0xff,
            Some(WgpuShader::Linear { colors, .. }) | Some(WgpuShader::Radial { colors, .. }) => {
                colors.iter().all(|color| color >> 24 == 0xff)
            }
        }
    }
}

impl RenderPaint for WgpuPaint {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn style(&mut self, style: RenderPaintStyle) {
        self.style = style;
    }

    fn color(&mut self, value: ColorInt) {
        self.color = value;
    }

    fn thickness(&mut self, value: f32) {
        self.thickness = value.abs();
    }

    fn join(&mut self, value: StrokeJoin) {
        self.join = value;
    }

    fn cap(&mut self, value: StrokeCap) {
        self.cap = value;
    }

    fn feather(&mut self, value: f32) {
        self.feather = value.abs();
    }

    fn blend_mode(&mut self, value: BlendMode) {
        self.blend_mode = value;
    }

    fn shader(&mut self, shader: Option<&dyn RenderShader>) {
        match shader {
            Some(shader) => {
                self.shader = shader.as_any().downcast_ref::<WgpuShader>().cloned();
                self.invalid_shader = self.shader.is_none();
            }
            None => {
                self.shader = None;
                self.invalid_shader = false;
            }
        }
    }

    fn invalidate_stroke(&mut self) {}
}

struct WgpuBuffer {
    context: Arc<Context>,
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    bytes: Vec<u8>,
    submitted: Option<Arc<wgpu::Buffer>>,
}

impl RenderBuffer for WgpuBuffer {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn buffer_type(&self) -> RenderBufferType {
        self.buffer_type
    }
    fn flags(&self) -> RenderBufferFlags {
        self.flags
    }
    fn size_in_bytes(&self) -> usize {
        self.bytes.len()
    }
    fn map_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
    fn unmap(&mut self) {
        let usage = match self.buffer_type {
            RenderBufferType::Vertex => wgpu::BufferUsages::VERTEX,
            RenderBufferType::Index => wgpu::BufferUsages::INDEX,
        };
        // C++'s RenderBufferWebGPUImpl advances a buffer ring on every unmap.
        // Snapshotting here gives queued draws the same immutable submission.
        let zero = [0u8; 4];
        let contents = if self.bytes.is_empty() {
            zero.as_slice()
        } else {
            self.bytes.as_slice()
        };
        self.submitted = Some(Arc::new(self.context.device.create_counted_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("nuxie-render-buffer"),
                contents,
                usage,
            },
        )));
    }
}

struct WgpuImage {
    width: u32,
    height: u32,
    texture: Option<Arc<WgpuImageTexture>>,
    owner: Weak<Context>,
}

impl WgpuImage {
    fn belongs_to(&self, context: &Arc<Context>) -> bool {
        self.owner
            .upgrade()
            .is_some_and(|owner| Arc::ptr_eq(&owner, context))
    }
}

struct WgpuImageTexture {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl RenderImage for WgpuImage {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Debug, Clone, Copy)]
struct ClipRectState {
    rect: [f32; 4],
    matrix: Mat2D,
}

#[derive(Debug, Clone)]
struct ClipElement {
    path: WgpuPath,
    matrix: Mat2D,
    // Mirrors RiveRenderer::ClipElement::clipID. The MSAA stencil contains
    // one specific stack element, not an arbitrary shared path prefix.
    clip_id: u16,
}

impl ClipElement {
    fn is_equivalent(&self, matrix: Mat2D, path: &WgpuPath) -> bool {
        self.matrix == matrix
            && self.path.raw_path.mutation_id() == path.raw_path.mutation_id()
            && self.path.fill_rule == path.fill_rule
    }
}

#[derive(Debug, Clone, Copy)]
struct DrawState {
    transform: Mat2D,
    opacity: f32,
    clip_rect: Option<ClipRectState>,
    clip_is_empty: bool,
    clip_stack_height: usize,
}

impl Default for DrawState {
    fn default() -> Self {
        Self {
            transform: Mat2D::IDENTITY,
            opacity: 1.0,
            clip_rect: None,
            clip_is_empty: false,
            clip_stack_height: 0,
        }
    }
}

#[derive(Clone)]
struct SolidDraw {
    path: WgpuPath,
    paint: WgpuPaint,
    state: DrawState,
    role: DrawRole,
    image: Option<ImageDraw>,
}

#[derive(Clone)]
struct ImageRectDraw {
    texture: Arc<WgpuImageTexture>,
    sampler: ImageSampler,
    opacity: f32,
    blend_mode: BlendMode,
}

#[derive(Clone)]
struct ImageMeshDraw {
    texture: Arc<WgpuImageTexture>,
    sampler: ImageSampler,
    opacity: f32,
    blend_mode: BlendMode,
    vertices: Arc<wgpu::Buffer>,
    uvs: Arc<wgpu::Buffer>,
    indices: Arc<wgpu::Buffer>,
    index_count: u32,
}

#[derive(Clone)]
enum ImageDraw {
    Rect(ImageRectDraw),
    Mesh(ImageMeshDraw),
}

impl ImageDraw {
    fn texture(&self) -> &Arc<WgpuImageTexture> {
        match self {
            Self::Rect(draw) => &draw.texture,
            Self::Mesh(draw) => &draw.texture,
        }
    }

    fn sampler(&self) -> ImageSampler {
        match self {
            Self::Rect(draw) => draw.sampler,
            Self::Mesh(draw) => draw.sampler,
        }
    }

    fn opacity(&self) -> f32 {
        match self {
            Self::Rect(draw) => draw.opacity,
            Self::Mesh(draw) => draw.opacity,
        }
    }

    fn blend_mode(&self) -> BlendMode {
        match self {
            Self::Rect(draw) => draw.blend_mode,
            Self::Mesh(draw) => draw.blend_mode,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DrawRole {
    Content {
        clip_id: u16,
    },
    ClipUpdate {
        replacement_id: u16,
        parent_id: u16,
    },
    ClipReset {
        bounds: [f32; 4],
        action: MsaaClipResetAction,
    },
}

#[derive(Debug, Clone, Copy)]
enum MsaaClipResetAction {
    ClearPrevious,
    IntersectPreviousNonZero,
    IntersectPreviousEvenOdd,
    IntersectPreviousClockwise,
}

pub struct WgpuFrame {
    context: Arc<Context>,
    width: u32,
    height: u32,
    clear_color: ColorInt,
    state: DrawState,
    stack: Vec<DrawState>,
    draws: Vec<SolidDraw>,
    draw_calls: u64,
    logical_flush: logical_flush::LogicalFlush,
    logical_flush_allocations: LogicalFlushAllocations,
    logical_flush_starts: Vec<usize>,
    clips: Vec<ClipElement>,
    next_clip_id: u32,
    msaa_path_clips: Vec<ClipElement>,
    msaa_path_clip_id: u16,
    unsupported: Option<&'static str>,
    mode: RenderMode,
    work_recorder: work_metrics::FrameWorkRecorder,
}

#[derive(Clone, Default)]
struct LogicalFlushAllocations {
    simple_gradient_count: usize,
    complex_gradient_count: usize,
    atlas_draw_sizes: Vec<(u32, u32)>,
    coverage_word_count: usize,
}

impl LogicalFlushAllocations {
    fn with_batch(&self, frame: &WgpuFrame, draws: &[SolidDraw]) -> Result<Self, &'static str> {
        const MAX_GRADIENT_HEIGHT: usize = 2048;
        const MAX_COVERAGE_WORD_COUNT: usize = (1 << 27) / std::mem::size_of::<u32>();
        const RAMPS_PER_SIMPLE_ROW: usize = gradient_pipeline::TEXTURE_WIDTH as usize / 2;

        let mut next = self.clone();
        for draw in draws {
            if let Some(gradient) = draw
                .paint
                .shader
                .as_ref()
                .and_then(|shader| normalize_gradient(shader, draw.state.opacity))
            {
                let simple = gradient.stops.len() == 1
                    || (gradient.stops.len() == 2
                        && gradient.stops[0] == 0.0
                        && gradient.stops[1] == 1.0);
                if simple {
                    next.simple_gradient_count = next
                        .simple_gradient_count
                        .checked_add(1)
                        .ok_or("logical flush gradient count overflow")?;
                } else {
                    next.complex_gradient_count = next
                        .complex_gradient_count
                        .checked_add(1)
                        .ok_or("logical flush gradient count overflow")?;
                }
            }

            let uses_clockwise_coverage =
                draw_requires_clockwise_atomic(draw, frame.width, frame.height)
                    || matches!(draw.role, DrawRole::ClipUpdate { parent_id, .. } if parent_id != 0)
                    || matches!(draw.role, DrawRole::Content { clip_id } if clip_id != 0);
            if frame.mode == RenderMode::ClockwiseAtomic
                && uses_clockwise_coverage
                && draw.image.is_none()
                && !matches!(draw.role, DrawRole::ClipUpdate { parent_id: 0, .. })
            {
                let inverse_clip_path = match draw.role {
                    DrawRole::ClipUpdate { parent_id, .. } if parent_id != 0 => Some(
                        invert_clockwise_path(
                            &draw.path.raw_path,
                            draw.path.fill_rule,
                            draw.state.transform,
                            frame.width,
                            frame.height,
                        )
                        .ok_or("nested clip has invalid inverse coverage path")?,
                    ),
                    _ => None,
                };
                let coverage_path = inverse_clip_path.as_ref().unwrap_or(&draw.path.raw_path);
                let coverage_bounds = if inverse_clip_path.is_some() {
                    draw::path_pixel_bounds(coverage_path, draw.state.transform)
                } else {
                    path_draw_pixel_bounds(&draw.path, &draw.paint, draw.state.transform)
                }
                .ok_or("draw has invalid clockwise coverage bounds")?;
                let (_, word_count) = draw::clockwise_atomic_coverage_range_from_bounds(
                    coverage_bounds,
                    frame.width,
                    frame.height,
                    next.coverage_word_count,
                )
                .ok_or("draw has invalid clockwise coverage allocation")?;
                next.coverage_word_count = next
                    .coverage_word_count
                    .checked_add(word_count)
                    .ok_or("logical flush coverage count overflow")?;
            }

            let uses_feather_atlas = frame.mode == RenderMode::Msaa
                || (frame.mode == RenderMode::ClockwiseAtomic
                    && draw::feather_requires_atlas(
                        draw.paint.feather,
                        draw.state.transform,
                        false,
                    ));
            if draw.paint.feather != 0.0 && uses_feather_atlas {
                let placement = feather_atlas_placement(
                    &draw.path.raw_path,
                    draw.state.transform,
                    draw.paint.feather,
                    draw.paint.effective_stroke(),
                    frame.width,
                    frame.height,
                )
                .ok_or("draw has invalid feather atlas placement")?;
                let draw_size = (
                    placement.width - FEATHER_ATLAS_PADDING * 2,
                    placement.height - FEATHER_ATLAS_PADDING * 2,
                );
                next.atlas_draw_sizes.push(draw_size);
            }
        }

        let gradient_height = next
            .simple_gradient_count
            .div_ceil(RAMPS_PER_SIMPLE_ROW)
            .checked_add(next.complex_gradient_count)
            .ok_or("logical flush gradient height overflow")?;
        let limits = frame.context.device.limits();
        if gradient_height > MAX_GRADIENT_HEIGHT.min(limits.max_texture_dimension_2d as usize) {
            return Err("draw batch exceeds logical flush gradient texture limit");
        }
        let max_coverage_words = MAX_COVERAGE_WORD_COUNT
            .min(limits.max_storage_buffer_binding_size as usize / std::mem::size_of::<u32>())
            .min(limits.max_buffer_size as usize / std::mem::size_of::<u32>());
        if next.coverage_word_count > max_coverage_words {
            return Err("draw batch exceeds logical flush coverage buffer limit");
        }
        if !next.atlas_draw_sizes.is_empty() {
            let atlas_result = pack_logical_feather_atlas_for_cpp(
                limits.max_texture_dimension_2d,
                &next.atlas_draw_sizes,
            );
            atlas_result
                .map_err(|_| "draw batch exceeds logical flush feather atlas texture limit")?;
        }
        Ok(next)
    }
}

#[allow(dead_code)]
struct ClockwiseAtomicCoverageSnapshot {
    borrowed: Vec<u32>,
    main: Vec<u32>,
    ranges: Vec<gpu::CoverageBufferRange>,
    kinds: Vec<clockwise_atomic_pipeline::ClockwiseAtomicDrawKind>,
    clip_updates: Vec<Vec<u8>>,
    clip_bytes_per_row: u32,
}

impl Renderer for WgpuFrame {
    fn save(&mut self) {
        self.stack.push(self.state);
    }

    fn restore(&mut self) {
        if let Some(state) = self.stack.pop() {
            self.state = state;
        }
    }

    fn transform(&mut self, transform: Mat2D) {
        self.state.transform = multiply(self.state.transform, transform);
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        self.draw_calls = self.draw_calls.saturating_add(1);
        if self.state.clip_is_empty {
            return;
        }
        let Some(path) = wgpu_path(path) else {
            self.unsupported
                .get_or_insert("path from another renderer backend");
            return;
        };
        if !path.valid {
            self.unsupported
                .get_or_insert("path contains resources from another renderer backend");
            return;
        }
        let Some(paint) = wgpu_paint(paint) else {
            self.unsupported
                .get_or_insert("paint from another renderer backend");
            return;
        };
        if paint.invalid_shader {
            self.unsupported
                .get_or_insert("paint shader from another renderer backend");
            return;
        }
        if path_draw_is_noop(path, paint, self.state.transform)
            || path_draw_is_outside_frame(
                path,
                paint,
                self.state.transform,
                self.width,
                self.height,
            )
        {
            return;
        }
        let Some((clip_updates, clip_id)) = self.prepare_scheduled_clip_updates() else {
            return;
        };
        let content = SolidDraw {
            path: path.clone(),
            paint: paint.clone(),
            state: self.state,
            role: DrawRole::Content { clip_id },
            image: None,
        };
        let msaa_feather_atlas = self.mode == RenderMode::Msaa && paint.feather != 0.0;
        if self.mode == RenderMode::Msaa && paint.feather != 0.0 {
            if self.state.clip_rect.is_some()
                && !self.context.msaa_atlas_pipeline.supports_clip_rect()
            {
                self.unsupported
                    .get_or_insert("clip rectangles on msaa feather atlas draws");
                return;
            }
        }
        if clip_id != 0 && !msaa_feather_atlas {
            if !atomic_draw_is_eligible(&content) {
                self.unsupported
                    .get_or_insert("non-rectangular clips on fallback draws");
                return;
            }
        }
        self.push_content_batch(clip_updates, content);
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        if self.state.clip_is_empty {
            return;
        }
        let Some(path) = wgpu_path(path) else {
            self.unsupported
                .get_or_insert("clip path from another renderer backend");
            return;
        };
        if !path.valid {
            self.unsupported
                .get_or_insert("clip path contains resources from another renderer backend");
            return;
        }
        if path.raw_path.verbs().is_empty() {
            self.state.clip_is_empty = true;
            return;
        }
        // RenderContext::frameSupportsClipRects() only enables clip planes for
        // MSAA when the C++ backend explicitly advertises them. The WebGPU
        // backend leaves supportsClipPlanes false, so its MSAA clip stack must
        // represent rectangles as stencil clips too. Clockwise atomics keep
        // the generated clip-rectangle shader path.
        if self.mode != RenderMode::Msaa {
            if let Some(rect) = path_aabb(&path.raw_path) {
                if apply_clip_rect(&mut self.state, rect) {
                    return;
                }
                // C++ retains the existing optimized clip rect and falls back
                // to the ordinary clip stack for an incompatible rectangle.
            }
        }
        self.push_clip_path(path);
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        self.draw_calls = self.draw_calls.saturating_add(1);
        if self.state.clip_is_empty {
            return;
        }
        let Some(image) = image else {
            return;
        };
        let Some(image) = image.as_any().downcast_ref::<WgpuImage>() else {
            self.unsupported
                .get_or_insert("image from another renderer backend");
            return;
        };
        if !image.belongs_to(&self.context) {
            self.unsupported
                .get_or_insert("image from another renderer factory");
            return;
        }
        let Some(texture) = &image.texture else {
            return;
        };
        let Some((clip_updates, clip_id)) = self.prepare_scheduled_clip_updates() else {
            return;
        };
        // C++ RiveRenderer::drawImage uses ImageRectDraw only for atomics;
        // MSAA draws this unit rectangle with an image paint.
        let mut raw_path = RawPath::new();
        raw_path.move_to(0.0, 0.0);
        raw_path.line_to(1.0, 0.0);
        raw_path.line_to(1.0, 1.0);
        raw_path.line_to(0.0, 1.0);
        raw_path.close();
        let image_matrix = multiply(
            self.state.transform,
            Mat2D([image.width as f32, 0.0, 0.0, image.height as f32, 0.0, 0.0]),
        );
        let mut paint = WgpuPaint::default();
        paint.blend_mode = blend_mode;
        let content = SolidDraw {
            path: WgpuPath {
                valid: true,
                raw_path,
                fill_rule: FillRule::NonZero,
            },
            paint,
            state: DrawState {
                transform: image_matrix,
                ..self.state
            },
            role: DrawRole::Content { clip_id },
            image: Some(ImageDraw::Rect(ImageRectDraw {
                texture: Arc::clone(texture),
                sampler,
                opacity: (opacity * self.state.opacity).max(0.0),
                blend_mode,
            })),
        };
        if !atomic_draw_is_eligible(&content) {
            self.unsupported
                .get_or_insert("images on fallback draw path");
            return;
        }
        self.push_content_batch(clip_updates, content);
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
        self.draw_calls = self.draw_calls.saturating_add(1);
        if self.state.clip_is_empty {
            return;
        }
        let Some(image) = image else {
            return;
        };
        let Some(image) = image.as_any().downcast_ref::<WgpuImage>() else {
            self.unsupported
                .get_or_insert("image from another renderer backend");
            return;
        };
        if !image.belongs_to(&self.context) {
            self.unsupported
                .get_or_insert("image from another renderer factory");
            return;
        }
        let Some(texture) = &image.texture else {
            return;
        };
        let Some(vertices) = vertices.and_then(wgpu_buffer) else {
            self.unsupported
                .get_or_insert("invalid image mesh vertex buffer");
            return;
        };
        let Some(uvs) = uv_coords.and_then(wgpu_buffer) else {
            self.unsupported
                .get_or_insert("invalid image mesh UV buffer");
            return;
        };
        let Some(indices) = indices.and_then(wgpu_buffer) else {
            self.unsupported
                .get_or_insert("invalid image mesh index buffer");
            return;
        };
        if !Arc::ptr_eq(&vertices.context, &self.context)
            || !Arc::ptr_eq(&uvs.context, &self.context)
            || !Arc::ptr_eq(&indices.context, &self.context)
        {
            self.unsupported
                .get_or_insert("image mesh buffers from another renderer factory");
            return;
        }
        let required_vertex_bytes = usize::try_from(vertex_count)
            .ok()
            .and_then(|count| count.checked_mul(8));
        let required_index_bytes = usize::try_from(index_count)
            .ok()
            .and_then(|count| count.checked_mul(2));
        if vertices.buffer_type != RenderBufferType::Vertex
            || uvs.buffer_type != RenderBufferType::Vertex
            || indices.buffer_type != RenderBufferType::Index
            || required_vertex_bytes
                .is_none_or(|size| vertices.bytes.len() < size || uvs.bytes.len() < size)
            || required_index_bytes.is_none_or(|size| indices.bytes.len() < size)
        {
            self.unsupported
                .get_or_insert("malformed image mesh buffers");
            return;
        }
        let (Some(vertex_buffer), Some(uv_buffer), Some(index_buffer)) =
            (&vertices.submitted, &uvs.submitted, &indices.submitted)
        else {
            self.unsupported
                .get_or_insert("unmapped image mesh buffers");
            return;
        };
        let Some((clip_updates, clip_id)) = self.prepare_scheduled_clip_updates() else {
            return;
        };
        let content = SolidDraw {
            path: WgpuPath {
                valid: true,
                raw_path: RawPath::new(),
                fill_rule: FillRule::NonZero,
            },
            paint: WgpuPaint::default(),
            state: self.state,
            role: DrawRole::Content { clip_id },
            image: Some(ImageDraw::Mesh(ImageMeshDraw {
                texture: Arc::clone(texture),
                sampler,
                opacity: (opacity * self.state.opacity).max(0.0),
                blend_mode,
                vertices: Arc::clone(vertex_buffer),
                uvs: Arc::clone(uv_buffer),
                indices: Arc::clone(index_buffer),
                index_count,
            })),
        };
        self.push_content_batch(clip_updates, content);
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        self.state.opacity *= opacity;
    }
}

impl WgpuFrame {
    fn push_clip_path(&mut self, path: &WgpuPath) {
        let height = self.state.clip_stack_height;
        if self
            .clips
            .get(height)
            .is_none_or(|clip| !clip.is_equivalent(self.state.transform, path))
        {
            self.clips.truncate(height);
            self.clips.push(ClipElement {
                path: path.clone(),
                matrix: self.state.transform,
                clip_id: 0,
            });
        }
        self.state.clip_stack_height = height + 1;
    }

    fn prepare_scheduled_clip_updates(&mut self) -> Option<(Vec<SolidDraw>, u16)> {
        if self.mode != RenderMode::Msaa {
            return self.prepare_clip_updates();
        }

        let height = self.state.clip_stack_height;
        let current_clips = self.clips[..height].to_vec();
        let previous_active = self.msaa_path_clips.last().cloned();

        if current_clips.is_empty() {
            return Some((Vec::new(), 0));
        }

        // This is RiveRenderer::applyClip's scan for the clip ID currently in
        // the stencil buffer. A shared path prefix is not enough: if the
        // resident leaf belongs to a different branch, MSAA must clear it and
        // replay the current stack from its root.
        let active_index = if self.msaa_path_clip_id == 0 {
            None
        } else {
            self.clips[..height]
                .iter()
                .rposition(|clip| clip.clip_id == self.msaa_path_clip_id)
        };
        let parent_id = active_index
            .map(|index| self.clips[index].clip_id)
            .unwrap_or(0);
        let update_start = active_index.map_or(0, |index| index + 1);
        let (updates, clip_id) = if update_start == height {
            (Vec::new(), parent_id)
        } else {
            self.prepare_clip_updates_from(update_start, parent_id)?
        };

        let mut scheduled = Vec::with_capacity(updates.len() * 2 + 1);
        if self.msaa_path_clip_id != 0 && active_index.is_none() {
            if let Some(active) = previous_active.as_ref() {
                scheduled
                    .push(self.msaa_clip_reset_draw(active, MsaaClipResetAction::ClearPrevious));
            }
        }
        for (offset, update) in updates.into_iter().enumerate() {
            scheduled.push(update);
            let clip_index = update_start + offset;
            if clip_index != 0 {
                let action = match current_clips[clip_index].path.fill_rule {
                    FillRule::NonZero => MsaaClipResetAction::IntersectPreviousNonZero,
                    FillRule::EvenOdd => MsaaClipResetAction::IntersectPreviousEvenOdd,
                    FillRule::Clockwise => MsaaClipResetAction::IntersectPreviousClockwise,
                };
                scheduled.push(self.msaa_clip_reset_draw(&current_clips[clip_index - 1], action));
            }
        }
        self.msaa_path_clips = current_clips;
        self.msaa_path_clip_id = clip_id;
        Some((scheduled, clip_id))
    }

    fn begin_logical_flush(&mut self) {
        debug_assert_ne!(self.logical_flush_starts.last(), Some(&self.draws.len()));
        self.logical_flush_starts.push(self.draws.len());
        self.logical_flush.rewind();
        self.logical_flush_allocations = LogicalFlushAllocations::default();
        self.next_clip_id = 1;
        self.msaa_path_clips.clear();
        self.msaa_path_clip_id = 0;
    }

    fn push_content_batch(&mut self, clip_updates: Vec<SolidDraw>, content: SolidDraw) {
        let make_batch = |updates: Vec<SolidDraw>, content: SolidDraw| {
            let mut batch = Vec::with_capacity(updates.len() + 1);
            batch.extend(updates);
            batch.push(content);
            batch
        };
        let batch = make_batch(clip_updates, content.clone());
        if self.try_push_logical_batch(&batch).is_ok() {
            self.draws.extend(batch);
            return;
        }
        if self.logical_flush_starts.last() == Some(&self.draws.len()) {
            self.unsupported
                .get_or_insert("draw batch exceeds logical flush resource limits");
            return;
        }

        self.begin_logical_flush();
        let Some((clip_updates, clip_id)) = self.prepare_scheduled_clip_updates() else {
            return;
        };
        let mut content = content;
        match &mut content.role {
            DrawRole::Content {
                clip_id: content_clip_id,
            } => *content_clip_id = clip_id,
            DrawRole::ClipUpdate { .. } | DrawRole::ClipReset { .. } => {
                unreachable!("content batch must end in a content draw")
            }
        }
        let batch = make_batch(clip_updates, content);
        if let Err(reason) = self.try_push_logical_batch(&batch) {
            self.unsupported.get_or_insert(reason);
            return;
        }
        self.draws.extend(batch);
    }

    fn try_push_logical_batch(&mut self, batch: &[SolidDraw]) -> Result<(), &'static str> {
        let resources = logical_flush_batch_resources(batch, self.mode, self.width, self.height)
            .ok_or("draw batch overflows logical flush resource accounting")?;
        let allocations = self.logical_flush_allocations.with_batch(self, batch)?;
        if !self.logical_flush.push_draws(resources) {
            return Err("draw batch exceeds logical flush resource counters");
        }
        self.logical_flush_allocations = allocations;
        Ok(())
    }

    fn msaa_clip_reset_draw(&self, clip: &ClipElement, action: MsaaClipResetAction) -> SolidDraw {
        let bounds = draw::path_pixel_bounds(&clip.path.raw_path, clip.matrix).unwrap_or([
            0,
            0,
            self.width as i32,
            self.height as i32,
        ]);
        let [left, top, right, bottom] = bounds;
        let bounds = [
            left.clamp(0, self.width as i32) as f32,
            top.clamp(0, self.height as i32) as f32,
            right.clamp(0, self.width as i32) as f32,
            bottom.clamp(0, self.height as i32) as f32,
        ];
        SolidDraw {
            path: clip.path.clone(),
            paint: WgpuPaint::default(),
            state: DrawState::default(),
            role: DrawRole::ClipReset { bounds, action },
            image: None,
        }
    }

    fn prepare_clip_updates(&mut self) -> Option<(Vec<SolidDraw>, u16)> {
        self.prepare_clip_updates_from(0, 0)
    }

    fn prepare_clip_updates_from(
        &mut self,
        start: usize,
        initial_parent_id: u16,
    ) -> Option<(Vec<SolidDraw>, u16)> {
        let height = self.state.clip_stack_height;
        if height == 0 {
            return Some((Vec::new(), 0));
        }
        debug_assert!(start < height);
        // C++ RiveRenderer::applyClip generates a new ID whenever a clip is
        // rendered. Reusing stack depth would accept stale coverage left by an
        // unrelated clip at the same depth in the storage-backed clip plane.
        let Ok(update_count) = u32::try_from(height - start) else {
            self.unsupported
                .get_or_insert("more than 65535 clip updates in one frame");
            return None;
        };
        let Some(end) = self.next_clip_id.checked_add(update_count) else {
            self.unsupported
                .get_or_insert("more than 65535 clip updates in one frame");
            return None;
        };
        if end > u16::MAX as u32 + 1 {
            self.unsupported
                .get_or_insert("more than 65535 clip updates in one frame");
            return None;
        }
        let mut updates = Vec::with_capacity(height - start);
        let mut parent_id = initial_parent_id;
        for (offset, clip) in self.clips[start..height].iter_mut().enumerate() {
            if self.mode == RenderMode::ClockwiseAtomic
                && parent_id != 0
                && invert_clockwise_path(
                    &clip.path.raw_path,
                    clip.path.fill_rule,
                    clip.matrix,
                    self.width,
                    self.height,
                )
                .is_none()
            {
                return None;
            }
            let replacement_id = (self.next_clip_id + offset as u32) as u16;
            clip.clip_id = replacement_id;
            updates.push(SolidDraw {
                path: clip.path.clone(),
                paint: WgpuPaint::default(),
                state: DrawState {
                    transform: clip.matrix,
                    clip_rect: None,
                    clip_stack_height: 0,
                    ..self.state
                },
                role: DrawRole::ClipUpdate {
                    replacement_id,
                    parent_id,
                },
                image: None,
            });
            parent_id = replacement_id;
        }
        self.next_clip_id = end;
        Some((updates, parent_id))
    }

    fn metrics(&self) -> WgpuFrameMetrics {
        let logical_flushes = u64::try_from(self.logical_flush_starts.len()).unwrap_or(u64::MAX);
        let mut atomic_strategy_partitions = 0_u64;
        if self.mode == RenderMode::ClockwiseAtomic {
            for (flush_index, &flush_start) in self.logical_flush_starts.iter().enumerate() {
                let flush_end = self
                    .logical_flush_starts
                    .get(flush_index + 1)
                    .copied()
                    .unwrap_or(self.draws.len());
                let mut start = flush_start;
                while start < flush_end {
                    atomic_strategy_partitions = atomic_strategy_partitions.saturating_add(1);
                    start = atomic_strategy_run_end(
                        &self.draws,
                        start,
                        flush_end,
                        self.width,
                        self.height,
                    );
                }
            }
        }
        WgpuFrameMetrics {
            draw_calls: self.draw_calls,
            logical_flushes,
            atomic_strategy_partitions,
            backend_work: BackendWorkMetrics::default(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn finish(self) -> Result<Vec<u8>, RendererError> {
        pollster::block_on(self.finish_async())
    }

    /// Submits the frame, waits asynchronously for GPU completion, and returns
    /// the offscreen target as tightly packed RGBA pixels.
    pub async fn finish_async(self) -> Result<Vec<u8>, RendererError> {
        self.finish_internal(false, false, true, true)
            .await
            .map(|(pixels, _, _, _, _, _)| pixels)
    }

    /// Encodes the frame, submits all work, and waits for GPU completion
    /// without copying the render target back to the CPU.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn finish_for_benchmark(self) -> Result<WgpuFrameMetrics, RendererError> {
        pollster::block_on(self.finish_for_benchmark_async())
    }

    /// Asynchronously encodes the frame, submits all work, and waits for GPU
    /// completion without copying the render target back to the CPU.
    pub async fn finish_for_benchmark_async(self) -> Result<WgpuFrameMetrics, RendererError> {
        let mut metrics = self.metrics();
        let (_, _, _, _, _, backend_work) = self.finish_internal(false, false, true, false).await?;
        metrics.backend_work = backend_work;
        Ok(metrics)
    }

    #[cfg(test)]
    fn finish_with_clockwise_atomic_coverage(
        self,
    ) -> Result<(Vec<u8>, Vec<ClockwiseAtomicCoverageSnapshot>), RendererError> {
        pollster::block_on(self.finish_internal(true, false, true, true))
            .map(|(pixels, coverage, _, _, _, _)| (pixels, coverage))
    }

    #[cfg(test)]
    fn finish_with_atomic_coverage(self) -> Result<(Vec<u8>, Vec<Vec<u32>>), RendererError> {
        pollster::block_on(self.finish_internal(false, true, true, true))
            .map(|(pixels, _, coverage, _, _, _)| (pixels, coverage))
    }

    #[cfg(test)]
    fn finish_with_atomic_planes(
        self,
    ) -> Result<(Vec<u8>, Vec<Vec<u32>>, Vec<Vec<u32>>, Vec<Vec<u32>>), RendererError> {
        pollster::block_on(self.finish_internal(false, true, true, true))
            .map(|(pixels, _, coverage, clips, colors, _)| (pixels, coverage, clips, colors))
    }

    #[cfg(test)]
    fn finish_without_msaa_board_scheduling(self) -> Result<Vec<u8>, RendererError> {
        pollster::block_on(self.finish_internal(false, false, false, true))
            .map(|(pixels, _, _, _, _, _)| pixels)
    }

    async fn finish_internal(
        self,
        capture_clockwise_atomic_coverage: bool,
        capture_atomic_planes: bool,
        schedule_msaa_draws: bool,
        read_pixels: bool,
    ) -> Result<
        (
            Vec<u8>,
            Vec<ClockwiseAtomicCoverageSnapshot>,
            Vec<Vec<u32>>,
            Vec<Vec<u32>>,
            Vec<Vec<u32>>,
            BackendWorkMetrics,
        ),
        RendererError,
    > {
        if let Some(feature) = self.unsupported {
            return Err(RendererError::Unsupported(feature));
        }
        let frame_attachments = self
            .context
            .frame_attachments
            .checkout(&self.context.device);
        let texture = frame_attachments.target_texture.clone();
        let view = frame_attachments.target_view.clone();
        let multisample_view = frame_attachments.multisample_view.clone();
        let stencil_view = frame_attachments.stencil_view.clone();
        let mut encoder =
            self.context
                .device
                .create_counted_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-frame-encoder"),
                });
        let tessellation_uploads = RefCell::new(
            self.context
                .tessellator
                .begin_frame_uploads(&self.context.device),
        );
        let atomic_backing = RefCell::new(None);
        let submit_and_wait = |encoder: &mut wgpu::CommandEncoder| {
            let next_encoder = self.context.device.create_counted_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-frame-encoder"),
                },
            );
            let submitted_encoder = std::mem::replace(encoder, next_encoder);
            tessellation_uploads.borrow_mut().flush(&self.context.queue);
            self.context
                .queue
                .submit_counted(Some(submitted_encoder.finish()));
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.context
                    .device
                    .poll(wgpu::PollType::wait_indefinitely())
                    .map_err(|error| RendererError::Map(error.to_string()))?;
                tessellation_uploads
                    .borrow_mut()
                    .begin_next_submission(&self.context.device);
            }
            #[cfg(target_arch = "wasm32")]
            tessellation_uploads
                .borrow_mut()
                .begin_next_submission_without_reuse();
            Ok::<(), RendererError>(())
        };
        let mut pending_coverage_readbacks = Vec::new();
        let mut pending_atomic_coverage_readbacks = Vec::new();
        let mut pending_atomic_clip_readbacks = Vec::new();
        let mut pending_atomic_color_readbacks = Vec::new();
        let mut encode_atomic_run =
            |draws: &[SolidDraw],
             draw_group_starts: &[usize],
             clear_target: bool,
             force_clockwise_atomic_batch: bool,
             load_color: Option<&wgpu::TextureView>,
             encoder: &mut wgpu::CommandEncoder| {
                struct PreparedAtomicDraw {
                    spans: Vec<gpu::TessVertexSpan>,
                    base_instance: u32,
                    instance_count: u32,
                    patch_index_range: std::ops::Range<u32>,
                    contour_range: std::ops::Range<usize>,
                    tessellation_index: usize,
                    triangles: Vec<gpu::TriangleVertex>,
                    atlas: Option<AtlasPlacement>,
                    atlas_blit_vertices: Vec<gpu::TriangleVertex>,
                    is_stroke: bool,
                    is_feather: bool,
                    batchable_direct_stroke: bool,
                    hsl_blend: bool,
                    image: Option<Arc<WgpuImageTexture>>,
                    image_sampler: ImageSampler,
                    image_uniforms: Option<gpu::ImageDrawUniforms>,
                    image_mesh: Option<PreparedImageMesh>,
                }

                struct PreparedImageMesh {
                    vertices: Arc<wgpu::Buffer>,
                    uvs: Arc<wgpu::Buffer>,
                    indices: Arc<wgpu::Buffer>,
                    index_count: u32,
                }

                validate_atomic_path_count(draws.len())?;

                if clear_target {
                    let attachments = [Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(color(self.clear_color)),
                            store: wgpu::StoreOp::Store,
                        },
                    })];
                    let _pass = encoder.begin_counted_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("nuxie-atomic-frame-clear"),
                        color_attachments: &attachments,
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                }
                let padded_width = align_to(self.width, 32);
                let padded_height = align_to(self.height, 32);
                let contour_count = |draw: &SolidDraw| {
                    draw.path
                        .raw_path
                        .verbs()
                        .iter()
                        .filter(|verb| **verb == PathVerb::Move)
                        .count()
                };
                let has_global_clip = draws.iter().any(|draw| {
                    matches!(draw.role, DrawRole::ClipUpdate { .. })
                        && contour_count(draw) > 1
                        && draw::should_use_interior_tessellation(
                            &draw.path.raw_path,
                            draw.state.transform,
                        )
                });
                let homogeneous_global_fill = draws.iter().all(|draw| {
                    matches!(draw.role, DrawRole::Content { clip_id: 0 })
                        && draw.paint.style == RenderPaintStyle::Fill
                        && draw.paint.feather == 0.0
                        && draw.state.clip_rect.is_none()
                        && contour_count(draw) > 1
                        && draw::should_use_interior_tessellation(
                            &draw.path.raw_path,
                            draw.state.transform,
                        )
                });
                let clockwise_atomic_clip_run = has_global_clip
                    && draws.iter().all(|draw| {
                        draw.image.is_none()
                            && draw.paint.style == RenderPaintStyle::Fill
                            && draw.paint.feather == 0.0
                            && draw.state.clip_rect.is_none()
                    });
                let use_clockwise_atomic_batch = force_clockwise_atomic_batch
                    || homogeneous_global_fill
                    || clockwise_atomic_clip_run;
                if load_color.is_some()
                    && use_clockwise_atomic_batch
                    && draws
                        .iter()
                        .any(|draw| !matches!(draw.role, DrawRole::Content { clip_id: 0 }))
                {
                    return Err(RendererError::Unsupported(
                        "advanced clockwise-atomic blending with path clips",
                    ));
                }
                let mut clockwise_atomic_coverage_words = 0usize;
                let gradient_batch = prepare_gradient_batch(draws);
                let mut prepared = Vec::with_capacity(draws.len());
                let mut paths = vec![gpu::PathData::zeroed()];
                let mut paints = vec![gpu::PaintData::solid(
                    0,
                    FillRule::NonZero,
                    BlendMode::SrcOver,
                )];
                let mut paint_aux = vec![gpu::PaintAuxData::zeroed()];
                let mut contours = Vec::new();
                for (draw_index, draw) in draws.iter().enumerate() {
                    let path_id = u16::try_from(draw_index + 1).expect("atomic path ID overflow");
                    let clockwise_override = use_clockwise_atomic_batch
                        || match draw.role {
                            DrawRole::Content { clip_id } => clip_id != 0,
                            DrawRole::ClipUpdate { parent_id, .. } => parent_id != 0,
                            DrawRole::ClipReset { .. } => {
                                unreachable!("MSAA clip reset escaped atomic partitioning")
                            }
                        };
                    let inverse_clip_path = match draw.role {
                        DrawRole::ClipUpdate { parent_id, .. }
                            if use_clockwise_atomic_batch && parent_id != 0 =>
                        {
                            Some(
                                invert_clockwise_path(
                                    &draw.path.raw_path,
                                    draw.path.fill_rule,
                                    draw.state.transform,
                                    self.width,
                                    self.height,
                                )
                                .expect("atomic eligibility already validated clip transform"),
                            )
                        }
                        _ => None,
                    };
                    let raw_path = inverse_clip_path.as_ref().unwrap_or(&draw.path.raw_path);
                    let source_fill_rule = if inverse_clip_path.is_some() {
                        FillRule::Clockwise
                    } else {
                        draw.path.fill_rule
                    };
                    let fill_rule = if clockwise_override {
                        FillRule::Clockwise
                    } else {
                        source_fill_rule
                    };
                    let paint_fill_rule =
                        atomic_paint_fill_rule(source_fill_rule, use_clockwise_atomic_batch);
                    let (
                        mut spans,
                        mut path,
                        mut draw_contours,
                        base_instance,
                        instance_count,
                        patch_index_range,
                        mut triangles,
                    ) = if matches!(draw.image, Some(ImageDraw::Mesh(_))) {
                        (
                            Vec::new(),
                            gpu::PathData::zeroed(),
                            Vec::new(),
                            0,
                            0,
                            0..0,
                            Vec::new(),
                        )
                    } else if draw.paint.feather != 0.0 {
                        let stroke = draw.paint.effective_stroke();
                        let is_stroke = stroke.is_some();
                        let requires_atlas = draw::feather_requires_atlas(
                            draw.paint.feather,
                            draw.state.transform,
                            false,
                        );
                        let fill_direction = if is_stroke {
                            draw::FeatherFillDirection::Forward
                        } else {
                            let negate_coverage = draw::clockwise_atomic_negate_coverage(
                                raw_path,
                                draw.state.transform,
                                source_fill_rule,
                                clockwise_override,
                            );
                            match (requires_atlas, negate_coverage) {
                                (true, true) => draw::FeatherFillDirection::Reverse,
                                (true, false) => draw::FeatherFillDirection::Forward,
                                (false, true) => draw::FeatherFillDirection::ForwardThenReverse,
                                (false, false) => draw::FeatherFillDirection::ReverseThenForward,
                            }
                        };
                        let tessellation = draw::build_feather_tessellation_with_direction(
                            raw_path,
                            draw.state.transform,
                            draw.paint.feather,
                            stroke,
                            fill_direction,
                        )
                        .expect("atomic eligibility already validated feather tessellation");
                        let patch_index_range = if is_stroke {
                            0..gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT as u32
                        } else {
                            gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT as u32
                                ..(gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT
                                    + gpu::MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT)
                                    as u32
                        };
                        (
                            tessellation.spans,
                            tessellation.path,
                            tessellation.contours,
                            tessellation.base_instance,
                            tessellation.instance_count,
                            patch_index_range,
                            Vec::new(),
                        )
                    } else if draw.paint.style == RenderPaintStyle::Stroke {
                        let tessellation = draw::build_stroke_tessellation(
                            raw_path,
                            draw.state.transform,
                            draw.paint.thickness,
                            draw.paint.join,
                            draw.paint.cap,
                        )
                        .expect("atomic eligibility already validated stroke tessellation");
                        (
                            tessellation.spans,
                            tessellation.path,
                            tessellation.contours,
                            tessellation.base_instance,
                            tessellation.instance_count,
                            0..gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT as u32,
                            Vec::new(),
                        )
                    } else if let Some(tessellation) =
                        draw::should_use_interior_tessellation(raw_path, draw.state.transform)
                            .then(|| {
                                draw::build_interior_tessellation(
                                    raw_path,
                                    draw.state.transform,
                                    source_fill_rule,
                                    clockwise_override,
                                )
                            })
                            .flatten()
                    {
                        (
                            tessellation.spans,
                            tessellation.path,
                            tessellation.contours,
                            tessellation.base_instance,
                            tessellation.instance_count,
                            (gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT
                                + gpu::MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT)
                                as u32
                                ..(gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT
                                    + gpu::MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT
                                    + gpu::OUTER_CURVE_PATCH_INDEX_COUNT)
                                    as u32,
                            tessellation.triangles,
                        )
                    } else {
                        let mut tessellation =
                            draw::build_fill_tessellation(raw_path, draw.state.transform)
                                .expect("atomic eligibility already validated tessellation");
                        tessellation.make_double_sided_with_direction(
                            draw::clockwise_atomic_negate_coverage(
                                raw_path,
                                draw.state.transform,
                                source_fill_rule,
                                clockwise_override,
                            ),
                        );
                        (
                            tessellation.spans,
                            tessellation.path,
                            tessellation.contours,
                            tessellation.base_instance,
                            tessellation.instance_count,
                            0..gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT as u32,
                            Vec::new(),
                        )
                    };
                    let contour_offset = contours.len() as u32;
                    for span in &mut spans {
                        let local_id = span.contour_id_with_flags & gpu::CONTOUR_ID_MASK;
                        if local_id != 0 {
                            let global_id = contour_offset + local_id;
                            assert!(global_id <= gpu::CONTOUR_ID_MASK);
                            span.contour_id_with_flags =
                                (span.contour_id_with_flags & !gpu::CONTOUR_ID_MASK) | global_id;
                        }
                    }
                    for contour in &mut draw_contours {
                        contour.path_id = u32::from(path_id);
                    }
                    for triangle in &mut triangles {
                        triangle.weight_path_id =
                            (triangle.weight_path_id & !0xffff) | i32::from(path_id);
                    }
                    let atlas = (draw.paint.feather != 0.0
                        && draw::feather_requires_atlas(
                            draw.paint.feather,
                            draw.state.transform,
                            false,
                        ))
                    .then(|| {
                        feather_atlas_placement(
                            &draw.path.raw_path,
                            draw.state.transform,
                            draw.paint.feather,
                            draw.paint.effective_stroke(),
                            self.width,
                            self.height,
                        )
                        .expect("atomic eligibility already validated feather bounds")
                    });
                    if let Some(placement) = atlas {
                        path.atlas_transform = gpu::AtlasTransform {
                            scale_factor: placement.scale,
                            translate_x: placement.translate[0],
                            translate_y: placement.translate[1],
                        };
                    }
                    let is_outermost_clip =
                        matches!(draw.role, DrawRole::ClipUpdate { parent_id: 0, .. });
                    if use_clockwise_atomic_batch && !is_outermost_clip {
                        let coverage_bounds = if inverse_clip_path.is_some() {
                            draw::path_pixel_bounds(raw_path, draw.state.transform)
                        } else {
                            path_draw_pixel_bounds(&draw.path, &draw.paint, draw.state.transform)
                        }
                        .expect("atomic eligibility already validated visible path bounds");
                        let (range, word_count) =
                            draw::clockwise_atomic_coverage_range_from_bounds(
                                coverage_bounds,
                                self.width,
                                self.height,
                                clockwise_atomic_coverage_words,
                            )
                            .expect("atomic eligibility already validated visible path bounds");
                        path.coverage_buffer_range = range;
                        clockwise_atomic_coverage_words += word_count;
                    } else {
                        path.coverage_buffer_range.pitch = padded_width;
                    }
                    paths.push(path);
                    let mut paint = match draw.role {
                        DrawRole::ClipUpdate {
                            replacement_id,
                            parent_id,
                        } => gpu::PaintData::clip_update(replacement_id, parent_id, fill_rule),
                        DrawRole::Content { clip_id } => {
                            let paint = if let Some(gradient) = gradient_batch.draws[draw_index] {
                                if draw.paint.style == RenderPaintStyle::Stroke {
                                    gpu::PaintData::gradient_stroke(
                                        gradient.paint_type,
                                        gradient.texture_y,
                                        draw.paint.blend_mode,
                                    )
                                } else {
                                    gpu::PaintData::gradient(
                                        gradient.paint_type,
                                        gradient.texture_y,
                                        paint_fill_rule,
                                        draw.paint.blend_mode,
                                    )
                                }
                            } else if draw.paint.style == RenderPaintStyle::Stroke {
                                gpu::PaintData::solid_stroke(
                                    modulate_color_alpha(draw.paint.color, draw.state.opacity),
                                    draw.paint.blend_mode,
                                )
                            } else {
                                gpu::PaintData::solid(
                                    modulate_color_alpha(draw.paint.color, draw.state.opacity),
                                    paint_fill_rule,
                                    draw.paint.blend_mode,
                                )
                            };
                            paint.with_clip_id(clip_id)
                        }
                        DrawRole::ClipReset { .. } => {
                            unreachable!("MSAA clip reset escaped atomic preparation")
                        }
                    };
                    if !use_clockwise_atomic_batch
                        && draw.paint.style == RenderPaintStyle::Fill
                        && draw.paint.feather != 0.0
                        && source_fill_rule == FillRule::Clockwise
                    {
                        paint = paint.with_generic_clockwise_fill();
                    }
                    if draw.state.clip_rect.is_some() {
                        paint = paint.with_clip_rect();
                    }
                    paints.push(paint);
                    paint_aux.push(gradient_batch.draws[draw_index].map_or_else(
                        || clip_rect_paint_aux(draw.state.clip_rect),
                        |gradient| gradient_paint_aux(draw.state.clip_rect, gradient),
                    ));
                    let contour_start = contours.len();
                    contours.extend(draw_contours);
                    let contour_range = contour_start..contours.len();
                    let atlas_blit_vertices = atlas
                        .map(|placement| {
                            let [left, top, right, bottom] = placement.bounds;
                            vec![
                                gpu::TriangleVertex::new([left, bottom], 1, path_id),
                                gpu::TriangleVertex::new([left, top], 1, path_id),
                                gpu::TriangleVertex::new([right, bottom], 1, path_id),
                                gpu::TriangleVertex::new([right, bottom], 1, path_id),
                                gpu::TriangleVertex::new([left, top], 1, path_id),
                                gpu::TriangleVertex::new([right, top], 1, path_id),
                            ]
                        })
                        .unwrap_or_default();
                    prepared.push(PreparedAtomicDraw {
                        spans,
                        base_instance,
                        instance_count,
                        patch_index_range,
                        contour_range,
                        tessellation_index: 0,
                        triangles,
                        atlas,
                        atlas_blit_vertices,
                        is_stroke: draw.paint.style == RenderPaintStyle::Stroke,
                        is_feather: draw.paint.feather != 0.0,
                        batchable_direct_stroke: direct_stroke_can_batch(
                            draw,
                            gradient_batch.draws[draw_index].is_some(),
                        ),
                        hsl_blend: blend_mode_uses_hsl(draw.paint.blend_mode),
                        image: draw.image.as_ref().map(|image| Arc::clone(image.texture())),
                        image_sampler: draw
                            .image
                            .as_ref()
                            .map(ImageDraw::sampler)
                            .unwrap_or_default(),
                        image_uniforms: draw.image.as_ref().map(|image| {
                            gpu::ImageDrawUniforms::new(
                                draw.state.transform,
                                image.opacity(),
                                image_clip_rect_inverse_matrix(draw.state.clip_rect),
                                match draw.role {
                                    DrawRole::Content { clip_id } => clip_id,
                                    DrawRole::ClipUpdate { .. } => 0,
                                    DrawRole::ClipReset { .. } => {
                                        unreachable!("MSAA clip reset escaped image preparation")
                                    }
                                },
                                image.blend_mode(),
                                draw_index as u32 + 1,
                            )
                        }),
                        image_mesh: match &draw.image {
                            Some(ImageDraw::Mesh(mesh)) => Some(PreparedImageMesh {
                                vertices: Arc::clone(&mesh.vertices),
                                uvs: Arc::clone(&mesh.uvs),
                                indices: Arc::clone(&mesh.indices),
                                index_count: mesh.index_count,
                            }),
                            _ => None,
                        },
                    });
                }
                let atlas_regions = prepared
                    .iter()
                    .filter_map(|draw| draw.atlas.map(|atlas| (atlas.width, atlas.height)))
                    .collect::<Vec<_>>();
                let max_atlas_region_width = atlas_regions
                    .iter()
                    .map(|&(width, _)| width)
                    .max()
                    .unwrap_or(1);
                let pack_width = self.width.max(1).max(max_atlas_region_width);
                let max_atlas_dimension = self.context.device.limits().max_texture_dimension_2d;
                let atlas_layout =
                    pack_atlas_for_device(pack_width, max_atlas_dimension, &atlas_regions)?;
                let mut atlas_origins = atlas_layout.origins().iter().copied();
                for (index, draw) in prepared.iter_mut().enumerate() {
                    let Some(atlas) = &mut draw.atlas else {
                        continue;
                    };
                    let [atlas_x, atlas_y] = atlas_origins
                        .next()
                        .expect("atlas layout must include every atlas region");
                    atlas.origin = [atlas_x, atlas_y];
                    atlas.translate[0] += atlas_x as f32;
                    atlas.translate[1] += atlas_y as f32;
                    paths[index + 1].atlas_transform.translate_x = atlas.translate[0];
                    paths[index + 1].atlas_transform.translate_y = atlas.translate[1];
                }
                debug_assert!(atlas_origins.next().is_none());
                let atlas_content_size = atlas_layout.extent();
                let atlas_physical_size =
                    atlas_physical_size(atlas_content_size, max_atlas_dimension);
                let [atlas_width, atlas_height] = atlas_content_size;
                let share_midpoint_tessellation = load_color.is_none()
                    && !force_clockwise_atomic_batch
                    && draws
                        .iter()
                        .all(|draw| !matches!(draw.role, DrawRole::ClipUpdate { .. }))
                    && prepared.iter().all(|draw| {
                        draw.image.is_none()
                            && draw.triangles.is_empty()
                            && draw.atlas.is_none()
                            && !draw.is_feather
                            && draw.patch_index_range.end
                                <= (gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT
                                    + gpu::MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT)
                                    as u32
                    });
                let midpoint_span = gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
                // C++ RenderContext::LogicalFlush emits midpoint padding once around the flush.
                // Layout sharing is independent of whether translucent fills can share a draw.
                let compact_shared_midpoint_end = (share_midpoint_tessellation
                    && prepared.len() > 1
                    && gradient_batch.draws.iter().all(Option::is_none)
                    && draws.iter().all(|draw| {
                        matches!(draw.role, DrawRole::Content { clip_id: 0 })
                            && draw.state.clip_rect.is_none()
                            && draw.paint.blend_mode == BlendMode::SrcOver
                    })
                    && (prepared.iter().all(|draw| draw.is_stroke)
                        || draws.iter().all(|draw| {
                            draw.paint.style == RenderPaintStyle::Fill
                                && draw.path.fill_rule == FillRule::NonZero
                        }))
                    && prepared.iter().all(|draw| {
                        draw.base_instance == 1
                            && midpoint_tessellation_single_row_width(&draw.spans).is_some()
                    }))
                .then(|| {
                    prepared.iter().try_fold(midpoint_span, |end, draw| {
                        draw.instance_count
                            .checked_mul(midpoint_span)
                            .and_then(|count| end.checked_add(count))
                    })
                })
                .flatten()
                .filter(|&end| {
                    end <= gpu::TESS_TEXTURE_WIDTH as u32
                        && align_to(end, gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32)
                            < gpu::TESS_TEXTURE_WIDTH as u32
                });
                let compact_shared_stroke_end = (share_midpoint_tessellation
                    && prepared.iter().all(|draw| draw.batchable_direct_stroke))
                .then(|| {
                    prepared.iter().try_fold(midpoint_span, |end, draw| {
                        draw.instance_count
                            .checked_mul(midpoint_span)
                            .and_then(|count| end.checked_add(count))
                    })
                })
                .flatten()
                .filter(|&geometry_end| {
                    align_to(geometry_end, gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32)
                        .checked_add(1)
                        .is_some_and(|final_end| {
                            final_end.div_ceil(gpu::TESS_TEXTURE_WIDTH as u32)
                                <= max_atlas_dimension
                        })
                });
                let outer_patch_range = (gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT
                    + gpu::MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT)
                    as u32
                    ..(gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT
                        + gpu::MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT
                        + gpu::OUTER_CURVE_PATCH_INDEX_COUNT) as u32;
                let outer_segment_span = gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32;
                let shared_outer_end = prepared.iter().try_fold(outer_segment_span, |end, draw| {
                    draw.instance_count
                        .checked_mul(outer_segment_span)
                        .and_then(|count| end.checked_add(count))
                });
                let share_outer_tessellation = load_color.is_none()
                    && !force_clockwise_atomic_batch
                    && gradient_batch.draws.iter().all(Option::is_none)
                    && draws.iter().all(|draw| {
                        matches!(draw.role, DrawRole::Content { clip_id: 0 })
                            && draw.state.clip_rect.is_none()
                            && draw.paint.blend_mode == BlendMode::SrcOver
                    })
                    && prepared.iter().all(|draw| {
                        draw.image.is_none()
                            && !draw.triangles.is_empty()
                            && draw.atlas.is_none()
                            && !draw.is_feather
                            && !draw.is_stroke
                            && draw.base_instance == 1
                            && draw.patch_index_range == outer_patch_range
                            && draw.spans.iter().all(|span| span.y == 0.0)
                    })
                    && shared_outer_end.is_some_and(|end| {
                        end.checked_add(1)
                            .is_some_and(|end| end <= gpu::TESS_TEXTURE_WIDTH as u32)
                    });
                let mixed_midpoint_draw = |draw: &PreparedAtomicDraw| {
                    draw.image.is_none()
                        && draw.triangles.is_empty()
                        && draw.atlas.is_none()
                        && !draw.is_feather
                        && draw.base_instance == 1
                        && !draw.spans.is_empty()
                        && draw.patch_index_range.start
                            < (gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT
                                + gpu::MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT)
                                as u32
                        && draw.patch_index_range.end
                            <= (gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT
                                + gpu::MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT)
                                as u32
                };
                let mixed_outer_draw = |draw: &PreparedAtomicDraw| {
                    draw.image.is_none()
                        && draw.atlas.is_none()
                        && !draw.is_feather
                        && !draw.is_stroke
                        && draw.base_instance == 1
                        && !draw.spans.is_empty()
                        && draw.patch_index_range == outer_patch_range
                };
                let share_mixed_tessellation = load_color.is_none()
                    && gradient_batch.draws.iter().all(Option::is_none)
                    && draws.iter().all(|draw| {
                        draw.image.is_none()
                            && draw.paint.feather == 0.0
                            && draw.paint.blend_mode == BlendMode::SrcOver
                            && draw.state.clip_rect.is_none()
                    })
                    && prepared.iter().any(|draw| mixed_midpoint_draw(draw))
                    && prepared.iter().any(|draw| mixed_outer_draw(draw))
                    && prepared
                        .iter()
                        .all(|draw| mixed_midpoint_draw(draw) || mixed_outer_draw(draw));
                let mut tessellation_span_batches = Vec::new();
                let mut tessellation_heights = Vec::new();
                let mut needs_dummy_tessellation = false;
                if share_mixed_tessellation {
                    let mut packed = Vec::new();
                    append_tessellation_padding_span(&mut packed, 0, midpoint_span);

                    let mut next_midpoint_base = 1u32;
                    for draw in prepared.iter_mut().filter(|draw| mixed_midpoint_draw(draw)) {
                        draw.spans
                            .retain(|span| span.contour_id_with_flags & gpu::CONTOUR_ID_MASK != 0);
                        relocate_tessellation_logically(
                            &mut draw.spans,
                            &mut draw.base_instance,
                            &mut contours[draw.contour_range.clone()],
                            next_midpoint_base,
                            midpoint_span,
                        );
                        packed.append(&mut draw.spans);
                        next_midpoint_base = next_midpoint_base
                            .checked_add(draw.instance_count)
                            .expect("mixed atomic midpoint instance range overflow");
                        draw.tessellation_index = 0;
                    }
                    let midpoint_end = next_midpoint_base
                        .checked_mul(midpoint_span)
                        .expect("mixed atomic midpoint end overflow");
                    let outer_start = align_to(midpoint_end, outer_segment_span);
                    append_tessellation_padding_span(&mut packed, midpoint_end, outer_start);

                    let mut next_outer_base = outer_start / outer_segment_span;
                    for draw in prepared.iter_mut().filter(|draw| mixed_outer_draw(draw)) {
                        draw.spans
                            .retain(|span| span.contour_id_with_flags & gpu::CONTOUR_ID_MASK != 0);
                        relocate_tessellation_logically(
                            &mut draw.spans,
                            &mut draw.base_instance,
                            &mut contours[draw.contour_range.clone()],
                            next_outer_base,
                            outer_segment_span,
                        );
                        packed.append(&mut draw.spans);
                        next_outer_base = next_outer_base
                            .checked_add(draw.instance_count)
                            .expect("mixed atomic outer instance range overflow");
                        draw.tessellation_index = 0;
                    }
                    let outer_end = next_outer_base
                        .checked_mul(outer_segment_span)
                        .expect("mixed atomic outer end overflow");
                    let final_end = outer_end
                        .checked_add(1)
                        .expect("mixed atomic final padding overflow");
                    append_tessellation_padding_span(&mut packed, outer_end, final_end);
                    let tessellation_height = final_end.div_ceil(gpu::TESS_TEXTURE_WIDTH as u32);
                    if tessellation_height > max_atlas_dimension {
                        return Err(RendererError::Device(
                            "tessellation texture exceeds device dimension limit".into(),
                        ));
                    }
                    tessellation_span_batches.push(packed);
                    tessellation_heights.push(tessellation_height);
                } else if let Some(geometry_end) = compact_shared_midpoint_end {
                    let mut packed = Vec::new();
                    append_tessellation_padding_span(&mut packed, 0, midpoint_span);
                    let mut next_base_instance = 1u32;
                    for draw in &mut prepared {
                        let relocation = next_base_instance
                            .checked_sub(draw.base_instance)
                            .and_then(|instances| instances.checked_mul(midpoint_span))
                            .expect("compact atomic midpoint relocation overflow");
                        draw.spans
                            .retain(|span| span.contour_id_with_flags & gpu::CONTOUR_ID_MASK != 0);
                        relocate_midpoint_tessellation(
                            &mut draw.spans,
                            &mut draw.base_instance,
                            &mut contours[draw.contour_range.clone()],
                            relocation,
                            0,
                        );
                        packed.append(&mut draw.spans);
                        next_base_instance = next_base_instance
                            .checked_add(draw.instance_count)
                            .expect("compact atomic midpoint instance range overflow");
                        draw.tessellation_index = 0;
                    }
                    debug_assert_eq!(next_base_instance * midpoint_span, geometry_end);
                    let outer_curve_start =
                        align_to(geometry_end, gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32);
                    append_tessellation_padding_span(&mut packed, geometry_end, outer_curve_start);
                    append_tessellation_padding_span(
                        &mut packed,
                        outer_curve_start,
                        outer_curve_start + 1,
                    );
                    tessellation_span_batches.push(packed);
                    tessellation_heights.push(1);
                } else if let Some(geometry_end) = compact_shared_stroke_end {
                    let mut packed = Vec::new();
                    append_tessellation_padding_span(&mut packed, 0, midpoint_span);
                    let mut next_base_instance = 1u32;
                    for draw in &mut prepared {
                        draw.spans
                            .retain(|span| span.contour_id_with_flags & gpu::CONTOUR_ID_MASK != 0);
                        relocate_tessellation_logically(
                            &mut draw.spans,
                            &mut draw.base_instance,
                            &mut contours[draw.contour_range.clone()],
                            next_base_instance,
                            midpoint_span,
                        );
                        packed.append(&mut draw.spans);
                        next_base_instance = next_base_instance
                            .checked_add(draw.instance_count)
                            .expect("compact atomic stroke instance range overflow");
                        draw.tessellation_index = 0;
                    }
                    debug_assert_eq!(next_base_instance * midpoint_span, geometry_end);
                    let outer_curve_start =
                        align_to(geometry_end, gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32);
                    append_tessellation_padding_span(&mut packed, geometry_end, outer_curve_start);
                    let final_end = outer_curve_start + 1;
                    append_tessellation_padding_span(&mut packed, outer_curve_start, final_end);
                    tessellation_span_batches.push(packed);
                    tessellation_heights.push(final_end.div_ceil(gpu::TESS_TEXTURE_WIDTH as u32));
                } else if share_midpoint_tessellation {
                    let mut packed = Vec::new();
                    let mut cursor_x = 0u32;
                    let mut cursor_y = 0u32;
                    let mut packed_height = 1u32;
                    for draw in &mut prepared {
                        let local_height = draw::tessellation_texture_height(&draw.spans);
                        let single_row_width = (local_height == 1)
                            .then(|| midpoint_tessellation_single_row_width(&draw.spans))
                            .flatten();
                        let mut placement = midpoint_shelf_placement(
                            cursor_x,
                            cursor_y,
                            local_height,
                            single_row_width,
                        );
                        if placement.height > max_atlas_dimension && !packed.is_empty() {
                            tessellation_span_batches.push(std::mem::take(&mut packed));
                            tessellation_heights.push(packed_height);
                            cursor_x = 0;
                            cursor_y = 0;
                            packed_height = 1;
                            placement = midpoint_shelf_placement(
                                cursor_x,
                                cursor_y,
                                local_height,
                                single_row_width,
                            );
                        }
                        if placement.height > max_atlas_dimension {
                            return Err(RendererError::Device(
                                "tessellation texture exceeds device dimension limit".into(),
                            ));
                        }
                        let (x, y) = (placement.x, placement.y);
                        cursor_x = placement.next_x;
                        cursor_y = placement.next_y;
                        packed_height = packed_height.max(placement.height);
                        relocate_midpoint_tessellation(
                            &mut draw.spans,
                            &mut draw.base_instance,
                            &mut contours[draw.contour_range.clone()],
                            x,
                            y,
                        );
                        packed.append(&mut draw.spans);
                        draw.tessellation_index = tessellation_span_batches.len();
                    }
                    if !packed.is_empty() {
                        tessellation_span_batches.push(packed);
                        tessellation_heights.push(packed_height);
                    }
                } else if share_outer_tessellation {
                    let mut packed = vec![gpu::TessVertexSpan::without_reflection(
                        [[0.0; 2]; 4],
                        [0.0; 2],
                        0.0,
                        0,
                        outer_segment_span as i32,
                        0,
                        0,
                        1,
                        0,
                    )];
                    let mut next_base_instance = 1u32;
                    for draw in &mut prepared {
                        let relocation = (next_base_instance - draw.base_instance)
                            .checked_mul(outer_segment_span)
                            .expect("outer tessellation relocation overflow");
                        relocate_tessellation(
                            &mut draw.spans,
                            &mut draw.base_instance,
                            &mut contours[draw.contour_range.clone()],
                            relocation,
                            0,
                            outer_segment_span,
                        );
                        draw.spans
                            .retain(|span| span.contour_id_with_flags & gpu::CONTOUR_ID_MASK != 0);
                        packed.append(&mut draw.spans);
                        next_base_instance = next_base_instance
                            .checked_add(draw.instance_count)
                            .expect("outer tessellation instance range overflow");
                        draw.tessellation_index = 0;
                    }
                    let final_vertex = next_base_instance
                        .checked_mul(outer_segment_span)
                        .expect("outer tessellation final padding overflow");
                    packed.push(gpu::TessVertexSpan::without_reflection(
                        [[0.0; 2]; 4],
                        [0.0; 2],
                        0.0,
                        final_vertex as i32,
                        final_vertex as i32 + 1,
                        0,
                        0,
                        1,
                        0,
                    ));
                    tessellation_span_batches.push(packed);
                    tessellation_heights.push(1);
                } else {
                    for draw in &mut prepared {
                        if draw.spans.is_empty() {
                            needs_dummy_tessellation = true;
                            draw.tessellation_index = usize::MAX;
                            continue;
                        }
                        draw.tessellation_index = tessellation_span_batches.len();
                        tessellation_heights.push(draw::tessellation_texture_height(&draw.spans));
                        tessellation_span_batches.push(std::mem::take(&mut draw.spans));
                    }
                    let common_height = tessellation_heights.iter().copied().max().unwrap_or(1);
                    tessellation_heights.fill(common_height);
                }
                let tessellation_height = tessellation_heights.iter().copied().max().unwrap_or(1);
                let mut uniforms = analytic_uniforms(self.width, self.height, tessellation_height);
                if gradient_batch.height != 0 {
                    uniforms.inverse_viewports[0] = -2.0 / gradient_batch.height as f32;
                }
                uniforms.color_clear_value = swizzle_rive_color_to_rgba_premul(self.clear_color);
                uniforms.max_path_id = u32::try_from(paths.len() - 1).expect("path ID overflow");
                uniforms.render_target_update_bounds =
                    [0, 0, self.width as i32, self.height as i32];
                uniforms.atlas_texture_inverse_size = [
                    1.0 / atlas_physical_size[0] as f32,
                    1.0 / atlas_physical_size[1] as f32,
                ];
                uniforms.atlas_content_inverse_viewport =
                    [2.0 / atlas_width as f32, -2.0 / atlas_height as f32];
                if use_clockwise_atomic_batch {
                    uniforms.coverage_buffer_prefix = 1 << 20;
                }
                let mut tessellation_textures = Vec::with_capacity(tessellation_span_batches.len());
                let mut tessellation_flush_resources = None;
                for (spans, height) in tessellation_span_batches
                    .iter()
                    .zip(tessellation_heights.iter().copied())
                {
                    let tessellation_texture = if let Some(flush_resources) =
                        &tessellation_flush_resources
                    {
                        self.context.tessellator.encode_with_flush_resources(
                            &self.context.device,
                            &mut tessellation_uploads.borrow_mut(),
                            encoder,
                            &self.context.feather_lut.view,
                            spans,
                            flush_resources,
                            height,
                        )
                    } else {
                        let encoding = self.context.tessellator.encode_with_new_flush_resources(
                            &self.context.device,
                            &mut tessellation_uploads.borrow_mut(),
                            encoder,
                            &self.context.feather_lut.view,
                            spans,
                            &uniforms,
                            &paths,
                            &contours,
                            height,
                        );
                        tessellation_flush_resources = Some(encoding.flush_resources);
                        encoding.texture
                    };
                    tessellation_textures.push(tessellation_texture);
                }
                let tessellation_flush_resources =
                    tessellation_flush_resources.unwrap_or_else(|| {
                        tessellation_uploads.borrow_mut().upload_flush_resources(
                            &self.context.device,
                            &uniforms,
                            &paths,
                            &contours,
                        )
                    });
                if needs_dummy_tessellation {
                    let dummy_index = tessellation_textures.len();
                    tessellation_textures.push(self.context.device.create_texture(
                        &wgpu::TextureDescriptor {
                            label: Some("nuxie-dummy-tessellation-data"),
                            size: wgpu::Extent3d {
                                width: 1,
                                height: 1,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba32Uint,
                            usage: wgpu::TextureUsages::TEXTURE_BINDING,
                            view_formats: &[],
                        },
                    ));
                    for draw in &mut prepared {
                        if draw.tessellation_index == usize::MAX {
                            draw.tessellation_index = dummy_index;
                        }
                    }
                }
                let tessellation_views = tessellation_textures
                    .iter()
                    .map(|texture| texture.create_view(&wgpu::TextureViewDescriptor::default()))
                    .collect::<Vec<_>>();
                let gradient_texture = self.context.gradient_pipeline.encode(
                    &self.context.device,
                    encoder,
                    &uniforms,
                    &gradient_batch.spans,
                    gradient_batch.height,
                );
                let atlas_texture = prepared.iter().any(|draw| draw.atlas.is_some()).then(|| {
                    let texture = self
                        .context
                        .device
                        .create_texture(&wgpu::TextureDescriptor {
                            label: Some("nuxie-feather-atlas"),
                            size: wgpu::Extent3d {
                                width: atlas_physical_size[0],
                                height: atlas_physical_size[1],
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::R16Float,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                                | wgpu::TextureUsages::TEXTURE_BINDING,
                            view_formats: &[],
                        });
                    let view = texture.create_view(&Default::default());
                    let mut clear = true;
                    for draw in &prepared {
                        if let Some(atlas) = draw.atlas {
                            self.context.atlas_pipeline.encode_mask(
                                &self.context.device,
                                encoder,
                                &view,
                                &self.context.patch_vertex_buffer,
                                &self.context.patch_index_buffer,
                                &tessellation_views[draw.tessellation_index],
                                &self.context.feather_lut.view,
                                &uniforms,
                                &paths,
                                &paints,
                                &paint_aux,
                                &contours,
                                draw.base_instance,
                                draw.instance_count,
                                draw.is_stroke,
                                clear,
                                atlas_content_size,
                                [atlas.origin[0], atlas.origin[1], atlas.width, atlas.height],
                            );
                            clear = false;
                        }
                    }
                    texture
                });
                let atlas_view = atlas_texture
                    .as_ref()
                    .map(|texture| texture.create_view(&Default::default()));
                let atomic_draws = prepared
                    .iter()
                    .map(|draw| atomic_pipeline::AtomicDraw {
                        tessellation: &tessellation_views[draw.tessellation_index],
                        base_instance: draw.base_instance,
                        instance_count: draw.instance_count,
                        patch_index_range: draw.patch_index_range.clone(),
                        triangle_vertices: &draw.triangles,
                        atlas: draw.atlas.and(atlas_view.as_ref()),
                        atlas_blit_vertices: &draw.atlas_blit_vertices,
                        is_stroke: draw.is_stroke,
                        is_feather: draw.is_feather,
                        batchable_direct_stroke: draw.batchable_direct_stroke,
                        hsl_blend: draw.hsl_blend,
                        image: draw.image.as_ref().map(|image| &image.view),
                        image_sampler: draw.image_sampler,
                        image_uniforms: draw.image_uniforms,
                        image_mesh: draw.image_mesh.as_ref().map(|mesh| {
                            atomic_pipeline::ImageMeshBuffers {
                                vertices: &mesh.vertices,
                                uvs: &mesh.uvs,
                                indices: &mesh.indices,
                                index_count: mesh.index_count,
                            }
                        }),
                    })
                    .collect::<Vec<_>>();
                if use_clockwise_atomic_batch {
                    let advanced_texture = load_color.map(|_| {
                        self.context
                            .device
                            .create_texture(&wgpu::TextureDescriptor {
                                label: Some("nuxie-cwa-advanced-source"),
                                size: texture.size(),
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: wgpu::TextureDimension::D2,
                                format: wgpu::TextureFormat::Rgba8Unorm,
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                                    | wgpu::TextureUsages::TEXTURE_BINDING,
                                view_formats: &[],
                            })
                    });
                    let advanced_view = advanced_texture
                        .as_ref()
                        .map(|texture| texture.create_view(&Default::default()));
                    if let Some(advanced_view) = &advanced_view {
                        let attachments = [Some(wgpu::RenderPassColorAttachment {
                            view: advanced_view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        })];
                        let _pass =
                            encoder.begin_counted_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("nuxie-cwa-advanced-source-clear"),
                                color_attachments: &attachments,
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                                multiview_mask: None,
                            });
                    }
                    let borrowed_triangles = prepared
                        .iter()
                        .map(|draw| {
                            draw.triangles
                                .iter()
                                .copied()
                                .filter(|vertex| vertex.weight_path_id >> 16 < 0)
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    let main_triangles = prepared
                        .iter()
                        .zip(draws)
                        .map(|(prepared, source)| {
                            clockwise_atomic_main_triangles(
                                &prepared.triangles,
                                matches!(source.role, DrawRole::Content { clip_id: 0 })
                                    && clockwise_atomic_clip_is_inactive(source),
                            )
                        })
                        .collect::<Vec<_>>();
                    let clockwise_atomic_draws = prepared
                        .iter()
                        .zip(draws)
                        .zip(&borrowed_triangles)
                        .zip(&main_triangles)
                        .map(
                            |(((prepared, source), borrowed_triangles), main_triangles)| {
                                let tessellation =
                                    &tessellation_views[prepared.tessellation_index];
                                let kind = match source.role {
                                    DrawRole::Content { clip_id: 0 } => {
                                        clockwise_atomic_pipeline::ClockwiseAtomicDrawKind::Content
                                    }
                                    DrawRole::Content { .. } => clockwise_atomic_pipeline::ClockwiseAtomicDrawKind::ClippedContent,
                                    DrawRole::ClipUpdate { parent_id: 0, .. } => clockwise_atomic_pipeline::ClockwiseAtomicDrawKind::OutermostClip,
                                    DrawRole::ClipUpdate { .. } => clockwise_atomic_pipeline::ClockwiseAtomicDrawKind::NestedClip,
                                    DrawRole::ClipReset { .. } => {
                                        unreachable!("MSAA clip reset escaped clockwise-atomic preparation")
                                    }
                                };
                                let (main_base_instance, instance_count) = if kind
                                    == clockwise_atomic_pipeline::ClockwiseAtomicDrawKind::OutermostClip
                                {
                                    (prepared.base_instance, prepared.instance_count)
                                } else {
                                    assert_eq!(prepared.instance_count % 2, 0);
                                    let instance_count = prepared.instance_count / 2;
                                    (prepared.base_instance + instance_count, instance_count)
                                };
                                clockwise_atomic_pipeline::ClockwiseAtomicDraw {
                                    tessellation,
                                    borrowed_base_instance: prepared.base_instance,
                                    main_base_instance,
                                    instance_count,
                                    patch_index_range: prepared.patch_index_range.clone(),
                                    borrowed_triangles,
                                    main_triangles: &main_triangles.vertices,
                                    main_triangle_batches: &main_triangles.batches,
                                    kind,
                                    has_clip_rect: source.state.clip_rect.is_some(),
                                }
                            },
                        )
                        .collect::<Vec<_>>();
                    let coverage_readback = self.context.clockwise_atomic_pipeline.encode_fills(
                        &self.context.device,
                        encoder,
                        advanced_view.as_ref().unwrap_or(&view),
                        &self.context.feather_lut.view,
                        gradient_texture.as_ref().map(|texture| &texture.view),
                        &self.context.patch_vertex_buffer,
                        &self.context.patch_index_buffer,
                        &clockwise_atomic_draws,
                        &uniforms,
                        &tessellation_flush_resources,
                        &paints,
                        &paint_aux,
                        clockwise_atomic_coverage_words,
                        capture_clockwise_atomic_coverage,
                    );
                    if let Some(readback) = coverage_readback {
                        pending_coverage_readbacks.push((
                            readback,
                            paths
                                .iter()
                                .skip(1)
                                .map(|path| path.coverage_buffer_range)
                                .collect::<Vec<_>>(),
                            clockwise_atomic_draws
                                .iter()
                                .map(|draw| draw.kind)
                                .collect::<Vec<_>>(),
                        ));
                    }
                    if let (Some(source), Some(destination)) = (&advanced_view, load_color) {
                        if draws.len() != 1 || !draw_uses_advanced_blend(&draws[0]) {
                            return Err(RendererError::Unsupported(
                                "batched advanced clockwise-atomic blending",
                            ));
                        }
                        self.context.composite_pipeline.encode_advanced(
                            &self.context.device,
                            encoder,
                            &view,
                            source,
                            destination,
                            gpu::blend_mode_id(draws[0].paint.blend_mode),
                        );
                    }
                } else {
                    let mut atomic_backing = atomic_backing.borrow_mut();
                    let atomic_backing = atomic_backing
                        .get_or_insert_with(|| self.context.atomic_pipeline.begin_frame_backing());
                    let batch_shared_draws = draws
                        .iter()
                        .all(|draw| matches!(draw.role, DrawRole::Content { clip_id: 0 }));
                    let readbacks = self.context.atomic_pipeline.encode_batch(
                        &self.context.device,
                        atomic_backing,
                        encoder,
                        &view,
                        load_color,
                        &self.context.feather_lut.view,
                        gradient_texture.as_ref().map(|texture| &texture.view),
                        &self.context.patch_vertex_buffer,
                        &self.context.patch_index_buffer,
                        &atomic_draws,
                        draw_group_starts,
                        batch_shared_draws,
                        &tessellation_flush_resources,
                        &paints,
                        &paint_aux,
                        padded_width as usize * padded_height as usize,
                        capture_atomic_planes,
                    );
                    if let Some(readback) = readbacks.coverage {
                        pending_atomic_coverage_readbacks.push(readback);
                    }
                    if let Some(readback) = readbacks.clip {
                        pending_atomic_clip_readbacks.push(readback);
                    }
                    if let Some(readback) = readbacks.color {
                        pending_atomic_color_readbacks.push(readback);
                    }
                }
                Ok::<(), RendererError>(())
            };
        let encode_fallback_run =
            |draws: &[SolidDraw],
             draw_groups: Option<&[u32]>,
             draw_prepasses: Option<&[bool]>,
             logical_flush_starts: &[usize],
             clear_target: bool,
             encoder: &mut wgpu::CommandEncoder| {
                debug_assert!(draw_groups.is_none_or(|groups| groups.len() == draws.len()));
                debug_assert!(draw_prepasses.is_none_or(|prepasses| prepasses.len() == draws.len()));
                debug_assert!(logical_flush_starts.first().is_none_or(|start| *start == 0));
                let has_advanced_msaa = draws.iter().any(draw_uses_advanced_blend);
                let resolve_fixed_msaa_directly = clear_target && !has_advanced_msaa;
                let destination_texture = has_advanced_msaa.then(|| {
                    self.context
                        .device
                        .create_texture(&wgpu::TextureDescriptor {
                            label: Some("nuxie-msaa-destination-copy"),
                            size: texture.size(),
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            usage: wgpu::TextureUsages::COPY_DST
                                | wgpu::TextureUsages::TEXTURE_BINDING,
                            view_formats: &[],
                        })
                });
                let destination_view = destination_texture
                    .as_ref()
                    .map(|texture| texture.create_view(&Default::default()));
                #[derive(Clone, Copy, PartialEq, Eq)]
                struct DirectPathOptions {
                    path_clip: bool,
                    clip_rect: bool,
                    opaque: bool,
                    advanced_blend: bool,
                    hsl_blend: bool,
                    batchable_midpoint_fill: bool,
                    batchable_direct_stroke: bool,
                    destination_copy_bounds: [u32; 4],
                }
                #[derive(Clone, Copy)]
                struct ImageMeshOptions {
                    path_clip: bool,
                    clip_rect: bool,
                    advanced_blend: bool,
                    hsl_blend: bool,
                    destination_copy_bounds: [u32; 4],
                }
                struct PendingPathDraw {
                    tessellation: draw::FillTessellation,
                    paint: gpu::PaintData,
                    paint_aux: gpu::PaintAuxData,
                    image: Option<(wgpu::TextureView, ImageSampler)>,
                    compact_midpoint_layout: bool,
                    compact_midpoint_group: Option<(u32, bool)>,
                    logical_flush: usize,
                    prepared: Option<path_pipeline::PreparedPathDraw>,
                }
                enum PendingDraw {
                    Stroke(usize, DirectPathOptions),
                    Fill(usize, FillRule, DirectPathOptions),
                    ImageMesh(
                        msaa_image_mesh_pipeline::PreparedImageMesh,
                        ImageMeshOptions,
                    ),
                    OutermostClipUpdate(usize, FillRule),
                    NestedClipUpdate(usize, FillRule),
                    ClipReset(
                        msaa_stencil_pipeline::PreparedStencilDraw,
                        MsaaClipResetAction,
                    ),
                    Atlas(msaa_atlas_pipeline::PreparedAtlasBlit),
                    Bootstrap(wgpu::Buffer, wgpu::Buffer, FillRule),
                }
                enum PreparedDraw {
                    Stroke(path_pipeline::PreparedPathDraw, DirectPathOptions),
                    Fill(path_pipeline::PreparedPathDraw, FillRule, DirectPathOptions),
                    ImageMesh(
                        msaa_image_mesh_pipeline::PreparedImageMesh,
                        ImageMeshOptions,
                    ),
                    OutermostClipUpdate(path_pipeline::PreparedPathDraw, FillRule),
                    NestedClipUpdate(path_pipeline::PreparedPathDraw, FillRule),
                    ClipReset(
                        msaa_stencil_pipeline::PreparedStencilDraw,
                        MsaaClipResetAction,
                    ),
                    Atlas(msaa_atlas_pipeline::PreparedAtlasBlit),
                    Bootstrap(wgpu::Buffer, wgpu::Buffer, FillRule),
                }
                #[derive(Clone, Copy)]
                struct PreparedDrawSchedule {
                    draw_group: u32,
                    is_prepass: bool,
                    logical_flush: usize,
                }
                let gradient_batch = prepare_gradient_batch(draws);
                let mut gradient_uniforms = analytic_uniforms(self.width, self.height, 1);
                if gradient_batch.height != 0 {
                    gradient_uniforms.inverse_viewports[0] = -2.0 / gradient_batch.height as f32;
                }
                let gradient_texture = self.context.gradient_pipeline.encode(
                    &self.context.device,
                    encoder,
                    &gradient_uniforms,
                    &gradient_batch.spans,
                    gradient_batch.height,
                );
                let mut pending_draws = Vec::with_capacity(draws.len());
                let mut pending_paths = Vec::new();
                let mut prepared_schedules = Vec::with_capacity(draws.len());
                for (draw_index, draw) in draws.iter().enumerate() {
                    let z_index = draw_groups.map_or_else(
                        || {
                            u32::try_from(draw_index + 1)
                                .expect("MSAA draw index must fit the path-data contract")
                        },
                        |groups| groups[draw_index],
                    );
                    let schedule = PreparedDrawSchedule {
                        draw_group: z_index,
                        is_prepass: draw_prepasses.is_some_and(|prepasses| prepasses[draw_index]),
                        logical_flush: logical_flush_starts
                            .partition_point(|&start| start <= draw_index)
                            .saturating_sub(1),
                    };
                    if let DrawRole::ClipReset { bounds, action } = draw.role {
                        let uniforms = analytic_uniforms(self.width, self.height, 1);
                        pending_draws.push(PendingDraw::ClipReset(
                            self.context.msaa_stencil_pipeline.prepare_clip_reset(
                                &self.context.device,
                                &uniforms,
                                bounds,
                                u16::try_from(z_index)
                                    .expect("MSAA clip reset z-index must fit the shader contract"),
                            ),
                            action,
                        ));
                        prepared_schedules.push(schedule);
                        continue;
                    }
                    if let DrawRole::ClipUpdate { parent_id, .. } = draw.role {
                        let oriented_path = draw::msaa_fill_requires_reverse(
                            &draw.path.raw_path,
                            draw.state.transform,
                            draw.path.fill_rule,
                        )
                        .then(|| {
                            let mut path = RawPath::new();
                            path.add_path_backwards(&draw.path.raw_path, Mat2D::IDENTITY);
                            path
                        });
                        let raw_path = oriented_path.as_ref().unwrap_or(&draw.path.raw_path);
                        if let Some(mut tessellation) =
                            draw::build_fill_tessellation(raw_path, draw.state.transform)
                        {
                            tessellation.path.z_index = z_index;
                            let paint =
                                gpu::PaintData::solid(0, draw.path.fill_rule, BlendMode::SrcOver);
                            let path_index = pending_paths.len();
                            pending_paths.push(PendingPathDraw {
                                tessellation,
                                paint,
                                paint_aux: gpu::PaintAuxData::zeroed(),
                                image: None,
                                // C++ LogicalFlush shares padding across clip and content paths.
                                compact_midpoint_layout: true,
                                compact_midpoint_group: None,
                                logical_flush: schedule.logical_flush,
                                prepared: None,
                            });
                            pending_draws.push(if parent_id == 0 {
                                PendingDraw::OutermostClipUpdate(path_index, draw.path.fill_rule)
                            } else {
                                PendingDraw::NestedClipUpdate(path_index, draw.path.fill_rule)
                            });
                            prepared_schedules.push(schedule);
                        }
                        continue;
                    }
                    if let Some(ImageDraw::Mesh(mesh)) = &draw.image {
                        let path_clip = matches!(
                            draw.role,
                            DrawRole::Content { clip_id } if clip_id != 0
                        );
                        let clip_rect = draw.state.clip_rect.is_some();
                        if clip_rect && !self.context.msaa_image_mesh_pipeline.supports_clip_rect()
                        {
                            return Err(RendererError::Unsupported(
                                "clip rectangles on msaa image meshes",
                            ));
                        }
                        let advanced_blend = mesh.blend_mode != BlendMode::SrcOver;
                        let options = ImageMeshOptions {
                            path_clip,
                            clip_rect,
                            advanced_blend,
                            hsl_blend: blend_mode_uses_hsl(mesh.blend_mode),
                            destination_copy_bounds: msaa_destination_copy_bounds(
                                draw,
                                self.width,
                                self.height,
                            ),
                        };
                        let uniforms = analytic_uniforms(self.width, self.height, 1);
                        let clip_id = match draw.role {
                            DrawRole::Content { clip_id } => clip_id,
                            DrawRole::ClipUpdate { .. } | DrawRole::ClipReset { .. } => {
                                unreachable!("non-content draw carried an image mesh")
                            }
                        };
                        let image_uniforms = gpu::ImageDrawUniforms::new(
                            draw.state.transform,
                            mesh.opacity,
                            image_clip_rect_inverse_matrix(draw.state.clip_rect),
                            clip_id,
                            mesh.blend_mode,
                            z_index,
                        );
                        pending_draws.push(PendingDraw::ImageMesh(
                            self.context.msaa_image_mesh_pipeline.prepare(
                                &self.context.device,
                                &uniforms,
                                &image_uniforms,
                                if advanced_blend {
                                    destination_view.as_ref()
                                } else {
                                    None
                                },
                                &mesh.texture.view,
                                mesh.sampler,
                                &mesh.vertices,
                                &mesh.uvs,
                                &mesh.indices,
                                mesh.index_count,
                            ),
                            options,
                        ));
                        prepared_schedules.push(schedule);
                        continue;
                    }
                    if draw.paint.feather != 0.0
                        && (draw.state.clip_rect.is_none()
                            || self.context.msaa_atlas_pipeline.supports_clip_rect())
                    {
                        let stroke = draw.paint.effective_stroke();
                        let fill_direction = draw::feather_atlas_fill_direction(
                            draw.state.transform,
                            draw.path.fill_rule,
                            stroke.is_some(),
                        );
                        if let (Some(mut tessellation), Some(placement)) = (
                            draw::build_feather_tessellation_with_direction(
                                &draw.path.raw_path,
                                draw.state.transform,
                                draw.paint.feather,
                                stroke,
                                fill_direction,
                            ),
                            feather_atlas_placement(
                                &draw.path.raw_path,
                                draw.state.transform,
                                draw.paint.feather,
                                stroke,
                                self.width,
                                self.height,
                            ),
                        ) {
                            tessellation.path.atlas_transform = gpu::AtlasTransform {
                                scale_factor: placement.scale,
                                translate_x: placement.translate[0],
                                translate_y: placement.translate[1],
                            };
                            tessellation.path.z_index = z_index;
                            for contour in &mut tessellation.contours {
                                contour.path_id = 1;
                            }
                            let paths = [gpu::PathData::zeroed(), tessellation.path];
                            let gradient = gradient_batch.draws[draw_index];
                            let mut paint = if let Some(gradient) = gradient {
                                if draw.paint.style == RenderPaintStyle::Stroke {
                                    gpu::PaintData::gradient_stroke(
                                        gradient.paint_type,
                                        gradient.texture_y,
                                        draw.paint.blend_mode,
                                    )
                                } else {
                                    gpu::PaintData::gradient(
                                        gradient.paint_type,
                                        gradient.texture_y,
                                        draw.path.fill_rule,
                                        draw.paint.blend_mode,
                                    )
                                }
                            } else if draw.paint.style == RenderPaintStyle::Stroke {
                                gpu::PaintData::solid_stroke(
                                    modulate_color_alpha(draw.paint.color, draw.state.opacity),
                                    draw.paint.blend_mode,
                                )
                            } else {
                                gpu::PaintData::solid(
                                    modulate_color_alpha(draw.paint.color, draw.state.opacity),
                                    draw.path.fill_rule,
                                    draw.paint.blend_mode,
                                )
                            };
                            if draw.state.clip_rect.is_some() {
                                paint = paint.with_clip_rect();
                            }
                            let paints = [
                                gpu::PaintData::solid(0, FillRule::NonZero, BlendMode::SrcOver),
                                paint,
                            ];
                            let paint_aux = [
                                gpu::PaintAuxData::zeroed(),
                                gradient.map_or_else(
                                    || clip_rect_paint_aux(draw.state.clip_rect),
                                    |gradient| gradient_paint_aux(draw.state.clip_rect, gradient),
                                ),
                            ];
                            let tessellation_height =
                                draw::tessellation_texture_height(&tessellation.spans);
                            let atlas_content_size = [placement.width, placement.height];
                            let atlas_physical_size = atlas_physical_size(
                                atlas_content_size,
                                self.context.device.limits().max_texture_dimension_2d,
                            );
                            let mut uniforms =
                                analytic_uniforms(self.width, self.height, tessellation_height);
                            uniforms.atlas_texture_inverse_size = [
                                1.0 / atlas_physical_size[0] as f32,
                                1.0 / atlas_physical_size[1] as f32,
                            ];
                            uniforms.atlas_content_inverse_viewport = [
                                2.0 / atlas_content_size[0] as f32,
                                -2.0 / atlas_content_size[1] as f32,
                            ];
                            let tessellation_texture = self.context.tessellator.encode(
                                &self.context.device,
                                &mut tessellation_uploads.borrow_mut(),
                                encoder,
                                &self.context.feather_lut.view,
                                &tessellation.spans,
                                &uniforms,
                                &paths,
                                &tessellation.contours,
                                tessellation_height,
                            );
                            let tessellation_view = tessellation_texture
                                .create_view(&wgpu::TextureViewDescriptor::default());
                            let atlas_texture =
                                self.context
                                    .device
                                    .create_texture(&wgpu::TextureDescriptor {
                                        label: Some("nuxie-msaa-feather-atlas"),
                                        size: wgpu::Extent3d {
                                            width: atlas_physical_size[0],
                                            height: atlas_physical_size[1],
                                            depth_or_array_layers: 1,
                                        },
                                        mip_level_count: 1,
                                        sample_count: 1,
                                        dimension: wgpu::TextureDimension::D2,
                                        format: wgpu::TextureFormat::R16Float,
                                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                                            | wgpu::TextureUsages::TEXTURE_BINDING,
                                        view_formats: &[],
                                    });
                            let atlas_view = atlas_texture.create_view(&Default::default());
                            self.context.atlas_pipeline.encode_mask(
                                &self.context.device,
                                encoder,
                                &atlas_view,
                                &self.context.patch_vertex_buffer,
                                &self.context.patch_index_buffer,
                                &tessellation_view,
                                &self.context.feather_lut.view,
                                &uniforms,
                                &paths,
                                &paints,
                                &paint_aux,
                                &tessellation.contours,
                                tessellation.base_instance,
                                tessellation.instance_count,
                                draw.paint.style == RenderPaintStyle::Stroke,
                                true,
                                atlas_content_size,
                                [0, 0, placement.width, placement.height],
                            );
                            let [left, top, right, bottom] = placement.bounds;
                            let vertices = [
                                gpu::TriangleVertex::new([left, bottom], 1, 1),
                                gpu::TriangleVertex::new([left, top], 1, 1),
                                gpu::TriangleVertex::new([right, bottom], 1, 1),
                                gpu::TriangleVertex::new([right, bottom], 1, 1),
                                gpu::TriangleVertex::new([left, top], 1, 1),
                                gpu::TriangleVertex::new([right, top], 1, 1),
                            ];
                            pending_draws.push(PendingDraw::Atlas(
                                self.context.msaa_atlas_pipeline.prepare(
                                    &self.context.device,
                                    &tessellation_view,
                                    &self.context.feather_lut.view,
                                    gradient_texture.as_ref().map(|texture| &texture.view),
                                    &atlas_view,
                                    &uniforms,
                                    &paths,
                                    &paints,
                                    &paint_aux,
                                    &tessellation.contours,
                                    &vertices,
                                    destination_view.as_ref(),
                                    draw.state.clip_rect.is_some(),
                                    matches!(
                                        draw.role,
                                        DrawRole::Content { clip_id } if clip_id != 0
                                    ),
                                    draw.paint.blend_mode != BlendMode::SrcOver,
                                    gpu::blend_mode_id(draw.paint.blend_mode) >= 12,
                                    [left as u32, top as u32, right as u32, bottom as u32],
                                ),
                            ));
                            prepared_schedules.push(schedule);
                            continue;
                        }
                    }
                    if draw.paint.feather == 0.0 {
                        let has_clip_rect = draw.state.clip_rect.is_some();
                        let advanced_blend = draw.paint.blend_mode != BlendMode::SrcOver;
                        let options = DirectPathOptions {
                            path_clip: matches!(
                                draw.role,
                                DrawRole::Content { clip_id } if clip_id != 0
                            ),
                            clip_rect: has_clip_rect,
                            opaque: msaa_draw_has_opaque_paint(draw),
                            advanced_blend,
                            hsl_blend: blend_mode_uses_hsl(draw.paint.blend_mode),
                            batchable_midpoint_fill: draw.paint.style == RenderPaintStyle::Fill
                                && draw.path.fill_rule == FillRule::NonZero
                                && matches!(draw.role, DrawRole::Content { .. })
                                && !has_clip_rect
                                && !advanced_blend
                                && msaa_draw_has_opaque_paint(draw)
                                && draw.image.is_none()
                                && gradient_batch.draws[draw_index].is_none(),
                            batchable_direct_stroke: direct_stroke_can_batch(
                                draw,
                                gradient_batch.draws[draw_index].is_some(),
                            ),
                            destination_copy_bounds: msaa_destination_copy_bounds(
                                draw,
                                self.width,
                                self.height,
                            ),
                        };
                        if has_clip_rect && !self.context.path_pipeline.supports_clip_rect() {
                            return Err(RendererError::Unsupported(
                                "clip rectangles on msaa direct path draws",
                            ));
                        }
                        let oriented_path = (draw.paint.style == RenderPaintStyle::Fill
                            && draw::msaa_fill_requires_reverse(
                                &draw.path.raw_path,
                                draw.state.transform,
                                draw.path.fill_rule,
                            ))
                        .then(|| {
                            let mut path = RawPath::new();
                            path.add_path_backwards(&draw.path.raw_path, Mat2D::IDENTITY);
                            path
                        });
                        let raw_path = oriented_path.as_ref().unwrap_or(&draw.path.raw_path);
                        let tessellation = match draw.paint.style {
                            RenderPaintStyle::Fill => {
                                draw::build_fill_tessellation(raw_path, draw.state.transform)
                            }
                            RenderPaintStyle::Stroke => draw::build_stroke_tessellation(
                                &draw.path.raw_path,
                                draw.state.transform,
                                draw.paint.thickness,
                                draw.paint.join,
                                draw.paint.cap,
                            ),
                        };
                        if let Some(mut tessellation) = tessellation {
                            tessellation.path.z_index = z_index;
                            let image = draw.image.as_ref().map(|image| match image {
                                ImageDraw::Rect(image) => image,
                                ImageDraw::Mesh(_) => {
                                    unreachable!(
                                        "MSAA image meshes are prepared before direct paths"
                                    )
                                }
                            });
                            let gradient = gradient_batch.draws[draw_index];
                            let mut paint = if let Some(gradient) = gradient {
                                if draw.paint.style == RenderPaintStyle::Stroke {
                                    gpu::PaintData::gradient_stroke(
                                        gradient.paint_type,
                                        gradient.texture_y,
                                        draw.paint.blend_mode,
                                    )
                                } else {
                                    gpu::PaintData::gradient(
                                        gradient.paint_type,
                                        gradient.texture_y,
                                        draw.path.fill_rule,
                                        draw.paint.blend_mode,
                                    )
                                }
                            } else if let Some(image) = image {
                                gpu::PaintData::image(
                                    image.opacity,
                                    draw.path.fill_rule,
                                    image.blend_mode,
                                )
                            } else if draw.paint.style == RenderPaintStyle::Stroke {
                                gpu::PaintData::solid_stroke(
                                    modulate_color_alpha(draw.paint.color, draw.state.opacity),
                                    draw.paint.blend_mode,
                                )
                            } else {
                                gpu::PaintData::solid(
                                    modulate_color_alpha(draw.paint.color, draw.state.opacity),
                                    draw.path.fill_rule,
                                    draw.paint.blend_mode,
                                )
                            };
                            if draw.state.clip_rect.is_some() {
                                paint = paint.with_clip_rect();
                            }
                            let paint_aux = if let Some(gradient) = gradient {
                                gradient_paint_aux(draw.state.clip_rect, gradient)
                            } else if let Some(image) = image {
                                image_paint_aux(
                                    draw.state.clip_rect,
                                    draw.state.transform,
                                    image.texture.as_ref(),
                                )
                            } else {
                                clip_rect_paint_aux(draw.state.clip_rect)
                            };
                            let path_index = pending_paths.len();
                            let compact_midpoint_layout = (draw.paint.style
                                == RenderPaintStyle::Fill
                                && draw.path.fill_rule == FillRule::NonZero
                                && matches!(draw.role, DrawRole::Content { .. })
                                && !has_clip_rect
                                && !advanced_blend
                                && image.is_none()
                                && gradient.is_none())
                                || (draw.paint.style == RenderPaintStyle::Stroke
                                    && matches!(draw.role, DrawRole::Content { .. })
                                    && !has_clip_rect
                                    && !advanced_blend
                                    && image.is_none()
                                    && gradient.is_none());
                            let compact_midpoint_group = options
                                .batchable_direct_stroke
                                .then_some((schedule.draw_group, schedule.is_prepass));
                            pending_paths.push(PendingPathDraw {
                                tessellation,
                                paint,
                                paint_aux,
                                image: image
                                    .map(|image| (image.texture.view.clone(), image.sampler)),
                                compact_midpoint_layout,
                                compact_midpoint_group,
                                logical_flush: schedule.logical_flush,
                                prepared: None,
                            });
                            pending_draws.push(if draw.paint.style == RenderPaintStyle::Fill {
                                PendingDraw::Fill(path_index, draw.path.fill_rule, options)
                            } else {
                                PendingDraw::Stroke(path_index, options)
                            });
                            prepared_schedules.push(schedule);
                            continue;
                        }
                    }
                    if let Some(path_vertices) = tessellate_solid(draw, self.width, self.height) {
                        let cover_vertices = cover_vertices(&path_vertices);
                        let path_buffer = self.context.device.create_counted_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("nuxie-path-vertices"),
                                contents: bytemuck::cast_slice(&path_vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            },
                        );
                        let cover_buffer = self.context.device.create_counted_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("nuxie-path-cover"),
                                contents: bytemuck::cast_slice(&cover_vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            },
                        );
                        pending_draws.push(PendingDraw::Bootstrap(
                            path_buffer,
                            cover_buffer,
                            draw.path.fill_rule,
                        ));
                        prepared_schedules.push(schedule);
                    }
                }
                debug_assert_eq!(pending_draws.len(), prepared_schedules.len());
                let logical_flush_count = logical_flush_starts.len().max(1);
                for logical_flush in 0..logical_flush_count {
                    let path_start =
                        pending_paths.partition_point(|path| path.logical_flush < logical_flush);
                    let path_end =
                        pending_paths.partition_point(|path| path.logical_flush <= logical_flush);
                    if path_start == path_end {
                        continue;
                    }
                    let paths_in_flush = &mut pending_paths[path_start..path_end];
                    let mut spans = Vec::new();
                    let mut paths = vec![gpu::PathData::zeroed()];
                    let mut paints = vec![gpu::PaintData::zeroed()];
                    let mut paint_aux = vec![gpu::PaintAuxData::zeroed()];
                    let mut contours = Vec::new();
                    let mut tessellation_height = 1;
                    let max_height = self.context.device.limits().max_texture_dimension_2d;
                    let midpoint_span = gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
                    let compact_geometry_end = (paths_in_flush.len() > 1
                        && paths_in_flush.iter().all(|path| {
                            path.compact_midpoint_layout
                                && path.tessellation.base_instance == 1
                                && midpoint_tessellation_single_row_width(&path.tessellation.spans)
                                    .is_some()
                        }))
                    .then(|| {
                        paths_in_flush.iter().try_fold(midpoint_span, |end, path| {
                            path.tessellation
                                .instance_count
                                .checked_mul(midpoint_span)
                                .and_then(|count| end.checked_add(count))
                        })
                    })
                    .flatten()
                    .filter(|&end| {
                        max_height
                            .checked_mul(gpu::TESS_TEXTURE_WIDTH as u32)
                            .is_some_and(|capacity| {
                                align_to(end, gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32)
                                    .checked_add(1)
                                    .is_some_and(|end| end <= capacity)
                            })
                    });
                    let compact_group_keys = paths_in_flush
                        .iter()
                        .map(|path| path.compact_midpoint_group)
                        .collect::<Option<Vec<_>>>();
                    let compact_group_ranges = paths_in_flush
                        .iter()
                        .map(|path| {
                            (
                                path.tessellation.base_instance,
                                path.tessellation.instance_count,
                            )
                        })
                        .collect::<Vec<_>>();
                    let compact_groups = compact_group_keys.as_ref().and_then(|keys| {
                        paths_in_flush
                            .iter()
                            .all(|path| {
                                midpoint_tessellation_single_row_width(&path.tessellation.spans)
                                    .is_some()
                            })
                            .then(|| {
                                compact_midpoint_groups(keys, &compact_group_ranges, max_height)
                            })
                            .flatten()
                    });
                    if let Some(geometry_end) = compact_geometry_end {
                        append_tessellation_padding_span(&mut spans, 0, midpoint_span);
                        let mut next_base_instance = 1;
                        for (path_offset, path) in paths_in_flush.iter_mut().enumerate() {
                            let path_id = u32::try_from(path_offset + 1)
                                .expect("MSAA path ID must fit the shader contract");
                            next_base_instance = append_compact_midpoint_tessellation_to_flush(
                                &mut path.tessellation,
                                path_id,
                                next_base_instance,
                                0,
                                &mut spans,
                                &mut contours,
                            );
                        }
                        debug_assert_eq!(next_base_instance * midpoint_span, geometry_end);
                        let outer_curve_start =
                            align_to(geometry_end, gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32);
                        append_tessellation_padding_span(
                            &mut spans,
                            geometry_end,
                            outer_curve_start,
                        );
                        append_tessellation_padding_span(
                            &mut spans,
                            outer_curve_start,
                            outer_curve_start + 1,
                        );
                        tessellation_height =
                            outer_curve_start.div_euclid(gpu::TESS_TEXTURE_WIDTH as u32) + 1;
                    } else if let Some(compact_groups) = compact_groups {
                        tessellation_height = u32::try_from(compact_groups.len())
                            .expect("compact MSAA stroke group count fits u32");
                        for (row, group) in compact_groups.into_iter().enumerate() {
                            let row =
                                u32::try_from(row).expect("compact MSAA stroke group row fits u32");
                            append_tessellation_padding_span_at_y(
                                &mut spans,
                                0,
                                midpoint_span,
                                row,
                            );
                            let mut next_base_instance = 1;
                            for path_offset in group.range {
                                let path = &mut paths_in_flush[path_offset];
                                let path_id = u32::try_from(path_offset + 1)
                                    .expect("MSAA path ID must fit the shader contract");
                                next_base_instance = append_compact_midpoint_tessellation_to_flush(
                                    &mut path.tessellation,
                                    path_id,
                                    next_base_instance,
                                    row,
                                    &mut spans,
                                    &mut contours,
                                );
                            }
                            debug_assert_eq!(
                                next_base_instance * midpoint_span,
                                group.geometry_end
                            );
                            let outer_curve_start = align_to(
                                group.geometry_end,
                                gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32,
                            );
                            append_tessellation_padding_span_at_y(
                                &mut spans,
                                group.geometry_end,
                                outer_curve_start,
                                row,
                            );
                            append_tessellation_padding_span_at_y(
                                &mut spans,
                                outer_curve_start,
                                outer_curve_start + 1,
                                row,
                            );
                        }
                    } else {
                        let mut cursor_x = 0;
                        let mut cursor_y = 0;
                        for (path_offset, path) in paths_in_flush.iter_mut().enumerate() {
                            let local_height =
                                draw::tessellation_texture_height(&path.tessellation.spans);
                            let single_row_width = (local_height == 1)
                                .then(|| {
                                    midpoint_tessellation_single_row_width(&path.tessellation.spans)
                                })
                                .flatten();
                            let placement = midpoint_shelf_placement(
                                cursor_x,
                                cursor_y,
                                local_height,
                                single_row_width,
                            );
                            if placement.height > max_height {
                                return Err(RendererError::Device(
                                    "flush-wide tessellation texture exceeds device dimension limit"
                                        .into(),
                                ));
                            }
                            cursor_x = placement.next_x;
                            cursor_y = placement.next_y;
                            tessellation_height = tessellation_height.max(placement.height);
                            let path_id = u32::try_from(path_offset + 1)
                                .expect("MSAA path ID must fit the shader contract");
                            append_midpoint_tessellation_to_flush(
                                &mut path.tessellation,
                                path_id,
                                placement.x,
                                placement.y,
                                &mut spans,
                                &mut contours,
                            );
                        }
                    }
                    for path in paths_in_flush.iter() {
                        paths.push(path.tessellation.path);
                        paints.push(path.paint);
                        paint_aux.push(path.paint_aux);
                    }
                    let mut uniforms =
                        analytic_uniforms(self.width, self.height, tessellation_height);
                    if gradient_batch.height != 0 {
                        uniforms.inverse_viewports[0] = -2.0 / gradient_batch.height as f32;
                    }
                    uniforms.max_path_id =
                        u32::try_from(paths.len() - 1).expect("MSAA path ID overflow");
                    let tessellation = self.context.tessellator.encode_with_new_flush_resources(
                        &self.context.device,
                        &mut tessellation_uploads.borrow_mut(),
                        encoder,
                        &self.context.feather_lut.view,
                        &spans,
                        &uniforms,
                        &paths,
                        &contours,
                        tessellation_height,
                    );
                    let tessellation_view = tessellation
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let resources = self.context.path_pipeline.prepare_resources(
                        &self.context.device,
                        &mut tessellation_uploads.borrow_mut(),
                        &tessellation_view,
                        &self.context.feather_lut.view,
                        gradient_texture.as_ref().map(|texture| &texture.view),
                        destination_view.as_ref(),
                        &tessellation.flush_resources,
                        &paints,
                        &paint_aux,
                    );
                    for path in paths_in_flush {
                        path.prepared = Some(self.context.path_pipeline.prepare_draw(
                            &self.context.device,
                            &resources,
                            path.image.as_ref().map(|(view, sampler)| (view, *sampler)),
                            path.tessellation.base_instance,
                            path.tessellation.instance_count,
                        ));
                    }
                }
                let mut take_path = |index: usize| {
                    pending_paths[index]
                        .prepared
                        .take()
                        .expect("MSAA path escaped flush-wide preparation")
                };
                let prepared_draws = pending_draws
                    .into_iter()
                    .map(|draw| match draw {
                        PendingDraw::Stroke(index, options) => {
                            PreparedDraw::Stroke(take_path(index), options)
                        }
                        PendingDraw::Fill(index, fill_rule, options) => {
                            PreparedDraw::Fill(take_path(index), fill_rule, options)
                        }
                        PendingDraw::ImageMesh(draw, options) => {
                            PreparedDraw::ImageMesh(draw, options)
                        }
                        PendingDraw::OutermostClipUpdate(index, fill_rule) => {
                            PreparedDraw::OutermostClipUpdate(take_path(index), fill_rule)
                        }
                        PendingDraw::NestedClipUpdate(index, fill_rule) => {
                            PreparedDraw::NestedClipUpdate(take_path(index), fill_rule)
                        }
                        PendingDraw::ClipReset(draw, action) => {
                            PreparedDraw::ClipReset(draw, action)
                        }
                        PendingDraw::Atlas(draw) => PreparedDraw::Atlas(draw),
                        PendingDraw::Bootstrap(path, cover, fill_rule) => {
                            PreparedDraw::Bootstrap(path, cover, fill_rule)
                        }
                    })
                    .collect::<Vec<_>>();
                debug_assert_eq!(prepared_draws.len(), prepared_schedules.len());
                let prepared_advanced_count = prepared_draws
                    .iter()
                    .filter(|draw| match draw {
                        PreparedDraw::Stroke(_, options) | PreparedDraw::Fill(_, _, options) => {
                            options.advanced_blend
                        }
                        PreparedDraw::ImageMesh(_, options) => options.advanced_blend,
                        PreparedDraw::Atlas(draw) => {
                            msaa_atlas_pipeline::MsaaAtlasPipeline::uses_advanced_blend(draw)
                        }
                        _ => false,
                    })
                    .count();
                let requested_advanced_count = draws
                    .iter()
                    .filter(|draw| draw_uses_advanced_blend(draw))
                    .count();
                if prepared_advanced_count != requested_advanced_count {
                    return Err(RendererError::Unsupported(
                        "advanced blending on unprepared msaa draws",
                    ));
                }
                let destination_copy_bounds = |draw: &PreparedDraw| match draw {
                    PreparedDraw::Stroke(_, options) | PreparedDraw::Fill(_, _, options)
                        if options.advanced_blend =>
                    {
                        Some(options.destination_copy_bounds)
                    }
                    PreparedDraw::ImageMesh(_, options) if options.advanced_blend => {
                        Some(options.destination_copy_bounds)
                    }
                    PreparedDraw::Atlas(draw) => {
                        msaa_atlas_pipeline::MsaaAtlasPipeline::destination_copy_bounds(draw)
                    }
                    _ => None,
                };
                let fallback_texture =
                    (!has_advanced_msaa && !resolve_fixed_msaa_directly).then(|| {
                        self.context
                            .device
                            .create_texture(&wgpu::TextureDescriptor {
                                label: Some("nuxie-fallback-resolve-target"),
                                size: texture.size(),
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: wgpu::TextureDimension::D2,
                                format: wgpu::TextureFormat::Rgba8Unorm,
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                                    | wgpu::TextureUsages::TEXTURE_BINDING,
                                view_formats: &[],
                            })
                    });
                let fallback_view = fallback_texture
                    .as_ref()
                    .map(|texture| texture.create_view(&Default::default()));
                #[derive(Clone)]
                struct SegmentEnd {
                    index: usize,
                    starts_logical_flush: bool,
                    destination_copy_bounds: Vec<[u32; 4]>,
                }
                let mut segment_ends = Vec::new();
                let prepared_draw_groups = prepared_schedules
                    .iter()
                    .map(|schedule| schedule.draw_group)
                    .collect::<Vec<_>>();
                let prepared_prepasses = prepared_schedules
                    .iter()
                    .map(|schedule| schedule.is_prepass)
                    .collect::<Vec<_>>();
                let prepared_logical_flushes = prepared_schedules
                    .iter()
                    .map(|schedule| schedule.logical_flush)
                    .collect::<Vec<_>>();
                // C++ attaches a destination-read barrier to the first
                // subpass-0 batch in its draw group. Negative-key opaque
                // prepasses stay before that barrier, so they can never be
                // interrupted by a resolve/copy.
                for (index, draw) in prepared_draws.iter().enumerate() {
                    let Some(bounds) = destination_copy_bounds(draw) else {
                        continue;
                    };
                    let group_head = msaa_destination_copy_head(
                        &prepared_draw_groups,
                        &prepared_prepasses,
                        &prepared_logical_flushes,
                        index,
                    );
                    segment_ends.push(SegmentEnd {
                        index: group_head,
                        starts_logical_flush: false,
                        destination_copy_bounds: vec![bounds],
                    });
                }
                segment_ends.extend(logical_flush_starts.iter().copied().skip(1).map(|index| {
                    SegmentEnd {
                        index,
                        starts_logical_flush: true,
                        destination_copy_bounds: Vec::new(),
                    }
                }));
                segment_ends.sort_unstable_by_key(|segment| segment.index);
                let mut merged_segment_ends =
                    Vec::<SegmentEnd>::with_capacity(segment_ends.len() + 1);
                for segment in segment_ends {
                    let merged = if let Some(previous) = merged_segment_ends.last_mut() {
                        if previous.index == segment.index {
                            previous.starts_logical_flush |= segment.starts_logical_flush;
                            previous
                                .destination_copy_bounds
                                .extend(segment.destination_copy_bounds.iter().copied());
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !merged {
                        merged_segment_ends.push(segment);
                    }
                }
                merged_segment_ends.push(SegmentEnd {
                    index: prepared_draws.len(),
                    starts_logical_flush: false,
                    destination_copy_bounds: Vec::new(),
                });
                let mut segment_start = 0;
                let mut starts_logical_flush = true;
                for (segment_index, segment_end) in merged_segment_ends.into_iter().enumerate() {
                    let first_segment = segment_index == 0;
                    let reset_depth = first_segment || starts_logical_flush;
                    let reset_stencil = first_segment || starts_logical_flush;
                    let resolve_target = if has_advanced_msaa || resolve_fixed_msaa_directly {
                        &view
                    } else {
                        fallback_view
                            .as_ref()
                            .expect("fixed MSAA draw prepared without fallback target")
                    };
                    if !first_segment && starts_logical_flush {
                        self.context.composite_pipeline.encode_msaa_preserve(
                            &self.context.device,
                            encoder,
                            &multisample_view,
                            resolve_target,
                        );
                    }
                    let mut pass = encoder.begin_counted_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("nuxie-solid-pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &multisample_view,
                            depth_slice: None,
                            resolve_target: Some(resolve_target),
                            ops: wgpu::Operations {
                                load: if first_segment {
                                    wgpu::LoadOp::Clear(
                                        if has_advanced_msaa || resolve_fixed_msaa_directly {
                                            color(self.clear_color)
                                        } else {
                                            wgpu::Color::TRANSPARENT
                                        },
                                    )
                                } else {
                                    wgpu::LoadOp::Load
                                },
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &stencil_view,
                            depth_ops: Some(wgpu::Operations {
                                load: if reset_depth {
                                    wgpu::LoadOp::Clear(1.0)
                                } else {
                                    wgpu::LoadOp::Load
                                },
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: Some(wgpu::Operations {
                                load: if reset_stencil {
                                    wgpu::LoadOp::Clear(0)
                                } else {
                                    wgpu::LoadOp::Load
                                },
                                store: wgpu::StoreOp::Store,
                            }),
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    pass.set_stencil_reference(0);
                    let segment_draws = &prepared_draws[segment_start..segment_end.index];
                    let segment_schedules = &prepared_schedules[segment_start..segment_end.index];
                    let batched_fill = match (segment_draws.first(), segment_schedules.first()) {
                        (
                            Some(PreparedDraw::Fill(first, fill_rule, options)),
                            Some(first_schedule),
                        ) if segment_draws.len() > 1 && options.batchable_midpoint_fill => {
                            let mut instance_end =
                                first.base_instance.checked_add(first.instance_count);
                            let mut compatible = instance_end.is_some();
                            for (candidate, schedule) in
                                segment_draws.iter().zip(segment_schedules).skip(1)
                            {
                                let PreparedDraw::Fill(
                                    candidate,
                                    candidate_fill_rule,
                                    candidate_options,
                                ) = candidate
                                else {
                                    compatible = false;
                                    break;
                                };
                                let same_pipeline = candidate_fill_rule == fill_rule
                                    && candidate_options.path_clip == options.path_clip
                                    && candidate_options.clip_rect == options.clip_rect
                                    && candidate_options.opaque == options.opaque
                                    && candidate_options.advanced_blend == options.advanced_blend
                                    && candidate_options.hsl_blend == options.hsl_blend
                                    && candidate_options.batchable_midpoint_fill
                                        == options.batchable_midpoint_fill;
                                let same_schedule = schedule.draw_group
                                    == first_schedule.draw_group
                                    && schedule.is_prepass == first_schedule.is_prepass
                                    && schedule.logical_flush == first_schedule.logical_flush;
                                if !same_pipeline
                                    || !same_schedule
                                    || instance_end != Some(candidate.base_instance)
                                {
                                    compatible = false;
                                    break;
                                }
                                instance_end = candidate
                                    .base_instance
                                    .checked_add(candidate.instance_count);
                                if instance_end.is_none() {
                                    compatible = false;
                                    break;
                                }
                            }
                            compatible.then(|| instance_end.unwrap())
                        }
                        _ => None,
                    };
                    // C++ LogicalFlush::pushDraw condenses contiguous msaaStrokes
                    // until a draw-group, pipeline, or element-range break.
                    let mut batched_stroke_ends = vec![None; segment_draws.len()];
                    let mut batched_stroke_continuations = vec![false; segment_draws.len()];
                    let mut batch_start = 0;
                    while batch_start < segment_draws.len() {
                        let (first, options) = match &segment_draws[batch_start] {
                            PreparedDraw::Stroke(draw, options)
                                if options.batchable_direct_stroke =>
                            {
                                (draw, options)
                            }
                            _ => {
                                batch_start += 1;
                                continue;
                            }
                        };
                        let first_schedule = prepared_schedules[segment_start + batch_start];
                        let Some(mut instance_end) =
                            first.base_instance.checked_add(first.instance_count)
                        else {
                            batch_start += 1;
                            continue;
                        };
                        let mut batch_end = batch_start + 1;
                        while batch_end < segment_draws.len() {
                            let PreparedDraw::Stroke(candidate, candidate_options) =
                                &segment_draws[batch_end]
                            else {
                                break;
                            };
                            let candidate_schedule = prepared_schedules[segment_start + batch_end];
                            let same_pipeline = candidate_options.batchable_direct_stroke
                                && candidate_options.path_clip == options.path_clip
                                && candidate_options.clip_rect == options.clip_rect
                                && candidate_options.opaque == options.opaque
                                && candidate_options.advanced_blend == options.advanced_blend
                                && candidate_options.hsl_blend == options.hsl_blend;
                            let same_schedule = candidate_schedule.draw_group
                                == first_schedule.draw_group
                                && candidate_schedule.is_prepass == first_schedule.is_prepass
                                && candidate_schedule.logical_flush == first_schedule.logical_flush;
                            if !same_pipeline
                                || !same_schedule
                                || instance_end != candidate.base_instance
                            {
                                break;
                            }
                            let Some(next_end) = candidate
                                .base_instance
                                .checked_add(candidate.instance_count)
                            else {
                                break;
                            };
                            instance_end = next_end;
                            batch_end += 1;
                        }
                        if batch_end > batch_start + 1 {
                            batched_stroke_ends[batch_start] = Some(instance_end);
                            batched_stroke_continuations[batch_start + 1..batch_end].fill(true);
                        }
                        batch_start = batch_end;
                    }
                    let mut direct_path_bindings_active = false;
                    for (prepared_offset, prepared) in segment_draws.iter().enumerate() {
                        match prepared {
                            PreparedDraw::Stroke(draw, options) => {
                                if batched_stroke_continuations[prepared_offset] {
                                    continue;
                                }
                                let instance_end = batched_stroke_ends[prepared_offset]
                                    .unwrap_or_else(|| {
                                        draw.base_instance
                                            .checked_add(draw.instance_count)
                                            .expect("MSAA stroke instance range overflow")
                                    });
                                pass.set_stencil_reference(if options.path_clip {
                                    0x80
                                } else {
                                    0
                                });
                                pass.set_pipeline(self.context.path_pipeline.direct_pipeline(
                                    path_pipeline::DirectPathPipelineKind::Stroke,
                                    options.path_clip,
                                    options.clip_rect,
                                    options.opaque,
                                    options.advanced_blend,
                                    options.hsl_blend,
                                ));
                                draw.bind_resources(&mut pass, !direct_path_bindings_active);
                                direct_path_bindings_active = true;
                                pass.set_vertex_buffer(
                                    0,
                                    self.context.patch_vertex_buffer.slice(..),
                                );
                                pass.set_index_buffer(
                                    self.context.patch_index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint16,
                                );
                                pass.draw_path_patches(
                                    0..gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT as u32,
                                    0,
                                    draw.base_instance..instance_end,
                                );
                            }
                            PreparedDraw::Fill(draw, fill_rule, options) => {
                                if batched_fill.is_some() && prepared_offset != 0 {
                                    continue;
                                }
                                let instance_end = batched_fill.unwrap_or_else(|| {
                                    draw.base_instance
                                        .checked_add(draw.instance_count)
                                        .expect("MSAA fill instance range overflow")
                                });
                                pass.set_stencil_reference(0x80);
                                draw.bind_resources(&mut pass, !direct_path_bindings_active);
                                direct_path_bindings_active = true;
                                pass.set_vertex_buffer(
                                    0,
                                    self.context.patch_vertex_buffer.slice(..),
                                );
                                pass.set_index_buffer(
                                    self.context.patch_index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint16,
                                );
                                let indices = gpu::MIDPOINT_FAN_PATCH_BORDER_INDEX_COUNT as u32
                                    ..gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT as u32;
                                for fill_pass in msaa_fill_passes(*fill_rule) {
                                    let pipeline = msaa_fill_pipeline_kind(*fill_pass);
                                    pass.set_pipeline(self.context.path_pipeline.direct_pipeline(
                                        pipeline,
                                        options.path_clip,
                                        options.clip_rect,
                                        options.opaque,
                                        options.advanced_blend,
                                        options.hsl_blend,
                                    ));
                                    pass.draw_path_patches(
                                        indices.clone(),
                                        0,
                                        draw.base_instance..instance_end,
                                    );
                                }
                            }
                            PreparedDraw::ImageMesh(draw, options) => {
                                direct_path_bindings_active = false;
                                pass.set_stencil_reference(if options.path_clip {
                                    0x80
                                } else {
                                    0
                                });
                                pass.set_pipeline(self.context.msaa_image_mesh_pipeline.pipeline(
                                    options.path_clip,
                                    options.clip_rect,
                                    options.advanced_blend,
                                    options.hsl_blend,
                                ));
                                pass.set_bind_group(0, &draw.flush_group, &[]);
                                pass.set_bind_group(1, &draw.image_group, &[]);
                                pass.set_vertex_buffer(0, draw.vertices.slice(..));
                                pass.set_vertex_buffer(1, draw.uvs.slice(..));
                                pass.set_index_buffer(
                                    draw.indices.slice(..),
                                    wgpu::IndexFormat::Uint16,
                                );
                                pass.draw_indexed(0..draw.index_count, 0, 0..1);
                            }
                            PreparedDraw::OutermostClipUpdate(draw, fill_rule) => {
                                pass.set_stencil_reference(0x80);
                                draw.bind_resources(&mut pass, !direct_path_bindings_active);
                                direct_path_bindings_active = true;
                                pass.set_vertex_buffer(
                                    0,
                                    self.context.patch_vertex_buffer.slice(..),
                                );
                                pass.set_index_buffer(
                                    self.context.patch_index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint16,
                                );
                                let indices = gpu::MIDPOINT_FAN_PATCH_BORDER_INDEX_COUNT as u32
                                    ..gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT as u32;
                                match fill_rule {
                                    FillRule::EvenOdd => {
                                        for pipeline in [
                                            &self
                                                .context
                                                .path_pipeline
                                                .even_odd_clip_stencil_pipeline,
                                            &self
                                                .context
                                                .path_pipeline
                                                .even_odd_clip_cover_pipeline,
                                        ] {
                                            pass.set_pipeline(pipeline);
                                            pass.draw_path_patches(
                                                indices.clone(),
                                                0,
                                                draw.base_instance
                                                    ..draw.base_instance + draw.instance_count,
                                            );
                                        }
                                    }
                                    FillRule::NonZero | FillRule::Clockwise => {
                                        let cleanup = if *fill_rule == FillRule::Clockwise {
                                            &self
                                                .context
                                                .path_pipeline
                                                .clockwise_clip_cleanup_pipeline
                                        } else {
                                            &self.context.path_pipeline.clip_cleanup_pipeline
                                        };
                                        for pipeline in [
                                            &self.context.path_pipeline.clip_borrowed_pipeline,
                                            &self.context.path_pipeline.clip_update_pipeline,
                                            cleanup,
                                        ] {
                                            pass.set_pipeline(pipeline);
                                            pass.draw_path_patches(
                                                indices.clone(),
                                                0,
                                                draw.base_instance
                                                    ..draw.base_instance + draw.instance_count,
                                            );
                                        }
                                    }
                                }
                            }
                            PreparedDraw::NestedClipUpdate(draw, fill_rule) => {
                                pass.set_stencil_reference(0x80);
                                pass.set_pipeline(if *fill_rule == FillRule::EvenOdd {
                                    &self.context.path_pipeline.nested_even_odd_clip_pipeline
                                } else {
                                    &self.context.path_pipeline.nested_clip_pipeline
                                });
                                draw.bind_resources(&mut pass, !direct_path_bindings_active);
                                direct_path_bindings_active = true;
                                pass.set_vertex_buffer(
                                    0,
                                    self.context.patch_vertex_buffer.slice(..),
                                );
                                pass.set_index_buffer(
                                    self.context.patch_index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint16,
                                );
                                pass.draw_path_patches(
                                    gpu::MIDPOINT_FAN_PATCH_BORDER_INDEX_COUNT as u32
                                        ..gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT as u32,
                                    0,
                                    draw.base_instance..draw.base_instance + draw.instance_count,
                                );
                            }
                            PreparedDraw::ClipReset(draw, action) => {
                                direct_path_bindings_active = false;
                                let (reference, pipeline) = match action {
                                    MsaaClipResetAction::ClearPrevious => {
                                        (0, &self.context.msaa_stencil_pipeline.clip_reset_pipeline)
                                    }
                                    MsaaClipResetAction::IntersectPreviousNonZero => (
                                        0x80,
                                        &self
                                            .context
                                            .msaa_stencil_pipeline
                                            .nested_clip_reset_pipeline,
                                    ),
                                    MsaaClipResetAction::IntersectPreviousEvenOdd => (
                                        0x80,
                                        &self
                                            .context
                                            .msaa_stencil_pipeline
                                            .nested_clip_reset_pipeline,
                                    ),
                                    MsaaClipResetAction::IntersectPreviousClockwise => (
                                        0x80,
                                        &self
                                            .context
                                            .msaa_stencil_pipeline
                                            .nested_clockwise_clip_reset_pipeline,
                                    ),
                                };
                                pass.set_stencil_reference(reference);
                                pass.set_pipeline(pipeline);
                                pass.set_bind_group(0, &draw.flush_group, &[]);
                                pass.set_vertex_buffer(0, draw.vertices.slice(..));
                                pass.draw(0..draw.vertex_count, 0..1);
                            }
                            PreparedDraw::Atlas(draw) => {
                                direct_path_bindings_active = false;
                                pass.set_stencil_reference(
                                    if msaa_atlas_pipeline::MsaaAtlasPipeline::uses_path_clip(draw)
                                    {
                                        0x80
                                    } else {
                                        0
                                    },
                                );
                                pass.set_pipeline(self.context.msaa_atlas_pipeline.pipeline(draw));
                                pass.set_bind_group(0, &draw.flush_group, &[]);
                                pass.set_bind_group(1, &draw.image_group, &[]);
                                pass.set_bind_group(3, &draw.sampler_group, &[]);
                                pass.set_vertex_buffer(0, draw.vertices.slice(..));
                                pass.draw(0..draw.vertex_count, 0..1);
                            }
                            PreparedDraw::Bootstrap(path_buffer, cover_buffer, fill_rule) => {
                                direct_path_bindings_active = false;
                                pass.set_stencil_reference(0);
                                pass.set_pipeline(match fill_rule {
                                    FillRule::EvenOdd => &self.context.even_odd_stencil_pipeline,
                                    FillRule::NonZero | FillRule::Clockwise => {
                                        &self.context.non_zero_stencil_pipeline
                                    }
                                });
                                pass.set_vertex_buffer(0, path_buffer.slice(..));
                                pass.draw(
                                    0..(path_buffer.size() / std::mem::size_of::<Vertex>() as u64)
                                        as u32,
                                    0..1,
                                );
                                pass.set_pipeline(&self.context.cover_pipeline);
                                pass.set_vertex_buffer(0, cover_buffer.slice(..));
                                pass.draw(0..6, 0..1);
                            }
                        }
                    }
                    drop(pass);
                    if !segment_end.destination_copy_bounds.is_empty() {
                        let destination_texture = destination_texture
                            .as_ref()
                            .expect("advanced MSAA draw prepared without destination texture");
                        for [left, top, right, bottom] in &segment_end.destination_copy_bounds {
                            if left == right || top == bottom {
                                continue;
                            }
                            let origin = wgpu::Origin3d {
                                x: *left,
                                y: *top,
                                z: 0,
                            };
                            encoder.copy_texture_to_texture(
                                wgpu::TexelCopyTextureInfo {
                                    texture: &texture,
                                    mip_level: 0,
                                    origin,
                                    aspect: wgpu::TextureAspect::All,
                                },
                                wgpu::TexelCopyTextureInfo {
                                    texture: destination_texture,
                                    mip_level: 0,
                                    origin,
                                    aspect: wgpu::TextureAspect::All,
                                },
                                wgpu::Extent3d {
                                    width: *right - *left,
                                    height: *bottom - *top,
                                    depth_or_array_layers: 1,
                                },
                            );
                        }
                    }
                    if segment_end.starts_logical_flush {
                        submit_and_wait(encoder)?;
                    }
                    segment_start = segment_end.index;
                    starts_logical_flush = segment_end.starts_logical_flush;
                }
                if !has_advanced_msaa && !resolve_fixed_msaa_directly {
                    self.context.composite_pipeline.encode(
                        &self.context.device,
                        encoder,
                        &view,
                        fallback_view
                            .as_ref()
                            .expect("fixed MSAA draw prepared without fallback target"),
                    );
                }
                Ok(())
            };
        if self.draws.is_empty() {
            encode_fallback_run(&self.draws, None, None, &[0], true, &mut encoder)?;
        } else if self.mode == RenderMode::Msaa && schedule_msaa_draws {
            if self.draws.len() > MAX_DRAWS_PER_SUBMISSION
                && msaa_draws_can_submit_independently(&self.draws)
            {
                let mut clear_target = true;
                for (flush_index, &flush_start) in self.logical_flush_starts.iter().enumerate() {
                    let flush_end = self
                        .logical_flush_starts
                        .get(flush_index + 1)
                        .copied()
                        .unwrap_or(self.draws.len());
                    let (scheduled_draws, draw_groups, draw_prepasses) = ordered_msaa_draws(
                        &self.draws[flush_start..flush_end],
                        self.width,
                        self.height,
                    );
                    for chunk_start in (0..scheduled_draws.len()).step_by(MAX_DRAWS_PER_SUBMISSION)
                    {
                        let chunk_end =
                            (chunk_start + MAX_DRAWS_PER_SUBMISSION).min(scheduled_draws.len());
                        encode_fallback_run(
                            &scheduled_draws[chunk_start..chunk_end],
                            Some(&draw_groups[chunk_start..chunk_end]),
                            Some(&draw_prepasses[chunk_start..chunk_end]),
                            &[0],
                            clear_target,
                            &mut encoder,
                        )?;
                        submit_and_wait(&mut encoder)?;
                        clear_target = false;
                    }
                }
            } else {
                let mut scheduled_draws = Vec::with_capacity(self.draws.len());
                let mut draw_groups = Vec::with_capacity(self.draws.len());
                let mut draw_prepasses = Vec::with_capacity(self.draws.len());
                let mut scheduled_flush_starts =
                    Vec::with_capacity(self.logical_flush_starts.len());
                for (flush_index, &flush_start) in self.logical_flush_starts.iter().enumerate() {
                    scheduled_flush_starts.push(scheduled_draws.len());
                    let flush_end = self
                        .logical_flush_starts
                        .get(flush_index + 1)
                        .copied()
                        .unwrap_or(self.draws.len());
                    let (flush_draws, flush_groups, flush_prepasses) = ordered_msaa_draws(
                        &self.draws[flush_start..flush_end],
                        self.width,
                        self.height,
                    );
                    scheduled_draws.extend(flush_draws);
                    draw_groups.extend(flush_groups);
                    draw_prepasses.extend(flush_prepasses);
                }
                debug_assert_eq!(scheduled_draws.len(), self.draws.len());
                encode_fallback_run(
                    &scheduled_draws,
                    Some(&draw_groups),
                    Some(&draw_prepasses),
                    &scheduled_flush_starts,
                    true,
                    &mut encoder,
                )?;
            }
        } else if self.mode == RenderMode::Msaa {
            encode_fallback_run(
                &self.draws,
                None,
                None,
                &self.logical_flush_starts,
                true,
                &mut encoder,
            )?;
        } else {
            let mut start = 0;
            let mut clear_target = true;
            let mut logical_flush_index = 0;
            let mut independent_atomic_draws_in_encoder = 0usize;
            while start < self.draws.len() {
                let logical_flush_end = self
                    .logical_flush_starts
                    .get(logical_flush_index + 1)
                    .copied()
                    .unwrap_or(self.draws.len());
                let atomic = atomic_draw_is_eligible(&self.draws[start]);
                let clockwise_atomic = atomic
                    && draw_requires_clockwise_atomic(&self.draws[start], self.width, self.height);
                let end = atomic_strategy_run_end(
                    &self.draws,
                    start,
                    logical_flush_end,
                    self.width,
                    self.height,
                );
                if atomic {
                    let has_clip_updates = self.draws[start..end]
                        .iter()
                        .any(|draw| matches!(draw.role, DrawRole::ClipUpdate { .. }));
                    let has_advanced_blend =
                        self.draws[start..end].iter().any(draw_uses_advanced_blend);
                    if has_advanced_blend {
                        if clear_target {
                            let attachments = [Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(color(self.clear_color)),
                                    store: wgpu::StoreOp::Store,
                                },
                            })];
                            let _pass =
                                encoder.begin_counted_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("nuxie-advanced-frame-clear"),
                                    color_attachments: &attachments,
                                    depth_stencil_attachment: None,
                                    timestamp_writes: None,
                                    occlusion_query_set: None,
                                    multiview_mask: None,
                                });
                        }
                        let load_texture =
                            self.context
                                .device
                                .create_texture(&wgpu::TextureDescriptor {
                                    label: Some("nuxie-advanced-destination-copy"),
                                    size: texture.size(),
                                    mip_level_count: 1,
                                    sample_count: 1,
                                    dimension: wgpu::TextureDimension::D2,
                                    format: wgpu::TextureFormat::Rgba8Unorm,
                                    usage: wgpu::TextureUsages::COPY_DST
                                        | wgpu::TextureUsages::TEXTURE_BINDING,
                                    view_formats: &[],
                                });
                        encoder.copy_texture_to_texture(
                            texture.as_image_copy(),
                            load_texture.as_image_copy(),
                            texture.size(),
                        );
                        let load_view =
                            load_texture.create_view(&wgpu::TextureViewDescriptor::default());
                        encode_atomic_run(
                            &self.draws[start..end],
                            &[0],
                            false,
                            clockwise_atomic,
                            Some(&load_view),
                            &mut encoder,
                        )?;
                    } else if clockwise_atomic || has_clip_updates {
                        encode_atomic_run(
                            &self.draws[start..end],
                            &[0],
                            clear_target,
                            clockwise_atomic,
                            None,
                            &mut encoder,
                        )?;
                    } else {
                        let groups = disjoint_atomic_draw_groups(
                            &self.draws[start..end],
                            self.width,
                            self.height,
                        );
                        let mut batch_draws = Vec::new();
                        let mut draw_group_starts = Vec::new();
                        for group in groups {
                            if independent_atomic_group_requires_submit(
                                independent_atomic_draws_in_encoder
                                    .saturating_add(batch_draws.len()),
                                group.len(),
                            ) {
                                if !batch_draws.is_empty() {
                                    encode_atomic_run(
                                        &batch_draws,
                                        &draw_group_starts,
                                        clear_target,
                                        false,
                                        None,
                                        &mut encoder,
                                    )?;
                                    clear_target = false;
                                    batch_draws.clear();
                                    draw_group_starts.clear();
                                }
                                submit_and_wait(&mut encoder)?;
                                independent_atomic_draws_in_encoder = 0;
                            }
                            draw_group_starts.push(batch_draws.len());
                            batch_draws.extend(group);
                        }
                        if !batch_draws.is_empty() {
                            encode_atomic_run(
                                &batch_draws,
                                &draw_group_starts,
                                clear_target,
                                false,
                                None,
                                &mut encoder,
                            )?;
                            independent_atomic_draws_in_encoder =
                                independent_atomic_draws_in_encoder
                                    .saturating_add(batch_draws.len());
                        }
                    }
                } else {
                    encode_fallback_run(
                        &self.draws[start..end],
                        None,
                        None,
                        &[0],
                        clear_target,
                        &mut encoder,
                    )?;
                }
                clear_target = false;
                start = end;
                if start == logical_flush_end {
                    logical_flush_index += 1;
                    if start < self.draws.len() {
                        submit_and_wait(&mut encoder)?;
                        independent_atomic_draws_in_encoder = 0;
                    }
                }
            }
        }

        if !read_pixels {
            debug_assert!(!capture_clockwise_atomic_coverage && !capture_atomic_planes);
            tessellation_uploads.borrow_mut().flush(&self.context.queue);
            self.context.queue.submit_counted(Some(encoder.finish()));
            wait_for_submitted_work(&self.context).await?;
            #[cfg(feature = "perf-diagnostics")]
            {
                let diagnostics = tessellation_uploads.borrow().diagnostics();
                eprintln!(
                    "renderer-upload-diagnostics submissions={} upload_calls={} populated_pages={} page_allocations={} payload_bytes={} used_bytes={} written_bytes={} populated_capacity_bytes={} cpu_pack_ns={} write_buffer_ns={}",
                    diagnostics.submissions,
                    diagnostics.upload_calls,
                    diagnostics.populated_pages,
                    diagnostics.page_allocations,
                    diagnostics.payload_bytes,
                    diagnostics.used_bytes,
                    diagnostics.written_bytes,
                    diagnostics.populated_capacity_bytes,
                    diagnostics.cpu_pack_ns,
                    diagnostics.write_buffer_ns,
                );
                if let Some(backing) = atomic_backing.borrow().as_ref() {
                    let diagnostics = backing.diagnostics();
                    eprintln!(
                        "renderer-atomic-encode-diagnostics batches={} draw_groups={} draws={} buffer_upload_ns={} backing_prepare_ns={} dummy_texture_ns={} sampler_create_ns={} flush_bind_groups={} flush_bind_group_ns={} image_bind_groups={} image_bind_group_ns={} load_color_bind_groups={} load_color_bind_group_ns={} atomic_bind_groups={} atomic_bind_group_ns={} sampler_bind_groups={} sampler_bind_group_ns={} render_passes={} render_encode_ns={} total_ns={}",
                        diagnostics.batches,
                        diagnostics.draw_groups,
                        diagnostics.draws,
                        diagnostics.buffer_upload_ns,
                        diagnostics.backing_prepare_ns,
                        diagnostics.dummy_texture_ns,
                        diagnostics.sampler_create_ns,
                        diagnostics.flush_bind_groups,
                        diagnostics.flush_bind_group_ns,
                        diagnostics.image_bind_groups,
                        diagnostics.image_bind_group_ns,
                        diagnostics.load_color_bind_groups,
                        diagnostics.load_color_bind_group_ns,
                        diagnostics.atomic_bind_groups,
                        diagnostics.atomic_bind_group_ns,
                        diagnostics.sampler_bind_groups,
                        diagnostics.sampler_bind_group_ns,
                        diagnostics.render_passes,
                        diagnostics.render_encode_ns,
                        diagnostics.total_ns,
                    );
                }
            }
            let backend_work = self.work_recorder.snapshot();
            self.context
                .frame_attachments
                .recycle(Arc::clone(&frame_attachments));
            return Ok((
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                backend_work,
            ));
        }

        let unpadded_bytes_per_row = self.width * 4;
        let padded_bytes_per_row =
            align_to(unpadded_bytes_per_row, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let readback = self.context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nuxie-frame-readback"),
            size: padded_bytes_per_row as u64 * self.height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            texture.size(),
        );
        tessellation_uploads.borrow_mut().flush(&self.context.queue);
        self.context.queue.submit_counted(Some(encoder.finish()));
        let slice = readback.slice(..);
        map_buffer(&self.context, &slice).await?;
        let mapped = slice
            .get_mapped_range()
            .map_err(|error| RendererError::Map(error.to_string()))?;
        let mut pixels = Vec::with_capacity(unpadded_bytes_per_row as usize * self.height as usize);
        for row in mapped.chunks_exact(padded_bytes_per_row as usize) {
            pixels.extend_from_slice(&row[..unpadded_bytes_per_row as usize]);
        }
        drop(mapped);
        readback.unmap();
        let mut coverage_snapshots = Vec::with_capacity(pending_coverage_readbacks.len());
        for (readback, ranges, kinds) in pending_coverage_readbacks {
            coverage_snapshots.push(ClockwiseAtomicCoverageSnapshot {
                borrowed: read_u32_buffer(&self.context, &readback.borrowed, readback.word_count)
                    .await?,
                main: read_u32_buffer(&self.context, &readback.main, readback.word_count).await?,
                ranges,
                kinds,
                clip_updates: {
                    let mut updates = Vec::with_capacity(readback.clip_updates.len());
                    for buffer in &readback.clip_updates {
                        updates.push(
                            read_u8_buffer(
                                &self.context,
                                buffer,
                                readback.clip_bytes_per_row as usize
                                    * readback.clip_height as usize,
                            )
                            .await?,
                        );
                    }
                    updates
                },
                clip_bytes_per_row: readback.clip_bytes_per_row,
            });
        }
        let read_atomic_snapshots = async |readbacks: &[atomic_pipeline::AtomicPlaneReadback]| {
            let mut snapshots = Vec::with_capacity(readbacks.len());
            for readback in readbacks {
                snapshots.push(
                    read_u32_buffer(&self.context, &readback.buffer, readback.word_count).await?,
                );
            }
            Ok::<_, RendererError>(snapshots)
        };
        let atomic_coverage_snapshots =
            read_atomic_snapshots(&pending_atomic_coverage_readbacks).await?;
        let atomic_clip_snapshots = read_atomic_snapshots(&pending_atomic_clip_readbacks).await?;
        let atomic_color_snapshots = read_atomic_snapshots(&pending_atomic_color_readbacks).await?;
        let backend_work = self.work_recorder.snapshot();
        self.context
            .frame_attachments
            .recycle(Arc::clone(&frame_attachments));
        Ok((
            pixels,
            coverage_snapshots,
            atomic_coverage_snapshots,
            atomic_clip_snapshots,
            atomic_color_snapshots,
            backend_work,
        ))
    }
}

async fn wait_for_submitted_work(context: &Context) -> Result<(), RendererError> {
    #[cfg(not(target_arch = "wasm32"))]
    context
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| RendererError::Map(error.to_string()))?;

    #[cfg(target_arch = "wasm32")]
    {
        let (sender, receiver) = futures_channel::oneshot::channel();
        context.queue.on_submitted_work_done(move || {
            let _ = sender.send(());
        });
        receiver
            .await
            .map_err(|error| RendererError::Map(error.to_string()))?;
    }

    Ok(())
}

async fn map_buffer(
    _context: &Context,
    slice: &wgpu::BufferSlice<'_>,
) -> Result<(), RendererError> {
    let (sender, receiver) = futures_channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    #[cfg(not(target_arch = "wasm32"))]
    _context
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| RendererError::Map(error.to_string()))?;
    receiver
        .await
        .map_err(|error| RendererError::Map(error.to_string()))?
        .map_err(|error| RendererError::Map(error.to_string()))
}

async fn read_u32_buffer(
    context: &Context,
    buffer: &wgpu::Buffer,
    word_count: usize,
) -> Result<Vec<u32>, RendererError> {
    let slice = buffer.slice(..);
    map_buffer(context, &slice).await?;
    let mapped = slice
        .get_mapped_range()
        .map_err(|error| RendererError::Map(error.to_string()))?;
    let words = mapped
        .chunks_exact(std::mem::size_of::<u32>())
        .take(word_count)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .collect();
    drop(mapped);
    buffer.unmap();
    Ok(words)
}

async fn read_u8_buffer(
    context: &Context,
    buffer: &wgpu::Buffer,
    byte_count: usize,
) -> Result<Vec<u8>, RendererError> {
    let slice = buffer.slice(..);
    map_buffer(context, &slice).await?;
    let mapped = slice
        .get_mapped_range()
        .map_err(|error| RendererError::Map(error.to_string()))?;
    let bytes = mapped[..byte_count].to_vec();
    drop(mapped);
    buffer.unmap();
    Ok(bytes)
}

fn pack_atlas_for_device(
    width: u32,
    max_dimension: u32,
    regions: &[(u32, u32)],
) -> Result<skyline::AtlasLayout, RendererError> {
    skyline::pack_atlas_regions(width, max_dimension, regions)
        .map_err(|error| RendererError::AtlasPacking(error.message()))
}

fn pack_logical_feather_atlas_for_cpp(
    max_texture_dimension: u32,
    draw_sizes: &[(u32, u32)],
) -> Result<skyline::AtlasLayout, RendererError> {
    let Some(&(first_width, first_height)) = draw_sizes.first() else {
        return skyline::pack_atlas_regions_in_dimensions(1, 1, &[])
            .map_err(|error| RendererError::AtlasPacking(error.message()));
    };
    let platform_max_texture_dimension =
        max_texture_dimension.min(CPP_WEBGPU_PLATFORM_MAX_TEXTURE_DIMENSION);
    let base_dimension = platform_max_texture_dimension.min(CPP_LOGICAL_ATLAS_MAX_DIMENSION);
    let atlas_width = base_dimension.max(first_width);
    let atlas_height = base_dimension.max(first_height);
    if atlas_width > max_texture_dimension || atlas_height > max_texture_dimension {
        return Err(RendererError::AtlasPacking(
            "atlas dimensions exceed the device texture limit",
        ));
    }
    let total_padding = FEATHER_ATLAS_PADDING * 2;
    let padded_regions = draw_sizes
        .iter()
        .map(|&(width, height)| {
            (
                width.saturating_add(total_padding).min(atlas_width),
                height.saturating_add(total_padding).min(atlas_height),
            )
        })
        .collect::<Vec<_>>();
    skyline::pack_atlas_regions_in_dimensions(atlas_width, atlas_height, &padded_regions)
        .map_err(|error| RendererError::AtlasPacking(error.message()))
}

fn atlas_physical_size(content: [u32; 2], max_dimension: u32) -> [u32; 2] {
    content.map(|dimension| (dimension.saturating_mul(5) / 4).max(1).min(max_dimension))
}

fn feather_atlas_placement(
    path: &RawPath,
    transform: Mat2D,
    feather: f32,
    stroke: Option<(f32, StrokeJoin, StrokeCap)>,
    frame_width: u32,
    frame_height: u32,
) -> Option<AtlasPlacement> {
    let scale = draw::feather_atlas_scale(feather, transform);
    let [left, top, right, bottom] = draw::feather_pixel_bounds(path, transform, feather, stroke)?;
    let left = left.clamp(0, frame_width as i32);
    let top = top.clamp(0, frame_height as i32);
    let right = right.clamp(left, frame_width as i32);
    let bottom = bottom.clamp(top, frame_height as i32);
    let padding = FEATHER_ATLAS_PADDING as f32;
    Some(AtlasPlacement {
        scale,
        translate: [
            (-(left as f32)).mul_add(scale, padding),
            (-(top as f32)).mul_add(scale, padding),
        ],
        bounds: [left as f32, top as f32, right as f32, bottom as f32],
        origin: [0, 0],
        width: ((right - left) as f32 * scale).ceil() as u32 + FEATHER_ATLAS_PADDING * 2,
        height: ((bottom - top) as f32 * scale).ceil() as u32 + FEATHER_ATLAS_PADDING * 2,
    })
}

fn msaa_destination_copy_bounds(draw: &SolidDraw, width: u32, height: u32) -> [u32; 4] {
    let width_i32 = i32::try_from(width).unwrap_or(i32::MAX);
    let height_i32 = i32::try_from(height).unwrap_or(i32::MAX);
    let [left, top, right, bottom] = draw::feather_pixel_bounds(
        &draw.path.raw_path,
        draw.state.transform,
        draw.paint.feather,
        draw.paint.effective_stroke(),
    )
    .unwrap_or([0, 0, width_i32, height_i32]);
    let left = left.clamp(0, width_i32);
    let top = top.clamp(0, height_i32);
    [
        left as u32,
        top as u32,
        right.clamp(left, width_i32) as u32,
        bottom.clamp(top, height_i32) as u32,
    ]
}

fn analytic_uniforms(width: u32, height: u32, tessellation_height: u32) -> gpu::FlushUniforms {
    let mut uniforms = gpu::FlushUniforms::zeroed();
    uniforms.inverse_viewports = [
        2.0,
        -2.0 / tessellation_height.max(1) as f32,
        2.0 / width as f32,
        -2.0 / height as f32,
    ];
    uniforms.render_target_width = width;
    uniforms.render_target_height = height;
    uniforms.path_id_granularity = 1;
    uniforms.vertex_discard_value = f32::NAN;
    uniforms.mip_map_lod_bias = gpu::MIP_MAP_LOD_BIAS;
    uniforms.max_path_id = 1;
    uniforms.dither_scale = 1.0 / 256.0;
    uniforms.dither_bias = -0.5 / 256.0;
    uniforms.dither_conversion_to_rgb10 = -0.25;
    uniforms
}

fn modulate_color_alpha(color: ColorInt, opacity: f32) -> ColorInt {
    let alpha = ((color >> 24) as f32 * opacity.clamp(0.0, 1.0)).round() as u32;
    alpha << 24 | color & 0x00ff_ffff
}

fn cover_vertices(path_vertices: &[Vertex]) -> [Vertex; 6] {
    let mut min = [f32::INFINITY; 2];
    let mut max = [f32::NEG_INFINITY; 2];
    for vertex in path_vertices {
        min[0] = min[0].min(vertex.position[0]);
        min[1] = min[1].min(vertex.position[1]);
        max[0] = max[0].max(vertex.position[0]);
        max[1] = max[1].max(vertex.position[1]);
    }
    let color = path_vertices[0].color;
    let vertex = |position| Vertex { position, color };
    [
        vertex([min[0], min[1]]),
        vertex([max[0], min[1]]),
        vertex([max[0], max[1]]),
        vertex([min[0], min[1]]),
        vertex([max[0], max[1]]),
        vertex([min[0], max[1]]),
    ]
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 8,
                    shader_location: 1,
                },
            ],
        }
    }
}

// Bootstrap subset of renderer/src/draw.cpp's path-to-triangle pipeline.
fn tessellate_solid(draw: &SolidDraw, width: u32, height: u32) -> Option<Vec<Vertex>> {
    if draw.paint.style != RenderPaintStyle::Fill || draw.paint.shader.is_some() {
        return None;
    }
    let rgba = rgba(draw.paint.color, draw.state.opacity);
    let vertex = |point: nuxie_render_api::Vec2D| Vertex {
        position: [
            point.x / width as f32 * 2.0 - 1.0,
            1.0 - point.y / height as f32 * 2.0,
        ],
        color: rgba,
    };
    let mut vertices = Vec::new();
    for contour in draw::flatten_path(&draw.path.raw_path, draw.state.transform) {
        let indices = draw::triangulate_contour(&contour.points)?;
        vertices.extend(
            indices
                .into_iter()
                .map(|index| vertex(contour.points[index as usize])),
        );
    }
    (!vertices.is_empty()).then_some(vertices)
}

fn path_aabb(path: &RawPath) -> Option<[f32; 4]> {
    let verbs = path.verbs();
    if verbs.len() < 4
        || verbs[..4]
            != [
                PathVerb::Move,
                PathVerb::Line,
                PathVerb::Line,
                PathVerb::Line,
            ]
    {
        return None;
    }
    let points = path.points();
    if points.len() < 4 || points[4..].iter().any(|point| *point != points[0]) {
        return None;
    }
    let [p0, p1, p2, p3] = points[..4] else {
        unreachable!()
    };
    let is_rect = (p0.x == p3.x && p0.y == p1.y && p2.x == p1.x && p2.y == p3.y)
        || (p0.x == p1.x && p0.y == p3.y && p2.x == p3.x && p2.y == p1.y);
    is_rect.then_some([
        p0.x.min(p2.x),
        p0.y.min(p2.y),
        p0.x.max(p2.x),
        p0.y.max(p2.y),
    ])
}

fn apply_clip_rect(state: &mut DrawState, mut rect: [f32; 4]) -> bool {
    if rect.iter().any(|value| value.is_nan()) || rect[0] >= rect[2] || rect[1] >= rect[3] {
        state.clip_is_empty = true;
        return true;
    }
    if let Some(existing) = state.clip_rect {
        let Some(transformed) = transform_rect_to_new_space(rect, state.transform, existing.matrix)
        else {
            return false;
        };
        rect = [
            existing.rect[0].max(transformed[0]),
            existing.rect[1].max(transformed[1]),
            existing.rect[2].min(transformed[2]),
            existing.rect[3].min(transformed[3]),
        ];
        if rect[0] >= rect[2] || rect[1] >= rect[3] {
            state.clip_is_empty = true;
        }
        state.clip_rect = Some(ClipRectState {
            rect,
            matrix: existing.matrix,
        });
    } else {
        state.clip_rect = Some(ClipRectState {
            rect,
            matrix: state.transform,
        });
    }
    true
}

fn transform_rect_to_new_space(
    rect: [f32; 4],
    current_matrix: Mat2D,
    new_matrix: Mat2D,
) -> Option<[f32; 4]> {
    if current_matrix == new_matrix {
        return Some(rect);
    }
    let current_to_new = multiply(invert(new_matrix)?, current_matrix);
    let [xx, yx, xy, yy, _, _] = current_to_new.0;
    let max_skew = xy.abs().max(yx.abs());
    let max_scale = xx.abs().max(yy.abs());
    if max_skew > 1e-5 && max_scale > 1e-5 {
        return None;
    }
    let p0 = current_to_new.transform_point(nuxie_render_api::Vec2D::new(rect[0], rect[1]));
    let p1 = current_to_new.transform_point(nuxie_render_api::Vec2D::new(rect[2], rect[3]));
    Some([
        p0.x.min(p1.x),
        p0.y.min(p1.y),
        p0.x.max(p1.x),
        p0.y.max(p1.y),
    ])
}

fn clip_rect_paint_aux(clip: Option<ClipRectState>) -> gpu::PaintAuxData {
    let Some(clip) = clip else {
        return gpu::PaintAuxData::zeroed();
    };
    let [left, top, right, bottom] = clip.rect;
    let normalized_rect = Mat2D([
        (right - left) * 0.5,
        0.0,
        0.0,
        (bottom - top) * 0.5,
        (left + right) * 0.5,
        (top + bottom) * 0.5,
    ]);
    let inverse = invert(multiply(clip.matrix, normalized_rect)).unwrap_or(Mat2D([0.0; 6]));
    let [xx, yx, xy, yy, _, _] = inverse.0;
    gpu::PaintAuxData {
        matrix: [0.0; 6],
        paint_value: [0.0; 2],
        clip_rect_inverse_matrix: inverse.0,
        inverse_fwidth: [-1.0 / (xx.abs() + xy.abs()), -1.0 / (yx.abs() + yy.abs())],
    }
}

fn image_paint_aux(
    clip: Option<ClipRectState>,
    transform: Mat2D,
    texture: &WgpuImageTexture,
) -> gpu::PaintAuxData {
    // Mirrors the image branch of gpu::PaintAuxData::set in renderer/src/gpu.cpp.
    let mut aux = clip_rect_paint_aux(clip);
    let inverse =
        invert(transform).expect("an enqueued image draw must have an invertible transform");
    aux.matrix = inverse.0;
    aux.paint_value[0] =
        image_texture_lod(inverse, texture.texture.width(), texture.texture.height());
    aux
}

fn image_texture_lod(inverse_transform: Mat2D, width: u32, height: u32) -> f32 {
    let [xx, yx, xy, yy, _, _] = inverse_transform.0;
    let width = width as f32;
    let height = height as f32;
    let dudx = xx * width;
    let dudy = yx * height;
    let dvdx = xy * width;
    let dvdy = yy * height;
    let max_scale_factor_pow2 = (dudx * dudx + dvdx * dvdx).max(dudy * dudy + dvdy * dvdy);
    max_scale_factor_pow2.max(1.0).log2() * 0.5 + gpu::MIP_MAP_LOD_BIAS
}

fn prepare_gradient_batch(draws: &[SolidDraw]) -> GradientBatch {
    const RAMPS_PER_SIMPLE_ROW: usize = gradient_pipeline::TEXTURE_WIDTH as usize / 2;
    const ONE_TEXEL_FIXED: u32 = 65_536 / gradient_pipeline::TEXTURE_WIDTH;
    const LEFT_BORDER: u32 = 0x8000_0000;
    const RIGHT_BORDER: u32 = 0x4000_0000;
    const COMPLEX_BORDER: u32 = 0x2000_0000;

    let definitions = draws
        .iter()
        .map(|draw| {
            draw.paint
                .shader
                .as_ref()
                .and_then(|shader| normalize_gradient(shader, draw.state.opacity))
        })
        .collect::<Vec<_>>();
    let is_simple = |gradient: &GradientDefinition| {
        gradient.stops.len() == 1
            || (gradient.stops.len() == 2 && gradient.stops[0] == 0.0 && gradient.stops[1] == 1.0)
    };
    let simple_count = definitions
        .iter()
        .flatten()
        .filter(|gradient| is_simple(gradient))
        .count();
    let complex_count = definitions
        .iter()
        .flatten()
        .filter(|gradient| !is_simple(gradient))
        .count();
    let simple_height = simple_count.div_ceil(RAMPS_PER_SIMPLE_ROW) as u32;
    let height = simple_height + complex_count as u32;
    let mut simple_index = 0usize;
    let mut complex_index = 0u32;
    let mut spans = Vec::new();
    let mut prepared = Vec::with_capacity(draws.len());
    for (draw, gradient) in draws.iter().zip(definitions) {
        let Some(gradient) = gradient else {
            prepared.push(None);
            continue;
        };
        let (row, texture_span) = if is_simple(&gradient) {
            let row = (simple_index / RAMPS_PER_SIMPLE_ROW) as u32;
            let left = ((simple_index % RAMPS_PER_SIMPLE_ROW) * 2) as u32;
            let center_fixed = (left + 1) * ONE_TEXEL_FIXED;
            let color0 = gradient.colors[0];
            let color1 = gradient.colors.get(1).copied().unwrap_or(color0);
            spans.push(gpu::GradientSpan::new(
                center_fixed,
                center_fixed,
                row,
                LEFT_BORDER | RIGHT_BORDER,
                color0,
                color1,
            ));
            simple_index += 1;
            (
                row,
                [
                    1.0 / gradient_pipeline::TEXTURE_WIDTH as f32,
                    (left as f32 + 0.5) / gradient_pipeline::TEXTURE_WIDTH as f32,
                ],
            )
        } else {
            let row = simple_height + complex_index;
            let scale = (gradient_pipeline::TEXTURE_WIDTH - 1) as f32 * ONE_TEXEL_FIXED as f32;
            let bias = 0.5 * ONE_TEXEL_FIXED as f32;
            let mut last_x = (gradient.stops[0] * scale + bias) as u32;
            let mut last_color = gradient.colors[0];
            for index in 1..gradient.stops.len() {
                let x = (gradient.stops[index] * scale + bias) as u32;
                let mut flags = COMPLEX_BORDER;
                if index == 1 {
                    flags |= LEFT_BORDER;
                }
                if index + 1 == gradient.stops.len() {
                    flags |= RIGHT_BORDER;
                }
                spans.push(gpu::GradientSpan::new(
                    last_x,
                    x,
                    row,
                    flags,
                    last_color,
                    gradient.colors[index],
                ));
                last_x = x;
                last_color = gradient.colors[index];
            }
            complex_index += 1;
            (
                row,
                [
                    (gradient_pipeline::TEXTURE_WIDTH - 1) as f32
                        / gradient_pipeline::TEXTURE_WIDTH as f32,
                    0.5 / gradient_pipeline::TEXTURE_WIDTH as f32,
                ],
            )
        };
        let inverse = invert(draw.state.transform).unwrap_or(Mat2D([0.0; 6]));
        let gradient_matrix = match gradient.paint_type {
            gpu::PaintType::LinearGradient => Mat2D([
                gradient.coeffs[0],
                0.0,
                gradient.coeffs[1],
                0.0,
                gradient.coeffs[2],
                0.0,
            ]),
            gpu::PaintType::RadialGradient => {
                let inverse_radius = gradient.coeffs[2].recip();
                Mat2D([
                    inverse_radius,
                    0.0,
                    0.0,
                    inverse_radius,
                    -gradient.coeffs[0] * inverse_radius,
                    -gradient.coeffs[1] * inverse_radius,
                ])
            }
            _ => unreachable!(),
        };
        prepared.push(Some(PreparedGradient {
            paint_type: gradient.paint_type,
            texture_y: (row as f32 + 0.5) / height as f32,
            matrix: multiply(gradient_matrix, inverse),
            texture_span,
        }));
    }
    GradientBatch {
        spans,
        height,
        draws: prepared,
    }
}

fn normalize_gradient(shader: &WgpuShader, opacity: f32) -> Option<GradientDefinition> {
    const EPSILON: f32 = 1.0 / 4096.0;
    let (paint_type, mut colors, stops, coeffs) = match shader {
        WgpuShader::Linear {
            start,
            end,
            colors,
            stops,
        } => {
            let mut start = *start;
            let mut end = *end;
            let mut stops = stops.clone();
            validate_gradient(colors, &stops)?;
            let first = stops[0];
            let last = *stops.last()?;
            if (first != 0.0 || last != 1.0) && last - first > EPSILON {
                let original_start = start;
                let original_end = end;
                start = (
                    original_start.0 + (original_end.0 - original_start.0) * first,
                    original_start.1 + (original_end.1 - original_start.1) * first,
                );
                end = (
                    original_start.0 + (original_end.0 - original_start.0) * last,
                    original_start.1 + (original_end.1 - original_start.1) * last,
                );
                let inverse_range = (last - first).recip();
                for stop in &mut stops {
                    *stop = (*stop - first) * inverse_range;
                }
                stops[0] = 0.0;
                *stops.last_mut().unwrap() = 1.0;
                let final_index = stops.len() - 1;
                for index in 1..final_index {
                    stops[index] = stops[index].max(stops[index - 1]);
                }
                for index in (1..final_index).rev() {
                    stops[index] = stops[index].min(stops[index + 1]);
                }
            }
            let dx = end.0 - start.0;
            let dy = end.1 - start.1;
            let inverse_length_squared = (dx * dx + dy * dy).recip();
            let vx = dx * inverse_length_squared;
            let vy = dy * inverse_length_squared;
            (
                gpu::PaintType::LinearGradient,
                colors.clone(),
                stops,
                [vx, vy, -(vx * start.0 + vy * start.1)],
            )
        }
        WgpuShader::Radial {
            center,
            radius,
            colors,
            stops,
        } => {
            let mut radius = *radius;
            let mut stops = stops.clone();
            validate_gradient(colors, &stops)?;
            let last = *stops.last()?;
            if last != 1.0 && last > EPSILON {
                radius *= last;
                let inverse_last = last.recip();
                let final_index = stops.len() - 1;
                for stop in &mut stops[..final_index] {
                    *stop *= inverse_last;
                }
                *stops.last_mut().unwrap() = 1.0;
                stops[0] = stops[0].max(0.0);
                for index in 1..final_index {
                    stops[index] = stops[index].max(stops[index - 1]);
                }
                for index in (0..final_index).rev() {
                    stops[index] = stops[index].min(stops[index + 1]);
                }
            }
            (
                gpu::PaintType::RadialGradient,
                colors.clone(),
                stops,
                [center.0, center.1, radius],
            )
        }
    };
    for color in &mut colors {
        *color = modulate_color_alpha(*color, opacity);
    }
    Some(GradientDefinition {
        paint_type,
        colors,
        stops,
        coeffs,
    })
}

fn validate_gradient(colors: &[ColorInt], stops: &[f32]) -> Option<()> {
    if colors.len() != stops.len()
        || stops.is_empty()
        || stops
            .iter()
            .any(|stop| !stop.is_finite() || !(0.0..=1.0).contains(stop))
        || stops.windows(2).any(|pair| pair[0] > pair[1])
    {
        return None;
    }
    Some(())
}

fn gradient_paint_aux(
    clip: Option<ClipRectState>,
    gradient: PreparedGradient,
) -> gpu::PaintAuxData {
    let mut aux = clip_rect_paint_aux(clip);
    aux.matrix = gradient.matrix.0;
    aux.paint_value = gradient.texture_span;
    aux
}

fn wgpu_path(path: &dyn RenderPath) -> Option<&WgpuPath> {
    path.as_any().downcast_ref()
}

fn wgpu_paint(paint: &dyn RenderPaint) -> Option<&WgpuPaint> {
    paint.as_any().downcast_ref()
}

fn wgpu_buffer(buffer: &dyn RenderBuffer) -> Option<&WgpuBuffer> {
    buffer.as_any().downcast_ref()
}

fn multiply(left: Mat2D, right: Mat2D) -> Mat2D {
    let [a, b, c, d, tx, ty] = left.0;
    let [e, f, g, h, ux, uy] = right.0;
    Mat2D([
        a * e + c * f,
        b * e + d * f,
        a * g + c * h,
        b * g + d * h,
        a * ux + c * uy + tx,
        b * ux + d * uy + ty,
    ])
}

fn decode_image_rgba(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        decode_png_rgba(data)
    } else if data.starts_with(&[0xff, 0xd8]) {
        decode_jpeg_rgba(data)
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CoreGraphicsPoint {
    x: f64,
    y: f64,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CoreGraphicsSize {
    width: f64,
    height: f64,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CoreGraphicsRect {
    origin: CoreGraphicsPoint,
    size: CoreGraphicsSize,
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
#[link(name = "CoreGraphics", kind = "framework")]
#[link(name = "ImageIO", kind = "framework")]
unsafe extern "C" {
    fn CFDataCreate(allocator: *const c_void, bytes: *const u8, length: isize) -> *const c_void;
    fn CFRelease(cf: *const c_void);
    fn CGImageSourceCreateWithData(data: *const c_void, options: *const c_void) -> *const c_void;
    fn CGImageSourceCreateImageAtIndex(
        source: *const c_void,
        index: usize,
        options: *const c_void,
    ) -> *const c_void;
    fn CGImageGetAlphaInfo(image: *const c_void) -> u32;
    fn CGImageGetWidth(image: *const c_void) -> usize;
    fn CGImageGetHeight(image: *const c_void) -> usize;
    fn CGColorSpaceCreateDeviceRGB() -> *const c_void;
    fn CGColorSpaceRelease(space: *const c_void);
    fn CGBitmapContextCreate(
        data: *mut c_void,
        width: usize,
        height: usize,
        bits_per_component: usize,
        bytes_per_row: usize,
        space: *const c_void,
        bitmap_info: u32,
    ) -> *mut c_void;
    fn CGContextSetBlendMode(context: *mut c_void, mode: i32);
    fn CGContextDrawImage(context: *mut c_void, rect: CoreGraphicsRect, image: *const c_void);
    fn CGContextRelease(context: *mut c_void);
}

#[cfg(target_os = "macos")]
fn decode_macos_image_rgba(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    const ALPHA_PREMULTIPLIED_LAST: u32 = 1;
    const ALPHA_NONE: u32 = 0;
    const ALPHA_NONE_SKIP_LAST: u32 = 5;
    const ALPHA_NONE_SKIP_FIRST: u32 = 6;
    const BYTE_ORDER_32_BIG: u32 = 4 << 12;
    const BLEND_MODE_COPY: i32 = 17;

    let data_length = isize::try_from(data.len()).ok()?;
    let encoded = unsafe { CFDataCreate(std::ptr::null(), data.as_ptr(), data_length) };
    if encoded.is_null() {
        return None;
    }
    let source = unsafe { CGImageSourceCreateWithData(encoded, std::ptr::null()) };
    unsafe { CFRelease(encoded) };
    if source.is_null() {
        return None;
    }
    let image = unsafe { CGImageSourceCreateImageAtIndex(source, 0, std::ptr::null()) };
    unsafe { CFRelease(source) };
    if image.is_null() {
        return None;
    }

    let image_width = unsafe { CGImageGetWidth(image) };
    let image_height = unsafe { CGImageGetHeight(image) };
    let Some(row_bytes) = image_width.checked_mul(4) else {
        unsafe { CFRelease(image) };
        return None;
    };
    let Some(byte_count) = row_bytes.checked_mul(image_height) else {
        unsafe { CFRelease(image) };
        return None;
    };
    let (Ok(width), Ok(height)) = (u32::try_from(image_width), u32::try_from(image_height)) else {
        unsafe { CFRelease(image) };
        return None;
    };
    let alpha_info = unsafe { CGImageGetAlphaInfo(image) };
    let opaque = matches!(
        alpha_info,
        ALPHA_NONE | ALPHA_NONE_SKIP_LAST | ALPHA_NONE_SKIP_FIRST
    );
    let color_space = unsafe { CGColorSpaceCreateDeviceRGB() };
    if color_space.is_null() {
        unsafe { CFRelease(image) };
        return None;
    }
    let mut pixels = vec![0; byte_count];
    let bitmap_info = BYTE_ORDER_32_BIG
        | if opaque {
            ALPHA_NONE_SKIP_LAST
        } else {
            ALPHA_PREMULTIPLIED_LAST
        };
    let context = unsafe {
        CGBitmapContextCreate(
            pixels.as_mut_ptr().cast(),
            image_width,
            image_height,
            8,
            row_bytes,
            color_space,
            bitmap_info,
        )
    };
    unsafe { CGColorSpaceRelease(color_space) };
    if context.is_null() {
        unsafe { CFRelease(image) };
        return None;
    }
    unsafe {
        CGContextSetBlendMode(context, BLEND_MODE_COPY);
        CGContextDrawImage(
            context,
            CoreGraphicsRect {
                origin: CoreGraphicsPoint { x: 0.0, y: 0.0 },
                size: CoreGraphicsSize {
                    width: f64::from(width),
                    height: f64::from(height),
                },
            },
            image,
        );
        CGContextRelease(context);
        CFRelease(image);
    }
    Some((width, height, pixels))
}

#[cfg(feature = "decode-oracle")]
#[doc(hidden)]
pub struct DecodedImageRgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[cfg(feature = "decode-oracle")]
#[doc(hidden)]
pub fn decode_image_rgba_for_oracle(data: &[u8]) -> Option<DecodedImageRgba> {
    decode_image_rgba(data).map(|(width, height, pixels)| DecodedImageRgba {
        width,
        height,
        pixels,
    })
}

fn decode_png_rgba(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = png::Decoder::new(Cursor::new(data));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let icc_profile = reader
        .info()
        .icc_profile
        .as_ref()
        .map(|profile| profile.as_ref().to_vec());
    let mut decoded = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut decoded).ok()?;
    decoded.truncate(info.buffer_size());
    let mut pixels = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => decoded,
        (png::ColorType::Rgb, png::BitDepth::Eight) => decoded
            .chunks_exact(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
            .collect(),
        (png::ColorType::Grayscale, png::BitDepth::Eight) => decoded
            .into_iter()
            .flat_map(|value| [value, value, value, 255])
            .collect(),
        (png::ColorType::GrayscaleAlpha, png::BitDepth::Eight) => decoded
            .chunks_exact(2)
            .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
            .collect(),
        _ => return None,
    };
    if let Some(profile) = icc_profile {
        convert_icc_rgba_to_srgb(&mut pixels, info.width, &profile);
    }
    premultiply_rgba(&mut pixels);
    Some((info.width, info.height, pixels))
}

fn convert_icc_rgba_to_srgb(pixels: &mut [u8], width: u32, icc_profile: &[u8]) {
    // C++ decoders/src/bitmap_decoder_thirdparty.cpp draws the profiled image
    // into a DeviceRGB RGBA bitmap before premultiplying its alpha.
    let Ok(source) = moxcms::ColorProfile::new_from_slice(icc_profile) else {
        return;
    };
    let destination = moxcms::ColorProfile::new_srgb();
    let Ok(transform) = source.create_transform_8bit(
        moxcms::Layout::Rgba,
        &destination,
        moxcms::Layout::Rgba,
        moxcms::TransformOptions::default(),
    ) else {
        return;
    };
    let Some(row_bytes) = usize::try_from(width)
        .ok()
        .and_then(|width| width.checked_mul(4))
    else {
        return;
    };
    if row_bytes == 0 || !pixels.len().is_multiple_of(row_bytes) {
        return;
    }
    let mut converted = vec![0; pixels.len()];
    for (source, destination) in pixels
        .chunks_exact(row_bytes)
        .zip(converted.chunks_exact_mut(row_bytes))
    {
        if transform.transform(source, destination).is_err() {
            return;
        }
    }
    pixels.copy_from_slice(&converted);
}

#[cfg(target_os = "macos")]
fn decode_jpeg_rgba(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    decode_macos_image_rgba(data)
}

#[cfg(not(target_os = "macos"))]
fn decode_jpeg_rgba(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(data));
    let decoded = decoder.decode().ok()?;
    let info = decoder.info()?;
    let icc_profile = decoder.icc_profile();
    let mut pixels: Vec<u8> = match info.pixel_format {
        jpeg_decoder::PixelFormat::L8 => decoded
            .into_iter()
            .flat_map(|value| [value, value, value, 255])
            .collect(),
        jpeg_decoder::PixelFormat::L16 => decoded
            .chunks_exact(2)
            .flat_map(|value| [value[0], value[0], value[0], 255])
            .collect(),
        jpeg_decoder::PixelFormat::RGB24 => decoded
            .chunks_exact(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
            .collect(),
        jpeg_decoder::PixelFormat::CMYK32 => decoded
            .chunks_exact(4)
            .flat_map(|cmyk| {
                let key = u16::from(cmyk[3]);
                [
                    ((u16::from(cmyk[0]) * key + 127) / 255) as u8,
                    ((u16::from(cmyk[1]) * key + 127) / 255) as u8,
                    ((u16::from(cmyk[2]) * key + 127) / 255) as u8,
                    255,
                ]
            })
            .collect(),
    };
    if let Some(profile) = icc_profile {
        convert_icc_rgba_to_srgb(&mut pixels, u32::from(info.width), &profile);
    }
    premultiply_rgba(&mut pixels);
    Some((u32::from(info.width), u32::from(info.height), pixels))
}

fn premultiply_rgba(pixels: &mut [u8]) {
    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = u16::from(pixel[3]);
        for channel in &mut pixel[..3] {
            *channel = ((u16::from(*channel) * alpha + 127) / 255) as u8;
        }
    }
}

fn image_clip_rect_inverse_matrix(clip: Option<ClipRectState>) -> [f32; 6] {
    clip.map(|clip| clip_rect_paint_aux(Some(clip)).clip_rect_inverse_matrix)
        .unwrap_or([0.0, 0.0, 0.0, 0.0, 1.0, 1.0])
}

fn invert(matrix: Mat2D) -> Option<Mat2D> {
    let [xx, yx, xy, yy, tx, ty] = matrix.0;
    let determinant = xx * yy - xy * yx;
    if determinant == 0.0 || !determinant.is_finite() {
        return None;
    }
    let inverse_determinant = determinant.recip();
    Some(Mat2D([
        yy * inverse_determinant,
        -yx * inverse_determinant,
        -xy * inverse_determinant,
        xx * inverse_determinant,
        (xy * ty - yy * tx) * inverse_determinant,
        (yx * tx - xx * ty) * inverse_determinant,
    ]))
}

fn rgba(value: ColorInt, opacity: f32) -> [f32; 4] {
    let [alpha, red, green, blue] = value.to_be_bytes();
    [
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
        alpha as f32 / 255.0 * opacity,
    ]
}

fn color(value: ColorInt) -> wgpu::Color {
    let [alpha, red, green, blue] = value.to_be_bytes();
    let premul = |channel: u8| f64::from(u16::from(channel) * u16::from(alpha) / 255) / 255.0;
    wgpu::Color {
        r: premul(red),
        g: premul(green),
        b: premul(blue),
        a: f64::from(alpha) / 255.0,
    }
}

fn swizzle_rive_color_to_rgba_premul(value: ColorInt) -> u32 {
    let [alpha, red, green, blue] = value.to_be_bytes();
    let premul = |channel: u8| u32::from(channel) * u32::from(alpha) / 255;
    premul(red) | premul(green) << 8 | premul(blue) << 16 | u32::from(alpha) << 24
}

fn align_to(value: u32, alignment: u32) -> u32 {
    value.div_ceil(alignment) * alignment
}

fn path_draw_is_noop(path: &WgpuPath, paint: &WgpuPaint, transform: Mat2D) -> bool {
    path.raw_path.verbs().is_empty()
        || (paint.style == RenderPaintStyle::Stroke && !(paint.thickness > 0.0))
        || !(paint.feather >= 0.0)
        || (paint.style == RenderPaintStyle::Fill
            && (draw::build_fill_tessellation(&path.raw_path, transform).is_none()
                || fill_path_is_collinear(&path.raw_path)))
}

fn path_draw_is_outside_frame(
    path: &WgpuPath,
    paint: &WgpuPaint,
    transform: Mat2D,
    width: u32,
    height: u32,
) -> bool {
    let Some([left, top, right, bottom]) = path_draw_pixel_bounds(path, paint, transform) else {
        return true;
    };
    left >= width as i32
        || top >= height as i32
        || right <= 0
        || bottom <= 0
        || left >= right
        || top >= bottom
}

fn path_draw_pixel_bounds(
    path: &WgpuPath,
    paint: &WgpuPaint,
    transform: Mat2D,
) -> Option<[i32; 4]> {
    if paint.style == RenderPaintStyle::Stroke || paint.feather != 0.0 {
        draw::feather_pixel_bounds(
            &path.raw_path,
            transform,
            paint.feather,
            paint.effective_stroke(),
        )
    } else {
        draw::path_pixel_bounds(&path.raw_path, transform)
    }
}

fn invert_clockwise_path(
    path: &RawPath,
    fill_rule: FillRule,
    transform: Mat2D,
    width: u32,
    height: u32,
) -> Option<RawPath> {
    let inverse = invert(transform)?;
    let mut bounds = [
        inverse.transform_point(nuxie_render_api::Vec2D::new(0.0, 0.0)),
        inverse.transform_point(nuxie_render_api::Vec2D::new(width as f32, 0.0)),
        inverse.transform_point(nuxie_render_api::Vec2D::new(width as f32, height as f32)),
        inverse.transform_point(nuxie_render_api::Vec2D::new(0.0, height as f32)),
    ];
    let determinant = transform.0[0] * transform.0[3] - transform.0[2] * transform.0[1];
    if determinant < 0.0 {
        bounds.swap(1, 3);
    }
    let mut inverse_path = RawPath::new();
    inverse_path.move_to(bounds[0].x, bounds[0].y);
    inverse_path.line_to(bounds[1].x, bounds[1].y);
    inverse_path.line_to(bounds[2].x, bounds[2].y);
    inverse_path.line_to(bounds[3].x, bounds[3].y);
    if fill_rule == FillRule::Clockwise || draw::path_coarse_area(path) >= 0.0 {
        inverse_path.add_path_backwards(path, Mat2D::IDENTITY);
    } else {
        inverse_path.add_path(path, Mat2D::IDENTITY);
    }
    Some(inverse_path)
}

fn atomic_strategy_run_end(
    draws: &[SolidDraw],
    start: usize,
    logical_flush_end: usize,
    width: u32,
    height: u32,
) -> usize {
    let atomic = atomic_draw_is_eligible(&draws[start]);
    let clockwise_atomic = atomic && draw_requires_clockwise_atomic(&draws[start], width, height);
    let advanced_clockwise_atomic = clockwise_atomic && draw_uses_advanced_blend(&draws[start]);
    let mut end = start + 1;
    while end < logical_flush_end
        && atomic_draw_is_eligible(&draws[end]) == atomic
        && (!atomic
            || draw_requires_clockwise_atomic(&draws[end], width, height) == clockwise_atomic)
        && (!clockwise_atomic
            || (!advanced_clockwise_atomic && !draw_uses_advanced_blend(&draws[end])))
    {
        end += 1;
    }
    end
}

fn atomic_draw_is_eligible(draw: &SolidDraw) -> bool {
    if matches!(draw.role, DrawRole::ClipUpdate { .. }) {
        return draw::build_fill_tessellation(&draw.path.raw_path, draw.state.transform).is_some();
    }
    if matches!(&draw.image, Some(ImageDraw::Mesh(_))) {
        return true;
    }
    if draw.paint.feather != 0.0 {
        return draw::build_feather_tessellation(
            &draw.path.raw_path,
            draw.state.transform,
            draw.paint.feather,
            draw.paint.effective_stroke(),
        )
        .is_some();
    }
    match draw.paint.style {
        RenderPaintStyle::Fill => {
            draw::build_fill_tessellation(&draw.path.raw_path, draw.state.transform).is_some()
                || (draw::should_use_interior_tessellation(
                    &draw.path.raw_path,
                    draw.state.transform,
                ) && draw::build_interior_tessellation(
                    &draw.path.raw_path,
                    draw.state.transform,
                    draw.path.fill_rule,
                    false,
                )
                .is_some())
        }
        RenderPaintStyle::Stroke => draw::build_stroke_tessellation(
            &draw.path.raw_path,
            draw.state.transform,
            draw.paint.thickness,
            draw.paint.join,
            draw.paint.cap,
        )
        .is_some(),
    }
}

fn tessellated_segment_count(spans: &[gpu::TessVertexSpan]) -> usize {
    spans
        .iter()
        .filter(|span| span.contour_id_with_flags & gpu::CONTOUR_ID_MASK != 0)
        .count()
}

fn midpoint_resource_counts(
    tessellation: &draw::FillTessellation,
    draw_pass_count: usize,
) -> logical_flush::ResourceCounters {
    logical_flush::ResourceCounters {
        midpoint_fan_tess_vertex_count: tessellation.instance_count as usize
            * gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN,
        path_count: 1,
        contour_count: tessellation.contours.len(),
        max_tessellated_segment_count: tessellated_segment_count(&tessellation.spans),
        draw_pass_count,
        ..Default::default()
    }
}

fn interior_resource_counts(
    tessellation: &draw::InteriorTessellation,
    draw_pass_count: usize,
) -> logical_flush::ResourceCounters {
    logical_flush::ResourceCounters {
        outer_cubic_tess_vertex_count: tessellation.instance_count as usize
            * gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN,
        path_count: 1,
        contour_count: tessellation.contours.len(),
        max_tessellated_segment_count: tessellated_segment_count(&tessellation.spans),
        max_triangle_vertex_count: tessellation.triangles.len(),
        draw_pass_count,
        ..Default::default()
    }
}

fn logical_flush_draw_resources(
    draw: &SolidDraw,
    mode: RenderMode,
    width: u32,
    height: u32,
) -> logical_flush::ResourceCounters {
    if matches!(draw.role, DrawRole::ClipReset { .. }) {
        return logical_flush::ResourceCounters {
            max_triangle_vertex_count: 6,
            draw_pass_count: 1,
            ..Default::default()
        };
    }
    if matches!(draw.image, Some(ImageDraw::Mesh(_)))
        || (mode == RenderMode::ClockwiseAtomic && matches!(draw.image, Some(ImageDraw::Rect(_))))
    {
        return logical_flush::ResourceCounters {
            image_draw_count: 1,
            draw_pass_count: 1,
            ..Default::default()
        };
    }

    if mode == RenderMode::Msaa {
        if draw.paint.feather != 0.0 {
            let stroke = draw.paint.effective_stroke();
            let direction = draw::feather_atlas_fill_direction(
                draw.state.transform,
                draw.path.fill_rule,
                stroke.is_some(),
            );
            return draw::build_feather_tessellation_with_direction(
                &draw.path.raw_path,
                draw.state.transform,
                draw.paint.feather,
                stroke,
                direction,
            )
            .map(|tessellation| midpoint_resource_counts(&tessellation, 1))
            .unwrap_or(logical_flush::ResourceCounters {
                draw_pass_count: 1,
                ..Default::default()
            });
        }
        let pass_count = if draw.paint.style == RenderPaintStyle::Stroke
            || matches!(draw.role, DrawRole::ClipUpdate { parent_id, .. } if parent_id != 0)
        {
            1
        } else if draw.path.fill_rule == FillRule::EvenOdd {
            2
        } else {
            3
        };
        let tessellation = match draw.paint.style {
            RenderPaintStyle::Fill => {
                draw::build_fill_tessellation(&draw.path.raw_path, draw.state.transform)
            }
            RenderPaintStyle::Stroke => draw::build_stroke_tessellation(
                &draw.path.raw_path,
                draw.state.transform,
                draw.paint.thickness,
                draw.paint.join,
                draw.paint.cap,
            ),
        };
        return tessellation
            .map(|tessellation| midpoint_resource_counts(&tessellation, pass_count))
            .unwrap_or(logical_flush::ResourceCounters {
                draw_pass_count: pass_count,
                ..Default::default()
            });
    }

    let outermost_clip = matches!(draw.role, DrawRole::ClipUpdate { parent_id: 0, .. });
    let nested_clip = matches!(draw.role, DrawRole::ClipUpdate { parent_id, .. } if parent_id != 0);
    let inverse_path = nested_clip
        .then(|| {
            invert_clockwise_path(
                &draw.path.raw_path,
                draw.path.fill_rule,
                draw.state.transform,
                width,
                height,
            )
        })
        .flatten();
    let raw_path = inverse_path.as_ref().unwrap_or(&draw.path.raw_path);
    let fill_rule = if inverse_path.is_some() {
        FillRule::Clockwise
    } else {
        draw.path.fill_rule
    };
    if draw.paint.feather != 0.0 {
        let stroke = draw.paint.effective_stroke();
        let is_stroke = stroke.is_some();
        let uses_atlas =
            draw::feather_requires_atlas(draw.paint.feather, draw.state.transform, false);
        let negate_coverage =
            draw::clockwise_atomic_negate_coverage(raw_path, draw.state.transform, fill_rule, true);
        let direction = if is_stroke {
            draw::FeatherFillDirection::Forward
        } else {
            match (uses_atlas, negate_coverage) {
                (true, true) => draw::FeatherFillDirection::Reverse,
                (true, false) => draw::FeatherFillDirection::Forward,
                (false, true) => draw::FeatherFillDirection::ForwardThenReverse,
                (false, false) => draw::FeatherFillDirection::ReverseThenForward,
            }
        };
        let pass_count = if uses_atlas || is_stroke || outermost_clip {
            1
        } else {
            2
        };
        return draw::build_feather_tessellation_with_direction(
            raw_path,
            draw.state.transform,
            draw.paint.feather,
            stroke,
            direction,
        )
        .map(|tessellation| midpoint_resource_counts(&tessellation, pass_count))
        .unwrap_or(logical_flush::ResourceCounters {
            draw_pass_count: pass_count,
            ..Default::default()
        });
    }
    if draw.paint.style == RenderPaintStyle::Stroke {
        return draw::build_stroke_tessellation(
            raw_path,
            draw.state.transform,
            draw.paint.thickness,
            draw.paint.join,
            draw.paint.cap,
        )
        .map(|tessellation| midpoint_resource_counts(&tessellation, 1))
        .unwrap_or(logical_flush::ResourceCounters {
            draw_pass_count: 1,
            ..Default::default()
        });
    }
    if draw::should_use_interior_tessellation(raw_path, draw.state.transform) {
        if let Some(tessellation) =
            draw::build_interior_tessellation(raw_path, draw.state.transform, fill_rule, true)
        {
            let interior =
                interior_resource_counts(&tessellation, if outermost_clip { 2 } else { 4 });
            let contour_count = raw_path
                .verbs()
                .iter()
                .filter(|verb| **verb == PathVerb::Move)
                .count();
            if contour_count > 1 {
                return interior;
            }
            if let Some(mut midpoint) =
                draw::build_fill_tessellation(raw_path, draw.state.transform)
            {
                midpoint.make_double_sided_with_direction(draw::clockwise_atomic_negate_coverage(
                    raw_path,
                    draw.state.transform,
                    fill_rule,
                    true,
                ));
                let midpoint =
                    midpoint_resource_counts(&midpoint, if outermost_clip { 1 } else { 2 });
                // A surrounding global clip can switch a single-contour draw
                // from interior to midpoint tessellation at run assembly.
                // Reserve both texture sections so either encoded form fits.
                return logical_flush::ResourceCounters {
                    midpoint_fan_tess_vertex_count: midpoint.midpoint_fan_tess_vertex_count,
                    outer_cubic_tess_vertex_count: interior.outer_cubic_tess_vertex_count,
                    path_count: 1,
                    contour_count: midpoint.contour_count.max(interior.contour_count),
                    max_tessellated_segment_count: midpoint
                        .max_tessellated_segment_count
                        .max(interior.max_tessellated_segment_count),
                    max_triangle_vertex_count: interior.max_triangle_vertex_count,
                    image_draw_count: 0,
                    draw_pass_count: midpoint.draw_pass_count.max(interior.draw_pass_count),
                };
            }
            return interior;
        }
    }
    draw::build_fill_tessellation(raw_path, draw.state.transform)
        .map(|mut tessellation| {
            tessellation.make_double_sided_with_direction(draw::clockwise_atomic_negate_coverage(
                raw_path,
                draw.state.transform,
                fill_rule,
                true,
            ));
            midpoint_resource_counts(&tessellation, if outermost_clip { 1 } else { 2 })
        })
        .unwrap_or(logical_flush::ResourceCounters {
            draw_pass_count: 1,
            ..Default::default()
        })
}

fn logical_flush_batch_resources(
    draws: &[SolidDraw],
    mode: RenderMode,
    width: u32,
    height: u32,
) -> Option<logical_flush::ResourceCounters> {
    draws.iter().try_fold(
        logical_flush::ResourceCounters::default(),
        |mut total, draw| {
            let draw = logical_flush_draw_resources(draw, mode, width, height);
            total.midpoint_fan_tess_vertex_count = total
                .midpoint_fan_tess_vertex_count
                .checked_add(draw.midpoint_fan_tess_vertex_count)?;
            total.outer_cubic_tess_vertex_count = total
                .outer_cubic_tess_vertex_count
                .checked_add(draw.outer_cubic_tess_vertex_count)?;
            total.path_count = total.path_count.checked_add(draw.path_count)?;
            total.contour_count = total.contour_count.checked_add(draw.contour_count)?;
            total.max_tessellated_segment_count = total
                .max_tessellated_segment_count
                .checked_add(draw.max_tessellated_segment_count)?;
            total.max_triangle_vertex_count = total
                .max_triangle_vertex_count
                .checked_add(draw.max_triangle_vertex_count)?;
            total.image_draw_count = total.image_draw_count.checked_add(draw.image_draw_count)?;
            total.draw_pass_count = total.draw_pass_count.checked_add(draw.draw_pass_count)?;
            Some(total)
        },
    )
}

fn atomic_paint_fill_rule(
    source_fill_rule: FillRule,
    use_clockwise_atomic_batch: bool,
) -> FillRule {
    if use_clockwise_atomic_batch {
        FillRule::Clockwise
    } else {
        source_fill_rule
    }
}

fn draw_uses_advanced_blend(draw: &SolidDraw) -> bool {
    draw.paint.blend_mode != BlendMode::SrcOver
        || draw
            .image
            .as_ref()
            .is_some_and(|image| image.blend_mode() != BlendMode::SrcOver)
}

fn blend_mode_uses_hsl(mode: BlendMode) -> bool {
    matches!(
        mode,
        BlendMode::Hue | BlendMode::Saturation | BlendMode::Color | BlendMode::Luminosity
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MsaaFillPass {
    BorrowedCoverage,
    Forward,
    Cleanup,
    ClockwiseCleanup,
    EvenOddStencil,
    EvenOddCover,
}

fn msaa_fill_passes(fill_rule: FillRule) -> &'static [MsaaFillPass] {
    const NON_ZERO: &[MsaaFillPass] = &[
        MsaaFillPass::BorrowedCoverage,
        MsaaFillPass::Forward,
        MsaaFillPass::Cleanup,
    ];
    const CLOCKWISE: &[MsaaFillPass] = &[
        MsaaFillPass::BorrowedCoverage,
        MsaaFillPass::Forward,
        MsaaFillPass::ClockwiseCleanup,
    ];
    const EVEN_ODD: &[MsaaFillPass] = &[MsaaFillPass::EvenOddStencil, MsaaFillPass::EvenOddCover];
    match fill_rule {
        FillRule::NonZero => NON_ZERO,
        FillRule::Clockwise => CLOCKWISE,
        FillRule::EvenOdd => EVEN_ODD,
    }
}

fn msaa_fill_pipeline_kind(fill_pass: MsaaFillPass) -> path_pipeline::DirectPathPipelineKind {
    match fill_pass {
        MsaaFillPass::BorrowedCoverage => path_pipeline::DirectPathPipelineKind::FillBorrowed,
        MsaaFillPass::Forward => path_pipeline::DirectPathPipelineKind::FillForward,
        MsaaFillPass::Cleanup => path_pipeline::DirectPathPipelineKind::FillCleanup,
        MsaaFillPass::ClockwiseCleanup => {
            path_pipeline::DirectPathPipelineKind::ClockwiseFillCleanup
        }
        MsaaFillPass::EvenOddStencil => path_pipeline::DirectPathPipelineKind::EvenOddFillStencil,
        MsaaFillPass::EvenOddCover => path_pipeline::DirectPathPipelineKind::EvenOddFillCover,
    }
}

fn msaa_draw_layer_count(draw: &SolidDraw, all_subpasses_in_same_group: bool) -> i16 {
    // C++ reserves max(prepassCount, subpassCount) board layers, except when
    // destination-copy blending requires every MSAA subpass to share a group.
    if all_subpasses_in_same_group
        || draw.paint.feather != 0.0
        || draw.paint.style == RenderPaintStyle::Stroke
        || matches!(
            draw.role,
            DrawRole::ClipUpdate { parent_id, .. } if parent_id != 0
        )
        || matches!(draw.role, DrawRole::ClipReset { .. })
    {
        1
    } else if draw.path.fill_rule == FillRule::EvenOdd {
        2
    } else {
        3
    }
}

fn msaa_draw_rect(
    draw: &SolidDraw,
    viewport_width: u32,
    viewport_height: u32,
) -> intersection_board::Rect {
    let bounds = match draw.role {
        DrawRole::ClipReset { bounds, .. } => [
            bounds[0].floor() as i32,
            bounds[1].floor() as i32,
            bounds[2].ceil() as i32,
            bounds[3].ceil() as i32,
        ],
        _ => draw::feather_pixel_bounds(
            &draw.path.raw_path,
            draw.state.transform,
            draw.paint.feather,
            draw.paint.effective_stroke(),
        )
        .unwrap_or([0, 0, viewport_width as i32, viewport_height as i32]),
    };
    intersection_board::Rect::new(
        bounds[0].saturating_sub(1),
        bounds[1].saturating_sub(1),
        bounds[2].saturating_add(1),
        bounds[3].saturating_add(1),
    )
}

fn disjoint_msaa_draw_indices(
    draws: &[SolidDraw],
    viewport_width: u32,
    viewport_height: u32,
) -> Vec<Vec<usize>> {
    const MAX_SAFE_GROUP: i32 = i16::MAX as i32 - 1;

    // Rust currently has one logical flush per frame, so this is the
    // conservative equivalent of C++'s combined-draw-contents check.
    let all_subpasses_in_same_group = draws.iter().any(draw_uses_advanced_blend);
    let mut board =
        intersection_board::IntersectionBoard::new(intersection_board::GroupingType::Disjoint);
    board.resize_and_reset(viewport_width, viewport_height);
    let mut groups = Vec::<Vec<usize>>::new();
    let mut group_base = 0usize;
    let mut board_max_group = 0i32;

    for (draw_index, draw) in draws.iter().enumerate() {
        let layer_count = msaa_draw_layer_count(draw, all_subpasses_in_same_group);
        if board_max_group > MAX_SAFE_GROUP - i32::from(layer_count) {
            board.resize_and_reset(viewport_width, viewport_height);
            group_base = groups.len();
            board_max_group = 0;
        }
        let rect = msaa_draw_rect(draw, viewport_width, viewport_height);
        let local_group = board.add_rectangle(rect, layer_count).max(1) as usize;
        board_max_group = board_max_group.max(local_group as i32 + i32::from(layer_count) - 1);
        let group_index = group_base + local_group - 1;
        if groups.len() <= group_index {
            groups.resize_with(group_index + 1, Vec::new);
        }
        groups[group_index].push(draw_index);
    }

    // Preserve empty slots because the vector index is the draw group's
    // z-index. A nonzero fill reserves three contiguous C++ draw groups, so
    // compacting the vector would move a later overlapping fill from z=4 to
    // z=2 and break depth ordering between its subpasses.
    groups
}

#[cfg(test)]
fn disjoint_msaa_draw_groups(
    draws: &[SolidDraw],
    viewport_width: u32,
    viewport_height: u32,
) -> Vec<Vec<SolidDraw>> {
    disjoint_msaa_draw_indices(draws, viewport_width, viewport_height)
        .into_iter()
        .map(|group| {
            group
                .into_iter()
                .map(|draw_index| draws[draw_index].clone())
                .collect()
        })
        .collect()
}

fn msaa_draw_has_opaque_paint(draw: &SolidDraw) -> bool {
    draw.image.is_none() && draw.paint.is_opaque()
}

fn direct_stroke_can_batch(draw: &SolidDraw, has_gradient: bool) -> bool {
    draw.paint.style == RenderPaintStyle::Stroke
        && draw.paint.feather == 0.0
        && draw.paint.blend_mode == BlendMode::SrcOver
        && draw.paint.is_opaque()
        && draw.state.opacity == 1.0
        && draw.state.clip_rect.is_none()
        && draw.state.clip_stack_height == 0
        && matches!(draw.role, DrawRole::Content { clip_id: 0 })
        && draw.image.is_none()
        && !has_gradient
}

fn msaa_draw_uses_opaque_prepass(draw: &SolidDraw) -> bool {
    msaa_draw_has_opaque_paint(draw) && matches!(draw.role, DrawRole::Content { clip_id: 0 })
}

fn msaa_destination_copy_head(
    draw_groups: &[u32],
    draw_prepasses: &[bool],
    logical_flushes: &[usize],
    destination_read_index: usize,
) -> usize {
    debug_assert_eq!(draw_groups.len(), draw_prepasses.len());
    debug_assert_eq!(draw_groups.len(), logical_flushes.len());
    let group = draw_groups[destination_read_index];
    let logical_flush = logical_flushes[destination_read_index];
    draw_groups
        .iter()
        .zip(draw_prepasses)
        .zip(logical_flushes)
        .position(|((&candidate_group, &is_prepass), &candidate_flush)| {
            candidate_group == group && candidate_flush == logical_flush && !is_prepass
        })
        .expect("MSAA destination-read group must contain a subpass")
}

fn ordered_msaa_draws(
    draws: &[SolidDraw],
    viewport_width: u32,
    viewport_height: u32,
) -> (Vec<SolidDraw>, Vec<u32>, Vec<bool>) {
    // renderer/src/render_context.cpp sorts opaque MSAA prepasses one subpass
    // at a time. Rust still prepares the passes of a fill as one draw, so it
    // can only move that draw as a unit when all passes share a draw group or
    // the draw has a single pass. Destination-read flushes force the former,
    // which preserves C++ barrier placement without reordering multi-layer
    // fills as indivisible units.
    let all_subpasses_in_same_group = draws.iter().any(draw_uses_advanced_blend);
    let mut prepasses = Vec::<(u32, usize)>::new();
    let mut subpasses = Vec::<(u32, usize)>::new();
    for (group_index, group) in disjoint_msaa_draw_indices(draws, viewport_width, viewport_height)
        .into_iter()
        .enumerate()
    {
        let z_index = u32::try_from(group_index + 1)
            .expect("MSAA draw group must fit the path-data contract");
        for draw_index in group {
            let draw = &draws[draw_index];
            let can_move_as_one_prepass =
                msaa_draw_layer_count(draw, all_subpasses_in_same_group) == 1;
            if msaa_draw_uses_opaque_prepass(draw) && can_move_as_one_prepass {
                prepasses.push((z_index, draw_index));
            } else {
                subpasses.push((z_index, draw_index));
            }
        }
    }
    prepasses.sort_by(|left, right| right.0.cmp(&left.0));
    let prepass_count = prepasses.len();
    prepasses.extend(subpasses);
    let mut scheduled_draws = Vec::with_capacity(prepasses.len());
    let mut draw_groups = Vec::with_capacity(prepasses.len());
    let mut draw_prepasses = Vec::with_capacity(prepasses.len());
    for (index, (z_index, draw_index)) in prepasses.into_iter().enumerate() {
        draw_groups.push(z_index);
        draw_prepasses.push(index < prepass_count);
        scheduled_draws.push(draws[draw_index].clone());
    }
    (scheduled_draws, draw_groups, draw_prepasses)
}

fn msaa_draws_can_submit_independently(draws: &[SolidDraw]) -> bool {
    draws.iter().all(|draw| {
        matches!(draw.role, DrawRole::Content { clip_id: 0 })
            && draw.state.clip_stack_height == 0
            && !draw_uses_advanced_blend(draw)
    })
}

fn disjoint_atomic_draw_groups(
    draws: &[SolidDraw],
    viewport_width: u32,
    viewport_height: u32,
) -> Vec<Vec<SolidDraw>> {
    disjoint_atomic_draw_groups_with_limit(
        draws,
        viewport_width,
        viewport_height,
        i16::MAX as usize - 1,
    )
}

fn independent_atomic_group_requires_submit(
    draws_in_encoder: usize,
    next_group_draws: usize,
) -> bool {
    draws_in_encoder != 0
        && draws_in_encoder.saturating_add(next_group_draws) > MAX_DRAWS_PER_SUBMISSION
}

fn disjoint_atomic_draw_groups_with_limit(
    draws: &[SolidDraw],
    viewport_width: u32,
    viewport_height: u32,
    group_limit: usize,
) -> Vec<Vec<SolidDraw>> {
    disjoint_atomic_draw_groups_with_limits(
        draws,
        viewport_width,
        viewport_height,
        group_limit,
        MAX_ATOMIC_PATHS,
    )
}

fn disjoint_atomic_draw_groups_with_limits(
    draws: &[SolidDraw],
    viewport_width: u32,
    viewport_height: u32,
    group_limit: usize,
    path_limit: usize,
) -> Vec<Vec<SolidDraw>> {
    assert!((1..i16::MAX as usize).contains(&group_limit));
    assert!(path_limit > 0);
    let mut board =
        intersection_board::IntersectionBoard::new(intersection_board::GroupingType::Disjoint);
    board.resize_and_reset(viewport_width, viewport_height);
    let mut groups = Vec::<Vec<SolidDraw>>::new();
    let mut group_base = 0usize;
    let mut board_group_count = 0usize;
    for draw in draws {
        // IntersectionBoard computes `bottom + layer_count - 1` in i16, so
        // i16::MAX itself is not a safe returned group for a one-layer draw.
        if board_group_count == group_limit {
            board.resize_and_reset(viewport_width, viewport_height);
            group_base = groups.len();
            board_group_count = 0;
        }
        let bounds = if matches!(draw.role, DrawRole::ClipUpdate { .. }) {
            [0, 0, viewport_width as i32, viewport_height as i32]
        } else {
            draw::feather_pixel_bounds(
                &draw.path.raw_path,
                draw.state.transform,
                draw.paint.feather,
                draw.paint.effective_stroke(),
            )
            .unwrap_or([0, 0, viewport_width as i32, viewport_height as i32])
        };
        let rect = intersection_board::Rect::new(
            bounds[0].saturating_sub(1),
            bounds[1].saturating_sub(1),
            bounds[2].saturating_add(1),
            bounds[3].saturating_add(1),
        );
        let local_group = board.add_rectangle(rect, 1).max(1) as usize;
        board_group_count = board_group_count.max(local_group);
        let group_index = group_base + local_group - 1;
        if groups.len() <= group_index {
            groups.resize_with(group_index + 1, Vec::new);
        }
        groups[group_index].push(draw.clone());
    }
    groups
        .into_iter()
        .flat_map(|group| {
            group
                .chunks(path_limit)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<_>>()
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompactMidpointGroup {
    range: std::ops::Range<usize>,
    geometry_end: u32,
}

fn compact_midpoint_groups<K: Eq>(
    keys: &[K],
    instance_ranges: &[(u32, u32)],
    max_height: u32,
) -> Option<Vec<CompactMidpointGroup>> {
    if keys.len() != instance_ranges.len() || keys.len() < 2 {
        return None;
    }
    let midpoint_span = gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
    let outer_span = gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32;
    let mut groups = Vec::new();
    let mut start = 0;
    while start < keys.len() {
        let end = (start + 1..keys.len())
            .find(|&index| keys[index] != keys[start])
            .unwrap_or(keys.len());
        let geometry_end = instance_ranges[start..end].iter().try_fold(
            midpoint_span,
            |end, &(base_instance, instance_count)| {
                (base_instance == 1)
                    .then_some(())
                    .and_then(|()| instance_count.checked_mul(midpoint_span))
                    .and_then(|count| end.checked_add(count))
            },
        )?;
        if geometry_end > gpu::TESS_TEXTURE_WIDTH as u32
            || align_to(geometry_end, outer_span) >= gpu::TESS_TEXTURE_WIDTH as u32
        {
            return None;
        }
        groups.push(CompactMidpointGroup {
            range: start..end,
            geometry_end,
        });
        start = end;
    }
    (groups.len() <= max_height as usize && groups.iter().any(|group| group.range.len() > 1))
        .then_some(groups)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MidpointShelfPlacement {
    x: u32,
    y: u32,
    next_x: u32,
    next_y: u32,
    height: u32,
}

fn midpoint_shelf_placement(
    mut cursor_x: u32,
    mut cursor_y: u32,
    local_height: u32,
    single_row_width: Option<u32>,
) -> MidpointShelfPlacement {
    if let Some(width) = single_row_width {
        if cursor_x + width > gpu::TESS_TEXTURE_WIDTH as u32 {
            cursor_x = 0;
            cursor_y += 1;
        }
        MidpointShelfPlacement {
            x: cursor_x,
            y: cursor_y,
            next_x: cursor_x + width,
            next_y: cursor_y,
            height: cursor_y + 1,
        }
    } else {
        if cursor_x != 0 {
            cursor_y += 1;
        }
        MidpointShelfPlacement {
            x: 0,
            y: cursor_y,
            next_x: 0,
            next_y: cursor_y + local_height,
            height: cursor_y + local_height,
        }
    }
}

fn midpoint_tessellation_single_row_width(spans: &[gpu::TessVertexSpan]) -> Option<u32> {
    let mut right = 0i32;
    for span in spans {
        if span.y != 0.0 {
            return None;
        }
        let (x0, x1) = span.x_range();
        if x0 < 0 || x1 < x0 {
            return None;
        }
        right = right.max(x1);
        if span.reflection_y.is_finite() {
            if span.reflection_y != 0.0 {
                return None;
            }
            let reflection_x0 = span.reflection_x0_x1 as i16 as i32;
            let reflection_x1 = (span.reflection_x0_x1 >> 16) as i16 as i32;
            if reflection_x0 < 0 || reflection_x1 < 0 {
                return None;
            }
            right = right.max(reflection_x0.max(reflection_x1));
        }
    }
    let right = u32::try_from(right).ok()?.checked_add(1)?;
    Some(align_to(
        right.max(gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32),
        gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32,
    ))
}

fn relocate_midpoint_tessellation(
    spans: &mut [gpu::TessVertexSpan],
    base_instance: &mut u32,
    contours: &mut [gpu::ContourData],
    x: u32,
    y: u32,
) {
    relocate_tessellation(
        spans,
        base_instance,
        contours,
        x,
        y,
        gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32,
    );
}

fn relocate_midpoint_tessellation_logically(
    spans: &mut Vec<gpu::TessVertexSpan>,
    base_instance: &mut u32,
    contours: &mut [gpu::ContourData],
    next_base_instance: u32,
) {
    relocate_tessellation_logically(
        spans,
        base_instance,
        contours,
        next_base_instance,
        gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32,
    );
}

fn relocate_tessellation_logically(
    spans: &mut Vec<gpu::TessVertexSpan>,
    base_instance: &mut u32,
    contours: &mut [gpu::ContourData],
    next_base_instance: u32,
    segment_span: u32,
) {
    #[derive(PartialEq, Eq)]
    struct SourceSpanKey {
        points: [[u32; 2]; 4],
        join_tangent: [u32; 2],
        logical_x0: i64,
        logical_x1: i64,
        reflection_location: Option<u32>,
        segment_counts: u32,
        contour_id_with_flags: u32,
    }

    let texture_width = gpu::TESS_TEXTURE_WIDTH as u32;
    let old_base = base_instance
        .checked_mul(segment_span)
        .expect("tessellation source location overflow");
    let new_base = next_base_instance
        .checked_mul(segment_span)
        .expect("tessellation destination location overflow");
    let relocation = new_base
        .checked_sub(old_base)
        .expect("compact tessellation layout must move forward");
    let source = std::mem::take(spans);
    spans.reserve(source.len());
    let mut previous_key = None;
    for mut span in source {
        let (source_x0, source_x1) = span.x_range();
        debug_assert!(span.y >= 0.0 && span.y.fract() == 0.0);
        debug_assert!(source_x1 >= source_x0);
        let vertex_count = u32::try_from(source_x1 - source_x0)
            .expect("tessellation span width must be non-negative");
        let vertex_count_i32 =
            i32::try_from(vertex_count).expect("tessellation span width fits i32");
        let source_logical_x0 = (span.y as i64)
            .checked_mul(i64::from(texture_width))
            .and_then(|row| row.checked_add(i64::from(source_x0)))
            .expect("tessellation source span location overflow");
        let source_logical_x1 = source_logical_x0
            .checked_add(i64::from(vertex_count))
            .expect("tessellation source span end overflow");
        let source_reflection_location = span.reflection_y.is_finite().then(|| {
            debug_assert!(span.reflection_y >= 0.0 && span.reflection_y.fract() == 0.0);
            let source_reflection_x0 = span.reflection_x0_x1 as i16 as i32;
            (span.reflection_y as u32)
                .wrapping_mul(texture_width)
                .wrapping_add_signed(source_reflection_x0)
        });
        let key = SourceSpanKey {
            points: span.points.map(|point| point.map(f32::to_bits)),
            join_tangent: span.join_tangent.map(f32::to_bits),
            logical_x0: source_logical_x0,
            logical_x1: source_logical_x1,
            reflection_location: source_reflection_location,
            segment_counts: span.segment_counts,
            contour_id_with_flags: span.contour_id_with_flags,
        };
        if previous_key.as_ref() == Some(&key) {
            continue;
        }
        previous_key = Some(key);

        let logical_x0 = u32::try_from(source_logical_x0)
            .expect("tessellation span start must be non-negative")
            .checked_add(relocation)
            .expect("tessellation span relocation overflow");
        let mut y = logical_x0 / texture_width;
        let mut x0 =
            i32::try_from(logical_x0 % texture_width).expect("tessellation span x must fit i32");
        let mut x1 = x0
            .checked_add(vertex_count_i32)
            .expect("tessellation span end overflow");

        if let Some(source_reflection_location) = source_reflection_location {
            let source_reflection_x0 = span.reflection_x0_x1 as i16 as i32;
            let source_reflection_x1 = (span.reflection_x0_x1 >> 16) as i16 as i32;
            debug_assert!(source_reflection_x0 >= source_reflection_x1);
            debug_assert_eq!(
                source_reflection_x0 - source_reflection_x1,
                vertex_count_i32
            );
            let reflection_location = source_reflection_location.wrapping_add(relocation);
            let reflection_last = reflection_location.wrapping_sub(1);
            let mut reflection_y = reflection_last / texture_width;
            let mut reflection_x0 = i32::try_from(reflection_last % texture_width + 1)
                .expect("tessellation reflection x must fit i32");
            let mut reflection_x1 = reflection_x0 - vertex_count_i32;
            loop {
                span.y = y as f32;
                span.set_ranges(x0, x1, reflection_x0, reflection_x1, reflection_y as f32);
                spans.push(span);
                if x1 <= gpu::TESS_TEXTURE_WIDTH && reflection_x1 >= 0 {
                    break;
                }
                y += 1;
                x0 -= gpu::TESS_TEXTURE_WIDTH;
                x1 -= gpu::TESS_TEXTURE_WIDTH;
                reflection_y = reflection_y.wrapping_sub(1);
                reflection_x0 += gpu::TESS_TEXTURE_WIDTH;
                reflection_x1 += gpu::TESS_TEXTURE_WIDTH;
            }
        } else {
            loop {
                span.y = y as f32;
                span.set_ranges(x0, x1, -1, -1, f32::NAN);
                spans.push(span);
                if x1 <= gpu::TESS_TEXTURE_WIDTH {
                    break;
                }
                y += 1;
                x0 -= gpu::TESS_TEXTURE_WIDTH;
                x1 -= gpu::TESS_TEXTURE_WIDTH;
            }
        }
    }
    *base_instance = next_base_instance;
    for contour in contours {
        contour.vertex_index0 = contour
            .vertex_index0
            .checked_add(relocation)
            .expect("MSAA midpoint contour relocation overflow");
    }
}

fn relocate_tessellation(
    spans: &mut [gpu::TessVertexSpan],
    base_instance: &mut u32,
    contours: &mut [gpu::ContourData],
    x: u32,
    y: u32,
    patch_segment_span: u32,
) {
    let logical_offset = y * gpu::TESS_TEXTURE_WIDTH as u32 + x;
    assert_eq!(logical_offset % patch_segment_span, 0);
    *base_instance += logical_offset / patch_segment_span;
    for contour in contours {
        contour.vertex_index0 += logical_offset;
    }
    for span in spans {
        span.y += y as f32;
        let (x0, x1) = span.x_range();
        let mut reflection_y = span.reflection_y;
        let mut reflection_x0 = span.reflection_x0_x1 as i16 as i32;
        let mut reflection_x1 = (span.reflection_x0_x1 >> 16) as i16 as i32;
        if reflection_y.is_finite() {
            reflection_y += y as f32;
            reflection_x0 += x as i32;
            reflection_x1 += x as i32;
        }
        span.set_ranges(
            x0 + x as i32,
            x1 + x as i32,
            reflection_x0,
            reflection_x1,
            reflection_y,
        );
    }
}

fn append_tessellation_padding_span(spans: &mut Vec<gpu::TessVertexSpan>, x0: u32, x1: u32) {
    if x0 == x1 {
        return;
    }
    let texture_width = gpu::TESS_TEXTURE_WIDTH as u32;
    let mut y = x0 / texture_width;
    let mut local_x0 = i32::try_from(x0 % texture_width).expect("padding x must fit i32");
    let mut local_x1 = local_x0 + i32::try_from(x1 - x0).expect("padding span width must fit i32");
    loop {
        spans.push(gpu::TessVertexSpan::without_reflection(
            [[0.0; 2]; 4],
            [0.0; 2],
            y as f32,
            local_x0,
            local_x1,
            0,
            0,
            1,
            0,
        ));
        if local_x1 <= gpu::TESS_TEXTURE_WIDTH {
            break;
        }
        y += 1;
        local_x0 -= gpu::TESS_TEXTURE_WIDTH;
        local_x1 -= gpu::TESS_TEXTURE_WIDTH;
    }
}

fn append_tessellation_padding_span_at_y(
    spans: &mut Vec<gpu::TessVertexSpan>,
    x0: u32,
    x1: u32,
    y: u32,
) {
    if x0 == x1 {
        return;
    }
    spans.push(gpu::TessVertexSpan::without_reflection(
        [[0.0; 2]; 4],
        [0.0; 2],
        y as f32,
        x0 as i32,
        x1 as i32,
        0,
        0,
        1,
        0,
    ));
}

fn append_compact_midpoint_tessellation_to_flush(
    tessellation: &mut draw::FillTessellation,
    path_id: u32,
    next_base_instance: u32,
    y: u32,
    spans: &mut Vec<gpu::TessVertexSpan>,
    contours: &mut Vec<gpu::ContourData>,
) -> u32 {
    let instance_count = tessellation.instance_count;
    tessellation
        .spans
        .retain(|span| span.contour_id_with_flags & gpu::CONTOUR_ID_MASK != 0);
    let row_base_instance = y
        .checked_mul(gpu::TESS_TEXTURE_WIDTH as u32 / gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32)
        .and_then(|base| base.checked_add(next_base_instance))
        .expect("compact MSAA midpoint row base overflow");
    relocate_midpoint_tessellation_logically(
        &mut tessellation.spans,
        &mut tessellation.base_instance,
        &mut tessellation.contours,
        row_base_instance,
    );
    append_midpoint_tessellation_data_to_flush(tessellation, path_id, spans, contours);
    next_base_instance
        .checked_add(instance_count)
        .expect("compact MSAA midpoint instance range overflow")
}

fn append_midpoint_tessellation_to_flush(
    tessellation: &mut draw::FillTessellation,
    path_id: u32,
    x: u32,
    y: u32,
    spans: &mut Vec<gpu::TessVertexSpan>,
    contours: &mut Vec<gpu::ContourData>,
) {
    relocate_midpoint_tessellation(
        &mut tessellation.spans,
        &mut tessellation.base_instance,
        &mut tessellation.contours,
        x,
        y,
    );
    append_midpoint_tessellation_data_to_flush(tessellation, path_id, spans, contours);
}

fn append_midpoint_tessellation_data_to_flush(
    tessellation: &mut draw::FillTessellation,
    path_id: u32,
    spans: &mut Vec<gpu::TessVertexSpan>,
    contours: &mut Vec<gpu::ContourData>,
) {
    let contour_offset = u32::try_from(contours.len()).expect("MSAA contour offset overflow");
    let mut local_contour_ids = tessellation
        .spans
        .iter()
        .map(|span| span.contour_id_with_flags & gpu::CONTOUR_ID_MASK)
        .filter(|id| *id != 0)
        .collect::<Vec<_>>();
    local_contour_ids.sort_unstable();
    local_contour_ids.dedup();
    assert_eq!(local_contour_ids.len(), tessellation.contours.len());
    let local_contour_slots = local_contour_ids.last().copied().unwrap_or(0);
    let contour_end = contour_offset
        .checked_add(local_contour_slots)
        .expect("MSAA contour ID overflow");
    assert!(contour_end <= gpu::CONTOUR_ID_MASK);
    contours.resize(contour_end as usize, gpu::ContourData::zeroed());
    for span in &mut tessellation.spans {
        let local_id = span.contour_id_with_flags & gpu::CONTOUR_ID_MASK;
        if local_id == 0 {
            continue;
        }
        let global_id = contour_offset
            .checked_add(local_id)
            .expect("MSAA contour ID overflow");
        assert!(global_id <= gpu::CONTOUR_ID_MASK);
        span.contour_id_with_flags =
            (span.contour_id_with_flags & !gpu::CONTOUR_ID_MASK) | global_id;
    }
    for (local_id, contour) in local_contour_ids
        .into_iter()
        .zip(&mut tessellation.contours)
    {
        contour.path_id = path_id;
        contours[(contour_offset + local_id - 1) as usize] = *contour;
    }
    spans.extend_from_slice(&tessellation.spans);
}

fn draw_requires_clockwise_atomic(
    draw: &SolidDraw,
    viewport_width: u32,
    viewport_height: u32,
) -> bool {
    // C++ keeps the frame-wide clockwise override when a clip is reduced to a
    // paint-space rectangle; the generic atomic shader only models non-zero.
    matches!(draw.role, DrawRole::Content { clip_id: 0 })
        && draw.paint.style == RenderPaintStyle::Fill
        && draw.paint.feather == 0.0
        && path_has_complex_fill_topology(&draw.path.raw_path)
        && draw::clockwise_atomic_coverage_range(
            &draw.path.raw_path,
            draw.state.transform,
            viewport_width,
            viewport_height,
            0,
        )
        .is_some()
}

struct ClockwiseAtomicMainTriangles {
    vertices: Vec<gpu::TriangleVertex>,
    batches: Vec<clockwise_atomic_pipeline::ClockwiseAtomicTriangleBatch>,
}

fn clockwise_atomic_main_triangles(
    triangles: &[gpu::TriangleVertex],
    expand_unclipped_winding: bool,
) -> ClockwiseAtomicMainTriangles {
    let mut vertices = Vec::new();
    let mut batches = Vec::<clockwise_atomic_pipeline::ClockwiseAtomicTriangleBatch>::new();
    for triangle in triangles.chunks_exact(3) {
        let weight = triangle[0].weight_path_id >> 16;
        debug_assert!(
            triangle
                .iter()
                .all(|vertex| vertex.weight_path_id >> 16 == weight),
            "interior triangle vertices must share a winding weight"
        );
        if weight > 0 {
            let vertex_start = vertices.len() as u32;
            if expand_unclipped_winding {
                vertices.extend(triangle.iter().copied().map(|mut vertex| {
                    vertex.weight_path_id = (1 << 16) | (vertex.weight_path_id & 0xffff);
                    vertex
                }));
                let instance_count = weight as u32;
                if let Some(batch) = batches.last_mut().filter(|batch| {
                    instance_count == 1
                        && batch.instance_count == 1
                        && batch.vertex_start + batch.vertex_count == vertex_start
                }) {
                    batch.vertex_count += 3;
                } else {
                    batches.push(clockwise_atomic_pipeline::ClockwiseAtomicTriangleBatch {
                        vertex_start,
                        vertex_count: 3,
                        instance_count,
                    });
                }
            } else {
                vertices.extend_from_slice(triangle);
            }
        }
    }
    if !expand_unclipped_winding && !vertices.is_empty() {
        batches.push(clockwise_atomic_pipeline::ClockwiseAtomicTriangleBatch {
            vertex_start: 0,
            vertex_count: vertices.len() as u32,
            instance_count: 1,
        });
    }
    ClockwiseAtomicMainTriangles { vertices, batches }
}

fn clockwise_atomic_clip_is_inactive(draw: &SolidDraw) -> bool {
    let Some(clip) = draw.state.clip_rect else {
        return true;
    };
    let Some(clip_bounds) = transform_rect_to_new_space(clip.rect, clip.matrix, Mat2D::IDENTITY)
    else {
        return false;
    };
    let Some(path_bounds) = draw::path_pixel_bounds(&draw.path.raw_path, draw.state.transform)
    else {
        return false;
    };
    clip_bounds[0] <= path_bounds[0] as f32 - 1.0
        && clip_bounds[1] <= path_bounds[1] as f32 - 1.0
        && clip_bounds[2] >= path_bounds[2] as f32 + 1.0
        && clip_bounds[3] >= path_bounds[3] as f32 + 1.0
}

fn path_has_complex_fill_topology(path: &RawPath) -> bool {
    let mut contours = Vec::<Vec<Vec2D>>::new();
    let mut contour = Vec::new();
    let mut point_index = 0;
    for verb in path.verbs() {
        let point_count = match verb {
            PathVerb::Move | PathVerb::Line => 1,
            PathVerb::Quad => 2,
            PathVerb::Cubic => 3,
            PathVerb::Close => 0,
        };
        if *verb == PathVerb::Move && !contour.is_empty() {
            contours.push(std::mem::take(&mut contour));
        }
        if point_count != 0 {
            contour.push(path.points()[point_index + point_count - 1]);
            point_index += point_count;
        }
    }
    if !contour.is_empty() {
        contours.push(contour);
    }
    for contour in &mut contours {
        contour.dedup();
        if contour.len() > 1 && contour.first() == contour.last() {
            contour.pop();
        }
    }
    contours.retain(|contour| !contour.is_empty());
    if contours.len() > 1 {
        return true;
    }
    let Some(points) = contours.first() else {
        return false;
    };
    if points.len() < 4 {
        return false;
    }
    for first in 0..points.len() {
        let first_end = (first + 1) % points.len();
        for second in first + 1..points.len() {
            let second_end = (second + 1) % points.len();
            if first_end == second || second_end == first {
                continue;
            }
            if line_segments_intersect(
                points[first],
                points[first_end],
                points[second],
                points[second_end],
            ) {
                return true;
            }
        }
    }
    false
}

fn line_segments_intersect(a: Vec2D, b: Vec2D, c: Vec2D, d: Vec2D) -> bool {
    fn cross(a: Vec2D, b: Vec2D, c: Vec2D) -> f32 {
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    }
    fn contains(a: Vec2D, b: Vec2D, point: Vec2D) -> bool {
        point.x >= a.x.min(b.x)
            && point.x <= a.x.max(b.x)
            && point.y >= a.y.min(b.y)
            && point.y <= a.y.max(b.y)
    }

    let ac = cross(a, b, c);
    let ad = cross(a, b, d);
    let ca = cross(c, d, a);
    let cb = cross(c, d, b);
    (ac.signum() != ad.signum() && ca.signum() != cb.signum())
        || (ac == 0.0 && contains(a, b, c))
        || (ad == 0.0 && contains(a, b, d))
        || (ca == 0.0 && contains(c, d, a))
        || (cb == 0.0 && contains(c, d, b))
}

fn fill_path_is_collinear(path: &RawPath) -> bool {
    let mut points = path.points().iter().copied();
    let Some(origin) = points.next() else {
        return true;
    };
    let Some(axis_point) = points.find(|point| point.x != origin.x || point.y != origin.y) else {
        return true;
    };
    let axis = (axis_point.x - origin.x, axis_point.y - origin.y);
    points.all(|point| {
        let relative = (point.x - origin.x, point.y - origin.y);
        axis.0 * relative.1 - axis.1 * relative.0 == 0.0
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::mpsc;

    // Keep these synchronized with tools/cpp-atlas-mask-oracle/runtime-src/main.cpp.
    const ATLAS_ORACLE_FRAME_SIZE: u32 = 64;
    const ATLAS_ORACLE_PHYSICAL_SIZE: u32 = 48;
    const ATLAS_ORACLE_LOGICAL_SIZE: u32 = 39;
    const ATLAS_ORACLE_PLACEMENT: [f32; 2] = [2.0, 2.0];
    const ATLAS_ORACLE_SQUARE_MIN: f32 = 16.0;
    const ATLAS_ORACLE_SQUARE_MAX: f32 = 48.0;
    const ATLAS_ORACLE_STROKE_THICKNESS: f32 = 8.0;
    const ATLAS_ORACLE_STROKE_JOIN: StrokeJoin = StrokeJoin::Miter;
    const ATLAS_ORACLE_STROKE_CAP: StrokeCap = StrokeCap::Butt;
    const ATLAS_ORACLE_FEATHER: f32 = 20.0;
    const LARGE_FEATHER_FRAME_SIZE: [u32; 2] = [1756, 2048];
    const LARGE_FEATHER_SCALE: f32 = 1.46300006;
    const LARGE_FEATHER_RADIUS: f32 = 403.428802;
    const STROKES_ROUND_ORACLE_FRAME_SIZE: u32 = 400;
    const STROKES_ROUND_ORACLE_THICKNESS: f32 = 4.5;
    const SPOTIFY_FOOT_ORACLE_FRAME_WIDTH: u32 = 369;
    const SPOTIFY_FOOT_ORACLE_FRAME_HEIGHT: u32 = 781;
    const RAWTEXT_ORACLE_FRAME_WIDTH: u32 = 400;
    const RAWTEXT_ORACLE_FRAME_HEIGHT: u32 = 335;
    const ATOMIC_COLORBURN_PAIR_FRAME_SIZE: u32 = 1024;
    const ATOMIC_INTERLEAVED_FEATHER_FULL_FRAME_SIZE: u32 = 1000;
    const ATOMIC_DSTREADSHUFFLE_FULL_FRAME_WIDTH: u32 = 530;
    const ATOMIC_DSTREADSHUFFLE_FULL_FRAME_HEIGHT: u32 = 690;
    const ATOMIC_SPOTIFY_FULL_FRAME_WIDTH: u32 = 1024;
    const ATOMIC_SPOTIFY_FULL_FRAME_HEIGHT: u32 = 1436;
    const ATOMIC_SPOTIFY_FULL_STORAGE_HEIGHT: u32 = 1440;
    const ATLAS_ORACLE_TOLERANCES: atlas_mask_oracle::MaskComparisonTolerances =
        atlas_mask_oracle::MaskComparisonTolerances {
            support: 1.0 / 1024.0,
            value: 1.0 / 512.0,
        };

    #[test]
    fn texture_extent_validation_rejects_zero_and_oversized_dimensions() {
        assert!(validate_texture_extent("test", 1, 8, 8).is_ok());
        assert!(validate_texture_extent("test", 8, 8, 8).is_ok());
        assert!(matches!(
            validate_texture_extent("test", 0, 8, 8),
            Err(RendererError::InvalidTextureExtent {
                width: 0,
                height: 8,
                max_dimension: 8,
                ..
            })
        ));
        assert!(matches!(
            validate_texture_extent("test", 8, 9, 8),
            Err(RendererError::InvalidTextureExtent {
                width: 8,
                height: 9,
                max_dimension: 8,
                ..
            })
        ));
    }

    #[test]
    fn atomic_path_count_rejects_only_values_that_overflow_path_ids() {
        assert!(validate_atomic_path_count(MAX_ATOMIC_PATHS).is_ok());
        assert!(matches!(
            validate_atomic_path_count(MAX_ATOMIC_PATHS + 1),
            Err(RendererError::Unsupported(
                "atomic runs exceed the C++ logical-flush path budget"
            ))
        ));
    }

    #[test]
    fn oversized_atlas_layout_returns_renderer_error_before_wgpu() {
        let result = pack_atlas_for_device(1920, 2048, &[(1920, 100); 21]);

        assert!(matches!(result, Err(RendererError::AtlasPacking(_))));
    }

    #[test]
    fn logical_feather_atlas_uses_cpp_capacity_and_padding() {
        let exact =
            pack_logical_feather_atlas_for_cpp(16_384, &[(1020, 2044), (1020, 2044)]).unwrap();
        assert_eq!(exact.extent(), [2048, 2048]);

        let overflow =
            pack_logical_feather_atlas_for_cpp(16_384, &[(1020, 2044), (1020, 2044), (1, 1)]);
        assert!(matches!(overflow, Err(RendererError::AtlasPacking(_))));
    }

    #[test]
    fn logical_feather_atlas_enlarges_for_the_first_oversized_draw() {
        let layout = pack_logical_feather_atlas_for_cpp(16_384, &[(5000, 32), (100, 32)]).unwrap();

        assert_eq!(layout.extent(), [5000, 72]);
    }

    #[test]
    fn atlas_allocation_overallocates_like_cpp_resource_growth() {
        assert_eq!(atlas_physical_size([39, 39], 2048), [48, 48]);
        assert_eq!(atlas_physical_size([1, 2], 2048), [1, 2]);
        assert_eq!(atlas_physical_size([2048, 2048], 2048), [2048, 2048]);
    }

    #[test]
    fn matrix_composition_matches_renderer_post_concat() {
        let translated = Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 20.0]);
        let scaled = Mat2D([2.0, 0.0, 0.0, 3.0, 0.0, 0.0]);
        let result = multiply(translated, scaled);
        assert_eq!(result.0, [2.0, 0.0, 0.0, 3.0, 10.0, 20.0]);
    }

    #[test]
    fn frame_clear_color_uses_cpp_integer_premultiplication() {
        assert_eq!(
            color(0x8040_2010),
            wgpu::Color {
                r: 32.0 / 255.0,
                g: 16.0 / 255.0,
                b: 8.0 / 255.0,
                a: 128.0 / 255.0,
            }
        );
    }

    #[test]
    fn radial_gradient_normalization_scales_the_radius_to_the_last_stop() {
        let gradient = normalize_gradient(
            &WgpuShader::Radial {
                center: (3.0, 4.0),
                radius: 20.0,
                colors: vec![0x80ff0000, 0x400000ff],
                stops: vec![0.0, 0.5],
            },
            0.5,
        )
        .unwrap();

        assert_eq!(gradient.paint_type, gpu::PaintType::RadialGradient);
        assert_eq!(gradient.coeffs, [3.0, 4.0, 10.0]);
        assert_eq!(gradient.stops, [0.0, 1.0]);
        assert_eq!(gradient.colors, [0x40ff0000, 0x200000ff]);
    }

    #[test]
    fn gradient_batch_packs_simple_ramps_before_complex_rows() {
        let draw = |shader| SolidDraw {
            path: WgpuPath {
                valid: true,
                raw_path: RawPath::new(),
                fill_rule: FillRule::NonZero,
            },
            paint: WgpuPaint {
                shader: Some(shader),
                ..Default::default()
            },
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };
        let batch = prepare_gradient_batch(&[
            draw(WgpuShader::Linear {
                start: (0.0, 0.0),
                end: (10.0, 0.0),
                colors: vec![0xff000000, 0xffffffff],
                stops: vec![0.0, 1.0],
            }),
            draw(WgpuShader::Radial {
                center: (0.0, 0.0),
                radius: 10.0,
                colors: vec![0xffff0000, 0xff00ff00, 0xff0000ff],
                stops: vec![0.0, 0.5, 1.0],
            }),
        ]);

        assert_eq!(batch.height, 2);
        assert_eq!(batch.spans.len(), 3);
        assert_eq!(batch.spans[0].y_with_flags & 0x1fff_ffff, 0);
        assert_eq!(batch.spans[1].y_with_flags & 0x1fff_ffff, 1);
        assert_eq!(batch.draws[0].unwrap().texture_y, 0.25);
        assert_eq!(batch.draws[1].unwrap().texture_y, 0.75);
    }

    #[test]
    fn decodes_profiled_corpus_jpeg_to_opaque_rgba() {
        let stream = include_str!(
            "../../../fixtures/renderer/streams/riv/clipping_and_draw_order.rive-stream"
        );
        let encoded = stream
            .lines()
            .find_map(|line| line.strip_prefix("decodeImage "))
            .and_then(|line| line.split_once("data="))
            .map(|(_, hex)| {
                hex.as_bytes()
                    .chunks_exact(2)
                    .map(|pair| {
                        let pair = std::str::from_utf8(pair).unwrap();
                        u8::from_str_radix(pair, 16).unwrap()
                    })
                    .collect::<Vec<_>>()
            })
            .expect("fixture must contain an encoded image");
        let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(&encoded));
        decoder.read_info().unwrap();
        assert!(decoder.icc_profile().is_some());

        let (width, height, rgba) = decode_image_rgba(&encoded).expect("JPEG must decode");
        assert_eq!((width, height), (278, 278));
        assert_eq!(rgba.len(), 278 * 278 * 4);
        assert!(rgba.chunks_exact(4).all(|pixel| pixel[3] == 255));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_jpeg_decode_fails_closed_when_imageio_rejects_input() {
        let encoded = [
            0xff, 0xd8, // SOI
            0xff, 0xc4, 0x00, 0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // DHT
            0xff, 0xc3, 0x00, 0x0b, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x13,
            0x00, // SOF3, invalid 1x3 sampling for a single grayscale component
            0xff, 0xda, 0x00, 0x08, 0x01, 0x01, 0x00, 0x01, 0x00, 0x00, // SOS
            0x7f, 0xff, 0xd9, // zero difference and EOI
        ];

        assert!(decode_macos_image_rgba(&encoded).is_none());
        let mut portable = jpeg_decoder::Decoder::new(Cursor::new(encoded));
        assert!(portable.decode().is_ok());
        assert!(decode_image_rgba(&encoded).is_none());
    }

    #[test]
    fn rejects_unknown_encoded_image_format() {
        assert!(decode_image_rgba(b"not an image").is_none());
        let mut factory = WgpuFactory::new_with_mode(16, 16, RenderMode::ClockwiseAtomic).unwrap();
        assert_eq!(
            factory.decode_image(b"not an image").err(),
            Some(ImageDecodeError)
        );
    }

    #[test]
    fn embedded_png_profile_transforms_rgb_and_preserves_alpha() {
        let stream = nuxie_render_stream::RenderStream::parse(include_str!(
            "../../../fixtures/renderer/streams/gm/image_aa_border.rive-stream"
        ))
        .unwrap();
        let encoded = stream
            .resources
            .iter()
            .find_map(|resource| match resource {
                nuxie_render_stream::Resource::Image { data, .. } => Some(data.as_slice()),
                _ => None,
            })
            .unwrap();
        let reader = png::Decoder::new(Cursor::new(encoded)).read_info().unwrap();
        let profile = reader.info().icc_profile.as_ref().unwrap();
        let mut pixel = [64, 128, 192, 77];
        let original = pixel;

        convert_icc_rgba_to_srgb(&mut pixel, 1, profile);

        assert_ne!(pixel[..3], original[..3]);
        assert_eq!(pixel[3], original[3]);
    }

    #[test]
    fn image_decode_uses_the_adapter_texture_dimension_limit() {
        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut encoded, 2080, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&vec![255; 2080 * 4])
                .unwrap();
        }
        let mut factory = WgpuFactory::new_with_mode(16, 16, RenderMode::ClockwiseAtomic).unwrap();

        let image = factory.decode_image(&encoded).expect("image decodes");

        assert_eq!(image.width(), 2080);
        assert_eq!(image.height(), 1);
        assert!(image
            .as_any()
            .downcast_ref::<WgpuImage>()
            .unwrap()
            .texture
            .is_some());
    }

    #[test]
    fn image_decode_rejects_dimensions_above_the_adapter_limit_before_wgpu() {
        let mut factory = WgpuFactory::new_with_mode(16, 16, RenderMode::ClockwiseAtomic).unwrap();
        let width = factory.context.device.limits().max_texture_dimension_2d + 1;
        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut encoded, width, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&vec![255; width as usize * 4])
                .unwrap();
        }

        assert_eq!(factory.decode_image(&encoded).err(), Some(ImageDecodeError));
    }

    #[test]
    fn render_buffer_unmap_snapshots_submitted_contents() {
        let mut factory = WgpuFactory::new_with_mode(4, 4, RenderMode::ClockwiseAtomic).unwrap();
        let mut buffer =
            factory.make_render_buffer(RenderBufferType::Vertex, RenderBufferFlags::None, 8);
        buffer.map_mut().copy_from_slice(&[1; 8]);
        buffer.unmap();
        let first = Arc::clone(
            wgpu_buffer(buffer.as_ref())
                .unwrap()
                .submitted
                .as_ref()
                .unwrap(),
        );

        buffer.map_mut().copy_from_slice(&[2; 8]);
        buffer.unmap();
        let second = Arc::clone(
            wgpu_buffer(buffer.as_ref())
                .unwrap()
                .submitted
                .as_ref()
                .unwrap(),
        );

        assert!(!Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn image_texture_lod_matches_cpp_constant_lod_contract() {
        assert_eq!(
            image_texture_lod(Mat2D([0.25, 0.0, 0.0, 0.125, 0.0, 0.0]), 4, 8),
            -0.5
        );
        assert_eq!(
            image_texture_lod(Mat2D([1.0, 0.0, 0.0, 0.5, 0.0, 0.0]), 4, 8),
            1.5
        );
    }

    #[test]
    fn msaa_image_rect_matches_atomic_image_sampling() {
        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut encoded, 2, 2);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&[
                    255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
                ])
                .unwrap();
        }
        let render = |mode| {
            let mut factory = WgpuFactory::new_with_mode(16, 16, mode).unwrap();
            let image = factory.decode_image(&encoded).expect("image decodes");
            let mut frame = factory.begin_frame(0xff00_0000);
            frame.transform(Mat2D([4.0, 0.0, 0.0, 4.0, 2.0, 2.0]));
            frame.draw_image(
                Some(image.as_ref()),
                ImageSampler {
                    wrap_x: nuxie_render_api::ImageWrap::Clamp,
                    wrap_y: nuxie_render_api::ImageWrap::Clamp,
                    filter: nuxie_render_api::ImageFilter::Nearest,
                },
                BlendMode::SrcOver,
                1.0,
            );
            frame.finish().unwrap()
        };

        let atomic = render(RenderMode::ClockwiseAtomic);
        let msaa = render(RenderMode::Msaa);
        assert_eq!(msaa, atomic);
        let pixel = |x: usize, y: usize| &msaa[(y * 16 + x) * 4..(y * 16 + x + 1) * 4];
        assert_eq!(pixel(3, 3), [255, 0, 0, 255]);
        assert_eq!(pixel(8, 3), [0, 255, 0, 255]);
        assert_eq!(pixel(3, 8), [0, 0, 255, 255]);
        assert_eq!(pixel(8, 8), [255, 255, 255, 255]);
        assert_eq!(pixel(11, 11), [0, 0, 0, 255]);
    }

    #[test]
    fn msaa_path_clip_applies_to_image_rect() {
        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut encoded, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&[255, 0, 0, 255])
                .unwrap();
        }
        let mut factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let image = factory.decode_image(&encoded).expect("image decodes");
        let mut raw_clip = RawPath::new();
        raw_clip.move_to(32.0, 8.0);
        raw_clip.line_to(56.0, 56.0);
        raw_clip.line_to(8.0, 56.0);
        raw_clip.close();
        let clip = WgpuPath {
            valid: true,
            raw_path: raw_clip,
            fill_rule: FillRule::NonZero,
        };
        let mut frame = factory.begin_frame(0);
        frame.clip_path(&clip);
        frame.transform(Mat2D([64.0, 0.0, 0.0, 64.0, 0.0, 0.0]));
        frame.draw_image(
            Some(image.as_ref()),
            ImageSampler::default(),
            BlendMode::SrcOver,
            1.0,
        );
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(8, 32), [0; 4]);
        assert_eq!(pixel(32, 32), [255, 0, 0, 255]);
        assert_eq!(pixel(32, 4), [0; 4]);
        assert_eq!(pixel(32, 48), [255, 0, 0, 255]);
    }

    #[test]
    fn image_mesh_draws_indexed_position_and_uv_buffers_in_both_modes() {
        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut encoded, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&[255, 0, 0, 255])
                .unwrap();
        }

        for mode in [RenderMode::ClockwiseAtomic, RenderMode::Msaa] {
            let mut factory = WgpuFactory::new_with_mode(16, 16, mode).unwrap();
            let image = factory.decode_image(&encoded).expect("image decodes");
            let mut vertices = factory.make_render_buffer(
                RenderBufferType::Vertex,
                RenderBufferFlags::MappedOnceAtInitialization,
                24,
            );
            vertices.map_mut().copy_from_slice(bytemuck::cast_slice(&[
                [2.0f32, 2.0],
                [14.0, 2.0],
                [2.0, 14.0],
            ]));
            vertices.unmap();
            let mut uvs = factory.make_render_buffer(
                RenderBufferType::Vertex,
                RenderBufferFlags::MappedOnceAtInitialization,
                24,
            );
            uvs.map_mut().copy_from_slice(bytemuck::cast_slice(&[
                [0.0f32, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
            ]));
            uvs.unmap();
            let mut indices = factory.make_render_buffer(
                RenderBufferType::Index,
                RenderBufferFlags::MappedOnceAtInitialization,
                6,
            );
            indices
                .map_mut()
                .copy_from_slice(bytemuck::cast_slice(&[0u16, 1, 2]));
            indices.unmap();

            let mut frame = factory.begin_frame(0xff00_0000);
            frame.draw_image_mesh(
                Some(image.as_ref()),
                ImageSampler::default(),
                Some(vertices.as_ref()),
                Some(uvs.as_ref()),
                Some(indices.as_ref()),
                3,
                3,
                BlendMode::SrcOver,
                1.0,
            );
            let pixels = frame.finish().unwrap();
            let pixel = |x: usize, y: usize| &pixels[(y * 16 + x) * 4..(y * 16 + x + 1) * 4];
            assert_eq!(pixel(4, 4), [255, 0, 0, 255], "{mode:?}");
            assert_eq!(pixel(15, 15), [0, 0, 0, 255], "{mode:?}");

            for (blend_mode, expected) in [
                (BlendMode::Screen, [255, 255, 0, 255]),
                (BlendMode::Darken, [0, 0, 0, 255]),
                (BlendMode::Exclusion, [255, 255, 0, 255]),
                (BlendMode::Luminosity, [0, 130, 0, 255]),
            ] {
                let mut advanced_blend = factory.begin_frame(0xff00_ff00);
                advanced_blend.draw_image_mesh(
                    Some(image.as_ref()),
                    ImageSampler::default(),
                    Some(vertices.as_ref()),
                    Some(uvs.as_ref()),
                    Some(indices.as_ref()),
                    3,
                    3,
                    blend_mode,
                    1.0,
                );
                let pixels = advanced_blend.finish().unwrap();
                let pixel = |x: usize, y: usize| &pixels[(y * 16 + x) * 4..(y * 16 + x + 1) * 4];
                assert_eq!(pixel(4, 4), expected, "{mode:?} {blend_mode:?}");
                assert_eq!(pixel(15, 15), [0, 255, 0, 255], "{mode:?} {blend_mode:?}");
            }
        }
    }

    #[test]
    fn msaa_path_clip_applies_to_image_mesh() {
        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut encoded, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&[255, 0, 0, 255])
                .unwrap();
        }

        let mut factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let image = factory.decode_image(&encoded).expect("image decodes");
        let mut vertices = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::MappedOnceAtInitialization,
            24,
        );
        vertices.map_mut().copy_from_slice(bytemuck::cast_slice(&[
            [0.0f32, 0.0],
            [64.0, 0.0],
            [0.0, 64.0],
        ]));
        vertices.unmap();
        let mut uvs = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::MappedOnceAtInitialization,
            24,
        );
        uvs.map_mut().copy_from_slice(bytemuck::cast_slice(&[
            [0.0f32, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
        ]));
        uvs.unmap();
        let mut indices = factory.make_render_buffer(
            RenderBufferType::Index,
            RenderBufferFlags::MappedOnceAtInitialization,
            6,
        );
        indices
            .map_mut()
            .copy_from_slice(bytemuck::cast_slice(&[0u16, 1, 2]));
        indices.unmap();

        let mut raw_clip = RawPath::new();
        raw_clip.move_to(32.0, 8.0);
        raw_clip.line_to(56.0, 56.0);
        raw_clip.line_to(8.0, 56.0);
        raw_clip.close();
        let clip = WgpuPath {
            valid: true,
            raw_path: raw_clip,
            fill_rule: FillRule::NonZero,
        };
        let mut frame = factory.begin_frame(0);
        frame.clip_path(&clip);
        frame.draw_image_mesh(
            Some(image.as_ref()),
            ImageSampler::default(),
            Some(vertices.as_ref()),
            Some(uvs.as_ref()),
            Some(indices.as_ref()),
            3,
            3,
            BlendMode::SrcOver,
            1.0,
        );
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(8, 32), [0; 4]);
        assert_eq!(pixel(24, 32), [255, 0, 0, 255]);
        assert_eq!(pixel(32, 4), [0; 4]);
        assert_eq!(pixel(20, 40), [255, 0, 0, 255]);
    }

    #[test]
    fn msaa_image_mesh_resets_stencil_between_diverging_rectangular_clips() {
        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut encoded, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&[255, 0, 0, 255])
                .unwrap();
        }

        let mut factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let image = factory.decode_image(&encoded).expect("image decodes");
        let mut vertices = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::MappedOnceAtInitialization,
            24,
        );
        vertices.map_mut().copy_from_slice(bytemuck::cast_slice(&[
            [0.0f32, 0.0],
            [64.0, 0.0],
            [0.0, 64.0],
        ]));
        vertices.unmap();
        let mut uvs = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::MappedOnceAtInitialization,
            24,
        );
        uvs.map_mut().copy_from_slice(bytemuck::cast_slice(&[
            [0.0f32, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
        ]));
        uvs.unmap();
        let mut indices = factory.make_render_buffer(
            RenderBufferType::Index,
            RenderBufferFlags::MappedOnceAtInitialization,
            6,
        );
        indices
            .map_mut()
            .copy_from_slice(bytemuck::cast_slice(&[0u16, 1, 2]));
        indices.unmap();

        let left = rect_path([0.0, 0.0, 32.0, 64.0], FillRule::NonZero);
        let right = rect_path([32.0, 0.0, 64.0, 64.0], FillRule::NonZero);
        let mut frame = factory.begin_frame(0);
        let draw_mesh = |frame: &mut WgpuFrame| {
            frame.draw_image_mesh(
                Some(image.as_ref()),
                ImageSampler::default(),
                Some(vertices.as_ref()),
                Some(uvs.as_ref()),
                Some(indices.as_ref()),
                3,
                3,
                BlendMode::SrcOver,
                1.0,
            );
        };

        frame.save();
        frame.clip_path(&left);
        draw_mesh(&mut frame);
        frame.restore();
        frame.clip_path(&right);
        draw_mesh(&mut frame);

        assert_eq!(frame.draws.len(), 5);
        assert!(matches!(
            frame.draws[0].role,
            DrawRole::ClipUpdate {
                replacement_id: 1,
                parent_id: 0
            }
        ));
        assert!(matches!(
            frame.draws[1].role,
            DrawRole::Content { clip_id: 1 }
        ));
        assert!(matches!(
            frame.draws[2].role,
            DrawRole::ClipReset {
                action: MsaaClipResetAction::ClearPrevious,
                ..
            }
        ));
        assert!(matches!(
            frame.draws[3].role,
            DrawRole::ClipUpdate {
                replacement_id: 2,
                parent_id: 0
            }
        ));
        assert!(matches!(
            frame.draws[4].role,
            DrawRole::Content { clip_id: 2 }
        ));
    }

    #[test]
    fn atomic_path_uses_shader_advanced_blending() {
        let mut factory = WgpuFactory::new_with_mode(16, 16, RenderMode::ClockwiseAtomic).unwrap();
        let mut path = RawPath::new();
        path.move_to(2.0, 2.0);
        path.line_to(14.0, 2.0);
        path.line_to(2.0, 14.0);
        path.close();
        let path = factory.make_render_path(path, FillRule::NonZero);
        let mut paint = factory.make_render_paint();
        paint.color(0xffff_0000);
        paint.blend_mode(BlendMode::Screen);

        let mut frame = factory.begin_frame(0xff00_ff00);
        frame.draw_path(path.as_ref(), paint.as_ref());
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 16 + x) * 4..(y * 16 + x + 1) * 4];
        assert_eq!(pixel(4, 4), [255, 255, 0, 255]);
        assert_eq!(pixel(15, 15), [0, 255, 0, 255]);
    }

    #[test]
    fn complex_fill_topology_detects_crossings_and_compound_contours() {
        let mut concave = RawPath::new();
        concave.move_to(20.0, 20.0);
        concave.line_to(80.0, 20.0);
        concave.line_to(30.0, 30.0);
        concave.line_to(20.0, 80.0);
        assert!(!path_has_complex_fill_topology(&concave));

        let mut bowtie = RawPath::new();
        bowtie.move_to(20.0, 20.0);
        bowtie.line_to(80.0, 80.0);
        bowtie.line_to(80.0, 20.0);
        bowtie.line_to(20.0, 80.0);
        assert!(path_has_complex_fill_topology(&bowtie));

        let mut closed_cubic = RawPath::new();
        append_oval(&mut closed_cubic, [0.0, 0.0, 100.0, 100.0]);
        assert!(!path_has_complex_fill_topology(&closed_cubic));

        let mut compound = RawPath::new();
        compound.move_to(0.0, 0.0);
        compound.line_to(10.0, 0.0);
        compound.line_to(0.0, 10.0);
        compound.move_to(20.0, 20.0);
        compound.line_to(30.0, 20.0);
        compound.line_to(20.0, 30.0);
        assert!(path_has_complex_fill_topology(&compound));
    }

    #[test]
    fn generic_atomic_large_interior_draw_is_repeatable() {
        let mut factory =
            WgpuFactory::new_with_mode(640, 640, RenderMode::ClockwiseAtomic).unwrap();
        let mut raw_path = RawPath::new();
        append_oval(&mut raw_path, [20.0, 20.0, 620.0, 620.0]);
        let path = factory.make_render_path(raw_path, FillRule::NonZero);
        let paint = WgpuPaint {
            color: 0xff39_529f,
            ..WgpuPaint::default()
        };

        let mut expected: Option<Vec<u8>> = None;
        for _ in 0..5 {
            let mut frame = factory.begin_frame(0xff31_3131);
            frame.draw_path(path.as_ref(), &paint);
            let pixels = frame.finish().unwrap();
            if let Some(expected) = &expected {
                if &pixels != expected {
                    let differing_pixels = pixels
                        .chunks_exact(4)
                        .zip(expected.chunks_exact(4))
                        .filter(|(actual, expected)| actual != expected)
                        .count();
                    panic!("repeat render differs at {differing_pixels} pixels");
                }
            } else {
                expected = Some(pixels);
            }
        }
    }

    #[test]
    fn midpoint_tessellation_relocates_into_shared_texture_shelves() {
        let path = rect_path([0.0, 0.0, 10.0, 10.0], FillRule::NonZero);
        let mut tessellation =
            draw::build_fill_tessellation(&path.raw_path, Mat2D::IDENTITY).unwrap();
        tessellation.make_double_sided();
        let width = midpoint_tessellation_single_row_width(&tessellation.spans).unwrap();
        let base_instance = tessellation.base_instance;
        let vertex_index0 = tessellation.contours[0].vertex_index0;

        relocate_midpoint_tessellation(
            &mut tessellation.spans,
            &mut tessellation.base_instance,
            &mut tessellation.contours,
            width,
            2,
        );

        let logical_offset = 2 * gpu::TESS_TEXTURE_WIDTH as u32 + width;
        assert_eq!(width % gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32, 0);
        assert_eq!(
            tessellation.base_instance,
            base_instance + logical_offset / gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32
        );
        assert_eq!(
            tessellation.contours[0].vertex_index0,
            vertex_index0 + logical_offset
        );
        assert!(tessellation.spans.iter().all(|span| span.y == 2.0));
    }

    #[test]
    fn stroke_midpoint_tessellation_relocates_for_shared_atomic_texture() {
        let path = rect_path([0.0, 0.0, 10.0, 10.0], FillRule::NonZero);
        let mut tessellation = draw::build_stroke_tessellation(
            &path.raw_path,
            Mat2D::IDENTITY,
            2.0,
            StrokeJoin::Bevel,
            StrokeCap::Butt,
        )
        .unwrap();
        let base_instance = tessellation.base_instance;
        let vertex_indices = tessellation
            .contours
            .iter()
            .map(|contour| contour.vertex_index0)
            .collect::<Vec<_>>();
        let x = gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let y = 2;

        relocate_midpoint_tessellation(
            &mut tessellation.spans,
            &mut tessellation.base_instance,
            &mut tessellation.contours,
            x,
            y,
        );

        let logical_offset = y * gpu::TESS_TEXTURE_WIDTH as u32 + x;
        assert_eq!(
            tessellation.base_instance,
            base_instance + logical_offset / gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32
        );
        assert!(tessellation
            .contours
            .iter()
            .zip(vertex_indices)
            .all(|(contour, original)| contour.vertex_index0 == original + logical_offset));
    }

    #[test]
    fn compact_midpoint_tessellation_wraps_across_texture_rows() {
        let mut path = RawPath::new();
        append_oval(&mut path, [0.0, 0.0, 100.0, 100.0]);
        let mut tessellation = draw::build_fill_tessellation(&path, Mat2D::IDENTITY).unwrap();
        tessellation
            .spans
            .retain(|span| span.contour_id_with_flags & gpu::CONTOUR_ID_MASK != 0);
        let source_span_count = tessellation.spans.len();
        let source_vertex_index = tessellation.contours[0].vertex_index0;
        let instances_per_row =
            gpu::TESS_TEXTURE_WIDTH as u32 / gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let next_base_instance = instances_per_row - 1;

        relocate_midpoint_tessellation_logically(
            &mut tessellation.spans,
            &mut tessellation.base_instance,
            &mut tessellation.contours,
            next_base_instance,
        );

        let relocation = (next_base_instance - 1) * gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        assert_eq!(tessellation.base_instance, next_base_instance);
        assert_eq!(
            tessellation.contours[0].vertex_index0,
            source_vertex_index + relocation
        );
        assert!(tessellation.spans.len() > source_span_count);
        assert!(tessellation.spans.iter().any(|span| span.y == 1.0));
        assert!(tessellation.spans.iter().any(|span| span.x_range().0 < 0));
    }

    #[test]
    fn compact_double_sided_tessellation_wraps_reflected_spans_across_rows() {
        let segment_span = gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as i32;
        let vertex_count = segment_span * 2;
        let mut spans = vec![gpu::TessVertexSpan::new(
            [[0.0; 2]; 4],
            [0.0; 2],
            0.0,
            0,
            vertex_count,
            0.0,
            vertex_count,
            0,
            1,
            1,
            0,
            1,
        )];
        let mut base_instance = 0;
        let mut contours = [];
        let instances_per_row =
            gpu::TESS_TEXTURE_WIDTH as u32 / gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;

        relocate_midpoint_tessellation_logically(
            &mut spans,
            &mut base_instance,
            &mut contours,
            instances_per_row - 1,
        );

        let texture_width = gpu::TESS_TEXTURE_WIDTH;
        assert_eq!(base_instance, instances_per_row - 1);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].y, 0.0);
        assert_eq!(
            spans[0].x_range(),
            (texture_width - segment_span, texture_width + segment_span)
        );
        assert_eq!(spans[0].reflection_y, 1.0);
        assert_eq!(spans[0].reflection_x0_x1 as i16 as i32, segment_span);
        assert_eq!(
            (spans[0].reflection_x0_x1 >> 16) as i16 as i32,
            -segment_span
        );
        assert_eq!(spans[1].y, 1.0);
        assert_eq!(spans[1].x_range(), (-segment_span, segment_span));
        assert_eq!(spans[1].reflection_y, 0.0);
        assert_eq!(
            spans[1].reflection_x0_x1 as i16 as i32,
            texture_width + segment_span
        );
        assert_eq!(
            (spans[1].reflection_x0_x1 >> 16) as i16 as i32,
            texture_width - segment_span
        );
    }

    #[test]
    fn logical_relocation_rebuilds_previously_wrapped_reflected_spans_once() {
        let texture_width = gpu::TESS_TEXTURE_WIDTH;
        let segment_span = gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let source_base_instance = texture_width as u32 / segment_span - 1;
        let first = gpu::TessVertexSpan::new(
            [[1.0; 2]; 4],
            [2.0; 2],
            0.0,
            texture_width - segment_span as i32,
            texture_width + 9,
            0.0,
            16,
            -1,
            1,
            1,
            0,
            1,
        );
        let mut second = first;
        second.y = 1.0;
        second.set_ranges(
            -(segment_span as i32),
            9,
            texture_width + 16,
            texture_width - 1,
            u32::MAX as f32,
        );
        let mut spans = vec![first, second];
        let mut base_instance = source_base_instance;
        let source_vertex_index = source_base_instance * segment_span;
        let mut contours = [gpu::ContourData::new([0.0, 0.0], 1, source_vertex_index)];

        relocate_midpoint_tessellation_logically(
            &mut spans,
            &mut base_instance,
            &mut contours,
            source_base_instance + 1,
        );

        assert_eq!(base_instance, source_base_instance + 1);
        assert_eq!(
            contours[0].vertex_index0,
            source_vertex_index + segment_span
        );
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].y, 1.0);
        assert_eq!(spans[0].x_range(), (0, segment_span as i32 + 9));
        assert_eq!(spans[0].reflection_y, 0.0);
        assert_eq!(spans[0].reflection_x0_x1 as i16 as i32, 24);
        assert_eq!((spans[0].reflection_x0_x1 >> 16) as i16 as i32, 7);
    }

    #[test]
    fn logical_relocation_preserves_unsigned_reflection_row_wrap() {
        let texture_width = gpu::TESS_TEXTURE_WIDTH;
        let segment_span = gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let source_base_instance = texture_width as u32 / segment_span - 1;
        let first = gpu::TessVertexSpan::new(
            [[1.0; 2]; 4],
            [2.0; 2],
            0.0,
            texture_width - segment_span as i32,
            texture_width + 9,
            0.0,
            16,
            -1,
            1,
            1,
            0,
            1,
        );
        let mut second = first;
        second.y = 1.0;
        second.set_ranges(
            -(segment_span as i32),
            9,
            texture_width + 16,
            texture_width - 1,
            u32::MAX as f32,
        );
        let expected = [first, second];
        let mut spans = expected.to_vec();
        let mut base_instance = source_base_instance;
        let source_vertex_index = source_base_instance * segment_span;
        let mut contours = [gpu::ContourData::new([0.0, 0.0], 1, source_vertex_index)];

        relocate_midpoint_tessellation_logically(
            &mut spans,
            &mut base_instance,
            &mut contours,
            source_base_instance,
        );

        assert_eq!(base_instance, source_base_instance);
        assert_eq!(contours[0].vertex_index0, source_vertex_index);
        assert_eq!(spans.len(), expected.len());
        for (actual, expected) in spans.iter().zip(expected.iter()) {
            assert_eq!(bytemuck::bytes_of(actual), bytemuck::bytes_of(expected));
        }
    }

    #[test]
    fn logical_relocation_keeps_distinct_reflection_mappings() {
        let segment_span = gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let first = gpu::TessVertexSpan::new(
            [[1.0; 2]; 4],
            [2.0; 2],
            0.0,
            segment_span as i32,
            segment_span as i32 * 2,
            0.0,
            segment_span as i32 * 3,
            segment_span as i32 * 2,
            1,
            1,
            0,
            1,
        );
        let mut second = first;
        second.set_ranges(
            segment_span as i32,
            segment_span as i32 * 2,
            segment_span as i32 * 4,
            segment_span as i32 * 3,
            0.0,
        );
        let mut spans = vec![first, second];
        let mut base_instance = 1;
        let mut contours = [gpu::ContourData::new([0.0, 0.0], 1, segment_span)];

        relocate_midpoint_tessellation_logically(&mut spans, &mut base_instance, &mut contours, 2);

        assert_eq!(spans.len(), 2);
        assert_eq!(
            spans[0].x_range(),
            (segment_span as i32 * 2, segment_span as i32 * 3)
        );
        assert_eq!(
            spans[0].reflection_x0_x1 as i16 as i32,
            segment_span as i32 * 4
        );
        assert_eq!(
            spans[1].x_range(),
            (segment_span as i32 * 2, segment_span as i32 * 3)
        );
        assert_eq!(
            spans[1].reflection_x0_x1 as i16 as i32,
            segment_span as i32 * 5
        );
        assert_eq!(base_instance, 2);
        assert_eq!(contours[0].vertex_index0, segment_span * 2);
    }

    #[test]
    fn outer_tessellations_relocate_into_contiguous_batch_ranges() {
        let mut path = RawPath::new();
        append_oval(&mut path, [0.0, 0.0, 100.0, 100.0]);
        let first =
            draw::build_interior_tessellation(&path, Mat2D::IDENTITY, FillRule::NonZero, false)
                .unwrap();
        let mut second =
            draw::build_interior_tessellation(&path, Mat2D::IDENTITY, FillRule::NonZero, false)
                .unwrap();
        let original_vertex_indices = second
            .contours
            .iter()
            .map(|contour| contour.vertex_index0)
            .collect::<Vec<_>>();
        let segment_span = gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32;
        let relocation = first.instance_count * segment_span;

        relocate_tessellation(
            &mut second.spans,
            &mut second.base_instance,
            &mut second.contours,
            relocation,
            0,
            segment_span,
        );

        assert_eq!(
            second.base_instance,
            first.base_instance + first.instance_count
        );
        assert!(second
            .contours
            .iter()
            .zip(original_vertex_indices)
            .all(|(contour, original)| contour.vertex_index0 == original + relocation));
    }

    #[test]
    fn midpoint_tessellations_share_flush_wide_path_and_contour_ids() {
        let path = rect_path([0.0, 0.0, 10.0, 10.0], FillRule::NonZero);
        let mut first = draw::build_fill_tessellation(&path.raw_path, Mat2D::IDENTITY).unwrap();
        let mut second = draw::build_fill_tessellation(&path.raw_path, Mat2D::IDENTITY).unwrap();
        let second_x = midpoint_tessellation_single_row_width(&first.spans).unwrap();
        let mut spans = Vec::new();
        let mut contours = Vec::new();

        append_midpoint_tessellation_to_flush(&mut first, 1, 0, 0, &mut spans, &mut contours);
        let second_span_start = spans.len();
        append_midpoint_tessellation_to_flush(
            &mut second,
            2,
            second_x,
            0,
            &mut spans,
            &mut contours,
        );

        assert_eq!(contours.len(), 2);
        assert_eq!(contours[0].path_id, 1);
        assert_eq!(contours[1].path_id, 2);
        assert_eq!(
            second.base_instance,
            1 + second_x / gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32
        );
        assert!(spans[..second_span_start].iter().all(|span| {
            let id = span.contour_id_with_flags & gpu::CONTOUR_ID_MASK;
            id == 0 || id == 1
        }));
        assert!(spans[second_span_start..].iter().all(|span| {
            let id = span.contour_id_with_flags & gpu::CONTOUR_ID_MASK;
            id == 0 || id == 2
        }));
    }

    #[test]
    fn flush_wide_tessellation_preserves_sparse_empty_contour_ids() {
        let mut path = RawPath::new();
        path.move_to(40.0, 40.0);
        path.move_to(80.0, 40.0);
        path.close();
        path.move_to(120.0, 40.0);
        path.line_to(120.0, 40.0);
        let mut tessellation = draw::build_stroke_tessellation(
            &path,
            Mat2D::IDENTITY,
            21.0,
            StrokeJoin::Miter,
            StrokeCap::Butt,
        )
        .unwrap();
        assert_eq!(tessellation.contours.len(), 1);
        assert!(tessellation
            .spans
            .iter()
            .any(|span| { span.contour_id_with_flags & gpu::CONTOUR_ID_MASK == 2 }));
        let mut spans = Vec::new();
        let mut contours = Vec::new();

        append_midpoint_tessellation_to_flush(
            &mut tessellation,
            1,
            0,
            0,
            &mut spans,
            &mut contours,
        );

        assert_eq!(contours.len(), 2);
        assert_eq!(contours[0].path_id, 0);
        assert_eq!(contours[1].path_id, 1);
        assert!(spans.iter().all(|span| {
            let id = span.contour_id_with_flags & gpu::CONTOUR_ID_MASK;
            id == 0 || id == 2
        }));
    }

    #[test]
    fn intersection_board_separates_overlapping_atomic_aa_bounds() {
        let make_draw = |bounds| SolidDraw {
            path: rect_path(bounds, FillRule::NonZero),
            paint: WgpuPaint::default(),
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };
        let draws = [
            make_draw([10.0, 10.0, 11.0, 11.0]),
            make_draw([11.0, 10.0, 12.0, 11.0]),
            make_draw([30.0, 30.0, 31.0, 31.0]),
        ];

        let groups = disjoint_atomic_draw_groups(&draws, 64, 64);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 2);
        assert_eq!(groups[1].len(), 1);
    }

    #[test]
    fn intersection_board_rolls_over_before_group_index_overflow() {
        let draw = SolidDraw {
            path: rect_path([10.0, 10.0, 11.0, 11.0], FillRule::NonZero),
            paint: WgpuPaint::default(),
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };
        let draws = vec![draw; 3];

        let groups = disjoint_atomic_draw_groups_with_limit(&draws, 64, 64, 2);

        assert_eq!(groups.len(), draws.len());
        assert!(groups.iter().all(|group| group.len() == 1));
    }

    #[test]
    fn independent_atomic_groups_share_an_encoder_until_the_draw_budget() {
        assert!(!independent_atomic_group_requires_submit(0, 2_048));
        assert!(!independent_atomic_group_requires_submit(1_023, 1));
        assert!(independent_atomic_group_requires_submit(1_024, 1));
        assert!(independent_atomic_group_requires_submit(1, 1_024));
    }

    #[test]
    fn disjoint_atomic_groups_split_before_atomic_path_id_overflow() {
        let draws = (0..5)
            .map(|index| SolidDraw {
                path: rect_path(
                    [index as f32 * 10.0, 0.0, index as f32 * 10.0 + 2.0, 2.0],
                    FillRule::NonZero,
                ),
                paint: WgpuPaint {
                    color: index,
                    ..WgpuPaint::default()
                },
                state: DrawState::default(),
                role: DrawRole::Content { clip_id: 0 },
                image: None,
            })
            .collect::<Vec<_>>();

        let groups = disjoint_atomic_draw_groups_with_limits(&draws, 64, 64, 32, 2);
        let colors = groups
            .iter()
            .map(|group| {
                group
                    .iter()
                    .map(|draw| draw.paint.color)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        assert_eq!(colors, vec![vec![0, 1], vec![2, 3], vec![4]]);
    }

    #[test]
    fn msaa_intersection_board_groups_disjoint_draws_before_overlapping_draws() {
        let make_draw = |bounds, color| SolidDraw {
            path: rect_path(bounds, FillRule::NonZero),
            paint: WgpuPaint {
                color: 0xff00_0000u32 | color,
                ..WgpuPaint::default()
            },
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };
        let draws = [
            make_draw([10.0, 10.0, 20.0, 20.0], 1),
            make_draw([15.0, 10.0, 25.0, 20.0], 2),
            make_draw([40.0, 40.0, 50.0, 50.0], 3),
        ];

        let groups = disjoint_msaa_draw_groups(&draws, 64, 64);
        let colors = groups
            .iter()
            .map(|group| {
                group
                    .iter()
                    .map(|draw| draw.paint.color & 0xff)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        assert_eq!(colors, vec![vec![1, 3], vec![], vec![], vec![2]]);

        let (scheduled, z_indices, prepasses) = ordered_msaa_draws(&draws, 64, 64);
        assert_eq!(
            scheduled
                .iter()
                .map(|draw| draw.paint.color & 0xff)
                .collect::<Vec<_>>(),
            [1, 3, 2]
        );
        assert_eq!(z_indices, [1, 1, 4]);
        assert_eq!(prepasses, [false, false, false]);
    }

    #[test]
    fn msaa_scheduler_runs_opaque_prepasses_before_translucent_subpasses() {
        let make_draw = |color, blend_mode| SolidDraw {
            path: rect_path([10.0, 10.0, 20.0, 20.0], FillRule::NonZero),
            paint: WgpuPaint {
                color: 0xff00_0000u32 | color,
                blend_mode,
                ..WgpuPaint::default()
            },
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };
        let draws = [
            make_draw(1, BlendMode::Difference),
            make_draw(2, BlendMode::SrcOver),
            make_draw(3, BlendMode::SrcOver),
        ];

        let (scheduled, z_indices, prepasses) = ordered_msaa_draws(&draws, 64, 64);

        assert_eq!(
            scheduled
                .iter()
                .map(|draw| draw.paint.color & 0xff)
                .collect::<Vec<_>>(),
            [3, 2, 1]
        );
        assert_eq!(z_indices, [3, 2, 1]);
        assert_eq!(prepasses, [true, true, false]);
    }

    #[test]
    fn msaa_scheduler_promotes_non_overlapping_opaque_draws_before_destination_reads() {
        let make_draw = |rect, color, blend_mode| SolidDraw {
            path: rect_path(rect, FillRule::NonZero),
            paint: WgpuPaint {
                color: 0xff00_0000u32 | color,
                blend_mode,
                ..WgpuPaint::default()
            },
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };
        let draws = [
            make_draw([0.0, 0.0, 10.0, 10.0], 1, BlendMode::Difference),
            make_draw([20.0, 20.0, 30.0, 30.0], 2, BlendMode::SrcOver),
        ];

        let (scheduled, z_indices, prepasses) = ordered_msaa_draws(&draws, 64, 64);

        assert_eq!(
            scheduled
                .iter()
                .map(|draw| draw.paint.color & 0xff)
                .collect::<Vec<_>>(),
            [2, 1]
        );
        assert_eq!(z_indices, [1, 1]);
        assert_eq!(prepasses, [true, false]);
    }

    #[test]
    fn msaa_destination_copy_starts_at_subpass_head_in_its_logical_flush() {
        // The first two entries model an opaque negative-key prepass and a
        // clipped SrcOver subpass in one group. The latter is where C++ puts
        // the dstBlend barrier for the advanced draw at index 2. The final
        // pair reuses group 1 in a new logical flush and must not find index 1.
        let draw_groups = [1, 1, 1, 1, 1];
        let draw_prepasses = [true, false, false, true, false];
        let logical_flushes = [0, 0, 0, 1, 1];

        assert_eq!(
            msaa_destination_copy_head(&draw_groups, &draw_prepasses, &logical_flushes, 2),
            1
        );
        assert_eq!(
            msaa_destination_copy_head(&draw_groups, &draw_prepasses, &logical_flushes, 4),
            4
        );
    }

    #[test]
    fn msaa_large_draw_frame_submits_before_metal_resource_limit() {
        const FIRST_FAILING_DRAW_COUNT: usize = 2_044;
        const WIDTH: usize = 64;
        const HEIGHT: usize = 64;

        let factory =
            WgpuFactory::new_with_mode(WIDTH as u32, HEIGHT as u32, RenderMode::Msaa).unwrap();
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_0000);
        for index in 0..FIRST_FAILING_DRAW_COUNT {
            let x = (index % WIDTH) as f32;
            let y = (index / WIDTH) as f32;
            let path = rect_path([x, y, x + 1.0, y + 1.0], FillRule::NonZero);
            frame.draw_path(&path, &paint);
        }

        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * WIDTH + x) * 4..(y * WIDTH + x + 1) * 4];
        assert_eq!(pixel(0, 0), [255, 255, 255, 255]);
        assert_eq!(pixel(63, 63), [0, 0, 0, 255]);
    }

    #[test]
    fn msaa_split_submission_composites_translucent_src_over() {
        let factory = WgpuFactory::new_with_mode(2, 2, RenderMode::Msaa).unwrap();
        let path = rect_path([0.0, 0.0, 2.0, 2.0], FillRule::NonZero);
        let red = WgpuPaint {
            color: 0xffff_0000,
            ..WgpuPaint::default()
        };
        let green = WgpuPaint {
            color: 0x8000_ff00,
            ..WgpuPaint::default()
        };

        let mut split = factory.begin_frame(0xff00_0000);
        for _ in 0..MAX_DRAWS_PER_SUBMISSION {
            split.draw_path(&path, &red);
        }
        split.draw_path(&path, &green);

        let mut control = factory.begin_frame(0xff00_0000);
        control.draw_path(&path, &red);
        control.draw_path(&path, &green);

        assert_eq!(split.finish().unwrap(), control.finish().unwrap());
    }

    #[test]
    fn msaa_submission_splitting_requires_source_over_without_path_clips() {
        let mut draw = SolidDraw {
            path: rect_path([0.0, 0.0, 1.0, 1.0], FillRule::NonZero),
            paint: WgpuPaint::default(),
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };
        assert!(msaa_draws_can_submit_independently(std::slice::from_ref(
            &draw
        )));

        draw.paint.blend_mode = BlendMode::Multiply;
        assert!(!msaa_draws_can_submit_independently(std::slice::from_ref(
            &draw
        )));
        draw.paint.blend_mode = BlendMode::SrcOver;
        draw.role = DrawRole::Content { clip_id: 1 };
        assert!(!msaa_draws_can_submit_independently(&[draw]));
    }

    #[test]
    fn msaa_intersection_board_reserves_cpp_subpass_layers() {
        let mut draw = SolidDraw {
            path: rect_path([10.0, 10.0, 20.0, 20.0], FillRule::NonZero),
            paint: WgpuPaint::default(),
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };

        assert_eq!(msaa_draw_layer_count(&draw, false), 3);
        draw.path.fill_rule = FillRule::EvenOdd;
        assert_eq!(msaa_draw_layer_count(&draw, false), 2);
        draw.paint.style = RenderPaintStyle::Stroke;
        assert_eq!(msaa_draw_layer_count(&draw, false), 1);
        draw.paint.style = RenderPaintStyle::Fill;
        draw.role = DrawRole::ClipUpdate {
            replacement_id: 2,
            parent_id: 1,
        };
        assert_eq!(msaa_draw_layer_count(&draw, false), 1);
        assert_eq!(msaa_draw_layer_count(&draw, true), 1);
    }

    #[test]
    fn msaa_fill_pass_schedule_matches_cpp_draw_types() {
        assert_eq!(
            msaa_fill_passes(FillRule::NonZero),
            &[
                MsaaFillPass::BorrowedCoverage,
                MsaaFillPass::Forward,
                MsaaFillPass::Cleanup,
            ]
        );
        assert_eq!(
            msaa_fill_passes(FillRule::Clockwise),
            &[
                MsaaFillPass::BorrowedCoverage,
                MsaaFillPass::Forward,
                MsaaFillPass::ClockwiseCleanup,
            ]
        );
        assert_eq!(
            msaa_fill_passes(FillRule::EvenOdd),
            &[MsaaFillPass::EvenOddStencil, MsaaFillPass::EvenOddCover]
        );
    }

    #[cfg(feature = "perf-counters")]
    #[test]
    fn msaa_batches_disjoint_opaque_fills_by_subpass() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let make_frame = || {
            let mut frame = factory.begin_frame_for_benchmark(0xff00_0000, true);
            for (bounds, color) in [
                ([2.0, 2.0, 26.0, 26.0], 0xffff_0000),
                ([38.0, 2.0, 62.0, 26.0], 0xff00_ff00),
                ([2.0, 38.0, 26.0, 62.0], 0xff00_00ff),
                ([38.0, 38.0, 62.0, 62.0], 0xffff_ffff),
            ] {
                frame.draw_path(
                    &rect_path(bounds, FillRule::NonZero),
                    &WgpuPaint {
                        color,
                        ..WgpuPaint::default()
                    },
                );
            }
            frame
        };

        let metrics = make_frame().finish_for_benchmark().unwrap();
        assert_eq!(metrics.backend_work.render_passes, 2);
        assert_eq!(metrics.backend_work.gpu_draw_calls, 4);
        assert_eq!(metrics.backend_work.tessellation_spans, 19);
        assert_eq!(metrics.backend_work.path_patches, 12);

        let scheduled = make_frame().finish().unwrap();
        let serialized = make_frame().finish_without_msaa_board_scheduling().unwrap();
        assert_eq!(scheduled, serialized);
    }

    #[cfg(feature = "perf-counters")]
    #[test]
    fn plain_strokes_use_one_flush_wide_midpoint_padding_envelope() {
        let work = [RenderMode::ClockwiseAtomic, RenderMode::Msaa].map(|mode| {
            let factory = WgpuFactory::new_with_mode(300, 300, mode).unwrap();
            let mut frame = factory.begin_frame_for_benchmark(0xff00_0000, true);
            frame.transform(Mat2D([141.5, 0.0, 0.0, 141.5, 150.0, 150.0]));
            for index in 0..20 {
                let theta = std::f32::consts::TAU * index as f32 / 20.0;
                let mut raw_path = RawPath::new();
                raw_path.line_to(theta.cos(), theta.sin());
                raw_path.line_to(0.0, 0.0);
                frame.draw_path(
                    &WgpuPath {
                        valid: true,
                        raw_path,
                        fill_rule: FillRule::NonZero,
                    },
                    &WgpuPaint {
                        style: RenderPaintStyle::Stroke,
                        color: 0xff80_8080 | index,
                        thickness: 15.0 / 141.5,
                        join: StrokeJoin::Bevel,
                        cap: StrokeCap::Butt,
                        ..WgpuPaint::default()
                    },
                );
            }
            let work = frame.finish_for_benchmark().unwrap().backend_work;
            (
                work.tessellation_spans,
                work.gpu_draw_instances,
                work.path_patches,
            )
        });

        assert_eq!(work, [(63, 105, 40), (63, 103, 40)]);
    }

    #[cfg(feature = "perf-counters")]
    #[test]
    fn translucent_fills_use_one_flush_wide_midpoint_padding_envelope() {
        use nuxie_render_stream::RenderStream;

        let stream = RenderStream::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/renderer/streams/gm/batchedconvexpaths.rive-stream"
        )))
        .unwrap();
        let (width, height) = stream.frame_size.unwrap();
        let work = [RenderMode::ClockwiseAtomic, RenderMode::Msaa].map(|mode| {
            let mut factory = WgpuFactory::new_with_mode(width, height, mode).unwrap();
            let mut frame =
                factory.begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true);
            stream.replay_frame(0, &mut factory, &mut frame).unwrap();
            let work = frame.finish_for_benchmark().unwrap().backend_work;
            (
                work.tessellation_spans,
                work.gpu_draw_calls,
                work.path_patches,
            )
        });

        assert_eq!(work, [(78, 13, 142), (78, 31, 213)]);
    }

    #[cfg(feature = "perf-counters")]
    #[test]
    fn overstroke_condenses_compatible_direct_stroke_ranges() {
        use nuxie_render_stream::RenderStream;

        let stream = RenderStream::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/renderer/streams/gm/OverStroke.rive-stream"
        )))
        .unwrap();
        let (width, height) = stream.frame_size.unwrap();
        let work = [RenderMode::ClockwiseAtomic, RenderMode::Msaa].map(|mode| {
            let mut factory = WgpuFactory::new_with_mode(width, height, mode).unwrap();
            let mut frame =
                factory.begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true);
            stream.replay_frame(0, &mut factory, &mut frame).unwrap();
            frame.finish_for_benchmark().unwrap().backend_work
        });

        assert_eq!(
            (
                work[0].gpu_draw_calls,
                work[0].gpu_draw_instances,
                work[0].tessellation_spans,
                work[0].path_patches,
                work[0].buffer_upload_bytes,
                work[0].buffer_clear_calls,
                work[0].buffer_clear_bytes,
            ),
            (10, 989, 490, 497, 37_544, 0, 0)
        );
        assert_eq!(
            (
                work[1].gpu_draw_calls,
                work[1].gpu_draw_instances,
                work[1].tessellation_spans,
                work[1].path_patches,
                work[1].buffer_upload_bytes,
                work[1].buffer_clear_calls,
                work[1].buffer_clear_bytes,
            ),
            (8, 986, 489, 497, 37_696, 0, 0)
        );
    }

    #[cfg(feature = "perf-counters")]
    #[test]
    fn clockwise_atomic_large_single_contours_share_cpp_tessellation_layout() {
        use nuxie_render_stream::RenderStream;

        for (source, expected_work) in [
            (
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../fixtures/renderer/streams/gm/bug339297_as_clip.rive-stream"
                )),
                (8, 8, 556, 121, 431),
            ),
            (
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../fixtures/renderer/streams/gm/bug339297.rive-stream"
                )),
                (6, 6, 543, 117, 423),
            ),
        ] {
            let stream = RenderStream::parse(source).unwrap();
            let (width, height) = stream.frame_size.unwrap();
            let mut factory =
                WgpuFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic).unwrap();
            let mut frame =
                factory.begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true);
            stream.replay_frame(0, &mut factory, &mut frame).unwrap();
            let full = frame.finish_for_benchmark().unwrap().backend_work;

            assert_eq!(
                (
                    full.render_passes,
                    full.gpu_draw_calls,
                    full.gpu_draw_instances,
                    full.tessellation_spans,
                    full.path_patches,
                ),
                expected_work
            );
        }
    }

    #[cfg(feature = "perf-counters")]
    #[test]
    fn bug339297_msaa_clip_work_matches_cpp() {
        use nuxie_render_stream::RenderStream;

        let stream = RenderStream::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/renderer/streams/gm/bug339297_as_clip.rive-stream"
        )))
        .unwrap();
        let (width, height) = stream.frame_size.unwrap();
        let mut factory = WgpuFactory::new_with_mode(width, height, RenderMode::Msaa).unwrap();
        let mut frame = factory.begin_frame_for_benchmark(stream.clear_color.unwrap_or(0), true);
        stream.replay_frame(0, &mut factory, &mut frame).unwrap();
        let work = frame.finish_for_benchmark().unwrap().backend_work;

        assert_eq!(
            (
                work.bind_group_sets,
                work.gpu_draw_calls,
                work.gpu_draw_instances,
                work.tessellation_spans,
                work.path_patches,
            ),
            (5, 8, 848, 18, 830)
        );
    }

    #[test]
    fn overstroke_grouped_rows_match_unbatched_msaa_pixels() {
        use nuxie_render_stream::RenderStream;

        let stream = RenderStream::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/renderer/streams/gm/OverStroke.rive-stream"
        )))
        .unwrap();
        let (width, height) = stream.frame_size.unwrap();
        let make_frame = || {
            let mut factory = WgpuFactory::new_with_mode(width, height, RenderMode::Msaa).unwrap();
            let mut frame = factory.begin_frame(stream.clear_color.unwrap_or(0));
            stream.replay_frame(0, &mut factory, &mut frame).unwrap();
            frame
        };

        let grouped = make_frame().finish().unwrap();
        let unbatched = make_frame().finish_without_msaa_board_scheduling().unwrap();
        assert_eq!(grouped, unbatched);
    }

    #[cfg(feature = "perf-counters")]
    #[test]
    fn direct_stroke_batching_stops_at_overlap_opacity_and_flush_boundaries() {
        let work = |mode, second_bounds, second_opacity, split_flush| {
            let factory = WgpuFactory::new_with_mode(64, 64, mode).unwrap();
            let mut frame = factory.begin_frame_for_benchmark(0xff00_0000, true);
            let paint = WgpuPaint {
                style: RenderPaintStyle::Stroke,
                color: 0xffff_ffff,
                thickness: 2.0,
                ..WgpuPaint::default()
            };
            frame.draw_path(
                &rect_path([4.0, 4.0, 20.0, 20.0], FillRule::NonZero),
                &paint,
            );
            if split_flush {
                frame.begin_logical_flush();
            }
            frame.state.opacity = second_opacity;
            frame.draw_path(&rect_path(second_bounds, FillRule::NonZero), &paint);
            frame
                .finish_for_benchmark()
                .unwrap()
                .backend_work
                .gpu_draw_calls
        };

        for mode in [RenderMode::ClockwiseAtomic, RenderMode::Msaa] {
            let batched = work(mode, [36.0, 36.0, 52.0, 52.0], 1.0, false);
            let overlap = work(mode, [12.0, 12.0, 28.0, 28.0], 1.0, false);
            let translucent = work(mode, [36.0, 36.0, 52.0, 52.0], 0.5, false);
            let split_flush = work(mode, [36.0, 36.0, 52.0, 52.0], 1.0, true);

            assert_eq!(overlap, batched + 1);
            assert_eq!(translucent, batched + 1);
            assert!(split_flush >= batched + 2);
        }
    }

    #[test]
    fn direct_stroke_batching_rejects_pipeline_and_order_boundaries() {
        let draw = SolidDraw {
            path: rect_path([0.0, 0.0, 1.0, 1.0], FillRule::NonZero),
            paint: WgpuPaint {
                style: RenderPaintStyle::Stroke,
                ..WgpuPaint::default()
            },
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };
        assert!(direct_stroke_can_batch(&draw, false));
        assert!(!direct_stroke_can_batch(&draw, true));

        let mut candidate = draw.clone();
        candidate.paint.style = RenderPaintStyle::Fill;
        assert!(!direct_stroke_can_batch(&candidate, false));
        candidate = draw.clone();
        candidate.paint.feather = 1.0;
        assert!(!direct_stroke_can_batch(&candidate, false));
        candidate = draw.clone();
        candidate.paint.blend_mode = BlendMode::Multiply;
        assert!(!direct_stroke_can_batch(&candidate, false));
        candidate = draw.clone();
        candidate.paint.color = 0x8000_0000;
        assert!(!direct_stroke_can_batch(&candidate, false));
        candidate = draw.clone();
        candidate.state.opacity = 0.5;
        assert!(!direct_stroke_can_batch(&candidate, false));
        candidate = draw.clone();
        candidate.state.clip_rect = Some(ClipRectState {
            rect: [0.0, 0.0, 1.0, 1.0],
            matrix: Mat2D::IDENTITY,
        });
        assert!(!direct_stroke_can_batch(&candidate, false));
        candidate = draw.clone();
        candidate.state.clip_stack_height = 1;
        assert!(!direct_stroke_can_batch(&candidate, false));
        candidate = draw;
        candidate.role = DrawRole::Content { clip_id: 1 };
        assert!(!direct_stroke_can_batch(&candidate, false));
    }

    #[test]
    fn compact_midpoint_groups_preserve_group_and_range_boundaries() {
        let groups =
            compact_midpoint_groups(&[0, 1, 1, 2], &[(1, 10), (1, 20), (1, 30), (1, 40)], 4)
                .unwrap();
        assert_eq!(
            groups,
            [
                CompactMidpointGroup {
                    range: 0..1,
                    geometry_end: 88,
                },
                CompactMidpointGroup {
                    range: 1..3,
                    geometry_end: 408,
                },
                CompactMidpointGroup {
                    range: 3..4,
                    geometry_end: 328,
                },
            ]
        );
        assert!(compact_midpoint_groups(&[0, 0], &[(1, 1), (2, 1)], 2).is_none());
        assert!(compact_midpoint_groups(&[0, 1], &[(1, 1), (1, 1)], 2).is_none());
        assert!(compact_midpoint_groups(&[0, 0], &[(1, 128), (1, 128)], 2).is_none());
        assert!(compact_midpoint_groups(&[0, 1, 1], &[(1, 1); 3], 1).is_none());
    }

    #[test]
    fn empty_msaa_frame_resolves_its_clear_directly() {
        let factory = WgpuFactory::new_with_mode(2, 2, RenderMode::Msaa).unwrap();

        let pixels = factory.begin_frame(0x8040_2010).finish().unwrap();

        for pixel in pixels.chunks_exact(4) {
            assert_eq!(pixel, [32, 16, 8, 128]);
        }
    }

    #[test]
    fn async_factory_and_frame_completion_render_without_sync_wrappers() {
        let pixels = pollster::block_on(async {
            let factory = WgpuFactory::new_async_with_mode(2, 2, RenderMode::Msaa).await?;
            factory.begin_frame(0x8040_2010).finish_async().await
        })
        .unwrap();

        for pixel in pixels.chunks_exact(4) {
            assert_eq!(pixel, [32, 16, 8, 128]);
        }
    }

    #[test]
    fn foreign_wgpu_path_paint_and_shader_fail_closed() {
        let mut factory = WgpuFactory::new_with_mode(4, 4, RenderMode::Msaa).unwrap();
        let mut recording = nuxie_render_api::RecordingFactory::new();
        let foreign_path = recording.make_empty_render_path();
        let foreign_paint = recording.make_render_paint();
        let foreign_shader = recording.make_linear_gradient(
            0.0,
            0.0,
            1.0,
            1.0,
            &[0xff00_0000, 0xffff_ffff],
            &[0.0, 1.0],
        );

        let local_path = factory.make_empty_render_path();
        let local_paint = factory.make_render_paint();

        let mut frame = factory.begin_frame(0xff00_0000);
        frame.draw_path(foreign_path.as_ref(), local_paint.as_ref());
        assert!(matches!(
            frame.finish(),
            Err(RendererError::Unsupported(
                "path from another renderer backend"
            ))
        ));

        let mut composed_path = factory.make_empty_render_path();
        composed_path.move_to(0.0, 0.0);
        composed_path.line_to(4.0, 0.0);
        composed_path.line_to(0.0, 4.0);
        composed_path.close();
        composed_path.add_render_path(foreign_path.as_ref(), Mat2D::IDENTITY);
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.draw_path(composed_path.as_ref(), local_paint.as_ref());
        assert!(matches!(
            frame.finish(),
            Err(RendererError::Unsupported(
                "path contains resources from another renderer backend"
            ))
        ));

        let mut frame = factory.begin_frame(0xff00_0000);
        frame.draw_path(local_path.as_ref(), foreign_paint.as_ref());
        assert!(matches!(
            frame.finish(),
            Err(RendererError::Unsupported(
                "paint from another renderer backend"
            ))
        ));

        let mut invalid_paint = factory.make_render_paint();
        invalid_paint.shader(Some(foreign_shader.as_ref()));
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.draw_path(local_path.as_ref(), invalid_paint.as_ref());
        assert!(matches!(
            frame.finish(),
            Err(RendererError::Unsupported(
                "paint shader from another renderer backend"
            ))
        ));
    }

    #[test]
    fn foreign_wgpu_device_resources_fail_before_gpu_validation() {
        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut encoded, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&[255, 255, 255, 255])
                .unwrap();
        }

        let mut first = WgpuFactory::new_with_mode(4, 4, RenderMode::ClockwiseAtomic).unwrap();
        let mut second = WgpuFactory::new_with_mode(4, 4, RenderMode::ClockwiseAtomic).unwrap();
        let foreign_image = first.decode_image(&encoded).expect("decode fixture image");
        let mut frame = second.begin_frame(0xff00_0000);
        frame.draw_image(
            Some(foreign_image.as_ref()),
            ImageSampler::LINEAR_CLAMP,
            BlendMode::SrcOver,
            1.0,
        );
        assert!(matches!(
            frame.finish(),
            Err(RendererError::Unsupported(
                "image from another renderer factory"
            ))
        ));

        let image = second.decode_image(&encoded).expect("decode fixture image");
        let vertices =
            first.make_render_buffer(RenderBufferType::Vertex, RenderBufferFlags::None, 8);
        let uvs = first.make_render_buffer(RenderBufferType::Vertex, RenderBufferFlags::None, 8);
        let indices = first.make_render_buffer(RenderBufferType::Index, RenderBufferFlags::None, 2);
        let mut frame = second.begin_frame(0xff00_0000);
        frame.draw_image_mesh(
            Some(image.as_ref()),
            ImageSampler::LINEAR_CLAMP,
            Some(vertices.as_ref()),
            Some(uvs.as_ref()),
            Some(indices.as_ref()),
            1,
            1,
            BlendMode::SrcOver,
            1.0,
        );
        assert!(matches!(
            frame.finish(),
            Err(RendererError::Unsupported(
                "image mesh buffers from another renderer factory"
            ))
        ));
    }

    #[test]
    fn frame_attachments_are_reused_after_gpu_completion() {
        for mode in [RenderMode::ClockwiseAtomic, RenderMode::Msaa] {
            let factory = WgpuFactory::new_with_mode(8, 8, mode).unwrap();
            let initial = factory.context.frame_attachments.cached();

            factory
                .begin_frame_for_benchmark(0xff00_0000, false)
                .finish_for_benchmark()
                .unwrap();
            let after_first = factory.context.frame_attachments.cached();
            assert!(Arc::ptr_eq(&initial, &after_first));

            factory
                .begin_frame_for_benchmark(0xff00_0000, false)
                .finish_for_benchmark()
                .unwrap();
            let after_second = factory.context.frame_attachments.cached();
            assert!(Arc::ptr_eq(&initial, &after_second));
        }
    }

    #[test]
    fn frame_attachment_pool_drops_concurrent_overflow() {
        let factory = WgpuFactory::new_with_mode(8, 8, RenderMode::Msaa).unwrap();
        let first = factory
            .context
            .frame_attachments
            .checkout(&factory.context.device);
        let second = factory
            .context
            .frame_attachments
            .checkout(&factory.context.device);
        assert!(!Arc::ptr_eq(&first, &second));

        factory.context.frame_attachments.recycle(first);
        factory.context.frame_attachments.recycle(second);
        assert_eq!(factory.context.frame_attachments.cached_len(), 1);
    }

    #[test]
    fn msaa_intersection_board_reordering_preserves_overlapping_source_order() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let make_frame = || {
            let mut frame = factory.begin_frame(0xff00_0000);
            for (bounds, color) in [
                ([10.0, 10.0, 20.0, 20.0], 0xffff_0000),
                ([15.0, 10.0, 25.0, 20.0], 0xff00_ff00),
                ([40.0, 40.0, 50.0, 50.0], 0xff00_00ff),
            ] {
                frame.draw_path(
                    &rect_path(bounds, FillRule::NonZero),
                    &WgpuPaint {
                        color,
                        ..WgpuPaint::default()
                    },
                );
            }
            frame
        };

        let scheduled = make_frame().finish().unwrap();
        let serialized = make_frame().finish_without_msaa_board_scheduling().unwrap();
        assert_eq!(scheduled, serialized);
        let pixel = |x: usize, y: usize| &scheduled[(y * 64 + x) * 4..][..4];
        assert_eq!(pixel(12, 15), [255, 0, 0, 255]);
        assert_eq!(pixel(18, 15), [0, 255, 0, 255]);
        assert_eq!(pixel(45, 45), [0, 0, 255, 255]);
    }

    #[test]
    fn midpoint_shelf_placement_reports_texture_height_rollover() {
        let width = gpu::TESS_TEXTURE_WIDTH as u32;
        assert_eq!(
            midpoint_shelf_placement(width - 8, 3, 1, Some(16)),
            MidpointShelfPlacement {
                x: 0,
                y: 4,
                next_x: 16,
                next_y: 4,
                height: 5,
            }
        );
        assert_eq!(
            midpoint_shelf_placement(8, 3, 2, None),
            MidpointShelfPlacement {
                x: 0,
                y: 4,
                next_x: 0,
                next_y: 6,
                height: 6,
            }
        );
    }

    fn rect_path(bounds: [f32; 4], fill_rule: FillRule) -> WgpuPath {
        let [left, top, right, bottom] = bounds;
        let mut raw_path = RawPath::new();
        raw_path.move_to(left, top);
        raw_path.line_to(right, top);
        raw_path.line_to(right, bottom);
        raw_path.line_to(left, bottom);
        raw_path.close();
        WgpuPath {
            valid: true,
            raw_path,
            fill_rule,
        }
    }

    #[test]
    fn clockwise_atomic_main_triangles_uses_bounded_unit_weight_instances() {
        let triangle = |weight, path_id, x| {
            [
                gpu::TriangleVertex::new([x, 0.0], weight, path_id),
                gpu::TriangleVertex::new([x + 1.0, 0.0], weight, path_id),
                gpu::TriangleVertex::new([x, 1.0], weight, path_id),
            ]
        };
        let triangles = [
            triangle(-1, 3, 0.0),
            triangle(0, 5, 2.0),
            triangle(2, 7, 4.0),
            triangle(2, 9, 6.0),
            triangle(1, 11, 8.0),
        ]
        .concat();

        let inactive_clip = clockwise_atomic_main_triangles(&triangles, true);
        assert_eq!(inactive_clip.vertices.len(), 9);
        assert!(inactive_clip.vertices[..3]
            .iter()
            .all(|vertex| vertex.weight_path_id == ((1 << 16) | 7)));
        assert!(inactive_clip.vertices[3..6]
            .iter()
            .all(|vertex| vertex.weight_path_id == ((1 << 16) | 9)));
        assert!(inactive_clip.vertices[6..]
            .iter()
            .all(|vertex| vertex.weight_path_id == ((1 << 16) | 11)));
        assert_eq!(
            inactive_clip.batches,
            [
                clockwise_atomic_pipeline::ClockwiseAtomicTriangleBatch {
                    vertex_start: 0,
                    vertex_count: 3,
                    instance_count: 2,
                },
                clockwise_atomic_pipeline::ClockwiseAtomicTriangleBatch {
                    vertex_start: 3,
                    vertex_count: 3,
                    instance_count: 2,
                },
                clockwise_atomic_pipeline::ClockwiseAtomicTriangleBatch {
                    vertex_start: 6,
                    vertex_count: 3,
                    instance_count: 1,
                },
            ]
        );

        let active_clip = clockwise_atomic_main_triangles(&triangles, false);
        assert_eq!(active_clip.vertices.len(), 9);
        assert!(active_clip.vertices[..3]
            .iter()
            .all(|vertex| vertex.weight_path_id == ((2 << 16) | 7)));
        assert_eq!(
            active_clip.batches,
            [clockwise_atomic_pipeline::ClockwiseAtomicTriangleBatch {
                vertex_start: 0,
                vertex_count: 9,
                instance_count: 1,
            }]
        );
    }

    #[test]
    fn clockwise_atomic_main_triangles_stay_linear_for_rising_weights_and_clip_states() {
        let triangle = |weight, path_id, x| {
            [
                gpu::TriangleVertex::new([x, 0.0], weight, path_id),
                gpu::TriangleVertex::new([x + 1.0, 0.0], weight, path_id),
                gpu::TriangleVertex::new([x, 1.0], weight, path_id),
            ]
        };
        let triangles = (-1i16..=1024)
            .flat_map(|weight| triangle(weight, 7, f32::from(weight) * 2.0))
            .collect::<Vec<_>>();

        let inactive_clip = clockwise_atomic_main_triangles(&triangles, true);
        assert_eq!(inactive_clip.vertices.len(), 1024 * 3);
        assert!(inactive_clip.vertices.capacity() <= triangles.len() * 2);
        assert_eq!(inactive_clip.batches.len(), 1024);
        assert!(inactive_clip
            .vertices
            .iter()
            .all(|vertex| vertex.weight_path_id >> 16 == 1));
        assert_eq!(inactive_clip.batches[0].instance_count, 1);
        assert_eq!(inactive_clip.batches.last().unwrap().instance_count, 1024);

        let active_clip = clockwise_atomic_main_triangles(&triangles, false);
        assert_eq!(active_clip.vertices.len(), 1024 * 3);
        assert_eq!(active_clip.batches.len(), 1);
        assert_eq!(active_clip.vertices[0].weight_path_id >> 16, 1);
        assert_eq!(
            active_clip.vertices.last().unwrap().weight_path_id >> 16,
            1024
        );

        let unit_triangles = (0..1024)
            .flat_map(|index| triangle(1, 7, index as f32 * 2.0))
            .collect::<Vec<_>>();
        let unit = clockwise_atomic_main_triangles(&unit_triangles, true);
        assert_eq!(unit.vertices.len(), 1024 * 3);
        assert_eq!(
            unit.batches,
            [clockwise_atomic_pipeline::ClockwiseAtomicTriangleBatch {
                vertex_start: 0,
                vertex_count: 1024 * 3,
                instance_count: 1,
            }]
        );
    }

    #[test]
    fn clockwise_atomic_clip_is_inactive_only_with_full_pixel_margin() {
        let make_draw = |clip_rect| SolidDraw {
            path: rect_path([10.0, 10.0, 20.0, 20.0], FillRule::Clockwise),
            paint: WgpuPaint::default(),
            state: DrawState {
                clip_rect,
                ..DrawState::default()
            },
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };
        let clip = |rect| {
            Some(ClipRectState {
                rect,
                matrix: Mat2D::IDENTITY,
            })
        };

        assert!(clockwise_atomic_clip_is_inactive(&make_draw(None)));
        assert!(clockwise_atomic_clip_is_inactive(&make_draw(clip([
            9.0, 9.0, 21.0, 21.0
        ]))));
        assert!(!clockwise_atomic_clip_is_inactive(&make_draw(clip([
            10.0, 10.0, 20.0, 20.0
        ]))));
        assert!(!clockwise_atomic_clip_is_inactive(&make_draw(clip([
            9.0, 9.0, 19.0, 21.0
        ]))));
    }

    fn append_oval(path: &mut RawPath, bounds: [f32; 4]) {
        const KAPPA: f32 = 0.552_284_8;
        let [left, top, right, bottom] = bounds;
        let center_x = (left + right) * 0.5;
        let center_y = (top + bottom) * 0.5;
        let radius_x = (right - left) * 0.5;
        let radius_y = (bottom - top) * 0.5;
        path.move_to(right, center_y);
        path.cubic_to(
            right,
            center_y + radius_y * KAPPA,
            center_x + radius_x * KAPPA,
            bottom,
            center_x,
            bottom,
        );
        path.cubic_to(
            center_x - radius_x * KAPPA,
            bottom,
            left,
            center_y + radius_y * KAPPA,
            left,
            center_y,
        );
        path.cubic_to(
            left,
            center_y - radius_y * KAPPA,
            center_x - radius_x * KAPPA,
            top,
            center_x,
            top,
        );
        path.cubic_to(
            center_x + radius_x * KAPPA,
            top,
            right,
            center_y - radius_y * KAPPA,
            right,
            center_y,
        );
        path.close();
    }

    #[test]
    fn clockwise_atomic_override_unions_overlapping_cubic_contours() {
        let factory = WgpuFactory::new_with_mode(256, 160, RenderMode::ClockwiseAtomic).unwrap();
        let mut raw_path = RawPath::new();
        append_oval(&mut raw_path, [20.0, 20.0, 140.0, 140.0]);
        append_oval(&mut raw_path, [100.0, 20.0, 220.0, 140.0]);
        let path = WgpuPath {
            valid: true,
            raw_path,
            fill_rule: FillRule::EvenOdd,
        };
        let mut frame = factory.begin_frame(0xffff_ffff);
        frame.draw_path(
            &path,
            &WgpuPaint {
                color: 0xff44_88cc,
                ..WgpuPaint::default()
            },
        );
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 256 + x) * 4..][..4];

        assert_eq!(pixel(0, 0), [0xff; 4]);
        assert_eq!(pixel(80, 80), [0x44, 0x88, 0xcc, 0xff]);
        assert_eq!(pixel(120, 80), [0x44, 0x88, 0xcc, 0xff]);
    }

    #[test]
    fn clockwise_atomic_compound_draw_preserves_prior_draw_and_hole() {
        let factory = WgpuFactory::new_with_mode(256, 256, RenderMode::ClockwiseAtomic).unwrap();
        let mut red_raw_path = RawPath::new();
        append_oval(&mut red_raw_path, [10.0, 10.0, 100.0, 50.0]);
        let red_path = WgpuPath {
            valid: true,
            raw_path: red_raw_path,
            fill_rule: FillRule::NonZero,
        };
        let mut ring_raw_path = RawPath::new();
        append_oval(&mut ring_raw_path, [70.0, 70.0, 200.0, 200.0]);
        let mut inner = RawPath::new();
        append_oval(&mut inner, [90.0, 90.0, 180.0, 180.0]);
        ring_raw_path.add_path_backwards(&inner, Mat2D::IDENTITY);
        let ring_path = WgpuPath {
            valid: true,
            raw_path: ring_raw_path,
            fill_rule: FillRule::NonZero,
        };
        let mut frame = factory.begin_frame(0xffff_ffff);
        frame.draw_path(
            &red_path,
            &WgpuPaint {
                color: 0xffff_0000,
                ..WgpuPaint::default()
            },
        );
        frame.draw_path(
            &ring_path,
            &WgpuPaint {
                color: 0xff00_00ff,
                ..WgpuPaint::default()
            },
        );
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 256 + x) * 4..][..4];

        assert_eq!(pixel(55, 30), [0xff, 0, 0, 0xff]);
        assert_eq!(pixel(135, 80), [0, 0, 0xff, 0xff]);
        assert_eq!(pixel(135, 135), [0xff; 4]);
    }

    #[test]
    fn clockwise_atomic_override_rejects_joel_signed_opposite_winding_leaf() {
        use nuxie_render_stream::{Command, RenderStream};

        let mut stream = RenderStream::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/renderer/streams/riv/joel_signed.rive-stream"
        )))
        .unwrap();
        let first_draw = stream.frames[0]
            .commands
            .iter()
            .position(|command| matches!(command, Command::DrawPath { .. }))
            .unwrap();
        stream.frames[0].commands.truncate(first_draw + 1);
        let mut factory =
            WgpuFactory::new_with_mode(1000, 1000, RenderMode::ClockwiseAtomic).unwrap();
        let mut frame = factory.begin_frame(stream.clear_color.unwrap_or(0));
        stream.replay_frame(0, &mut factory, &mut frame).unwrap();
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 1000 + x) * 4..][..4];

        assert_eq!(pixel(321, 119), [0, 0, 0, 0]);
    }

    fn negative_interior_path() -> WgpuPath {
        let mut raw_path = RawPath::new();
        raw_path.move_to(1600.0, 0.0);
        raw_path.line_to(0.0, 0.0);
        raw_path.line_to(0.0, 1600.0);
        raw_path.line_to(1600.0, 1600.0);
        raw_path.close();
        for x in [800.0, 0.0, 800.0] {
            raw_path.move_to(x + 50.0, 640.0);
            raw_path.cubic_to(x + 50.0, 0.0, x + 750.0, 0.0, x + 750.0, 640.0);
            raw_path.cubic_to(x + 750.0, 1600.0, x + 50.0, 1600.0, x + 50.0, 640.0);
        }
        WgpuPath {
            valid: true,
            raw_path,
            fill_rule: FillRule::Clockwise,
        }
    }

    fn negative_interior_checkerboard() -> WgpuPath {
        let mut raw_path = RawPath::new();
        for index in 0..50 {
            let offset = index as f32 * 32.0;
            let (horizontal, vertical) = if index & 1 == 0 {
                (
                    [
                        [0.0, offset],
                        [0.0, offset + 32.0],
                        [1600.0, offset + 32.0],
                        [1600.0, offset],
                    ],
                    [
                        [offset, 0.0],
                        [offset, 1600.0],
                        [offset + 32.0, 1600.0],
                        [offset + 32.0, 0.0],
                    ],
                )
            } else {
                (
                    [
                        [0.0, offset],
                        [1600.0, offset],
                        [1600.0, offset + 32.0],
                        [0.0, offset + 32.0],
                    ],
                    [
                        [offset, 0.0],
                        [offset + 32.0, 0.0],
                        [offset + 32.0, 1600.0],
                        [offset, 1600.0],
                    ],
                )
            };
            for points in [horizontal, vertical] {
                raw_path.move_to(points[0][0], points[0][1]);
                for point in &points[1..] {
                    raw_path.line_to(point[0], point[1]);
                }
                raw_path.close();
            }
        }
        WgpuPath {
            valid: true,
            raw_path,
            fill_rule: FillRule::Clockwise,
        }
    }

    fn assert_post_contour_padding(tessellation: &draw::FillTessellation) {
        let logical_end = (tessellation.base_instance + tessellation.instance_count)
            * gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let alignment = gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32;
        let index = logical_end.div_ceil(alignment) * alignment;
        let padding = tessellation
            .spans
            .iter()
            .find(|span| {
                span.x_range() == (index as i32, index as i32 + 1)
                    && span.segment_counts == 0x0010_0000
                    && span.contour_id_with_flags == 0
            })
            .expect("final C++-ordered padding span");
        assert_eq!(padding.x_range(), (index as i32, index as i32 + 1));
        assert_eq!(padding.segment_counts, 0x0010_0000);
        assert_eq!(padding.contour_id_with_flags, 0);
    }

    #[test]
    fn feathered_stroke_uses_effective_round_style_for_direct_and_atlas_tessellation() {
        let paint = WgpuPaint {
            style: RenderPaintStyle::Stroke,
            thickness: 8.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 18.0,
            ..WgpuPaint::default()
        };
        assert_eq!(
            paint.effective_stroke(),
            Some((8.0, StrokeJoin::Round, StrokeCap::Round))
        );
        assert_eq!(paint.join, StrokeJoin::Miter);
        assert_eq!(paint.cap, StrokeCap::Butt);

        let mut path = RawPath::new();
        path.move_to(16.0, 16.0);
        path.line_to(48.0, 16.0);
        path.line_to(48.0, 48.0);
        path.line_to(16.0, 48.0);
        path.close();

        for tessellation in [
            draw::build_feather_tessellation(
                &path,
                Mat2D::IDENTITY,
                paint.feather,
                paint.effective_stroke(),
            )
            .unwrap(),
            draw::build_feather_atlas_tessellation(
                &path,
                Mat2D::IDENTITY,
                paint.feather,
                paint.effective_stroke(),
            )
            .unwrap(),
        ] {
            assert_eq!(tessellation.base_instance, 1);
            assert_eq!(tessellation.instance_count, 5);
            assert_eq!(tessellation.spans.len(), 7);
            let geometry = tessellation
                .spans
                .iter()
                .filter(|span| span.contour_id_with_flags != 0)
                .collect::<Vec<_>>();
            assert_eq!(
                geometry
                    .iter()
                    .map(|span| span.x_range())
                    .collect::<Vec<_>>(),
                vec![(8, 18), (18, 28), (28, 38), (38, 48)]
            );
            assert!(geometry
                .iter()
                .all(|span| span.segment_counts == 0x0090_0401));
            assert!(geometry
                .iter()
                .all(|span| span.contour_id_with_flags == 0x0800_0001));
            assert_post_contour_padding(&tessellation);
        }
    }

    #[test]
    fn feathered_open_stroke_uses_effective_round_caps_for_direct_and_atlas_tessellation() {
        let paint = WgpuPaint {
            style: RenderPaintStyle::Stroke,
            thickness: 8.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 18.0,
            ..WgpuPaint::default()
        };
        let mut path = RawPath::new();
        path.move_to(16.0, 32.0);
        path.line_to(48.0, 32.0);

        for tessellation in [
            draw::build_feather_tessellation(
                &path,
                Mat2D::IDENTITY,
                paint.feather,
                paint.effective_stroke(),
            )
            .unwrap(),
            draw::build_feather_atlas_tessellation(
                &path,
                Mat2D::IDENTITY,
                paint.feather,
                paint.effective_stroke(),
            )
            .unwrap(),
        ] {
            assert_eq!(tessellation.base_instance, 1);
            assert_eq!(tessellation.instance_count, 5);
            assert_eq!(tessellation.spans.len(), 5);
            let geometry = tessellation
                .spans
                .iter()
                .filter(|span| span.contour_id_with_flags != 0)
                .collect::<Vec<_>>();
            assert_eq!(geometry.len(), 2);
            assert_eq!(geometry[0].x_range(), (8, 27));
            assert_eq!(geometry[1].x_range(), (27, 48));
            assert_eq!(geometry[0].segment_counts, 0x0140_0000);
            assert_eq!(geometry[1].segment_counts, 0x0140_0401);
            assert_eq!(geometry[0].contour_id_with_flags, 0x0a00_0001);
            assert_eq!(geometry[1].contour_id_with_flags, 0x0a00_0001);
            assert_post_contour_padding(&tessellation);
        }
    }

    #[test]
    fn culls_empty_and_invalid_path_draws_like_cpp() {
        let empty = WgpuPath {
            valid: true,
            raw_path: RawPath::new(),
            fill_rule: FillRule::NonZero,
        };
        let mut path = empty.clone();
        path.raw_path.move_to(0.0, 0.0);
        path.raw_path.line_to(1.0, 0.0);
        path.raw_path.line_to(0.0, 1.0);
        path.raw_path.close();
        let mut paint = WgpuPaint::default();

        assert!(path_draw_is_noop(&empty, &paint, Mat2D::IDENTITY));
        assert!(!path_draw_is_noop(&path, &paint, Mat2D::IDENTITY));

        paint.style = RenderPaintStyle::Stroke;
        paint.thickness = 0.0;
        assert!(path_draw_is_noop(&path, &paint, Mat2D::IDENTITY));
        paint.thickness = f32::NAN;
        assert!(path_draw_is_noop(&path, &paint, Mat2D::IDENTITY));
        paint.thickness = 1.0;
        paint.feather = f32::NAN;
        assert!(path_draw_is_noop(&path, &paint, Mat2D::IDENTITY));

        let mut move_only = empty.clone();
        move_only.raw_path.move_to(4.0, 4.0);
        paint = WgpuPaint::default();
        assert!(path_draw_is_noop(&move_only, &paint, Mat2D::IDENTITY));
    }

    #[test]
    fn culls_path_draws_outside_the_cpp_frame_bounds() {
        let mut path = WgpuPath {
            valid: true,
            raw_path: RawPath::new(),
            fill_rule: FillRule::NonZero,
        };
        path.raw_path.move_to(-20.0, -20.0);
        path.raw_path.line_to(-10.0, -20.0);
        path.raw_path.line_to(-10.0, -10.0);
        path.raw_path.close();
        let mut paint = WgpuPaint::default();

        assert!(path_draw_is_outside_frame(
            &path,
            &paint,
            Mat2D::IDENTITY,
            64,
            64
        ));

        paint.style = RenderPaintStyle::Stroke;
        paint.thickness = 24.0;
        assert!(!path_draw_is_outside_frame(
            &path,
            &paint,
            Mat2D::IDENTITY,
            64,
            64
        ));
    }

    #[test]
    fn paint_thickness_matches_cpp_absolute_value_setter() {
        let mut paint = WgpuPaint::default();
        RenderPaint::thickness(&mut paint, -3.5);
        assert_eq!(paint.thickness, 3.5);

        RenderPaint::thickness(&mut paint, f32::NAN);
        assert!(paint.thickness.is_nan());
    }

    #[test]
    fn paint_feather_matches_cpp_absolute_value_setter() {
        let mut paint = WgpuPaint::default();
        RenderPaint::feather(&mut paint, -8.0);
        assert_eq!(paint.feather, 8.0);

        RenderPaint::feather(&mut paint, f32::NAN);
        assert!(paint.feather.is_nan());
    }

    #[test]
    fn absurd_stroke_width_does_not_overflow_tessellation() {
        let mut path = RawPath::new();
        path.move_to(4.0, 4.0);
        path.line_to(28.0, 28.0);
        assert!(draw::build_stroke_tessellation(
            &path,
            Mat2D::IDENTITY,
            f32::MAX,
            StrokeJoin::Miter,
            StrokeCap::Butt,
        )
        .is_some());
    }

    #[test]
    fn solid_triangle_tessellates_to_one_gpu_triangle() {
        let mut raw_path = RawPath::new();
        raw_path.move_to(0.0, 0.0);
        raw_path.line_to(10.0, 0.0);
        raw_path.line_to(0.0, 10.0);
        raw_path.close();
        let draw = SolidDraw {
            path: WgpuPath {
                valid: true,
                raw_path,
                fill_rule: FillRule::NonZero,
            },
            paint: WgpuPaint::default(),
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };
        assert_eq!(tessellate_solid(&draw, 10, 10).unwrap().len(), 3);
    }

    #[test]
    fn recognizes_cpp_axis_aligned_clip_path() {
        let mut raw_path = RawPath::new();
        raw_path.move_to(0.0, 0.0);
        raw_path.line_to(64.0, 0.0);
        raw_path.line_to(64.0, 32.0);
        raw_path.line_to(0.0, 32.0);
        raw_path.close();
        assert_eq!(path_aabb(&raw_path), Some([0.0, 0.0, 64.0, 32.0]));
    }

    #[test]
    fn clip_rect_state_intersects_and_restores_like_cpp() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let outer = rect_path([8.0, 8.0, 56.0, 56.0], FillRule::NonZero);
        let inner = rect_path([16.0, 4.0, 60.0, 48.0], FillRule::NonZero);
        let mut frame = factory.begin_frame(0xff00_0000);

        frame.save();
        frame.clip_path(&outer);
        frame.clip_path(&inner);
        assert_eq!(frame.state.clip_rect.unwrap().rect, [16.0, 8.0, 56.0, 48.0]);
        let aux = clip_rect_paint_aux(frame.state.clip_rect);
        for (actual, expected) in aux
            .clip_rect_inverse_matrix
            .into_iter()
            .zip([0.05, 0.0, 0.0, 0.05, -1.8, -1.4])
        {
            assert!((actual - expected).abs() < 1e-6);
        }
        for value in aux.inverse_fwidth {
            assert!((value + 20.0).abs() < 1e-5);
        }
        frame.restore();
        assert!(frame.state.clip_rect.is_none());
    }

    #[test]
    fn incompatible_transformed_clip_rect_falls_back_to_path_like_cpp() {
        let factory = WgpuFactory::new_with_mode(500, 500, RenderMode::ClockwiseAtomic).unwrap();
        let outer = rect_path([0.0, 0.0, 500.0, 500.0], FillRule::NonZero);
        let inner = rect_path([0.0, 0.0, 50.0, 50.0], FillRule::NonZero);
        let inner_matrix = Mat2D([
            0.5135926,
            4.121487e-9,
            0.00041051814,
            0.5135926,
            92.882,
            302.4731,
        ]);
        let mut frame = factory.begin_frame(0xff00_0000);

        frame.clip_path(&outer);
        frame.transform(inner_matrix);
        frame.clip_path(&inner);

        let clip_rect = frame.state.clip_rect.unwrap();
        assert_eq!(clip_rect.rect, [0.0, 0.0, 500.0, 500.0]);
        assert_eq!(clip_rect.matrix, Mat2D::IDENTITY);
        assert_eq!(frame.state.clip_stack_height, 1);
        assert_eq!(frame.clips.len(), 1);
        assert_eq!(frame.clips[0].path, inner);
        assert_eq!(frame.clips[0].matrix, inner_matrix);
        assert!(frame.unsupported.is_none());
    }

    #[test]
    fn msaa_incompatible_transformed_clip_rects_use_the_stencil_clip_stack() {
        let factory = WgpuFactory::new_with_mode(500, 500, RenderMode::Msaa).unwrap();
        let outer = rect_path([0.0, 0.0, 500.0, 500.0], FillRule::NonZero);
        let inner = rect_path([0.0, 0.0, 50.0, 50.0], FillRule::NonZero);
        let inner_matrix = Mat2D([
            0.5135926,
            4.121487e-9,
            0.00041051814,
            0.5135926,
            92.882,
            302.4731,
        ]);
        let mut frame = factory.begin_frame(0xff00_0000);

        frame.clip_path(&outer);
        frame.transform(inner_matrix);
        frame.clip_path(&inner);
        let (scheduled, clip_id) = frame.prepare_scheduled_clip_updates().unwrap();

        assert!(frame.state.clip_rect.is_none());
        assert_eq!(frame.state.clip_stack_height, 2);
        assert_eq!(frame.clips.len(), 2);
        assert_eq!(frame.clips[0].path, outer);
        assert_eq!(frame.clips[0].matrix, Mat2D::IDENTITY);
        assert_eq!(frame.clips[1].path, inner);
        assert_eq!(frame.clips[1].matrix, inner_matrix);
        assert!(matches!(
            scheduled[0].role,
            DrawRole::ClipUpdate {
                replacement_id: 1,
                parent_id: 0
            }
        ));
        assert!(matches!(
            scheduled[1].role,
            DrawRole::ClipUpdate {
                replacement_id: 2,
                parent_id: 1
            }
        ));
        assert!(matches!(
            scheduled[2].role,
            DrawRole::ClipReset {
                action: MsaaClipResetAction::IntersectPreviousNonZero,
                ..
            }
        ));
        assert_eq!(clip_id, 2);
    }

    #[test]
    fn msaa_clip_stack_replays_from_root_when_the_resident_leaf_diverges() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let triangle = |points: [[f32; 2]; 3]| {
            let mut raw_path = RawPath::new();
            raw_path.move_to(points[0][0], points[0][1]);
            raw_path.line_to(points[1][0], points[1][1]);
            raw_path.line_to(points[2][0], points[2][1]);
            raw_path.close();
            WgpuPath {
                valid: true,
                raw_path,
                fill_rule: FillRule::NonZero,
            }
        };
        let outer = triangle([[32.0, 4.0], [60.0, 60.0], [4.0, 60.0]]);
        let old_leaf = triangle([[32.0, 12.0], [48.0, 48.0], [16.0, 48.0]]);
        let new_leaf = triangle([[32.0, 18.0], [42.0, 42.0], [22.0, 42.0]]);
        let mut frame = factory.begin_frame(0xff00_0000);

        frame.clip_path(&outer);
        frame.save();
        frame.clip_path(&old_leaf);
        let (_, initial_clip_id) = frame.prepare_scheduled_clip_updates().unwrap();
        assert_eq!(initial_clip_id, 2);
        frame.restore();

        frame.clip_path(&new_leaf);
        let (scheduled, clip_id) = frame.prepare_scheduled_clip_updates().unwrap();

        assert_eq!(scheduled.len(), 4);
        assert!(matches!(
            scheduled[0].role,
            DrawRole::ClipReset {
                action: MsaaClipResetAction::ClearPrevious,
                ..
            }
        ));
        assert!(matches!(
            scheduled[1].role,
            DrawRole::ClipUpdate {
                replacement_id: 3,
                parent_id: 0
            }
        ));
        assert!(matches!(
            scheduled[2].role,
            DrawRole::ClipUpdate {
                replacement_id: 4,
                parent_id: 3
            }
        ));
        assert!(matches!(
            scheduled[3].role,
            DrawRole::ClipReset {
                action: MsaaClipResetAction::IntersectPreviousNonZero,
                ..
            }
        ));
        assert_eq!(clip_id, 4);
        assert_eq!(frame.clips[0].clip_id, 3);
        assert_eq!(frame.clips[1].clip_id, 4);
    }

    #[test]
    fn msaa_clip_stack_reuses_only_the_same_path_snapshot() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let clip = rect_path([8.0, 8.0, 56.0, 56.0], FillRule::NonZero);
        let equivalent_geometry = rect_path([8.0, 8.0, 56.0, 56.0], FillRule::NonZero);
        let mut frame = factory.begin_frame(0xff00_0000);

        frame.save();
        frame.clip_path(&clip);
        let (initial, initial_id) = frame.prepare_scheduled_clip_updates().unwrap();
        assert_eq!(initial.len(), 1);
        assert_eq!(initial_id, 1);
        frame.restore();

        let (unclipped, clip_id) = frame.prepare_scheduled_clip_updates().unwrap();
        assert_eq!(clip_id, 0);
        assert!(unclipped.is_empty());
        assert_eq!(frame.msaa_path_clip_id, 1);
        assert_eq!(frame.msaa_path_clips.len(), 1);

        frame.save();
        frame.clip_path(&clip);
        let (reentered, reentered_id) = frame.prepare_scheduled_clip_updates().unwrap();
        assert_eq!(reentered_id, 1);
        assert!(reentered.is_empty());
        frame.restore();

        frame.save();
        frame.clip_path(&equivalent_geometry);
        let (replaced, replaced_id) = frame.prepare_scheduled_clip_updates().unwrap();
        assert_eq!(replaced_id, 2);
        assert!(matches!(
            replaced.as_slice(),
            [
                SolidDraw {
                    role: DrawRole::ClipReset {
                        action: MsaaClipResetAction::ClearPrevious,
                        ..
                    },
                    ..
                },
                SolidDraw {
                    role: DrawRole::ClipUpdate {
                        replacement_id: 2,
                        parent_id: 0
                    },
                    ..
                }
            ]
        ));
    }

    #[test]
    fn axis_aligned_clip_path_limits_atomic_fill_pixels() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let clip = rect_path([16.0, 16.0, 48.0, 48.0], FillRule::NonZero);
        let fill = rect_path([0.0, 0.0, 64.0, 64.0], FillRule::Clockwise);
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.clip_path(&clip);
        frame.draw_path(&fill, &paint);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(8, 8), [0, 0, 0, 255]);
        assert_eq!(pixel(32, 32), [255, 255, 255, 255]);
        assert_eq!(pixel(56, 56), [0, 0, 0, 255]);
    }

    #[test]
    fn msaa_axis_aligned_clip_path_uses_stencil_coverage() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let clip = rect_path([16.0, 16.0, 48.0, 48.0], FillRule::NonZero);
        let fill = rect_path([0.0, 0.0, 64.0, 64.0], FillRule::Clockwise);
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.clip_path(&clip);
        frame.draw_path(&fill, &paint);
        assert!(frame.state.clip_rect.is_none());
        assert_eq!(frame.state.clip_stack_height, 1);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(8, 8), [0, 0, 0, 255]);
        assert_eq!(pixel(32, 32), [255, 255, 255, 255]);
        assert_eq!(pixel(56, 56), [0, 0, 0, 255]);
    }

    #[test]
    fn msaa_direct_fill_reads_destination_for_advanced_blending() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let fill = rect_path([16.0, 16.0, 48.0, 48.0], FillRule::Clockwise);
        let paint = WgpuPaint {
            color: 0xff00_00ff,
            blend_mode: BlendMode::Multiply,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_ff00);
        frame.draw_path(&fill, &paint);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(8, 8), [0, 255, 0, 255]);
        assert_eq!(pixel(32, 32), [0, 0, 0, 255]);
        assert_eq!(pixel(56, 56), [0, 255, 0, 255]);
    }

    #[test]
    fn msaa_direct_fill_samples_the_generated_gradient_ramp() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let fill = rect_path([0.0, 0.0, 64.0, 64.0], FillRule::Clockwise);
        let paint = WgpuPaint {
            shader: Some(WgpuShader::Linear {
                start: (0.0, 0.0),
                end: (64.0, 0.0),
                colors: vec![0xffff_0000, 0xff00_00ff],
                stops: vec![0.0, 1.0],
            }),
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.draw_path(&fill, &paint);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];
        let left = pixel(8, 32);
        let center = pixel(32, 32);
        let right = pixel(56, 32);

        assert_eq!(left[3], 255);
        assert_eq!(center[3], 255);
        assert_eq!(right[3], 255);
        assert!(left[0] > left[2], "left gradient sample: {left:?}");
        assert!(right[2] > right[0], "right gradient sample: {right:?}");
        assert!(
            center[0] > 64 && center[2] > 64,
            "center sample: {center:?}"
        );
    }

    #[test]
    fn msaa_stroke_depth_rejects_duplicate_contour_self_overdraw() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let make_path = |contour_count| {
            let mut raw_path = RawPath::new();
            for _ in 0..contour_count {
                raw_path.move_to(8.0, 32.0);
                raw_path.line_to(56.0, 32.0);
            }
            WgpuPath {
                valid: true,
                raw_path,
                fill_rule: FillRule::NonZero,
            }
        };
        let single = make_path(1);
        let duplicate = make_path(2);
        let paint = WgpuPaint {
            color: 0x80ff_0000,
            style: RenderPaintStyle::Stroke,
            thickness: 16.0,
            ..WgpuPaint::default()
        };
        let render = |path: &WgpuPath| {
            let mut frame = factory.begin_frame(0xff00_0000);
            frame.draw_path(path, &paint);
            frame.finish().unwrap()
        };

        assert_eq!(render(&single), render(&duplicate));
    }

    #[test]
    fn msaa_direct_advanced_blends_preserve_order_and_skip_empty_copies() {
        let msaa_factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let atomic_factory =
            WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let base = rect_path([4.0, 4.0, 60.0, 60.0], FillRule::NonZero);
        let multiply = rect_path([12.0, 12.0, 52.0, 52.0], FillRule::EvenOdd);
        let luminosity = rect_path([24.0, 24.0, 56.0, 56.0], FillRule::NonZero);
        let offscreen = rect_path([80.0, 80.0, 96.0, 96.0], FillRule::Clockwise);
        let mut stroke_path = RawPath::new();
        stroke_path.move_to(8.0, 32.0);
        stroke_path.line_to(56.0, 32.0);
        let stroke = WgpuPath {
            valid: true,
            raw_path: stroke_path,
            fill_rule: FillRule::NonZero,
        };
        let base_paint = WgpuPaint {
            color: 0xff20_80c0,
            ..WgpuPaint::default()
        };
        let multiply_paint = WgpuPaint {
            color: 0x80c0_4020,
            blend_mode: BlendMode::Multiply,
            ..WgpuPaint::default()
        };
        let luminosity_paint = WgpuPaint {
            color: 0xa040_c080,
            blend_mode: BlendMode::Luminosity,
            ..WgpuPaint::default()
        };
        let stroke_paint = WgpuPaint {
            color: 0x80ff_2000,
            style: RenderPaintStyle::Stroke,
            thickness: 8.0,
            blend_mode: BlendMode::Screen,
            ..WgpuPaint::default()
        };
        let render = |factory: &WgpuFactory, include_offscreen| {
            let mut frame = factory.begin_frame(0xff04_080c);
            frame.draw_path(&base, &base_paint);
            frame.draw_path(&multiply, &multiply_paint);
            frame.draw_path(&luminosity, &luminosity_paint);
            frame.draw_path(&stroke, &stroke_paint);
            if include_offscreen {
                frame.draw_path(&offscreen, &luminosity_paint);
            }
            frame.finish().unwrap()
        };

        let msaa = render(&msaa_factory, false);
        let msaa_with_offscreen = render(&msaa_factory, true);
        assert_eq!(msaa_with_offscreen, msaa);

        let atomic = render(&atomic_factory, false);
        for [x, y] in [[8, 8], [16, 16], [32, 32], [48, 48]] {
            let offset = (y * 64 + x) * 4;
            let msaa_pixel = &msaa[offset..offset + 4];
            let atomic_pixel = &atomic[offset..offset + 4];
            assert!(
                msaa_pixel
                    .iter()
                    .zip(atomic_pixel)
                    .all(|(left, right)| left.abs_diff(*right) <= 2),
                "at ({x}, {y}): msaa={msaa_pixel:?} atomic={atomic_pixel:?}"
            );
        }
    }

    #[test]
    fn msaa_gradient_destination_reads_restart_with_loaded_depth() {
        use nuxie_render_stream::RenderStream;

        let stream = RenderStream::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/renderer/streams/gm/xfermodes2.rive-stream"
        )))
        .unwrap();
        let (width, height) = stream.frame_size.unwrap();
        let mut factory = WgpuFactory::new_with_mode(width, height, RenderMode::Msaa).unwrap();
        let mut frame = factory.begin_frame(stream.clear_color.unwrap_or(0));
        stream.replay_frame(0, &mut factory, &mut frame).unwrap();
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * width as usize + x) * 4..][..4];

        // These destination-read gradient cells require the C++ WebGPU restart
        // to Load the prior MSAA depth/stencil attachments after its copy.
        assert_eq!(pixel(306, 83), [63, 0, 0, 255]);
        assert_eq!(pixel(332, 20), [1, 22, 0, 181]);
        assert_eq!(pixel(443, 24), [0, 121, 0, 254]);
    }

    #[test]
    fn axis_aligned_clip_path_limits_clockwise_atomic_compound_fill_pixels() {
        let factory = WgpuFactory::new_with_mode(128, 64, RenderMode::ClockwiseAtomic).unwrap();
        let clip = rect_path([32.0, 0.0, 96.0, 64.0], FillRule::NonZero);
        let mut fill = rect_path([0.0, 0.0, 128.0, 64.0], FillRule::NonZero);
        fill.raw_path.add_path(
            &rect_path([16.0, 8.0, 112.0, 56.0], FillRule::NonZero).raw_path,
            Mat2D::IDENTITY,
        );
        let paint = WgpuPaint {
            color: 0xffff_0000,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.clip_path(&clip);
        frame.draw_path(&fill, &paint);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 128 + x) * 4..][..4];

        assert_eq!(pixel(16, 32), [0, 0, 0, 255]);
        assert_eq!(pixel(64, 32), [255, 0, 0, 255]);
        assert_eq!(pixel(112, 32), [0, 0, 0, 255]);
    }

    #[test]
    fn clockwise_atomic_compound_fill_applies_advanced_blend_through_clip_rect() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let clip = rect_path([0.0, 0.0, 48.0, 64.0], FillRule::NonZero);
        let mut fill = rect_path([8.0, 8.0, 28.0, 56.0], FillRule::NonZero);
        fill.raw_path.add_path(
            &rect_path([36.0, 8.0, 56.0, 56.0], FillRule::NonZero).raw_path,
            Mat2D::IDENTITY,
        );
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            blend_mode: BlendMode::Overlay,
            ..WgpuPaint::default()
        };
        let base_paint = WgpuPaint {
            color: 0xff80_4000,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xffff_6600);
        frame.clip_path(&clip);
        frame.draw_path(&fill, &base_paint);
        frame.draw_path(&fill, &paint);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(16, 32), [255, 128, 0, 255]);
        assert_eq!(pixel(40, 32), [255, 128, 0, 255]);
        assert_eq!(pixel(52, 32), [255, 102, 0, 255]);
    }

    #[test]
    fn clockwise_atomic_advanced_blends_match_generated_atomic_shader() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let simple = rect_path([8.0, 8.0, 28.0, 56.0], FillRule::NonZero);
        let mut compound = simple.clone();
        compound.raw_path.add_path(
            &rect_path([36.0, 8.0, 56.0, 56.0], FillRule::NonZero).raw_path,
            Mat2D::IDENTITY,
        );
        for blend_mode in [
            BlendMode::Screen,
            BlendMode::Overlay,
            BlendMode::Darken,
            BlendMode::Lighten,
            BlendMode::ColorDodge,
            BlendMode::ColorBurn,
            BlendMode::HardLight,
            BlendMode::SoftLight,
            BlendMode::Difference,
            BlendMode::Exclusion,
            BlendMode::Multiply,
            BlendMode::Hue,
            BlendMode::Saturation,
            BlendMode::Color,
            BlendMode::Luminosity,
        ] {
            let render = |path: &WgpuPath| {
                let paint = WgpuPaint {
                    color: 0x80c0_4020,
                    blend_mode,
                    ..WgpuPaint::default()
                };
                let mut frame = factory.begin_frame(0xff20_80c0);
                frame.draw_path(path, &paint);
                let pixels = frame.finish().unwrap();
                <[u8; 4]>::try_from(&pixels[(16 * 64 + 16) * 4..][..4]).unwrap()
            };

            let clockwise = render(&compound);
            let generic = render(&simple);
            assert!(
                clockwise
                    .iter()
                    .zip(generic)
                    .all(|(left, right)| left.abs_diff(right) <= 1),
                "{blend_mode:?}: clockwise={clockwise:?} generic={generic:?}"
            );
        }
    }

    #[test]
    fn advanced_clockwise_atomic_path_clip_returns_unsupported() {
        let factory = WgpuFactory::new_with_mode(1600, 1600, RenderMode::ClockwiseAtomic).unwrap();
        let clip = negative_interior_checkerboard();
        let fill = rect_path([0.0, 0.0, 1600.0, 1600.0], FillRule::Clockwise);
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            blend_mode: BlendMode::Overlay,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xffff_6600);
        frame.clip_path(&clip);
        frame.draw_path(&fill, &paint);

        assert_eq!(
            frame.finish().unwrap_err().to_string(),
            "unsupported renderer feature: advanced clockwise-atomic blending with path clips"
        );
    }

    #[test]
    fn clockwise_atomic_global_fill_runs_borrowed_then_main_passes() {
        let factory = WgpuFactory::new_with_mode(640, 640, RenderMode::ClockwiseAtomic).unwrap();
        let mut compound = rect_path([20.0, 20.0, 620.0, 620.0], FillRule::NonZero);
        compound.raw_path.add_path(
            &rect_path([200.0, 200.0, 440.0, 440.0], FillRule::NonZero).raw_path,
            Mat2D::IDENTITY,
        );
        let red = WgpuPaint {
            color: 0xffff_0000,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.draw_path(&compound, &red);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 640 + x) * 4..][..4];

        assert_eq!(pixel(10, 10), [0, 0, 0, 255]);
        assert_eq!(pixel(100, 100), [255, 0, 0, 255]);
        assert_eq!(pixel(320, 320), [255, 0, 0, 255]);
    }

    #[test]
    fn clockwise_atomic_outer_clip_writes_attachment_coverage() {
        let factory = WgpuFactory::new_with_mode(640, 640, RenderMode::ClockwiseAtomic).unwrap();
        let mut clip = rect_path([40.0, 40.0, 600.0, 600.0], FillRule::NonZero);
        clip.raw_path.add_path(
            &rect_path([200.0, 200.0, 440.0, 440.0], FillRule::NonZero).raw_path,
            Mat2D::IDENTITY,
        );
        let prepared_clip = draw::build_interior_tessellation(
            &clip.raw_path,
            Mat2D::IDENTITY,
            clip.fill_rule,
            true,
        )
        .unwrap();
        assert!(prepared_clip
            .triangles
            .iter()
            .any(|vertex| vertex.weight_path_id >> 16 > 0));
        let fill = rect_path([0.0, 0.0, 640.0, 640.0], FillRule::Clockwise);
        let red = WgpuPaint {
            color: 0xffff_0000,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.clip_path(&clip);
        frame.draw_path(&fill, &red);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 640 + x) * 4..][..4];

        assert_eq!(pixel(20, 20), [0, 0, 0, 255]);
        assert_eq!(pixel(100, 100), [255, 0, 0, 255]);
        assert_eq!(pixel(320, 320), [255, 0, 0, 255]);
    }

    #[test]
    fn clockwise_atomic_nested_clip_erases_the_inverse_path() {
        let factory = WgpuFactory::new_with_mode(640, 640, RenderMode::ClockwiseAtomic).unwrap();
        let mut outer = rect_path([40.0, 40.0, 600.0, 600.0], FillRule::NonZero);
        outer.raw_path.add_path(
            &rect_path([80.0, 80.0, 560.0, 560.0], FillRule::NonZero).raw_path,
            Mat2D::IDENTITY,
        );
        let inner = rect_path([200.0, 200.0, 440.0, 440.0], FillRule::NonZero);
        let fill = rect_path([0.0, 0.0, 640.0, 640.0], FillRule::Clockwise);
        let red = WgpuPaint {
            color: 0xffff_0000,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.clip_path(&outer);
        frame.clip_path(&inner);
        frame.draw_path(&fill, &red);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 640 + x) * 4..][..4];

        assert_eq!(pixel(20, 20), [0, 0, 0, 255]);
        assert_eq!(pixel(100, 100), [0, 0, 0, 255]);
        assert_eq!(pixel(320, 320), [255, 0, 0, 255]);
    }

    #[test]
    fn clockwise_atomic_nested_interior_clip_culls_counterclockwise_faces() {
        let factory = WgpuFactory::new_with_mode(1600, 1600, RenderMode::ClockwiseAtomic).unwrap();
        let checkerboard = negative_interior_checkerboard();
        let clip = negative_interior_path();
        let fill = rect_path([0.0, 0.0, 1600.0, 1600.0], FillRule::Clockwise);
        let red = WgpuPaint {
            color: 0xffff_0000,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_ffff);
        frame.clip_path(&checkerboard);
        frame.transform(Mat2D([1.0, 0.0, 0.0, 1.0, 29.0, -100.0]));
        frame.clip_path(&clip);
        frame.draw_path(&fill, &red);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 1600 + x) * 4..][..4];

        assert_eq!(pixel(1074, 116), [255, 0, 0, 255]);
        assert_eq!(pixel(1331, 103), [255, 0, 0, 255]);
        assert_eq!(pixel(939, 302), [255, 0, 0, 255]);
    }

    fn coverage_word_index(range: gpu::CoverageBufferRange, x: u32, y: u32) -> usize {
        let x = (x as f32 + range.offset_x).floor() as u32;
        let y = (y as f32 + range.offset_y).floor() as u32;
        (range.offset
            + (y >> 5) * (range.pitch << 5)
            + (x >> 5) * 1024
            + ((x & 28) << 5)
            + ((y & 28) << 2)
            + ((y & 3) << 2)
            + (x & 3)) as usize
    }

    fn coverage_word_at(words: &[u32], range: gpu::CoverageBufferRange, x: u32, y: u32) -> u32 {
        words[coverage_word_index(range, x, y)]
    }

    #[test]
    fn captures_clockwise_atomic_coverage_between_borrowed_and_main_passes() {
        let factory = WgpuFactory::new_with_mode(1600, 1600, RenderMode::ClockwiseAtomic).unwrap();
        let checkerboard = negative_interior_checkerboard();
        let clip = negative_interior_path();
        let fill = rect_path([0.0, 0.0, 1600.0, 1600.0], FillRule::Clockwise);
        let mut frame = factory.begin_frame(0xff00_ffff);
        frame.clip_path(&checkerboard);
        for (transform, color) in [
            (Mat2D([1.0, 0.0, 0.0, 1.0, 29.0, -100.0]), 0xffff_0000),
            (Mat2D([-1.0, 0.0, 0.0, 1.0, 1593.0, 207.0]), 0xd090_0000),
        ] {
            frame.save();
            frame.transform(transform);
            frame.clip_path(&clip);
            frame.draw_path(
                &fill,
                &WgpuPaint {
                    color,
                    ..WgpuPaint::default()
                },
            );
            frame.restore();
        }
        let (pixels, captures) = frame.finish_with_clockwise_atomic_coverage().unwrap();
        assert_eq!(captures.len(), 1);
        let capture = &captures[0];
        let nested = capture
            .kinds
            .iter()
            .enumerate()
            .filter_map(|(index, kind)| {
                (*kind == clockwise_atomic_pipeline::ClockwiseAtomicDrawKind::NestedClip)
                    .then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(nested.len(), 2);
        assert_eq!(capture.clip_updates.len(), 2);
        for ((index, point), clip_update) in nested
            .into_iter()
            .zip([(1074, 116), (238, 430)])
            .zip(&capture.clip_updates)
        {
            let range = capture.ranges[index];
            let clip_index =
                point.1 as usize * capture.clip_bytes_per_row as usize + point.0 as usize * 4;
            assert_eq!(
                coverage_word_at(&capture.borrowed, range, point.0, point.1),
                0x13f800
            );
            assert_eq!(
                coverage_word_at(&capture.main, range, point.0, point.1),
                0x140000
            );
            assert_eq!(&clip_update[clip_index..clip_index + 4], [255; 4]);
        }
        let pixel = |x: usize, y: usize| &pixels[(y * 1600 + x) * 4..][..4];
        assert_eq!(pixel(1074, 116), [255, 0, 0, 255]);
        assert_eq!(pixel(238, 430), [117, 47, 47, 255]);
    }

    #[test]
    fn nested_clip_probe_uses_sampled_clockwise_atomic_plane() {
        let factory = WgpuFactory::new_with_mode(640, 640, RenderMode::ClockwiseAtomic).unwrap();
        let path = |contours: &[&[[f32; 2]]]| {
            let mut raw_path = RawPath::new();
            for contour in contours {
                raw_path.move_to(contour[0][0], contour[0][1]);
                for point in &contour[1..] {
                    raw_path.line_to(point[0], point[1]);
                }
                raw_path.close();
            }
            WgpuPath {
                valid: true,
                raw_path,
                fill_rule: FillRule::Clockwise,
            }
        };
        let outer = path(&[
            &[
                [40.0, 60.0],
                [600.0, 60.0],
                [600.0, 280.0],
                [380.0, 280.0],
                [380.0, 600.0],
                [40.0, 600.0],
            ],
            &[
                [420.0, 420.0],
                [580.0, 420.0],
                [580.0, 580.0],
                [420.0, 580.0],
            ],
        ]);
        let nested = path(&[&[
            [140.0, 160.0],
            [520.0, 160.0],
            [520.0, 520.0],
            [440.0, 520.0],
            [440.0, 320.0],
            [300.0, 320.0],
            [300.0, 520.0],
            [140.0, 520.0],
        ]]);
        let fill = rect_path([0.0, 0.0, 640.0, 640.0], FillRule::Clockwise);
        let mut frame = factory.begin_frame(0x0000_0000);
        frame.clip_path(&outer);
        frame.clip_path(&nested);
        frame.draw_path(
            &fill,
            &WgpuPaint {
                color: 0xffff_ffff,
                ..WgpuPaint::default()
            },
        );

        let (pixels, captures) = frame.finish_with_clockwise_atomic_coverage().unwrap();
        assert_eq!(captures.len(), 1);
        assert_eq!(
            captures[0].kinds,
            [
                clockwise_atomic_pipeline::ClockwiseAtomicDrawKind::OutermostClip,
                clockwise_atomic_pipeline::ClockwiseAtomicDrawKind::NestedClip,
                clockwise_atomic_pipeline::ClockwiseAtomicDrawKind::ClippedContent,
            ]
        );
        assert_eq!(captures[0].clip_updates.len(), 1);
        assert_eq!(captures[0].clip_bytes_per_row, 640 * 4);
        assert_eq!(captures[0].clip_updates[0], pixels);
    }

    #[test]
    fn arbitrary_clip_path_updates_atomic_clip_buffer() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let mut clip_path = RawPath::new();
        clip_path.move_to(8.0, 8.0);
        clip_path.line_to(56.0, 8.0);
        clip_path.line_to(32.0, 56.0);
        clip_path.close();
        let clip = WgpuPath {
            valid: true,
            raw_path: clip_path,
            fill_rule: FillRule::NonZero,
        };
        let fill = rect_path([0.0, 0.0, 64.0, 64.0], FillRule::Clockwise);
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.clip_path(&clip);
        frame.draw_path(&fill, &paint);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(4, 4), [0, 0, 0, 255]);
        assert_eq!(pixel(32, 24), [255, 255, 255, 255]);
        assert_eq!(pixel(60, 60), [0, 0, 0, 255]);
    }

    #[test]
    fn direct_feather_path_clip_preserves_authored_fill_rules() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let clip = triangle_path([[4.0, 4.0], [60.0, 4.0], [32.0, 60.0]], FillRule::NonZero);
        let render = |fill_rule| {
            let fill = rect_path([8.0, 8.0, 56.0, 56.0], fill_rule);
            let mut frame = factory.begin_frame(0xff00_0000);
            frame.clip_path(&clip);
            frame.draw_path(
                &fill,
                &WgpuPaint {
                    color: 0xffff_ffff,
                    feather: 1.0,
                    ..WgpuPaint::default()
                },
            );
            frame.finish().unwrap()
        };
        let pixel = |pixels: &[u8], x: usize, y: usize| {
            <[u8; 4]>::try_from(&pixels[(y * 64 + x) * 4..][..4]).unwrap()
        };

        for fill_rule in [FillRule::NonZero, FillRule::EvenOdd, FillRule::Clockwise] {
            assert_eq!(
                atomic_paint_fill_rule(fill_rule, false),
                fill_rule,
                "generic atomic paint must retain its authored rule"
            );
            assert_eq!(
                atomic_paint_fill_rule(fill_rule, true),
                FillRule::Clockwise,
                "the dedicated clockwise batch uses the frame override"
            );
            let pixels = render(fill_rule);
            assert_eq!(pixel(&pixels, 20, 20), [255, 255, 255, 255]);
            assert_eq!(pixel(&pixels, 32, 32), [255, 255, 255, 255]);
            assert_eq!(pixel(&pixels, 4, 60), [0, 0, 0, 255]);
        }
    }

    #[test]
    fn arbitrary_clip_applies_to_atomic_gradient_content() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let mut clip_path = RawPath::new();
        clip_path.move_to(8.0, 8.0);
        clip_path.line_to(56.0, 8.0);
        clip_path.line_to(32.0, 56.0);
        clip_path.close();
        let clip = WgpuPath {
            valid: true,
            raw_path: clip_path,
            fill_rule: FillRule::NonZero,
        };
        let fill = rect_path([0.0, 0.0, 64.0, 64.0], FillRule::Clockwise);
        let paint = WgpuPaint {
            shader: Some(WgpuShader::Linear {
                start: (0.0, 0.0),
                end: (64.0, 64.0),
                colors: vec![0xffff_ffff, 0xff00_0000],
                stops: vec![0.0, 1.0],
            }),
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.clip_path(&clip);
        frame.draw_path(&fill, &paint);

        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(4, 4), [0, 0, 0, 255]);
        let inside = pixel(32, 24);
        assert_eq!(inside[3], 255);
        assert_eq!(inside[0], inside[1]);
        assert_eq!(inside[1], inside[2]);
        assert!(inside[0] > 64 && inside[0] < 224);
        assert_eq!(pixel(60, 60), [0, 0, 0, 255]);
    }

    #[test]
    fn sequential_root_clips_do_not_reuse_stale_clip_coverage() {
        fn triangle(points: [[f32; 2]; 3]) -> WgpuPath {
            let mut path = RawPath::new();
            path.move_to(points[0][0], points[0][1]);
            path.line_to(points[1][0], points[1][1]);
            path.line_to(points[2][0], points[2][1]);
            path.close();
            WgpuPath {
                valid: true,
                raw_path: path,
                fill_rule: FillRule::NonZero,
            }
        }

        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let fill = rect_path([0.0, 0.0, 64.0, 64.0], FillRule::Clockwise);
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.save();
        frame.clip_path(&triangle([[32.0, 32.0], [60.0, 32.0], [60.0, 60.0]]));
        frame.draw_path(
            &fill,
            &WgpuPaint {
                color: 0xffff_0000,
                ..WgpuPaint::default()
            },
        );
        frame.restore();
        frame.save();
        frame.clip_path(&triangle([[4.0, 4.0], [28.0, 4.0], [4.0, 28.0]]));
        frame.draw_path(
            &fill,
            &WgpuPaint {
                color: 0xff00_ffff,
                ..WgpuPaint::default()
            },
        );
        frame.restore();

        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];
        assert_eq!(pixel(10, 10), [0, 255, 255, 255]);
        assert_eq!(pixel(50, 40), [255, 0, 0, 255]);
        assert_eq!(pixel(30, 30), [0, 0, 0, 255]);
    }

    #[test]
    fn nested_arbitrary_clips_intersect_in_atomic_clip_buffer() {
        fn diamond(radius: f32) -> WgpuPath {
            let mut path = RawPath::new();
            path.move_to(32.0, 32.0 - radius);
            path.line_to(32.0 + radius, 32.0);
            path.line_to(32.0, 32.0 + radius);
            path.line_to(32.0 - radius, 32.0);
            path.close();
            WgpuPath {
                valid: true,
                raw_path: path,
                fill_rule: FillRule::NonZero,
            }
        }

        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let fill = rect_path([0.0, 0.0, 64.0, 64.0], FillRule::Clockwise);
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.clip_path(&diamond(28.0));
        frame.clip_path(&diamond(12.0));
        frame.draw_path(&fill, &paint);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(32, 32), [255, 255, 255, 255]);
        assert_eq!(pixel(12, 32), [0, 0, 0, 255]);
        assert_eq!(pixel(2, 32), [0, 0, 0, 255]);
    }

    #[test]
    fn arbitrary_clip_stack_reuses_restored_elements() {
        fn triangle(offset: f32) -> WgpuPath {
            let mut path = RawPath::new();
            path.move_to(offset, offset);
            path.line_to(60.0 - offset, offset);
            path.line_to(32.0, 60.0 - offset);
            path.close();
            WgpuPath {
                valid: true,
                raw_path: path,
                fill_rule: FillRule::NonZero,
            }
        }

        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let outer = triangle(4.0);
        let inner = triangle(16.0);
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.clip_path(&outer);
        frame.save();
        frame.clip_path(&inner);
        assert_eq!(frame.state.clip_stack_height, 2);
        assert_eq!(frame.clips.len(), 2);

        frame.restore();
        frame.save();
        frame.clip_path(&inner);
        assert_eq!(frame.state.clip_stack_height, 2);
        assert_eq!(frame.clips.len(), 2);
    }

    #[test]
    fn repeated_draws_reapply_the_same_arbitrary_clip_stack() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let mut clip_path = RawPath::new();
        clip_path.move_to(4.0, 4.0);
        clip_path.line_to(60.0, 4.0);
        clip_path.line_to(32.0, 60.0);
        clip_path.close();
        let clip = WgpuPath {
            valid: true,
            raw_path: clip_path,
            fill_rule: FillRule::NonZero,
        };
        let full = rect_path([0.0, 0.0, 64.0, 64.0], FillRule::Clockwise);
        let inset = rect_path([24.0, 20.0, 40.0, 44.0], FillRule::Clockwise);
        let red = WgpuPaint {
            color: 0xffff_0000,
            ..WgpuPaint::default()
        };
        let green = WgpuPaint {
            color: 0xff00_ff00,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.clip_path(&clip);
        frame.draw_path(&full, &red);
        frame.draw_path(&inset, &green);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(32, 12), [255, 0, 0, 255]);
        assert_eq!(pixel(32, 32), [0, 255, 0, 255]);
        assert_eq!(pixel(2, 2), [0, 0, 0, 255]);
    }

    #[test]
    fn large_clip_uses_global_interior_triangulation() {
        let factory = WgpuFactory::new_with_mode(640, 480, RenderMode::ClockwiseAtomic).unwrap();
        let mut clip_path = RawPath::new();
        clip_path.move_to(-469_515.0, -10_354_890.0);
        clip_path.cubic_to(
            771_919.625,
            -10_411_179.0,
            2_013_360.125,
            -10_243_774.0,
            3_195_542.75,
            -9_860_664.0,
        );
        clip_path.line_to(3_195_550.0, -9_860_655.0);
        clip_path.line_to(3_195_539.0, -9_860_652.0);
        clip_path.line_to(3_195_539.0, -9_860_652.0);
        clip_path.line_to(3_195_539.0, -9_860_652.0);
        clip_path.cubic_to(
            2_013_358.125,
            -10_243_761.0,
            771_919.25,
            -10_411_166.0,
            -469_513.844,
            -10_354_877.0,
        );
        clip_path.line_to(-469_515.0, -10_354_890.0);
        clip_path.close();
        let clip = WgpuPath {
            valid: true,
            raw_path: clip_path,
            fill_rule: FillRule::NonZero,
        };
        assert!(draw::should_use_interior_tessellation(
            &clip.raw_path,
            Mat2D([1.0, 0.0, 0.0, 1.0, 258.0, 10_365_663.0])
        ));
        assert!(draw::build_interior_tessellation(
            &clip.raw_path,
            Mat2D([1.0, 0.0, 0.0, 1.0, 258.0, 10_365_663.0]),
            FillRule::NonZero,
            true,
        )
        .is_some());

        let fill = rect_path([-1.0e9, -1.0e9, 1.0e9, 1.0e9], FillRule::NonZero);
        let mut frame = factory.begin_frame(0xffff_ffff);
        frame.transform(Mat2D([1.0, 0.0, 0.0, 1.0, 258.0, 10_365_663.0]));
        frame.clip_path(&clip);
        frame.draw_path(&fill, &WgpuPaint::default());

        assert!(frame.finish().is_ok());
    }

    #[test]
    fn resolved_fallback_composites_with_premultiplied_src_over() {
        let factory = WgpuFactory::new_with_mode(2, 2, RenderMode::ClockwiseAtomic).unwrap();
        let source = factory
            .context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("nuxie-composite-test-source"),
                size: wgpu::Extent3d {
                    width: 2,
                    height: 2,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
        factory.context.queue.write_counted_texture(
            source.as_image_copy(),
            &[128, 0, 0, 128].repeat(4),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(8),
                rows_per_image: Some(2),
            },
            source.size(),
        );
        let target = factory
            .context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("nuxie-composite-test-target"),
                size: source.size(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
        let target_view = target.create_view(&Default::default());
        let mut encoder = factory.context.device.create_counted_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("nuxie-composite-test-encoder"),
            },
        );
        {
            let attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let _pass = encoder.begin_counted_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("nuxie-composite-test-clear"),
                color_attachments: &attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        factory.context.composite_pipeline.encode(
            &factory.context.device,
            &mut encoder,
            &target_view,
            &source.create_view(&Default::default()),
        );
        let readback = factory
            .context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("nuxie-composite-test-readback"),
                size: 512,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256),
                    rows_per_image: Some(2),
                },
            },
            target.size(),
        );
        factory.context.queue.submit_counted(Some(encoder.finish()));
        let slice = readback.slice(..);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        factory
            .context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        receiver.recv().unwrap().unwrap();
        let mapped = slice.get_mapped_range().unwrap();
        assert_eq!(&mapped[..4], &[128, 0, 127, 255]);
    }

    #[test]
    fn atomic_and_fallback_runs_preserve_draw_order() {
        let factory = WgpuFactory::new_with_mode(32, 32, RenderMode::ClockwiseAtomic).unwrap();
        let red = WgpuPaint {
            color: 0xffff_0000,
            ..WgpuPaint::default()
        };
        let green = WgpuPaint {
            color: 0xff00_ff00,
            ..WgpuPaint::default()
        };
        let blue = WgpuPaint {
            color: 0xff00_00ff,
            ..WgpuPaint::default()
        };
        let background = rect_path([1.0, 1.0, 31.0, 31.0], FillRule::NonZero);
        let mut compound = rect_path([4.0, 4.0, 28.0, 28.0], FillRule::EvenOdd);
        compound.raw_path.add_path(
            &rect_path([10.0, 10.0, 22.0, 22.0], FillRule::EvenOdd).raw_path,
            Mat2D::IDENTITY,
        );
        let foreground = rect_path([16.0, 16.0, 30.0, 30.0], FillRule::NonZero);
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.draw_path(&background, &red);
        frame.draw_path(&compound, &green);
        frame.draw_path(&foreground, &blue);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 32 + x) * 4..][..4];

        assert_eq!(pixel(2, 2), [255, 0, 0, 255]);
        assert_eq!(pixel(6, 6), [0, 255, 0, 255]);
        assert_eq!(pixel(12, 12), [0, 255, 0, 255]);
        assert_eq!(pixel(18, 18), [0, 0, 255, 255]);
    }

    #[test]
    fn generated_upstream_wgsl_validates_with_naga() {
        let generated = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/generated");
        let mut modules = fs::read_dir(&generated)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("wgsl"))
            .collect::<Vec<_>>();
        modules.sort();
        assert!(!modules.is_empty(), "no generated WGSL modules found");
        for path in modules {
            let source = fs::read_to_string(&path).unwrap();
            let module = naga::front::wgsl::parse_str(&source)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::all(),
            )
            .validate(&module)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        }
    }

    #[test]
    fn upstream_tessellation_pass_writes_across_texture_rows() {
        let factory = WgpuFactory::new(64, 64).unwrap();
        let mut uniforms = gpu::FlushUniforms::zeroed();
        uniforms.inverse_viewports[1] = -1.0;
        let points = [[4.0, 4.0], [20.0, 4.0], [44.0, 4.0], [60.0, 4.0]];
        let first = gpu::TessVertexSpan::without_reflection(
            points,
            [1.0, 0.0],
            0.0,
            2046,
            2052,
            1,
            0,
            1,
            1,
        );
        let second =
            gpu::TessVertexSpan::without_reflection(points, [1.0, 0.0], 1.0, -2, 4, 1, 0, 1, 1);
        let paths = [gpu::PathData::zeroed()];
        let contours = [gpu::ContourData::new([32.0, 4.0], 0, 0)];
        let mut encoder = factory.context.device.create_counted_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("nuxie-tessellation-test-encoder"),
            },
        );
        let mut tessellation_uploads = factory
            .context
            .tessellator
            .begin_frame_uploads(&factory.context.device);
        let texture = factory.context.tessellator.encode(
            &factory.context.device,
            &mut tessellation_uploads,
            &mut encoder,
            &factory.context.feather_lut.view,
            &[first, second],
            &uniforms,
            &paths,
            &contours,
            2,
        );
        let bytes_per_row = gpu::TESS_TEXTURE_WIDTH as u32 * 16;
        let readback = factory
            .context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("nuxie-tessellation-test-readback"),
                size: u64::from(bytes_per_row) * 2,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(2),
                },
            },
            texture.size(),
        );
        tessellation_uploads.flush(&factory.context.queue);
        factory.context.queue.submit_counted(Some(encoder.finish()));
        let slice = readback.slice(..);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        factory
            .context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        receiver.recv().unwrap().unwrap();
        let mapped = slice.get_mapped_range().unwrap();
        let mut written = Vec::new();
        for y in 0usize..2 {
            for x in 0..gpu::TESS_TEXTURE_WIDTH {
                let flags_offset = y * bytes_per_row as usize + x as usize * 16 + 12;
                let flags =
                    u32::from_ne_bytes(mapped[flags_offset..flags_offset + 4].try_into().unwrap());
                if flags != 0 {
                    written.push((x, y));
                }
            }
        }
        assert_eq!(
            written,
            [(2046, 0), (2047, 0), (0, 1), (1, 1), (2, 1), (3, 1)]
        );
    }

    struct FixedFeatherAtlasOracle {
        mask: atlas_mask_oracle::AtlasMask,
        inputs: atlas_input_oracle::AtlasInputs,
        placement: atlas_placement_oracle::AtlasPlacement,
    }

    fn feather_cusp_large_radius_path() -> RawPath {
        let mut path = RawPath::new();
        path.move_to(0.0, 100.0);
        path.move_to(0.0, 100.0);
        path.cubic_to(90.0, 0.0, 10.0, 0.0, 100.0, 100.0);
        path
    }

    fn feather_shapes_large_radius_cusp_path() -> RawPath {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(100.0, 0.0);
        path.cubic_to(0.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        path.line_to(0.0, 100.0);
        path.cubic_to(50.0, 67.0, -50.0, 33.0, 0.0, 0.0);
        path
    }

    fn feather_grid_transform(x: f32, y: f32) -> Mat2D {
        multiply(
            Mat2D([LARGE_FEATHER_SCALE, 0.0, 0.0, LARGE_FEATHER_SCALE, 0.0, 0.0]),
            Mat2D([1.0, 0.0, 0.0, 1.0, x * 200.0 + 50.0, y * 200.0 + 50.0]),
        )
    }

    fn fixed_feather_atlas_oracle(join: StrokeJoin) -> FixedFeatherAtlasOracle {
        let paint = WgpuPaint {
            style: RenderPaintStyle::Stroke,
            thickness: ATLAS_ORACLE_STROKE_THICKNESS,
            join,
            cap: ATLAS_ORACLE_STROKE_CAP,
            feather: ATLAS_ORACLE_FEATHER,
            ..WgpuPaint::default()
        };
        let mut raw_path = RawPath::new();
        raw_path.move_to(ATLAS_ORACLE_SQUARE_MIN, ATLAS_ORACLE_SQUARE_MIN);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MAX, ATLAS_ORACLE_SQUARE_MIN);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MAX, ATLAS_ORACLE_SQUARE_MAX);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MIN, ATLAS_ORACLE_SQUARE_MAX);
        raw_path.close();
        fixed_feather_atlas_oracle_for(raw_path, paint)
    }

    fn fixed_feather_atlas_empty_stroke_oracle() -> FixedFeatherAtlasOracle {
        let paint = WgpuPaint {
            style: RenderPaintStyle::Stroke,
            thickness: ATLAS_ORACLE_STROKE_THICKNESS,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Round,
            feather: ATLAS_ORACLE_FEATHER,
            ..WgpuPaint::default()
        };
        let mut raw_path = RawPath::new();
        let center = (ATLAS_ORACLE_SQUARE_MIN + ATLAS_ORACLE_SQUARE_MAX) * 0.5;
        raw_path.move_to(center, center);
        fixed_feather_atlas_oracle_for(raw_path, paint)
    }

    fn fixed_feather_atlas_fill_oracle() -> FixedFeatherAtlasOracle {
        const CONTROL_OFFSET: f32 = 8.83064;
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            feather: ATLAS_ORACLE_FEATHER,
            ..WgpuPaint::default()
        };
        let min = ATLAS_ORACLE_SQUARE_MIN;
        let max = ATLAS_ORACLE_SQUARE_MAX;
        let center = (min + max) * 0.5;
        let mut raw_path = RawPath::new();
        raw_path.move_to(max, center);
        raw_path.cubic_to(
            max,
            center + CONTROL_OFFSET,
            center + CONTROL_OFFSET,
            max,
            center,
            max,
        );
        raw_path.cubic_to(
            center - CONTROL_OFFSET,
            max,
            min,
            center + CONTROL_OFFSET,
            min,
            center,
        );
        raw_path.cubic_to(
            min,
            center - CONTROL_OFFSET,
            center - CONTROL_OFFSET,
            min,
            center,
            min,
        );
        raw_path.cubic_to(
            center + CONTROL_OFFSET,
            min,
            max,
            center - CONTROL_OFFSET,
            max,
            center,
        );
        raw_path.close();
        fixed_feather_atlas_oracle_for(raw_path, paint)
    }

    fn fixed_feather_atlas_cusp_oracle() -> FixedFeatherAtlasOracle {
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            feather: ATLAS_ORACLE_FEATHER,
            ..WgpuPaint::default()
        };
        let mut raw_path = RawPath::new();
        raw_path.move_to(16.0, 48.0);
        raw_path.cubic_to(51.2, 16.0, 12.8, 16.0, 48.0, 48.0);
        raw_path.close();
        fixed_feather_atlas_oracle_for(raw_path, paint)
    }

    fn fixed_feather_direct_inputs(
        raw_path: RawPath,
        transform: Mat2D,
    ) -> atlas_input_oracle::AtlasInputs {
        let tessellation =
            draw::build_feather_tessellation(&raw_path, transform, 1.0, None).unwrap();
        fixed_direct_inputs_from_tessellation(
            tessellation,
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
        )
    }

    fn fixed_direct_inputs_from_tessellation(
        mut tessellation: draw::FillTessellation,
        frame_width: u32,
        frame_height: u32,
    ) -> atlas_input_oracle::AtlasInputs {
        for contour in &mut tessellation.contours {
            contour.path_id = 1;
        }
        let factory = WgpuFactory::new(frame_width, frame_height).unwrap();
        let logical_tessellation_height = draw::tessellation_texture_height(&tessellation.spans);
        // The C++ oracle exports the complete allocation, including its 125% growth tail.
        let tessellation_height = logical_tessellation_height * 5 / 4;
        let uniforms = analytic_uniforms(frame_width, frame_height, tessellation_height);
        let paths = [gpu::PathData::zeroed(), tessellation.path];
        let mut encoder = factory.context.device.create_counted_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("nuxie-direct-input-encoder"),
            },
        );
        let mut tessellation_uploads = factory
            .context
            .tessellator
            .begin_frame_uploads(&factory.context.device);
        let texture = factory.context.tessellator.encode(
            &factory.context.device,
            &mut tessellation_uploads,
            &mut encoder,
            &factory.context.feather_lut.view,
            &tessellation.spans,
            &uniforms,
            &paths,
            &tessellation.contours,
            tessellation_height,
        );
        let size = texture.size();
        let bytes_per_row = size.width.checked_mul(16).unwrap();
        let readback = factory
            .context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("nuxie-direct-input-readback"),
                size: u64::from(bytes_per_row) * u64::from(size.height),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(size.height),
                },
            },
            size,
        );
        tessellation_uploads.flush(&factory.context.queue);
        factory.context.queue.submit_counted(Some(encoder.finish()));
        let slice = readback.slice(..);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        factory
            .context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        receiver.recv().unwrap().unwrap();
        let mapped = slice.get_mapped_range().unwrap();
        let texels = mapped
            .chunks_exact(16)
            .map(|texel| {
                [
                    u32::from_le_bytes(texel[0..4].try_into().unwrap()),
                    u32::from_le_bytes(texel[4..8].try_into().unwrap()),
                    u32::from_le_bytes(texel[8..12].try_into().unwrap()),
                    u32::from_le_bytes(texel[12..16].try_into().unwrap()),
                ]
            })
            .collect();
        drop(mapped);
        readback.unmap();
        atlas_input_oracle::AtlasInputs::new(
            tessellation.base_instance,
            tessellation.instance_count,
            tessellation
                .contours
                .iter()
                .map(|contour| atlas_input_oracle::ContourRecord {
                    midpoint_x_bits: contour.midpoint[0].to_bits(),
                    midpoint_y_bits: contour.midpoint[1].to_bits(),
                    path_id: contour.path_id,
                    vertex_index0: contour.vertex_index0,
                })
                .collect(),
            size.width,
            size.height,
            texels,
        )
        .unwrap()
    }

    fn fixed_feather_direct_cusp_inputs() -> atlas_input_oracle::AtlasInputs {
        let mut raw_path = RawPath::new();
        raw_path.move_to(0.0, 100.0);
        raw_path.move_to(0.0, 100.0);
        raw_path.cubic_to(133.635864, 0.0, -33.6358566, 0.0, 100.0, 100.0);
        fixed_feather_direct_inputs(
            raw_path,
            Mat2D([1.46300006, 0.0, 0.0, 1.46300006, -40.0, -20.0]),
        )
    }

    fn fixed_feather_direct_cusp_frame() -> WgpuFrame {
        let mut raw_path = RawPath::new();
        raw_path.move_to(0.0, 100.0);
        raw_path.move_to(0.0, 100.0);
        raw_path.cubic_to(133.635864, 0.0, -33.6358566, 0.0, 100.0, 100.0);
        let path = WgpuPath {
            valid: true,
            raw_path,
            fill_rule: FillRule::Clockwise,
        };
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            feather: 1.0,
            ..WgpuPaint::default()
        };
        let factory = WgpuFactory::new_with_mode(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            RenderMode::ClockwiseAtomic,
        )
        .unwrap();
        let mut frame = factory.begin_frame(0);
        frame.transform(Mat2D([1.46300006, 0.0, 0.0, 1.46300006, -40.0, -20.0]));
        frame.draw_path(&path, &paint);
        frame
    }

    fn fixed_feather_direct_cusp_blit() -> atlas_blit_oracle::AtlasBlit {
        let frame = fixed_feather_direct_cusp_frame();
        atlas_blit_oracle::AtlasBlit::new(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            frame.finish().unwrap(),
        )
        .unwrap()
    }

    fn fixed_feather_direct_polyshark_inputs() -> atlas_input_oracle::AtlasInputs {
        use nuxie_render_stream::{Command, RenderStream};

        let stream = RenderStream::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/renderer/streams/gm/feather_polyshapes.rive-stream"
        )))
        .unwrap();
        let transform = stream.frames[0]
            .commands
            .iter()
            .find_map(|command| match command {
                Command::Transform(transform) => Some(*transform),
                _ => None,
            })
            .expect("feather_polyshapes top-level transform");
        let (path, paint) = stream.frames[0]
            .commands
            .iter()
            .filter_map(|command| match command {
                Command::DrawPath { path, paint } => Some((path, paint)),
                _ => None,
            })
            .nth(2)
            .expect("feather_polyshapes row 0 shark draw");
        assert_eq!(paint.feather, 1.0);
        fixed_feather_direct_inputs(path.raw_path.clone(), transform)
    }

    fn strokes_round_draw_38_path() -> RawPath {
        let mut path = RawPath::new();
        path.move_to(25.5016327, 70.300293);
        path.line_to(67.7646637, 70.300293);
        path.cubic_to(
            79.4274673, 70.300293, 88.8961792, 80.9101868, 88.8961792, 89.5240784,
        );
        path.line_to(88.8961792, 127.971649);
        path.cubic_to(
            88.8961792, 138.581543, 79.4274673, 147.195435, 67.7646637, 147.195435,
        );
        path.line_to(25.5016327, 147.195435);
        path.cubic_to(
            16.0329189, 147.195435, 4.37011719, 138.581543, 4.37011719, 127.971649,
        );
        path.line_to(4.37011719, 89.5240784);
        path.cubic_to(
            4.37011719, 80.9101868, 16.0329189, 70.300293, 25.5016327, 70.300293,
        );
        path.close();
        path
    }

    fn fixed_rawtext_draw_one_tessellation() -> draw::FillTessellation {
        use nuxie_render_stream::{Command, RenderStream};

        let stream = RenderStream::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/renderer/streams/gm/rawtext.rive-stream"
        )))
        .unwrap();
        assert_eq!(
            stream.frame_size,
            Some((RAWTEXT_ORACLE_FRAME_WIDTH, RAWTEXT_ORACLE_FRAME_HEIGHT))
        );
        let (path, paint) = stream.frames[0]
            .commands
            .iter()
            .find_map(|command| match command {
                Command::DrawPath { path, paint } => Some((path, paint)),
                _ => None,
            })
            .expect("rawtext draw 1");
        assert_eq!(path.fill_rule, FillRule::Clockwise);
        assert_eq!(paint.style, RenderPaintStyle::Fill);
        assert_eq!(paint.feather, 0.0);
        let mut tessellation =
            draw::build_fill_tessellation(&path.raw_path, Mat2D::IDENTITY).unwrap();
        tessellation.make_double_sided_with_direction(draw::clockwise_atomic_negate_coverage(
            &path.raw_path,
            Mat2D::IDENTITY,
            path.fill_rule,
            true,
        ));
        tessellation
    }

    fn fixed_rawtext_direct_inputs() -> atlas_input_oracle::AtlasInputs {
        fixed_direct_inputs_from_tessellation(
            fixed_rawtext_draw_one_tessellation(),
            RAWTEXT_ORACLE_FRAME_WIDTH,
            RAWTEXT_ORACLE_FRAME_HEIGHT,
        )
    }

    fn fixed_rawtext_spans() -> tess_span_oracle::TessSpanArtifact {
        let tessellation = fixed_rawtext_draw_one_tessellation();
        tess_span_oracle::TessSpanArtifact::from_spans(0, &tessellation.spans)
    }

    fn fixed_strokes_round_tessellation() -> draw::FillTessellation {
        let path = strokes_round_draw_38_path();
        draw::build_stroke_tessellation(
            &path,
            Mat2D::IDENTITY,
            STROKES_ROUND_ORACLE_THICKNESS,
            StrokeJoin::Miter,
            StrokeCap::Butt,
        )
        .unwrap()
    }

    fn fixed_strokes_round_direct_inputs() -> atlas_input_oracle::AtlasInputs {
        fixed_direct_inputs_from_tessellation(
            fixed_strokes_round_tessellation(),
            STROKES_ROUND_ORACLE_FRAME_SIZE,
            STROKES_ROUND_ORACLE_FRAME_SIZE,
        )
    }

    fn fixed_strokes_round_spans() -> tess_span_oracle::TessSpanArtifact {
        let tessellation = fixed_strokes_round_tessellation();
        tess_span_oracle::TessSpanArtifact::from_spans(0, &tessellation.spans)
    }

    fn fixed_overstroke_quad_tessellation() -> draw::FillTessellation {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(100.0, 0.0);
        path.cubic_to(66.666_664, -26.666_668, 33.333_336, -26.666_668, 0.0, 0.0);
        path.close();
        draw::build_stroke_tessellation(
            &path,
            Mat2D([0.2, 0.0, 0.0, 0.2, 290.0, 80.0]),
            500.0,
            StrokeJoin::Miter,
            StrokeCap::Butt,
        )
        .unwrap()
    }

    fn fixed_overstroke_quad_spans() -> tess_span_oracle::TessSpanArtifact {
        let tessellation = fixed_overstroke_quad_tessellation();
        tess_span_oracle::TessSpanArtifact::from_spans(0, &tessellation.spans)
    }

    fn spotify_prefix_blit(draw_count: usize) -> atlas_blit_oracle::AtlasBlit {
        use nuxie_render_stream::{Command, RenderStream};

        let mut stream = RenderStream::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/renderer/streams/riv/spotify_kids_demo.rive-stream"
        )))
        .unwrap();
        let mut draws = 0usize;
        let end = stream.frames[0]
            .commands
            .iter()
            .position(|command| {
                if matches!(command, Command::DrawPath { .. }) {
                    draws += 1;
                }
                draws == draw_count && matches!(command, Command::Restore)
            })
            .expect("Spotify draw prefix restore");
        stream.frames[0].commands.truncate(end + 1);
        let mut factory = WgpuFactory::new_with_mode(
            SPOTIFY_FOOT_ORACLE_FRAME_WIDTH,
            SPOTIFY_FOOT_ORACLE_FRAME_HEIGHT,
            RenderMode::Msaa,
        )
        .unwrap();
        let mut frame = factory.begin_frame(stream.clear_color.unwrap_or(0));
        stream.replay_frame(0, &mut factory, &mut frame).unwrap();
        atlas_blit_oracle::AtlasBlit::new(
            SPOTIFY_FOOT_ORACLE_FRAME_WIDTH,
            SPOTIFY_FOOT_ORACLE_FRAME_HEIGHT,
            frame.finish().unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn spotify_right_foot_prefix_matches_dawn_boundary_pixel() {
        let actual = spotify_prefix_blit(4);
        let offset = (382 * SPOTIFY_FOOT_ORACLE_FRAME_WIDTH as usize + 202) * 4;

        assert_eq!(&actual.pixels()[offset..offset + 4], &[152, 156, 186, 255]);
    }

    #[test]
    fn spotify_distinct_equivalent_clip_reentry_matches_dawn_prefix() {
        use sha2::Digest as _;

        let actual = spotify_prefix_blit(7);
        assert_eq!(
            format!("{:x}", sha2::Sha256::digest(actual.pixels())),
            "d986e99004c24ef06fd383b3d7c2180f05844e23b7c4de20f25823d938b46882"
        );
    }

    fn fixed_degenerate_cubic_draw(selector: &str) -> (WgpuPath, Mat2D, WgpuPaint) {
        let mut path = RawPath::new();
        let (transform, thickness) = match selector {
            "tricky-path20" => {
                path.move_to(1.0, 1.0);
                path.cubic_to(1.66666675, 1.0, 1.66666675, 1.0, 1.0, 1.0);
                (
                    Mat2D([3.32997298, 0.0, 0.0, 3.32997298, 0.0, 0.0]),
                    9.00908184,
                )
            }
            "wide-row0" => {
                path.move_to(0.0, 0.0);
                path.cubic_to(10.0, 0.0, 10.0, 0.0, 10.0, 10.0);
                (Mat2D::IDENTITY, 100.0)
            }
            "wide-row1" => {
                path.move_to(0.0, 0.0);
                path.cubic_to(0.0, -10.0, 0.0, -10.0, 0.0, 10.0);
                (Mat2D::IDENTITY, 100.0)
            }
            "wide-row2" => {
                path.move_to(0.0, 0.0);
                path.cubic_to(0.0, -10.0, 10.0, 10.0, 0.0, 10.0);
                (Mat2D::IDENTITY, 100.0)
            }
            "wide-row3" => {
                path.move_to(0.0, 0.0);
                path.cubic_to(0.0, -10.0, 10.0, 0.0, 0.0, 0.0);
                (Mat2D::IDENTITY, 100.0)
            }
            _ => panic!("unknown degenerate cubic selector {selector}"),
        };
        (
            WgpuPath {
                valid: true,
                raw_path: path,
                fill_rule: FillRule::Clockwise,
            },
            transform,
            WgpuPaint {
                color: 0xffff_ffff,
                style: RenderPaintStyle::Stroke,
                thickness,
                join: StrokeJoin::Miter,
                cap: StrokeCap::Butt,
                feather: 0.0,
                ..WgpuPaint::default()
            },
        )
    }

    fn fixed_degenerate_cubic_tessellation(selector: &str) -> draw::FillTessellation {
        let (path, transform, paint) = fixed_degenerate_cubic_draw(selector);
        draw::build_stroke_tessellation(
            &path.raw_path,
            transform,
            paint.thickness,
            paint.join,
            paint.cap,
        )
        .unwrap()
    }

    fn fixed_degenerate_cubic_blit(selector: &str) -> atlas_blit_oracle::AtlasBlit {
        let (path, transform, paint) = fixed_degenerate_cubic_draw(selector);
        let factory = WgpuFactory::new_with_mode(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            RenderMode::Msaa,
        )
        .unwrap();
        let mut frame = factory.begin_frame(0);
        frame.transform(transform);
        frame.draw_path(&path, &paint);
        atlas_blit_oracle::AtlasBlit::new(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            frame.finish().unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn msaa_stroke_culls_counterclockwise_degenerate_join_triangles() {
        let blit = fixed_degenerate_cubic_blit("wide-row1");
        let pixel = (10 * ATLAS_ORACLE_FRAME_SIZE + 6) as usize * 4;
        assert_eq!(&blit.pixels()[pixel..pixel + 4], &[191; 4]);
    }

    fn fixed_degenerate_cubic_spans(selector: &str) -> tess_span_oracle::TessSpanArtifact {
        let tessellation = fixed_degenerate_cubic_tessellation(selector);
        tess_span_oracle::TessSpanArtifact::from_spans(0, &tessellation.spans)
    }

    fn fixed_degenerate_cubic_direct_inputs(selector: &str) -> atlas_input_oracle::AtlasInputs {
        fixed_direct_inputs_from_tessellation(
            fixed_degenerate_cubic_tessellation(selector),
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
        )
    }

    fn fixed_feather_atlas_oracle_for(
        raw_path: RawPath,
        paint: WgpuPaint,
    ) -> FixedFeatherAtlasOracle {
        let oracle = feather_atlas_oracle_for(
            raw_path,
            paint,
            Mat2D::IDENTITY,
            [ATLAS_ORACLE_FRAME_SIZE; 2],
        );
        assert_eq!(oracle.placement.bounds, [0, 0, 64, 64]);
        assert_eq!(oracle.placement.content_size, [39, 39]);
        assert_eq!(oracle.placement.physical_size, [48, 48]);
        assert_eq!(oracle.placement.origin, [0, 0]);
        assert_eq!(
            oracle.placement.translate_bits,
            ATLAS_ORACLE_PLACEMENT.map(f32::to_bits)
        );
        oracle
    }

    fn feather_atlas_oracle_for(
        raw_path: RawPath,
        paint: WgpuPaint,
        transform: Mat2D,
        frame_size: [u32; 2],
    ) -> FixedFeatherAtlasOracle {
        let stroke = paint.effective_stroke();
        let factory = WgpuFactory::new(frame_size[0], frame_size[1]).unwrap();
        let mut placement = feather_atlas_placement(
            &raw_path,
            transform,
            paint.feather,
            stroke,
            frame_size[0],
            frame_size[1],
        )
        .unwrap();
        let layout = pack_atlas_for_device(
            frame_size[0],
            factory.context.device.limits().max_texture_dimension_2d,
            &[(placement.width, placement.height)],
        )
        .unwrap();
        assert_eq!(layout.origins(), &[[0, 0]]);
        placement.origin = layout.origins()[0];
        placement.translate[0] += placement.origin[0] as f32;
        placement.translate[1] += placement.origin[1] as f32;
        let physical_size = atlas_physical_size(
            layout.extent(),
            factory.context.device.limits().max_texture_dimension_2d,
        );
        let mut tessellation =
            draw::build_feather_atlas_tessellation(&raw_path, transform, paint.feather, stroke)
                .unwrap();
        tessellation.path.atlas_transform = gpu::AtlasTransform {
            scale_factor: placement.scale,
            translate_x: placement.translate[0],
            translate_y: placement.translate[1],
        };
        for contour in &mut tessellation.contours {
            contour.path_id = 1;
        }
        let paths = [gpu::PathData::zeroed(), tessellation.path];
        let paints = [
            gpu::PaintData::solid(0, FillRule::NonZero, BlendMode::SrcOver),
            if paint.style == RenderPaintStyle::Stroke {
                gpu::PaintData::solid_stroke(0xffff_ffff, BlendMode::SrcOver)
            } else {
                gpu::PaintData::solid(0xffff_ffff, FillRule::Clockwise, BlendMode::SrcOver)
            },
        ];
        let paint_aux = [gpu::PaintAuxData::zeroed(); 2];
        let tessellation_height = draw::tessellation_texture_height(&tessellation.spans);
        let mut uniforms = analytic_uniforms(frame_size[0], frame_size[1], tessellation_height);
        uniforms.atlas_texture_inverse_size =
            [1.0 / physical_size[0] as f32, 1.0 / physical_size[1] as f32];
        uniforms.atlas_content_inverse_viewport = [
            2.0 / layout.extent()[0] as f32,
            -2.0 / layout.extent()[1] as f32,
        ];
        let mut encoder = factory.context.device.create_counted_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("nuxie-atlas-test-encoder"),
            },
        );
        let mut tessellation_uploads = factory
            .context
            .tessellator
            .begin_frame_uploads(&factory.context.device);
        let tessellation_texture = factory.context.tessellator.encode(
            &factory.context.device,
            &mut tessellation_uploads,
            &mut encoder,
            &factory.context.feather_lut.view,
            &tessellation.spans,
            &uniforms,
            &paths,
            &tessellation.contours,
            tessellation_height,
        );
        let tessellation_view =
            tessellation_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let tessellation_size = tessellation_texture.size();
        let atlas = factory
            .context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("nuxie-atlas-test-target"),
                size: wgpu::Extent3d {
                    width: physical_size[0],
                    height: physical_size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
        let atlas_view = atlas.create_view(&wgpu::TextureViewDescriptor::default());
        factory.context.atlas_pipeline.encode_mask(
            &factory.context.device,
            &mut encoder,
            &atlas_view,
            &factory.context.patch_vertex_buffer,
            &factory.context.patch_index_buffer,
            &tessellation_view,
            &factory.context.feather_lut.view,
            &uniforms,
            &paths,
            &paints,
            &paint_aux,
            &tessellation.contours,
            tessellation.base_instance,
            tessellation.instance_count,
            paint.style == RenderPaintStyle::Stroke,
            true,
            layout.extent(),
            [
                placement.origin[0],
                placement.origin[1],
                placement.width,
                placement.height,
            ],
        );
        let bytes_per_row = (physical_size[0] * 2).div_ceil(256) * 256;
        let atlas_readback = factory
            .context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("nuxie-atlas-test-readback"),
                size: u64::from(bytes_per_row) * u64::from(physical_size[1]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
        encoder.copy_texture_to_buffer(
            atlas.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &atlas_readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(physical_size[1]),
                },
            },
            atlas.size(),
        );
        let tessellation_bytes_per_row = tessellation_size.width.checked_mul(16).unwrap();
        let tessellation_readback = factory
            .context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("nuxie-atlas-input-tessellation-readback"),
                size: u64::from(tessellation_bytes_per_row) * u64::from(tessellation_size.height),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
        encoder.copy_texture_to_buffer(
            tessellation_texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &tessellation_readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(tessellation_bytes_per_row),
                    rows_per_image: Some(tessellation_size.height),
                },
            },
            tessellation_size,
        );
        tessellation_uploads.flush(&factory.context.queue);
        factory.context.queue.submit_counted(Some(encoder.finish()));
        let atlas_slice = atlas_readback.slice(..);
        let tessellation_slice = tessellation_readback.slice(..);
        let (atlas_sender, atlas_receiver) = mpsc::channel();
        atlas_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = atlas_sender.send(result);
        });
        let (tessellation_sender, tessellation_receiver) = mpsc::channel();
        tessellation_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tessellation_sender.send(result);
        });
        factory
            .context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        atlas_receiver.recv().unwrap().unwrap();
        tessellation_receiver.recv().unwrap().unwrap();
        let mapped = atlas_slice.get_mapped_range().unwrap();
        let width = physical_size[0] as usize;
        let height = physical_size[1] as usize;
        let mut pixels = Vec::with_capacity(width * height);
        for y in 0..height {
            let row = &mapped[y * bytes_per_row as usize..][..width * 2];
            pixels.extend(
                row.chunks_exact(2)
                    .map(|sample| u16::from_le_bytes(sample.try_into().unwrap())),
            );
        }
        drop(mapped);
        atlas_readback.unmap();
        let mask =
            atlas_mask_oracle::AtlasMask::new(physical_size[0], physical_size[1], pixels).unwrap();
        let mapped = tessellation_slice.get_mapped_range().unwrap();
        let mut texels = Vec::with_capacity(
            tessellation_size.width as usize * tessellation_size.height as usize,
        );
        for row in mapped.chunks_exact(tessellation_bytes_per_row as usize) {
            texels.extend(row.chunks_exact(16).map(|texel| {
                [
                    u32::from_le_bytes(texel[0..4].try_into().unwrap()),
                    u32::from_le_bytes(texel[4..8].try_into().unwrap()),
                    u32::from_le_bytes(texel[8..12].try_into().unwrap()),
                    u32::from_le_bytes(texel[12..16].try_into().unwrap()),
                ]
            }));
        }
        drop(mapped);
        tessellation_readback.unmap();
        let inputs = atlas_input_oracle::AtlasInputs::new(
            tessellation.base_instance,
            tessellation.instance_count,
            tessellation
                .contours
                .iter()
                .map(|contour| atlas_input_oracle::ContourRecord {
                    midpoint_x_bits: contour.midpoint[0].to_bits(),
                    midpoint_y_bits: contour.midpoint[1].to_bits(),
                    path_id: contour.path_id,
                    vertex_index0: contour.vertex_index0,
                })
                .collect(),
            tessellation_size.width,
            tessellation_size.height,
            texels,
        )
        .unwrap();
        let placement = atlas_placement_oracle::AtlasPlacement {
            frame_size,
            bounds: placement.bounds.map(|value| value as i32),
            origin: placement.origin,
            content_size: layout.extent(),
            physical_size,
            scale_bits: placement.scale.to_bits(),
            translate_bits: placement.translate.map(f32::to_bits),
            scissor: [
                placement.origin[0],
                placement.origin[1],
                placement.origin[0] + placement.width,
                placement.origin[1] + placement.height,
            ],
        };
        FixedFeatherAtlasOracle {
            mask,
            inputs,
            placement,
        }
    }

    #[derive(Clone, Copy)]
    enum LargeFeatherAtlasCase {
        Cusp,
        ShapesCusp,
    }

    fn large_feather_atlas_spec(case: LargeFeatherAtlasCase) -> (RawPath, Mat2D) {
        match case {
            LargeFeatherAtlasCase::Cusp => (
                feather_cusp_large_radius_path(),
                feather_grid_transform(2.0, 6.0),
            ),
            LargeFeatherAtlasCase::ShapesCusp => (
                feather_shapes_large_radius_cusp_path(),
                feather_grid_transform(3.0, 6.0),
            ),
        }
    }

    fn large_feather_atlas_oracle(case: LargeFeatherAtlasCase) -> FixedFeatherAtlasOracle {
        let (raw_path, transform) = large_feather_atlas_spec(case);
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            feather: LARGE_FEATHER_RADIUS,
            ..WgpuPaint::default()
        };
        feather_atlas_oracle_for(raw_path, paint, transform, LARGE_FEATHER_FRAME_SIZE)
    }

    fn large_feather_atlas_blit(case: LargeFeatherAtlasCase) -> atlas_blit_oracle::AtlasBlit {
        let (raw_path, transform) = large_feather_atlas_spec(case);
        let path = WgpuPath {
            valid: true,
            raw_path,
            fill_rule: FillRule::Clockwise,
        };
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            feather: LARGE_FEATHER_RADIUS,
            ..WgpuPaint::default()
        };
        let factory = WgpuFactory::new_with_mode(
            LARGE_FEATHER_FRAME_SIZE[0],
            LARGE_FEATHER_FRAME_SIZE[1],
            RenderMode::Msaa,
        )
        .unwrap();
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.transform(transform);
        frame.draw_path(&path, &paint);
        atlas_blit_oracle::AtlasBlit::new(
            LARGE_FEATHER_FRAME_SIZE[0],
            LARGE_FEATHER_FRAME_SIZE[1],
            frame.finish().unwrap(),
        )
        .unwrap()
    }

    fn fixed_feather_atlas_mask(join: StrokeJoin) -> atlas_mask_oracle::AtlasMask {
        fixed_feather_atlas_oracle(join).mask
    }

    fn fixed_feather_atlas_blit_with_clip(
        clip_rect: Option<[f32; 4]>,
    ) -> Result<atlas_blit_oracle::AtlasBlit, RendererError> {
        fixed_feather_atlas_blit_with_clips(clip_rect, false)
    }

    fn fixed_feather_atlas_blit_with_clips(
        clip_rect: Option<[f32; 4]>,
        path_clip: bool,
    ) -> Result<atlas_blit_oracle::AtlasBlit, RendererError> {
        let factory = WgpuFactory::new_with_mode(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            RenderMode::Msaa,
        )
        .unwrap();
        let mut raw_path = RawPath::new();
        raw_path.move_to(ATLAS_ORACLE_SQUARE_MIN, ATLAS_ORACLE_SQUARE_MIN);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MAX, ATLAS_ORACLE_SQUARE_MIN);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MAX, ATLAS_ORACLE_SQUARE_MAX);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MIN, ATLAS_ORACLE_SQUARE_MAX);
        raw_path.close();
        let path = WgpuPath {
            valid: true,
            raw_path,
            fill_rule: FillRule::NonZero,
        };
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            style: RenderPaintStyle::Stroke,
            thickness: ATLAS_ORACLE_STROKE_THICKNESS,
            join: ATLAS_ORACLE_STROKE_JOIN,
            cap: ATLAS_ORACLE_STROKE_CAP,
            feather: ATLAS_ORACLE_FEATHER,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0);
        if let Some([left, top, right, bottom]) = clip_rect {
            let mut raw_clip = RawPath::new();
            raw_clip.move_to(left, top);
            raw_clip.line_to(right, top);
            raw_clip.line_to(right, bottom);
            raw_clip.line_to(left, bottom);
            raw_clip.close();
            frame.clip_path(&WgpuPath {
                valid: true,
                raw_path: raw_clip,
                fill_rule: FillRule::NonZero,
            });
        }
        if path_clip {
            let mut raw_clip = RawPath::new();
            raw_clip.move_to(32.0, 16.0);
            raw_clip.line_to(48.0, 48.0);
            raw_clip.line_to(16.0, 48.0);
            raw_clip.close();
            frame.clip_path(&WgpuPath {
                valid: true,
                raw_path: raw_clip,
                fill_rule: FillRule::NonZero,
            });
        }
        frame.draw_path(&path, &paint);
        let pixels = frame.finish()?;
        Ok(atlas_blit_oracle::AtlasBlit::new(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            pixels,
        )
        .unwrap())
    }

    fn fixed_feather_atlas_blit() -> atlas_blit_oracle::AtlasBlit {
        fixed_feather_atlas_blit_with_clip(None).unwrap()
    }

    fn fixed_feather_atlas_empty_stroke_blit() -> atlas_blit_oracle::AtlasBlit {
        let factory = WgpuFactory::new_with_mode(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            RenderMode::Msaa,
        )
        .unwrap();
        let center = (ATLAS_ORACLE_SQUARE_MIN + ATLAS_ORACLE_SQUARE_MAX) * 0.5;
        let mut raw_path = RawPath::new();
        raw_path.move_to(center, center);
        let path = WgpuPath {
            valid: true,
            raw_path,
            fill_rule: FillRule::NonZero,
        };
        let marker_radius = 3.5;
        let mut marker_path = RawPath::new();
        marker_path.move_to(center - marker_radius, center - marker_radius);
        marker_path.line_to(center + marker_radius, center - marker_radius);
        marker_path.line_to(center + marker_radius, center + marker_radius);
        marker_path.line_to(center - marker_radius, center + marker_radius);
        marker_path.close();
        let marker_path = WgpuPath {
            valid: true,
            raw_path: marker_path,
            fill_rule: FillRule::NonZero,
        };
        let marker_paint = WgpuPaint {
            color: 0xffff_0000,
            ..WgpuPaint::default()
        };
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            style: RenderPaintStyle::Stroke,
            thickness: ATLAS_ORACLE_STROKE_THICKNESS,
            join: ATLAS_ORACLE_STROKE_JOIN,
            cap: StrokeCap::Round,
            feather: ATLAS_ORACLE_FEATHER,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0);
        frame.draw_path(&marker_path, &marker_paint);
        frame.draw_path(&path, &paint);
        atlas_blit_oracle::AtlasBlit::new(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            frame.finish().unwrap(),
        )
        .unwrap()
    }

    fn advanced_feather_atlas_blit_with_mode(
        mode: RenderMode,
    ) -> Result<atlas_blit_oracle::AtlasBlit, RendererError> {
        let factory =
            WgpuFactory::new_with_mode(ATLAS_ORACLE_FRAME_SIZE, ATLAS_ORACLE_FRAME_SIZE, mode)
                .unwrap();
        let mut raw_path = RawPath::new();
        raw_path.move_to(ATLAS_ORACLE_SQUARE_MIN, ATLAS_ORACLE_SQUARE_MIN);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MAX, ATLAS_ORACLE_SQUARE_MIN);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MAX, ATLAS_ORACLE_SQUARE_MAX);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MIN, ATLAS_ORACLE_SQUARE_MAX);
        raw_path.close();
        let path = WgpuPath {
            valid: true,
            raw_path,
            fill_rule: FillRule::Clockwise,
        };
        let paint = WgpuPaint {
            color: 0xc0e0_8040,
            style: RenderPaintStyle::Fill,
            feather: ATLAS_ORACLE_FEATHER,
            blend_mode: BlendMode::ColorDodge,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff20_4080);
        frame.draw_path(&path, &paint);
        let pixels = frame.finish()?;
        Ok(atlas_blit_oracle::AtlasBlit::new(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            pixels,
        )
        .unwrap())
    }

    fn advanced_feather_atlas_blit() -> Result<atlas_blit_oracle::AtlasBlit, RendererError> {
        advanced_feather_atlas_blit_with_mode(RenderMode::Msaa)
    }

    fn advanced_feather_atomic_output() -> Result<atlas_blit_oracle::AtlasBlit, RendererError> {
        advanced_feather_atlas_blit_with_mode(RenderMode::ClockwiseAtomic)
    }

    fn interleaved_feather_colorburn_pair_stream() -> nuxie_render_stream::RenderStream {
        use nuxie_render_stream::{Command, Frame, RenderStream};

        let source = RenderStream::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/renderer/streams/gm/interleavedfeather.rive-stream"
        )))
        .unwrap();
        let commands = &source.frames[0].commands;
        let mut draw_count = 0usize;
        let mut latest_save = 0usize;
        let mut pair_start = None;
        let mut pair_end = None;
        for (index, command) in commands.iter().enumerate() {
            match command {
                Command::Save => latest_save = index,
                Command::DrawPath { .. } => {
                    draw_count += 1;
                    if draw_count == 13 {
                        pair_start = Some(latest_save);
                    }
                }
                Command::Restore if draw_count == 14 && pair_start.is_some() => {
                    pair_end = Some(index + 1);
                    break;
                }
                _ => {}
            }
        }
        let pair_start = pair_start.expect("interleavedfeather draw 13 group");
        let pair_end = pair_end.expect("interleavedfeather draw 14 restore");
        let pair_commands = commands[pair_start..pair_end].to_vec();
        let pair_draws = pair_commands
            .iter()
            .filter_map(|command| match command {
                Command::DrawPath { path, paint } => Some((path, paint)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(pair_draws.len(), 2);
        assert_eq!(pair_draws[0].1.style, RenderPaintStyle::Fill);
        assert_eq!(pair_draws[0].1.color, 0x4aff_afc5);
        assert_eq!(pair_draws[1].1.style, RenderPaintStyle::Stroke);
        assert_eq!(pair_draws[1].1.color, 0xe000_0000);
        assert_eq!(pair_draws[1].1.thickness, 5.00454855);
        assert_eq!(pair_draws[1].1.join, StrokeJoin::Round);
        assert!(pair_draws.iter().all(|(_, paint)| {
            paint.feather == 9.56621265 && paint.blend_mode == BlendMode::ColorBurn
        }));

        RenderStream {
            frame_size: Some((
                ATOMIC_COLORBURN_PAIR_FRAME_SIZE,
                ATOMIC_COLORBURN_PAIR_FRAME_SIZE,
            )),
            clear_color: Some(0),
            resources: source.resources,
            frames: vec![Frame {
                commands: pair_commands,
            }],
        }
    }

    fn interleaved_feather_colorburn_pair_frame() -> WgpuFrame {
        let pair = interleaved_feather_colorburn_pair_stream();
        let mut factory = WgpuFactory::new_with_mode(
            ATOMIC_COLORBURN_PAIR_FRAME_SIZE,
            ATOMIC_COLORBURN_PAIR_FRAME_SIZE,
            RenderMode::ClockwiseAtomic,
        )
        .unwrap();
        let mut frame = factory.begin_frame(0);
        pair.replay_frame(0, &mut factory, &mut frame).unwrap();
        frame
    }

    fn path_stream_full_output(
        source: &str,
        width: u32,
        height: u32,
        clear_color: u32,
        blend_mode_override: Option<BlendMode>,
    ) -> atlas_blit_oracle::AtlasBlit {
        let mut stream = nuxie_render_stream::RenderStream::parse(source).unwrap();
        assert_eq!(stream.frame_size, Some((width, height)));
        assert_eq!(stream.clear_color, Some(clear_color));
        assert_eq!(stream.frames.len(), 1);
        if let Some(blend_mode) = blend_mode_override {
            for command in &mut stream.frames[0].commands {
                if let nuxie_render_stream::Command::DrawPath { paint, .. } = command {
                    paint.blend_mode = blend_mode;
                }
            }
        }
        let mut factory =
            WgpuFactory::new_with_mode(width, height, RenderMode::ClockwiseAtomic).unwrap();
        let mut frame = factory.begin_frame(clear_color);
        stream.replay_frame(0, &mut factory, &mut frame).unwrap();
        atlas_blit_oracle::AtlasBlit::new(width, height, frame.finish().unwrap()).unwrap()
    }

    fn interleaved_feather_full_output() -> atlas_blit_oracle::AtlasBlit {
        path_stream_full_output(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../fixtures/renderer/streams/gm/interleavedfeather.rive-stream"
            )),
            ATOMIC_INTERLEAVED_FEATHER_FULL_FRAME_SIZE,
            ATOMIC_INTERLEAVED_FEATHER_FULL_FRAME_SIZE,
            0,
            None,
        )
    }

    fn dstreadshuffle_full_output() -> atlas_blit_oracle::AtlasBlit {
        path_stream_full_output(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../fixtures/renderer/streams/gm/dstreadshuffle.rive-stream"
            )),
            ATOMIC_DSTREADSHUFFLE_FULL_FRAME_WIDTH,
            ATOMIC_DSTREADSHUFFLE_FULL_FRAME_HEIGHT,
            0xffff_ffff,
            None,
        )
    }

    fn dstreadshuffle_srcover_control_output() -> atlas_blit_oracle::AtlasBlit {
        path_stream_full_output(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../fixtures/renderer/streams/gm/dstreadshuffle.rive-stream"
            )),
            ATOMIC_DSTREADSHUFFLE_FULL_FRAME_WIDTH,
            ATOMIC_DSTREADSHUFFLE_FULL_FRAME_HEIGHT,
            0xffff_ffff,
            Some(BlendMode::SrcOver),
        )
    }

    fn spotify_kids_app_icon_full_frame() -> WgpuFrame {
        let stream = nuxie_render_stream::RenderStream::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/renderer/streams/riv/spotify_kids_app_icon.rive-stream"
        )))
        .unwrap();
        assert_eq!(
            stream.frame_size,
            Some((
                ATOMIC_SPOTIFY_FULL_FRAME_WIDTH,
                ATOMIC_SPOTIFY_FULL_FRAME_HEIGHT
            ))
        );
        assert_eq!(stream.clear_color, None);
        assert_eq!(stream.frames.len(), 1);
        let mut factory = WgpuFactory::new_with_mode(
            ATOMIC_SPOTIFY_FULL_FRAME_WIDTH,
            ATOMIC_SPOTIFY_FULL_FRAME_HEIGHT,
            RenderMode::ClockwiseAtomic,
        )
        .unwrap();
        let mut frame = factory.begin_frame(0);
        stream.replay_frame(0, &mut factory, &mut frame).unwrap();
        frame
    }

    fn fixed_feather_atlas_clipped_blit() -> Result<atlas_blit_oracle::AtlasBlit, RendererError> {
        fixed_feather_atlas_blit_with_clip(Some([16.0, 8.0, 32.0, 56.0]))
    }

    fn fixed_feather_atlas_path_clipped_blit() -> atlas_blit_oracle::AtlasBlit {
        fixed_feather_atlas_blit_with_clips(None, true).unwrap()
    }

    fn fixed_feather_atlas_blit_with_path_clips(
        clips: &[WgpuPath],
        fill_content: bool,
    ) -> Result<atlas_blit_oracle::AtlasBlit, RendererError> {
        let factory = WgpuFactory::new_with_mode(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            RenderMode::Msaa,
        )
        .unwrap();
        let mut raw_path = RawPath::new();
        raw_path.move_to(ATLAS_ORACLE_SQUARE_MIN, ATLAS_ORACLE_SQUARE_MIN);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MAX, ATLAS_ORACLE_SQUARE_MIN);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MAX, ATLAS_ORACLE_SQUARE_MAX);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MIN, ATLAS_ORACLE_SQUARE_MAX);
        raw_path.close();
        let path = WgpuPath {
            valid: true,
            raw_path,
            fill_rule: if fill_content {
                FillRule::Clockwise
            } else {
                FillRule::NonZero
            },
        };
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            style: if fill_content {
                RenderPaintStyle::Fill
            } else {
                RenderPaintStyle::Stroke
            },
            thickness: ATLAS_ORACLE_STROKE_THICKNESS,
            join: ATLAS_ORACLE_STROKE_JOIN,
            cap: ATLAS_ORACLE_STROKE_CAP,
            feather: ATLAS_ORACLE_FEATHER,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0);
        for clip in clips {
            frame.clip_path(clip);
        }
        frame.draw_path(&path, &paint);
        let pixels = frame.finish()?;
        Ok(atlas_blit_oracle::AtlasBlit::new(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            pixels,
        )
        .unwrap())
    }

    fn triangle_path(points: [[f32; 2]; 3], fill_rule: FillRule) -> WgpuPath {
        let mut raw_path = RawPath::new();
        raw_path.move_to(points[0][0], points[0][1]);
        raw_path.line_to(points[1][0], points[1][1]);
        raw_path.line_to(points[2][0], points[2][1]);
        raw_path.close();
        WgpuPath {
            valid: true,
            raw_path,
            fill_rule,
        }
    }

    fn fixed_feather_atlas_nested_path_clipped_blit(
    ) -> Result<atlas_blit_oracle::AtlasBlit, RendererError> {
        fixed_feather_atlas_blit_with_path_clips(
            &[
                triangle_path([[32.0, 8.0], [56.0, 56.0], [8.0, 56.0]], FillRule::NonZero),
                triangle_path(
                    [[32.0, 20.0], [44.0, 48.0], [20.0, 48.0]],
                    FillRule::NonZero,
                ),
            ],
            false,
        )
    }

    fn fixed_feather_atlas_nested_even_odd_path_clipped_blit(
    ) -> Result<atlas_blit_oracle::AtlasBlit, RendererError> {
        let mut outer = rect_path([8.0, 8.0, 32.0, 56.0], FillRule::Clockwise);
        outer.raw_path.add_path_backwards(
            &rect_path([32.0, 8.0, 56.0, 56.0], FillRule::Clockwise).raw_path,
            Mat2D::IDENTITY,
        );
        let mut inner = rect_path([12.0, 12.0, 52.0, 52.0], FillRule::EvenOdd);
        inner.raw_path.add_path(
            &rect_path([20.0, 20.0, 44.0, 44.0], FillRule::EvenOdd).raw_path,
            Mat2D::IDENTITY,
        );
        fixed_feather_atlas_blit_with_path_clips(&[outer, inner], true)
    }

    fn fixed_feather_atlas_nested_clockwise_path_clipped_blit(
    ) -> Result<atlas_blit_oracle::AtlasBlit, RendererError> {
        let mut outer = rect_path([8.0, 8.0, 56.0, 56.0], FillRule::EvenOdd);
        outer.raw_path.add_path(
            &rect_path([24.0, 24.0, 40.0, 40.0], FillRule::EvenOdd).raw_path,
            Mat2D::IDENTITY,
        );
        let mut inner = rect_path([8.0, 8.0, 32.0, 56.0], FillRule::Clockwise);
        inner.raw_path.add_path_backwards(
            &rect_path([32.0, 8.0, 56.0, 56.0], FillRule::Clockwise).raw_path,
            Mat2D::IDENTITY,
        );
        fixed_feather_atlas_blit_with_path_clips(&[outer, inner], true)
    }

    fn fixed_feather_atlas_changing_path_clipped_blit_with_unclipped_middle(
        unclipped_middle: bool,
    ) -> Result<atlas_blit_oracle::AtlasBlit, RendererError> {
        let factory = WgpuFactory::new_with_mode(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            RenderMode::Msaa,
        )
        .unwrap();
        let mut raw_path = RawPath::new();
        raw_path.move_to(ATLAS_ORACLE_SQUARE_MIN, ATLAS_ORACLE_SQUARE_MIN);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MAX, ATLAS_ORACLE_SQUARE_MIN);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MAX, ATLAS_ORACLE_SQUARE_MAX);
        raw_path.line_to(ATLAS_ORACLE_SQUARE_MIN, ATLAS_ORACLE_SQUARE_MAX);
        raw_path.close();
        let path = WgpuPath {
            valid: true,
            raw_path,
            fill_rule: FillRule::NonZero,
        };
        let paint = WgpuPaint {
            color: 0xffff_ffff,
            style: RenderPaintStyle::Stroke,
            thickness: ATLAS_ORACLE_STROKE_THICKNESS,
            join: ATLAS_ORACLE_STROKE_JOIN,
            cap: ATLAS_ORACLE_STROKE_CAP,
            feather: ATLAS_ORACLE_FEATHER,
            ..WgpuPaint::default()
        };
        let triangle = |points: [[f32; 2]; 3]| {
            let mut raw_path = RawPath::new();
            raw_path.move_to(points[0][0], points[0][1]);
            raw_path.line_to(points[1][0], points[1][1]);
            raw_path.line_to(points[2][0], points[2][1]);
            raw_path.close();
            WgpuPath {
                valid: true,
                raw_path,
                fill_rule: FillRule::NonZero,
            }
        };
        let mut frame = factory.begin_frame(0);
        frame.save();
        frame.clip_path(&triangle([[16.0, 16.0], [32.0, 48.0], [8.0, 48.0]]));
        frame.draw_path(&path, &paint);
        frame.restore();
        if unclipped_middle {
            frame.draw_path(&path, &paint);
        }
        frame.clip_path(&triangle([[48.0, 16.0], [56.0, 48.0], [32.0, 48.0]]));
        frame.draw_path(&path, &paint);
        let pixels = frame.finish()?;
        Ok(atlas_blit_oracle::AtlasBlit::new(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            pixels,
        )
        .unwrap())
    }

    fn fixed_feather_atlas_changing_path_clipped_blit(
    ) -> Result<atlas_blit_oracle::AtlasBlit, RendererError> {
        fixed_feather_atlas_changing_path_clipped_blit_with_unclipped_middle(false)
    }

    #[test]
    fn msaa_feather_atlas_blit_renders_premultiplied_coverage() {
        let blit = fixed_feather_atlas_blit();
        let pixels = blit.pixels();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(0, 0), [8; 4]);
        assert_eq!(pixel(32, 16), [79; 4]);
        assert_eq!(pixel(32, 32), [27; 4]);
        assert!(pixels
            .chunks_exact(4)
            .all(|rgba| { rgba.iter().max().unwrap() - rgba.iter().min().unwrap() <= 1 }));
    }

    #[test]
    fn msaa_feather_atlas_blits_multiple_fills_in_draw_order() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let square = |left: f32, right: f32| {
            let mut raw_path = RawPath::new();
            raw_path.move_to(left, 16.0);
            raw_path.line_to(right, 16.0);
            raw_path.line_to(right, 48.0);
            raw_path.line_to(left, 48.0);
            raw_path.close();
            WgpuPath {
                valid: true,
                raw_path,
                fill_rule: FillRule::NonZero,
            }
        };
        let red = WgpuPaint {
            color: 0xffff_0000,
            feather: 4.0,
            ..WgpuPaint::default()
        };
        let blue = WgpuPaint {
            color: 0xff00_00ff,
            feather: 4.0,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0);
        frame.draw_path(&square(4.0, 36.0), &red);
        frame.draw_path(&square(28.0, 60.0), &blue);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert!(pixel(16, 32)[0] > pixel(16, 32)[2]);
        assert!(pixel(48, 32)[2] > pixel(48, 32)[0]);
        assert!(pixel(32, 32)[2] > pixel(32, 32)[0]);
    }

    #[test]
    fn msaa_feather_atlas_blit_applies_axis_aligned_stencil_clip() {
        let blit = fixed_feather_atlas_clipped_blit().unwrap();
        let pixels = blit.pixels();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(15, 32), [0; 4]);
        assert_ne!(pixel(24, 16), [0; 4]);
        assert_ne!(pixel(31, 16), [0; 4]);
        assert_eq!(pixel(32, 16), [0; 4]);
        assert!(pixels.chunks_exact(4).enumerate().all(|(index, rgba)| {
            let x = index % 64;
            let y = index / 64;
            (16..32).contains(&x) && (8..56).contains(&y) || rgba == [0; 4]
        }));
    }

    #[test]
    fn msaa_feather_atlas_blit_applies_outer_path_clip_with_stencil() {
        let blit = fixed_feather_atlas_path_clipped_blit();
        let pixels = blit.pixels();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(8, 32), [0; 4]);
        assert_ne!(pixel(32, 32), [0; 4]);
        assert_eq!(pixel(32, 8), [0; 4]);
        assert_ne!(pixel(32, 47), [0; 4]);
    }

    #[test]
    fn msaa_feather_atlas_blit_reuses_unchanged_outer_path_clip() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let mut raw_clip = RawPath::new();
        raw_clip.move_to(32.0, 16.0);
        raw_clip.line_to(48.0, 48.0);
        raw_clip.line_to(16.0, 48.0);
        raw_clip.close();
        let clip = WgpuPath {
            valid: true,
            raw_path: raw_clip,
            fill_rule: FillRule::NonZero,
        };
        let paint = WgpuPaint {
            feather: ATLAS_ORACLE_FEATHER,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0);
        frame.clip_path(&clip);
        frame.draw_path(&clip, &paint);
        frame.draw_path(&clip, &paint);

        assert_eq!(frame.next_clip_id, 2);
        assert_eq!(frame.draws.len(), 3);
        frame.finish().unwrap();
    }

    #[test]
    fn msaa_feather_atlas_blit_intersects_path_and_axis_aligned_stencil_clips() {
        let blit =
            fixed_feather_atlas_blit_with_clips(Some([28.0, 0.0, 40.0, 64.0]), true).unwrap();
        let pixels = blit.pixels();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(24, 32), [0; 4]);
        assert_ne!(pixel(32, 32), [0; 4]);
        assert_eq!(pixel(40, 32), [0; 4]);
    }

    #[test]
    fn msaa_feather_atlas_blit_intersects_nested_non_zero_path_clips() {
        let blit = fixed_feather_atlas_nested_path_clipped_blit().unwrap();
        let pixels = blit.pixels();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(22, 40), [0; 4]);
        assert_eq!(pixel(24, 40), [61; 4]);
    }

    #[test]
    fn msaa_feather_atlas_blit_extends_the_rendered_clip_stack_incrementally() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let triangle = |points: [[f32; 2]; 3]| {
            let mut raw_path = RawPath::new();
            raw_path.move_to(points[0][0], points[0][1]);
            raw_path.line_to(points[1][0], points[1][1]);
            raw_path.line_to(points[2][0], points[2][1]);
            raw_path.close();
            WgpuPath {
                valid: true,
                raw_path,
                fill_rule: FillRule::NonZero,
            }
        };
        let outer = triangle([[32.0, 8.0], [56.0, 56.0], [8.0, 56.0]]);
        let inner = triangle([[32.0, 20.0], [44.0, 48.0], [20.0, 48.0]]);
        let paint = WgpuPaint {
            feather: ATLAS_ORACLE_FEATHER,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0);
        frame.clip_path(&outer);
        frame.draw_path(&outer, &paint);

        assert_eq!(frame.next_clip_id, 2);
        assert_eq!(frame.draws.len(), 2);

        frame.clip_path(&inner);
        frame.draw_path(&outer, &paint);

        assert_eq!(frame.next_clip_id, 3);
        assert_eq!(frame.draws.len(), 5);
        assert!(matches!(
            frame.draws[2].role,
            DrawRole::ClipUpdate {
                replacement_id: 2,
                parent_id: 1
            }
        ));
        assert!(matches!(
            frame.draws[3].role,
            DrawRole::ClipReset {
                action: MsaaClipResetAction::IntersectPreviousNonZero,
                ..
            }
        ));
        assert!(matches!(
            frame.draws[4].role,
            DrawRole::Content { clip_id: 2 }
        ));
        frame.finish().unwrap();
    }

    #[test]
    fn msaa_feather_atlas_blit_resets_stencil_between_outer_path_clips() {
        let blit = fixed_feather_atlas_changing_path_clipped_blit().unwrap();
        let pixels = blit.pixels();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(16, 32), [79; 4]);
        assert_eq!(pixel(32, 32), [0; 4]);
        assert_eq!(pixel(48, 32), [79; 4]);
    }

    #[test]
    fn msaa_unclipped_atlas_draw_ignores_retained_outer_path_stencil() {
        let blit =
            fixed_feather_atlas_changing_path_clipped_blit_with_unclipped_middle(true).unwrap();
        let pixels = blit.pixels();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_ne!(pixel(32, 16), [0; 4]);
        assert_ne!(pixel(32, 32), [0; 4]);
        assert_eq!(pixel(16, 32), pixel(48, 32));
    }

    #[test]
    fn msaa_feather_atlas_blit_intersects_clockwise_and_nested_even_odd_path_clips() {
        let blit = fixed_feather_atlas_nested_even_odd_path_clipped_blit().unwrap();
        let pixels = blit.pixels();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_ne!(pixel(18, 18), [0; 4]);
        assert_eq!(pixel(24, 32), [0; 4]);
        assert_eq!(pixel(46, 18), [0; 4]);
    }

    #[test]
    fn msaa_feather_atlas_blit_intersects_even_odd_and_nested_clockwise_path_clips() {
        let blit = fixed_feather_atlas_nested_clockwise_path_clipped_blit().unwrap();
        let pixels = blit.pixels();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_ne!(pixel(20, 32), [0; 4]);
        assert_eq!(pixel(28, 32), [0; 4]);
        assert_eq!(pixel(44, 32), [0; 4]);
    }

    #[test]
    fn msaa_path_clip_applies_to_direct_fill_draw() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let mut raw_clip = RawPath::new();
        raw_clip.move_to(32.0, 16.0);
        raw_clip.line_to(48.0, 48.0);
        raw_clip.line_to(16.0, 48.0);
        raw_clip.close();
        let clip = WgpuPath {
            valid: true,
            raw_path: raw_clip,
            fill_rule: FillRule::NonZero,
        };
        let path = rect_path([8.0, 8.0, 56.0, 56.0], FillRule::NonZero);
        let paint = WgpuPaint {
            color: 0xffff_0000,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0);
        frame.clip_path(&clip);
        frame.draw_path(&path, &paint);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(8, 32), [0; 4]);
        assert_eq!(pixel(32, 32), [255, 0, 0, 255]);
        assert_eq!(pixel(32, 8), [0; 4]);
        assert_eq!(pixel(32, 47), [255, 0, 0, 255]);
    }

    #[test]
    fn msaa_nested_path_clip_survives_prior_overlapping_content() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let outer = triangle_path([[32.0, 4.0], [60.0, 60.0], [4.0, 60.0]], FillRule::NonZero);
        let inner = triangle_path(
            [[32.0, 20.0], [48.0, 52.0], [16.0, 52.0]],
            FillRule::NonZero,
        );
        let content = rect_path([0.0, 0.0, 64.0, 64.0], FillRule::NonZero);
        let green = WgpuPaint {
            color: 0xff00_ff00,
            ..WgpuPaint::default()
        };
        let red = WgpuPaint {
            color: 0xffff_0000,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0);
        frame.clip_path(&outer);
        frame.draw_path(&content, &green);
        frame.clip_path(&inner);
        frame.draw_path(&content, &red);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(32, 36), [255, 0, 0, 255]);
        assert_eq!(pixel(8, 56), [0, 255, 0, 255]);
        assert_eq!(pixel(2, 2), [0; 4]);
    }

    #[test]
    fn msaa_path_clip_intersects_axis_aligned_stencil_clip() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let mut raw_clip = RawPath::new();
        raw_clip.move_to(32.0, 8.0);
        raw_clip.line_to(56.0, 56.0);
        raw_clip.line_to(8.0, 56.0);
        raw_clip.close();
        let path_clip = WgpuPath {
            valid: true,
            raw_path: raw_clip,
            fill_rule: FillRule::NonZero,
        };
        let clip_rect = rect_path([28.0, 0.0, 40.0, 64.0], FillRule::NonZero);
        let path = rect_path([0.0, 0.0, 64.0, 64.0], FillRule::NonZero);
        let paint = WgpuPaint {
            color: 0xffff_0000,
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0);
        frame.clip_path(&path_clip);
        frame.clip_path(&clip_rect);
        frame.draw_path(&path, &paint);
        assert!(frame.state.clip_rect.is_none());
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(24, 40), [0; 4]);
        assert_eq!(pixel(32, 40), [255, 0, 0, 255]);
        assert_eq!(pixel(40, 40), [0; 4]);
    }

    #[test]
    fn msaa_feather_atlas_advanced_gradient_samples_ramp_and_destination() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let path = rect_path([8.0, 8.0, 56.0, 56.0], FillRule::Clockwise);
        let paint = WgpuPaint {
            feather: 4.0,
            blend_mode: BlendMode::Multiply,
            shader: Some(WgpuShader::Linear {
                start: (8.0, 8.0),
                end: (56.0, 56.0),
                colors: vec![0xffff_0000, 0xff00_00ff],
                stops: vec![0.0, 1.0],
            }),
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0xff20_80c0);
        frame.draw_path(&path, &paint);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert_eq!(pixel(0, 0), [32, 128, 192, 255]);
        assert!(pixel(12, 12)[0] > pixel(52, 52)[0]);
        assert!(pixel(52, 52)[2] > pixel(12, 12)[2]);
        assert!(pixel(32, 32)[3] > 0);
    }

    #[test]
    fn msaa_feather_atlas_gradient_stroke_samples_ramp() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let path = rect_path([8.0, 8.0, 56.0, 56.0], FillRule::Clockwise);
        let paint = WgpuPaint {
            style: RenderPaintStyle::Stroke,
            thickness: 8.0,
            feather: 4.0,
            shader: Some(WgpuShader::Linear {
                start: (8.0, 8.0),
                end: (56.0, 56.0),
                colors: vec![0xffff_0000, 0xff00_00ff],
                stops: vec![0.0, 1.0],
            }),
            ..WgpuPaint::default()
        };
        let mut frame = factory.begin_frame(0);
        frame.draw_path(&path, &paint);
        let pixels = frame.finish().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 64 + x) * 4..][..4];

        assert!(pixel(12, 12)[0] > pixel(12, 12)[2]);
        assert!(pixel(52, 52)[2] > pixel(52, 52)[0]);
        assert_eq!(pixel(32, 32), [0; 4]);
    }

    #[test]
    fn missing_image_mesh_is_a_noop_in_msaa() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let mut frame = factory.begin_frame(0);
        frame.draw_image_mesh(
            None,
            ImageSampler::default(),
            None,
            None,
            None,
            0,
            0,
            BlendMode::SrcOver,
            1.0,
        );

        assert!(frame.finish().unwrap().iter().all(|channel| *channel == 0));
    }

    #[test]
    fn msaa_feather_atlas_blit_uses_shader_advanced_blending() {
        let blit = advanced_feather_atlas_blit().unwrap();
        let pixels = blit.pixels();
        let center = &pixels[(32 * 64 + 32) * 4..][..4];
        let corner = &pixels[..4];
        assert_ne!(&center[..3], &corner[..3]);
        assert_eq!(center[3], 255);
        assert_eq!(corner[3], 255);
    }

    #[test]
    fn msaa_and_atomic_feather_advanced_blends_match_at_opaque_centers() {
        let msaa_factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let atomic_factory =
            WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let path = rect_path([8.0, 8.0, 56.0, 56.0], FillRule::Clockwise);
        for blend_mode in [
            BlendMode::Screen,
            BlendMode::Overlay,
            BlendMode::Darken,
            BlendMode::Lighten,
            BlendMode::ColorDodge,
            BlendMode::ColorBurn,
            BlendMode::HardLight,
            BlendMode::SoftLight,
            BlendMode::Difference,
            BlendMode::Exclusion,
            BlendMode::Multiply,
            BlendMode::Hue,
            BlendMode::Saturation,
            BlendMode::Color,
            BlendMode::Luminosity,
        ] {
            let render = |factory: &WgpuFactory, feather| {
                let paint = WgpuPaint {
                    color: 0x80c0_4020,
                    feather,
                    blend_mode,
                    ..WgpuPaint::default()
                };
                let mut frame = factory.begin_frame(0xff20_80c0);
                frame.draw_path(&path, &paint);
                let pixels = frame.finish().unwrap();
                <[u8; 4]>::try_from(&pixels[(32 * 64 + 32) * 4..][..4]).unwrap()
            };

            let atlas = render(&msaa_factory, 1.0);
            let atomic = render(&atomic_factory, 1.0);
            assert!(
                atlas
                    .iter()
                    .zip(atomic)
                    .all(|(left, right)| left.abs_diff(right) <= 1),
                "{blend_mode:?}: atlas={atlas:?} atomic={atomic:?}"
            );
        }
    }

    #[test]
    fn msaa_and_atomic_advanced_feather_atlas_draws_preserve_multiple_contributions() {
        let msaa_factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let atomic_factory =
            WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let left = rect_path([4.0, 8.0, 40.0, 56.0], FillRule::Clockwise);
        let right = rect_path([24.0, 8.0, 60.0, 56.0], FillRule::Clockwise);

        for blend_mode in [BlendMode::ColorDodge, BlendMode::Hue] {
            let render = |factory: &WgpuFactory| {
                let mut frame = factory.begin_frame(0xff20_4080);
                frame.draw_path(
                    &left,
                    &WgpuPaint {
                        color: 0xc0e0_8040,
                        feather: 32.0,
                        blend_mode,
                        ..WgpuPaint::default()
                    },
                );
                frame.draw_path(
                    &right,
                    &WgpuPaint {
                        color: 0xc040_c0e0,
                        feather: 32.0,
                        blend_mode,
                        ..WgpuPaint::default()
                    },
                );
                frame.finish().unwrap()
            };

            let msaa = render(&msaa_factory);
            let atomic = render(&atomic_factory);
            for [x, y] in [[16, 32], [32, 32], [48, 32]] {
                let offset = (y * 64 + x) * 4;
                let msaa_pixel = &msaa[offset..offset + 4];
                let atomic_pixel = &atomic[offset..offset + 4];
                assert!(
                    msaa_pixel
                        .iter()
                        .zip(atomic_pixel)
                        .all(|(left, right)| left.abs_diff(*right) <= 2),
                    "{blend_mode:?} at ({x}, {y}): msaa={msaa_pixel:?} atomic={atomic_pixel:?}"
                );
            }
        }
    }

    #[test]
    fn atomic_mixed_path_and_atlas_draws_preserve_order() {
        let factory = WgpuFactory::new_with_mode(256, 192, RenderMode::ClockwiseAtomic).unwrap();
        let path = rect_path([72.0, 16.0, 184.0, 176.0], FillRule::Clockwise);
        let background = 0xff64_6464;
        let mut frame = factory.begin_frame(0xff00_0000);
        frame.draw_path(
            &rect_path([0.0, 0.0, 256.0, 192.0], FillRule::NonZero),
            &WgpuPaint {
                color: background,
                ..WgpuPaint::default()
            },
        );
        frame.draw_path(
            &path,
            &WgpuPaint {
                color: 0x8053_5353,
                feather: 53.0,
                blend_mode: BlendMode::ColorDodge,
                ..WgpuPaint::default()
            },
        );
        let pixels = frame.finish().unwrap();
        let center = &pixels[(96 * 256 + 128) * 4..][..4];

        assert!(center[0] > 100, "atlas draw did not affect {center:?}");
        assert_eq!(center[0], center[1]);
        assert_eq!(center[1], center[2]);
        assert_eq!(center[3], 255);
    }

    #[test]
    fn atomic_atlas_draw_resolves_pending_hsl_path_with_combined_features() {
        let factory = WgpuFactory::new_with_mode(256, 192, RenderMode::ClockwiseAtomic).unwrap();
        let mut frame = factory.begin_frame(0xff29_2929);
        frame.draw_path(
            &rect_path([0.0, 0.0, 256.0, 192.0], FillRule::Clockwise),
            &WgpuPaint {
                color: 0xff00_0000,
                blend_mode: BlendMode::Hue,
                ..WgpuPaint::default()
            },
        );
        frame.draw_path(
            &rect_path([32.0, 48.0, 224.0, 144.0], FillRule::Clockwise),
            &WgpuPaint {
                color: 0x8010_070e,
                feather: 50.0,
                ..WgpuPaint::default()
            },
        );
        let pixels = frame.finish().unwrap();
        let center = &pixels[(96 * 256 + 128) * 4..][..4];

        assert!(
            center[0] > 20 && center[1] > 20 && center[2] > 20,
            "atlas draw resolved the pending HSL path without its shader feature: {center:?}"
        );
        assert_eq!(center[3], 255);
    }

    #[test]
    fn msaa_advanced_atlas_blends_preserve_path_clip_across_destination_copies() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let content = rect_path([8.0, 8.0, 56.0, 56.0], FillRule::Clockwise);
        let mut raw_clip = RawPath::new();
        raw_clip.move_to(32.0, 16.0);
        raw_clip.line_to(48.0, 48.0);
        raw_clip.line_to(16.0, 48.0);
        raw_clip.close();
        let clip = WgpuPath {
            valid: true,
            raw_path: raw_clip,
            fill_rule: FillRule::NonZero,
        };
        let first = WgpuPaint {
            color: 0x80c0_4020,
            feather: 1.0,
            blend_mode: BlendMode::ColorDodge,
            ..WgpuPaint::default()
        };
        let second = WgpuPaint {
            color: 0xa040_c080,
            feather: 1.0,
            blend_mode: BlendMode::Multiply,
            ..WgpuPaint::default()
        };
        let render = |draw_second| {
            let mut frame = factory.begin_frame(0xff20_80c0);
            frame.clip_path(&clip);
            frame.draw_path(&content, &first);
            if draw_second {
                frame.draw_path(&content, &second);
            }
            frame.finish().unwrap()
        };

        let single = render(false);
        let double = render(true);
        let pixel = |pixels: &[u8], x: usize, y: usize| {
            <[u8; 4]>::try_from(&pixels[(y * 64 + x) * 4..][..4]).unwrap()
        };
        assert_eq!(pixel(&single, 8, 8), [32, 128, 192, 255]);
        assert_eq!(pixel(&double, 8, 8), [32, 128, 192, 255]);
        assert_ne!(pixel(&single, 32, 32), pixel(&double, 32, 32));
    }

    #[test]
    fn msaa_logical_flush_replays_clips_and_preserves_destination_read_order() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::Msaa).unwrap();
        let content = rect_path([8.0, 8.0, 56.0, 56.0], FillRule::Clockwise);
        let mut raw_clip = RawPath::new();
        raw_clip.move_to(32.0, 16.0);
        raw_clip.line_to(48.0, 48.0);
        raw_clip.line_to(16.0, 48.0);
        raw_clip.close();
        let clip = WgpuPath {
            valid: true,
            raw_path: raw_clip,
            fill_rule: FillRule::NonZero,
        };
        let first = WgpuPaint {
            color: 0x80c0_4020,
            feather: 1.0,
            blend_mode: BlendMode::ColorDodge,
            ..WgpuPaint::default()
        };
        let second = WgpuPaint {
            color: 0xa040_c080,
            feather: 1.0,
            blend_mode: BlendMode::Multiply,
            ..WgpuPaint::default()
        };
        let make_frame = |force_rollover: bool| {
            let mut frame = factory.begin_frame(0xff20_80c0);
            frame.clip_path(&clip);
            frame.draw_path(&content, &first);
            if force_rollover {
                let used = frame.logical_flush.counters().path_count;
                assert!(used > 0 && used < logical_flush::MAX_PATH_COUNT);
                assert!(frame
                    .logical_flush
                    .push_draws(logical_flush::ResourceCounters {
                        path_count: logical_flush::MAX_PATH_COUNT - used,
                        ..Default::default()
                    }));
            }
            frame.draw_path(&content, &second);
            frame
        };

        let forced = make_frame(true);
        assert_eq!(forced.logical_flush_starts, [0, 2]);
        assert!(matches!(
            forced.draws[2].role,
            DrawRole::ClipUpdate {
                replacement_id: 1,
                parent_id: 0
            }
        ));
        assert!(matches!(
            forced.draws[3].role,
            DrawRole::Content { clip_id: 1 }
        ));

        let scheduled = forced.finish().unwrap();
        let serialized = make_frame(true)
            .finish_without_msaa_board_scheduling()
            .unwrap();
        let uninterrupted = make_frame(false).finish().unwrap();
        assert_eq!(scheduled, serialized);
        assert_ne!(scheduled, uninterrupted);
    }

    #[test]
    fn msaa_logical_flush_resolves_and_reloads_color_samples() {
        let factory = WgpuFactory::new_with_mode(32, 32, RenderMode::Msaa).unwrap();
        let edge = rect_path([0.0, 0.0, 10.5, 32.0], FillRule::NonZero);
        let red = WgpuPaint {
            color: 0xffff_0000,
            ..WgpuPaint::default()
        };
        let green = WgpuPaint {
            color: 0xff00_ff00,
            ..WgpuPaint::default()
        };
        let render = |force_rollover: bool| {
            let mut frame = factory.begin_frame(0x0000_0000);
            frame.draw_path(&edge, &red);
            if force_rollover {
                let used = frame.logical_flush.counters().path_count;
                assert!(frame
                    .logical_flush
                    .push_draws(logical_flush::ResourceCounters {
                        path_count: logical_flush::MAX_PATH_COUNT - used,
                        ..Default::default()
                    }));
            }
            frame.draw_path(&edge, &green);
            if force_rollover {
                assert_eq!(frame.logical_flush_starts, [0, 1]);
            }
            frame.finish().unwrap()
        };

        let uninterrupted = render(false);
        let rolled = render(true);
        let pixel = |pixels: &[u8], x: usize, y: usize| {
            <[u8; 4]>::try_from(&pixels[(y * 32 + x) * 4..][..4]).unwrap()
        };
        assert_eq!(pixel(&uninterrupted, 4, 16), [0, 255, 0, 255]);
        assert_eq!(pixel(&rolled, 4, 16), [0, 255, 0, 255]);
        assert_eq!(pixel(&uninterrupted, 10, 16), [0, 128, 0, 128]);
        assert_eq!(pixel(&rolled, 10, 16), [64, 128, 0, 192]);
    }

    #[test]
    fn real_msaa_stroke_accounting_reaches_the_cpp_path_boundary() {
        let draw = SolidDraw {
            path: rect_path([4.0, 4.0, 60.0, 60.0], FillRule::NonZero),
            paint: WgpuPaint {
                style: RenderPaintStyle::Stroke,
                thickness: 1.0,
                ..WgpuPaint::default()
            },
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };
        let resources = logical_flush_draw_resources(&draw, RenderMode::Msaa, 64, 64);
        assert_eq!(resources.path_count, 1);
        assert_eq!(resources.draw_pass_count, 1);

        let mut flush = logical_flush::LogicalFlush::default();
        for _ in 0..logical_flush::MAX_PATH_COUNT {
            assert!(flush.push_draws(resources));
        }
        assert!(!flush.push_draws(resources));
    }

    #[test]
    fn logical_flush_allocations_bound_complex_gradient_rows() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let frame = factory.begin_frame(0);
        let mut allocations = LogicalFlushAllocations::default();
        let mut draw = SolidDraw {
            path: rect_path([4.0, 4.0, 60.0, 60.0], FillRule::NonZero),
            paint: WgpuPaint::default(),
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };
        for index in 0..2048u32 {
            draw.paint.shader = Some(WgpuShader::Linear {
                start: (0.0, 0.0),
                end: (64.0, 64.0),
                colors: vec![0xff00_0000 | index, 0xff00_ff00, 0xff00_00ff],
                stops: vec![0.0, 0.5, 1.0],
            });
            allocations = allocations
                .with_batch(&frame, std::slice::from_ref(&draw))
                .unwrap();
        }
        assert_eq!(allocations.complex_gradient_count, 2048);
        assert!(allocations
            .with_batch(&frame, std::slice::from_ref(&draw))
            .is_err());
        assert!(LogicalFlushAllocations::default()
            .with_batch(&frame, std::slice::from_ref(&draw))
            .is_ok());
    }

    #[test]
    fn logical_flush_allocations_roll_atlas_and_coverage_independently() {
        for mode in [RenderMode::ClockwiseAtomic, RenderMode::Msaa] {
            let atlas_factory = WgpuFactory::new_with_mode(64, 64, mode).unwrap();
            let atlas_frame = atlas_factory.begin_frame(0);
            let atlas_draw = SolidDraw {
                path: rect_path([0.0, 0.0, 64.0, 64.0], FillRule::Clockwise),
                paint: WgpuPaint {
                    feather: 32.0,
                    ..WgpuPaint::default()
                },
                state: DrawState::default(),
                role: DrawRole::Content { clip_id: 0 },
                image: None,
            };
            let mut atlas = LogicalFlushAllocations::default();
            let atlas_count = (1..10_000)
                .find(|_| {
                    let Ok(next) =
                        atlas.with_batch(&atlas_frame, std::slice::from_ref(&atlas_draw))
                    else {
                        return true;
                    };
                    atlas = next;
                    false
                })
                .expect("atlas allocation must reach the device texture limit");
            assert!(atlas_count > 1, "{mode:?} rolled before one atlas draw");
            assert!(LogicalFlushAllocations::default()
                .with_batch(&atlas_frame, std::slice::from_ref(&atlas_draw))
                .is_ok());
        }

        let coverage_factory =
            WgpuFactory::new_with_mode(1024, 1024, RenderMode::ClockwiseAtomic).unwrap();
        let coverage_frame = coverage_factory.begin_frame(0);
        let mut raw_path = RawPath::new();
        raw_path.move_to(0.0, 0.0);
        raw_path.line_to(1024.0, 1024.0);
        raw_path.line_to(1024.0, 0.0);
        raw_path.line_to(0.0, 1024.0);
        raw_path.close();
        let coverage_draw = SolidDraw {
            path: WgpuPath {
                valid: true,
                raw_path,
                fill_rule: FillRule::Clockwise,
            },
            paint: WgpuPaint::default(),
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 0 },
            image: None,
        };
        assert!(draw_requires_clockwise_atomic(
            &coverage_draw,
            coverage_frame.width,
            coverage_frame.height
        ));
        let mut coverage = LogicalFlushAllocations::default();
        let coverage_count = (1..1_000)
            .find(|_| {
                let Ok(next) =
                    coverage.with_batch(&coverage_frame, std::slice::from_ref(&coverage_draw))
                else {
                    return true;
                };
                coverage = next;
                false
            })
            .expect("coverage allocation must reach the storage-buffer limit");
        assert!(coverage_count > 1);
        assert!(LogicalFlushAllocations::default()
            .with_batch(&coverage_frame, std::slice::from_ref(&coverage_draw))
            .is_ok());
    }

    #[test]
    fn logical_flush_allocates_nested_clip_inverse_coverage() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let frame = factory.begin_frame(0);
        let draw = SolidDraw {
            path: rect_path([-20.0, -20.0, -10.0, -10.0], FillRule::Clockwise),
            paint: WgpuPaint::default(),
            state: DrawState::default(),
            role: DrawRole::ClipUpdate {
                replacement_id: 2,
                parent_id: 1,
            },
            image: None,
        };

        let allocations = LogicalFlushAllocations::default()
            .with_batch(&frame, std::slice::from_ref(&draw))
            .unwrap();
        assert_eq!(allocations.coverage_word_count, 96 * 96);
    }

    #[test]
    fn logical_flush_coverage_includes_visible_stroke_outset() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let frame = factory.begin_frame(0);
        let draw = SolidDraw {
            path: rect_path([-20.0, -20.0, -10.0, -10.0], FillRule::Clockwise),
            paint: WgpuPaint {
                style: RenderPaintStyle::Stroke,
                thickness: 24.0,
                ..WgpuPaint::default()
            },
            state: DrawState::default(),
            role: DrawRole::Content { clip_id: 1 },
            image: None,
        };

        let allocations = LogicalFlushAllocations::default()
            .with_batch(&frame, std::slice::from_ref(&draw))
            .unwrap();
        assert_eq!(allocations.coverage_word_count, 64 * 64);
    }

    #[test]
    fn singular_nested_clip_is_empty_instead_of_unsupported() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let mut frame = factory.begin_frame(0);
        frame.clips = vec![
            ClipElement {
                path: rect_path([0.0, 0.0, 64.0, 64.0], FillRule::Clockwise),
                matrix: Mat2D::IDENTITY,
                clip_id: 0,
            },
            ClipElement {
                path: rect_path([8.0, 8.0, 56.0, 56.0], FillRule::Clockwise),
                matrix: Mat2D([0.0; 6]),
                clip_id: 0,
            },
        ];
        frame.state.clip_stack_height = 2;

        assert!(frame.prepare_clip_updates().is_none());
        assert!(frame.unsupported.is_none());
    }

    #[test]
    fn atomic_logical_flush_replays_the_active_clip_stack() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let content = rect_path([8.0, 8.0, 56.0, 56.0], FillRule::Clockwise);
        let mut raw_clip = RawPath::new();
        raw_clip.move_to(32.0, 16.0);
        raw_clip.line_to(48.0, 48.0);
        raw_clip.line_to(16.0, 48.0);
        raw_clip.close();
        let clip = WgpuPath {
            valid: true,
            raw_path: raw_clip,
            fill_rule: FillRule::NonZero,
        };
        let paint = WgpuPaint {
            color: 0x80ff_0000,
            ..WgpuPaint::default()
        };
        let render = |force_rollover: bool| {
            let mut frame = factory.begin_frame(0xff20_80c0);
            frame.clip_path(&clip);
            frame.draw_path(&content, &paint);
            if force_rollover {
                let used = frame.logical_flush.counters().path_count;
                assert!(frame
                    .logical_flush
                    .push_draws(logical_flush::ResourceCounters {
                        path_count: logical_flush::MAX_PATH_COUNT - used,
                        ..Default::default()
                    }));
            }
            frame.draw_path(&content, &paint);
            frame
        };

        let forced = render(true);
        let boundary = forced.logical_flush_starts[1];
        assert!(matches!(
            forced.draws[boundary].role,
            DrawRole::ClipUpdate {
                replacement_id: 1,
                parent_id: 0
            }
        ));
        assert_eq!(forced.finish().unwrap(), render(false).finish().unwrap());
    }

    #[test]
    fn feather_atlas_stroke_pass_writes_r16_coverage() {
        let mask = fixed_feather_atlas_mask(ATLAS_ORACLE_STROKE_JOIN);
        let mask_value = |x: usize, y: usize| mask.sample_bits(x, y);
        let size = ATLAS_ORACLE_PHYSICAL_SIZE as usize;
        assert_eq!(mask_value(size - 1, size - 1), 0);
        let logical_size = ATLAS_ORACLE_LOGICAL_SIZE as usize;
        let nonzero = (0..logical_size)
            .flat_map(|y| (0..logical_size).map(move |x| (x, y)))
            .filter(|&(x, y)| mask_value(x, y) != 0 && mask_value(x, y) & 0x8000 == 0)
            .collect::<Vec<_>>();
        assert!(!nonzero.is_empty(), "atlas stroke mask is empty");
        for y in 0..size {
            for x in 0..size {
                if x >= logical_size || y >= logical_size {
                    assert_eq!(mask_value(x, y), 0, "uncleared atlas tail at ({x}, {y})");
                }
            }
        }
    }

    #[test]
    fn feather_atlas_empty_round_stroke_preserves_cpp_patch_and_center_coverage() {
        let oracle = fixed_feather_atlas_empty_stroke_oracle();
        assert_eq!(oracle.inputs.base_patch, 1);
        assert_eq!(oracle.inputs.patch_count, 5);
        assert_eq!(oracle.inputs.contours.len(), 1);
        assert_eq!(oracle.mask.sample_bits(19, 19), 0x34f1);
    }

    #[test]
    fn large_radius_feather_atlas_oracles_cover_the_two_residual_contours() {
        for (case, expected_bounds, expected_translate_bits, expected_patches) in [
            (
                LargeFeatherAtlasCase::Cusp,
                [0, 978, 1691, 2048],
                [2.0f32.to_bits(), (-15.674875f32).to_bits()],
                41,
            ),
            (
                LargeFeatherAtlasCase::ShapesCusp,
                [42, 942, 1756, 2048],
                [1.2409563f32.to_bits(), 0xc170_6365],
                59,
            ),
        ] {
            let oracle = large_feather_atlas_oracle(case);
            assert_eq!(oracle.placement.bounds, expected_bounds);
            assert_eq!(oracle.placement.origin, [0, 0]);
            assert_eq!(oracle.placement.content_size, [35, 24]);
            assert_eq!(oracle.placement.physical_size, [43, 30]);
            assert_eq!(oracle.placement.translate_bits, expected_translate_bits);
            assert_eq!(oracle.placement.scissor, [0, 0, 35, 24]);
            assert!((0..oracle.placement.physical_size[1] as usize).any(|y| {
                (0..oracle.placement.physical_size[0] as usize)
                    .any(|x| oracle.mask.sample_bits(x, y) != 0)
            }));
            assert!(!oracle.inputs.contours.is_empty());
            assert_eq!(oracle.inputs.patch_count, expected_patches);
        }
    }

    #[test]
    #[ignore = "requires the paired RIVE_CPP_ATLAS_LARGE_FEATHER_* C++ WebGPU oracle artifacts"]
    fn cpp_webgpu_large_radius_feather_atlas_stages_match_rust_when_configured() {
        for (case, prefix) in [
            (
                LargeFeatherAtlasCase::Cusp,
                "RIVE_CPP_ATLAS_LARGE_FEATHER_CUSP",
            ),
            (
                LargeFeatherAtlasCase::ShapesCusp,
                "RIVE_CPP_ATLAS_LARGE_FEATHER_SHAPES_CUSP",
            ),
        ] {
            let artifact = |suffix: &str| {
                let name = format!("{prefix}_{suffix}");
                let path = PathBuf::from(
                    std::env::var_os(&name)
                        .unwrap_or_else(|| panic!("{name} is required for the ignored test")),
                );
                assert!(path.is_absolute(), "{name} must be an absolute path");
                fs::read(&path).unwrap_or_else(|error| {
                    panic!("failed to read {name} at {}: {error}", path.display())
                })
            };
            let cpp_placement =
                atlas_placement_oracle::AtlasPlacement::parse(&artifact("PLACEMENT")).unwrap();
            let cpp_inputs = atlas_input_oracle::AtlasInputs::parse(&artifact("INPUTS")).unwrap();
            let cpp_mask = atlas_mask_oracle::AtlasMask::parse(&artifact("MASK")).unwrap();
            let cpp_blit = atlas_blit_oracle::AtlasBlit::parse(&artifact("BLIT")).unwrap();
            let rust_oracle = large_feather_atlas_oracle(case);

            assert_eq!(
                cpp_placement, rust_oracle.placement,
                "{prefix} placement diverged"
            );
            atlas_input_oracle::compare_cpp_to_rust_with_float_tolerances(
                &cpp_inputs,
                &rust_oracle.inputs,
                4,
                0.01,
            )
            .unwrap_or_else(|error| panic!("{prefix} tessellation inputs diverged: {error}"));
            atlas_mask_oracle::compare_cpp_to_rust_signed(
                &cpp_mask,
                &rust_oracle.mask,
                // 2^-10 rejects a real ShapesCusp sample; 2^-9 is the
                // smallest tested power-of-two budget that accepts both masks.
                1.0 / 512.0,
            )
            .unwrap_or_else(|error| panic!("{prefix} R16 atlas mask diverged: {error}"));

            let rust_blit = large_feather_atlas_blit(case);
            atlas_blit_oracle::compare_cpp_to_rust_with_pixel_tolerance(
                &cpp_blit, &rust_blit, 2, 32,
            )
            .unwrap_or_else(|error| panic!("{prefix} final MSAA sampling diverged: {error}"));
        }
    }

    #[test]
    fn msaa_feather_atlas_empty_stroke_uses_scheduled_depth_over_prior_draw() {
        let blit = fixed_feather_atlas_empty_stroke_blit();
        let center = (ATLAS_ORACLE_FRAME_SIZE / 2) as usize;
        let offset = (center * ATLAS_ORACLE_FRAME_SIZE as usize + center) * 4;
        let pixel = &blit.pixels()[offset..offset + 4];
        assert_eq!(pixel[3], 255);
        assert_eq!(pixel[1], pixel[2]);
        assert!(
            pixel[1] > 0,
            "atlas cap coverage must pass depth over the earlier red marker: {pixel:?}"
        );
    }

    #[test]
    fn feather_atlas_fill_pass_writes_r16_coverage() {
        let mask = fixed_feather_atlas_fill_oracle().mask;
        assert!((0..ATLAS_ORACLE_PHYSICAL_SIZE as usize).any(|y| {
            (0..ATLAS_ORACLE_PHYSICAL_SIZE as usize).any(|x| mask.sample_bits(x, y) != 0)
        }));
    }

    #[test]
    fn feather_atlas_stroke_uses_the_same_mask_for_all_requested_joins() {
        let miter = fixed_feather_atlas_mask(StrokeJoin::Miter);
        let bevel = fixed_feather_atlas_mask(StrokeJoin::Bevel);

        assert_eq!(miter, bevel);
    }

    #[test]
    fn atlas_input_oracle_detects_fixed_stroke_sensitivity() {
        let oracle = fixed_feather_atlas_oracle(ATLAS_ORACLE_STROKE_JOIN);
        let inputs = oracle.inputs;

        let mut batch = inputs.clone();
        batch.patch_count ^= 1;
        assert_eq!(
            atlas_input_oracle::compare_cpp_to_rust(&inputs, &batch),
            Err(
                atlas_input_oracle::AtlasInputComparisonError::BatchOrDimensionField {
                    field: "patch_count",
                    cpp: inputs.patch_count,
                    rust: batch.patch_count,
                }
            )
        );

        let mut contour = inputs.clone();
        contour.contours[0].vertex_index0 ^= 1;
        assert_eq!(
            atlas_input_oracle::compare_cpp_to_rust(&inputs, &contour),
            Err(
                atlas_input_oracle::AtlasInputComparisonError::ContourField {
                    index: 0,
                    field: "vertex_index0",
                    cpp: inputs.contours[0].vertex_index0,
                    rust: contour.contours[0].vertex_index0,
                }
            )
        );

        let used_texel = inputs
            .texels
            .iter()
            .position(|texel| *texel != [0; 4])
            .expect("fixed stroke tessellation must write a texel");
        let mut tessellation = inputs.clone();
        tessellation.texels[used_texel][0] ^= 1;
        assert_eq!(
            atlas_input_oracle::compare_cpp_to_rust(&inputs, &tessellation),
            Err(atlas_input_oracle::AtlasInputComparisonError::Texel {
                x: (used_texel % inputs.tess_width as usize) as u32,
                y: (used_texel / inputs.tess_width as usize) as u32,
                channel: 0,
                cpp: inputs.texels[used_texel][0],
                rust: tessellation.texels[used_texel][0],
            })
        );
    }

    #[test]
    fn atlas_tessellation_writes_the_final_post_contour_padding_sentinel() {
        let inputs = fixed_feather_atlas_oracle(ATLAS_ORACLE_STROKE_JOIN).inputs;
        let logical_end =
            (inputs.base_patch + inputs.patch_count) * gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let alignment = gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32;
        let final_index = (logical_end.div_ceil(alignment) * alignment) as usize;
        assert!(inputs.texels[logical_end as usize..=final_index]
            .iter()
            .all(|texel| *texel == [0, 0, 0x4049_0fdb, 0x0008_0000]));
        assert!(inputs.texels[final_index + 1..]
            .iter()
            .all(|texel| *texel == [0; 4]));
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_MASK from the C++ WebGPU oracle"]
    fn cpp_webgpu_atlas_mask_oracle_matches_fixed_rust_mask_when_configured() {
        let path = std::env::var_os("RIVE_CPP_ATLAS_MASK")
            .expect("RIVE_CPP_ATLAS_MASK is required for the ignored C++ atlas-mask oracle test");
        assert!(
            !path.is_empty(),
            "RIVE_CPP_ATLAS_MASK is set but empty; set it to a C++ atlas-mask oracle file"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_MASK must be absolute because Cargo runs unit tests from the package directory"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ atlas-mask oracle at {}: {error}",
                path.display()
            )
        });
        let cpp_mask = atlas_mask_oracle::AtlasMask::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ atlas-mask oracle at {}: {error}",
                path.display()
            )
        });
        let rust_mask = fixed_feather_atlas_mask(ATLAS_ORACLE_STROKE_JOIN);
        atlas_mask_oracle::compare_cpp_to_rust(&cpp_mask, &rust_mask, ATLAS_ORACLE_TOLERANCES)
            .unwrap_or_else(|error| {
                panic!(
                    "C++ atlas-mask oracle mismatch at {}: {error}",
                    path.display()
                )
            });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_INPUTS from the C++ WebGPU oracle"]
    fn cpp_webgpu_atlas_input_oracle_matches_fixed_rust_inputs_when_configured() {
        let path = std::env::var_os("RIVE_CPP_ATLAS_INPUTS").expect(
            "RIVE_CPP_ATLAS_INPUTS is required for the ignored C++ atlas-input oracle test",
        );
        assert!(
            !path.is_empty(),
            "RIVE_CPP_ATLAS_INPUTS is set but empty; set it to a C++ atlas-input oracle file"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_INPUTS must be absolute because Cargo runs unit tests from the package directory"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ atlas-input oracle at {}: {error}",
                path.display()
            )
        });
        let cpp_inputs = atlas_input_oracle::AtlasInputs::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ atlas-input oracle at {}: {error}",
                path.display()
            )
        });
        let rust_inputs = fixed_feather_atlas_oracle(ATLAS_ORACLE_STROKE_JOIN).inputs;
        atlas_input_oracle::compare_cpp_to_rust(&cpp_inputs, &rust_inputs).unwrap_or_else(
            |error| {
                panic!(
                    "C++ atlas-input oracle mismatch at {}: {error}",
                    path.display()
                )
            },
        );
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_EMPTY_STROKE_MASK from the C++ WebGPU oracle"]
    fn cpp_webgpu_atlas_empty_stroke_mask_matches_rust_when_configured() {
        let path = PathBuf::from(
            std::env::var_os("RIVE_CPP_ATLAS_EMPTY_STROKE_MASK")
                .expect("RIVE_CPP_ATLAS_EMPTY_STROKE_MASK is required"),
        );
        assert!(path.is_absolute());
        let cpp_mask =
            atlas_mask_oracle::AtlasMask::parse(&fs::read(&path).unwrap_or_else(|error| {
                panic!(
                    "failed to read C++ empty-stroke mask at {}: {error}",
                    path.display()
                )
            }))
            .unwrap_or_else(|error| {
                panic!(
                    "malformed C++ empty-stroke mask at {}: {error}",
                    path.display()
                )
            });
        let rust_mask = fixed_feather_atlas_empty_stroke_oracle().mask;
        atlas_mask_oracle::compare_cpp_to_rust(&cpp_mask, &rust_mask, ATLAS_ORACLE_TOLERANCES)
            .unwrap_or_else(|error| {
                panic!(
                    "C++ empty-stroke mask mismatch at {}: {error}",
                    path.display()
                )
            });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_EMPTY_STROKE_INPUTS from the C++ WebGPU oracle"]
    fn cpp_webgpu_atlas_empty_stroke_inputs_match_rust_when_configured() {
        let path = PathBuf::from(
            std::env::var_os("RIVE_CPP_ATLAS_EMPTY_STROKE_INPUTS")
                .expect("RIVE_CPP_ATLAS_EMPTY_STROKE_INPUTS is required"),
        );
        assert!(path.is_absolute());
        let cpp_inputs =
            atlas_input_oracle::AtlasInputs::parse(&fs::read(&path).unwrap_or_else(|error| {
                panic!(
                    "failed to read C++ empty-stroke inputs at {}: {error}",
                    path.display()
                )
            }))
            .unwrap_or_else(|error| {
                panic!(
                    "malformed C++ empty-stroke inputs at {}: {error}",
                    path.display()
                )
            });
        let rust_inputs = fixed_feather_atlas_empty_stroke_oracle().inputs;
        atlas_input_oracle::compare_cpp_to_rust(&cpp_inputs, &rust_inputs).unwrap_or_else(
            |error| {
                panic!(
                    "C++ empty-stroke input mismatch at {}: {error}",
                    path.display()
                )
            },
        );
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_FILL_MASK from the C++ WebGPU oracle"]
    fn cpp_webgpu_atlas_fill_mask_oracle_matches_fixed_rust_mask_when_configured() {
        let path = std::env::var_os("RIVE_CPP_ATLAS_FILL_MASK").expect(
            "RIVE_CPP_ATLAS_FILL_MASK is required for the ignored C++ atlas-fill mask test",
        );
        assert!(!path.is_empty(), "RIVE_CPP_ATLAS_FILL_MASK is empty");
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_FILL_MASK must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ atlas-fill mask at {}: {error}",
                path.display()
            )
        });
        let cpp_mask = atlas_mask_oracle::AtlasMask::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ atlas-fill mask at {}: {error}",
                path.display()
            )
        });
        let rust_mask = fixed_feather_atlas_fill_oracle().mask;
        atlas_mask_oracle::compare_cpp_to_rust(&cpp_mask, &rust_mask, ATLAS_ORACLE_TOLERANCES)
            .unwrap_or_else(|error| {
                panic!(
                    "C++ atlas-fill mask mismatch at {}: {error}",
                    path.display()
                )
            });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_FILL_INPUTS from the C++ WebGPU oracle"]
    fn cpp_webgpu_atlas_fill_input_oracle_matches_fixed_rust_inputs_when_configured() {
        let path = std::env::var_os("RIVE_CPP_ATLAS_FILL_INPUTS").expect(
            "RIVE_CPP_ATLAS_FILL_INPUTS is required for the ignored C++ atlas-fill input test",
        );
        assert!(!path.is_empty(), "RIVE_CPP_ATLAS_FILL_INPUTS is empty");
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_FILL_INPUTS must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ atlas-fill inputs at {}: {error}",
                path.display()
            )
        });
        let cpp_inputs = atlas_input_oracle::AtlasInputs::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ atlas-fill inputs at {}: {error}",
                path.display()
            )
        });
        let rust_inputs = fixed_feather_atlas_fill_oracle().inputs;
        atlas_input_oracle::compare_cpp_to_rust_with_position_ulps(&cpp_inputs, &rust_inputs, 1)
            .unwrap_or_else(|error| {
                panic!(
                    "C++ atlas-fill input mismatch at {}: {error}",
                    path.display()
                )
            });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_CUSP_MASK from the C++ WebGPU oracle"]
    fn cpp_webgpu_atlas_cusp_mask_oracle_matches_fixed_rust_mask_when_configured() {
        let path = std::env::var_os("RIVE_CPP_ATLAS_CUSP_MASK").expect(
            "RIVE_CPP_ATLAS_CUSP_MASK is required for the ignored C++ atlas-cusp mask test",
        );
        assert!(!path.is_empty(), "RIVE_CPP_ATLAS_CUSP_MASK is empty");
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_CUSP_MASK must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ atlas-cusp mask at {}: {error}",
                path.display()
            )
        });
        let cpp_mask = atlas_mask_oracle::AtlasMask::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ atlas-cusp mask at {}: {error}",
                path.display()
            )
        });
        let rust_mask = fixed_feather_atlas_cusp_oracle().mask;
        atlas_mask_oracle::compare_cpp_to_rust(&cpp_mask, &rust_mask, ATLAS_ORACLE_TOLERANCES)
            .unwrap_or_else(|error| {
                panic!(
                    "C++ atlas-cusp mask mismatch at {}: {error}",
                    path.display()
                )
            });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_CUSP_INPUTS from the C++ WebGPU oracle"]
    fn cpp_webgpu_atlas_cusp_input_oracle_matches_fixed_rust_inputs_when_configured() {
        let path = std::env::var_os("RIVE_CPP_ATLAS_CUSP_INPUTS").expect(
            "RIVE_CPP_ATLAS_CUSP_INPUTS is required for the ignored C++ atlas-cusp input test",
        );
        assert!(!path.is_empty(), "RIVE_CPP_ATLAS_CUSP_INPUTS is empty");
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_CUSP_INPUTS must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ atlas-cusp inputs at {}: {error}",
                path.display()
            )
        });
        let cpp_inputs = atlas_input_oracle::AtlasInputs::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ atlas-cusp inputs at {}: {error}",
                path.display()
            )
        });
        let rust_inputs = fixed_feather_atlas_cusp_oracle().inputs;
        atlas_input_oracle::compare_cpp_to_rust_with_float_tolerances(
            &cpp_inputs,
            &rust_inputs,
            4,
            0.0001,
        )
        .unwrap_or_else(|error| {
            panic!(
                "C++ atlas-cusp input mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_SOFTENED_CUSP from the C++ oracle"]
    fn cpp_softened_cusp_path_oracle_matches_rust_when_configured() {
        let path = std::env::var_os("RIVE_CPP_SOFTENED_CUSP")
            .expect("RIVE_CPP_SOFTENED_CUSP is required for the ignored softened-cusp path test");
        assert!(!path.is_empty(), "RIVE_CPP_SOFTENED_CUSP is empty");
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_SOFTENED_CUSP must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ softened-cusp path at {}: {error}",
                path.display()
            )
        });
        assert!(bytes.len() >= 20, "softened-cusp header is truncated");
        assert_eq!(&bytes[..8], b"RIVESFT\0");
        let read_u32 = |offset| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        assert_eq!(read_u32(8), 1, "unsupported softened-cusp version");
        let verb_count = read_u32(12) as usize;
        let point_count = read_u32(16) as usize;
        assert_eq!(bytes.len(), 20 + verb_count * 4 + point_count * 8);
        let cpp_verbs = (0..verb_count)
            .map(|index| read_u32(20 + index * 4))
            .collect::<Vec<_>>();
        let points_offset = 20 + verb_count * 4;
        let cpp_points = (0..point_count)
            .map(|index| {
                let offset = points_offset + index * 8;
                [read_u32(offset), read_u32(offset + 4)]
            })
            .collect::<Vec<_>>();

        let mut source = RawPath::new();
        source.move_to(0.0, 100.0);
        source.move_to(0.0, 100.0);
        source.cubic_to(110.0, 0.0, -10.0, 0.0, 100.0, 100.0);
        let softened = draw::softened_path_for_feathering(&source, 1.5, 1.46300006);
        let rust_verbs = softened
            .verbs()
            .iter()
            .map(|verb| match verb {
                nuxie_render_api::PathVerb::Move => 0,
                nuxie_render_api::PathVerb::Line => 1,
                nuxie_render_api::PathVerb::Quad => 2,
                nuxie_render_api::PathVerb::Cubic => 4,
                nuxie_render_api::PathVerb::Close => 5,
            })
            .collect::<Vec<_>>();
        let rust_points = softened
            .points()
            .iter()
            .map(|point| [point.x.to_bits(), point.y.to_bits()])
            .collect::<Vec<_>>();
        assert_eq!(cpp_verbs, rust_verbs, "softened-cusp verbs differ");
        assert_eq!(cpp_points.len(), rust_points.len());
        for (index, (cpp_point, rust_point)) in cpp_points.iter().zip(&rust_points).enumerate() {
            for channel in 0..2 {
                assert!(
                    atlas_input_oracle::float_bits_within_ulps(
                        cpp_point[channel],
                        rust_point[channel],
                        2,
                    ),
                    "softened-cusp point {index} channel {channel} differs by more than 2 ULP: C++={:#010x}, Rust={:#010x}",
                    cpp_point[channel],
                    rust_point[channel],
                );
            }
        }
    }

    #[test]
    #[ignore = "requires RIVE_CPP_DIRECT_CUSP_INPUTS from the C++ WebGPU oracle"]
    fn cpp_webgpu_direct_cusp_input_oracle_matches_rust_when_configured() {
        let path = std::env::var_os("RIVE_CPP_DIRECT_CUSP_INPUTS").expect(
            "RIVE_CPP_DIRECT_CUSP_INPUTS is required for the ignored direct-cusp input test",
        );
        assert!(!path.is_empty(), "RIVE_CPP_DIRECT_CUSP_INPUTS is empty");
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_DIRECT_CUSP_INPUTS must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ direct-cusp inputs at {}: {error}",
                path.display()
            )
        });
        let cpp_inputs = atlas_input_oracle::AtlasInputs::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ direct-cusp inputs at {}: {error}",
                path.display()
            )
        });
        let rust_inputs = fixed_feather_direct_cusp_inputs();
        atlas_input_oracle::compare_cpp_to_rust_with_float_tolerances(
            &cpp_inputs,
            &rust_inputs,
            4,
            0.0001,
        )
        .unwrap_or_else(|error| {
            panic!(
                "C++ direct-cusp input mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_DIRECT_POLYSHARK_INPUTS from the C++ WebGPU oracle"]
    fn cpp_webgpu_direct_polyshark_input_oracle_matches_rust_when_configured() {
        let path = std::env::var_os("RIVE_CPP_DIRECT_POLYSHARK_INPUTS").expect(
            "RIVE_CPP_DIRECT_POLYSHARK_INPUTS is required for the ignored direct-polyshark input test",
        );
        assert!(
            !path.is_empty(),
            "RIVE_CPP_DIRECT_POLYSHARK_INPUTS is empty"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_DIRECT_POLYSHARK_INPUTS must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ direct-polyshark inputs at {}: {error}",
                path.display()
            )
        });
        let mut cpp_inputs =
            atlas_input_oracle::AtlasInputs::parse(&bytes).unwrap_or_else(|error| {
                panic!(
                    "malformed C++ direct-polyshark inputs at {}: {error}",
                    path.display()
                )
            });
        let rust_inputs = fixed_feather_direct_polyshark_inputs();
        const JOIN_SIDE_MASK: u32 = 1 << 20 | 1 << 19;
        const JOIN_TYPE_MASK: u32 = 7 << 26;
        const FEATHER_JOIN: u32 = 1 << 26;
        for (cpp, rust) in cpp_inputs.texels.iter_mut().zip(&rust_inputs.texels) {
            if cpp[3] ^ rust[3] == JOIN_SIDE_MASK
                && (cpp[3] & JOIN_SIDE_MASK).count_ones() == 1
                && (rust[3] & JOIN_SIDE_MASK).count_ones() == 1
                && cpp[3] & JOIN_TYPE_MASK == FEATHER_JOIN
                && rust[3] & JOIN_TYPE_MASK == FEATHER_JOIN
            {
                cpp[3] = rust[3];
            }
        }
        atlas_input_oracle::compare_cpp_to_rust_with_float_tolerances(
            &cpp_inputs,
            &rust_inputs,
            4,
            0.0001,
        )
        .unwrap_or_else(|error| {
            panic!(
                "C++ direct-polyshark input mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_DIRECT_STROKES_ROUND_INPUTS from the C++ WebGPU oracle"]
    fn cpp_webgpu_direct_strokes_round_tessellation_matches_bounded_tangent_angles() {
        let path = std::env::var_os("RIVE_CPP_DIRECT_STROKES_ROUND_INPUTS").expect(
            "RIVE_CPP_DIRECT_STROKES_ROUND_INPUTS is required for the ignored direct-strokes-round input test",
        );
        assert!(
            !path.is_empty(),
            "RIVE_CPP_DIRECT_STROKES_ROUND_INPUTS is empty"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_DIRECT_STROKES_ROUND_INPUTS must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ direct-strokes-round inputs at {}: {error}",
                path.display()
            )
        });
        let cpp_inputs = atlas_input_oracle::AtlasInputs::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ direct-strokes-round inputs at {}: {error}",
                path.display()
            )
        });
        let rust_inputs = fixed_strokes_round_direct_inputs();
        assert_eq!(cpp_inputs.base_patch, 1);
        assert_eq!(cpp_inputs.patch_count, 10);
        assert_eq!(cpp_inputs.contours, rust_inputs.contours);
        atlas_input_oracle::compare_cpp_to_rust_with_float_tolerances(
            &cpp_inputs,
            &rust_inputs,
            0,
            0.00035,
        )
        .unwrap_or_else(|error| {
            panic!(
                "C++ direct-strokes-round input mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_DIRECT_STROKES_ROUND_SPANS from the C++ WebGPU oracle"]
    fn cpp_direct_strokes_round_cpu_spans_match_rust_record_for_record() {
        let path = std::env::var_os("RIVE_CPP_DIRECT_STROKES_ROUND_SPANS").expect(
            "RIVE_CPP_DIRECT_STROKES_ROUND_SPANS is required for the ignored direct-strokes-round span test",
        );
        assert!(
            !path.is_empty(),
            "RIVE_CPP_DIRECT_STROKES_ROUND_SPANS is empty"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_DIRECT_STROKES_ROUND_SPANS must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ direct-strokes-round spans at {}: {error}",
                path.display()
            )
        });
        let cpp_spans = tess_span_oracle::TessSpanArtifact::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ direct-strokes-round spans at {}: {error}",
                path.display()
            )
        });
        let rust_spans = fixed_strokes_round_spans();
        tess_span_oracle::compare_exact(&cpp_spans, &rust_spans).unwrap_or_else(|error| {
            panic!(
                "C++ direct-strokes-round CPU span mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_DIRECT_OVERSTROKE_QUAD_SPANS from the C++ WebGPU oracle"]
    fn cpp_direct_overstroke_quad_cpu_spans_match_rust_record_for_record() {
        fn summarize(artifact: &tess_span_oracle::TessSpanArtifact) -> String {
            artifact
                .records
                .iter()
                .enumerate()
                .map(|(index, record)| {
                    let x0 = record[12] as u16 as i16;
                    let x1 = (record[12] >> 16) as u16 as i16;
                    let parametric = record[14] & 0x3ff;
                    let polar = record[14] >> 10 & 0x3ff;
                    let join = record[14] >> 20 & 0x3ff;
                    format!("{index}:x={x0}..{x1},segments={parametric}/{polar}/{join}")
                })
                .collect::<Vec<_>>()
                .join(", ")
        }

        let path = std::env::var_os("RIVE_CPP_DIRECT_OVERSTROKE_QUAD_SPANS").expect(
            "RIVE_CPP_DIRECT_OVERSTROKE_QUAD_SPANS is required for the ignored direct-overstroke-quad span test",
        );
        assert!(
            !path.is_empty(),
            "RIVE_CPP_DIRECT_OVERSTROKE_QUAD_SPANS is empty"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_DIRECT_OVERSTROKE_QUAD_SPANS must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ direct-overstroke-quad spans at {}: {error}",
                path.display()
            )
        });
        let cpp_spans = tess_span_oracle::TessSpanArtifact::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ direct-overstroke-quad spans at {}: {error}",
                path.display()
            )
        });
        let rust_spans = fixed_overstroke_quad_spans();
        tess_span_oracle::compare_exact(&cpp_spans, &rust_spans).unwrap_or_else(|error| {
            panic!(
                "C++ direct-overstroke-quad CPU span mismatch at {}: {error}\n  C++ {}\n  Rust {}",
                path.display(),
                summarize(&cpp_spans),
                summarize(&rust_spans),
            )
        });
    }
    #[test]
    #[ignore = "requires RIVE_CPP_DIRECT_DEGENERATE_SPANS_DIR from the C++ WebGPU oracle"]
    fn cpp_direct_degenerate_cubic_cpu_spans_match_rust_record_for_record() {
        fn summarize(artifact: &tess_span_oracle::TessSpanArtifact) -> String {
            artifact
                .records
                .iter()
                .enumerate()
                .map(|(index, record)| {
                    let x0 = record[12] as u16 as i16;
                    let x1 = (record[12] >> 16) as u16 as i16;
                    let parametric = record[14] & 0x3ff;
                    let polar = record[14] >> 10 & 0x3ff;
                    let join = record[14] >> 20 & 0x3ff;
                    format!("{index}:x={x0}..{x1},segments={parametric}/{polar}/{join}")
                })
                .collect::<Vec<_>>()
                .join(", ")
        }

        fn differing_words(
            cpp: &tess_span_oracle::TessSpanArtifact,
            rust: &tess_span_oracle::TessSpanArtifact,
        ) -> String {
            cpp.records
                .iter()
                .zip(&rust.records)
                .enumerate()
                .flat_map(|(record_index, (cpp_record, rust_record))| {
                    cpp_record
                        .iter()
                        .zip(rust_record)
                        .enumerate()
                        .filter(|(_, (cpp_word, rust_word))| cpp_word != rust_word)
                        .map(move |(word_index, (&cpp_word, &rust_word))| {
                            format!(
                                "{record_index}.{word_index}: {cpp_word:#010x}/{} vs {rust_word:#010x}/{}",
                                f32::from_bits(cpp_word),
                                f32::from_bits(rust_word)
                            )
                        })
                })
                .take(24)
                .collect::<Vec<_>>()
                .join(", ")
        }

        let directory = std::env::var_os("RIVE_CPP_DIRECT_DEGENERATE_SPANS_DIR").expect(
            "RIVE_CPP_DIRECT_DEGENERATE_SPANS_DIR is required for the ignored degenerate-cubic span test",
        );
        assert!(
            !directory.is_empty(),
            "RIVE_CPP_DIRECT_DEGENERATE_SPANS_DIR is empty"
        );
        let directory = PathBuf::from(directory);
        assert!(
            directory.is_absolute(),
            "RIVE_CPP_DIRECT_DEGENERATE_SPANS_DIR must be absolute"
        );
        let mut mismatches = Vec::new();
        for selector in [
            "tricky-path20",
            "wide-row0",
            "wide-row1",
            "wide-row2",
            "wide-row3",
        ] {
            let path = directory.join(format!("direct-degenerate-{selector}-spans.bin"));
            let bytes = fs::read(&path).unwrap_or_else(|error| {
                panic!(
                    "failed to read C++ {selector} spans at {}: {error}",
                    path.display()
                )
            });
            let cpp_spans =
                tess_span_oracle::TessSpanArtifact::parse(&bytes).unwrap_or_else(|error| {
                    panic!(
                        "malformed C++ {selector} spans at {}: {error}",
                        path.display()
                    )
                });
            let rust_spans = fixed_degenerate_cubic_spans(selector);
            if let Err(error) = tess_span_oracle::compare_exact(&cpp_spans, &rust_spans) {
                mismatches.push(format!(
                    "{selector}: {error}\n  C++ {}\n  Rust {}\n  Words {}",
                    summarize(&cpp_spans),
                    summarize(&rust_spans),
                    differing_words(&cpp_spans, &rust_spans)
                ));
            }
        }
        assert!(
            mismatches.is_empty(),
            "C++ degenerate-cubic CPU span mismatches:\n{}",
            mismatches.join("\n")
        );
    }

    #[test]
    #[ignore = "requires RIVE_CPP_DIRECT_DEGENERATE_INPUTS_DIR from the C++ WebGPU oracle"]
    fn cpp_direct_degenerate_cubic_tessellation_texture_matches_rust() {
        let directory = std::env::var_os("RIVE_CPP_DIRECT_DEGENERATE_INPUTS_DIR").expect(
            "RIVE_CPP_DIRECT_DEGENERATE_INPUTS_DIR is required for the ignored degenerate-cubic tessellation test",
        );
        assert!(
            !directory.is_empty(),
            "RIVE_CPP_DIRECT_DEGENERATE_INPUTS_DIR is empty"
        );
        let directory = PathBuf::from(directory);
        assert!(
            directory.is_absolute(),
            "RIVE_CPP_DIRECT_DEGENERATE_INPUTS_DIR must be absolute"
        );
        let mut mismatches = Vec::new();
        for selector in [
            "tricky-path20",
            "wide-row0",
            "wide-row1",
            "wide-row2",
            "wide-row3",
        ] {
            let path = directory.join(format!("direct-degenerate-{selector}-inputs.bin"));
            let bytes = fs::read(&path).unwrap_or_else(|error| {
                panic!(
                    "failed to read C++ {selector} tessellation inputs at {}: {error}",
                    path.display()
                )
            });
            let mut cpp_inputs =
                atlas_input_oracle::AtlasInputs::parse(&bytes).unwrap_or_else(|error| {
                    panic!(
                        "malformed C++ {selector} tessellation inputs at {}: {error}",
                        path.display()
                    )
                });
            let rust_inputs = fixed_degenerate_cubic_direct_inputs(selector);
            // The direct MSAA oracle does not expose an atlas-patch batch; the
            // artifact is used here to compare its contour and tessellation
            // texture payloads.
            cpp_inputs.base_patch = rust_inputs.base_patch;
            cpp_inputs.patch_count = rust_inputs.patch_count;
            if let Err(error) = atlas_input_oracle::compare_cpp_to_rust(&cpp_inputs, &rust_inputs) {
                mismatches.push(format!("{selector}: {error}"));
            }
        }
        assert!(
            mismatches.is_empty(),
            "C++ degenerate-cubic tessellation mismatches:\n{}",
            mismatches.join("\n")
        );
    }

    #[test]
    #[ignore = "requires RIVE_CPP_DIRECT_DEGENERATE_BLITS_DIR from the C++ WebGPU oracle"]
    fn cpp_direct_degenerate_cubic_msaa_blits_match_rust() {
        fn difference_summary(
            cpp: &atlas_blit_oracle::AtlasBlit,
            rust: &atlas_blit_oracle::AtlasBlit,
        ) -> String {
            let mut bounds = [usize::MAX, usize::MAX, 0, 0];
            let mut cpp_values = std::collections::BTreeMap::<[u8; 4], usize>::new();
            let mut rust_values = std::collections::BTreeMap::<[u8; 4], usize>::new();
            for (index, (cpp, rust)) in cpp
                .pixels()
                .chunks_exact(4)
                .zip(rust.pixels().chunks_exact(4))
                .enumerate()
            {
                if cpp == rust {
                    continue;
                }
                let x = index % ATLAS_ORACLE_FRAME_SIZE as usize;
                let y = index / ATLAS_ORACLE_FRAME_SIZE as usize;
                bounds[0] = bounds[0].min(x);
                bounds[1] = bounds[1].min(y);
                bounds[2] = bounds[2].max(x);
                bounds[3] = bounds[3].max(y);
                *cpp_values.entry(cpp.try_into().unwrap()).or_default() += 1;
                *rust_values.entry(rust.try_into().unwrap()).or_default() += 1;
            }
            format!("bounds={bounds:?} C++={cpp_values:?} Rust={rust_values:?}")
        }

        let directory = std::env::var_os("RIVE_CPP_DIRECT_DEGENERATE_BLITS_DIR").expect(
            "RIVE_CPP_DIRECT_DEGENERATE_BLITS_DIR is required for the ignored degenerate-cubic MSAA blit test",
        );
        assert!(
            !directory.is_empty(),
            "RIVE_CPP_DIRECT_DEGENERATE_BLITS_DIR is empty"
        );
        let directory = PathBuf::from(directory);
        assert!(
            directory.is_absolute(),
            "RIVE_CPP_DIRECT_DEGENERATE_BLITS_DIR must be absolute"
        );
        let mut mismatches = Vec::new();
        for selector in [
            "tricky-path20",
            "wide-row0",
            "wide-row1",
            "wide-row2",
            "wide-row3",
        ] {
            let path = directory.join(format!("direct-degenerate-{selector}-blit.rgba"));
            let bytes = fs::read(&path).unwrap_or_else(|error| {
                panic!(
                    "failed to read C++ {selector} MSAA blit at {}: {error}",
                    path.display()
                )
            });
            let cpp_blit = atlas_blit_oracle::AtlasBlit::parse(&bytes).unwrap_or_else(|error| {
                panic!(
                    "malformed C++ {selector} MSAA blit at {}: {error}",
                    path.display()
                )
            });
            let rust_blit = fixed_degenerate_cubic_blit(selector);
            if let Err(error) = atlas_blit_oracle::compare_cpp_to_rust(&cpp_blit, &rust_blit) {
                mismatches.push(format!(
                    "{selector}: {error}; {}",
                    difference_summary(&cpp_blit, &rust_blit)
                ));
            }
        }
        assert!(
            mismatches.is_empty(),
            "C++ degenerate-cubic MSAA blit mismatches:\n{}",
            mismatches.join("\n")
        );
    }

    #[test]
    #[ignore = "requires RIVE_CPP_DIRECT_RAWTEXT_INPUTS from the C++ WebGPU oracle"]
    fn cpp_webgpu_direct_rawtext_tessellation_matches_rust() {
        let path = std::env::var_os("RIVE_CPP_DIRECT_RAWTEXT_INPUTS").expect(
            "RIVE_CPP_DIRECT_RAWTEXT_INPUTS is required for the ignored direct-rawtext input test",
        );
        assert!(!path.is_empty(), "RIVE_CPP_DIRECT_RAWTEXT_INPUTS is empty");
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_DIRECT_RAWTEXT_INPUTS must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ direct-rawtext inputs at {}: {error}",
                path.display()
            )
        });
        let cpp_inputs = atlas_input_oracle::AtlasInputs::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ direct-rawtext inputs at {}: {error}",
                path.display()
            )
        });
        let rust_inputs = fixed_rawtext_direct_inputs();
        atlas_input_oracle::compare_cpp_to_rust(&cpp_inputs, &rust_inputs).unwrap_or_else(
            |error| {
                panic!(
                    "C++ direct-rawtext input mismatch at {}: {error}",
                    path.display()
                )
            },
        );
    }

    #[test]
    #[ignore = "diagnostic generic-atomic coverage readback"]
    fn direct_cusp_atomic_coverage_capture_has_one_full_frame() {
        let (_, coverage) = fixed_feather_direct_cusp_frame()
            .finish_with_atomic_coverage()
            .unwrap();
        assert_eq!(coverage.len(), 1);
        assert_eq!(coverage[0].len(), 64 * 64);
    }

    #[test]
    #[ignore = "requires RIVE_CPP_DIRECT_CUSP_COVERAGE and RIVE_CPP_DIRECT_CUSP_BLIT"]
    fn cpp_webgpu_direct_cusp_atomic_coverage_matches_rust_when_configured() {
        let path = std::env::var_os("RIVE_CPP_DIRECT_CUSP_COVERAGE").expect(
            "RIVE_CPP_DIRECT_CUSP_COVERAGE is required for the ignored direct-cusp coverage test",
        );
        assert!(!path.is_empty(), "RIVE_CPP_DIRECT_CUSP_COVERAGE is empty");
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_DIRECT_CUSP_COVERAGE must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ direct-cusp coverage at {}: {error}",
                path.display()
            )
        });
        assert!(
            bytes.len() >= 24,
            "direct-cusp coverage header is truncated"
        );
        assert_eq!(&bytes[..8], b"RIVEAPC\0");
        let read_u32 = |offset| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        assert_eq!(read_u32(8), 1, "unsupported direct-cusp coverage version");
        let width = read_u32(12);
        let height = read_u32(16);
        let word_count = read_u32(20) as usize;
        assert_eq!((width, height, word_count), (64, 64, 64 * 64));
        assert_eq!(bytes.len(), 24 + word_count * 4);
        let blit_path = PathBuf::from(
            std::env::var_os("RIVE_CPP_DIRECT_CUSP_BLIT")
                .expect("RIVE_CPP_DIRECT_CUSP_BLIT is required for coverage normalization"),
        );
        let cpp_blit =
            atlas_blit_oracle::AtlasBlit::parse(&fs::read(&blit_path).unwrap_or_else(|error| {
                panic!(
                    "failed to read C++ direct-cusp blit at {}: {error}",
                    blit_path.display()
                )
            }))
            .unwrap_or_else(|error| {
                panic!(
                    "malformed C++ direct-cusp blit at {}: {error}",
                    blit_path.display()
                )
            });
        assert_eq!(cpp_blit.pixels().len(), word_count * 4);
        let mut cpp_coverage = bytes[24..]
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
            .collect::<Vec<_>>();
        let range = gpu::CoverageBufferRange {
            offset: 0,
            pitch: width,
            offset_x: 0.0,
            offset_y: 0.0,
        };
        for y in 0..height {
            for x in 0..width {
                let pixel_index = (y * width + x) as usize;
                let word_index = coverage_word_index(range, x, y);
                if cpp_coverage[word_index] == 1 << 16
                    && cpp_blit.pixels()[pixel_index * 4..][..4] == [0, 0, 0, 0]
                {
                    cpp_coverage[word_index] = 0;
                }
            }
        }
        let (_, rust_captures) = fixed_feather_direct_cusp_frame()
            .finish_with_atomic_coverage()
            .unwrap();
        assert_eq!(rust_captures.len(), 1);
        let mismatch = cpp_coverage
            .iter()
            .zip(&rust_captures[0])
            .enumerate()
            .find(|(_, (cpp, rust))| cpp != rust);
        assert!(mismatch.is_none(), "coverage mismatch: {mismatch:?}");
    }

    #[test]
    #[ignore = "requires RIVE_CPP_DIRECT_CUSP_BLIT from the C++ WebGPU oracle"]
    fn cpp_webgpu_direct_cusp_blit_matches_rust_when_configured() {
        let path = std::env::var_os("RIVE_CPP_DIRECT_CUSP_BLIT")
            .expect("RIVE_CPP_DIRECT_CUSP_BLIT is required for the ignored direct-cusp blit test");
        assert!(!path.is_empty(), "RIVE_CPP_DIRECT_CUSP_BLIT is empty");
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_DIRECT_CUSP_BLIT must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ direct-cusp blit at {}: {error}",
                path.display()
            )
        });
        let cpp_blit = atlas_blit_oracle::AtlasBlit::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ direct-cusp blit at {}: {error}",
                path.display()
            )
        });
        let rust_blit = fixed_feather_direct_cusp_blit();
        atlas_blit_oracle::compare_cpp_to_rust_with_tolerance(&cpp_blit, &rust_blit, 2, 32)
            .unwrap_or_else(|error| {
                panic!(
                    "C++ direct-cusp blit mismatch at {}: {error}",
                    path.display()
                )
            });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_DIRECT_RAWTEXT_SPANS from the C++ WebGPU oracle"]
    fn cpp_direct_rawtext_cpu_spans_match_rust_record_for_record() {
        let path = std::env::var_os("RIVE_CPP_DIRECT_RAWTEXT_SPANS").expect(
            "RIVE_CPP_DIRECT_RAWTEXT_SPANS is required for the ignored direct-rawtext span test",
        );
        assert!(!path.is_empty(), "RIVE_CPP_DIRECT_RAWTEXT_SPANS is empty");
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_DIRECT_RAWTEXT_SPANS must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ direct-rawtext spans at {}: {error}",
                path.display()
            )
        });
        let cpp_spans = tess_span_oracle::TessSpanArtifact::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ direct-rawtext spans at {}: {error}",
                path.display()
            )
        });
        let rust_spans = fixed_rawtext_spans();
        tess_span_oracle::compare_exact(&cpp_spans, &rust_spans).unwrap_or_else(|error| {
            panic!(
                "C++ direct-rawtext CPU span mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_BLIT from the C++ WebGPU MSAA oracle"]
    fn cpp_webgpu_msaa_atlas_blit_oracle_matches_fixed_rust_output_when_configured() {
        let path = std::env::var_os("RIVE_CPP_ATLAS_BLIT")
            .expect("RIVE_CPP_ATLAS_BLIT is required for the ignored C++ atlas-blit oracle test");
        assert!(
            !path.is_empty(),
            "RIVE_CPP_ATLAS_BLIT is set but empty; set it to a C++ atlas-blit oracle file"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_BLIT must be absolute because Cargo runs unit tests from the package directory"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let cpp_blit = atlas_blit_oracle::AtlasBlit::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let rust_blit = fixed_feather_atlas_blit();
        atlas_blit_oracle::compare_cpp_to_rust(&cpp_blit, &rust_blit).unwrap_or_else(|error| {
            panic!(
                "C++ atlas-blit oracle mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_EMPTY_STROKE_OVERLAP_BLIT from the C++ WebGPU MSAA oracle"]
    fn cpp_webgpu_msaa_atlas_empty_stroke_blit_matches_rust_when_configured() {
        let path = std::env::var_os("RIVE_CPP_ATLAS_EMPTY_STROKE_OVERLAP_BLIT").expect(
            "RIVE_CPP_ATLAS_EMPTY_STROKE_OVERLAP_BLIT is required for the ignored empty-stroke blit test",
        );
        assert!(
            !path.is_empty(),
            "RIVE_CPP_ATLAS_EMPTY_STROKE_OVERLAP_BLIT is empty"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_EMPTY_STROKE_OVERLAP_BLIT must be absolute"
        );
        let cpp_blit =
            atlas_blit_oracle::AtlasBlit::parse(&fs::read(&path).unwrap_or_else(|error| {
                panic!(
                    "failed to read C++ empty-stroke blit at {}: {error}",
                    path.display()
                )
            }))
            .unwrap_or_else(|error| {
                panic!(
                    "malformed C++ empty-stroke blit at {}: {error}",
                    path.display()
                )
            });
        let rust_blit = fixed_feather_atlas_empty_stroke_blit();
        let center = (ATLAS_ORACLE_FRAME_SIZE / 2) as usize;
        let center_offset = (center * ATLAS_ORACLE_FRAME_SIZE as usize + center) * 4;
        assert_eq!(
            &rust_blit.pixels()[center_offset..center_offset + 4],
            &cpp_blit.pixels()[center_offset..center_offset + 4],
            "empty feather-stroke cap center must pass the scheduled MSAA depth test"
        );
        atlas_blit_oracle::compare_cpp_to_rust(&cpp_blit, &rust_blit).unwrap_or_else(|error| {
            panic!(
                "C++ empty-stroke blit mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_CLIPPED_BLIT from the C++ WebGPU MSAA oracle"]
    fn cpp_webgpu_msaa_atlas_clipped_blit_matches_fixed_rust_output_when_configured() {
        let path = std::env::var_os("RIVE_CPP_ATLAS_CLIPPED_BLIT").expect(
            "RIVE_CPP_ATLAS_CLIPPED_BLIT is required for the ignored C++ clipped atlas-blit test",
        );
        assert!(!path.is_empty(), "RIVE_CPP_ATLAS_CLIPPED_BLIT is empty");
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_CLIPPED_BLIT must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ clipped atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let cpp_blit = atlas_blit_oracle::AtlasBlit::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ clipped atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let rust_blit = fixed_feather_atlas_clipped_blit().unwrap();
        atlas_blit_oracle::compare_cpp_to_rust(&cpp_blit, &rust_blit).unwrap_or_else(|error| {
            panic!(
                "C++ clipped atlas-blit oracle mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_PATH_CLIPPED_BLIT from the C++ WebGPU MSAA oracle"]
    fn cpp_webgpu_msaa_atlas_path_clipped_blit_matches_fixed_rust_output_when_configured() {
        let path = std::env::var_os("RIVE_CPP_ATLAS_PATH_CLIPPED_BLIT").expect(
            "RIVE_CPP_ATLAS_PATH_CLIPPED_BLIT is required for the ignored C++ path-clipped atlas-blit test",
        );
        assert!(
            !path.is_empty(),
            "RIVE_CPP_ATLAS_PATH_CLIPPED_BLIT is empty"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_PATH_CLIPPED_BLIT must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ path-clipped atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let cpp_blit = atlas_blit_oracle::AtlasBlit::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ path-clipped atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let rust_blit = fixed_feather_atlas_path_clipped_blit();
        atlas_blit_oracle::compare_cpp_to_rust(&cpp_blit, &rust_blit).unwrap_or_else(|error| {
            panic!(
                "C++ path-clipped atlas-blit oracle mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_CHANGING_PATH_CLIPPED_BLIT from the C++ WebGPU MSAA oracle"]
    fn cpp_webgpu_msaa_atlas_changing_path_clipped_blit_matches_fixed_rust_output_when_configured()
    {
        let path = std::env::var_os("RIVE_CPP_ATLAS_CHANGING_PATH_CLIPPED_BLIT").expect(
            "RIVE_CPP_ATLAS_CHANGING_PATH_CLIPPED_BLIT is required for the ignored C++ changing-path-clipped atlas-blit test",
        );
        assert!(
            !path.is_empty(),
            "RIVE_CPP_ATLAS_CHANGING_PATH_CLIPPED_BLIT is empty"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_CHANGING_PATH_CLIPPED_BLIT must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ changing-path-clipped atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let cpp_blit = atlas_blit_oracle::AtlasBlit::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ changing-path-clipped atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let rust_blit = fixed_feather_atlas_changing_path_clipped_blit().unwrap();
        atlas_blit_oracle::compare_cpp_to_rust(&cpp_blit, &rust_blit).unwrap_or_else(|error| {
            panic!(
                "C++ changing-path-clipped atlas-blit oracle mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_NESTED_PATH_CLIPPED_BLIT from the C++ WebGPU MSAA oracle"]
    fn cpp_webgpu_msaa_atlas_nested_path_clipped_blit_matches_fixed_rust_output_when_configured() {
        let path = std::env::var_os("RIVE_CPP_ATLAS_NESTED_PATH_CLIPPED_BLIT").expect(
            "RIVE_CPP_ATLAS_NESTED_PATH_CLIPPED_BLIT is required for the ignored C++ nested-path-clipped atlas-blit test",
        );
        assert!(
            !path.is_empty(),
            "RIVE_CPP_ATLAS_NESTED_PATH_CLIPPED_BLIT is empty"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_NESTED_PATH_CLIPPED_BLIT must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ nested-path-clipped atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let cpp_blit = atlas_blit_oracle::AtlasBlit::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ nested-path-clipped atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let rust_blit = fixed_feather_atlas_nested_path_clipped_blit().unwrap();
        atlas_blit_oracle::compare_cpp_to_rust(&cpp_blit, &rust_blit).unwrap_or_else(|error| {
            panic!(
                "C++ nested-path-clipped atlas-blit oracle mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_NESTED_EVENODD_PATH_CLIPPED_BLIT from the C++ WebGPU MSAA oracle"]
    fn cpp_webgpu_msaa_atlas_nested_even_odd_path_clipped_blit_matches_fixed_rust_output_when_configured(
    ) {
        let path = std::env::var_os("RIVE_CPP_ATLAS_NESTED_EVENODD_PATH_CLIPPED_BLIT").expect(
            "RIVE_CPP_ATLAS_NESTED_EVENODD_PATH_CLIPPED_BLIT is required for the ignored C++ nested even-odd path-clipped atlas-blit test",
        );
        assert!(
            !path.is_empty(),
            "RIVE_CPP_ATLAS_NESTED_EVENODD_PATH_CLIPPED_BLIT is empty"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_NESTED_EVENODD_PATH_CLIPPED_BLIT must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ nested even-odd path-clipped atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let cpp_blit = atlas_blit_oracle::AtlasBlit::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ nested even-odd path-clipped atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let rust_blit = fixed_feather_atlas_nested_even_odd_path_clipped_blit().unwrap();
        atlas_blit_oracle::compare_cpp_to_rust(&cpp_blit, &rust_blit).unwrap_or_else(|error| {
            panic!(
                "C++ nested even-odd path-clipped atlas-blit oracle mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_NESTED_CLOCKWISE_PATH_CLIPPED_BLIT from the C++ WebGPU MSAA oracle"]
    fn cpp_webgpu_msaa_atlas_nested_clockwise_path_clipped_blit_matches_fixed_rust_output_when_configured(
    ) {
        let path = std::env::var_os("RIVE_CPP_ATLAS_NESTED_CLOCKWISE_PATH_CLIPPED_BLIT").expect(
            "RIVE_CPP_ATLAS_NESTED_CLOCKWISE_PATH_CLIPPED_BLIT is required for the ignored C++ nested clockwise path-clipped atlas-blit test",
        );
        assert!(
            !path.is_empty(),
            "RIVE_CPP_ATLAS_NESTED_CLOCKWISE_PATH_CLIPPED_BLIT is empty"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_NESTED_CLOCKWISE_PATH_CLIPPED_BLIT must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ nested clockwise path-clipped atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let cpp_blit = atlas_blit_oracle::AtlasBlit::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ nested clockwise path-clipped atlas-blit oracle at {}: {error}",
                path.display()
            )
        });
        let rust_blit = fixed_feather_atlas_nested_clockwise_path_clipped_blit().unwrap();
        atlas_blit_oracle::compare_cpp_to_rust(&cpp_blit, &rust_blit).unwrap_or_else(|error| {
            panic!(
                "C++ nested clockwise path-clipped atlas-blit oracle mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATLAS_ADVANCED_BLEND_BLIT from the C++ WebGPU MSAA oracle"]
    fn cpp_webgpu_msaa_atlas_advanced_blend_matches_rust_output_when_configured() {
        let path = std::env::var_os("RIVE_CPP_ATLAS_ADVANCED_BLEND_BLIT").expect(
            "RIVE_CPP_ATLAS_ADVANCED_BLEND_BLIT is required for the ignored C++ advanced-blend atlas test",
        );
        assert!(
            !path.is_empty(),
            "RIVE_CPP_ATLAS_ADVANCED_BLEND_BLIT is empty"
        );
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATLAS_ADVANCED_BLEND_BLIT must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ advanced-blend atlas oracle at {}: {error}",
                path.display()
            )
        });
        let cpp_blit = atlas_blit_oracle::AtlasBlit::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ advanced-blend atlas oracle at {}: {error}",
                path.display()
            )
        });
        let rust_blit = advanced_feather_atlas_blit().unwrap();
        atlas_blit_oracle::compare_cpp_to_rust(&cpp_blit, &rust_blit).unwrap_or_else(|error| {
            panic!(
                "C++ advanced-blend atlas oracle mismatch at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATOMIC_ADVANCED_BLEND from the C++ WebGPU atomic oracle"]
    fn cpp_webgpu_atomic_advanced_blend_matches_within_backend_quantization_when_configured() {
        let path = std::env::var_os("RIVE_CPP_ATOMIC_ADVANCED_BLEND").expect(
            "RIVE_CPP_ATOMIC_ADVANCED_BLEND is required for the ignored C++ atomic advanced-blend test",
        );
        assert!(!path.is_empty(), "RIVE_CPP_ATOMIC_ADVANCED_BLEND is empty");
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_ATOMIC_ADVANCED_BLEND must be absolute"
        );
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ atomic advanced-blend oracle at {}: {error}",
                path.display()
            )
        });
        let cpp_blit = atlas_blit_oracle::AtlasBlit::parse(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ atomic advanced-blend oracle at {}: {error}",
                path.display()
            )
        });
        let rust_blit = advanced_feather_atomic_output().unwrap();
        atlas_blit_oracle::compare_cpp_to_rust_with_tolerance(&cpp_blit, &rust_blit, 1, 8)
            .unwrap_or_else(|error| {
                panic!(
                    "C++ atomic advanced-blend oracle exceeded the 8-pixel/delta-1 backend quantization budget at {}: {error}",
                    path.display()
                )
            });
    }

    #[test]
    #[ignore = "requires the C++ WebGPU atomic colorburn-pair color, coverage, and blit artifacts"]
    fn cpp_webgpu_atomic_colorburn_pair_has_only_coupled_quantization_when_configured() {
        let read_plane = |variable: &str, magic: &[u8; 8]| {
            let path = std::env::var_os(variable)
                .unwrap_or_else(|| panic!("{variable} is required for the ignored pair test"));
            assert!(!path.is_empty(), "{variable} is empty");
            let path = PathBuf::from(path);
            assert!(path.is_absolute(), "{variable} must be absolute");
            let bytes = fs::read(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            assert!(
                bytes.len() >= 24,
                "{} has a truncated header",
                path.display()
            );
            assert_eq!(&bytes[..8], magic);
            let header_word =
                |offset| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            assert_eq!(header_word(8), 1, "unsupported plane version");
            assert_eq!(header_word(12), ATOMIC_COLORBURN_PAIR_FRAME_SIZE);
            assert_eq!(header_word(16), ATOMIC_COLORBURN_PAIR_FRAME_SIZE);
            let word_count = header_word(20) as usize;
            assert_eq!(
                word_count,
                ATOMIC_COLORBURN_PAIR_FRAME_SIZE as usize
                    * ATOMIC_COLORBURN_PAIR_FRAME_SIZE as usize
            );
            assert_eq!(bytes.len(), 24 + word_count * 4);
            bytes[24..]
                .chunks_exact(4)
                .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
                .collect::<Vec<_>>()
        };
        let cpp_colors = read_plane("RIVE_CPP_ATOMIC_COLORBURN_PAIR_COLOR", b"RIVEACO\0");
        let mut cpp_coverage = read_plane("RIVE_CPP_ATOMIC_COLORBURN_PAIR_COVERAGE", b"RIVEAPC\0");
        let blit_path = PathBuf::from(
            std::env::var_os("RIVE_CPP_ATOMIC_COLORBURN_PAIR_BLIT")
                .expect("RIVE_CPP_ATOMIC_COLORBURN_PAIR_BLIT is required"),
        );
        assert!(blit_path.is_absolute());
        let cpp_blit = atlas_blit_oracle::AtlasBlit::parse(&fs::read(&blit_path).unwrap()).unwrap();

        let size = ATOMIC_COLORBURN_PAIR_FRAME_SIZE;
        let word_index = |x: u32, y: u32| {
            ((y >> 5) * (size << 5)
                + (x >> 5) * 1024
                + ((x & 28) << 5)
                + ((y & 28) << 2)
                + ((y & 3) << 2)
                + (x & 3)) as usize
        };
        assert_eq!(cpp_blit.pixels().len(), (size * size * 4) as usize);
        for y in 0..size {
            for x in 0..size {
                let pixel_index = (y * size + x) as usize;
                let coverage_index = word_index(x, y);
                if cpp_coverage[coverage_index] == 1 << 16
                    && cpp_blit.pixels()[pixel_index * 4..][..4] == [0, 0, 0, 0]
                {
                    cpp_coverage[coverage_index] = 0;
                }
            }
        }

        let (rust_pixels, rust_coverage, rust_clips, rust_colors) =
            interleaved_feather_colorburn_pair_frame()
                .finish_with_atomic_planes()
                .unwrap();
        assert_eq!(rust_coverage.len(), 1);
        assert_eq!(rust_clips.len(), 1);
        assert_eq!(rust_colors.len(), 1);
        let mut coverage_mismatch_count = 0usize;
        let mut coverage_max_delta = 0u32;
        let mut first_coverage_mismatches = Vec::new();
        for y in 0..size {
            for x in 0..size {
                let index = word_index(x, y);
                let cpp = cpp_coverage[index];
                let rust = rust_coverage[0][index];
                if cpp == rust {
                    continue;
                }
                coverage_mismatch_count += 1;
                coverage_max_delta = coverage_max_delta.max(cpp.abs_diff(rust));
                if first_coverage_mismatches.len() < 4 {
                    let pixel_index = (y * size + x) as usize * 4;
                    first_coverage_mismatches.push((
                        x,
                        y,
                        cpp,
                        rust,
                        cpp_colors[index],
                        rust_colors[0][index],
                        <[u8; 4]>::try_from(&cpp_blit.pixels()[pixel_index..pixel_index + 4])
                            .unwrap(),
                        <[u8; 4]>::try_from(&rust_pixels[pixel_index..pixel_index + 4]).unwrap(),
                    ));
                }
            }
        }
        assert_eq!(
            coverage_mismatch_count, 0,
            "normalized atomic coverage must be exact; first mismatches={first_coverage_mismatches:?} max-word-delta={coverage_max_delta}"
        );

        let mut color_mismatch_coordinates = Vec::new();
        let mut color_max_delta = 0u8;
        for y in 0..size {
            for x in 0..size {
                let index = word_index(x, y);
                let cpp = cpp_colors[index];
                let rust = rust_colors[0][index];
                if cpp == rust {
                    continue;
                }
                color_mismatch_coordinates.push((x, y));
                for (cpp, rust) in cpp.to_le_bytes().into_iter().zip(rust.to_le_bytes()) {
                    color_max_delta = color_max_delta.max(cpp.abs_diff(rust));
                }
            }
        }
        assert!(
            color_mismatch_coordinates.len() <= 3 && color_max_delta <= 1,
            "atomic color plane exceeded the reviewed quantization bound: coordinates={color_mismatch_coordinates:?} max-channel-delta={color_max_delta}"
        );

        let mut blit_mismatch_coordinates = Vec::new();
        let mut blit_max_delta = 0u8;
        for (index, (cpp, rust)) in cpp_blit
            .pixels()
            .chunks_exact(4)
            .zip(rust_pixels.chunks_exact(4))
            .enumerate()
        {
            if cpp == rust {
                continue;
            }
            blit_mismatch_coordinates.push((index as u32 % size, index as u32 / size));
            for (&cpp, &rust) in cpp.iter().zip(rust) {
                blit_max_delta = blit_max_delta.max(cpp.abs_diff(rust));
            }
        }
        assert_eq!(
            blit_mismatch_coordinates, color_mismatch_coordinates,
            "resolved mismatches must be exactly the deswizzled color-plane mismatches"
        );
        assert!(
            blit_mismatch_coordinates.len() <= 3 && blit_max_delta <= 15,
            "resolved output exceeded the reviewed ColorBurn amplification bound: coordinates={blit_mismatch_coordinates:?} max-channel-delta={blit_max_delta}"
        );
    }

    fn configured_cpp_full_stream_artifact(
        artifact_env: &str,
        provenance_env: &str,
        expected_stream_sha256: &str,
        extra_provenance: &[&str],
    ) -> (PathBuf, atlas_blit_oracle::AtlasBlit) {
        let path = std::env::var_os(artifact_env).unwrap_or_else(|| {
            panic!("{artifact_env} is required for the ignored full-stream test")
        });
        assert!(!path.is_empty(), "{artifact_env} is empty");
        let path = PathBuf::from(path);
        assert!(path.is_absolute(), "{artifact_env} must be absolute");
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        use sha2::Digest as _;
        let artifact_sha256 = format!("{:x}", sha2::Sha256::digest(&bytes));
        let provenance_path =
            PathBuf::from(std::env::var_os(provenance_env).unwrap_or_else(|| {
                panic!("{provenance_env} is required for the ignored full-stream test")
            }));
        assert!(
            provenance_path.is_absolute(),
            "{provenance_env} must be absolute"
        );
        let provenance = fs::read_to_string(&provenance_path).unwrap_or_else(|error| {
            panic!("failed to read {}: {error}", provenance_path.display())
        });
        let stream_provenance = format!("stream_sha256={expected_stream_sha256}");
        for expected in [
            "backend=metal",
            stream_provenance.as_str(),
            "runtime_revision=7c778d13c5d903b3b74eec1dd6bb68a811dea5f2",
            "dawn_revision=211333b2e3e429c3508f25c81c547f602adf448c",
        ] {
            assert!(
                provenance.lines().any(|line| line == expected),
                "{} is missing {expected}",
                provenance_path.display()
            );
        }
        for expected in extra_provenance {
            assert!(
                provenance.lines().any(|line| line == *expected),
                "{} is missing {expected}",
                provenance_path.display()
            );
        }
        let artifact_provenance = format!("artifact_sha256={artifact_sha256}");
        assert!(
            provenance.lines().any(|line| line == artifact_provenance),
            "{} does not match the RGBA artifact SHA-256",
            provenance_path.display()
        );
        assert!(
            provenance
                .lines()
                .any(|line| line.starts_with("adapter_device=") && line.len() > 15),
            "{} does not identify the selected adapter",
            provenance_path.display()
        );
        let blit = atlas_blit_oracle::AtlasBlit::parse(&bytes)
            .unwrap_or_else(|error| panic!("malformed C++ output at {}: {error}", path.display()));
        (path, blit)
    }

    fn largest_full_stream_mismatches(
        cpp: &atlas_blit_oracle::AtlasBlit,
        rust: &atlas_blit_oracle::AtlasBlit,
        width: u32,
    ) -> Vec<(u8, usize, usize, [u8; 4], [u8; 4])> {
        let mut largest = cpp
            .pixels()
            .chunks_exact(4)
            .zip(rust.pixels().chunks_exact(4))
            .enumerate()
            .filter_map(|(index, (cpp, rust))| {
                let max_delta = cpp
                    .iter()
                    .zip(rust)
                    .map(|(&cpp, &rust)| cpp.abs_diff(rust))
                    .max()
                    .unwrap();
                (max_delta != 0).then(|| {
                    (
                        max_delta,
                        index % width as usize,
                        index / width as usize,
                        <[u8; 4]>::try_from(cpp).unwrap(),
                        <[u8; 4]>::try_from(rust).unwrap(),
                    )
                })
            })
            .collect::<Vec<_>>();
        largest.sort_unstable_by(|left, right| right.cmp(left));
        largest
    }

    fn configured_cpp_atomic_plane(
        artifact_env: &str,
        provenance_env: &str,
        expected_magic: &[u8; 8],
        expected_width: u32,
        expected_height: u32,
        provenance_sha_key: &str,
    ) -> Vec<u32> {
        let path = PathBuf::from(std::env::var_os(artifact_env).unwrap_or_else(|| {
            panic!("{artifact_env} is required for the ignored atomic-plane test")
        }));
        assert!(path.is_absolute(), "{artifact_env} must be absolute");
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        assert!(
            bytes.len() >= 24,
            "{} has a truncated header",
            path.display()
        );
        assert_eq!(
            &bytes[..8],
            expected_magic,
            "{} has wrong magic",
            path.display()
        );
        let header_word =
            |offset| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        assert_eq!(
            header_word(8),
            1,
            "{} has unsupported version",
            path.display()
        );
        assert_eq!(header_word(12), expected_width);
        assert_eq!(header_word(16), expected_height);
        let word_count = header_word(20) as usize;
        assert_eq!(
            word_count,
            expected_width as usize * expected_height as usize
        );
        assert_eq!(bytes.len(), 24 + word_count * 4);

        use sha2::Digest as _;
        let sha256 = format!("{:x}", sha2::Sha256::digest(&bytes));
        let provenance_path =
            PathBuf::from(std::env::var_os(provenance_env).unwrap_or_else(|| {
                panic!("{provenance_env} is required for the ignored atomic-plane test")
            }));
        assert!(provenance_path.is_absolute());
        let provenance = fs::read_to_string(&provenance_path).unwrap_or_else(|error| {
            panic!("failed to read {}: {error}", provenance_path.display())
        });
        let expected_sha = format!("{provenance_sha_key}={sha256}");
        assert!(
            provenance.lines().any(|line| line == expected_sha),
            "{} does not match {}",
            provenance_path.display(),
            path.display()
        );

        bytes[24..]
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
            .collect()
    }

    fn raw_rgba_png(path: &std::path::Path) -> atlas_blit_oracle::AtlasBlit {
        let file = std::io::BufReader::new(fs::File::open(path).unwrap());
        let mut decoder = png::Decoder::new(file);
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder.read_info().unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        pixels.truncate(info.buffer_size());
        assert_eq!(info.color_type, png::ColorType::Rgba);
        assert_eq!(info.bit_depth, png::BitDepth::Eight);
        atlas_blit_oracle::AtlasBlit::new(info.width, info.height, pixels).unwrap()
    }

    fn pixels_over_delta(
        left: &atlas_blit_oracle::AtlasBlit,
        right: &atlas_blit_oracle::AtlasBlit,
        threshold: u8,
    ) -> Vec<bool> {
        assert_eq!(left.pixels().len(), right.pixels().len());
        left.pixels()
            .chunks_exact(4)
            .zip(right.pixels().chunks_exact(4))
            .map(|(left, right)| {
                left.iter()
                    .zip(right)
                    .any(|(&left, &right)| left.abs_diff(right) > threshold)
            })
            .collect()
    }

    #[test]
    #[ignore = "requires the pinned C++ Dawn Spotify full-stream, coverage, clip, and provenance artifacts"]
    fn cpp_webgpu_atomic_spotify_kids_app_icon_is_fixed_color_backend_residual() {
        const STREAM_SHA256: &str =
            "1c230de80579ddfc9953541ec3311c981e8f53d94c4d023c5429635186ebbd88";
        const PROVENANCE_ENV: &str = "RIVE_CPP_ATOMIC_SPOTIFY_PROVENANCE";
        let (cpp_path, cpp_blit) = configured_cpp_full_stream_artifact(
            "RIVE_CPP_ATOMIC_SPOTIFY_FULL",
            PROVENANCE_ENV,
            STREAM_SHA256,
            &[
                "frame_width=1024",
                "frame_height=1436",
                "storage_width=1024",
                "storage_height=1440",
                "sample_seconds_bits=00000000",
                "draw_batch_count=24",
                "fixed_function_color_output=true",
                "packed_color_backing=absent",
                "replay_sha256=941d08d82b2059c9094017db4478cd2e3c48684c064b195278c0045d90751e38",
                "draw_schedule_sha256=8adcf15e8277becc884a19c1fbbefa0abddb8b8e95bdc9c1faab41189807de2b",
            ],
        );
        let cpp_coverage = configured_cpp_atomic_plane(
            "RIVE_CPP_ATOMIC_SPOTIFY_COVERAGE",
            PROVENANCE_ENV,
            b"RIVEAPC\0",
            ATOMIC_SPOTIFY_FULL_FRAME_WIDTH,
            ATOMIC_SPOTIFY_FULL_STORAGE_HEIGHT,
            "coverage_sha256",
        );
        let cpp_clips = configured_cpp_atomic_plane(
            "RIVE_CPP_ATOMIC_SPOTIFY_CLIP",
            PROVENANCE_ENV,
            b"RIVEACL\0",
            ATOMIC_SPOTIFY_FULL_FRAME_WIDTH,
            ATOMIC_SPOTIFY_FULL_STORAGE_HEIGHT,
            "clip_sha256",
        );
        let (rust_pixels, rust_coverage, rust_clips, rust_colors) =
            spotify_kids_app_icon_full_frame()
                .finish_with_atomic_planes()
                .unwrap();
        assert_eq!(rust_coverage.len(), 2, "Spotify run partition drifted");
        assert_eq!(rust_clips.len(), 2, "Spotify run partition drifted");
        assert!(
            rust_colors.is_empty(),
            "fixed-function SrcOver must not capture a packed color plane"
        );
        assert_ne!(
            cpp_clips, rust_clips[0],
            "the pre-clip Rust run unexpectedly contains the final clip state"
        );
        assert_eq!(cpp_clips, rust_clips[1], "final atomic clip plane diverged");

        let word_index = |x: u32, y: u32| {
            ((y >> 5) * (ATOMIC_SPOTIFY_FULL_FRAME_WIDTH << 5)
                + (x >> 5) * 1024
                + ((x & 28) << 5)
                + ((y & 28) << 2)
                + ((y & 3) << 2)
                + (x & 3)) as usize
        };
        assert!(cpp_coverage.iter().any(|&word| word != 0));
        assert!(rust_coverage
            .iter()
            .all(|run| run.iter().any(|&word| word != 0)));
        // C++ retains one coverage backing across its 24-batch flush. Rust
        // partitions generic and clockwise draws into two runs with fresh
        // path IDs, so their raw words are not a cross-schedule semantic
        // format. Final alpha below is the coverage comparison; these checks
        // keep the captured backings honest without normalizing path state.
        assert!(rust_coverage.iter().all(|run| run != &cpp_coverage));
        for y in ATOMIC_SPOTIFY_FULL_FRAME_HEIGHT..ATOMIC_SPOTIFY_FULL_STORAGE_HEIGHT {
            for x in 0..ATOMIC_SPOTIFY_FULL_FRAME_WIDTH {
                let index = word_index(x, y);
                assert_eq!(cpp_coverage[index], 0, "C++ padded coverage was touched");
                for (run, coverage) in rust_coverage.iter().enumerate() {
                    assert_eq!(
                        coverage[index], 0,
                        "Rust run {run} padded coverage was touched"
                    );
                }
            }
        }

        let rust_blit = atlas_blit_oracle::AtlasBlit::new(
            ATOMIC_SPOTIFY_FULL_FRAME_WIDTH,
            ATOMIC_SPOTIFY_FULL_FRAME_HEIGHT,
            rust_pixels,
        )
        .unwrap();
        let cpp_rust_byte_pixels = cpp_blit
            .pixels()
            .chunks_exact(4)
            .zip(rust_blit.pixels().chunks_exact(4))
            .filter(|(cpp, rust)| cpp != rust)
            .count();
        let cpp_rust_alpha_pixels = cpp_blit
            .pixels()
            .chunks_exact(4)
            .zip(rust_blit.pixels().chunks_exact(4))
            .filter(|(cpp, rust)| cpp[3] != rust[3])
            .count();
        eprintln!(
            "Spotify C++ Dawn vs Rust: byte-inexact={cpp_rust_byte_pixels} alpha-inexact={cpp_rust_alpha_pixels}"
        );
        atlas_blit_oracle::compare_cpp_to_rust_with_pixel_tolerance(&cpp_blit, &rust_blit, 2, 32)
            .unwrap_or_else(|error| {
                panic!(
                    "C++ Dawn and Rust exceed the unchanged 2/32 contract at {}: {error}",
                    cpp_path.display()
                )
            });

        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let native = raw_rgba_png(&repo_root.join(
            "fixtures/renderer/reference/metal/riv/spotify_kids_app_icon-frame-0-clockwise-atomic.png",
        ));
        let cpp_native = pixels_over_delta(&cpp_blit, &native, 2);
        let rust_native = pixels_over_delta(&rust_blit, &native, 2);
        let cpp_native_count = cpp_native.iter().filter(|&&differs| differs).count();
        let rust_native_count = rust_native.iter().filter(|&&differs| differs).count();
        let max_delta = |left: &atlas_blit_oracle::AtlasBlit| {
            left.pixels()
                .iter()
                .zip(native.pixels())
                .map(|(&left, &right)| left.abs_diff(right))
                .max()
                .unwrap()
        };
        eprintln!(
            "Spotify native Metal residual: C++ Dawn over-delta2={cpp_native_count} max-delta={} Rust over-delta2={rust_native_count} max-delta={}",
            max_delta(&cpp_blit),
            max_delta(&rust_blit)
        );
        assert!(
            cpp_native_count > 32,
            "C++ Dawn unexpectedly satisfies the native-Metal contract"
        );
        assert!(
            rust_native_count > 32,
            "Rust unexpectedly satisfies the native-Metal contract"
        );
        let shared_native_residual = cpp_native
            .iter()
            .zip(&rust_native)
            .filter(|(cpp, rust)| **cpp && **rust)
            .count();
        let native_residual_union = cpp_native
            .iter()
            .zip(&rust_native)
            .filter(|(cpp, rust)| **cpp || **rust)
            .count();
        let native_residual_symmetric_difference = cpp_native
            .iter()
            .zip(&rust_native)
            .filter(|(cpp, rust)| cpp != rust)
            .count();
        assert!(
            native_residual_symmetric_difference <= 64
                && shared_native_residual * 1000 >= native_residual_union * 999,
            "C++ Dawn and Rust no longer isolate the same native-Metal residual: shared={shared_native_residual} union={native_residual_union} symmetric-difference={native_residual_symmetric_difference}"
        );
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATOMIC_INTERLEAVEDFEATHER_FULL and its provenance from the C++ WebGPU-on-Metal oracle"]
    fn cpp_webgpu_atomic_interleavedfeather_full_matches_rust_when_configured() {
        let (path, cpp_blit) = configured_cpp_full_stream_artifact(
            "RIVE_CPP_ATOMIC_INTERLEAVEDFEATHER_FULL",
            "RIVE_CPP_ATOMIC_INTERLEAVEDFEATHER_FULL_PROVENANCE",
            "8868c228229b6708e4e46c947177bfd982c6e7a60ee9b1c3a7da43a7ec0ee17a",
            &[],
        );
        let rust_blit = interleaved_feather_full_output();
        let largest = largest_full_stream_mismatches(
            &cpp_blit,
            &rust_blit,
            ATOMIC_INTERLEAVED_FEATHER_FULL_FRAME_SIZE,
        );
        eprintln!(
            "largest full-stream mismatches: {:?}",
            &largest[..largest.len().min(12)]
        );
        atlas_blit_oracle::compare_cpp_to_rust_with_pixel_tolerance(
            &cpp_blit,
            &rust_blit,
            2,
            32,
        )
        .unwrap_or_else(|error| {
            panic!(
                "C++ WebGPU-on-Metal full stream exceeded the corpus delta-2/32-pixel contract at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATOMIC_DSTREADSHUFFLE_FULL and its provenance from the C++ WebGPU-on-Metal oracle"]
    fn cpp_webgpu_atomic_dstreadshuffle_full_matches_rust_when_configured() {
        let (path, cpp_blit) = configured_cpp_full_stream_artifact(
            "RIVE_CPP_ATOMIC_DSTREADSHUFFLE_FULL",
            "RIVE_CPP_ATOMIC_DSTREADSHUFFLE_FULL_PROVENANCE",
            "0e08ecd19e6a9e1f89f3ae2291181cea3513edf5bbe8cadcd3e1e10a0c33f195",
            &[],
        );
        let rust_blit = dstreadshuffle_full_output();
        let largest = largest_full_stream_mismatches(
            &cpp_blit,
            &rust_blit,
            ATOMIC_DSTREADSHUFFLE_FULL_FRAME_WIDTH,
        );
        eprintln!(
            "dstreadshuffle full-stream byte-inexact={} over-delta2={} max-delta={}; largest: {:?}",
            largest.len(),
            largest.iter().take_while(|mismatch| mismatch.0 > 2).count(),
            largest.first().map_or(0, |mismatch| mismatch.0),
            &largest[..largest.len().min(12)]
        );
        atlas_blit_oracle::compare_cpp_to_rust_with_pixel_tolerance(
            &cpp_blit,
            &rust_blit,
            2,
            32,
        )
        .unwrap_or_else(|error| {
            panic!(
                "C++ WebGPU-on-Metal dstreadshuffle exceeded the corpus delta-2/32-pixel contract at {}: {error}",
                path.display()
            )
        });
    }

    #[test]
    #[ignore = "requires RIVE_CPP_ATOMIC_DSTREADSHUFFLE_SRCOVER and its provenance from the C++ WebGPU-on-Metal oracle"]
    fn cpp_webgpu_atomic_dstreadshuffle_srcover_control_matches_rust_when_configured() {
        let (path, cpp_blit) = configured_cpp_full_stream_artifact(
            "RIVE_CPP_ATOMIC_DSTREADSHUFFLE_SRCOVER",
            "RIVE_CPP_ATOMIC_DSTREADSHUFFLE_SRCOVER_PROVENANCE",
            "0e08ecd19e6a9e1f89f3ae2291181cea3513edf5bbe8cadcd3e1e10a0c33f195",
            &["blend_mode_override=srcOver"],
        );
        for sample in 0..3 {
            let rust_blit = dstreadshuffle_srcover_control_output();
            let largest = largest_full_stream_mismatches(
                &cpp_blit,
                &rust_blit,
                ATOMIC_DSTREADSHUFFLE_FULL_FRAME_WIDTH,
            );
            eprintln!(
                "dstreadshuffle SrcOver-control sample={sample} byte-inexact={} over-delta2={} max-delta={}; largest: {:?}",
                largest.len(),
                largest.iter().take_while(|mismatch| mismatch.0 > 2).count(),
                largest.first().map_or(0, |mismatch| mismatch.0),
                &largest[..largest.len().min(12)]
            );
            atlas_blit_oracle::compare_cpp_to_rust_with_pixel_tolerance(
                &cpp_blit,
                &rust_blit,
                2,
                32,
            )
            .unwrap_or_else(|error| {
                panic!(
                    "C++ WebGPU-on-Metal dstreadshuffle SrcOver control sample {sample} exceeded the corpus delta-2/32-pixel contract at {}: {error}",
                    path.display()
                )
            });
        }
    }

    #[test]
    fn documented_cpp_atlas_mask_path_is_absolute_from_repo_root() {
        let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = package_dir.join("../..").canonicalize().unwrap();
        let cargo_test_cwd = std::env::current_dir().unwrap().canonicalize().unwrap();
        assert_eq!(cargo_test_cwd, package_dir.canonicalize().unwrap());

        let documented_path = repo_root.join("tools/cpp-atlas-mask-oracle/out/atlas-mask.r16f");
        assert!(documented_path.is_absolute());
        assert_eq!(
            documented_path
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .canonicalize()
                .unwrap(),
            repo_root
                .join("tools/cpp-atlas-mask-oracle")
                .canonicalize()
                .unwrap()
        );
    }
}
