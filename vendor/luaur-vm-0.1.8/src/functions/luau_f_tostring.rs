use crate::enums::lua_type::lua_Type;
use crate::functions::lua_s_newlstr::luaS_newlstr;
use crate::functions::luai_int_2_str::luai_int2str;
use crate::functions::luai_num_2_str::luai_num2str;
use crate::macros::bvalue::bvalue;
use crate::macros::lua_c_needs_gc::luaC_needsGC;
use crate::macros::lua_s_newliteral::luaS_newliteral;
use crate::macros::luai_maxint_2_str::LUAI_MAXINT2STR;
use crate::macros::luai_maxnum_2_str::LUAI_MAXNUM2STR;
use crate::macros::lvalue::lvalue;
use crate::macros::nvalue::nvalue;
use crate::macros::setsvalue::setsvalue;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttype::ttype;
use crate::records::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_tostring(
    l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 {
        match ttype!(arg0) {
            t if t == lua_Type::LUA_TNIL as i32 => {
                let s = (*(*l).global).ttname[lua_Type::LUA_TNIL as usize];
                setsvalue!(l, res, s);
                return 1;
            }
            t if t == lua_Type::LUA_TBOOLEAN as i32 => {
                // bvalue returns i32 (0 or 1) in Luau VM; compare to 0 for boolean check
                let s = if bvalue!(arg0) != 0 {
                    luaS_newliteral(l, c"true".as_ptr())
                } else {
                    luaS_newliteral(l, c"false".as_ptr())
                };
                setsvalue!(l, res, s);
                return 1;
            }
            t if t == lua_Type::LUA_TNUMBER as i32 => {
                if luaC_needsGC!(l) {
                    return -1;
                }
                let mut s = [0 as core::ffi::c_char; LUAI_MAXNUM2STR as usize];
                let e = luai_num2str(s.as_mut_ptr(), nvalue!(arg0));
                setsvalue!(
                    l,
                    res,
                    luaS_newlstr(l, s.as_ptr(), e.offset_from(s.as_ptr()) as usize)
                );
                return 1;
            }
            t if t == lua_Type::LUA_TSTRING as i32 => {
                setsvalue!(l, res, tsvalue!(arg0));
                return 1;
            }
            t if t == lua_Type::LUA_TINTEGER as i32 => {
                if luaC_needsGC!(l) {
                    return -1;
                }
                let mut s = [0 as core::ffi::c_char; LUAI_MAXINT2STR as usize];
                let e = luai_int2str(s.as_mut_ptr(), lvalue!(arg0));
                setsvalue!(
                    l,
                    res,
                    luaS_newlstr(l, s.as_ptr(), e.offset_from(s.as_ptr()) as usize)
                );
                return 1;
            }
            _ => {}
        }
    }
    -1
}
