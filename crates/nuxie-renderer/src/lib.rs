//! Pure-Rust wgpu renderer behind the `nuxie-render-api` trait boundary.

mod atomic_pipeline;
mod draw;
mod gpu;
mod path_pipeline;
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
    Device(String),
    Map(String),
    Unsupported(&'static str),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adapter(message) => write!(f, "wgpu adapter error: {message}"),
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
        if path_draw_is_noop(path, paint) {
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
        let atomic_eligible = self.mode == RenderMode::ClockwiseAtomic
            && !self.draws.is_empty()
            && self.draws.iter().all(|draw| {
                draw.paint.shader.is_none()
                    && draw.paint.feather == 0.0
                    && match draw.paint.style {
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
            });
        let used_atomic = if atomic_eligible {
            struct PreparedAtomicDraw {
                spans: Vec<gpu::TessVertexSpan>,
                base_instance: u32,
                instance_count: u32,
                patch_index_range: std::ops::Range<u32>,
                triangles: Vec<gpu::TriangleVertex>,
                is_stroke: bool,
            }

            {
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
            let mut prepared = Vec::with_capacity(self.draws.len());
            let mut paths = vec![gpu::PathData::zeroed()];
            let mut paints = vec![gpu::PaintData::solid(
                0,
                FillRule::NonZero,
                BlendMode::SrcOver,
            )];
            let mut contours = Vec::new();
            for (draw_index, draw) in self.draws.iter().enumerate() {
                let path_id = u16::try_from(draw_index + 1).expect("atomic path ID overflow");
                let (
                    mut spans,
                    mut path,
                    mut draw_contours,
                    base_instance,
                    instance_count,
                    patch_index_range,
                    mut triangles,
                ) = if draw.paint.style == RenderPaintStyle::Stroke {
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
                    let mut tessellation =
                        draw::build_fill_tessellation(&draw.path.raw_path, draw.state.transform)
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
                prepared.push(PreparedAtomicDraw {
                    spans,
                    base_instance,
                    instance_count,
                    patch_index_range,
                    triangles,
                    is_stroke: draw.paint.style == RenderPaintStyle::Stroke,
                });
            }
            let tessellation_height = prepared
                .iter()
                .map(|draw| draw::tessellation_texture_height(&draw.spans))
                .max()
                .unwrap_or(1);
            let mut uniforms = analytic_uniforms(self.width, self.height, tessellation_height);
            uniforms.render_target_update_bounds = [0, 0, self.width as i32, self.height as i32];
            let mut tessellation_textures = Vec::with_capacity(prepared.len());
            for draw in &prepared {
                let tessellation_texture = self.context.tessellator.encode(
                    &self.context.device,
                    &mut encoder,
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
            let atomic_draws = prepared
                .iter()
                .zip(&tessellation_views)
                .map(|(draw, tessellation)| atomic_pipeline::AtomicDraw {
                    tessellation,
                    base_instance: draw.base_instance,
                    instance_count: draw.instance_count,
                    patch_index_range: draw.patch_index_range.clone(),
                    triangle_vertices: &draw.triangles,
                    is_stroke: draw.is_stroke,
                })
                .collect::<Vec<_>>();
            self.context.atomic_pipeline.encode_batch(
                &self.context.device,
                &mut encoder,
                &view,
                &self.context.patch_vertex_buffer,
                &self.context.patch_index_buffer,
                &atomic_draws,
                &uniforms,
                &paths,
                &paints,
                &vec![gpu::PaintAuxData::zeroed(); paths.len()],
                &contours,
                padded_width as usize * padded_height as usize,
            );
            true
        } else {
            false
        };
        enum PreparedDraw {
            Analytic(path_pipeline::PreparedPathDraw),
            Bootstrap(wgpu::Buffer, wgpu::Buffer, FillRule),
        }
        let mut prepared_draws = Vec::with_capacity(self.draws.len());
        for draw in self
            .draws
            .iter()
            .take(if used_atomic { 0 } else { usize::MAX })
        {
            if draw.paint.shader.is_none() && draw.paint.feather == 0.0 {
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
                            &mut encoder,
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
                let path_buffer =
                    self.context
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("nuxie-path-vertices"),
                            contents: bytemuck::cast_slice(&path_vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                let cover_buffer =
                    self.context
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("nuxie-path-cover"),
                            contents: bytemuck::cast_slice(&cover_vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                prepared_draws.push(PreparedDraw::Bootstrap(
                    path_buffer,
                    cover_buffer,
                    draw.path.fill_rule,
                ));
            }
        }
        if !used_atomic {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("nuxie-solid-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &multisample_view,
                    depth_slice: None,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(color(self.clear_color)),
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
                        pass.set_vertex_buffer(0, self.context.patch_vertex_buffer.slice(..));
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
                            0..(path_buffer.size() / std::mem::size_of::<Vertex>() as u64) as u32,
                            0..1,
                        );
                        pass.set_pipeline(&self.context.cover_pipeline);
                        pass.set_vertex_buffer(0, cover_buffer.slice(..));
                        pass.draw(0..6, 0..1);
                    }
                }
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

fn analytic_uniforms(width: u32, height: u32, tessellation_height: u32) -> gpu::FlushUniforms {
    let mut uniforms = gpu::FlushUniforms::zeroed();
    uniforms.inverse_viewports = [
        2.0,
        2.0 / tessellation_height.max(1) as f32,
        2.0 / width as f32,
        -2.0 / height as f32,
    ];
    uniforms.render_target_width = width;
    uniforms.render_target_height = height;
    uniforms.path_id_granularity = 1;
    uniforms.vertex_discard_value = f32::NAN;
    uniforms.mip_map_lod_bias = gpu::MIP_MAP_LOD_BIAS;
    uniforms.max_path_id = 1;
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

fn path_draw_is_noop(path: &WgpuPath, paint: &WgpuPaint) -> bool {
    path.raw_path.verbs().is_empty()
        || (paint.style == RenderPaintStyle::Stroke && !(paint.thickness > 0.0))
        || !(paint.feather >= 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn matrix_composition_matches_renderer_post_concat() {
        let translated = Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 20.0]);
        let scaled = Mat2D([2.0, 0.0, 0.0, 3.0, 0.0, 0.0]);
        let result = multiply(translated, scaled);
        assert_eq!(result.0, [2.0, 0.0, 0.0, 3.0, 10.0, 20.0]);
    }

    #[test]
    fn culls_empty_and_invalid_path_draws_like_cpp() {
        let empty = WgpuPath {
            raw_path: RawPath::new(),
            fill_rule: FillRule::NonZero,
        };
        let mut path = empty.clone();
        path.raw_path.move_to(0.0, 0.0);
        path.raw_path.line_to(1.0, 1.0);
        let mut paint = WgpuPaint::default();

        assert!(path_draw_is_noop(&empty, &paint));
        assert!(!path_draw_is_noop(&path, &paint));

        paint.style = RenderPaintStyle::Stroke;
        paint.thickness = 0.0;
        assert!(path_draw_is_noop(&path, &paint));
        paint.thickness = f32::NAN;
        assert!(path_draw_is_noop(&path, &paint));
        paint.thickness = 1.0;
        paint.feather = f32::NAN;
        assert!(path_draw_is_noop(&path, &paint));
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
    fn upstream_tessellation_pass_executes() {
        let factory = WgpuFactory::new(64, 64).unwrap();
        let mut uniforms = gpu::FlushUniforms::zeroed();
        uniforms.inverse_viewports[1] = 2.0;
        let span = gpu::TessVertexSpan::without_reflection(
            [[4.0, 4.0], [20.0, 4.0], [44.0, 4.0], [60.0, 4.0]],
            [1.0, 0.0],
            0.0,
            0,
            2,
            1,
            0,
            1,
            1,
        );
        let paths = [gpu::PathData::zeroed()];
        let contours = [gpu::ContourData::new([32.0, 4.0], 0, 0)];
        let mut encoder =
            factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-tessellation-test-encoder"),
                });
        let _texture = factory.context.tessellator.encode(
            &factory.context.device,
            &mut encoder,
            &[span],
            &uniforms,
            &paths,
            &contours,
            1,
        );
        factory.context.queue.submit(Some(encoder.finish()));
        factory
            .context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
    }
}
