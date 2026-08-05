use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn luau_f_frexp(
    _L: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: c_int,
    _args: StkId,
    nparams: c_int,
) -> c_int {
    if nparams >= 1 && nresults <= 2 && ttisnumber!(arg0) {
        let a1 = nvalue!(arg0);
        let (f, e) = frexp(a1);
        setnvalue!(res, f);
        setnvalue!(res.add(1), e as f64);
        2
    } else {
        -1
    }
}

fn frexp(value: f64) -> (f64, c_int) {
    let mut bits = value.to_bits();
    let biased_exponent = ((bits >> 52) & 0x7ff) as c_int;

    if biased_exponent == 0 {
        if value != 0.0 {
            let (fraction, exponent) = frexp(value * f64::from_bits(0x43f0000000000000));
            return (fraction, exponent - 64);
        }
        return (value, 0);
    }
    if biased_exponent == 0x7ff {
        return (value, 0);
    }

    let exponent = biased_exponent - 0x3fe;
    bits &= 0x800f_ffff_ffff_ffff;
    bits |= 0x3fe0_0000_0000_0000;
    (f64::from_bits(bits), exponent)
}
