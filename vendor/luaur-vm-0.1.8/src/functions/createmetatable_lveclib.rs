use crate::functions::lua_createtable::lua_createtable;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::lua_pushvector_lapi::lua_pushvector_lua_state_f32_f32_f32_f32;
use crate::functions::lua_pushvector_lapi_alt_b::lua_pushvector_lua_state_f32_f32_f32;
use crate::functions::lua_setfield::lua_setfield;
use crate::functions::lua_setmetatable::lua_setmetatable;
use crate::functions::lua_setreadonly::lua_setreadonly;
use crate::functions::vector_index::vector_index;
use crate::macros::lua_pop::lua_pop;
use crate::macros::lua_pushcfunction::LUA_PUSHCFUNCTION;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn createmetatable(l: *mut lua_State) {
    lua_createtable(l, 0, 1); // create metatable for vectors

    // push dummy vector
    if LUA_VECTOR_SIZE == 4 {
        lua_pushvector_lua_state_f32_f32_f32_f32(l, 0.0, 0.0, 0.0, 0.0);
    } else {
        lua_pushvector_lua_state_f32_f32_f32(l, 0.0, 0.0, 0.0);
    }

    lua_pushvalue(l, -2);

    lua_setmetatable(l, -2); // set vector metatable
    lua_pop(l, 1); // pop dummy vector

    LUA_PUSHCFUNCTION(l, Some(vector_index), core::ptr::null());

    lua_setfield(l, -2, c"__index".as_ptr());

    lua_setreadonly(l, -1, 1); // true is 1 in C API

    lua_pop(l, 1); // pop the metatable
}
