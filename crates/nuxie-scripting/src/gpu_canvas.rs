//! Pure-Rust execution boundary for editor-authored GPU-canvas Luau.
//!
//! The classes installed here are Rust userdata, not a second Lua
//! implementation of the GPU contract. Luau owns authored control flow while
//! Rust owns buffer bounds, pipeline layout, binding identity, pass lifecycle,
//! and the typed renderer handoff.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use luaur_rt::{
    AnyUserData, Buffer as LuaBuffer, Function, MultiValue, Table, UserData, UserDataFields,
    UserDataMethods, Value, VmState,
};
use nuxie_render_api::{
    Factory as RenderFactory, GpuCanvasPipelineShaders, GpuCanvasPlan, GpuCanvasShader,
    GpuCanvasShaderEntry, GpuCanvasShaderEntrySelection, GpuCanvasShaderStage,
    RenderGpuCanvasShader, RenderImage,
};
pub use nuxie_render_api::{
    GpuCanvasAttachmentView, GpuCanvasBlendState, GpuCanvasColorAttachment, GpuCanvasColorTarget,
    GpuCanvasDepthStencilAttachment, GpuCanvasDepthStencilState, GpuCanvasDrawCommand,
    GpuCanvasIndexBuffer, GpuCanvasIndexedDraw, GpuCanvasPassState, GpuCanvasPipelinePlan,
    GpuCanvasPipelineState, GpuCanvasRenderPass, GpuCanvasResourceLifetime,
    GpuCanvasSamplerBinding, GpuCanvasStencilFace, GpuCanvasTextureBinding, GpuCanvasTextureUpload,
    GpuCanvasUniformBuffer, GpuCanvasVertexAttribute, GpuCanvasVertexBuffer, GpuCanvasVertexLayout,
};

use crate::shader_asset::ShaderAsset;
use crate::vm::{Error, RendererBindings, Result, ScriptVm};

/// Product GPU-canvas resource fences. These are deliberately below WebGPU's
/// portable minimum limits so malformed authored scripts fail in Rust before
/// reaching a backend allocation or validation path.
pub const MAX_GPU_CANVAS_DIMENSION: u32 = 2_048;
pub const MAX_CPU_BUFFER_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_UNIFORM_BUFFER_BYTES: usize = 64 * 1024;
pub const MAX_VERTEX_BUFFER_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_TOTAL_BUFFER_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_LUAU_VM_MEMORY_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_LUAU_INTERRUPTS_PER_CALL: u32 = 50_000;
pub const MAX_GPU_CANVAS_DRAW_INVOCATIONS: u64 = 1_000_000;
pub const MAX_GPU_CANVAS_VERTEX_BUFFERS: usize = 8;
pub const MAX_GPU_CANVAS_VERTEX_ATTRIBUTES: usize = 16;
pub const MAX_GPU_CANVAS_BIND_GROUPS: u32 = 4;
pub const MAX_GPU_CANVAS_UNIFORM_BINDINGS_PER_GROUP: usize = 8;
pub const MAX_GPU_CANVAS_BINDING_INDEX: u32 = 7;
static NEXT_GPU_TEXTURE_RESOURCE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Default)]
struct GpuCanvasResourceBudget {
    allocated_buffer_bytes: usize,
}

impl GpuCanvasResourceBudget {
    fn reserve(&mut self, bytes: usize) -> Result<()> {
        let total = self
            .allocated_buffer_bytes
            .checked_add(bytes)
            .ok_or_else(|| Error::runtime("GPU-canvas buffer budget overflow"))?;
        if total > MAX_TOTAL_BUFFER_BYTES {
            return Err(Error::runtime(format!(
                "GPU-canvas scripts may allocate at most {MAX_TOTAL_BUFFER_BYTES} buffer bytes"
            )));
        }
        self.allocated_buffer_bytes = total;
        Ok(())
    }
}

/// Backwards-compatible name for the backend-neutral plan now owned by the
/// renderer API seam.
pub type GpuCanvasDrawPlan = GpuCanvasPlan;

#[derive(Debug, Clone)]
struct GpuBuffer {
    usage: GpuBufferUsage,
    immutable: bool,
    bytes: Rc<RefCell<Vec<u8>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GpuBufferUsage {
    Uniform,
    Vertex,
    Index,
}

impl UserData for GpuBuffer {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.bytes.borrow().len()));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "write",
            |_,
             this,
             (source, destination_offset, source_offset, byte_length): (
                LuaBuffer,
                Option<usize>,
                Option<usize>,
                Option<usize>,
            )| {
                if this.immutable {
                    return Err(Error::runtime(
                        "GPUBuffer:write: buffer was created with immutable=true",
                    ));
                }
                let destination_offset = destination_offset.unwrap_or(0);
                let source_offset = source_offset.unwrap_or(0);
                if source_offset > source.len() {
                    return Err(Error::runtime(format!(
                        "GPUBuffer:write srcOffset {source_offset} exceeds {} source bytes",
                        source.len()
                    )));
                }
                let byte_length = byte_length.unwrap_or(source.len() - source_offset);
                let source_end = source_offset
                    .checked_add(byte_length)
                    .ok_or_else(|| Error::runtime("GPUBuffer:write source range overflow"))?;
                if source_end > source.len() {
                    return Err(Error::runtime(format!(
                        "GPUBuffer:write source range {source_offset}..{source_end} exceeds {} bytes",
                        source.len()
                    )));
                }
                let destination_len = this.bytes.borrow().len();
                let (destination_offset, destination_end) = checked_gpu_buffer_write_range(
                    byte_length,
                    destination_offset,
                    destination_len,
                )?;
                // LuaBuffer::to_vec performs a host allocation. Only copy
                // after the source length and destination range are accepted.
                let source = source.to_vec();
                let mut destination = this.bytes.borrow_mut();
                destination[destination_offset..destination_end]
                    .copy_from_slice(&source[source_offset..source_end]);
                Ok(())
            },
        );
    }
}

#[derive(Debug, Clone)]
struct GpuTexture {
    resource_id: u64,
    lifetime: GpuCanvasResourceLifetime,
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    format: String,
    texture_type: String,
    render_target: bool,
    sample_count: u32,
    mip_level_count: u32,
    uploads: Rc<RefCell<Vec<GpuCanvasTextureUpload>>>,
}

#[derive(Debug, Clone)]
struct GpuTextureView {
    texture: Option<GpuTexture>,
    canvas: Option<Rc<RefCell<GpuCanvasState>>>,
    dimension: String,
    base_mip_level: u32,
    mip_level_count: u32,
    base_array_layer: u32,
    array_layer_count: u32,
}

impl GpuTextureView {
    fn format(&self) -> String {
        self.texture
            .as_ref()
            .map(|texture| texture.format.clone())
            .unwrap_or_else(|| "rgba8unorm".into())
    }

    fn to_binding(&self, group: u32, binding: u32) -> Result<GpuCanvasTextureBinding> {
        let texture = self
            .texture
            .as_ref()
            .ok_or_else(|| Error::runtime("the GPU canvas presentation view cannot be sampled"))?;
        Ok(GpuCanvasTextureBinding {
            resource_id: texture.resource_id,
            lifetime: texture.lifetime.clone(),
            group,
            binding,
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: texture.depth_or_array_layers,
            format: texture.format.clone(),
            texture_type: texture.texture_type.clone(),
            render_target: texture.render_target,
            sample_count: texture.sample_count,
            mip_level_count: texture.mip_level_count,
            view_dimension: self.dimension.clone(),
            base_mip_level: self.base_mip_level,
            mip_level_count_in_view: self.mip_level_count,
            base_array_layer: self.base_array_layer,
            array_layer_count: self.array_layer_count,
            uploads: texture.uploads.borrow().clone(),
        })
    }

    fn to_attachment_view(&self) -> Result<GpuCanvasAttachmentView> {
        if self.canvas.is_some() {
            Ok(GpuCanvasAttachmentView::Canvas)
        } else {
            self.to_binding(0, 0).map(GpuCanvasAttachmentView::Texture)
        }
    }

    fn sample_count(&self) -> u32 {
        self.texture
            .as_ref()
            .map_or(1, |texture| texture.sample_count)
    }
}

impl UserData for GpuTextureView {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("format", |_, this| Ok(this.format()));
    }
}

fn validate_attachment_canvas(
    view: &GpuTextureView,
    canvas_state: &Rc<RefCell<GpuCanvasState>>,
) -> Result<()> {
    if let Some(state) = &view.canvas {
        if !Rc::ptr_eq(state, canvas_state) {
            return Err(Error::runtime(
                "GPU render-pass canvas view belongs to another GPUCanvas",
            ));
        }
    } else if view
        .texture
        .as_ref()
        .is_none_or(|texture| !texture.render_target)
    {
        return Err(Error::runtime(
            "GPU render-pass external views require renderTarget=true",
        ));
    }
    Ok(())
}

fn record_attachment_sample_count(current: &mut Option<u32>, sample_count: u32) -> Result<()> {
    if current.is_some_and(|current| current != sample_count) {
        return Err(Error::runtime(
            "all GPU render-pass attachments must share one sampleCount",
        ));
    }
    *current = Some(sample_count);
    Ok(())
}

fn decode_clear_color(clear: Option<Table>) -> Result<[f64; 4]> {
    let Some(clear) = clear else {
        return Ok([0.0; 4]);
    };
    if clear.raw_len() != 4 {
        return Err(Error::runtime(
            "GPU clearColor must contain exactly four components",
        ));
    }
    Ok([clear.get(1)?, clear.get(2)?, clear.get(3)?, clear.get(4)?])
}

#[derive(Debug, Clone)]
struct GpuSampler {
    min_filter: String,
    mag_filter: String,
    mipmap_filter: String,
    address_mode_u: String,
    address_mode_v: String,
    address_mode_w: String,
    compare: Option<String>,
    lod_min_clamp: f32,
    lod_max_clamp: f32,
    max_anisotropy: u16,
}

impl GpuSampler {
    fn to_binding(&self, group: u32, binding: u32) -> GpuCanvasSamplerBinding {
        GpuCanvasSamplerBinding {
            group,
            binding,
            min_filter: self.min_filter.clone(),
            mag_filter: self.mag_filter.clone(),
            mipmap_filter: self.mipmap_filter.clone(),
            address_mode_u: self.address_mode_u.clone(),
            address_mode_v: self.address_mode_v.clone(),
            address_mode_w: self.address_mode_w.clone(),
            compare: self.compare.clone(),
            lod_min_clamp: self.lod_min_clamp,
            lod_max_clamp: self.lod_max_clamp,
            max_anisotropy: self.max_anisotropy,
        }
    }
}

impl UserData for GpuSampler {}

impl UserData for GpuTexture {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| Ok(this.width));
        fields.add_field_method_get("height", |_, this| Ok(this.height));
        fields.add_field_method_get("format", |_, this| Ok(this.format.clone()));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("view", |lua, this, descriptor: Option<Table>| {
            let default_dimension = match this.texture_type.as_str() {
                "cube" => "cube",
                "3d" => "3d",
                "2d-array" => "2d-array",
                _ => "2d",
            };
            let (dimension, base_mip_level, mip_level_count, base_array_layer, array_layer_count) =
                if let Some(descriptor) = descriptor {
                    reject_unknown_fields(
                        &descriptor,
                        &[
                            "dimension",
                            "baseMipLevel",
                            "mipCount",
                            "baseLayer",
                            "layerCount",
                        ],
                        "GPUTexture view",
                    )?;
                    (
                        descriptor
                            .get::<Option<String>>("dimension")?
                            .unwrap_or_else(|| default_dimension.into()),
                        descriptor.get::<Option<u32>>("baseMipLevel")?.unwrap_or(0),
                        descriptor
                            .get::<Option<u32>>("mipCount")?
                            .unwrap_or(this.mip_level_count),
                        descriptor.get::<Option<u32>>("baseLayer")?.unwrap_or(0),
                        descriptor
                            .get::<Option<u32>>("layerCount")?
                            .unwrap_or(this.depth_or_array_layers),
                    )
                } else {
                    (
                        default_dimension.into(),
                        0,
                        this.mip_level_count,
                        0,
                        this.depth_or_array_layers,
                    )
                };
            validate_texture_view(
                this,
                &dimension,
                base_mip_level,
                mip_level_count,
                base_array_layer,
                array_layer_count,
            )?;
            lua.create_userdata(GpuTextureView {
                texture: Some(this.clone()),
                canvas: None,
                dimension,
                base_mip_level,
                mip_level_count,
                base_array_layer,
                array_layer_count,
            })
        });
        methods.add_method_mut("upload", |_, this, descriptor: Table| {
            reject_unknown_fields(
                &descriptor,
                &[
                    "data",
                    "width",
                    "height",
                    "depth",
                    "x",
                    "y",
                    "z",
                    "mipLevel",
                    "layer",
                    "bytesPerRow",
                    "rowsPerImage",
                ],
                "GPUTexture upload",
            )?;
            if this.sample_count != 1 {
                return Err(Error::runtime(
                    "GPUTexture:upload cannot write a multisampled texture",
                ));
            }
            let data: LuaBuffer = descriptor.get("data")?;
            let mip_level = descriptor.get::<Option<u32>>("mipLevel")?.unwrap_or(0);
            if mip_level >= this.mip_level_count {
                return Err(Error::runtime(format!(
                    "upload mipLevel {mip_level} is outside {} levels",
                    this.mip_level_count
                )));
            }
            let mip_width = (this.width >> mip_level).max(1);
            let mip_height = (this.height >> mip_level).max(1);
            let width = descriptor.get::<Option<u32>>("width")?.unwrap_or(mip_width);
            let height = descriptor
                .get::<Option<u32>>("height")?
                .unwrap_or(mip_height);
            let depth = descriptor.get::<Option<u32>>("depth")?.unwrap_or(1);
            let x = descriptor.get::<Option<u32>>("x")?.unwrap_or(0);
            let y = descriptor.get::<Option<u32>>("y")?.unwrap_or(0);
            let z = descriptor.get::<Option<u32>>("z")?.unwrap_or(0);
            let array_layer = descriptor.get::<Option<u32>>("layer")?.unwrap_or(0);
            if x.checked_add(width).is_none_or(|end| end > mip_width)
                || y.checked_add(height).is_none_or(|end| end > mip_height)
                || array_layer >= this.depth_or_array_layers
            {
                return Err(Error::runtime("GPUTexture upload region is out of bounds"));
            }
            let bytes_per_texel =
                texture_format_bytes_per_texel(&this.format).ok_or_else(|| {
                    Error::runtime(
                        "GPUTexture upload requires bytesPerRow for a block-compressed format",
                    )
                })?;
            let bytes_per_row = descriptor
                .get::<Option<u32>>("bytesPerRow")?
                .unwrap_or_else(|| width.saturating_mul(bytes_per_texel));
            let rows_per_image = descriptor
                .get::<Option<u32>>("rowsPerImage")?
                .unwrap_or(height);
            let required_bytes = usize::try_from(bytes_per_row)
                .ok()
                .and_then(|bytes| bytes.checked_mul(rows_per_image as usize))
                .and_then(|bytes| bytes.checked_mul(depth.max(1) as usize))
                .ok_or_else(|| Error::runtime("GPUTexture upload byte length overflow"))?;
            if data.len() < required_bytes {
                return Err(Error::runtime(format!(
                    "GPUTexture upload has {} bytes but requires {required_bytes}",
                    data.len()
                )));
            }
            this.uploads.borrow_mut().push(GpuCanvasTextureUpload {
                bytes: data.to_vec(),
                width,
                height,
                depth,
                x,
                y,
                z,
                mip_level,
                array_layer,
                bytes_per_row,
                rows_per_image,
            });
            Ok(())
        });
    }
}

fn validate_texture_view(
    texture: &GpuTexture,
    dimension: &str,
    base_mip_level: u32,
    mip_level_count: u32,
    base_array_layer: u32,
    array_layer_count: u32,
) -> Result<()> {
    if !matches!(dimension, "2d" | "cube" | "3d" | "2d-array") {
        return Err(Error::runtime(format!(
            "invalid GPUTextureView dimension '{dimension}'"
        )));
    }
    if mip_level_count == 0
        || base_mip_level
            .checked_add(mip_level_count)
            .is_none_or(|end| end > texture.mip_level_count)
        || array_layer_count == 0
        || base_array_layer
            .checked_add(array_layer_count)
            .is_none_or(|end| end > texture.depth_or_array_layers)
    {
        return Err(Error::runtime("GPUTexture view range is out of bounds"));
    }
    if dimension == "cube" && array_layer_count != 6 {
        return Err(Error::runtime(
            "GPUTexture cube views require exactly six array layers",
        ));
    }
    Ok(())
}

fn texture_format_bytes_per_texel(format: &str) -> Option<u32> {
    match format {
        "r8unorm" => Some(1),
        "rg8unorm" | "r16float" => Some(2),
        "rgba8unorm" | "bgra8unorm" | "rg16float" | "rgb10a2unorm" | "rg11b10ufloat"
        | "depth32float" => Some(4),
        "rgba16float" | "rg32float" => Some(8),
        "rgba32float" => Some(16),
        _ => None,
    }
}

fn checked_gpu_buffer_write_range(
    source_len: usize,
    offset: usize,
    destination_len: usize,
) -> Result<(usize, usize)> {
    let end = offset
        .checked_add(source_len)
        .ok_or_else(|| Error::runtime("GPUBuffer:write byte range overflow"))?;
    if end > destination_len {
        return Err(Error::runtime(format!(
            "GPUBuffer:write range {offset}..{end} exceeds {destination_len} bytes"
        )));
    }
    Ok((offset, end))
}

#[derive(Clone)]
struct GpuShader {
    name: String,
    entries: Vec<GpuCanvasShaderEntry>,
    module: Option<Arc<dyn RenderGpuCanvasShader>>,
}

impl std::fmt::Debug for GpuShader {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GpuShader")
            .field("name", &self.name)
            .field("entries", &self.entries)
            .field("has_module", &self.module.is_some())
            .finish()
    }
}

impl UserData for GpuShader {}

#[derive(Debug, Clone)]
struct GpuPipeline {
    vertex_shader: GpuShader,
    fragment_shader: Option<GpuShader>,
    vertex_entry: GpuCanvasShaderEntrySelection,
    fragment_entry: Option<GpuCanvasShaderEntrySelection>,
    vertex_layouts: Vec<GpuCanvasVertexLayout>,
    state: GpuCanvasPipelineState,
    bind_group_layouts: Vec<GpuBindGroupLayout>,
    explicit_bind_group_layouts: bool,
}

impl UserData for GpuPipeline {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("getBindGroupLayout", |lua, this, group: u32| {
            if this.explicit_bind_group_layouts {
                return Err(Error::runtime(
                    "getBindGroupLayout: pipeline uses explicit bindGroupLayouts",
                ));
            }
            let layout = this
                .bind_group_layouts
                .iter()
                .find(|layout| layout.group == group)
                .cloned()
                .ok_or_else(|| {
                    Error::runtime(format!(
                        "getBindGroupLayout: group {group} not present in shader"
                    ))
                })?;
            lua.create_userdata(layout)
        });
    }
}

#[derive(Debug, Clone)]
struct GpuBindGroupLayout {
    group: u32,
    dynamic_uniform_bindings: Vec<u32>,
}

impl UserData for GpuBindGroupLayout {}

#[derive(Debug, Clone)]
struct GpuUniformBinding {
    binding: u32,
    buffer: GpuBuffer,
    offset: usize,
    size: usize,
}

#[derive(Debug, Clone)]
struct GpuTextureBinding {
    binding: u32,
    view: GpuTextureView,
}

#[derive(Debug, Clone)]
struct GpuSamplerResourceBinding {
    binding: u32,
    sampler: GpuSampler,
}

#[derive(Debug, Clone)]
struct GpuBindGroup {
    group: u32,
    uniforms: Vec<GpuUniformBinding>,
    textures: Vec<GpuTextureBinding>,
    samplers: Vec<GpuSamplerResourceBinding>,
    dynamic_uniform_bindings: Vec<u32>,
}

impl UserData for GpuBindGroup {}

#[derive(Debug)]
struct CompletedGpuCanvasPass {
    vertex_shader: Option<GpuShader>,
    fragment_shader: Option<GpuShader>,
    pipelines: Vec<CompletedGpuCanvasPipeline>,
    plan: GpuCanvasDrawPlan,
}

#[derive(Debug)]
struct CompletedGpuCanvasPipeline {
    vertex_shader: GpuShader,
    fragment_shader: Option<GpuShader>,
    plan: GpuCanvasPipelinePlan,
}

#[derive(Default)]
struct GpuCanvasState {
    width: u32,
    height: u32,
    unfinished_passes: usize,
    completed: Option<CompletedGpuCanvasPass>,
    image: Option<Box<dyn RenderImage>>,
}

impl std::fmt::Debug for GpuCanvasState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GpuCanvasState")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("unfinished_passes", &self.unfinished_passes)
            .field("completed", &self.completed)
            .field("has_image", &self.image.is_some())
            .finish()
    }
}

#[derive(Debug, Clone)]
struct GpuCanvas {
    state: Rc<RefCell<GpuCanvasState>>,
}

impl UserData for GpuCanvas {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("image", |lua, this| {
            if this.state.borrow().image.is_none() {
                return Ok(Value::Nil);
            }
            lua.create_userdata(GpuCanvasImage {
                state: Rc::clone(&this.state),
            })
            .map(Value::UserData)
        });
        fields.add_field_method_get("width", |_, this| Ok(this.state.borrow().width));
        fields.add_field_method_get("height", |_, this| Ok(this.state.borrow().height));
        fields.add_field_method_get("format", |_, _| Ok("rgba8unorm"));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("resize", |_, this, (width, height): (u32, u32)| {
            if width > MAX_GPU_CANVAS_DIMENSION || height > MAX_GPU_CANVAS_DIMENSION {
                return Err(Error::runtime(format!(
                    "GPUCanvas:resize dimensions must be at most {MAX_GPU_CANVAS_DIMENSION}"
                )));
            }
            let mut state = this.state.borrow_mut();
            state.width = width;
            state.height = height;
            if width == 0 || height == 0 {
                state.image = None;
            }
            Ok(())
        });
        methods.add_method("colorView", |lua, this, ()| {
            let state = this.state.borrow();
            if state.width == 0 || state.height == 0 {
                return Err(Error::runtime(
                    "GPUCanvas:colorView requires a non-zero canvas size",
                ));
            }
            drop(state);
            lua.create_userdata(GpuTextureView {
                texture: None,
                canvas: Some(Rc::clone(&this.state)),
                dimension: "2d".into(),
                base_mip_level: 0,
                mip_level_count: 1,
                base_array_layer: 0,
                array_layer_count: 1,
            })
        });
        methods.add_method("beginRenderPass", |lua, this, descriptor: Table| {
            reject_unknown_fields(
                &descriptor,
                &["color", "depthStencil", "label"],
                "GPU render-pass descriptor",
            )?;
            let mut sample_count = None;
            let mut color_attachments = Vec::new();
            if let Some(colors) = descriptor.get::<Option<Table>>("color")? {
                if colors.raw_len() > 4 {
                    return Err(Error::runtime(
                        "GPU render pass supports at most four color attachments",
                    ));
                }
                for color in colors.sequence_values::<Table>() {
                    let color = color?;
                    reject_unknown_fields(
                        &color,
                        &["view", "resolveTarget", "loadOp", "storeOp", "clearColor"],
                        "GPU color attachment",
                    )?;
                    let view = match color.get::<Option<AnyUserData>>("view")? {
                        Some(view) => view.borrow::<GpuTextureView>()?.clone(),
                        None => GpuTextureView {
                            texture: None,
                            canvas: Some(Rc::clone(&this.state)),
                            dimension: "2d".into(),
                            base_mip_level: 0,
                            mip_level_count: 1,
                            base_array_layer: 0,
                            array_layer_count: 1,
                        },
                    };
                    validate_attachment_canvas(&view, &this.state)?;
                    record_attachment_sample_count(&mut sample_count, view.sample_count())?;
                    let resolve_target = color
                        .get::<Option<AnyUserData>>("resolveTarget")?
                        .map(|target| {
                            let target = target.borrow::<GpuTextureView>()?;
                            validate_attachment_canvas(&target, &this.state)?;
                            if view.sample_count() == 1 {
                                return Err(Error::runtime(
                                    "GPU color resolveTarget requires a multisampled source view",
                                ));
                            }
                            if target.sample_count() != 1 {
                                return Err(Error::runtime(
                                    "GPU color resolveTarget must have sampleCount=1",
                                ));
                            }
                            if target.format() != view.format() {
                                return Err(Error::runtime(
                                    "GPU color resolveTarget format must match its source view",
                                ));
                            }
                            target.to_attachment_view()
                        })
                        .transpose()?;
                    let load_op = optional_enum(&color, "loadOp", "clear", &["clear", "load"])?;
                    let store_op = optional_enum(&color, "storeOp", "", &["store", "discard"])?;
                    let clear_color =
                        decode_clear_color(color.get::<Option<Table>>("clearColor")?)?;
                    color_attachments.push(GpuCanvasColorAttachment {
                        view: view.to_attachment_view()?,
                        resolve_target,
                        load_op,
                        store_op,
                        clear_color,
                    });
                }
            }
            let depth_stencil_attachment = descriptor
                .get::<Option<Table>>("depthStencil")?
                .map(|depth| {
                    reject_unknown_fields(
                        &depth,
                        &["view", "depthLoadOp", "depthStoreOp", "depthClearValue"],
                        "GPU depth/stencil attachment",
                    )?;
                    let view: AnyUserData = depth.get("view")?;
                    let view = view.borrow::<GpuTextureView>()?;
                    validate_attachment_canvas(&view, &this.state)?;
                    if !view.format().starts_with("depth") {
                        return Err(Error::runtime(
                            "GPU depth/stencil attachment requires a depth texture view",
                        ));
                    }
                    record_attachment_sample_count(&mut sample_count, view.sample_count())?;
                    Ok(GpuCanvasDepthStencilAttachment {
                        view: view.to_attachment_view()?,
                        depth_load_op: optional_enum(
                            &depth,
                            "depthLoadOp",
                            "clear",
                            &["clear", "load"],
                        )?,
                        depth_store_op: optional_enum(
                            &depth,
                            "depthStoreOp",
                            "",
                            &["store", "discard"],
                        )?,
                        depth_clear_value: depth
                            .get::<Option<f32>>("depthClearValue")?
                            .unwrap_or(1.0),
                    })
                })
                .transpose()?;
            if color_attachments.is_empty() && depth_stencil_attachment.is_none() {
                return Err(Error::runtime(
                    "GPU render pass requires at least one color or depth/stencil attachment",
                ));
            }
            {
                let mut state = this.state.borrow_mut();
                state.unfinished_passes = state
                    .unfinished_passes
                    .checked_add(1)
                    .ok_or_else(|| Error::runtime("GPU render-pass count overflow"))?;
            }
            lua.create_userdata(GpuRenderPass {
                state: Rc::clone(&this.state),
                color_attachments,
                depth_stencil_attachment,
                sample_count: sample_count.unwrap_or(1),
                pipeline: None,
                bind_groups: BTreeMap::new(),
                dynamic_offsets: BTreeMap::new(),
                vertex_buffers: BTreeMap::new(),
                index_buffer: None,
                draws: Vec::new(),
                pass_state: GpuCanvasPassState::default(),
                finished: false,
            })
        });
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GpuCanvasImage {
    state: Rc<RefCell<GpuCanvasState>>,
}

impl UserData for GpuCanvasImage {}

#[derive(Debug)]
pub(crate) struct RegisteredGpuCanvasShaderAsset {
    asset: RegisteredGpuCanvasShaderAssetState,
    decoded: Option<GpuCanvasShader>,
}

#[derive(Debug)]
enum RegisteredGpuCanvasShaderAssetState {
    Valid(ShaderAsset),
    Invalid(String),
}

impl RegisteredGpuCanvasShaderAsset {
    pub(crate) fn new(name: &str, payload: &[u8]) -> Self {
        let asset = match ShaderAsset::decode(name, payload) {
            Ok(asset) => RegisteredGpuCanvasShaderAssetState::Valid(asset),
            Err(error) => RegisteredGpuCanvasShaderAssetState::Invalid(format!(
                "ShaderAsset '{name}' neutral decode failed: {error}"
            )),
        };
        Self {
            asset,
            decoded: None,
        }
    }

    fn resolve(&mut self, name: &str) -> Result<&GpuCanvasShader> {
        if self.decoded.is_none() {
            let asset = match &self.asset {
                RegisteredGpuCanvasShaderAssetState::Valid(asset) => asset,
                RegisteredGpuCanvasShaderAssetState::Invalid(error) => {
                    return Err(Error::runtime(error.clone()));
                }
            };
            self.decoded = Some(asset.decode_webgpu(name)?);
        }
        self.decoded.as_ref().ok_or_else(|| {
            Error::runtime(format!(
                "GPU-canvas shader '{name}' resolved without a decoded shader"
            ))
        })
    }
}

pub(crate) type ImportedGpuCanvasShaderAssetOwner = Rc<RefCell<RegisteredGpuCanvasShaderAsset>>;

#[derive(Debug, Clone)]
pub(crate) struct ImportedGpuCanvasShaderAssetEntry {
    pub(crate) name: String,
    pub(crate) short_name: String,
    pub(crate) owner: ImportedGpuCanvasShaderAssetOwner,
}

pub(crate) type ImportedGpuCanvasShaderAssets = Rc<RefCell<Vec<ImportedGpuCanvasShaderAssetEntry>>>;

#[derive(Debug, Clone)]
enum GpuCanvasShaderCatalog {
    Direct(Rc<BTreeMap<String, Vec<GpuCanvasShaderEntry>>>),
    Imported(ImportedGpuCanvasShaderAssets),
}

impl GpuCanvasShaderCatalog {
    fn lookup(
        &self,
        lua: &luaur_rt::Lua,
        name: &str,
    ) -> Option<(String, Vec<GpuCanvasShaderEntry>, Option<GpuCanvasShader>)> {
        match self {
            Self::Direct(shaders) => {
                let entries = shaders.get(name)?.clone();
                if entries.is_empty() {
                    return None;
                }
                Some((name.to_owned(), entries, None))
            }
            Self::Imported(shaders) => {
                let reference = crate::vm::lua_blob::ScopedAssetReference::new(lua, name);
                let mut best_rank = 0;
                let mut selected = None;
                for entry in shaders.borrow().iter() {
                    let rank = reference.rank(&entry.name, &entry.short_name);
                    if rank > best_rank {
                        best_rank = rank;
                        selected = Some((entry.name.clone(), Rc::clone(&entry.owner)));
                    }
                }
                let (registered_name, owner) = selected?;
                let shader = owner.borrow_mut().resolve(&registered_name).ok()?.clone();
                if shader.entries.is_empty() {
                    return None;
                }
                Some((name.to_owned(), shader.entries.clone(), Some(shader)))
            }
        }
    }

    fn shader(
        &self,
        lua: &luaur_rt::Lua,
        name: &str,
        renderer_bindings: Option<&RendererBindings>,
    ) -> Option<GpuShader> {
        match self {
            Self::Direct(shaders) => {
                let entries = shaders.get(name)?.clone();
                if entries.is_empty() {
                    return None;
                }
                Some(GpuShader {
                    name: name.to_owned(),
                    entries,
                    module: None,
                })
            }
            Self::Imported(shaders) => {
                let reference = crate::vm::lua_blob::ScopedAssetReference::new(lua, name);
                let mut best_rank = 0;
                let mut selected = None;
                for entry in shaders.borrow().iter() {
                    let rank = reference.rank(&entry.name, &entry.short_name);
                    if rank > best_rank {
                        best_rank = rank;
                        selected = Some((entry.name.clone(), Rc::clone(&entry.owner)));
                    }
                }
                let (registered_name, owner) = selected?;
                let shader = owner.borrow_mut().resolve(&registered_name).ok()?.clone();
                if shader.entries.is_empty() {
                    return None;
                }
                let module = renderer_bindings?
                    .with_factory(|factory| {
                        factory
                            .make_gpu_canvas_shader(&shader)
                            .map_err(|error| Error::runtime(error.to_string()))
                    })
                    .ok()?;
                Some(GpuShader {
                    name: name.to_owned(),
                    entries: shader.entries,
                    module: Some(module),
                })
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct GpuCanvasContextBindings {
    canvases: Rc<RefCell<Vec<Rc<RefCell<GpuCanvasState>>>>>,
    shaders: GpuCanvasShaderCatalog,
    renderer_bindings: Option<RendererBindings>,
}

impl GpuCanvasContextBindings {
    pub(crate) fn canvas_userdata(&self, lua: &luaur_rt::Lua) -> Result<AnyUserData> {
        self.canvas_userdata_with_size(lua, 0, 0)
    }

    pub(crate) fn canvas_userdata_with_size(
        &self,
        lua: &luaur_rt::Lua,
        width: u32,
        height: u32,
    ) -> Result<AnyUserData> {
        if width > MAX_GPU_CANVAS_DIMENSION || height > MAX_GPU_CANVAS_DIMENSION {
            return Err(Error::runtime(format!(
                "GPUCanvas:resize dimensions must be at most {MAX_GPU_CANVAS_DIMENSION}"
            )));
        }
        let state = Rc::new(RefCell::new(GpuCanvasState {
            width,
            height,
            ..GpuCanvasState::default()
        }));
        self.canvases.borrow_mut().push(Rc::clone(&state));
        lua.create_userdata(GpuCanvas { state })
    }

    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        Self {
            canvases: Rc::new(RefCell::new(Vec::new())),
            shaders: GpuCanvasShaderCatalog::Direct(Rc::new(BTreeMap::new())),
            renderer_bindings: None,
        }
    }

    pub(crate) fn shader_userdata(&self, lua: &luaur_rt::Lua, name: String) -> Result<MultiValue> {
        let Some(shader) = self
            .shaders
            .shader(lua, &name, self.renderer_bindings.as_ref())
        else {
            return Ok(MultiValue::new());
        };
        lua.create_userdata(shader)
            .map(|shader| MultiValue::from_vec(vec![Value::UserData(shader)]))
    }

    pub(crate) async fn shader_userdata_async(
        &self,
        lua: luaur_rt::Lua,
        name: String,
    ) -> Result<MultiValue> {
        let Some((name, entries, imported)) = self.shaders.lookup(&lua, &name) else {
            return Ok(MultiValue::new());
        };
        let module = match imported {
            None => None,
            Some(shader) => {
                let Some(bindings) = self.renderer_bindings.as_ref() else {
                    return Ok(MultiValue::new());
                };
                let load =
                    bindings.with_factory(|factory| Ok(factory.load_gpu_canvas_shader(&shader)))?;
                match load.resolve().await {
                    Ok(module) => Some(module),
                    // Pinned C++ `lua_gpu_load_shader_by_name` returns false
                    // when physical module construction fails; context:shader
                    // then pops its temporary value and returns zero Lua values
                    // (`lua_gpu.cpp:519-656`; `lua_scripted_context.cpp:531-558`).
                    Err(_) => return Ok(MultiValue::new()),
                }
            }
        };
        lua.create_userdata(GpuShader {
            name,
            entries,
            module,
        })
        .map(|shader| MultiValue::from_vec(vec![Value::UserData(shader)]))
    }

    pub(crate) fn async_shader_function(&self, lua: &luaur_rt::Lua) -> Result<Function> {
        let bindings = self.clone();
        lua.create_async_function(move |lua, (_context, name): (Value, String)| {
            let bindings = bindings.clone();
            async move { bindings.shader_userdata_async(lua, name).await }
        })
    }
}

impl UserData for GpuCanvasContextBindings {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("gpuCanvas", |lua, this, ()| this.canvas_userdata(lua));
        methods.add_method("shader", |lua, this, name: String| {
            this.shader_userdata(lua, name)
        });
    }
}

/// Per-script retained GPU-canvas occurrences. The Lua context creates each
/// state independently and image userdata retains its occurrence, while
/// canonical shader bytes remain VM-owned.
pub(crate) struct ImportedGpuCanvasInstance {
    canvases: Rc<RefCell<Vec<Rc<RefCell<GpuCanvasState>>>>>,
    renderer_bindings: RendererBindings,
}

impl ImportedGpuCanvasInstance {
    pub(crate) fn new(
        shaders: ImportedGpuCanvasShaderAssets,
        renderer_bindings: RendererBindings,
    ) -> (Self, GpuCanvasContextBindings) {
        let canvases = Rc::new(RefCell::new(Vec::new()));
        let bindings = GpuCanvasContextBindings {
            canvases: Rc::clone(&canvases),
            shaders: GpuCanvasShaderCatalog::Imported(Rc::clone(&shaders)),
            renderer_bindings: Some(renderer_bindings.clone()),
        };
        (
            Self {
                canvases,
                renderer_bindings,
            },
            bindings,
        )
    }

    pub(crate) fn execute_draw_canvas(
        &self,
        table: &Table,
        factory: &mut dyn RenderFactory,
    ) -> Result<()> {
        let value: Value = table.get("drawCanvas")?;
        let Value::Function(function) = value else {
            return Ok(());
        };
        let canvases = self.canvases.borrow().clone();
        for canvas in &canvases {
            let mut state = canvas.borrow_mut();
            state.completed = None;
            state.image = None;
            state.unfinished_passes = 0;
        }
        self.renderer_bindings.verify_render_context(factory)?;
        function.call::<()>((table.clone(),))?;
        let mut completed_count = 0;
        for canvas in canvases {
            if canvas.borrow().unfinished_passes != 0 {
                canvas.borrow_mut().completed = None;
                return Err(Error::runtime(
                    "GPU render pass left open at script return; call finish() on every pass",
                ));
            }
            let Some(completed) = canvas.borrow_mut().completed.take() else {
                continue;
            };
            completed_count += 1;
            let pipelines = completed
                .pipelines
                .iter()
                .map(|pipeline| {
                    let vertex = pipeline.vertex_shader.module.clone().ok_or_else(|| {
                        Error::runtime("GPU-canvas vertex shader has no backend module occurrence")
                    })?;
                    let fragment = pipeline
                        .fragment_shader
                        .as_ref()
                        .map(|shader| {
                            shader.module.clone().ok_or_else(|| {
                                Error::runtime(
                                    "GPU-canvas fragment shader has no backend module occurrence",
                                )
                            })
                        })
                        .transpose()?;
                    Ok(GpuCanvasPipelineShaders { vertex, fragment })
                })
                .collect::<Result<Vec<_>>>()?;
            let image = factory
                .make_gpu_canvas_image_with_pipelines(&pipelines, &completed.plan)
                .map_err(|error| Error::runtime(format!("GPU-canvas render failed: {error}")))?;
            canvas.borrow_mut().image = Some(image);
        }
        if completed_count == 0 {
            return Err(Error::runtime(
                "gpu-canvas drawCanvas did not finish a render pass",
            ));
        }
        Ok(())
    }
}

pub(crate) fn with_gpu_canvas_image<R>(
    image: &GpuCanvasImage,
    callback: impl FnOnce(&dyn RenderImage) -> R,
) -> Result<R> {
    let state = image.state.borrow();
    let image = state
        .image
        .as_deref()
        .ok_or_else(|| Error::runtime("GPU-canvas image is unavailable before drawCanvas"))?;
    Ok(callback(image))
}

#[derive(Debug, Clone, Copy)]
enum GpuDrawCall {
    NonIndexed {
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    },
    Indexed {
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    },
}

#[derive(Debug)]
struct GpuRenderPass {
    state: Rc<RefCell<GpuCanvasState>>,
    color_attachments: Vec<GpuCanvasColorAttachment>,
    depth_stencil_attachment: Option<GpuCanvasDepthStencilAttachment>,
    sample_count: u32,
    pipeline: Option<GpuPipeline>,
    bind_groups: BTreeMap<u32, GpuBindGroup>,
    dynamic_offsets: BTreeMap<u32, Vec<u32>>,
    vertex_buffers: BTreeMap<u32, GpuBuffer>,
    index_buffer: Option<(GpuBuffer, String)>,
    draws: Vec<(GpuDrawCall, GpuCanvasPassState)>,
    pass_state: GpuCanvasPassState,
    finished: bool,
}

impl UserData for GpuRenderPass {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("setPipeline", |_, this, pipeline: AnyUserData| {
            this.ensure_open()?;
            if !this.draws.is_empty() {
                return Err(Error::runtime(
                    "GPU render pass supports exactly one draw call",
                ));
            }
            if this.pipeline.is_some() {
                return Err(Error::runtime(
                    "GPU render pass pipeline has already been set",
                ));
            }
            this.pipeline = Some(pipeline.borrow::<GpuPipeline>()?.clone());
            Ok(())
        });
        methods.add_method_mut(
            "setBindGroup",
            |_, this, (index, bind_group, dynamic_offsets): (u32, AnyUserData, Option<Table>)| {
                this.ensure_open()?;
                if !this.draws.is_empty() {
                    return Err(Error::runtime(
                        "GPU render pass supports exactly one draw call",
                    ));
                }
                if index >= MAX_GPU_CANVAS_BIND_GROUPS {
                    return Err(Error::runtime(format!(
                        "setBindGroup index must be less than {MAX_GPU_CANVAS_BIND_GROUPS}"
                    )));
                }
                if this.bind_groups.contains_key(&index) {
                    return Err(Error::runtime(format!(
                        "setBindGroup index {index} is already bound"
                    )));
                }
                let bind_group = bind_group.borrow::<GpuBindGroup>()?.clone();
                if index != bind_group.group {
                    return Err(Error::runtime(format!(
                        "setBindGroup index {index} does not match layout group {}",
                        bind_group.group
                    )));
                }
                let mut decoded_offsets = Vec::new();
                if let Some(dynamic_offsets) = dynamic_offsets {
                    for offset in dynamic_offsets.sequence_values::<u32>() {
                        let offset = offset?;
                        if offset % 256 != 0 {
                            return Err(Error::runtime(
                                "setBindGroup dynamic offsets must be 256-byte aligned",
                            ));
                        }
                        decoded_offsets.push(offset);
                    }
                }
                if decoded_offsets.len() != bind_group.dynamic_uniform_bindings.len() {
                    return Err(Error::runtime(format!(
                        "setBindGroup received {} dynamic offsets for {} dynamic UBOs",
                        decoded_offsets.len(),
                        bind_group.dynamic_uniform_bindings.len()
                    )));
                }
                this.dynamic_offsets.insert(index, decoded_offsets);
                this.bind_groups.insert(index, bind_group);
                Ok(())
            },
        );
        methods.add_method_mut(
            "setIndexBuffer",
            |_, this, (buffer, format): (AnyUserData, Option<String>)| {
                this.ensure_open()?;
                let buffer = buffer.borrow::<GpuBuffer>()?.clone();
                if buffer.usage != GpuBufferUsage::Index {
                    return Err(Error::runtime("setIndexBuffer requires an index GPUBuffer"));
                }
                let format = format.unwrap_or_else(|| "uint16".into());
                if !matches!(format.as_str(), "uint16" | "uint32") {
                    return Err(Error::runtime(format!(
                        "unsupported GPU index format '{format}'"
                    )));
                }
                this.index_buffer = Some((buffer, format));
                Ok(())
            },
        );
        methods.add_method_mut(
            "setViewport",
            |_, this, (x, y, width, height): (f32, f32, f32, f32)| {
                this.ensure_open()?;
                if ![x, y, width, height].iter().all(|value| value.is_finite())
                    || width <= 0.0
                    || height <= 0.0
                {
                    return Err(Error::runtime(
                        "setViewport requires finite coordinates and positive dimensions",
                    ));
                }
                this.pass_state.viewport = Some([x, y, width, height]);
                Ok(())
            },
        );
        methods.add_method_mut("setScissorRect", |_, this, rect: (u32, u32, u32, u32)| {
            this.ensure_open()?;
            let (x, y, width, height) = rect;
            if width == 0 || height == 0 {
                return Err(Error::runtime(
                    "setScissorRect requires positive dimensions",
                ));
            }
            this.pass_state.scissor_rect = Some([x, y, width, height]);
            Ok(())
        });
        methods.add_method_mut("setStencilReference", |_, this, reference: u32| {
            this.ensure_open()?;
            this.pass_state.stencil_reference = reference;
            Ok(())
        });
        methods.add_method_mut("setBlendColor", |_, this, color: (f64, f64, f64, f64)| {
            this.ensure_open()?;
            let color = [color.0, color.1, color.2, color.3];
            if color.iter().any(|value| !value.is_finite()) {
                return Err(Error::runtime("setBlendColor requires finite components"));
            }
            this.pass_state.blend_color = color;
            Ok(())
        });
        methods.add_method_mut(
            "setVertexBuffer",
            |_, this, (slot, buffer): (u32, AnyUserData)| {
                this.ensure_open()?;
                if !this.draws.is_empty() {
                    return Err(Error::runtime(
                        "GPU render pass supports exactly one draw call",
                    ));
                }
                if usize::try_from(slot).unwrap_or(usize::MAX) >= MAX_GPU_CANVAS_VERTEX_BUFFERS {
                    return Err(Error::runtime(format!(
                        "setVertexBuffer slot must be less than {MAX_GPU_CANVAS_VERTEX_BUFFERS}"
                    )));
                }
                if this.vertex_buffers.contains_key(&slot) {
                    return Err(Error::runtime(format!(
                        "setVertexBuffer slot {slot} is already bound"
                    )));
                }
                let buffer = buffer.borrow::<GpuBuffer>()?.clone();
                if buffer.usage != GpuBufferUsage::Vertex {
                    return Err(Error::runtime(
                        "setVertexBuffer requires a vertex GPUBuffer",
                    ));
                }
                this.vertex_buffers.insert(slot, buffer);
                Ok(())
            },
        );
        methods.add_method_mut(
            "draw",
            |_,
             this,
             (vertex_count, instance_count, first_vertex, first_instance): (
                u32,
                Option<u32>,
                Option<u32>,
                Option<u32>,
            )| {
                this.ensure_open()?;
                if this.pipeline.is_none() {
                    return Err(Error::runtime(
                        "GPU render pass must set a pipeline before draw",
                    ));
                }
                let instance_count = instance_count.unwrap_or(1);
                let first_vertex = first_vertex.unwrap_or(0);
                let first_instance = first_instance.unwrap_or(0);
                let vertex_end = first_vertex
                    .checked_add(vertex_count)
                    .ok_or_else(|| Error::runtime("GPU draw vertex range overflow"))?;
                let instance_end = first_instance
                    .checked_add(instance_count)
                    .ok_or_else(|| Error::runtime("GPU draw instance range overflow"))?;
                let invocation_count = u64::from(vertex_count)
                    .checked_mul(u64::from(instance_count))
                    .ok_or_else(|| Error::runtime("GPU draw invocation count overflow"))?;
                if vertex_count == 0 || instance_count == 0 {
                    return Err(Error::runtime(
                        "GPU render pass vertex and instance counts must be positive",
                    ));
                }
                if invocation_count > MAX_GPU_CANVAS_DRAW_INVOCATIONS
                    || u64::from(vertex_end) > MAX_GPU_CANVAS_DRAW_INVOCATIONS
                    || u64::from(instance_end) > MAX_GPU_CANVAS_DRAW_INVOCATIONS
                {
                    return Err(Error::runtime(format!(
                        "GPU render pass draw ranges may cover at most {MAX_GPU_CANVAS_DRAW_INVOCATIONS} invocations"
                    )));
                }
                this.draws.push((
                    GpuDrawCall::NonIndexed {
                        vertex_count,
                        instance_count,
                        first_vertex,
                        first_instance,
                    },
                    this.pass_state.clone(),
                ));
                Ok(())
            },
        );
        methods.add_method_mut(
            "drawIndexed",
            |_,
             this,
             (index_count, instance_count, first_index, base_vertex, first_instance): (
                u32,
                Option<u32>,
                Option<u32>,
                Option<i32>,
                Option<u32>,
            )| {
                this.ensure_open()?;
                if this.pipeline.is_none() {
                    return Err(Error::runtime(
                        "GPU render pass must set a pipeline before drawIndexed",
                    ));
                }
                if this.index_buffer.is_none() {
                    return Err(Error::runtime(
                        "GPU render pass must set an index buffer before drawIndexed",
                    ));
                }
                let instance_count = instance_count.unwrap_or(1);
                let first_index = first_index.unwrap_or(0);
                let base_vertex = base_vertex.unwrap_or(0);
                let first_instance = first_instance.unwrap_or(0);
                let index_end = first_index
                    .checked_add(index_count)
                    .ok_or_else(|| Error::runtime("GPU indexed draw range overflow"))?;
                let instance_end = first_instance
                    .checked_add(instance_count)
                    .ok_or_else(|| Error::runtime("GPU indexed instance range overflow"))?;
                let invocation_count = u64::from(index_count)
                    .checked_mul(u64::from(instance_count))
                    .ok_or_else(|| Error::runtime("GPU indexed invocation count overflow"))?;
                if index_count == 0 || instance_count == 0 {
                    return Err(Error::runtime(
                        "GPU indexed draw counts must be positive",
                    ));
                }
                if invocation_count > MAX_GPU_CANVAS_DRAW_INVOCATIONS
                    || u64::from(index_end) > MAX_GPU_CANVAS_DRAW_INVOCATIONS
                    || u64::from(instance_end) > MAX_GPU_CANVAS_DRAW_INVOCATIONS
                {
                    return Err(Error::runtime(format!(
                        "GPU render pass draw ranges may cover at most {MAX_GPU_CANVAS_DRAW_INVOCATIONS} invocations"
                    )));
                }
                this.draws.push((
                    GpuDrawCall::Indexed {
                        index_count,
                        instance_count,
                        first_index,
                        base_vertex,
                        first_instance,
                    },
                    this.pass_state.clone(),
                ));
                Ok(())
            },
        );
        methods.add_method_mut("finish", |_, this, ()| {
            this.ensure_open()?;
            if this.draws.is_empty() {
                let mut state = this.state.borrow_mut();
                if state.width == 0 || state.height == 0 {
                    return Err(Error::runtime(
                        "GPU canvas must be resized before its first render pass",
                    ));
                }
                let render_pass = GpuCanvasRenderPass {
                    color_attachments: this.color_attachments.clone(),
                    depth_stencil_attachment: this.depth_stencil_attachment.clone(),
                    draws: Vec::new(),
                };
                if let Some(completed) = state.completed.as_mut() {
                    completed.plan.render_passes.push(render_pass);
                } else {
                    state.completed = Some(CompletedGpuCanvasPass {
                        vertex_shader: None,
                        fragment_shader: None,
                        pipelines: Vec::new(),
                        plan: GpuCanvasDrawPlan {
                            vertex_entry: None,
                            fragment_entry: None,
                            width: state.width,
                            height: state.height,
                            clear_color: this
                                .color_attachments
                                .first()
                                .map_or([0.0; 4], |attachment| attachment.clear_color),
                            vertex_count: 0,
                            instance_count: 0,
                            first_vertex: 0,
                            first_instance: 0,
                            uniform_buffers: Vec::new(),
                            vertex_layouts: Vec::new(),
                            vertex_buffers: Vec::new(),
                            index_buffer: None,
                            indexed_draw: None,
                            texture_bindings: Vec::new(),
                            sampler_bindings: Vec::new(),
                            pipeline_state: GpuCanvasPipelineState::default(),
                            pass_state: GpuCanvasPassState::default(),
                            pipelines: Vec::new(),
                            render_passes: vec![render_pass],
                        },
                    });
                }
                state.unfinished_passes = state.unfinished_passes.saturating_sub(1);
                this.finished = true;
                return Ok(());
            }
            let pipeline = this.pipeline.as_ref().ok_or_else(|| {
                Error::runtime("GPU render pass must set a pipeline before finish")
            })?;
            validate_pipeline_attachments(
                pipeline,
                &this.color_attachments,
                this.depth_stencil_attachment.as_ref(),
                this.sample_count,
            )?;
            if pipeline.vertex_layouts.len() != this.vertex_buffers.len() {
                return Err(Error::runtime(format!(
                    "GPU render pass has {} vertex layouts but {} bound vertex buffers",
                    pipeline.vertex_layouts.len(),
                    this.vertex_buffers.len()
                )));
            }
            for (slot, layout) in pipeline.vertex_layouts.iter().enumerate() {
                let slot = u32::try_from(slot)
                    .map_err(|_| Error::runtime("GPU vertex slot conversion overflow"))?;
                let buffer = this.vertex_buffers.get(&slot).ok_or_else(|| {
                    Error::runtime(format!("GPU vertex buffer slot {slot} is not bound"))
                })?;
                for (draw, _) in &this.draws {
                    let GpuDrawCall::NonIndexed {
                        vertex_count,
                        first_vertex,
                        ..
                    } = draw
                    else {
                        continue;
                    };
                    let vertex_end = u64::from(*first_vertex)
                        .checked_add(u64::from(*vertex_count))
                        .ok_or_else(|| Error::runtime("GPU vertex byte range overflow"))?;
                    let required_bytes = vertex_end
                        .checked_mul(layout.stride)
                        .ok_or_else(|| Error::runtime("GPU vertex byte range overflow"))?;
                    if required_bytes > buffer.bytes.borrow().len() as u64 {
                        return Err(Error::runtime(format!(
                            "GPU vertex buffer slot {slot} requires {required_bytes} bytes"
                        )));
                    }
                }
            }
            let mut state = this.state.borrow_mut();
            if state.width == 0 || state.height == 0 {
                return Err(Error::runtime(
                    "GPU canvas must be resized before its first render pass",
                ));
            }
            let mut uniform_buffers = Vec::new();
            let mut texture_bindings = Vec::new();
            let mut sampler_bindings = Vec::new();
            for bind_group in this.bind_groups.values() {
                let dynamic_offsets = this
                    .dynamic_offsets
                    .get(&bind_group.group)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                for uniform in &bind_group.uniforms {
                    let dynamic_offset = bind_group
                        .dynamic_uniform_bindings
                        .iter()
                        .position(|binding| *binding == uniform.binding)
                        .and_then(|index| dynamic_offsets.get(index))
                        .copied()
                        .unwrap_or(0) as usize;
                    let start = uniform
                        .offset
                        .checked_add(dynamic_offset)
                        .ok_or_else(|| Error::runtime("GPU uniform offset overflow"))?;
                    let end = start
                        .checked_add(uniform.size)
                        .ok_or_else(|| Error::runtime("GPU uniform range overflow"))?;
                    let bytes = uniform.buffer.bytes.borrow();
                    if end > bytes.len() {
                        return Err(Error::runtime(format!(
                            "GPU uniform binding {} range {start}..{end} exceeds {} bytes",
                            uniform.binding,
                            bytes.len()
                        )));
                    }
                    uniform_buffers.push(GpuCanvasUniformBuffer {
                        group: bind_group.group,
                        binding: uniform.binding,
                        bytes: bytes[start..end].to_vec(),
                    });
                }
                for texture in &bind_group.textures {
                    texture_bindings
                        .push(texture.view.to_binding(bind_group.group, texture.binding)?);
                }
                for sampler in &bind_group.samplers {
                    sampler_bindings.push(
                        sampler
                            .sampler
                            .to_binding(bind_group.group, sampler.binding),
                    );
                }
            }
            let vertex_buffers = this
                .vertex_buffers
                .iter()
                .map(|(slot, buffer)| GpuCanvasVertexBuffer {
                    slot: *slot,
                    bytes: buffer.bytes.borrow().clone(),
                })
                .collect::<Vec<_>>();
            let draws = this
                .draws
                .iter()
                .map(|(draw, pass_state)| match *draw {
                    GpuDrawCall::NonIndexed {
                        vertex_count,
                        instance_count,
                        first_vertex,
                        first_instance,
                    } => GpuCanvasDrawCommand {
                        pipeline_index: 0,
                        vertex_count,
                        instance_count,
                        first_vertex,
                        first_instance,
                        indexed_draw: None,
                        pass_state: pass_state.clone(),
                    },
                    GpuDrawCall::Indexed {
                        index_count,
                        instance_count,
                        first_index,
                        base_vertex,
                        first_instance,
                    } => GpuCanvasDrawCommand {
                        pipeline_index: 0,
                        vertex_count: 0,
                        instance_count,
                        first_vertex: 0,
                        first_instance,
                        indexed_draw: Some(GpuCanvasIndexedDraw {
                            index_count,
                            instance_count,
                            first_index,
                            base_vertex,
                            first_instance,
                        }),
                        pass_state: pass_state.clone(),
                    },
                })
                .collect::<Vec<_>>();
            let first_draw = draws
                .first()
                .cloned()
                .ok_or_else(|| Error::runtime("GPU render pass must issue a draw before finish"))?;
            let index_buffer =
                this.index_buffer
                    .as_ref()
                    .map(|(buffer, format)| GpuCanvasIndexBuffer {
                        bytes: buffer.bytes.borrow().clone(),
                        format: format.clone(),
                    });
            let render_pass = GpuCanvasRenderPass {
                color_attachments: this.color_attachments.clone(),
                depth_stencil_attachment: this.depth_stencil_attachment.clone(),
                draws,
            };
            let clear_color = this
                .color_attachments
                .first()
                .map_or([0.0; 4], |attachment| attachment.clear_color);
            let pipeline_plan = GpuCanvasPipelinePlan {
                vertex_entry: Some(pipeline.vertex_entry.clone()),
                fragment_entry: pipeline.fragment_entry.clone(),
                uniform_buffers: uniform_buffers.clone(),
                vertex_layouts: pipeline.vertex_layouts.clone(),
                vertex_buffers: vertex_buffers.clone(),
                index_buffer: index_buffer.clone(),
                texture_bindings: texture_bindings.clone(),
                sampler_bindings: sampler_bindings.clone(),
                pipeline_state: pipeline.state.clone(),
            };
            let plan = GpuCanvasDrawPlan {
                vertex_entry: Some(pipeline.vertex_entry.clone()),
                fragment_entry: pipeline.fragment_entry.clone(),
                width: state.width,
                height: state.height,
                clear_color,
                vertex_count: first_draw.vertex_count,
                instance_count: first_draw.instance_count,
                first_vertex: first_draw.first_vertex,
                first_instance: first_draw.first_instance,
                uniform_buffers,
                vertex_layouts: pipeline.vertex_layouts.clone(),
                vertex_buffers,
                index_buffer,
                indexed_draw: first_draw.indexed_draw.clone(),
                texture_bindings,
                sampler_bindings,
                pipeline_state: pipeline.state.clone(),
                pass_state: first_draw.pass_state.clone(),
                pipelines: vec![pipeline_plan.clone()],
                render_passes: vec![render_pass],
            };
            if let Some(completed) = state.completed.as_mut() {
                if completed.pipelines.is_empty() {
                    completed.vertex_shader = Some(pipeline.vertex_shader.clone());
                    completed.fragment_shader = pipeline.fragment_shader.clone();
                    completed.plan.vertex_entry = plan.vertex_entry.clone();
                    completed.plan.fragment_entry = plan.fragment_entry.clone();
                    completed.plan.clear_color = plan.clear_color;
                    completed.plan.vertex_count = plan.vertex_count;
                    completed.plan.instance_count = plan.instance_count;
                    completed.plan.first_vertex = plan.first_vertex;
                    completed.plan.first_instance = plan.first_instance;
                    completed.plan.uniform_buffers = plan.uniform_buffers.clone();
                    completed.plan.vertex_layouts = plan.vertex_layouts.clone();
                    completed.plan.vertex_buffers = plan.vertex_buffers.clone();
                    completed.plan.index_buffer = plan.index_buffer.clone();
                    completed.plan.indexed_draw = plan.indexed_draw.clone();
                    completed.plan.texture_bindings = plan.texture_bindings.clone();
                    completed.plan.sampler_bindings = plan.sampler_bindings.clone();
                    completed.plan.pipeline_state = plan.pipeline_state.clone();
                    completed.plan.pass_state = plan.pass_state.clone();
                }
                let pipeline_index = completed
                    .pipelines
                    .iter()
                    .position(|candidate| {
                        same_shader(&candidate.vertex_shader, &pipeline.vertex_shader)
                            && same_optional_shader(
                                candidate.fragment_shader.as_ref(),
                                pipeline.fragment_shader.as_ref(),
                            )
                            && candidate.plan == pipeline_plan
                    })
                    .unwrap_or_else(|| {
                        let index = completed.pipelines.len();
                        completed.pipelines.push(CompletedGpuCanvasPipeline {
                            vertex_shader: pipeline.vertex_shader.clone(),
                            fragment_shader: pipeline.fragment_shader.clone(),
                            plan: pipeline_plan.clone(),
                        });
                        completed.plan.pipelines.push(pipeline_plan);
                        index
                    });
                let pipeline_index = u32::try_from(pipeline_index)
                    .map_err(|_| Error::runtime("GPU pipeline index overflow"))?;
                let mut render_passes = plan.render_passes;
                for draw in &mut render_passes[0].draws {
                    draw.pipeline_index = pipeline_index;
                }
                completed.plan.render_passes.extend(render_passes);
            } else {
                state.completed = Some(CompletedGpuCanvasPass {
                    vertex_shader: Some(pipeline.vertex_shader.clone()),
                    fragment_shader: pipeline.fragment_shader.clone(),
                    pipelines: vec![CompletedGpuCanvasPipeline {
                        vertex_shader: pipeline.vertex_shader.clone(),
                        fragment_shader: pipeline.fragment_shader.clone(),
                        plan: pipeline_plan,
                    }],
                    plan,
                });
            }
            state.unfinished_passes = state.unfinished_passes.saturating_sub(1);
            this.finished = true;
            Ok(())
        });
    }
}

impl GpuRenderPass {
    fn ensure_open(&self) -> Result<()> {
        if self.finished {
            Err(Error::runtime("GPU render pass has already finished"))
        } else {
            Ok(())
        }
    }
}

fn attachment_format(view: &GpuCanvasAttachmentView) -> &str {
    match view {
        GpuCanvasAttachmentView::Canvas => "rgba8unorm",
        GpuCanvasAttachmentView::Texture(texture) => &texture.format,
    }
}

fn validate_pipeline_attachments(
    pipeline: &GpuPipeline,
    colors: &[GpuCanvasColorAttachment],
    depth: Option<&GpuCanvasDepthStencilAttachment>,
    sample_count: u32,
) -> Result<()> {
    if pipeline.state.sample_count != sample_count {
        return Err(Error::runtime(format!(
            "pipeline sampleCount ({}) does not match render pass sampleCount ({sample_count})",
            pipeline.state.sample_count
        )));
    }
    if pipeline.state.color_targets.len() != colors.len() {
        return Err(Error::runtime(format!(
            "pipeline has {} color targets but render pass has {} attachments",
            pipeline.state.color_targets.len(),
            colors.len()
        )));
    }
    for (index, (target, attachment)) in pipeline.state.color_targets.iter().zip(colors).enumerate()
    {
        if target.format != attachment_format(&attachment.view) {
            return Err(Error::runtime(format!(
                "pipeline color target {} format '{}' does not match attachment format '{}'",
                index + 1,
                target.format,
                attachment_format(&attachment.view)
            )));
        }
    }
    match (&pipeline.state.depth_stencil, depth) {
        (None, None) => {}
        (Some(pipeline_depth), Some(attachment))
            if pipeline_depth.format == attachment_format(&attachment.view) => {}
        (Some(pipeline_depth), Some(attachment)) => {
            return Err(Error::runtime(format!(
                "pipeline depth format '{}' does not match attachment format '{}'",
                pipeline_depth.format,
                attachment_format(&attachment.view)
            )));
        }
        (Some(_), None) => {
            return Err(Error::runtime(
                "pipeline depth/stencil state requires a render-pass depth attachment",
            ));
        }
        (None, Some(_)) => {
            return Err(Error::runtime(
                "render-pass depth attachment requires pipeline depth/stencil state",
            ));
        }
    }
    Ok(())
}

fn same_shader(left: &GpuShader, right: &GpuShader) -> bool {
    left.name == right.name
        && left.entries == right.entries
        && match (&left.module, &right.module) {
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
}

fn same_optional_shader(left: Option<&GpuShader>, right: Option<&GpuShader>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => same_shader(left, right),
        (None, None) => true,
        _ => false,
    }
}

/// Retained bytecode-backed GPU-canvas instance.
///
/// Source compilation and temporal sampling policy belong to editor tooling;
/// the baseline owns only execution of already compiled Luau and the imported
/// GPUCanvas userdata/plan contract.
pub struct GpuCanvasBytecodeProgram {
    vm: ScriptVm,
    instance: Table,
    state: Rc<RefCell<GpuCanvasState>>,
    execution_budget: Rc<Cell<u32>>,
}

impl std::fmt::Debug for GpuCanvasBytecodeProgram {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GpuCanvasBytecodeProgram")
            .finish_non_exhaustive()
    }
}

impl GpuCanvasBytecodeProgram {
    /// Load precompiled Luau bytecode, run its protocol generator, and retain
    /// the returned script instance.
    pub fn load(bytecode: &[u8]) -> Result<Self> {
        let vm = ScriptVm::new();
        vm.lua().set_memory_limit(MAX_LUAU_VM_MEMORY_BYTES)?;
        let execution_budget = Rc::new(Cell::new(MAX_LUAU_INTERRUPTS_PER_CALL));
        let interrupt_budget = Rc::clone(&execution_budget);
        vm.lua().set_interrupt(move |_| {
            let remaining = interrupt_budget.get();
            if remaining == 0 {
                return Err(Error::runtime(format!(
                    "GPU-canvas Luau execution exceeded {MAX_LUAU_INTERRUPTS_PER_CALL} interrupt safepoints"
                )));
            }
            interrupt_budget.set(remaining - 1);
            Ok(VmState::Continue)
        });
        let resource_budget = Rc::new(RefCell::new(GpuCanvasResourceBudget::default()));
        install_gpu_canvas_globals_with_budget(&vm, resource_budget)?;
        let canvases = Rc::new(RefCell::new(Vec::new()));
        let bindings = GpuCanvasContextBindings {
            canvases: Rc::clone(&canvases),
            shaders: GpuCanvasShaderCatalog::Direct(Rc::new(BTreeMap::from([(
                "scene".into(),
                vec![
                    GpuCanvasShaderEntry {
                        stage: GpuCanvasShaderStage::Vertex,
                        logical_entry_point: "vs_main".into(),
                        physical_entry_point: "vs_main".into(),
                    },
                    GpuCanvasShaderEntry {
                        stage: GpuCanvasShaderStage::Fragment,
                        logical_entry_point: "fs_main".into(),
                        physical_entry_point: "fs_main".into(),
                    },
                ],
            )]))),
            renderer_bindings: None,
        };
        let context = vm.lua().create_userdata(bindings)?;
        let chunk = vm.load_bytecode("gpu-canvas", bytecode)?;
        execution_budget.set(MAX_LUAU_INTERRUPTS_PER_CALL);
        let generator: Function = chunk.call(()).map_err(|error| {
            Error::runtime(format!(
                "gpu-canvas script must return a generator: {error}"
            ))
        })?;
        execution_budget.set(MAX_LUAU_INTERRUPTS_PER_CALL);
        let instance: Table = generator
            .call(context)
            .map_err(|error| Error::runtime(format!("gpu-canvas generator failed: {error}")))?;
        let state = canvases.borrow().first().cloned().ok_or_else(|| {
            Error::runtime("gpu-canvas generator did not create a GPUCanvas occurrence")
        })?;
        Ok(Self {
            vm,
            instance,
            state,
            execution_budget,
        })
    }

    /// Advance the retained script by an exact fixed-step delta. A missing
    /// `advance` method represents a static scene and is accepted.
    pub fn advance(&mut self, elapsed_seconds: f64) -> Result<bool> {
        let method: Value = self.instance.get("advance")?;
        let Value::Function(method) = method else {
            return Ok(false);
        };
        self.execution_budget.set(MAX_LUAU_INTERRUPTS_PER_CALL);
        let value: Value = method.call((self.instance.clone(), elapsed_seconds))?;
        Ok(matches!(value, Value::Boolean(true)))
    }

    /// Override one authored numeric input on the retained script instance.
    ///
    /// Hosts use this to apply editor-controlled values before advancing or
    /// drawing. Unknown keys and type mismatches fail closed so a stale
    /// inspector schema cannot silently create a new Lua field.
    pub fn set_number_input(&mut self, key: &str, value: f64) -> Result<bool> {
        if !value.is_finite() {
            return Err(Error::runtime(format!(
                "GPU-canvas input `{key}` must be a finite number"
            )));
        }
        let current: Value = self.instance.get(key)?;
        let changed = match current {
            Value::Integer(current) => current as f64 != value,
            Value::Number(current) => current != value,
            Value::Nil => return Err(unknown_gpu_canvas_input(key)),
            other => return Err(gpu_canvas_input_type_mismatch(key, "number", &other)),
        };
        self.instance.set(key, value)?;
        Ok(changed)
    }

    /// Override one authored boolean input on the retained script instance.
    pub fn set_boolean_input(&mut self, key: &str, value: bool) -> Result<bool> {
        let current: Value = self.instance.get(key)?;
        let changed = match current {
            Value::Boolean(current) => current != value,
            Value::Nil => return Err(unknown_gpu_canvas_input(key)),
            other => return Err(gpu_canvas_input_type_mismatch(key, "boolean", &other)),
        };
        self.instance.set(key, value)?;
        Ok(changed)
    }

    /// Override one authored string or color input on the retained script
    /// instance. Colors use their canonical string representation.
    pub fn set_string_input(&mut self, key: &str, value: &str) -> Result<bool> {
        let current: Value = self.instance.get(key)?;
        let changed = match current {
            Value::String(current) => current.as_bytes() != value.as_bytes(),
            Value::Nil => return Err(unknown_gpu_canvas_input(key)),
            other => return Err(gpu_canvas_input_type_mismatch(key, "string", &other)),
        };
        self.instance.set(key, value)?;
        Ok(changed)
    }

    /// Execute `drawCanvas` and return the exact Rust-owned completed pass.
    pub fn draw(&mut self) -> Result<GpuCanvasDrawPlan> {
        {
            let mut state = self.state.borrow_mut();
            state.completed = None;
            state.unfinished_passes = 0;
        }
        let method: Value = self.instance.get("drawCanvas")?;
        let Value::Function(method) = method else {
            return Err(Error::runtime(
                "gpu-canvas script instance has no drawCanvas function",
            ));
        };
        self.execution_budget.set(MAX_LUAU_INTERRUPTS_PER_CALL);
        method.call::<()>((self.instance.clone(),))?;
        if self.state.borrow().unfinished_passes != 0 {
            self.state.borrow_mut().completed = None;
            return Err(Error::runtime(
                "GPU render pass left open at script return; call finish() on every pass",
            ));
        }
        self.state
            .borrow_mut()
            .completed
            .take()
            .map(|completed| completed.plan)
            .ok_or_else(|| Error::runtime("gpu-canvas drawCanvas did not finish a render pass"))
    }

    pub fn vm(&self) -> &ScriptVm {
        &self.vm
    }
}

fn unknown_gpu_canvas_input(key: &str) -> Error {
    Error::runtime(format!("GPU-canvas input `{key}` is not defined"))
}

fn gpu_canvas_input_type_mismatch(key: &str, expected: &str, actual: &Value) -> Error {
    Error::runtime(format!(
        "GPU-canvas input `{key}` expected {expected}, found {}",
        actual.type_name()
    ))
}

pub(crate) fn install_gpu_canvas_globals(vm: &ScriptVm) -> Result<()> {
    install_gpu_canvas_globals_with_budget(
        vm,
        Rc::new(RefCell::new(GpuCanvasResourceBudget::default())),
    )
}

fn resolve_shader_entry(
    shader: &GpuShader,
    stage: GpuCanvasShaderStage,
    requested_logical: Option<&str>,
    stage_name: &str,
) -> Result<GpuCanvasShaderEntrySelection> {
    let requested_logical = requested_logical.filter(|name| !name.is_empty());
    let entry = match requested_logical {
        Some(logical) => shader
            .entries
            .iter()
            .find(|entry| entry.stage == stage && entry.logical_entry_point == logical),
        None => shader.entries.iter().find(|entry| entry.stage == stage),
    };
    let entry = entry.ok_or_else(|| {
        let available = shader
            .entries
            .iter()
            .filter(|entry| entry.stage == stage)
            .map(|entry| entry.logical_entry_point.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        match requested_logical {
            Some(logical) => Error::runtime(format!(
                "GPUPipeline {stage_name} entry point '{logical}' not found (available: {})",
                if available.is_empty() {
                    "<none>"
                } else {
                    available.as_str()
                }
            )),
            None => Error::runtime(format!(
                "GPUPipeline shader has no {stage_name} entry point"
            )),
        }
    })?;
    Ok(GpuCanvasShaderEntrySelection {
        logical_entry_point: entry.logical_entry_point.clone(),
        physical_entry_point: entry.physical_entry_point.clone(),
    })
}

fn decode_pipeline_stage(
    value: Value,
    stage: GpuCanvasShaderStage,
    stage_name: &str,
) -> Result<(GpuShader, GpuCanvasShaderEntrySelection)> {
    let (shader, requested_logical) = match value {
        Value::UserData(shader) => (shader.borrow::<GpuShader>()?.clone(), None),
        Value::Table(descriptor) => {
            reject_unknown_fields(
                &descriptor,
                &["module", "entryPoint"],
                "GPU pipeline stage descriptor",
            )?;
            let module: AnyUserData = descriptor.get("module")?;
            let shader = module.borrow::<GpuShader>()?.clone();
            let requested: Option<String> = descriptor.get("entryPoint")?;
            (shader, requested)
        }
        _ => {
            return Err(Error::runtime(format!(
                "GPUPipeline {stage_name} must be a Shader or {{ module = Shader, entryPoint = string? }}"
            )));
        }
    };
    let selection = resolve_shader_entry(&shader, stage, requested_logical.as_deref(), stage_name)?;
    Ok((shader, selection))
}

fn install_gpu_canvas_globals_with_budget(
    vm: &ScriptVm,
    resource_budget: Rc<RefCell<GpuCanvasResourceBudget>>,
) -> Result<()> {
    let lua = vm.lua();
    let standard_buffer: Option<Table> = lua.globals().get("buffer")?;
    let buffer = lua.create_table();
    if let Some(standard_buffer) = standard_buffer {
        for pair in standard_buffer.pairs::<Value, Value>() {
            let (key, value) = pair?;
            buffer.raw_set(key, value)?;
        }
    }
    let cpu_buffer_budget = Rc::clone(&resource_budget);
    buffer.set(
        "create",
        lua.create_function(move |lua, size: usize| {
            if size == 0 || size > MAX_CPU_BUFFER_BYTES {
                return Err(Error::runtime(format!(
                    "buffer.create size must be between 1 and {MAX_CPU_BUFFER_BYTES} bytes"
                )));
            }
            cpu_buffer_budget.borrow_mut().reserve(size)?;
            lua.create_buffer_with_capacity(size)
        })?,
    )?;
    buffer.set(
        "writef32",
        lua.create_function(|_, (target, offset, value): (LuaBuffer, usize, f64)| {
            let end = offset
                .checked_add(4)
                .ok_or_else(|| Error::runtime("buffer.writef32 byte range overflow"))?;
            if offset % 4 != 0 || end > target.len() {
                return Err(Error::runtime(format!(
                    "buffer.writef32 offset {offset} is outside {} bytes",
                    target.len()
                )));
            }
            target.write_bytes(offset, &(value as f32).to_le_bytes());
            Ok(())
        })?,
    )?;
    lua.globals().set("buffer", buffer)?;

    let gpu_buffer_budget = Rc::clone(&resource_budget);
    install_constructor(lua, "GPUBuffer", move |lua, descriptor| {
        reject_unknown_fields(
            &descriptor,
            &["size", "usage", "data", "immutable", "label"],
            "GPUBuffer",
        )?;
        let size: usize = descriptor.get("size")?;
        let usage_name = decode_buffer_usage(&descriptor)?;
        let usage = match usage_name.as_str() {
            "uniform" => GpuBufferUsage::Uniform,
            "vertex" => GpuBufferUsage::Vertex,
            "index" => GpuBufferUsage::Index,
            usage => {
                return Err(Error::runtime(format!(
                    "unsupported GPUBuffer usage '{usage}'"
                )));
            }
        };
        let max_size = match usage {
            GpuBufferUsage::Uniform => MAX_UNIFORM_BUFFER_BYTES,
            GpuBufferUsage::Vertex | GpuBufferUsage::Index => MAX_VERTEX_BUFFER_BYTES,
        };
        if size == 0 || size > max_size {
            return Err(Error::runtime(format!(
                "GPUBuffer {usage:?} size must be between 1 and {max_size} bytes"
            )));
        }
        let immutable = descriptor
            .get::<Option<bool>>("immutable")?
            .unwrap_or(false);
        let source: Option<LuaBuffer> = descriptor.get("data")?;
        if immutable && source.is_none() {
            return Err(Error::runtime(
                "GPUBuffer immutable=true requires initial data",
            ));
        }
        if let Some(source) = &source
            && source.len() != size
        {
            return Err(Error::runtime(format!(
                "GPUBuffer size {size} does not match {} source bytes",
                source.len()
            )));
        }
        gpu_buffer_budget.borrow_mut().reserve(size)?;
        // Validate both the descriptor and aggregate budget before copying
        // Luau-owned bytes into the retained host buffer.
        let source = source.map_or_else(|| vec![0; size], |source| source.to_vec());
        lua.create_userdata(GpuBuffer {
            usage,
            immutable,
            bytes: Rc::new(RefCell::new(source)),
        })
    })?;
    install_constructor(lua, "GPUTexture", |lua, descriptor| {
        reject_unknown_fields(
            &descriptor,
            &[
                "width",
                "height",
                "format",
                "type",
                "renderTarget",
                "sampleCount",
                "mipmaps",
                "layers",
                "label",
            ],
            "GPUTexture",
        )?;
        let width: u32 = descriptor.get("width")?;
        let height: u32 = descriptor.get("height")?;
        if width == 0
            || height == 0
            || width > MAX_GPU_CANVAS_DIMENSION
            || height > MAX_GPU_CANVAS_DIMENSION
        {
            return Err(Error::runtime(format!(
                "GPUTexture dimensions must be between 1 and {MAX_GPU_CANVAS_DIMENSION}"
            )));
        }
        let format = descriptor
            .get::<Option<String>>("format")?
            .unwrap_or_else(|| "rgba8unorm".into());
        validate_texture_format(&format)?;
        let texture_type = descriptor
            .get::<Option<String>>("type")?
            .unwrap_or_else(|| "2d".into());
        if !matches!(texture_type.as_str(), "2d" | "cube" | "3d" | "2d-array") {
            return Err(Error::runtime(format!(
                "invalid GPUTexture type '{texture_type}'"
            )));
        }
        let render_target = descriptor
            .get::<Option<bool>>("renderTarget")?
            .unwrap_or(false);
        let sample_count = descriptor.get::<Option<u32>>("sampleCount")?.unwrap_or(1);
        if sample_count == 0 || !sample_count.is_power_of_two() || sample_count > 16 {
            return Err(Error::runtime(
                "GPUTexture sampleCount must be a power of two from 1 through 16",
            ));
        }
        if sample_count > 1 && !render_target {
            return Err(Error::runtime(
                "multisampled GPUTexture resources must be render targets",
            ));
        }
        let mip_level_count = descriptor.get::<Option<u32>>("mipmaps")?.unwrap_or(1);
        let depth_or_array_layers = descriptor
            .get::<Option<u32>>("layers")?
            .unwrap_or_else(|| if texture_type == "cube" { 6 } else { 1 });
        if mip_level_count == 0 || depth_or_array_layers == 0 {
            return Err(Error::runtime(
                "GPUTexture mipmaps and layers must be positive",
            ));
        }
        lua.create_userdata(GpuTexture {
            resource_id: NEXT_GPU_TEXTURE_RESOURCE_ID.fetch_add(1, Ordering::Relaxed),
            lifetime: GpuCanvasResourceLifetime::new(),
            width,
            height,
            depth_or_array_layers,
            format,
            texture_type,
            render_target,
            sample_count,
            mip_level_count,
            uploads: Rc::new(RefCell::new(Vec::new())),
        })
    })?;
    install_constructor(lua, "GPUSampler", |lua, descriptor| {
        reject_unknown_fields(
            &descriptor,
            &[
                "min",
                "mag",
                "mipmap",
                "wrapU",
                "wrapV",
                "wrapW",
                "compare",
                "minLod",
                "maxLod",
                "maxAnisotropy",
                "label",
            ],
            "GPUSampler",
        )?;
        let min_filter = optional_enum(&descriptor, "min", "nearest", &["nearest", "linear"])?;
        let mag_filter = optional_enum(&descriptor, "mag", "nearest", &["nearest", "linear"])?;
        let mipmap_filter =
            optional_enum(&descriptor, "mipmap", "nearest", &["nearest", "linear"])?;
        let wraps = &["repeat", "mirror-repeat", "clamp-to-edge"];
        let address_mode_u = optional_enum(&descriptor, "wrapU", "clamp-to-edge", wraps)?;
        let address_mode_v = optional_enum(&descriptor, "wrapV", "clamp-to-edge", wraps)?;
        let address_mode_w = optional_enum(&descriptor, "wrapW", "clamp-to-edge", wraps)?;
        let compare = descriptor.get::<Option<String>>("compare")?;
        if let Some(compare) = compare.as_deref() {
            validate_compare(compare)?;
        }
        let lod_min_clamp = descriptor.get::<Option<f32>>("minLod")?.unwrap_or(0.0);
        let lod_max_clamp = descriptor.get::<Option<f32>>("maxLod")?.unwrap_or(32.0);
        if !lod_min_clamp.is_finite() || !lod_max_clamp.is_finite() || lod_min_clamp > lod_max_clamp
        {
            return Err(Error::runtime("GPUSampler minLod must not exceed maxLod"));
        }
        let max_anisotropy = descriptor.get::<Option<u16>>("maxAnisotropy")?.unwrap_or(1);
        if !(1..=16).contains(&max_anisotropy) || !max_anisotropy.is_power_of_two() {
            return Err(Error::runtime(
                "GPUSampler maxAnisotropy must be a power of two in [1, 16]",
            ));
        }
        lua.create_userdata(GpuSampler {
            min_filter,
            mag_filter,
            mipmap_filter,
            address_mode_u,
            address_mode_v,
            address_mode_w,
            compare,
            lod_min_clamp,
            lod_max_clamp,
            max_anisotropy,
        })
    })?;
    install_constructor(lua, "GPUPipeline", |lua, descriptor| {
        reject_unknown_fields(
            &descriptor,
            &[
                "vertex",
                "fragment",
                "vertexLayout",
                "colorTargets",
                "depthStencil",
                "stencilFront",
                "stencilBack",
                "stencilReadMask",
                "stencilWriteMask",
                "bindGroupLayouts",
                "cullMode",
                "winding",
                "topology",
                "sampleCount",
                "label",
            ],
            "GPUPipeline",
        )?;
        let (vertex, vertex_entry) = decode_pipeline_stage(
            descriptor.get("vertex")?,
            GpuCanvasShaderStage::Vertex,
            "vertex",
        )?;
        let targets = descriptor.get::<Option<Table>>("colorTargets")?;
        let mut decoded_targets = Vec::new();
        if let Some(targets) = targets {
            if targets.raw_len() > 4 {
                return Err(Error::runtime(
                    "GPUPipeline colorTargets supports at most four entries",
                ));
            }
            for target in targets.sequence_values::<Table>() {
                let target = target?;
                reject_unknown_fields(
                    &target,
                    &["format", "writeMask", "blend"],
                    "GPU color target",
                )?;
                decoded_targets.push(decode_color_target(&target)?);
            }
        }
        let fragment_value: Value = descriptor.get("fragment")?;
        let (fragment, fragment_entry) = if matches!(fragment_value, Value::Nil) {
            if decoded_targets.is_empty() {
                (None, None)
            } else {
                let fragment_entry = resolve_shader_entry(
                    &vertex,
                    GpuCanvasShaderStage::Fragment,
                    None,
                    "fragment",
                )?;
                (Some(vertex.clone()), Some(fragment_entry))
            }
        } else {
            let (fragment, fragment_entry) =
                decode_pipeline_stage(fragment_value, GpuCanvasShaderStage::Fragment, "fragment")?;
            (Some(fragment), Some(fragment_entry))
        };
        let state = decode_pipeline_state(&descriptor, decoded_targets)?;
        let explicit_layouts: Option<Table> = descriptor.get("bindGroupLayouts")?;
        let explicit_bind_group_layouts = explicit_layouts.is_some();
        let mut bind_group_layouts = Vec::new();
        if let Some(layouts) = explicit_layouts {
            for layout in layouts.sequence_values::<AnyUserData>() {
                bind_group_layouts.push(layout?.borrow::<GpuBindGroupLayout>()?.clone());
            }
        } else {
            bind_group_layouts.push(GpuBindGroupLayout {
                group: 0,
                dynamic_uniform_bindings: Vec::new(),
            });
        }
        lua.create_userdata(GpuPipeline {
            vertex_shader: vertex,
            fragment_shader: fragment,
            vertex_entry,
            fragment_entry,
            vertex_layouts: decode_vertex_layouts(&descriptor)?,
            state,
            bind_group_layouts,
            explicit_bind_group_layouts,
        })
    })?;
    install_constructor(lua, "GPUBindGroupLayout", |lua, descriptor| {
        reject_unknown_fields(
            &descriptor,
            &["groupIndex", "shader", "dynamicUBOs"],
            "GPUBindGroupLayout",
        )?;
        let shader: AnyUserData = descriptor.get("shader")?;
        let _shader = shader.borrow::<GpuShader>()?;
        let group: u32 = descriptor.get("groupIndex")?;
        if group >= MAX_GPU_CANVAS_BIND_GROUPS {
            return Err(Error::runtime(format!(
                "GPUBindGroupLayout groupIndex must be less than {MAX_GPU_CANVAS_BIND_GROUPS}"
            )));
        }
        let mut dynamic_uniform_bindings = Vec::new();
        if let Some(dynamic) = descriptor.get::<Option<Table>>("dynamicUBOs")? {
            for binding in dynamic.sequence_values::<u32>() {
                let binding = binding?;
                if !dynamic_uniform_bindings.contains(&binding) {
                    dynamic_uniform_bindings.push(binding);
                }
            }
            dynamic_uniform_bindings.sort_unstable();
        }
        lua.create_userdata(GpuBindGroupLayout {
            group,
            dynamic_uniform_bindings,
        })
    })?;
    install_constructor(lua, "GPUBindGroup", |lua, descriptor| {
        reject_unknown_fields(
            &descriptor,
            &["layout", "ubos", "textures", "samplers"],
            "GPUBindGroup",
        )?;
        let layout: AnyUserData = descriptor.get("layout")?;
        let layout = layout.borrow::<GpuBindGroupLayout>()?;
        let ubos = descriptor.get::<Option<Table>>("ubos")?;
        let mut uniforms = Vec::new();
        let mut bindings = BTreeSet::new();
        for entry in ubos
            .into_iter()
            .flat_map(|table| table.sequence_values::<Table>())
        {
            if uniforms.len() >= MAX_GPU_CANVAS_UNIFORM_BINDINGS_PER_GROUP {
                return Err(Error::runtime(format!(
                    "GPUBindGroup supports at most {MAX_GPU_CANVAS_UNIFORM_BINDINGS_PER_GROUP} uniform bindings"
                )));
            }
            let entry = entry?;
            reject_unknown_fields(
                &entry,
                &["slot", "buffer", "offset", "size"],
                "GPUBindGroup UBO",
            )?;
            let binding: u32 = entry.get("slot")?;
            if binding > MAX_GPU_CANVAS_BINDING_INDEX {
                return Err(Error::runtime(format!(
                    "GPUBindGroup binding must be at most {MAX_GPU_CANVAS_BINDING_INDEX}"
                )));
            }
            if !bindings.insert(binding) {
                return Err(Error::runtime(format!(
                    "GPUBindGroup binding {binding} is duplicated"
                )));
            }
            let buffer: AnyUserData = entry.get("buffer")?;
            let buffer = buffer.borrow::<GpuBuffer>()?.clone();
            if buffer.usage != GpuBufferUsage::Uniform {
                return Err(Error::runtime(
                    "GPUBindGroup UBO entry requires a uniform GPUBuffer",
                ));
            }
            let offset = entry.get::<Option<usize>>("offset")?.unwrap_or(0);
            let size = entry
                .get::<Option<usize>>("size")?
                .unwrap_or_else(|| buffer.bytes.borrow().len().saturating_sub(offset));
            if size == 0
                || offset
                    .checked_add(size)
                    .is_none_or(|end| end > buffer.bytes.borrow().len())
            {
                return Err(Error::runtime(format!(
                    "GPUBindGroup UBO binding {binding} range is out of bounds"
                )));
            }
            uniforms.push(GpuUniformBinding {
                binding,
                buffer,
                offset,
                size,
            });
        }
        let mut textures = Vec::new();
        if let Some(entries) = descriptor.get::<Option<Table>>("textures")? {
            for entry in entries.sequence_values::<Table>() {
                if textures.len() >= 8 {
                    return Err(Error::runtime(
                        "GPUBindGroup supports at most 8 texture bindings",
                    ));
                }
                let entry = entry?;
                reject_unknown_fields(&entry, &["slot", "view"], "GPUBindGroup texture")?;
                let binding: u32 = entry.get("slot")?;
                if binding > MAX_GPU_CANVAS_BINDING_INDEX {
                    return Err(Error::runtime("GPUBindGroup texture slot must be 0-7"));
                }
                if !bindings.insert(binding) {
                    return Err(Error::runtime(format!(
                        "GPUBindGroup binding {binding} is duplicated"
                    )));
                }
                let view: AnyUserData = entry.get("view")?;
                textures.push(GpuTextureBinding {
                    binding,
                    view: view.borrow::<GpuTextureView>()?.clone(),
                });
            }
        }
        let mut samplers = Vec::new();
        if let Some(entries) = descriptor.get::<Option<Table>>("samplers")? {
            for entry in entries.sequence_values::<Table>() {
                if samplers.len() >= 8 {
                    return Err(Error::runtime(
                        "GPUBindGroup supports at most 8 sampler bindings",
                    ));
                }
                let entry = entry?;
                reject_unknown_fields(&entry, &["slot", "sampler"], "GPUBindGroup sampler")?;
                let binding: u32 = entry.get("slot")?;
                if binding > MAX_GPU_CANVAS_BINDING_INDEX {
                    return Err(Error::runtime("GPUBindGroup sampler slot must be 0-7"));
                }
                if !bindings.insert(binding) {
                    return Err(Error::runtime(format!(
                        "GPUBindGroup binding {binding} is duplicated"
                    )));
                }
                let sampler: AnyUserData = entry.get("sampler")?;
                samplers.push(GpuSamplerResourceBinding {
                    binding,
                    sampler: sampler.borrow::<GpuSampler>()?.clone(),
                });
            }
        }
        lua.create_userdata(GpuBindGroup {
            group: layout.group,
            uniforms,
            textures,
            samplers,
            dynamic_uniform_bindings: layout.dynamic_uniform_bindings.clone(),
        })
    })?;
    Ok(())
}

fn install_constructor(
    lua: &luaur_rt::Lua,
    name: &str,
    constructor: impl Fn(&luaur_rt::Lua, Table) -> Result<AnyUserData> + 'static,
) -> Result<()> {
    let table = lua.create_table();
    table.set("new", lua.create_function(constructor)?)?;
    lua.globals().set(name, table)
}

fn decode_buffer_usage(descriptor: &Table) -> Result<String> {
    match descriptor.get::<Value>("usage")? {
        Value::String(value) => Ok(value.to_str()?.to_owned()),
        Value::Table(values) if values.raw_len() == 1 => values.get(1),
        Value::Table(_) => Err(Error::runtime(
            "GPUBuffer usage array must contain exactly one string",
        )),
        _ => Err(Error::runtime(
            "GPUBuffer usage must be a string or one-element string array",
        )),
    }
}

fn validate_texture_format(format: &str) -> Result<()> {
    if matches!(
        format,
        "r8unorm"
            | "rg8unorm"
            | "rgba8unorm"
            | "bgra8unorm"
            | "rgba16float"
            | "rg16float"
            | "r16float"
            | "rgba32float"
            | "rg32float"
            | "r32float"
            | "rgb10a2unorm"
            | "rg11b10ufloat"
            | "depth16unorm"
            | "depth24plus-stencil8"
            | "depth32float"
            | "depth32float-stencil8"
            | "bc1-rgba-unorm"
            | "bc3-rgba-unorm"
            | "bc7-rgba-unorm"
            | "etc2-rgb8unorm"
            | "etc2-rgba8unorm"
            | "astc-4x4-unorm"
            | "astc-6x6-unorm"
            | "astc-8x8-unorm"
    ) {
        Ok(())
    } else {
        Err(Error::runtime(format!(
            "invalid GPU texture format '{format}'"
        )))
    }
}

fn optional_enum(
    descriptor: &Table,
    field: &str,
    default: &str,
    allowed: &[&str],
) -> Result<String> {
    let value = descriptor
        .get::<Option<String>>(field)?
        .unwrap_or_else(|| default.into());
    if allowed.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(Error::runtime(format!("invalid {field} value '{value}'")))
    }
}

fn validate_compare(value: &str) -> Result<()> {
    if matches!(
        value,
        "never"
            | "less"
            | "equal"
            | "less-equal"
            | "greater"
            | "not-equal"
            | "greater-equal"
            | "always"
    ) {
        Ok(())
    } else {
        Err(Error::runtime(format!(
            "invalid GPU compare function '{value}'"
        )))
    }
}

fn decode_color_target(target: &Table) -> Result<GpuCanvasColorTarget> {
    let format = target
        .get::<Option<String>>("format")?
        .unwrap_or_else(|| "rgba8unorm".into());
    validate_texture_format(&format)?;
    let write_mask = target
        .get::<Option<String>>("writeMask")?
        .unwrap_or_else(|| "rgba".into());
    if !matches!(write_mask.as_str(), "" | "none" | "all" | "rgba")
        && write_mask
            .chars()
            .any(|channel| !matches!(channel.to_ascii_lowercase(), 'r' | 'g' | 'b' | 'a'))
    {
        return Err(Error::runtime(format!(
            "invalid GPU color writeMask '{write_mask}'"
        )));
    }
    let blend = target
        .get::<Option<Table>>("blend")?
        .map(|blend| decode_blend_state(&blend))
        .transpose()?;
    Ok(GpuCanvasColorTarget {
        format,
        write_mask,
        blend,
    })
}

fn decode_pipeline_state(
    descriptor: &Table,
    color_targets: Vec<GpuCanvasColorTarget>,
) -> Result<GpuCanvasPipelineState> {
    let depth_stencil = descriptor
        .get::<Option<Table>>("depthStencil")?
        .map(|depth| decode_depth_stencil(descriptor, &depth))
        .transpose()?;
    let cull_mode = optional_enum(descriptor, "cullMode", "none", &["none", "front", "back"])?;
    let winding = optional_enum(descriptor, "winding", "ccw", &["cw", "ccw"])?;
    let topology = optional_enum(
        descriptor,
        "topology",
        "triangle-list",
        &[
            "triangle-list",
            "triangle-strip",
            "line-list",
            "line-strip",
            "point-list",
        ],
    )?;
    let sample_count = descriptor.get::<Option<u32>>("sampleCount")?.unwrap_or(1);
    if sample_count == 0 || !sample_count.is_power_of_two() || sample_count > 16 {
        return Err(Error::runtime(
            "GPUPipeline sampleCount must be a power of two from 1 through 16",
        ));
    }
    Ok(GpuCanvasPipelineState {
        color_targets,
        depth_stencil,
        cull_mode,
        winding,
        topology,
        sample_count,
    })
}

fn decode_blend_state(descriptor: &Table) -> Result<GpuCanvasBlendState> {
    reject_unknown_fields(
        descriptor,
        &[
            "srcColor", "dstColor", "colorOp", "srcAlpha", "dstAlpha", "alphaOp",
        ],
        "GPU blend",
    )?;
    let factors = &[
        "zero",
        "one",
        "src",
        "one-minus-src",
        "src-alpha",
        "one-minus-src-alpha",
        "dst",
        "one-minus-dst",
        "dst-alpha",
        "one-minus-dst-alpha",
        "src-alpha-saturated",
        "constant",
        "one-minus-constant",
    ];
    let operations = &["add", "subtract", "reverse-subtract", "min", "max"];
    Ok(GpuCanvasBlendState {
        src_color: optional_enum(descriptor, "srcColor", "one", factors)?,
        dst_color: optional_enum(descriptor, "dstColor", "zero", factors)?,
        color_op: optional_enum(descriptor, "colorOp", "add", operations)?,
        src_alpha: optional_enum(descriptor, "srcAlpha", "one", factors)?,
        dst_alpha: optional_enum(descriptor, "dstAlpha", "zero", factors)?,
        alpha_op: optional_enum(descriptor, "alphaOp", "add", operations)?,
    })
}

fn decode_depth_stencil(
    pipeline: &Table,
    descriptor: &Table,
) -> Result<GpuCanvasDepthStencilState> {
    reject_unknown_fields(
        descriptor,
        &[
            "format",
            "compare",
            "write",
            "depthBias",
            "depthBiasSlopeScale",
            "depthBiasClamp",
        ],
        "GPU depthStencil",
    )?;
    let format = descriptor
        .get::<Option<String>>("format")?
        .unwrap_or_else(|| "depth32float".into());
    if !matches!(
        format.as_str(),
        "depth16unorm" | "depth24plus-stencil8" | "depth32float" | "depth32float-stencil8"
    ) {
        return Err(Error::runtime(format!(
            "invalid depth/stencil format '{format}'"
        )));
    }
    let depth_compare = descriptor
        .get::<Option<String>>("compare")?
        .unwrap_or_else(|| "always".into());
    validate_compare(&depth_compare)?;
    Ok(GpuCanvasDepthStencilState {
        format,
        depth_compare,
        depth_write_enabled: descriptor.get::<Option<bool>>("write")?.unwrap_or(false),
        depth_bias: descriptor.get::<Option<i32>>("depthBias")?.unwrap_or(0),
        depth_bias_slope_scale: descriptor
            .get::<Option<f32>>("depthBiasSlopeScale")?
            .unwrap_or(0.0),
        depth_bias_clamp: descriptor
            .get::<Option<f32>>("depthBiasClamp")?
            .unwrap_or(0.0),
        stencil_front: decode_stencil_face(pipeline.get::<Option<Table>>("stencilFront")?)?,
        stencil_back: decode_stencil_face(pipeline.get::<Option<Table>>("stencilBack")?)?,
        stencil_read_mask: pipeline
            .get::<Option<u32>>("stencilReadMask")?
            .unwrap_or(0xff),
        stencil_write_mask: pipeline
            .get::<Option<u32>>("stencilWriteMask")?
            .unwrap_or(0xff),
    })
}

fn decode_stencil_face(descriptor: Option<Table>) -> Result<GpuCanvasStencilFace> {
    let Some(descriptor) = descriptor else {
        return Ok(GpuCanvasStencilFace {
            compare: "always".into(),
            fail_op: "keep".into(),
            depth_fail_op: "keep".into(),
            pass_op: "keep".into(),
        });
    };
    reject_unknown_fields(
        &descriptor,
        &["compare", "failOp", "depthFailOp", "passOp"],
        "GPU stencil face",
    )?;
    let compare = descriptor
        .get::<Option<String>>("compare")?
        .unwrap_or_else(|| "always".into());
    validate_compare(&compare)?;
    let operations = &[
        "keep",
        "zero",
        "replace",
        "increment-clamp",
        "decrement-clamp",
        "invert",
        "increment-wrap",
        "decrement-wrap",
    ];
    Ok(GpuCanvasStencilFace {
        compare,
        fail_op: optional_enum(&descriptor, "failOp", "keep", operations)?,
        depth_fail_op: optional_enum(&descriptor, "depthFailOp", "keep", operations)?,
        pass_op: optional_enum(&descriptor, "passOp", "keep", operations)?,
    })
}

fn reject_unknown_fields(table: &Table, allowed: &[&str], label: &str) -> Result<()> {
    for pair in table.clone().pairs::<String, Value>() {
        let (key, _) = pair?;
        if !allowed.contains(&key.as_str()) {
            return Err(Error::runtime(format!(
                "{label} field '{key}' is unsupported"
            )));
        }
    }
    Ok(())
}

fn decode_vertex_layouts(descriptor: &Table) -> Result<Vec<GpuCanvasVertexLayout>> {
    let layouts: Table = descriptor.get("vertexLayout")?;
    let mut decoded = Vec::new();
    let mut shader_locations = BTreeSet::new();
    let mut attribute_count = 0;
    for layout in layouts.sequence_values::<Table>() {
        if decoded.len() >= MAX_GPU_CANVAS_VERTEX_BUFFERS {
            return Err(Error::runtime(format!(
                "GPU pipeline supports at most {MAX_GPU_CANVAS_VERTEX_BUFFERS} vertex layouts"
            )));
        }
        let layout = layout?;
        reject_unknown_fields(
            &layout,
            &["stride", "stepMode", "attributes"],
            "GPU vertex layout",
        )?;
        let stride: u64 = layout.get("stride")?;
        if stride == 0 || stride > 2_048 {
            return Err(Error::runtime(
                "GPU vertex layout stride must be between 1 and 2048 bytes",
            ));
        }
        let source_attributes: Table = layout.get("attributes")?;
        let mut attributes = Vec::new();
        for attribute in source_attributes.sequence_values::<Table>() {
            attribute_count += 1;
            if attribute_count > MAX_GPU_CANVAS_VERTEX_ATTRIBUTES {
                return Err(Error::runtime(format!(
                    "GPU pipeline supports at most {MAX_GPU_CANVAS_VERTEX_ATTRIBUTES} vertex attributes"
                )));
            }
            let attribute = attribute?;
            reject_unknown_fields(
                &attribute,
                &["format", "slot", "offset"],
                "GPU vertex attribute",
            )?;
            let format: String = attribute.get("format")?;
            let format_size = match format.as_str() {
                "float32" => 4,
                "float32x2" => 8,
                "float32x3" => 12,
                "float32x4" => 16,
                "uint8x4" | "unorm8x4" | "snorm8x4" | "float16x2" => 4,
                "float16x4" => 8,
                _ => {
                    return Err(Error::runtime(format!(
                        "unsupported GPU vertex format '{format}'"
                    )));
                }
            };
            let shader_location: u32 = attribute.get("slot")?;
            if shader_location >= MAX_GPU_CANVAS_VERTEX_ATTRIBUTES as u32 {
                return Err(Error::runtime(format!(
                    "GPU vertex attribute slot must be less than {MAX_GPU_CANVAS_VERTEX_ATTRIBUTES}"
                )));
            }
            if !shader_locations.insert(shader_location) {
                return Err(Error::runtime(format!(
                    "GPU vertex attribute slot {shader_location} is duplicated"
                )));
            }
            let offset: u64 = attribute.get("offset")?;
            if offset
                .checked_add(format_size)
                .is_none_or(|end| end > stride)
            {
                return Err(Error::runtime(format!(
                    "GPU vertex attribute at offset {offset} exceeds stride {stride}"
                )));
            }
            attributes.push(GpuCanvasVertexAttribute {
                shader_location,
                offset,
                format,
            });
        }
        if attributes.is_empty() {
            return Err(Error::runtime(
                "GPU vertex layouts must contain at least one attribute",
            ));
        }
        let step_mode = optional_enum(&layout, "stepMode", "vertex", &["vertex", "instance"])?;
        decoded.push(GpuCanvasVertexLayout {
            stride,
            step_mode,
            attributes,
        });
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::{
        GpuCanvasShaderEntry, GpuCanvasShaderStage, GpuShader, checked_gpu_buffer_write_range,
        resolve_shader_entry,
    };

    #[test]
    fn gpu_buffer_write_range_is_validated_before_source_copy() {
        assert_eq!(
            checked_gpu_buffer_write_range(4, 12, 16).expect("exact range"),
            (12, 16)
        );

        let error = checked_gpu_buffer_write_range(16 * 1024 * 1024, 0, 64 * 1024)
            .expect_err("large rejected sources must fail before to_vec");
        assert!(error.to_string().contains("exceeds 65536 bytes"), "{error}");

        let error = checked_gpu_buffer_write_range(1, usize::MAX, usize::MAX)
            .expect_err("range arithmetic must be checked before to_vec");
        assert!(error.to_string().contains("range overflow"), "{error}");
    }

    #[test]
    fn shader_entries_use_declaration_order_defaults_and_named_logical_selection() {
        let shader = GpuShader {
            name: "scene".into(),
            entries: vec![
                GpuCanvasShaderEntry {
                    stage: GpuCanvasShaderStage::Vertex,
                    logical_entry_point: "first_vertex".into(),
                    physical_entry_point: "physical_vertex_0".into(),
                },
                GpuCanvasShaderEntry {
                    stage: GpuCanvasShaderStage::Vertex,
                    logical_entry_point: "chosen_vertex".into(),
                    physical_entry_point: "physical_vertex_1".into(),
                },
                GpuCanvasShaderEntry {
                    stage: GpuCanvasShaderStage::Fragment,
                    logical_entry_point: "first_fragment".into(),
                    physical_entry_point: "physical_fragment_0".into(),
                },
                GpuCanvasShaderEntry {
                    stage: GpuCanvasShaderStage::Fragment,
                    logical_entry_point: "chosen_fragment".into(),
                    physical_entry_point: "physical_fragment_1".into(),
                },
            ],
            module: None,
        };

        let default = resolve_shader_entry(&shader, GpuCanvasShaderStage::Vertex, None, "vertex")
            .expect("bare shader selects the first declaration of its stage");
        assert_eq!(default.logical_entry_point, "first_vertex");
        assert_eq!(default.physical_entry_point, "physical_vertex_0");
        assert_eq!(
            resolve_shader_entry(&shader, GpuCanvasShaderStage::Vertex, Some(""), "vertex",)
                .expect("an empty entryPoint selects the first declaration like C++"),
            default,
        );

        let chosen = resolve_shader_entry(
            &shader,
            GpuCanvasShaderStage::Fragment,
            Some("chosen_fragment"),
            "fragment",
        )
        .expect("named selection resolves logical to physical");
        assert_eq!(chosen.logical_entry_point, "chosen_fragment");
        assert_eq!(chosen.physical_entry_point, "physical_fragment_1");

        let error = resolve_shader_entry(
            &shader,
            GpuCanvasShaderStage::Fragment,
            Some("missing"),
            "fragment",
        )
        .expect_err("unknown logical entry fails before renderer allocation");
        assert!(
            error
                .to_string()
                .contains("available: first_fragment, chosen_fragment"),
            "{error}",
        );
    }
}
