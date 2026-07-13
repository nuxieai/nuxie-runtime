//! Pure-Rust wgpu renderer behind the `nuxie-render-api` trait boundary.

#[cfg(test)]
mod atlas_blit_oracle;
#[cfg(test)]
mod atlas_input_oracle;
#[cfg(test)]
mod atlas_mask_oracle;
mod atlas_pipeline;
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
#[allow(dead_code)]
mod intersection_board;
mod mipmap_pipeline;
mod path_pipeline;
mod skyline;
mod tessellator;

use bytemuck::{Pod, Zeroable};
use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageSampler, Mat2D, PathVerb, RawPath, RenderBuffer,
    RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint, RenderPaintStyle, RenderPath,
    RenderShader, Renderer, StrokeCap, StrokeJoin, Vec2D,
};
use std::any::Any;
use std::error::Error;
use std::fmt;
use std::io::Cursor;
use std::sync::{mpsc, Arc};
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub enum RendererError {
    Adapter(String),
    AtlasPacking(&'static str),
    Device(String),
    Map(String),
    Unsupported(&'static str),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adapter(message) => write!(f, "wgpu adapter error: {message}"),
            Self::AtlasPacking(message) => write!(f, "atlas packing error: {message}"),
            Self::Device(message) => write!(f, "wgpu device error: {message}"),
            Self::Map(message) => write!(f, "wgpu readback error: {message}"),
            Self::Unsupported(feature) => write!(f, "unsupported renderer feature: {feature}"),
        }
    }
}

impl Error for RendererError {}

struct Context {
    device: wgpu::Device,
    queue: wgpu::Queue,
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
    feather_lut: feather_lut::FeatherLut,
}

pub struct WgpuFactory {
    context: Arc<Context>,
    width: u32,
    height: u32,
    mode: RenderMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Msaa,
    ClockwiseAtomic,
}

impl WgpuFactory {
    pub fn new(width: u32, height: u32) -> Result<Self, RendererError> {
        Self::new_with_mode(width, height, RenderMode::Msaa)
    }

    pub fn new_with_mode(width: u32, height: u32, mode: RenderMode) -> Result<Self, RendererError> {
        pollster::block_on(Self::new_async(width, height, mode))
    }

    async fn new_async(width: u32, height: u32, mode: RenderMode) -> Result<Self, RendererError> {
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
        let adapter_limits = adapter.limits();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("nuxie-renderer-device"),
                required_features: wgpu::Features::empty(),
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
            format: wgpu::TextureFormat::Stencil8,
            depth_write_enabled: None,
            depth_compare: None,
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
        let patch_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("nuxie-patch-vertices"),
            contents: bytemuck::cast_slice(&patch_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let patch_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
        let feather_lut = feather_lut::FeatherLut::new(&device, &queue);
        Ok(Self {
            context: Arc::new(Context {
                device,
                queue,
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
                feather_lut,
            }),
            width,
            height,
            mode,
        })
    }

    pub fn begin_frame(&self, clear_color: ColorInt) -> WgpuFrame {
        WgpuFrame {
            context: Arc::clone(&self.context),
            width: self.width,
            height: self.height,
            clear_color,
            state: DrawState::default(),
            stack: Vec::new(),
            draws: Vec::new(),
            clips: Vec::new(),
            next_clip_id: 1,
            unsupported: None,
            mode: self.mode,
        }
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

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        Box::new(WgpuPath {
            raw_path,
            fill_rule,
        })
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        Box::new(WgpuPath {
            raw_path: RawPath::new(),
            fill_rule: FillRule::NonZero,
        })
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        Box::new(WgpuPaint::default())
    }

    fn decode_image(&mut self, data: &[u8]) -> Box<dyn RenderImage> {
        let Some((width, height, pixels)) = decode_image_rgba(data) else {
            return Box::new(WgpuImage {
                width: 0,
                height: 0,
                texture: None,
            });
        };
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
        self.context.queue.write_texture(
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
        Box::new(WgpuImage {
            width,
            height,
            texture: Some(Arc::new(WgpuImageTexture { texture, view })),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct WgpuPath {
    raw_path: RawPath,
    fill_rule: FillRule,
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
        self.raw_path.add_path(&wgpu_path(path).raw_path, transform);
    }

    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        self.raw_path
            .add_path_backwards(&wgpu_path(path).raw_path, transform);
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
        self.feather = value;
    }

    fn blend_mode(&mut self, value: BlendMode) {
        self.blend_mode = value;
    }

    fn shader(&mut self, shader: Option<&dyn RenderShader>) {
        self.shader = shader.map(|shader| {
            shader
                .as_any()
                .downcast_ref::<WgpuShader>()
                .expect("nuxie-renderer received a foreign shader")
                .clone()
        });
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
        self.submitted = Some(Arc::new(self.context.device.create_buffer_init(
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
    Content { clip_id: u16 },
    ClipUpdate { replacement_id: u16, parent_id: u16 },
}

pub struct WgpuFrame {
    context: Arc<Context>,
    width: u32,
    height: u32,
    clear_color: ColorInt,
    state: DrawState,
    stack: Vec<DrawState>,
    draws: Vec<SolidDraw>,
    clips: Vec<ClipElement>,
    next_clip_id: u32,
    unsupported: Option<&'static str>,
    mode: RenderMode,
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
        if self.state.clip_is_empty {
            return;
        }
        let path = wgpu_path(path);
        let paint = wgpu_paint(paint);
        if path_draw_is_noop(path, paint, self.state.transform) {
            return;
        }
        let Some((clip_updates, clip_id)) = self.prepare_clip_updates() else {
            return;
        };
        let content = SolidDraw {
            path: path.clone(),
            paint: paint.clone(),
            state: self.state,
            role: DrawRole::Content { clip_id },
            image: None,
        };
        if clip_id != 0 {
            if !atomic_draw_is_eligible(&content) {
                self.unsupported
                    .get_or_insert("non-rectangular clips on fallback draws");
                return;
            }
        }
        self.draws.extend(clip_updates);
        self.draws.push(content);
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        if self.state.clip_is_empty {
            return;
        }
        let path = wgpu_path(path);
        if path.raw_path.verbs().is_empty() {
            self.state.clip_is_empty = true;
            return;
        }
        let Some(rect) = path_aabb(&path.raw_path) else {
            let height = self.state.clip_stack_height;
            if self
                .clips
                .get(height)
                .is_none_or(|clip| clip.matrix != self.state.transform || clip.path != *path)
            {
                self.clips.truncate(height);
                self.clips.push(ClipElement {
                    path: path.clone(),
                    matrix: self.state.transform,
                });
            }
            self.state.clip_stack_height = height + 1;
            return;
        };
        if !apply_clip_rect(&mut self.state, rect) {
            self.unsupported
                .get_or_insert("incompatible clip rectangles");
        }
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        if self.mode != RenderMode::ClockwiseAtomic {
            self.unsupported.get_or_insert("images in msaa mode");
            return;
        }
        if self.state.clip_is_empty {
            return;
        }
        let Some(image) = image.and_then(|image| image.as_any().downcast_ref::<WgpuImage>()) else {
            return;
        };
        let Some(texture) = &image.texture else {
            return;
        };
        let Some((clip_updates, clip_id)) = self.prepare_clip_updates() else {
            return;
        };
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
        self.draws.extend(clip_updates);
        self.draws.push(content);
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
        if self.state.clip_is_empty {
            return;
        }
        let Some(image) = image.and_then(|image| image.as_any().downcast_ref::<WgpuImage>()) else {
            return;
        };
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
        let Some((clip_updates, clip_id)) = self.prepare_clip_updates() else {
            return;
        };
        let content = SolidDraw {
            path: WgpuPath {
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
        self.draws.extend(clip_updates);
        self.draws.push(content);
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        self.state.opacity *= opacity;
    }
}

impl WgpuFrame {
    fn prepare_clip_updates(&mut self) -> Option<(Vec<SolidDraw>, u16)> {
        let height = self.state.clip_stack_height;
        if height == 0 {
            return Some((Vec::new(), 0));
        }
        // C++ RiveRenderer::applyClip generates a new ID whenever a clip is
        // rendered. Reusing stack depth would accept stale coverage left by an
        // unrelated clip at the same depth in the storage-backed clip plane.
        let Ok(update_count) = u32::try_from(height) else {
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
        let mut updates = Vec::with_capacity(height);
        let mut parent_id = 0;
        for (offset, clip) in self.clips[..height].iter().enumerate() {
            let replacement_id = (self.next_clip_id + offset as u32) as u16;
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

    pub fn finish(self) -> Result<Vec<u8>, RendererError> {
        self.finish_internal(false).map(|(pixels, _)| pixels)
    }

    #[cfg(test)]
    fn finish_with_clockwise_atomic_coverage(
        self,
    ) -> Result<(Vec<u8>, Vec<ClockwiseAtomicCoverageSnapshot>), RendererError> {
        self.finish_internal(true)
    }

    fn finish_internal(
        self,
        capture_clockwise_atomic_coverage: bool,
    ) -> Result<(Vec<u8>, Vec<ClockwiseAtomicCoverageSnapshot>), RendererError> {
        if let Some(feature) = self.unsupported {
            return Err(RendererError::Unsupported(feature));
        }
        let texture = self
            .context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("nuxie-offscreen-target"),
                size: wgpu::Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let multisample_texture = self
            .context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("nuxie-multisample-target"),
                size: texture.size(),
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
        let multisample_view =
            multisample_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let stencil_texture = self
            .context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("nuxie-stencil-target"),
                size: texture.size(),
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Stencil8,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
        let stencil_view = stencil_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-frame-encoder"),
                });
        let mut pending_coverage_readbacks = Vec::new();
        let mut encode_atomic_run =
            |draws: &[SolidDraw],
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
                    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                        let tessellation = if requires_atlas {
                            draw::build_feather_atlas_tessellation(
                                raw_path,
                                draw.state.transform,
                                draw.paint.feather,
                                stroke,
                            )
                        } else {
                            draw::build_feather_tessellation(
                                raw_path,
                                draw.state.transform,
                                draw.paint.feather,
                                stroke,
                            )
                        }
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
                        (draw::should_use_interior_tessellation(raw_path, draw.state.transform)
                            && (!use_clockwise_atomic_batch || contour_count(draw) > 1))
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
                        let (range, word_count) = draw::clockwise_atomic_coverage_range(
                            raw_path,
                            draw.state.transform,
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
                                        fill_rule,
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
                                    fill_rule,
                                    draw.paint.blend_mode,
                                )
                            };
                            paint.with_clip_id(clip_id)
                        }
                    };
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
                let share_midpoint_tessellation = !force_clockwise_atomic_batch
                    && draws
                        .iter()
                        .all(|draw| !matches!(draw.role, DrawRole::ClipUpdate { .. }))
                    && prepared.iter().all(|draw| {
                        draw.image.is_none()
                            && draw.triangles.is_empty()
                            && draw.atlas.is_none()
                            && !draw.is_stroke
                            && !draw.is_feather
                            && draw.patch_index_range.end
                                <= (gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT
                                    + gpu::MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT)
                                    as u32
                    });
                let mut tessellation_span_batches = Vec::new();
                let mut tessellation_heights = Vec::new();
                let mut needs_dummy_tessellation = false;
                if share_midpoint_tessellation {
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
                let mut tessellation_textures = Vec::with_capacity(tessellation_span_batches.len());
                for (spans, height) in tessellation_span_batches
                    .iter()
                    .zip(tessellation_heights.iter().copied())
                {
                    let tessellation_texture = self.context.tessellator.encode(
                        &self.context.device,
                        encoder,
                        &self.context.feather_lut.view,
                        spans,
                        &uniforms,
                        &paths,
                        &contours,
                        height,
                    );
                    tessellation_textures.push(tessellation_texture);
                }
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
                    uniforms.coverage_buffer_prefix = 1 << 20;
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
                        .map(|draw| {
                            draw.triangles
                                .iter()
                                .copied()
                                .filter(|vertex| vertex.weight_path_id >> 16 > 0)
                                .collect::<Vec<_>>()
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
                                    main_triangles,
                                    kind,
                                }
                            },
                        )
                        .collect::<Vec<_>>();
                    let coverage_readback = self.context.clockwise_atomic_pipeline.encode_fills(
                        &self.context.device,
                        encoder,
                        &view,
                        &self.context.feather_lut.view,
                        gradient_texture.as_ref().map(|texture| &texture.view),
                        &self.context.patch_vertex_buffer,
                        &self.context.patch_index_buffer,
                        &clockwise_atomic_draws,
                        &uniforms,
                        &paths,
                        &paints,
                        &paint_aux,
                        &contours,
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
                } else {
                    self.context.atomic_pipeline.encode_batch(
                        &self.context.device,
                        encoder,
                        &view,
                        load_color,
                        &self.context.feather_lut.view,
                        gradient_texture.as_ref().map(|texture| &texture.view),
                        &self.context.patch_vertex_buffer,
                        &self.context.patch_index_buffer,
                        &atomic_draws,
                        &uniforms,
                        &paths,
                        &paints,
                        &paint_aux,
                        &contours,
                        padded_width as usize * padded_height as usize,
                    );
                }
                Ok::<(), RendererError>(())
            };
        let encode_fallback_run =
            |draws: &[SolidDraw], clear_target: bool, encoder: &mut wgpu::CommandEncoder| {
                enum PreparedDraw {
                    Analytic(path_pipeline::PreparedPathDraw),
                    Bootstrap(wgpu::Buffer, wgpu::Buffer, FillRule),
                }
                let mut prepared_draws = Vec::with_capacity(draws.len());
                for draw in draws {
                    if draw.paint.shader.is_none() && draw.paint.feather == 0.0 {
                        let tessellation = match draw.paint.style {
                            RenderPaintStyle::Fill => draw::build_fill_tessellation(
                                &draw.path.raw_path,
                                draw.state.transform,
                            ),
                            RenderPaintStyle::Stroke => draw::build_stroke_tessellation(
                                &draw.path.raw_path,
                                draw.state.transform,
                                draw.paint.thickness,
                                draw.paint.join,
                                draw.paint.cap,
                            ),
                        };
                        if let Some(tessellation) = tessellation {
                            if draw.paint.style == RenderPaintStyle::Fill
                                && tessellation.contours.len() != 1
                            {
                                // Compound fills require the upstream stencil-then-cover path.
                            } else {
                                let tessellation_height =
                                    draw::tessellation_texture_height(&tessellation.spans);
                                let uniforms =
                                    analytic_uniforms(self.width, self.height, tessellation_height);
                                let tessellation_texture = self.context.tessellator.encode(
                                    &self.context.device,
                                    encoder,
                                    &self.context.feather_lut.view,
                                    &tessellation.spans,
                                    &uniforms,
                                    std::slice::from_ref(&tessellation.path),
                                    &tessellation.contours,
                                    tessellation_height,
                                );
                                let tessellation_view = tessellation_texture
                                    .create_view(&wgpu::TextureViewDescriptor::default());
                                let mut paint = if draw.paint.style == RenderPaintStyle::Stroke {
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
                                let paint_aux = clip_rect_paint_aux(draw.state.clip_rect);
                                prepared_draws.push(PreparedDraw::Analytic(
                                    self.context.path_pipeline.prepare(
                                        &self.context.device,
                                        &tessellation_view,
                                        &self.context.feather_lut.view,
                                        &uniforms,
                                        &tessellation.path,
                                        &paint,
                                        &paint_aux,
                                        &tessellation.contours,
                                        tessellation.base_instance,
                                        tessellation.instance_count,
                                    ),
                                ));
                                continue;
                            }
                        }
                    }
                    if let Some(path_vertices) = tessellate_solid(draw, self.width, self.height) {
                        let cover_vertices = cover_vertices(&path_vertices);
                        let path_buffer = self.context.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("nuxie-path-vertices"),
                                contents: bytemuck::cast_slice(&path_vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            },
                        );
                        let cover_buffer = self.context.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("nuxie-path-cover"),
                                contents: bytemuck::cast_slice(&cover_vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            },
                        );
                        prepared_draws.push(PreparedDraw::Bootstrap(
                            path_buffer,
                            cover_buffer,
                            draw.path.fill_rule,
                        ));
                    }
                }
                let fallback_texture =
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
                        });
                let fallback_view = fallback_texture.create_view(&Default::default());
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
                    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("nuxie-fallback-frame-clear"),
                        color_attachments: &attachments,
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                }
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("nuxie-solid-pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &multisample_view,
                            depth_slice: None,
                            resolve_target: Some(&fallback_view),
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &stencil_view,
                            depth_ops: None,
                            stencil_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(0),
                                store: wgpu::StoreOp::Discard,
                            }),
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    pass.set_stencil_reference(0);
                    for prepared in &prepared_draws {
                        match prepared {
                            PreparedDraw::Analytic(draw) => {
                                pass.set_pipeline(&self.context.path_pipeline.pipeline);
                                pass.set_bind_group(0, &draw.flush_group, &[]);
                                pass.set_bind_group(1, &draw.image_group, &[]);
                                pass.set_bind_group(3, &draw.sampler_group, &[]);
                                pass.set_vertex_buffer(
                                    0,
                                    self.context.patch_vertex_buffer.slice(..),
                                );
                                pass.set_index_buffer(
                                    self.context.patch_index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint16,
                                );
                                pass.draw_indexed(
                                    0..gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT as u32,
                                    0,
                                    draw.base_instance..draw.base_instance + draw.instance_count,
                                );
                            }
                            PreparedDraw::Bootstrap(path_buffer, cover_buffer, fill_rule) => {
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
                }
                self.context.composite_pipeline.encode(
                    &self.context.device,
                    encoder,
                    &view,
                    &fallback_view,
                );
            };
        if self.draws.is_empty() || self.mode == RenderMode::Msaa {
            encode_fallback_run(&self.draws, true, &mut encoder);
        } else {
            let mut start = 0;
            let mut clear_target = true;
            while start < self.draws.len() {
                let atomic = atomic_draw_is_eligible(&self.draws[start]);
                let clockwise_atomic = atomic && draw_requires_clockwise_atomic(&self.draws[start]);
                let mut end = start + 1;
                while end < self.draws.len()
                    && atomic_draw_is_eligible(&self.draws[end]) == atomic
                    && (!atomic
                        || draw_requires_clockwise_atomic(&self.draws[end]) == clockwise_atomic)
                {
                    end += 1;
                }
                if atomic {
                    let has_clip_updates = self.draws[start..end]
                        .iter()
                        .any(|draw| matches!(draw.role, DrawRole::ClipUpdate { .. }));
                    let has_advanced_blend =
                        self.draws[start..end].iter().any(draw_uses_advanced_blend);
                    if has_advanced_blend {
                        if self.draws[start..end]
                            .iter()
                            .any(|draw| draw.paint.feather != 0.0)
                        {
                            return Err(RendererError::Unsupported(
                                "advanced atomic blending with feather",
                            ));
                        }
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
                            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                            false,
                            clockwise_atomic,
                            Some(&load_view),
                            &mut encoder,
                        )?;
                    } else if clockwise_atomic || has_clip_updates {
                        encode_atomic_run(
                            &self.draws[start..end],
                            clear_target,
                            clockwise_atomic,
                            None,
                            &mut encoder,
                        )?;
                    } else {
                        for group in disjoint_atomic_draw_groups(
                            &self.draws[start..end],
                            self.width,
                            self.height,
                        ) {
                            encode_atomic_run(&group, clear_target, false, None, &mut encoder)?;
                            let next_encoder = self.context.device.create_command_encoder(
                                &wgpu::CommandEncoderDescriptor {
                                    label: Some("nuxie-frame-encoder"),
                                },
                            );
                            let submitted_encoder = std::mem::replace(&mut encoder, next_encoder);
                            self.context.queue.submit(Some(submitted_encoder.finish()));
                            self.context
                                .device
                                .poll(wgpu::PollType::wait_indefinitely())
                                .map_err(|error| RendererError::Map(error.to_string()))?;
                            clear_target = false;
                        }
                    }
                } else {
                    encode_fallback_run(&self.draws[start..end], clear_target, &mut encoder);
                }
                clear_target = false;
                start = end;
            }
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
        self.context.queue.submit(Some(encoder.finish()));
        let slice = readback.slice(..);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| RendererError::Map(error.to_string()))?;
        receiver
            .recv()
            .map_err(|error| RendererError::Map(error.to_string()))?
            .map_err(|error| RendererError::Map(error.to_string()))?;
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
                borrowed: read_u32_buffer(&self.context, &readback.borrowed, readback.word_count)?,
                main: read_u32_buffer(&self.context, &readback.main, readback.word_count)?,
                ranges,
                kinds,
                clip_updates: readback
                    .clip_updates
                    .iter()
                    .map(|buffer| {
                        read_u8_buffer(
                            &self.context,
                            buffer,
                            readback.clip_bytes_per_row as usize * readback.clip_height as usize,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                clip_bytes_per_row: readback.clip_bytes_per_row,
            });
        }
        Ok((pixels, coverage_snapshots))
    }
}

fn read_u32_buffer(
    context: &Context,
    buffer: &wgpu::Buffer,
    word_count: usize,
) -> Result<Vec<u32>, RendererError> {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    context
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| RendererError::Map(error.to_string()))?;
    receiver
        .recv()
        .map_err(|error| RendererError::Map(error.to_string()))?
        .map_err(|error| RendererError::Map(error.to_string()))?;
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

fn read_u8_buffer(
    context: &Context,
    buffer: &wgpu::Buffer,
    byte_count: usize,
) -> Result<Vec<u8>, RendererError> {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    context
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| RendererError::Map(error.to_string()))?;
    receiver
        .recv()
        .map_err(|error| RendererError::Map(error.to_string()))?
        .map_err(|error| RendererError::Map(error.to_string()))?;
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
    const PADDING: f32 = 2.0;
    Some(AtlasPlacement {
        scale,
        translate: [PADDING - left as f32 * scale, PADDING - top as f32 * scale],
        bounds: [left as f32, top as f32, right as f32, bottom as f32],
        origin: [0, 0],
        width: ((right - left) as f32 * scale).ceil() as u32 + 4,
        height: ((bottom - top) as f32 * scale).ceil() as u32 + 4,
    })
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

fn wgpu_path(path: &dyn RenderPath) -> &WgpuPath {
    path.as_any()
        .downcast_ref()
        .expect("nuxie-renderer received a foreign path")
}

fn wgpu_paint(paint: &dyn RenderPaint) -> &WgpuPaint {
    paint
        .as_any()
        .downcast_ref()
        .expect("nuxie-renderer received a foreign paint")
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

fn decode_jpeg_rgba(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(data));
    let decoded = decoder.decode().ok()?;
    let info = decoder.info()?;
    let pixels = match info.pixel_format {
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
    let rgba = rgba(value, 1.0);
    wgpu::Color {
        r: rgba[0] as f64,
        g: rgba[1] as f64,
        b: rgba[2] as f64,
        a: rgba[3] as f64,
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

fn draw_uses_advanced_blend(draw: &SolidDraw) -> bool {
    draw.paint.blend_mode != BlendMode::SrcOver
        || draw
            .image
            .as_ref()
            .is_some_and(|image| image.blend_mode() != BlendMode::SrcOver)
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

fn disjoint_atomic_draw_groups_with_limit(
    draws: &[SolidDraw],
    viewport_width: u32,
    viewport_height: u32,
    group_limit: usize,
) -> Vec<Vec<SolidDraw>> {
    assert!((1..i16::MAX as usize).contains(&group_limit));
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
    let logical_offset = y * gpu::TESS_TEXTURE_WIDTH as u32 + x;
    assert_eq!(
        logical_offset % gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32,
        0
    );
    *base_instance += logical_offset / gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
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

fn draw_requires_clockwise_atomic(draw: &SolidDraw) -> bool {
    matches!(draw.role, DrawRole::Content { clip_id: 0 })
        && draw.paint.style == RenderPaintStyle::Fill
        && draw.paint.feather == 0.0
        && draw.state.clip_rect.is_none()
        && path_has_complex_fill_topology(&draw.path.raw_path)
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
    const ATLAS_ORACLE_TOLERANCES: atlas_mask_oracle::MaskComparisonTolerances =
        atlas_mask_oracle::MaskComparisonTolerances {
            support: 1.0 / 1024.0,
            value: 1.0 / 512.0,
        };

    #[test]
    fn oversized_atlas_layout_returns_renderer_error_before_wgpu() {
        let result = pack_atlas_for_device(1920, 2048, &[(1920, 100); 21]);

        assert!(matches!(result, Err(RendererError::AtlasPacking(_))));
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
    fn decodes_corpus_jpeg_to_opaque_rgba() {
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

        let (width, height, rgba) = decode_image_rgba(&encoded).expect("JPEG must decode");
        assert_eq!((width, height), (278, 278));
        assert_eq!(rgba.len(), 278 * 278 * 4);
        assert!(rgba.chunks_exact(4).all(|pixel| pixel[3] == 255));
    }

    #[test]
    fn rejects_unknown_encoded_image_format() {
        assert!(decode_image_rgba(b"not an image").is_none());
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

        let image = factory.decode_image(&encoded);

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
    fn atomic_image_mesh_draws_indexed_position_and_uv_buffers() {
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

        let mut factory = WgpuFactory::new_with_mode(16, 16, RenderMode::ClockwiseAtomic).unwrap();
        let image = factory.decode_image(&encoded);
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
        assert_eq!(pixel(4, 4), [255, 0, 0, 255]);
        assert_eq!(pixel(15, 15), [0, 0, 0, 255]);

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
            assert_eq!(pixel(4, 4), expected, "{blend_mode:?}");
            assert_eq!(pixel(15, 15), [0, 255, 0, 255], "{blend_mode:?}");
        }
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
            raw_path,
            fill_rule,
        }
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
            raw_path: red_raw_path,
            fill_rule: FillRule::NonZero,
        };
        let mut ring_raw_path = RawPath::new();
        append_oval(&mut ring_raw_path, [70.0, 70.0, 200.0, 200.0]);
        let mut inner = RawPath::new();
        append_oval(&mut inner, [90.0, 90.0, 180.0, 180.0]);
        ring_raw_path.add_path_backwards(&inner, Mat2D::IDENTITY);
        let ring_path = WgpuPath {
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
            raw_path,
            fill_rule: FillRule::Clockwise,
        }
    }

    fn assert_post_contour_padding(tessellation: &draw::FillTessellation) {
        let logical_end = (tessellation.base_instance + tessellation.instance_count)
            * gpu::MIDPOINT_FAN_PATCH_SEGMENT_SPAN as u32;
        let alignment = gpu::OUTER_CURVE_PATCH_SEGMENT_SPAN as u32;
        let index = logical_end.div_ceil(alignment) * alignment;
        let padding = tessellation.spans.last().unwrap();
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
            assert_eq!(
                tessellation.spans[1..5]
                    .iter()
                    .map(|span| span.x_range())
                    .collect::<Vec<_>>(),
                vec![(8, 18), (18, 28), (28, 38), (38, 48)]
            );
            assert!(tessellation.spans[1..5]
                .iter()
                .all(|span| span.segment_counts == 0x0090_0401));
            assert!(tessellation.spans[1..5]
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
            assert_eq!(tessellation.spans[1].x_range(), (8, 27));
            assert_eq!(tessellation.spans[2].x_range(), (27, 48));
            assert_eq!(tessellation.spans[1].segment_counts, 0x0140_0000);
            assert_eq!(tessellation.spans[2].segment_counts, 0x0140_0401);
            assert_eq!(tessellation.spans[1].contour_id_with_flags, 0x0a00_0001);
            assert_eq!(tessellation.spans[2].contour_id_with_flags, 0x0a00_0001);
            assert_post_contour_padding(&tessellation);
        }
    }

    #[test]
    fn culls_empty_and_invalid_path_draws_like_cpp() {
        let empty = WgpuPath {
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
    fn paint_thickness_matches_cpp_absolute_value_setter() {
        let mut paint = WgpuPaint::default();
        RenderPaint::thickness(&mut paint, -3.5);
        assert_eq!(paint.thickness, 3.5);

        RenderPaint::thickness(&mut paint, f32::NAN);
        assert!(paint.thickness.is_nan());
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

    fn coverage_word_at(words: &[u32], range: gpu::CoverageBufferRange, x: u32, y: u32) -> u32 {
        let x = (x as f32 + range.offset_x).floor() as u32;
        let y = (y as f32 + range.offset_y).floor() as u32;
        let index = range.offset
            + (y >> 5) * (range.pitch << 5)
            + (x >> 5) * 1024
            + ((x & 28) << 5)
            + ((y & 28) << 2)
            + ((y & 3) << 2)
            + (x & 3);
        words[index as usize]
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
    fn arbitrary_clip_path_updates_atomic_clip_buffer() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let mut clip_path = RawPath::new();
        clip_path.move_to(8.0, 8.0);
        clip_path.line_to(56.0, 8.0);
        clip_path.line_to(32.0, 56.0);
        clip_path.close();
        let clip = WgpuPath {
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
    fn arbitrary_clip_applies_to_atomic_gradient_content() {
        let factory = WgpuFactory::new_with_mode(64, 64, RenderMode::ClockwiseAtomic).unwrap();
        let mut clip_path = RawPath::new();
        clip_path.move_to(8.0, 8.0);
        clip_path.line_to(56.0, 8.0);
        clip_path.line_to(32.0, 56.0);
        clip_path.close();
        let clip = WgpuPath {
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
    fn large_clip_falls_back_when_interior_triangulation_fails() {
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
        .is_none());

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
        factory.context.queue.write_texture(
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
        let mut encoder =
            factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-composite-test-encoder"),
                });
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
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        factory.context.queue.submit(Some(encoder.finish()));
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
        let mut encoder =
            factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-tessellation-test-encoder"),
                });
        let texture = factory.context.tessellator.encode(
            &factory.context.device,
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
        factory.context.queue.submit(Some(encoder.finish()));
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
        let mut tessellation =
            draw::build_feather_tessellation(&raw_path, transform, 1.0, None).unwrap();
        for contour in &mut tessellation.contours {
            contour.path_id = 1;
        }
        let factory = WgpuFactory::new(ATLAS_ORACLE_FRAME_SIZE, ATLAS_ORACLE_FRAME_SIZE).unwrap();
        let logical_tessellation_height = draw::tessellation_texture_height(&tessellation.spans);
        // The C++ oracle exports the complete allocation, including its 125% growth tail.
        let tessellation_height = logical_tessellation_height * 5 / 4;
        let uniforms = analytic_uniforms(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            tessellation_height,
        );
        let paths = [gpu::PathData::zeroed(), tessellation.path];
        let mut encoder =
            factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-direct-input-encoder"),
                });
        let texture = factory.context.tessellator.encode(
            &factory.context.device,
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
        factory.context.queue.submit(Some(encoder.finish()));
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

    fn fixed_feather_atlas_oracle_for(
        raw_path: RawPath,
        paint: WgpuPaint,
    ) -> FixedFeatherAtlasOracle {
        let stroke = paint.effective_stroke();
        let factory = WgpuFactory::new(ATLAS_ORACLE_FRAME_SIZE, ATLAS_ORACLE_FRAME_SIZE).unwrap();
        let mut placement = feather_atlas_placement(
            &raw_path,
            Mat2D::IDENTITY,
            ATLAS_ORACLE_FEATHER,
            stroke,
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
        )
        .unwrap();
        assert_eq!(placement.bounds, [0.0, 0.0, 64.0, 64.0]);
        assert_eq!([placement.width, placement.height], [39, 39]);
        let layout = pack_atlas_for_device(
            ATLAS_ORACLE_FRAME_SIZE,
            factory.context.device.limits().max_texture_dimension_2d,
            &[(placement.width, placement.height)],
        )
        .unwrap();
        assert_eq!(layout.extent(), [ATLAS_ORACLE_LOGICAL_SIZE; 2]);
        assert_eq!(layout.origins(), &[[0, 0]]);
        placement.origin = layout.origins()[0];
        placement.translate[0] += placement.origin[0] as f32;
        placement.translate[1] += placement.origin[1] as f32;
        assert_eq!(placement.translate, ATLAS_ORACLE_PLACEMENT);
        let mut tessellation = draw::build_feather_atlas_tessellation(
            &raw_path,
            Mat2D::IDENTITY,
            ATLAS_ORACLE_FEATHER,
            stroke,
        )
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
        let mut uniforms = analytic_uniforms(
            ATLAS_ORACLE_FRAME_SIZE,
            ATLAS_ORACLE_FRAME_SIZE,
            tessellation_height,
        );
        uniforms.atlas_texture_inverse_size = [1.0 / ATLAS_ORACLE_PHYSICAL_SIZE as f32; 2];
        uniforms.atlas_content_inverse_viewport =
            [2.0 / placement.width as f32, -2.0 / placement.height as f32];
        let mut encoder =
            factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-atlas-test-encoder"),
                });
        let tessellation_texture = factory.context.tessellator.encode(
            &factory.context.device,
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
                    width: ATLAS_ORACLE_PHYSICAL_SIZE,
                    height: ATLAS_ORACLE_PHYSICAL_SIZE,
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
            [ATLAS_ORACLE_LOGICAL_SIZE; 2],
            [
                placement.origin[0],
                placement.origin[1],
                placement.width,
                placement.height,
            ],
        );
        let bytes_per_row = 256;
        let atlas_readback = factory
            .context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("nuxie-atlas-test-readback"),
                size: u64::from(bytes_per_row * ATLAS_ORACLE_PHYSICAL_SIZE),
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
                    rows_per_image: Some(ATLAS_ORACLE_PHYSICAL_SIZE),
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
        factory.context.queue.submit(Some(encoder.finish()));
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
        let size = ATLAS_ORACLE_PHYSICAL_SIZE as usize;
        let mut pixels = Vec::with_capacity(size * size);
        for y in 0..size {
            let row = &mapped[y * bytes_per_row as usize..][..size * 2];
            pixels.extend(
                row.chunks_exact(2)
                    .map(|sample| u16::from_le_bytes(sample.try_into().unwrap())),
            );
        }
        drop(mapped);
        atlas_readback.unmap();
        let mask = atlas_mask_oracle::AtlasMask::new(
            ATLAS_ORACLE_PHYSICAL_SIZE,
            ATLAS_ORACLE_PHYSICAL_SIZE,
            pixels,
        )
        .unwrap();
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
        FixedFeatherAtlasOracle { mask, inputs }
    }

    fn fixed_feather_atlas_mask(join: StrokeJoin) -> atlas_mask_oracle::AtlasMask {
        fixed_feather_atlas_oracle(join).mask
    }

    fn fixed_feather_atlas_blit() -> atlas_blit_oracle::AtlasBlit {
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
        frame.draw_path(&path, &paint);
        let pixels = frame.finish().unwrap();
        atlas_blit_oracle::AtlasBlit::new(ATLAS_ORACLE_FRAME_SIZE, ATLAS_ORACLE_FRAME_SIZE, pixels)
            .unwrap()
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
