use crate::enums::lua_type::lua_Type;
use crate::functions::hashint::hashint;
use crate::functions::hashnum::hashnum;
use crate::functions::hashpointer::hashpointer;
use crate::functions::hashvec::hashvec;
use crate::macros::bvalue::bvalue;
use crate::macros::gcvalue::gcvalue;
use crate::macros::lvalue::lvalue;
use crate::macros::nvalue::nvalue;
use crate::macros::pvalue::pvalue;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttype::ttype;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_node::LuaNode;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn mainposition(t: *const LuaTable, key: *const TValue) -> *mut LuaNode {
    match ttype!(key) as i32 {
        x if x == lua_Type::LUA_TNUMBER as i32 => hashnum(t as *mut LuaTable, nvalue!(key)),
        x if x == lua_Type::LUA_TINTEGER as i32 => hashint(t as *const LuaTable, lvalue!(key)),
        x if x == lua_Type::LUA_TVECTOR as i32 => {
            hashvec(t as *const LuaTable, vvalue!(key).as_ptr())
        }
        x if x == lua_Type::LUA_TSTRING as i32 => unsafe {
            crate::macros::hashstr::hashstr!(t as *const LuaTable, tsvalue!(key))
        },
        x if x == lua_Type::LUA_TBOOLEAN as i32 => unsafe {
            crate::macros::hashboolean::hashboolean!(t as *const LuaTable, bvalue!(key))
        },
        x if x == lua_Type::LUA_TLIGHTUSERDATA as i32 => {
            hashpointer(t as *const LuaTable, pvalue!(key))
        }
        _ => hashpointer(
            t as *const LuaTable,
            gcvalue!(key) as *const core::ffi::c_void,
        ),
    }
}
