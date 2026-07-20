use crate::enums::lua_type::lua_Type;
use crate::functions::lua_h_getstr::lua_h_getstr;
use crate::macros::classvalue::classvalue;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::objectvalue::objectvalue;
use crate::macros::ttype::ttype;
use crate::macros::uvalue::uvalue;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_t_gettmbyobj(
    L: *mut crate::type_aliases::lua_state::lua_State,
    o: *const TValue,
    event: crate::type_aliases::tms::TMS,
) -> *const TValue {
    /*
      NB: Tag-methods were replaced by meta-methods in Lua 5.0, but the
      old names are still around (this function, for example).
    */
    let mt: *mut crate::records::lua_table::LuaTable;

    match ttype!(o) as i32 {
        t if t == lua_Type::LUA_TTABLE as i32 => {
            mt = (*hvalue!(o)).metatable;
        }
        t if t == lua_Type::LUA_TUSERDATA as i32 => {
            mt = (*uvalue!(o)).metatable;
        }
        t if t == lua_Type::LUA_TCLASS as i32 => {
            // We store a metatable for class objects on the
            // class object itself, use that.
            mt = (*classvalue!(o)).metatable;
        }
        t if t == lua_Type::LUA_TOBJECT as i32 => {
            mt = (*(*objectvalue!(o)).lclass).instancemetatable;
        }
        _ => {
            mt = (*(*L).global).mt[ttype!(o) as usize];
        }
    }

    if !mt.is_null() {
        lua_h_getstr(
            mt,
            (*(*L).global).tmname[event as usize] as *mut crate::records::t_string::TString,
        )
    } else {
        luaO_nilobject
    }
}
