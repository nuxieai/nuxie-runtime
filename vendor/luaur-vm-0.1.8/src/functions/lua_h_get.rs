use crate::enums::lua_type::lua_Type;
use crate::functions::lua_h_getnum::lua_h_getnum;
use crate::functions::lua_h_getstr::lua_h_getstr;
use crate::functions::lua_o_rawequal_key::luaO_rawequalKey;
use crate::functions::mainposition::mainposition;
use crate::macros::cast_num::cast_num;
use crate::macros::gkey::gkey;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::luai_num_2_int::luai_num2int;
use crate::macros::luai_numeq::luai_numeq;
use crate::macros::nvalue::nvalue;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttype::ttype;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_fallthrough::LUAU_FALLTHROUGH;

#[allow(non_snake_case)]
pub unsafe fn lua_h_get(t: *mut LuaTable, key: *const TValue) -> *const TValue {
    let tt = ttype!(key);
    match tt {
        0 => luaO_nilobject,
        6 => lua_h_getstr(t, tsvalue!(key) as *mut _),
        3 => {
            let mut k: core::ffi::c_int = 0;
            let n = nvalue!(key);
            luai_num2int!(k, n);
            if luai_numeq(cast_num!(k), nvalue!(key)) {
                return lua_h_getnum(t, k);
            }
            let mut n = mainposition(t, key);
            loop {
                if luaO_rawequalKey(gkey!(n), key) != 0 {
                    return crate::gval!(n);
                }
                let next = (*n).key.next();
                if next == 0 {
                    break;
                }
                n = n.offset(next as isize);
            }
            luaO_nilobject
        }
        _ => {
            let mut n = mainposition(t, key);
            loop {
                if luaO_rawequalKey(gkey!(n), key) != 0 {
                    return crate::gval!(n);
                }
                let next = (*n).key.next();
                if next == 0 {
                    break;
                }
                n = n.offset(next as isize);
            }
            luaO_nilobject
        }
    }
}

pub use lua_h_get as luaH_get;
