//! Live `src/lua/renderer/lua_gpu.cpp` owners at e949498e.
//! These userdata issue ORE operations; the separate direct-host plan collector
//! is not used to execute imported script GPU work.
use crate::vm::RendererBindings;
use crate::vm::host_memory_budget::HostMemoryCharge;
use crate::luau::{
    AnyUserData, Buffer as LuaBuffer, Error, Loan, Lua, Result, Table, UserData, UserDataFields,
    UserDataMethods, Value, reborrow_loan,
};
use nuxie_ore_metal::{context::ContextApi, gpu_resource::AnyResourceHandle, types::*};
use nuxie_render_api::OreContextHandle;
use std::{cell::RefCell, rc::Rc};

mod canvas;
mod enums;
mod pass;
mod pipeline;
mod resources;
pub(super) mod shader;
mod target;
pub(crate) use target::target_userdata;
#[cfg(all(test, feature = "compiler"))]
mod tests;
pub(super) use canvas::Canvas;
use enums::*;
pub(crate) use resources::TextureView;
use resources::*;
use shader::*;

#[cfg(test)]
pub(crate) fn view_aspect(value: AnyUserData) -> Result<i32> {
    Ok(value
        .borrow::<TextureView>()?
        .resource
        .textureViewBase()
        .expect("texture view")
        .aspect() as i32)
}

// The direct-host plan collector shares the live Lua enum mapping and ORE
// descriptor validation without manufacturing a backend resource.
pub(super) fn texture_format_for_host(value: &str) -> Result<TextureFormat> {
    texture_format(value)
}
pub(super) fn texture_type_for_host(value: &str) -> Result<TextureType> {
    texture_type(value)
}

/// Largest width or height of a canvas or texture a script creates: the
/// image decoder's per-side limit (`nuxie_image_codec::MAX_IMAGE_DIMENSION`).
pub(crate) const MAX_SCRIPT_GPU_DIMENSION: u32 = 8_192;

/// Most pixels in one canvas or texture a script creates (width x height x
/// layers): the image decoder's 64 MiB RGBA ceiling
/// (`nuxie_image_codec::MAX_DECODED_IMAGE_BYTES`) at 4 bytes per pixel.
pub(crate) const MAX_SCRIPT_GPU_PIXELS: u64 = 16_777_216;

/// Refuses a canvas or texture size over the fixed caps, before anything is
/// charged or allocated.
pub(crate) fn check_gpu_size(caller: &str, width: u32, height: u32, layers: u32) -> Result<()> {
    if width > MAX_SCRIPT_GPU_DIMENSION || height > MAX_SCRIPT_GPU_DIMENSION {
        return Err(Error::runtime(format!(
            "{caller}: width and height must be at most {MAX_SCRIPT_GPU_DIMENSION} (got {width}x{height})"
        )));
    }
    let pixels = u64::from(width)
        .saturating_mul(u64::from(height))
        .saturating_mul(u64::from(layers.max(1)));
    if pixels > MAX_SCRIPT_GPU_PIXELS {
        return Err(Error::runtime(format!(
            "{caller}: {width}x{height}x{layers} is more than {MAX_SCRIPT_GPU_PIXELS} pixels"
        )));
    }
    Ok(())
}

/// The VM's host-memory charge for one GPU allocation. GPU memory and host
/// memory share the VM's one budget. Every script-visible owner of the
/// allocation (its userdata, the views and bind groups made from it, a
/// canvas's image) holds a clone, so the bytes are refunded when the last of
/// them is collected.
pub(crate) type GpuCharge = Rc<HostMemoryCharge>;

/// Charges `bytes` of GPU memory to the VM's budget, before the backend is
/// asked for them.
pub(crate) fn charge_gpu_bytes(lua: &Lua, bytes: u64) -> Result<GpuCharge> {
    let bytes = usize::try_from(bytes).unwrap_or(usize::MAX);
    HostMemoryCharge::for_lua(lua, bytes).map(Rc::new)
}

/// Bytes a canvas's backing takes: one RGBA8 colour texture.
pub(crate) fn canvas_bytes(width: u32, height: u32) -> u64 {
    u64::from(width)
        .saturating_mul(u64::from(height))
        .saturating_mul(4)
}

/// An upper bound of the bytes the backend allocates for `desc`: every mip
/// level of every layer and sample. A 3D texture's depth is charged at every
/// level as if it did not shrink.
pub(super) fn texture_bytes(desc: &TextureDesc<'_>) -> u64 {
    // (block width, block height, bytes per block)
    let (block_width, block_height, block_bytes) = match desc.format {
        TextureFormat::bc1unorm | TextureFormat::etc2rgb8 => (4, 4, 8),
        TextureFormat::bc3unorm
        | TextureFormat::bc7unorm
        | TextureFormat::etc2rgba8
        | TextureFormat::astc4x4 => (4, 4, 16),
        TextureFormat::astc6x6 => (6, 6, 16),
        TextureFormat::astc8x8 => (8, 8, 16),
        format => (1, 1, u64::from(textureFormatBytesPerTexel(format))),
    };
    let layers = match desc.r#type {
        TextureType::cube => desc.depthOrArrayLayers.max(6),
        _ => desc.depthOrArrayLayers.max(1),
    };
    let levels = desc.numMipmaps.max(1);
    let mut level_bytes_sum: u64 = 0;
    for level in 0..levels {
        let width = (desc.width >> level.min(31)).max(1);
        let height = (desc.height >> level.min(31)).max(1);
        let blocks = u64::from(width.div_ceil(block_width))
            .saturating_mul(u64::from(height.div_ceil(block_height)));
        level_bytes_sum = level_bytes_sum.saturating_add(blocks.saturating_mul(block_bytes));
        if width == 1 && height == 1 {
            // Every later level is one block too.
            let rest = u64::from(levels.saturating_sub(level).saturating_sub(1));
            level_bytes_sum = level_bytes_sum.saturating_add(rest.saturating_mul(block_bytes));
            break;
        }
    }
    level_bytes_sum
        .saturating_mul(u64::from(layers))
        .saturating_mul(u64::from(desc.sampleCount.max(1)))
}

fn context(lua: &Lua) -> Result<OreContextHandle> {
    RendererBindings::for_lua(lua)
        .and_then(|bindings| bindings.ore_context())
        .ok_or_else(|| Error::runtime("GPU context not initialized"))
}
// Descriptor reads honor `__index`, and pass on the loan of the call that
// reads them, so a metamethod reached during a draw can still draw.
fn number(table: &Table, loan: &mut Loan<'_>, key: &str, default: f64) -> Result<f64> {
    number_value(&table.lua(), table.get_with::<Value>(reborrow_loan(loan), key)?, default)
}
fn number_value(lua: &Lua, value: Value, default: f64) -> Result<f64> {
    // lua_isnumber/lua_tonumber accept integral numbers and Lua numeric strings.
    Ok(lua.coerce_number(value)?.unwrap_or(default))
}
fn optional_table(table: &Table, loan: &mut Loan<'_>, key: &str) -> Result<Option<Table>> {
    Ok(match table.get_with::<Value>(reborrow_loan(loan), key)? {
        Value::Table(value) => Some(value),
        _ => None,
    })
}
fn boolean(table: &Table, loan: &mut Loan<'_>, key: &str, default: bool) -> Result<bool> {
    Ok(match table.get_with::<Value>(reborrow_loan(loan), key)? {
        Value::Boolean(value) => value,
        _ => default,
    })
}
pub(super) fn string(table: &Table, loan: &mut Loan<'_>, key: &str) -> Result<Option<String>> {
    string_value(&table.lua(), table.get_with::<Value>(reborrow_loan(loan), key)?)
}
fn string_value(lua: &Lua, value: Value) -> Result<Option<String>> {
    if !matches!(
        value,
        Value::String(_) | Value::Integer(_) | Value::Number(_)
    ) {
        return Ok(None);
    }
    // luaur's FromLua<String> formats numbers with Rust's formatter, while the
    // source uses lua_tostring. Retain the converted Lua string as the result.
    // Seam row R31: `lua_tolstring` through the safe VM's `coerce_string`.
    let Some(value) = lua.coerce_string(value)? else {
        return Ok(None);
    };
    let bytes = value.as_bytes();
    let bytes = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
    std::str::from_utf8(bytes)
        .map(|value| Some(value.to_owned()))
        .map_err(|error| Error::runtime(error.to_string()))
}
fn checked_string(table: &Table, loan: &mut Loan<'_>, key: &str) -> Result<Option<String>> {
    let value = table.get_with::<Value>(reborrow_loan(loan), key)?;
    if matches!(value, Value::Nil) {
        return Ok(None);
    }
    string_value(&table.lua(), value)?
        .map(Some)
        .ok_or_else(|| Error::runtime(format!("{key}: expected string")))
}
fn resource_result(
    ctx: &dyn ContextApi,
    value: Option<AnyResourceHandle>,
    caller: &str,
    noun: &str,
) -> Result<AnyResourceHandle> {
    value.ok_or_else(|| {
        let error = ctx.lastError();
        Error::runtime(if error.is_empty() {
            format!("{caller}: failed to create {noun}")
        } else {
            format!("{caller}: {error}")
        })
    })
}
fn check_sample_count(ctx: &dyn ContextApi, samples: u32) -> Result<()> {
    check_guard(|error| {
        nuxie_ore_metal::script_guards::validateSampleCount(Some(ctx), samples, Some(error))
    })
}
// Rust errors unwind owned strings normally; unlike luaL_error's C longjmp,
// this boundary does not require a temporary character array for destruction.
fn check_guard(guard: impl FnOnce(&mut String) -> bool) -> Result<()> {
    let mut error = String::new();
    if !guard(&mut error) && !error.is_empty() {
        // The source carries at most 255 message bytes to luaL_error.
        let end = error
            .char_indices()
            .map(|(index, _)| index)
            .take_while(|index| *index <= 255)
            .last()
            .unwrap_or(0);
        if error.len() > 255 {
            error.truncate(end);
        }
        Err(Error::runtime(error))
    } else {
        Ok(())
    }
}
fn constructor(
    lua: &Lua,
    name: &str,
    function: impl Fn(&Lua, &mut Loan<'_>, Table) -> Result<AnyUserData> + 'static,
) -> Result<()> {
    let table = lua.create_table();
    table.set(
        "new",
        lua.create_function_lent(move |lua, mut loan, descriptor: Table| {
            function(lua, &mut loan, descriptor)
        })?,
    )?;
    table.set_readonly(true);
    lua.globals().set(name, table)
}
pub(super) fn install(lua: &Lua) -> Result<()> {
    resources::install(lua)?;
    shader::install(lua)?;
    let target = lua.create_table();
    target.set_readonly(true);
    lua.globals().set("GPUTarget", target)?;
    pipeline::install(lua)
}

pub(super) fn shader_userdata(
    lua: &Lua,
    shader: crate::gpu_canvas::GpuShader,
) -> Result<Option<AnyUserData>> {
    match shader::shader_from_existing(shader) {
        Ok(shader) => lua.create_userdata(shader).map(Some),
        Err(_) => Ok(None),
    }
}

pub(crate) fn image_view(
    lua: &Lua,
    image: Rc<dyn nuxie_render_api::RenderImage>,
    source_canvas: Option<&nuxie_render_api::RenderCanvasHandle>,
    cached: &RefCell<Option<AnyResourceHandle>>,
    charge: Option<GpuCharge>,
) -> Result<AnyUserData> {
    let mut view = image_texture_view(lua, image, source_canvas, cached)?;
    view.charge = charge;
    lua.create_userdata(view)
}

pub(crate) fn image_texture_view(
    lua: &Lua,
    image: Rc<dyn nuxie_render_api::RenderImage>,
    source_canvas: Option<&nuxie_render_api::RenderCanvasHandle>,
    cached: &RefCell<Option<AnyResourceHandle>>,
) -> Result<TextureView> {
    let context =
        context(lua).map_err(|_| Error::runtime("GPU context not available for Image:view()"))?;
    let existing = cached.borrow().clone();
    let resource = if let Some(view) = existing {
        view
    } else {
        let mut ctx = context.borrow_mut();
        #[allow(unsafe_code)]
        let view = if ctx.isRecording() {
            // Canvas provenance is authoritative: canvas backings can share
            // the same renderer type as decoded images. Recording must not
            // touch the driver; foreign images resolve through the registry.
            let view = if let Some(source_canvas) = source_canvas {
                ctx.recordWrapCanvasImage(nuxie_render_api::canvas_texture_info(source_canvas))
            } else if let Some(id) = image.deferred_image_id() {
                ctx.recordWrapImageView(id, image.width(), image.height())
            } else {
                ctx.recordWrapForeignImageView(nuxie_ore_metal::context::CanvasImageInfo {
                    identity: image.image_identity(),
                    width: image.width(),
                    height: image.height(),
                    owner: Rc::new(image.clone()),
                })
            };
            view.ok_or_else(|| Error::runtime("Image:view() recording failed"))?
        } else if let Some(source_canvas) = source_canvas {
            // A canvas imports through the backend's own sampling wrap.
            unsafe {
                ctx.wrapCanvasSampleView(nuxie_render_api::canvas_texture_info(source_canvas))
            }
            .ok_or_else(|| Error::runtime("Image:view() not supported on this backend"))?
        } else {
            let info = image
                .ore_texture_info()
                .ok_or_else(|| Error::runtime("Image is not a GPU-backed RiveRenderImage"))?;
            if info.texture.is_null() {
                return Err(Error::runtime("Image GPU texture not available"));
            }
            unsafe { ctx.wrapImageSampleView(info) }
                .ok_or_else(|| Error::runtime("Image:view() not supported on this backend"))?
        };
        *cached.borrow_mut() = Some(view.clone());
        view
    };
    Ok(TextureView {
        resource,
        retained_image: Some(image),
        follows: None,
        charge: None,
    })
}

pub(super) fn close_orphan_render_pass(bindings: &RendererBindings, token: u64) -> Result<bool> {
    let Some(context) = bindings.ore_context() else {
        return Ok(false);
    };
    // Finishing an inline recording replays through this same context. Release
    // its host borrow before invoking pass callbacks, as the native owner does.
    let registry = context.borrow().contextBase().openRenderPassRegistry();
    Ok(registry.finishOpenRenderPassesFrom(token) != 0)
}
