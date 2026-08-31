//! Buffer, texture, texture-view and sampler userdata from lua_gpu.cpp.
use super::*;

#[derive(Clone)]
pub(super) struct Buffer {
    pub resource: AnyResourceHandle,
    pub immutable: bool,
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
            if src as usize > bytes.len() { return Err(Error::runtime(format!("GPUBuffer:write: srcOffset({src}) exceeds source buffer size({})", bytes.len()))); }
            let length = number_value(lua, length, (bytes.len() - src as usize) as f64)? as u32;
            if u64::from(src) + u64::from(length) > bytes.len() as u64 { return Err(Error::runtime("GPUBuffer:write: source range exceeds source buffer size")); }
            let size = this.resource.size().unwrap_or(0);
            if u64::from(dst) + u64::from(length) > u64::from(size) { return Err(Error::runtime("GPUBuffer:write: destination range exceeds buffer size")); }
            this.resource.update(&bytes[src as usize..], length, dst).map_err(|error| Error::runtime(format!("GPUBuffer:write: {error:?}")))
        });
    }
}
#[derive(Clone)]
pub(super) struct Texture {
    pub resource: AnyResourceHandle,
    pub desc: TextureDesc<'static>,
}
#[derive(Clone)]
pub(super) struct TextureView {
    pub resource: AnyResourceHandle,
    pub retained_image: Option<Rc<dyn nuxie_render_api::RenderImage>>,
}
impl TextureView {
    pub fn format(&self) -> TextureFormat {
        self.resource
            .textureViewBase()
            .expect("texture view")
            .texture()
            .format()
            .expect("texture")
    }
    pub fn sample_count(&self) -> u32 {
        self.resource
            .textureViewBase()
            .expect("texture view")
            .texture()
            .sampleCount()
            .expect("texture")
    }
}
impl UserData for TextureView {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| {
            Ok(this
                .resource
                .textureViewBase()
                .expect("texture view")
                .texture()
                .width()
                .unwrap_or(0))
        });
        fields.add_field_method_get("height", |_, this| {
            Ok(this
                .resource
                .textureViewBase()
                .expect("texture view")
                .texture()
                .height()
                .unwrap_or(0))
        });
        fields.add_field_method_get("format", |_, this| Ok(format_string(this.format())));
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
        methods.add_method("view", |lua, this, table: Value| {
            let mut desc = TextureViewDesc {
                texture: Some(&this.resource),
                mipCount: this.desc.numMipmaps,
                layerCount: this.desc.depthOrArrayLayers,
                dimension: match this.desc.r#type {
                    TextureType::texture2D => TextureViewDimension::texture2D,
                    TextureType::cube => TextureViewDimension::cube,
                    TextureType::texture3D => TextureViewDimension::texture3D,
                    TextureType::array2D => TextureViewDimension::array2D,
                },
                ..TextureViewDesc::default()
            };
            if let Value::Table(table) = table {
                if let Some(value) = string(&table, "dimension")? {
                    desc.dimension = match value.as_str() {
                        "2d" => TextureViewDimension::texture2D,
                        "cube" => TextureViewDimension::cube,
                        "3d" => TextureViewDimension::texture3D,
                        "2d-array" => TextureViewDimension::array2D,
                        _ => desc.dimension,
                    };
                }
                desc.baseMipLevel = number(&table, "baseMipLevel", 0.0)? as u32;
                desc.mipCount = number(&table, "mipCount", desc.mipCount as f64)? as u32;
                desc.baseLayer = number(&table, "baseLayer", 0.0)? as u32;
                desc.layerCount = number(&table, "layerCount", desc.layerCount as f64)? as u32;
            }
            let context = context(lua)?;
            let mut ctx = context.borrow_mut();
            ctx.clearLastError();
            let result = ctx.makeTextureView(&desc);
            let resource = resource_result(&*ctx, result, "GPUTexture:view", "texture view")?;
            lua.create_userdata(TextureView {
                resource,
                retained_image: None,
            })
        });
        methods.add_method("upload", |_, this, table: Table| {
            let data: LuaBuffer = table.get("data")?;
            let bytes = data.to_vec();
            let mut desc = TextureDataDesc {
                data: Some(&bytes),
                width: number(&table, "width", this.desc.width as f64)? as u32,
                height: number(&table, "height", this.desc.height as f64)? as u32,
                depth: number(&table, "depth", 1.0)? as u32,
                x: number(&table, "x", 0.0)? as u32,
                y: number(&table, "y", 0.0)? as u32,
                z: number(&table, "z", 0.0)? as u32,
                mipLevel: number(&table, "mipLevel", 0.0)? as u32,
                layer: number(&table, "layer", 0.0)? as u32,
                bytesPerRow: number(&table, "bytesPerRow", 0.0)? as u32,
                rowsPerImage: number(&table, "rowsPerImage", 0.0)? as u32,
            };
            if desc.mipLevel >= this.desc.numMipmaps {
                return Err(Error::runtime(format!(
                    "upload: mipLevel {} out of range [0, {})",
                    desc.mipLevel, this.desc.numMipmaps
                )));
            }
            if desc.layer >= this.desc.depthOrArrayLayers {
                return Err(Error::runtime(format!(
                    "upload: layer {} out of range [0, {})",
                    desc.layer, this.desc.depthOrArrayLayers
                )));
            }
            let width = this
                .desc
                .width
                .checked_shr(desc.mipLevel)
                .unwrap_or(0)
                .max(1);
            let height = this
                .desc
                .height
                .checked_shr(desc.mipLevel)
                .unwrap_or(0)
                .max(1);
            if desc.x > width || desc.width > width - desc.x {
                return Err(Error::runtime("upload: x+width exceeds mip width"));
            }
            if desc.y > height || desc.height > height - desc.y {
                return Err(Error::runtime("upload: y+height exceeds mip height"));
            }
            if desc.bytesPerRow == 0 {
                let bpt = textureFormatBytesPerTexel(this.desc.format);
                if bpt == 0 {
                    return Err(Error::runtime(
                        "upload: bytesPerRow must be provided for block-compressed formats",
                    ));
                }
                desc.bytesPerRow = desc.width.wrapping_mul(bpt);
            }
            if desc.rowsPerImage == 0 {
                desc.rowsPerImage = desc.height;
            }
            let required = u64::from(desc.bytesPerRow)
                * u64::from(desc.rowsPerImage)
                * u64::from(desc.depth.max(1));
            if (bytes.len() as u64) < required {
                return Err(Error::runtime(format!(
                    "upload: data buffer is {} bytes but region requires {required}",
                    bytes.len()
                )));
            }
            this.resource
                .upload(&desc)
                .map_err(|error| Error::runtime(format!("upload: {error:?}")))
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
    constructor(lua, "GPUBuffer", |lua, table| {
        let context = context(lua)?;
        let size = number(&table, "size", 0.0)?;
        if !(size >= 1.0 && size <= u32::MAX as f64 && size == size as u32 as f64) {
            return Err(Error::runtime(
                "GPUBuffer.new: 'size' must be a positive integer number of bytes",
            ));
        }
        let usage = match table.get::<Value>("usage")? {
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
        let immutable = boolean(&table, "immutable", false)?;
        let bytes = table
            .get::<Option<LuaBuffer>>("data")?
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
        let label = string(&table, "label")?;
        let desc = BufferDesc {
            usage,
            size: size as u32,
            data: bytes.as_deref(),
            immutable,
            label: label.as_deref(),
        };
        let mut ctx = context.borrow_mut();
        ctx.clearLastError();
        let value = ctx.makeBuffer(&desc);
        let resource = resource_result(&*ctx, value, "GPUBuffer.new", "buffer")?;
        lua.create_userdata(Buffer {
            resource,
            immutable,
        })
    })?;
    constructor(lua, "GPUTexture", |lua, table| {
        let context = context(lua)?;
        let mut desc = TextureDesc {
            width: number(&table, "width", 0.0)? as u32,
            height: number(&table, "height", 0.0)? as u32,
            ..TextureDesc::default()
        };
        if desc.width == 0 || desc.height == 0 {
            return Err(Error::runtime("GPUTexture requires width and height"));
        }
        if let Some(value) = string(&table, "format")? {
            desc.format = texture_format(&value)?;
        }
        if let Some(value) = string(&table, "type")? {
            desc.r#type = texture_type(&value)?;
        }
        desc.renderTarget = boolean(&table, "renderTarget", false)?;
        desc.sampleCount = number(&table, "sampleCount", 1.0)? as u32;
        check_sample_count(&*context.borrow(), desc.sampleCount)?;
        desc.numMipmaps = number(&table, "mipmaps", 1.0)? as u32;
        desc.depthOrArrayLayers = number(&table, "layers", 1.0)? as u32;
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
        lua.create_userdata(Texture { resource, desc })
    })?;
    let sampler = lua.create_table();
    sampler.set("new",lua.create_function(|lua,table:Value| {
        let context=context(lua)?;let mut desc=SamplerDesc::default();
        if let Value::Table(table)=table {
            if let Some(value)=string(&table,"min")? {desc.minFilter=filter(&value)?;}
            if let Some(value)=string(&table,"mag")? {desc.magFilter=filter(&value)?;}
            if let Some(value)=string(&table,"mipmap")? {desc.mipmapFilter=filter(&value)?;}
            if let Some(value)=string(&table,"wrapU")? {desc.wrapU=wrap_mode(&value)?;}
            if let Some(value)=string(&table,"wrapV")? {desc.wrapV=wrap_mode(&value)?;}
            if let Some(value)=string(&table,"wrapW")? {desc.wrapW=wrap_mode(&value)?;}
            if let Some(value)=string(&table,"compare")? {desc.compare=compare(&value)?;}
            desc.minLod=number(&table,"minLod",0.0)? as f32;desc.maxLod=number(&table,"maxLod",32.0)? as f32;
            if desc.minLod>desc.maxLod {return Err(Error::runtime(format!("GPUSampler.new: minLod ({}) > maxLod ({})",desc.minLod,desc.maxLod)));}
            desc.maxAnisotropy=number(&table,"maxAnisotropy",1.0)? as u32;
            if desc.maxAnisotropy<1 || desc.maxAnisotropy>16 || !desc.maxAnisotropy.is_power_of_two() {return Err(Error::runtime(format!("GPUSampler.new: maxAnisotropy must be a power of two in [1, 16] (got {})",desc.maxAnisotropy)));}
            let ctx=context.borrow();
            if desc.maxAnisotropy>1 && ctx.featuresKnown() && !ctx.features().anisotropicFiltering {return Err(Error::runtime(format!("GPUSampler.new: maxAnisotropy={} requires anisotropicFiltering feature, which the active backend does not support",desc.maxAnisotropy)));}
        }
        let mut ctx=context.borrow_mut();
        ctx.clearLastError();let value=ctx.makeSampler(&desc);let resource=resource_result(&*ctx,value,"GPUSampler.new","sampler")?;
        lua.create_userdata(Sampler {resource})
    })?)?;
    sampler.set_readonly(true);
    lua.globals().set("GPUSampler", sampler)
}
