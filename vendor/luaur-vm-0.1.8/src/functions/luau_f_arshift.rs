use crate::enums::lua_type::lua_Type;
use crate::macros::luai_num_2_unsigned::luai_num2unsigned;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_arshift(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttisnumber!(arg0) && ttisnumber!(args) {
        let a1 = nvalue!(arg0);
        let a2 = nvalue!(args);

        let mut u: u32 = 0;
        luai_num2unsigned(&mut u, a1);
        let s = a2 as i32;

        // note: we only specialize fast-path that doesn't require further conditionals (negative shifts and shifts greater or equal to bit width can
        // be handled generically)
        if (s as u32) < 32 {
            // note: technically right shift of negative values is UB, but this behavior is getting defined in C++20 and all compilers do the right
            // (shift) thing.
            // In Rust, arithmetic right shift is performed on signed integers.
            let r = (u as i32) >> s;

            setnvalue!(res, r as f64);
            return 1;
        }
    }

    -1
}
