use crate::functions::stringresizeprotected::stringresizeprotected;
use crate::macros::lua_minstrtabsize::LUA_MINSTRTABSIZE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn shrinkbuffersfull(l: *mut lua_State) {
    let g = (*l).global;
    let mut hashsize = (*g).strt.size;

    while (*g).strt.nuse < (hashsize / 4) as u32 && hashsize > LUA_MINSTRTABSIZE * 2 {
        hashsize /= 2;
    }

    if hashsize != (*g).strt.size {
        stringresizeprotected(l, hashsize);
    }
}
