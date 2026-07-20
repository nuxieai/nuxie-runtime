use crate::functions::index_2_addr::index_2_addr;
use crate::functions::lua_h_clone::lua_h_clone;
use crate::macros::api_check::api_check;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::hvalue::hvalue;
use crate::macros::sethvalue::sethvalue;
use crate::macros::ttistable::ttistable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_clonetable(L: *mut lua_State, idx: core::ffi::c_int) {
    // The C++ source:
    // StkId t = index2addr(L, idx);
    // api_check(L, ttistable(t));
    // LuaTable* tt = luaH_clone(L, hvalue(t));
    // sethvalue(L, L->top, tt);
    // api_incr_top(L);

    // We must cast the stubbed function pointers to their real signatures to call them.
    // Rust does not allow direct casting from fn item to fn pointer with different signature,
    // so we cast through a usize.
    let index2addr_fn: unsafe fn(*mut lua_State, core::ffi::c_int) -> StkId =
        core::mem::transmute(index_2_addr as *const core::ffi::c_void);
    let t: StkId = index2addr_fn(L, idx);

    api_check!(L, ttistable!(t));

    let lua_h_clone_fn: unsafe fn(*mut lua_State, *mut LuaTable) -> *mut LuaTable =
        core::mem::transmute(lua_h_clone as *const core::ffi::c_void);
    let tt: *mut LuaTable = lua_h_clone_fn(L, hvalue!(t));

    sethvalue!(L, (*L).top, tt);
    api_incr_top!(L);
}
