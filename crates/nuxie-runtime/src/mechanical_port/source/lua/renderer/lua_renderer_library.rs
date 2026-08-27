#[cfg(feature = "rive_scripting")]
use crate::mechanical_port::source::lua::rive_lua_libs::{LuaFunction, LuaState};

#[cfg(feature = "rive_scripting")]
use super::{
    lua_blob::luaopen_rive_blob, lua_gradient::luaopen_rive_gradient,
    lua_image::luaopen_rive_image, lua_mesh::luaopen_rive_mesh, lua_paint::luaopen_rive_paint,
    lua_path::luaopen_rive_path, lua_renderer::luaopen_rive_renderer,
};

#[cfg(all(
    feature = "rive_scripting",
    feature = "rive_canvas",
    feature = "rive_ore"
))]
use super::lua_gpu::luaopen_rive_gpu;

#[cfg(feature = "rive_scripting")]
pub fn luaopen_rive_renderer_library(state: &mut LuaState) -> i32 {
    let mut renderer_types: Vec<LuaFunction> = vec![
        luaopen_rive_path,
        luaopen_rive_gradient,
        luaopen_rive_mesh,
        luaopen_rive_image,
        luaopen_rive_blob,
        luaopen_rive_paint,
        luaopen_rive_renderer,
    ];
    #[cfg(all(feature = "rive_canvas", feature = "rive_ore"))]
    renderer_types.push(luaopen_rive_gpu);
    renderer_types.into_iter().map(|open| open(state)).sum()
}
