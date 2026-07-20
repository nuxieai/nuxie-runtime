use crate::functions::lua_pushinteger::lua_pushinteger;
use crate::functions::lua_replace::lua_replace;
use crate::functions::lua_tolstring::lua_tolstring;
use crate::functions::prepstate::prepstate;
use crate::functions::push_captures::push_captures;
use crate::functions::r#match::match_item;
use crate::functions::reprepstate::reprepstate;
use crate::macros::lua_tointeger::lua_tointeger;
use crate::macros::lua_upvalueindex::lua_upvalueindex;
use crate::records::match_state::MatchState;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

pub unsafe fn gmatch_aux(L: *mut lua_State) -> c_int {
    let mut ms = MatchState::default();
    let mut ls: usize = 0;
    let mut lp: usize = 0;
    let s = lua_tolstring(L, lua_upvalueindex(1), &mut ls);
    let p = lua_tolstring(L, lua_upvalueindex(2), &mut lp);

    prepstate(&mut ms, L, s, ls, p, lp);

    let mut src = s.add(lua_tointeger!(L, lua_upvalueindex(3)) as usize);
    while src <= ms.src_end {
        reprepstate(&mut ms);
        let e = match_item(&mut ms, src, p);
        if !e.is_null() {
            let mut newstart = e.offset_from(s) as c_int;
            if e == src {
                newstart += 1;
            }
            lua_pushinteger(L, newstart);
            lua_replace(L, lua_upvalueindex(3));
            return push_captures(&mut ms, src, e);
        }
        src = src.add(1);
    }

    0
}
