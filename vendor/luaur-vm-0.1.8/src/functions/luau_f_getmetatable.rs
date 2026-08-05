use crate::functions::lua_h_getstr::lua_h_getstr;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::sethvalue::sethvalue;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::setobj_2_s::setobj_2_s;
use crate::macros::ttisnil::ttisnil;
use crate::macros::ttistable::ttistable;
use crate::macros::ttisuserdata::ttisuserdata;
use crate::macros::ttype::ttype;
use crate::macros::uvalue::uvalue;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_getmetatable(
    L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 {
        let mut mt: *mut LuaTable = core::ptr::null_mut();
        if ttistable!(arg0) {
            mt = (*hvalue!(arg0)).metatable;
        } else if ttisuserdata!(arg0) {
            mt = (*uvalue!(arg0)).metatable;
        } else {
            mt = (*(*L).global).mt[ttype!(arg0) as usize];
        }

        let mtv = if !mt.is_null() {
            // TM_METATABLE is index 1 in the tag method names array (TM_METATABLE = 1)
            let key = (*(*L).global).tmname[1];
            lua_h_getstr(mt, key)
        } else {
            luaO_nilobject
        };

        if !ttisnil!(mtv) {
            setobj_2_s!(L, res, mtv);
            return 1;
        }

        if !mt.is_null() {
            sethvalue!(L, res, mt);
            return 1;
        } else {
            setnilvalue!(res);
            return 1;
        }
    }

    -1
}
