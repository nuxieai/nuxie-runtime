//! Pure-Rust wgpu renderer behind the `nuxie-render-api` trait boundary.

#[cfg(test)]
mod atlas_blit_oracle;
#[cfg(test)]
mod atlas_input_oracle;
#[cfg(test)]
mod atlas_mask_oracle;
mod atlas_pipeline;
mod atomic_pipeline;
mod composite_pipeline;
mod draw;
mod feather_lut;
mod gpu;
// Kept standalone until a renderer path has a proven grouping integration.
#[allow(dead_code)]
mod intersection_board;
mod path_pipeline;
mod skyline;
mod tessellator;

use bytemuck::{Pod, Zeroable};
use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageSampler, Mat2D, RawPath, RenderBuffer,
    RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint, RenderPaintStyle, RenderPath,
    RenderShader, Renderer, StrokeCap, StrokeJoin,
};
use std::any::Any;
use std::error::Error;
use std::fmt;
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
    atlas_pipeline: atlas_pipeline::AtlasPipeline,
    composite_pipeline: composite_pipeline::CompositePipeline,
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
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("nuxie-renderer-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_storage_buffers_per_shader_stage: 6,
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
        let atlas_pipeline = atlas_pipeline::AtlasPipeline::new(&device);
        let composite_pipeline = composite_pipeline::CompositePipeline::new(&device);
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
                atlas_pipeline,
                composite_pipeline,
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
            buffer_type,
            flags,
            bytes: vec![0; size_in_bytes],
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

    fn decode_image(&mut self, _data: &[u8]) -> Box<dyn RenderImage> {
        Box::new(WgpuImage {
            width: 0,
            height: 0,
        })
    }
}

#[derive(Debug, Clone)]
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
        self.thickness = value;
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
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    bytes: Vec<u8>,
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
    fn unmap(&mut self) {}
}

struct WgpuImage {
    width: u32,
    height: u32,
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
struct DrawState {
    transform: Mat2D,
    opacity: f32,
}

impl Default for DrawState {
    fn default() -> Self {
        Self {
            transform: Mat2D::IDENTITY,
            opacity: 1.0,
        }
    }
}

struct SolidDraw {
    path: WgpuPath,
    paint: WgpuPaint,
    state: DrawState,
}

pub struct WgpuFrame {
    context: Arc<Context>,
    width: u32,
    height: u32,
    clear_color: ColorInt,
    state: DrawState,
    stack: Vec<DrawState>,
    draws: Vec<SolidDraw>,
    unsupported: Option<&'static str>,
    mode: RenderMode,
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
        let path = wgpu_path(path);
        let paint = wgpu_paint(paint);
        if path_draw_is_noop(path, paint, self.state.transform) {
            return;
        }
        self.draws.push(SolidDraw {
            path: path.clone(),
            paint: paint.clone(),
            state: self.state,
        });
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        if !is_full_target_clip(
            wgpu_path(path),
            self.state.transform,
            self.width,
            self.height,
        ) {
            self.unsupported.get_or_insert("clip paths");
        }
    }

    fn draw_image(
        &mut self,
        _image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        _blend_mode: BlendMode,
        _opacity: f32,
    ) {
        self.unsupported.get_or_insert("images");
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
        self.unsupported.get_or_insert("image meshes");
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        self.state.opacity *= opacity;
    }
}

impl WgpuFrame {
    pub fn finish(self) -> Result<Vec<u8>, RendererError> {
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
        let encode_atomic_run =
            |draws: &[SolidDraw], clear_target: bool, encoder: &mut wgpu::CommandEncoder| {
                struct PreparedAtomicDraw {
                    spans: Vec<gpu::TessVertexSpan>,
                    base_instance: u32,
                    instance_count: u32,
                    patch_index_range: std::ops::Range<u32>,
                    triangles: Vec<gpu::TriangleVertex>,
                    atlas: Option<AtlasPlacement>,
                    atlas_blit_vertices: Vec<gpu::TriangleVertex>,
                    is_stroke: bool,
                    is_feather: bool,
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
                let mut prepared = Vec::with_capacity(draws.len());
                let mut paths = vec![gpu::PathData::zeroed()];
                let mut paints = vec![gpu::PaintData::solid(
                    0,
                    FillRule::NonZero,
                    BlendMode::SrcOver,
                )];
                let mut contours = Vec::new();
                for (draw_index, draw) in draws.iter().enumerate() {
                    let path_id = u16::try_from(draw_index + 1).expect("atomic path ID overflow");
                    let (
                        mut spans,
                        mut path,
                        mut draw_contours,
                        base_instance,
                        instance_count,
                        patch_index_range,
                        mut triangles,
                    ) = if draw.paint.feather != 0.0 {
                        let stroke = draw.paint.effective_stroke();
                        let is_stroke = stroke.is_some();
                        let requires_atlas = draw::feather_requires_atlas(
                            draw.paint.feather,
                            draw.state.transform,
                            false,
                        );
                        let tessellation = if requires_atlas {
                            draw::build_feather_atlas_tessellation(
                                &draw.path.raw_path,
                                draw.state.transform,
                                draw.paint.feather,
                                stroke,
                            )
                        } else {
                            draw::build_feather_tessellation(
                                &draw.path.raw_path,
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
                            &draw.path.raw_path,
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
                    } else if draw::should_use_interior_tessellation(
                        &draw.path.raw_path,
                        draw.state.transform,
                    ) {
                        let tessellation = draw::build_interior_tessellation(
                            &draw.path.raw_path,
                            draw.state.transform,
                        )
                        .expect("atomic eligibility already validated tessellation");
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
                        let mut tessellation = draw::build_fill_tessellation(
                            &draw.path.raw_path,
                            draw.state.transform,
                        )
                        .expect("atomic eligibility already validated tessellation");
                        tessellation.make_double_sided();
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
                    path.coverage_buffer_range.pitch = padded_width;
                    paths.push(path);
                    paints.push(if draw.paint.style == RenderPaintStyle::Stroke {
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
                    });
                    contours.extend(draw_contours);
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
                        triangles,
                        atlas,
                        atlas_blit_vertices,
                        is_stroke: draw.paint.style == RenderPaintStyle::Stroke,
                        is_feather: draw.paint.feather != 0.0,
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
                let tessellation_height = prepared
                    .iter()
                    .map(|draw| draw::tessellation_texture_height(&draw.spans))
                    .max()
                    .unwrap_or(1);
                let mut uniforms = analytic_uniforms(self.width, self.height, tessellation_height);
                uniforms.render_target_update_bounds =
                    [0, 0, self.width as i32, self.height as i32];
                uniforms.atlas_texture_inverse_size = [
                    1.0 / atlas_physical_size[0] as f32,
                    1.0 / atlas_physical_size[1] as f32,
                ];
                uniforms.atlas_content_inverse_viewport =
                    [2.0 / atlas_width as f32, -2.0 / atlas_height as f32];
                let mut tessellation_textures = Vec::with_capacity(prepared.len());
                for draw in &prepared {
                    let tessellation_texture = self.context.tessellator.encode(
                        &self.context.device,
                        encoder,
                        &self.context.feather_lut.view,
                        &draw.spans,
                        &uniforms,
                        &paths,
                        &contours,
                        tessellation_height,
                    );
                    tessellation_textures.push(tessellation_texture);
                }
                let tessellation_views = tessellation_textures
                    .iter()
                    .map(|texture| texture.create_view(&wgpu::TextureViewDescriptor::default()))
                    .collect::<Vec<_>>();
                let paint_aux = vec![gpu::PaintAuxData::zeroed(); paths.len()];
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
                    for (index, draw) in prepared.iter().enumerate() {
                        if let Some(atlas) = draw.atlas {
                            self.context.atlas_pipeline.encode_mask(
                                &self.context.device,
                                encoder,
                                &view,
                                &self.context.patch_vertex_buffer,
                                &self.context.patch_index_buffer,
                                &tessellation_views[index],
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
                    .zip(&tessellation_views)
                    .map(|(draw, tessellation)| atomic_pipeline::AtomicDraw {
                        tessellation,
                        base_instance: draw.base_instance,
                        instance_count: draw.instance_count,
                        patch_index_range: draw.patch_index_range.clone(),
                        triangle_vertices: &draw.triangles,
                        atlas: draw.atlas.and(atlas_view.as_ref()),
                        atlas_blit_vertices: &draw.atlas_blit_vertices,
                        is_stroke: draw.is_stroke,
                        is_feather: draw.is_feather,
                    })
                    .collect::<Vec<_>>();
                self.context.atomic_pipeline.encode_batch(
                    &self.context.device,
                    encoder,
                    &view,
                    &self.context.feather_lut.view,
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
                                let paint = if draw.paint.style == RenderPaintStyle::Stroke {
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
                                prepared_draws.push(PreparedDraw::Analytic(
                                    self.context.path_pipeline.prepare(
                                        &self.context.device,
                                        &tessellation_view,
                                        &self.context.feather_lut.view,
                                        &uniforms,
                                        &tessellation.path,
                                        &paint,
                                        &gpu::PaintAuxData::zeroed(),
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
                let mut end = start + 1;
                while end < self.draws.len() && atomic_draw_is_eligible(&self.draws[end]) == atomic
                {
                    end += 1;
                }
                if atomic {
                    encode_atomic_run(&self.draws[start..end], clear_target, &mut encoder)?;
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
        Ok(pixels)
    }
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

fn is_full_target_clip(path: &WgpuPath, transform: Mat2D, width: u32, height: u32) -> bool {
    let points = path
        .raw_path
        .points()
        .iter()
        .copied()
        .map(|point| transform.transform_point(point))
        .collect::<Vec<_>>();
    if points.len() != 4 {
        return false;
    }
    let min_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::INFINITY, f32::min);
    let max_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::INFINITY, f32::min);
    let max_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::NEG_INFINITY, f32::max);
    min_x <= 0.0 && min_y <= 0.0 && max_x >= width as f32 && max_y >= height as f32
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

fn atomic_draw_is_eligible(draw: &SolidDraw) -> bool {
    if draw.paint.shader.is_some() {
        return false;
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
            draw::build_fill_tessellation(&draw.path.raw_path, draw.state.transform)
                .is_some_and(|tessellation| tessellation.contours.len() == 1)
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
        };
        assert_eq!(tessellate_solid(&draw, 10, 10).unwrap().len(), 3);
    }

    #[test]
    fn recognizes_full_target_clip() {
        let mut raw_path = RawPath::new();
        raw_path.move_to(0.0, 0.0);
        raw_path.line_to(64.0, 0.0);
        raw_path.line_to(64.0, 32.0);
        raw_path.line_to(0.0, 32.0);
        raw_path.close();
        assert!(is_full_target_clip(
            &WgpuPath {
                raw_path,
                fill_rule: FillRule::Clockwise,
            },
            Mat2D::IDENTITY,
            64,
            32,
        ));
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
        assert_eq!(pixel(12, 12), [255, 0, 0, 255]);
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
