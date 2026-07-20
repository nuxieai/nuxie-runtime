use crate::enums::lua_type::lua_Type;
use crate::functions::lua_m_newgco::luaM_newgco_;
use crate::functions::lua_m_toobig::lua_m_toobig;
use crate::macros::lua_c_init::luaC_init;
use crate::macros::max_buffer_size::MAX_BUFFER_SIZE;
use crate::macros::sizebuffer::sizebuffer;
use crate::type_aliases::buffer::Buffer;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_b_newbuffer(L: *mut lua_State, s: usize) -> *mut Buffer {
    if s > MAX_BUFFER_SIZE as usize {
        lua_m_toobig(L);
    }

    let b = luaM_newgco_(L, sizebuffer(s), (*L).activememcat) as *mut Buffer;
    luaC_init!(L, b, lua_Type::LUA_TBUFFER as i32);
    (*b).len = s as u32;
    core::ptr::write_bytes((*b).data.as_mut_ptr(), 0, (*b).len as usize);
    b
}
