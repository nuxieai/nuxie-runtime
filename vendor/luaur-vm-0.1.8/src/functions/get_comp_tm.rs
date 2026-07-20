use crate::functions::lua_o_rawequal_obj::luaO_rawequalObj;
use crate::macros::fasttm::fasttm;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::t_value::TValue;
use crate::type_aliases::tms::TMS;

pub fn get_comp_tm(
    L: *mut LuaState,
    mt1: *mut LuaTable,
    mt2: *mut LuaTable,
    event: TMS,
) -> *const TValue {
    unsafe {
        let tm1 = fasttm(L, mt1, event as i32);
        let tm2;

        if tm1.is_null() {
            return core::ptr::null();
        }

        if mt1 == mt2 {
            return tm1;
        }

        tm2 = fasttm(L, mt2, event as i32);
        if tm2.is_null() {
            return core::ptr::null();
        }

        if luaO_rawequalObj(tm1, tm2) != 0 {
            return tm1;
        }

        core::ptr::null()
    }
}
