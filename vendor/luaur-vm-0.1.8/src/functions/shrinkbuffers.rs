use crate::functions::stringresizeprotected::stringresizeprotected;
use crate::macros::lua_minstrtabsize::LUA_MINSTRTABSIZE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn shrinkbuffers(l: *mut lua_State) {
    let g = (*l).global;
    if (*g).strt.nuse < ((*g).strt.size / 4) as u32 && (*g).strt.size > LUA_MINSTRTABSIZE * 2 {
        stringresizeprotected(l, (*g).strt.size / 2);
    }
}
