use crate::macros::getstr::getstr;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::macros::ttisstring::ttisstring;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luauF_byte(
    _l: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && ttisstring!(arg0) && ttisnumber!(args) {
        let ts = tsvalue!(arg0);
        let i = nvalue!(args) as i32;
        let j = if nparams >= 3 {
            if ttisnumber!(args.add(1)) {
                nvalue!(args.add(1)) as i32
            } else {
                i
            }
        } else {
            i
        };

        if i >= 1 && j >= i && j <= (*ts).len as i32 {
            let c = j - i + 1;
            let s = getstr(ts);

            if c == (if nresults < 0 { 1 } else { nresults }) {
                for k in 0..c {
                    setnvalue!(
                        res.add(k as usize),
                        (*s.add((i + k - 1) as usize)) as u8 as f64
                    );
                }

                return c;
            }
        }
    }

    -1
}
