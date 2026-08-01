//! Rive's Luau math registration, mirroring `math/lua_math.cpp`.

use luaur_rt::{Lua, Result, Table};

use super::lua_mat4::install_mat4_global;
use super::lua_vec2d::install_vector_global;

pub(super) fn install_math_globals(lua: &Lua) -> Result<()> {
    install_vector_global(lua)?;
    install_mat4_global(lua)?;
    install_math_fround(lua)
}

fn install_math_fround(lua: &Lua) -> Result<()> {
    let math: Table = lua.globals().get("math")?;
    math.set(
        "fround",
        lua.create_function(|_, value: f64| Ok(f64::from(value as f32)))?,
    )
}
