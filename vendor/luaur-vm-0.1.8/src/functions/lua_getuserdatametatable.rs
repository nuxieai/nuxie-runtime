use crate::enums::lua_type::lua_Type;
use crate::functions::lua_c_barrierback::lua_c_barrierback;
use crate::macros::api_check::api_check;
use crate::macros::blackbit::BLACKBIT;
use crate::macros::lua_utag_limit::LUA_UTAG_LIMIT;
use crate::records::gc_object::GCObject;
use crate::records::lua_state::lua_State;

#[allow(non_snake_case)]
pub fn lua_getuserdatametatable(L: *mut lua_State, tag: core::ffi::c_int) {
    api_check!(L, (tag as u32) < LUA_UTAG_LIMIT as u32);

    unsafe {
        let marked = (*L).hdr.marked;
        if (marked as i32 & (1 << BLACKBIT)) != 0 {
            lua_c_barrierback(L, L as *mut GCObject, &mut (*L).gclist);
        }

        let h = (*(*L).global).udatamt[tag as usize];
        if !h.is_null() {
            let i_o = (*L).top;
            (*i_o).value.gc = h as *mut GCObject;
            (*i_o).tt = lua_Type::LUA_TTABLE as core::ffi::c_int;
        } else {
            (*(*L).top).tt = lua_Type::LUA_TNIL as i32;
        }

        api_check!(L, (*L).top < (*(*L).ci).top);
        (*L).top = (*L).top.add(1);
    }
}
