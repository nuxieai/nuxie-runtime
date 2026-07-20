use luaur_common::macros::luau_big_endian::LUAU_BIG_ENDIAN;

use crate::macros::bufvalue::bufvalue;
use crate::macros::checkoutofbounds::checkoutofbounds;
use crate::macros::luai_num_2_int::luai_num2int;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisbuffer::ttisbuffer;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luauF_readinteger<T>(
    _L: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int
where
    T: Copy + Into<f64>,
{
    if !LUAU_BIG_ENDIAN && nparams >= 2 && nresults <= 1 && ttisbuffer!(arg0) && ttisnumber!(args) {
        let mut offset: core::ffi::c_int = 0;
        luai_num2int!(offset, nvalue!(args));

        let len = (*bufvalue!(arg0)).len as usize;
        if checkoutofbounds(offset, len, core::mem::size_of::<T>()) {
            return -1;
        }

        let val: T = {
            let src = (*bufvalue!(arg0)).data.as_ptr().add(offset as usize) as *const T;
            core::ptr::read_unaligned(src)
        };

        setnvalue!(res, val.into());
        return 1;
    }

    -1
}
