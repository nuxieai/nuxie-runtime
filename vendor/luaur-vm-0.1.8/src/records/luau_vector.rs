use crate::records::g_cheader::GCheader;
use crate::type_aliases::lua_vector_type::LuaVectorType;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LuauVector {
    pub gch: GCheader,
    pub v: [LuaVectorType; crate::macros::lua_vector_size::LUA_VECTOR_SIZE as usize],
}
