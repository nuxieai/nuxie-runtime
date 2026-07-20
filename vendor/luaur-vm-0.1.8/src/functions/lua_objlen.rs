use core::ffi::c_int;

use crate::enums::lua_type::lua_Type;
use crate::functions::index_2_addr::index_2_addr;
use crate::functions::lua_h_getn::lua_h_getn;
use crate::macros::bufvalue::bufvalue;
use crate::macros::hvalue::hvalue;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttype::ttype;
use crate::macros::uvalue::uvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_objlen(L: *mut lua_State, idx: c_int) -> c_int {
    let o: StkId = index_2_addr(L, idx);
    let tt = ttype!(o);

    if tt == lua_Type::LUA_TSTRING as i32 {
        (*tsvalue!(o)).len as i32
    } else if tt == lua_Type::LUA_TUSERDATA as i32 {
        (*uvalue!(o)).len as i32
    } else if tt == lua_Type::LUA_TBUFFER as i32 {
        (*bufvalue!(o)).len as i32
    } else if tt == lua_Type::LUA_TTABLE as i32 {
        // The current stub for lua_h_getn takes 0 arguments and returns ().
        // However, the C++ source calls it with a LuaTable pointer and expects an int.
        // We must cast the function to the correct signature to match the C++ logic.
        let luaH_getn_ptr = lua_h_getn as *const ();
        let luaH_getn_real = core::mem::transmute::<
            *const (),
            unsafe fn(*mut crate::records::lua_table::LuaTable) -> c_int,
        >(luaH_getn_ptr);

        luaH_getn_real(hvalue!(o))
    } else {
        0
    }
}
