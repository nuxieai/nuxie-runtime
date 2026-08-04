use crate::macros::lua_m_freegcofixed::luaM_freegcofixed;
use crate::records::lua_page::lua_Page;
use crate::records::luau_vector::LuauVector;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn luaVec_freevector(l: *mut lua_State, v: *mut LuauVector, page: *mut lua_Page) {
    luaM_freegcofixed!(
        l,
        v,
        core::mem::size_of::<LuauVector>(),
        (*v).gch.memcat,
        page
    );
}

#[allow(unused_imports)]
pub use luaVec_freevector as lua_vec_freevector;
