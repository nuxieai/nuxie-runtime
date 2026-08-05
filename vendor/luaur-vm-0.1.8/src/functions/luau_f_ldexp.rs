use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn luau_f_ldexp(
    _l: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: c_int,
    args: StkId,
    nparams: c_int,
) -> c_int {
    if nparams >= 2 && nresults <= 1 && ttisnumber!(arg0) && ttisnumber!(args) {
        let a1 = nvalue!(arg0);
        let a2 = nvalue!(args);
        setnvalue!(res, ldexp(a1, a2 as i32));
        1
    } else {
        -1
    }
}

fn ldexp(mut value: f64, mut exponent: i32) -> f64 {
    const MAX_EXPONENT: i32 = 1023;
    const MIN_EXPONENT: i32 = -1022;
    const MAX_SCALE: f64 = f64::from_bits(0x7fe0_0000_0000_0000);
    const MIN_NORMAL: f64 = f64::from_bits(0x0010_0000_0000_0000);
    const NORMALIZE_SUBNORMAL: f64 = f64::from_bits(0x4340_0000_0000_0000);

    if exponent > MAX_EXPONENT {
        value *= MAX_SCALE;
        exponent -= MAX_EXPONENT;
        if exponent > MAX_EXPONENT {
            value *= MAX_SCALE;
            exponent -= MAX_EXPONENT;
            if exponent > MAX_EXPONENT {
                exponent = MAX_EXPONENT;
            }
        }
    } else if exponent < MIN_EXPONENT {
        let scale = MIN_NORMAL * NORMALIZE_SUBNORMAL;
        let adjustment = -MIN_EXPONENT - 53;

        value *= scale;
        exponent += adjustment;
        if exponent < MIN_EXPONENT {
            value *= scale;
            exponent += adjustment;
            if exponent < MIN_EXPONENT {
                exponent = MIN_EXPONENT;
            }
        }
    }

    let scale = f64::from_bits(((1023 + exponent) as u64) << 52);
    value * scale
}
