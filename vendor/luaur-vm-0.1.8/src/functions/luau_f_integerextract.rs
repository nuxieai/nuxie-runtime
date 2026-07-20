use crate::enums::lua_type::lua_Type;
use crate::macros::lvalue::lvalue;
use crate::macros::setlvalue::setlvalue;
use crate::macros::ttisinteger::ttisinteger;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_integerextract(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 3 && !ttisinteger!(args.offset(1)) {
        return -1;
    }

    if nparams >= 2 && nresults <= 1 && ttisinteger!(arg0) && ttisinteger!(args) {
        let n: i64 = lvalue!(arg0);
        let f: i64 = lvalue!(args);
        let w: i64 = if nparams >= 3 {
            lvalue!(args.offset(1))
        } else {
            1
        };

        if f < 0 || f > 63 || w < 1 || w > 64 || (f + w) > 64 {
            return -1;
        }

        // C++: (((uint64_t)n) >> f) & ((0xFFFFFFFFFFFFFFFFULL) >> (64 - w))
        // Note: In C++, 0xFF... >> 64 is undefined behavior if w=0, but w is at least 1 here.
        // If w=64, 64-w=0, mask is 0xFF...
        let mask: u64 = if w >= 64 {
            0xFFFFFFFFFFFFFFFFu64
        } else {
            0xFFFFFFFFFFFFFFFFu64 >> (64 - w)
        };

        let val: u64 = (n as u64).wrapping_shr(f as u32) & mask;

        setlvalue!(res, val as i64);
        return 1;
    }

    -1
}
