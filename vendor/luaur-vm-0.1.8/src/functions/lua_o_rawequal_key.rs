use crate::enums::lua_type::lua_Type;
use crate::functions::luai_veceq::luai_veceq;
use crate::macros::bvalue::bvalue;
use crate::macros::gcvalue::gcvalue;
use crate::macros::iscollectable::iscollectable;
use crate::macros::lightuserdatatag::lightuserdatatag;
use crate::macros::luai_inteq::luai_inteq;
use crate::macros::luai_numeq::luai_numeq;
use crate::macros::lvalue::lvalue;
use crate::macros::nvalue::nvalue;
use crate::macros::ttype::ttype;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::t_key::TKey;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn luaO_rawequalKey(t1: *const TKey, t2: *const TValue) -> core::ffi::c_int {
    if ttype!(t1) != ttype!(t2) {
        0
    } else {
        match ttype!(t1) {
            t if t == lua_Type::LUA_TNIL as i32 => 1,
            t if t == lua_Type::LUA_TNUMBER as i32 => luai_numeq(nvalue!(t1), nvalue!(t2)) as i32,
            t if t == lua_Type::LUA_TINTEGER as i32 => {
                luai_inteq(lvalue!(t1) as f64, lvalue!(t2) as f64) as i32
            }
            t if t == lua_Type::LUA_TVECTOR as i32 => {
                luai_veceq(vvalue!(t1).as_ptr(), vvalue!(t2).as_ptr()) as i32
            }
            t if t == lua_Type::LUA_TBOOLEAN as i32 => (bvalue!(t1) == bvalue!(t2)) as i32,
            t if t == lua_Type::LUA_TLIGHTUSERDATA as i32 => {
                ((*t1).value.p == (*t2).value.p && lightuserdatatag!(t1) == lightuserdatatag!(t2))
                    as i32
            }
            _ => {
                LUAU_ASSERT!(iscollectable!(t1));
                (gcvalue!(t1) == gcvalue!(t2)) as i32
            }
        }
    }
}
