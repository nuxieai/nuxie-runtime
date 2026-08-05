use crate::enums::lua_type::lua_Type;
use crate::macros::lua_c_init::luaC_init;
use crate::macros::lua_m_newgcofixed::luaM_newgcofixed;
use crate::records::luau_vector::LuauVector;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_vector_type::LuaVectorType;

#[allow(non_snake_case)]
pub unsafe fn luaVec_newvector(
    l: *mut lua_State,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
) -> *mut LuauVector {
    let v = luaM_newgcofixed!(
        l,
        LuauVector,
        core::mem::size_of::<LuauVector>(),
        (*l).activememcat
    );
    luaC_init!(l, v, lua_Type::LUA_TVECTOR as i32);
    (*v).v[0] = x as LuaVectorType;
    (*v).v[1] = y as LuaVectorType;
    (*v).v[2] = z as LuaVectorType;
    if crate::macros::lua_vector_size::LUA_VECTOR_SIZE == 4 {
        (*v).v[3] = w as LuaVectorType;
    } else {
        let _ = w;
    }
    v
}

#[allow(unused_imports)]
pub use luaVec_newvector as lua_vec_newvector;
