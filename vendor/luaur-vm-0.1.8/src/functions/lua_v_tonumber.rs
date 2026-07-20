use crate::functions::lua_o_str_2_d::lua_o_str_2_d;
use crate::macros::setnvalue::setnvalue;
use crate::macros::svalue::svalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::macros::ttisstring::ttisstring;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_v_tonumber(obj: *const TValue, n: *mut TValue) -> *const TValue {
    if ttisnumber!(obj) {
        return obj;
    }

    if ttisstring!(obj) {
        let mut num_out: f64 = 0.0;
        let p = svalue!(obj);
        if lua_o_str_2_d(p, &mut num_out) != 0 {
            setnvalue!(n, num_out);
            return n;
        }
    }

    core::ptr::null()
}
