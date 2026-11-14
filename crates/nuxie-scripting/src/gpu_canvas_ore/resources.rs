//! Buffer, texture, texture-view and sampler userdata from lua_gpu.cpp.
use super::*;

#[derive(Clone)]
pub(super) struct Buffer {
    pub resource: AnyResourceHandle,
    pub immutable: bool,
    /// `size` bytes.
    pub charge: GpuCharge,
}
impl UserData for Buffer {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.resource.size().unwrap_or(0)));
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("write", |lua, this, (data, dst, src, length): (LuaBuffer, Value, Value, Value)| {
            if this.immutable { return Err(Error::runtime("GPUBuffer:write: buffer was created with immutable=true; its contents are fixed at construction")); }
            let dst = number_value(lua, dst, 0.0)? as u32; let src = number_value(lua, src, 0.0)? as u32;
            let bytes = data.to_vec();
            let Some(source) = bytes.get(src as usize..) else { return Err(Error::runtime(format!("GPUBuffer:write: srcOffset({src}) exceeds source buffer size({})", bytes.len()))); };
            let length = number_value(lua, length, source.len() as f64)? as u32;
            if u64::from(length) > source.len() as u64 { return Err(Error::runtime("GPUBuffer:write: source range exceeds source buffer size")); }
            let size = this.resource.size().unwrap_or(0);
            // Both operands are widened u32s, so the sum cannot overflow u64.
            if u64::from(dst).saturating_add(u64::from(length)) > u64::from(size) { return Err(Error::runtime("GPUBuffer:write: destination range exceeds buffer size")); }
            this.resource.update(source, length, dst).map_err(|error| Error::runtime(format!("GPUBuffer:write: {error:?}")))
        });
    }
}
#[derive(Clone)]
pub(super) struct Texture {
    pub resource: AnyResourceHandle,
    pub desc: TextureDesc<'static>,
    /// [`texture_bytes`] of `desc`.
    pub charge: GpuCharge,
}
#[derive(Clone)]
pub(crate) struct TextureView {
    pub resource: AnyResourceHandle,
    pub retained_image: Option<Rc<dyn nuxie_render_api::RenderImage>>,
    /// A view of a video's picture follows the video: the picture's userdata
    /// and the frame this view was made from.
    pub follows: Option<(AnyUserData, usize)>,
    /// The charge of the texture or canvas viewed, when the script made it:
    /// a view keeps its texture alive.
    pub charge: Option<GpuCharge>,
}
impl TextureView {
    /// Move a view of a video's picture to the frame on show. True when it moved.
    pub(crate) fn follow(lua: &Lua, view: &AnyUserData) -> Result<bool> {
        let Some((image, frame)) = view.borrow::<TextureView>()?.follows.clone() else {
            return Ok(false);
        };
        let image_ref = image.borrow::<crate::vm::lua_image::ScriptedImage>()?;
        if image_ref.frame_identity() == Some(frame) {
            return Ok(false);
        }
        let Some((fresh, frame)) = image_ref.video_texture_view(lua)? else {
            return Ok(false);
        };
        drop(image_ref);
        let mut current = view.borrow_mut::<TextureView>()?;
        current.resource = fresh.resource;
        current.retained_image = fresh.retained_image;
        current.follows = Some((image, frame));
        Ok(true)
    }

    pub fn format(&self) -> Result<TextureFormat> {
        view_format(&self.resource)
    }
    pub fn sample_count(&self) -> Result<u32> {
        viewed_texture(&self.resource)?
            .sampleCount()
            .ok_or_else(texture_unavailable)
    }
}

/// The texture a texture-view resource views. ORE answers `None` when the
/// handle is not a texture view or is read off its recording thread; both are
/// an ordinary script error.
pub(super) fn viewed_texture(view: &AnyResourceHandle) -> Result<&AnyResourceHandle> {
    view.textureViewBase()
        .map(|base| base.texture())
        .ok_or_else(|| Error::runtime("GPU texture view is unavailable"))
}
pub(super) fn view_format(view: &AnyResourceHandle) -> Result<TextureFormat> {
    viewed_texture(view)?
        .format()
        .ok_or_else(texture_unavailable)
}
fn texture_unavailable() -> Error {
    Error::runtime("GPU texture is unavailable")
}
impl UserData for TextureView {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| {
            Ok(viewed_texture(&this.resource)?.width().unwrap_or(0))
        });
        fields.add_field_method_get("height", |_, this| {
            Ok(viewed_texture(&this.resource)?.height().unwrap_or(0))
        });
        fields.add_field_method_get("format", |_, this| Ok(format_string(this.format()?)));
    }
}
#[derive(Clone)]
pub(super) struct Sampler {
    pub resource: AnyResourceHandle,
}
impl UserData for Sampler {}

impl UserData for Texture {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| Ok(this.desc.width));
        fields.add_field_method_get("height", |_, this| Ok(this.desc.height));
        fields.add_field_method_get("format", |_, this| Ok(format_string(this.desc.format)));
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_lent("view", |lua, this, mut loan, table: Value| {
            let loan = &mut loan;
            let mut desc = TextureViewDesc {
                texture: Some(&this.resource),
                mipCount: 0,
                layerCount: 0,
                dimension: match this.desc.r#type {
                    TextureType::texture2D => TextureViewDimension::texture2D,
                    TextureType::cube => TextureViewDimension::cube,
                    TextureType::texture3D => TextureViewDimension::texture3D,
                    TextureType::array2D => TextureViewDimension::array2D,
                },
                ..TextureViewDesc::default()
            };
            if let Value::Table(table) = table {
                if let Some(value) = string(&table, loan, "dimension")? {
                    desc.dimension = match value.as_str() {
                        "2d" => TextureViewDimension::texture2D,
                        "cube" => TextureViewDimension::cube,
                        "3d" => TextureViewDimension::texture3D,
                        "2d-array" => TextureViewDimension::array2D,
                        _ => desc.dimension,
                    };
                }
                desc.baseMipLevel = number(&table, loan, "baseMipLevel", 0.0)? as u32;
                desc.mipCount = number(&table, loan, "mipCount", 0.0)? as u32;
                desc.baseLayer = number(&table, loan, "baseLayer", 0.0)? as u32;
                desc.layerCount = number(&table, loan, "layerCount", 0.0)? as u32;
                if let Some(value) = string(&table, loan, "aspect")? {
                    desc.aspect = match value.as_str() {
                        "depth-only" => TextureAspect::depthOnly,
                        "stencil-only" => TextureAspect::stencilOnly,
                        "all" => TextureAspect::all,
                        _ => desc.aspect,
                    };
                }
            }
            let context = context(lua)?;
            let mut ctx = context.borrow_mut();
            ctx.clearLastError();
            let result = ctx.makeTextureView(&desc);
            let resource = resource_result(&*ctx, result, "GPUTexture:view", "texture view")?;
            // Release the context before allocating in the VM: a collection step there
            // can drop other GPU userdata, which must find the context free.
            drop(ctx);
            lua.create_userdata(TextureView {
                resource,
                retained_image: None,
                follows: None,
                charge: Some(Rc::clone(&this.charge)),
            })
        });
        methods.add_method_lent("upload", |_, this, mut loan, table: Table| {
            let loan = &mut loan;
            let data: LuaBuffer = table.get_with(reborrow_loan(loan), "data")?;
            let bytes = data.to_vec();
            let desc = TextureDataDesc {
                data: Some(&bytes),
                dataSize: bytes.len() as u32,
                width: number(&table, loan, "width", 0.0)? as u32,
                height: number(&table, loan, "height", 0.0)? as u32,
                depth: number(&table, loan, "depth", 1.0)? as u32,
                x: number(&table, loan, "x", 0.0)? as u32,
                y: number(&table, loan, "y", 0.0)? as u32,
                z: number(&table, loan, "z", 0.0)? as u32,
                mipLevel: number(&table, loan, "mipLevel", 0.0)? as u32,
                layer: number(&table, loan, "layer", 0.0)? as u32,
                bytesPerRow: number(&table, loan, "bytesPerRow", 0.0)? as u32,
                rowsPerImage: number(&table, loan, "rowsPerImage", 0.0)? as u32,
            };
            this.resource
                .upload(&desc)
                .map_err(|error| Error::runtime(error.to_string()))
        });
    }
}

pub(super) fn format_string(format: TextureFormat) -> &'static str {
    match format {
        TextureFormat::r8unorm => "r8unorm",
        TextureFormat::rg8unorm => "rg8unorm",
        TextureFormat::rgba8unorm => "rgba8unorm",
        TextureFormat::bgra8unorm => "bgra8unorm",
        TextureFormat::rgba16float => "rgba16float",
        TextureFormat::rg16float => "rg16float",
        TextureFormat::r16float => "r16float",
        TextureFormat::rgba32float => "rgba32float",
        TextureFormat::rg32float => "rg32float",
        TextureFormat::r32float => "r32float",
        TextureFormat::rgb10a2unorm => "rgb10a2unorm",
        TextureFormat::r11g11b10float => "rg11b10ufloat",
        TextureFormat::depth16unorm => "depth16unorm",
        TextureFormat::depth24plusStencil8 => "depth24plus-stencil8",
        TextureFormat::depth32float => "depth32float",
        TextureFormat::depth32floatStencil8 => "depth32float-stencil8",
        _ => "rgba8unorm",
    }
}

pub(super) fn install(lua: &Lua) -> Result<()> {
    constructor(lua, "GPUBuffer", |lua, loan, table| {
        let context = context(lua)?;
        let size = number(&table, loan, "size", 0.0)?;
        if !(size >= 1.0 && size <= u32::MAX as f64 && size == size as u32 as f64) {
            return Err(Error::runtime(
                "GPUBuffer.new: 'size' must be a positive integer number of bytes",
            ));
        }
        let usage = match table.get_with::<Value>(reborrow_loan(loan), "usage")? {
            Value::String(value) => buffer_usage(&value.to_str()?)?,
            Value::Table(values) => {
                if values.raw_len() != 1 {
                    return Err(Error::runtime(
                        "GPUBuffer.new: usage array must hold exactly one value; multiple usages are not yet supported",
                    ));
                }
                buffer_usage(&values.raw_get::<String>(1)?)?
            }
            _ => {
                return Err(Error::runtime(
                    "GPUBuffer.new: 'usage' is required (a string or array of strings)",
                ));
            }
        };
        let immutable = boolean(&table, loan, "immutable", false)?;
        let bytes = table
            .get_with::<Option<LuaBuffer>>(reborrow_loan(loan), "data")?
            .map(|data| data.to_vec());
        if bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() != size as usize)
        {
            return Err(Error::runtime("GPUBuffer.new: data length must equal size"));
        }
        if immutable && bytes.is_none() {
            return Err(Error::runtime(
                "GPUBuffer.new: immutable=true requires 'data' (the GPU buffer is GPU-only after creation)",
            ));
        }
        let label = string(&table, loan, "label")?;
        let desc = BufferDesc {
            usage,
            size: size as u32,
            data: bytes.as_deref(),
            immutable,
            label: label.as_deref(),
        };
        let charge = charge_gpu_bytes(lua, u64::from(desc.size))?;
        let mut ctx = context.borrow_mut();
        ctx.clearLastError();
        let value = ctx.makeBuffer(&desc);
        let resource = resource_result(&*ctx, value, "GPUBuffer.new", "buffer")?;
        // Release the context before allocating in the VM: a collection step there
        // can drop other GPU userdata, which must find the context free.
        drop(ctx);
        lua.create_userdata(Buffer {
            resource,
            immutable,
            charge,
        })
    })?;
    constructor(lua, "GPUTexture", |lua, loan, table| {
        let context = context(lua)?;
        let mut desc = TextureDesc {
            width: number(&table, loan, "width", 0.0)? as u32,
            height: number(&table, loan, "height", 0.0)? as u32,
            ..TextureDesc::default()
        };
        if desc.width == 0 || desc.height == 0 {
            return Err(Error::runtime("GPUTexture requires width and height"));
        }
        check_gpu_size("GPUTexture.new", desc.width, desc.height, 1)?;
        if let Some(value) = string(&table, loan, "format")? {
            desc.format = texture_format(&value)?;
        }
        {
            let ctx = context.borrow();
            check_guard(|error| {
                nuxie_ore_metal::script_guards::validateTextureFormat(
                    Some(&*ctx),
                    desc.format,
                    Some(error),
                )
            })?;
        }
        if let Some(value) = string(&table, loan, "type")? {
            desc.r#type = texture_type(&value)?;
        }
        desc.renderTarget = boolean(&table, loan, "renderTarget", false)?;
        desc.sampleCount = number(&table, loan, "sampleCount", 1.0)? as u32;
        check_sample_count(&*context.borrow(), desc.sampleCount)?;
        desc.numMipmaps = number(&table, loan, "mipmaps", 1.0)? as u32;
        desc.depthOrArrayLayers = number(&table, loan, "layers", 1.0)? as u32;
        check_gpu_size("GPUTexture.new", desc.width, desc.height, desc.depthOrArrayLayers)?;
        let charge = charge_gpu_bytes(lua, texture_bytes(&desc))?;
        // Lua field access can invoke __index, including another ORE call.
        // Borrow the selected context only after those source field reads.
        let mut ctx = context.borrow_mut();
        if desc.renderTarget && ctx.featuresKnown() {
            let cap = match desc.format {
                TextureFormat::rgba16float | TextureFormat::rg16float | TextureFormat::r16float
                    if !ctx.features().colorBufferHalfFloat =>
                {
                    Some("colorBufferHalfFloat")
                }
                TextureFormat::rgba32float
                | TextureFormat::rg32float
                | TextureFormat::r32float
                | TextureFormat::r11g11b10float
                    if !ctx.features().colorBufferFloat =>
                {
                    Some("colorBufferFloat")
                }
                _ => None,
            };
            if let Some(cap) = cap {
                return Err(Error::runtime(format!(
                    "GPUTexture.new: float format {} as a renderTarget requires the {cap} feature, which the active backend does not support",
                    format_string(desc.format)
                )));
            }
        }
        ctx.clearLastError();
        let value = ctx.makeTexture(&desc);
        let resource = resource_result(&*ctx, value, "GPUTexture.new", "texture")?;
        // Release the context before allocating in the VM: a collection step there
        // can drop other GPU userdata, which must find the context free.
        drop(ctx);
        lua.create_userdata(Texture {
            resource,
            desc,
            charge,
        })
    })?;
    let sampler = lua.create_table();
    sampler.set(
        "new",
        lua.create_function_lent(|lua, mut loan, table: Value| {
            let loan = &mut loan;
            let context = context(lua)?;
            let mut desc = SamplerDesc::default();
            if let Value::Table(table) = table {
                if let Some(value) = string(&table, loan, "min")? {
                    desc.minFilter = filter(&value)?;
                }
                if let Some(value) = string(&table, loan, "mag")? {
                    desc.magFilter = filter(&value)?;
                }
                if let Some(value) = string(&table, loan, "mipmap")? {
                    desc.mipmapFilter = filter(&value)?;
                }
                if let Some(value) = string(&table, loan, "wrapU")? {
                    desc.wrapU = wrap_mode(&value)?;
                }
                if let Some(value) = string(&table, loan, "wrapV")? {
                    desc.wrapV = wrap_mode(&value)?;
                }
                if let Some(value) = string(&table, loan, "wrapW")? {
                    desc.wrapW = wrap_mode(&value)?;
                }
                if let Some(value) = string(&table, loan, "compare")? {
                    desc.compare = compare(&value)?;
                }
                desc.minLod = number(&table, loan, "minLod", 0.0)? as f32;
                desc.maxLod = number(&table, loan, "maxLod", 32.0)? as f32;
                desc.maxAnisotropy = number(&table, loan, "maxAnisotropy", 1.0)? as u32;
                let ctx = context.borrow();
                check_guard(|error| {
                    nuxie_ore_metal::script_guards::validateSamplerDesc(
                        Some(&*ctx),
                        &desc,
                        Some(error),
                    )
                })?;
            }
            let mut ctx = context.borrow_mut();
            ctx.clearLastError();
            let value = ctx.makeSampler(&desc);
            let resource = resource_result(&*ctx, value, "GPUSampler.new", "sampler")?;
            // Release the context before allocating in the VM: a collection step there
            // can drop other GPU userdata, which must find the context free.
            drop(ctx);
            lua.create_userdata(Sampler { resource })
        })?,
    )?;
    sampler.set_readonly(true);
    lua.globals().set("GPUSampler", sampler)
}
