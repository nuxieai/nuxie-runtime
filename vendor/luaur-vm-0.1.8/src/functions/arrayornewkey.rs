use crate::functions::newkey::newkey;
use crate::macros::cast_num::cast_num;
use crate::macros::luai_num_2_int::luai_num2int;
use crate::macros::luai_numeq::luai_numeq;
use crate::macros::nvalue::nvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn arrayornewkey(
    l: *mut lua_State,
    t: *mut LuaTable,
    key: *const TValue,
) -> *mut TValue {
    if ttisnumber!(key) {
        let mut k: core::ffi::c_int = 0;
        let n = nvalue!(key);
        luai_num2int!(k, n);

        if luai_numeq(cast_num!(k), n)
            && (k as core::ffi::c_uint).wrapping_sub(1) < (*t).sizearray as core::ffi::c_uint
        {
            return (*t).array.add((k - 1) as usize);
        }
    }

    newkey(l, t, key)
}
