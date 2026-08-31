//! Live `src/lua/renderer/lua_gpu.cpp` owners at e949498e.
//! These userdata issue ORE operations; the separate direct-host plan collector
//! is not used to execute imported script GPU work.
use crate::vm::RendererBindings;
use luaur_rt::{
    AnyUserData, Buffer as LuaBuffer, Error, Lua, Result, Table, UserData, UserDataFields,
    UserDataMethods, Value,
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
#[cfg(all(test, feature = "compiler"))]
mod tests;
pub(super) use canvas::Canvas;
use enums::*;
use resources::*;
use shader::*;

fn context(lua: &Lua) -> Result<OreContextHandle> {
    RendererBindings::for_lua(lua)
        .and_then(|bindings| bindings.ore_context())
        .ok_or_else(|| Error::runtime("GPU context not initialized"))
}
fn number(table: &Table, key: &str, default: f64) -> Result<f64> {
    number_value(&table.lua(), table.get::<Value>(key)?, default)
}
fn number_value(lua: &Lua, value: Value, default: f64) -> Result<f64> {
    // lua_isnumber/lua_tonumber accept integral numbers and Lua numeric strings.
    Ok(lua.coerce_number(value)?.unwrap_or(default))
}
fn optional_table(table: &Table, key: &str) -> Result<Option<Table>> {
    Ok(match table.get::<Value>(key)? {
        Value::Table(value) => Some(value),
        _ => None,
    })
}
fn boolean(table: &Table, key: &str, default: bool) -> Result<bool> {
    Ok(match table.get::<Value>(key)? {
        Value::Boolean(value) => value,
        _ => default,
    })
}
fn string(table: &Table, key: &str) -> Result<Option<String>> {
    string_value(&table.lua(), table.get::<Value>(key)?)
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
    // SAFETY: argument 1 is a string or number, and the closure pushes exactly
    // one rooted result without removing any of exec_raw's arguments.
    let value: luaur_rt::LuaString = unsafe {
        lua.exec_raw(value, |state| {
            luaur_vm::functions::lua_tolstring::lua_tolstring(state, 1, std::ptr::null_mut());
            luaur_vm::functions::lua_pushvalue::lua_pushvalue(state, 1);
        })?
    };
    let bytes = value.as_bytes();
    let bytes = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
    std::str::from_utf8(bytes)
        .map(|value| Some(value.to_owned()))
        .map_err(|error| Error::runtime(error.to_string()))
}
fn checked_string(table: &Table, key: &str) -> Result<Option<String>> {
    let value = table.get::<Value>(key)?;
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
    if samples <= 1 {
        return Ok(());
    }
    if !samples.is_power_of_two() {
        return Err(Error::runtime(format!(
            "sampleCount must be a power of two (got {samples})"
        )));
    }
    if ctx.featuresKnown() && samples > ctx.features().maxSamples {
        return Err(Error::runtime(format!(
            "sampleCount {samples} exceeds device maximum of {} — query context:features().maxSamples before creating MSAA textures",
            ctx.features().maxSamples
        )));
    }
    Ok(())
}
fn constructor(
    lua: &Lua,
    name: &str,
    function: impl Fn(&Lua, Table) -> Result<AnyUserData> + 'static,
) -> Result<()> {
    let table = lua.create_table();
    table.set("new", lua.create_function(function)?)?;
    table.set_readonly(true);
    lua.globals().set(name, table)
}
pub(super) fn install(lua: &Lua) -> Result<()> {
    resources::install(lua)?;
    shader::install(lua)?;
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
    cached: &RefCell<Option<AnyResourceHandle>>,
) -> Result<AnyUserData> {
    let context =
        context(lua).map_err(|_| Error::runtime("GPU context not available for Image:view()"))?;
    if cached.borrow().is_none() {
        let mut ctx = context.borrow_mut();
        let view = if ctx.isRecording() {
            let view = if let Some(id) = image.deferred_image_id() {
                ctx.recordWrapImageView(id, image.width(), image.height())
            } else {
                ctx.recordWrapCanvasImage(nuxie_ore_metal::context::CanvasImageInfo {
                    identity: image.image_identity(),
                    width: image.width(),
                    height: image.height(),
                    owner: Rc::new(image.clone()),
                })
            };
            view.ok_or_else(|| Error::runtime("Image:view() recording failed"))?
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
        *cached.borrow_mut() = Some(view);
    }
    lua.create_userdata(TextureView {
        resource: cached.borrow().as_ref().expect("cached image view").clone(),
        retained_image: Some(image),
    })
}

pub(super) fn close_orphan_render_pass(bindings: &RendererBindings) -> Result<bool> {
    let Some(context) = bindings.ore_context() else {
        return Ok(false);
    };
    let pass = {
        context
            .borrow()
            .activeRenderPass()
            .and_then(|pass| pass.upgrade())
    };
    let Some(pass) = pass else { return Ok(false) };
    if pass.isFinished() {
        return Ok(false);
    }
    pass.finish();
    context.borrow().setActiveRenderPass(None);
    Ok(true)
}
