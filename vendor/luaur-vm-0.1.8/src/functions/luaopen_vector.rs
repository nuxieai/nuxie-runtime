//! Node: `cxx:Function:Luau.VM:VM/src/lveclib.cpp:341:luaopen_vector`
//! Source: `VM/src/lveclib.cpp:301-359` (hand-ported)

use crate::functions::createmetatable_lveclib::createmetatable;
use crate::functions::lua_l_register::lua_l_register;
use crate::functions::lua_pushvector_lapi::lua_pushvector_lua_state_f32_f32_f32_f32;
use crate::functions::lua_pushvector_lapi_alt_b::lua_pushvector_lua_state_f32_f32_f32;
use crate::functions::lua_setfield::lua_setfield;
use crate::functions::vector_abs::vector_abs;
use crate::functions::vector_angle::vector_angle;
use crate::functions::vector_ceil::vector_ceil;
use crate::functions::vector_clamp::vector_clamp;
use crate::functions::vector_create::vector_create;
use crate::functions::vector_cross::vector_cross;
use crate::functions::vector_dot::vector_dot;
use crate::functions::vector_floor::vector_floor;
use crate::functions::vector_lerp::vector_lerp;
use crate::functions::vector_magnitude::vector_magnitude;
use crate::functions::vector_max::vector_max;
use crate::functions::vector_min::vector_min;
use crate::functions::vector_normalize::vector_normalize;
use crate::functions::vector_sign::vector_sign;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::records::lua_l_reg::LuaLReg;
use crate::type_aliases::lua_state::lua_State;

struct VectorFuncs([LuaLReg; 15]);
unsafe impl Sync for VectorFuncs {}

static VECTOR_FUNCS: VectorFuncs = VectorFuncs([
    LuaLReg {
        name: c"create".as_ptr(),
        func: Some(vector_create),
    },
    LuaLReg {
        name: c"magnitude".as_ptr(),
        func: Some(vector_magnitude),
    },
    LuaLReg {
        name: c"normalize".as_ptr(),
        func: Some(vector_normalize),
    },
    LuaLReg {
        name: c"cross".as_ptr(),
        func: Some(vector_cross),
    },
    LuaLReg {
        name: c"dot".as_ptr(),
        func: Some(vector_dot),
    },
    LuaLReg {
        name: c"angle".as_ptr(),
        func: Some(vector_angle),
    },
    LuaLReg {
        name: c"floor".as_ptr(),
        func: Some(vector_floor),
    },
    LuaLReg {
        name: c"ceil".as_ptr(),
        func: Some(vector_ceil),
    },
    LuaLReg {
        name: c"abs".as_ptr(),
        func: Some(vector_abs),
    },
    LuaLReg {
        name: c"sign".as_ptr(),
        func: Some(vector_sign),
    },
    LuaLReg {
        name: c"clamp".as_ptr(),
        func: Some(vector_clamp),
    },
    LuaLReg {
        name: c"max".as_ptr(),
        func: Some(vector_max),
    },
    LuaLReg {
        name: c"min".as_ptr(),
        func: Some(vector_min),
    },
    LuaLReg {
        name: c"lerp".as_ptr(),
        func: Some(vector_lerp),
    },
    LuaLReg {
        name: core::ptr::null(),
        func: None,
    },
]);

#[allow(non_snake_case)]
pub unsafe fn luaopen_vector(L: *mut lua_State) -> core::ffi::c_int {
    lua_l_register(L, c"vector".as_ptr(), VECTOR_FUNCS.0.as_ptr());

    if LUA_VECTOR_SIZE == 4 {
        lua_pushvector_lua_state_f32_f32_f32_f32(L, 0.0, 0.0, 0.0, 0.0);
        lua_setfield(L, -2, c"zero".as_ptr());
        lua_pushvector_lua_state_f32_f32_f32_f32(L, 1.0, 1.0, 1.0, 1.0);
        lua_setfield(L, -2, c"one".as_ptr());
    } else {
        lua_pushvector_lua_state_f32_f32_f32(L, 0.0, 0.0, 0.0);
        lua_setfield(L, -2, c"zero".as_ptr());
        lua_pushvector_lua_state_f32_f32_f32(L, 1.0, 1.0, 1.0);
        lua_setfield(L, -2, c"one".as_ptr());
    }

    createmetatable(L);

    1
}
